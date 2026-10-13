use std::collections::HashMap;

use crate::process::model::{Pid, ProcessInfo};

#[derive(Debug, Clone, Default)]
pub struct ProcessTree {
    pub by_pid: HashMap<Pid, ProcessInfo>,
    pub roots: Vec<Pid>,
}

impl ProcessTree {
    pub fn build(mut processes: Vec<ProcessInfo>) -> Self {
        // Populate children lists
        let pids: Vec<Pid> = processes.iter().map(|p| p.pid).collect();
        let pid_set: std::collections::HashSet<Pid> = pids.iter().copied().collect();

        let mut children_map: HashMap<Pid, Vec<Pid>> = HashMap::new();
        for p in &processes {
            if pid_set.contains(&p.ppid) && p.ppid != p.pid {
                children_map.entry(p.ppid).or_default().push(p.pid);
            }
        }

        for p in &mut processes {
            if let Some(children) = children_map.remove(&p.pid) {
                p.children = children;
            }
        }

        let mut roots: Vec<Pid> = Vec::new();
        let mut by_pid: HashMap<Pid, ProcessInfo> = HashMap::new();

        for p in processes {
            if !pid_set.contains(&p.ppid) || p.ppid == p.pid {
                roots.push(p.pid);
            }
            by_pid.insert(p.pid, p);
        }

        roots.sort_unstable();

        ProcessTree { by_pid, roots }
    }

    /// DFS flattened list for tree view, returns (pid, depth)
    pub fn flatten_dfs(&self) -> Vec<(Pid, usize)> {
        let mut result = Vec::new();
        for &root in &self.roots {
            self.dfs(root, 0, &mut result);
        }
        result
    }

    fn dfs(&self, pid: Pid, depth: usize, out: &mut Vec<(Pid, usize)>) {
        out.push((pid, depth));
        if let Some(proc) = self.by_pid.get(&pid) {
            let mut children = proc.children.clone();
            children.sort_unstable();
            for child in children {
                self.dfs(child, depth + 1, out);
            }
        }
    }

    /// All PIDs in subtree rooted at pid, including pid itself
    pub fn subtree_pids(&self, root: Pid) -> Vec<Pid> {
        let mut result = Vec::new();
        let mut stack = vec![root];
        while let Some(pid) = stack.pop() {
            result.push(pid);
            if let Some(proc) = self.by_pid.get(&pid) {
                stack.extend_from_slice(&proc.children);
            }
        }
        result
    }
}
