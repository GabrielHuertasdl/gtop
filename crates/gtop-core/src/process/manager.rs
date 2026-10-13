use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid as NixPid;

use crate::error::{Error, Pid, Result};
use crate::process::tree::ProcessTree;

pub struct ProcessManager;

impl ProcessManager {
    pub fn send_signal(pid: Pid, sig: Signal) -> Result<()> {
        kill(NixPid::from_raw(pid as i32), sig).map_err(Error::Nix)
    }

    pub fn kill(pid: Pid) -> Result<()> {
        Self::send_signal(pid, Signal::SIGKILL)
    }

    pub fn terminate(pid: Pid) -> Result<()> {
        Self::send_signal(pid, Signal::SIGTERM)
    }

    pub fn pause(pid: Pid) -> Result<()> {
        Self::send_signal(pid, Signal::SIGSTOP)
    }

    pub fn resume(pid: Pid) -> Result<()> {
        Self::send_signal(pid, Signal::SIGCONT)
    }

    pub fn renice(pid: Pid, nice: i32) -> Result<()> {
        let ret = unsafe {
            libc::setpriority(
                libc::PRIO_PROCESS,
                pid as libc::id_t,
                nice,
            )
        };
        if ret == 0 {
            Ok(())
        } else {
            Err(Error::Io(std::io::Error::last_os_error()))
        }
    }

    pub fn kill_tree(root: Pid, tree: &ProcessTree, sig: Signal) -> Vec<(Pid, Result<()>)> {
        let mut pids = tree.subtree_pids(root);
        pids.reverse();
        pids.iter().map(|&p| (p, Self::send_signal(p, sig))).collect()
    }

    pub fn can_signal(pid: Pid) -> bool {
        let my_uid = unsafe { libc::getuid() };
        std::fs::read_to_string(format!("/proc/{}/status", pid))
            .ok()
            .and_then(|s| {
                s.lines()
                    .find(|l| l.starts_with("Uid:"))
                    .and_then(|l| l.split_whitespace().nth(1))
                    .and_then(|v| v.parse::<u32>().ok())
            })
            .map(|uid| uid == my_uid || my_uid == 0)
            .unwrap_or(false)
    }

    pub fn verify_process(pid: Pid, expected_name: &str) -> bool {
        std::fs::read_to_string(format!("/proc/{}/comm", pid))
            .ok()
            .map(|s| s.trim() == expected_name)
            .unwrap_or(false)
    }
}

extern crate libc;
