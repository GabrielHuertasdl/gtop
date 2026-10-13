use std::collections::HashMap;
use std::time::Instant;

use crate::process::model::{Pid, ProcessInfo, ProcessStatus};
use crate::process::tree::ProcessTree;

struct ProcSnapshot {
    utime: u64,
    stime: u64,
    at: Instant,
}

pub struct ProcessCollector {
    prev: HashMap<Pid, ProcSnapshot>,
    uid_cache: HashMap<u32, String>,
    ticks_per_sec: u64,
    page_size: u64,
    mem_total: u64,
}

impl ProcessCollector {
    pub fn new() -> Self {
        let ticks_per_sec = unsafe { libc::sysconf(libc::_SC_CLK_TCK) as u64 };
        let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) as u64 };
        ProcessCollector {
            prev: HashMap::new(),
            uid_cache: HashMap::new(),
            ticks_per_sec: if ticks_per_sec == 0 { 100 } else { ticks_per_sec },
            page_size: if page_size == 0 { 4096 } else { page_size },
            mem_total: read_mem_total(),
        }
    }

    pub fn collect(&mut self) -> ProcessTree {
        self.mem_total = read_mem_total();

        let Ok(all) = procfs::process::all_processes() else {
            return ProcessTree::default();
        };

        let now = Instant::now();
        let mut processes = Vec::new();

        for proc_result in all {
            let Ok(proc) = proc_result else { continue };

            let Ok(stat) = proc.stat() else { continue };
            let pid = stat.pid as Pid;

            let utime = stat.utime;
            let stime = stat.stime;

            let cpu_pct = if let Some(prev) = self.prev.get(&pid) {
                let elapsed = now.duration_since(prev.at).as_secs_f64().max(0.001);
                let ticks_delta =
                    (utime + stime).saturating_sub(prev.utime + prev.stime) as f64;
                (ticks_delta / self.ticks_per_sec as f64 / elapsed * 100.0) as f32
            } else {
                0.0
            };

            self.prev.insert(
                pid,
                ProcSnapshot {
                    utime,
                    stime,
                    at: now,
                },
            );

            let mem_bytes = (stat.rss.max(0) as u64) * self.page_size;
            let mem_pct = if self.mem_total > 0 {
                mem_bytes as f32 / self.mem_total as f32 * 100.0
            } else {
                0.0
            };

            let cmdline = proc
                .cmdline()
                .ok()
                .map(|v| v.join(" "))
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| format!("[{}]", stat.comm));

            let uid = proc
                .status()
                .ok()
                .map(|s| s.ruid as u32)
                .unwrap_or(0);

            let username = self.resolve_uid(uid);

            let io = proc.io().ok();
            let io_read = io.as_ref().map(|i| i.read_bytes).unwrap_or(0);
            let io_write = io.as_ref().map(|i| i.write_bytes).unwrap_or(0);

            let is_kernel = stat.ppid == 0 || stat.comm.starts_with('[');

            processes.push(ProcessInfo {
                pid,
                ppid: stat.ppid.max(0) as Pid,
                name: stat.comm.clone(),
                cmdline,
                status: ProcessStatus::from(stat.state),
                cpu_pct: cpu_pct.clamp(0.0, 100.0 * self.ticks_per_sec as f32),
                mem_bytes,
                mem_pct: mem_pct.clamp(0.0, 100.0),
                threads: stat.num_threads.max(0) as u32,
                priority: stat.priority,
                nice: stat.nice,
                uid,
                username,
                starttime_ticks: stat.starttime,
                io_read_bytes: io_read,
                io_write_bytes: io_write,
                is_kernel,
                children: Vec::new(), // populated by ProcessTree::build
            });
        }

        // Clean up stale entries (processes that exited)
        let live: std::collections::HashSet<Pid> = processes.iter().map(|p| p.pid).collect();
        self.prev.retain(|pid, _| live.contains(pid));

        ProcessTree::build(processes)
    }

    fn resolve_uid(&mut self, uid: u32) -> String {
        if let Some(name) = self.uid_cache.get(&uid) {
            return name.clone();
        }
        let name = lookup_username(uid);
        self.uid_cache.insert(uid, name.clone());
        name
    }
}

impl Default for ProcessCollector {
    fn default() -> Self {
        Self::new()
    }
}

fn lookup_username(uid: u32) -> String {
    // Read /etc/passwd for uid lookup (no external deps)
    if let Ok(content) = std::fs::read_to_string("/etc/passwd") {
        for line in content.lines() {
            let fields: Vec<&str> = line.split(':').collect();
            if fields.len() >= 4 {
                if let Ok(file_uid) = fields[2].parse::<u32>() {
                    if file_uid == uid {
                        return fields[0].to_string();
                    }
                }
            }
        }
    }
    uid.to_string()
}

fn read_mem_total() -> u64 {
    std::fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("MemTotal:"))
                .and_then(|l| l.split_whitespace().nth(1))
                .and_then(|v| v.parse::<u64>().ok())
                .map(|kb| kb * 1024)
        })
        .unwrap_or(1)
}

// We need libc for sysconf
extern crate libc;
