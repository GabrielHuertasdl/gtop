use std::collections::HashSet;

use nix::sys::signal::Signal;

use crate::metrics::model::SystemMetrics;
use crate::metrics::MetricsHistory;
use crate::process::filter::{apply_filter_sort, FilterConfig, SortColumn};
use crate::process::history::ProcessHistory;
use crate::process::model::{Pid, ProcessInfo};
use crate::process::ProcessTree;

#[derive(Debug, Clone, PartialEq)]
pub enum ViewMode {
    List,
    Tree,
}

impl Default for ViewMode {
    fn default() -> Self {
        ViewMode::List
    }
}

#[derive(Debug, Clone)]
pub struct PendingAction {
    pub kind: ActionKind,
    pub pids: Vec<Pid>,
    pub names: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum ActionKind {
    Kill,
    Terminate,
    Pause,
    Resume,
    Renice(i32),
    Signal(Signal),
}

impl ActionKind {
    pub fn label(&self) -> String {
        match self {
            ActionKind::Kill => "KILL (SIGKILL)".to_string(),
            ActionKind::Terminate => "Terminate (SIGTERM)".to_string(),
            ActionKind::Pause => "Pause (SIGSTOP)".to_string(),
            ActionKind::Resume => "Resume (SIGCONT)".to_string(),
            ActionKind::Renice(v) => format!("Renice to {}", v),
            ActionKind::Signal(s) => format!("Send {}", s),
        }
    }

    pub fn is_destructive(&self) -> bool {
        matches!(self, ActionKind::Kill | ActionKind::Terminate)
    }
}

#[derive(Clone)]
pub struct AppState {
    // live data
    pub metrics: SystemMetrics,
    pub tree: ProcessTree,
    pub processes: Vec<ProcessInfo>, // filtered+sorted flat list

    // history
    pub history: MetricsHistory,
    pub proc_history: ProcessHistory,

    // view settings
    pub view_mode: ViewMode,
    pub sort_column: SortColumn,
    pub sort_ascending: bool,
    pub filter: FilterConfig,
    pub search: String,

    // interaction
    pub selected_pids: HashSet<Pid>,
    pub cursor_index: usize,
    pub focused_pid: Option<Pid>,
    pub detail_visible: bool,
    pub search_active: bool,

    // pending action awaiting confirmation
    pub pending_action: Option<PendingAction>,

    // renice input buffer
    pub renice_input: String,
}

impl AppState {
    pub fn new(core_count: usize) -> Self {
        AppState {
            metrics: SystemMetrics::default(),
            tree: ProcessTree::default(),
            processes: Vec::new(),
            history: MetricsHistory::new(core_count),
            proc_history: ProcessHistory::new(60),
            view_mode: ViewMode::List,
            sort_column: SortColumn::CpuPct,
            sort_ascending: false,
            filter: FilterConfig {
                hide_kernel: true,
                ..Default::default()
            },
            search: String::new(),
            selected_pids: HashSet::new(),
            cursor_index: 0,
            focused_pid: None,
            detail_visible: false,
            search_active: false,
            pending_action: None,
            renice_input: String::new(),
        }
    }

    pub fn update_metrics(&mut self, metrics: SystemMetrics) {
        self.history.push(&metrics);
        self.metrics = metrics;
    }

    pub fn update_processes(&mut self, tree: ProcessTree) {
        self.tree = tree;

        // Update per-process CPU history from ALL processes in tree
        let all: Vec<ProcessInfo> = self.tree.by_pid.values().cloned().collect();
        self.proc_history.update(&all);

        self.recompute_processes();

        // Update focused_pid from cursor
        self.focused_pid = self
            .processes
            .get(self.cursor_index)
            .map(|p| p.pid);
    }

    pub fn recompute_processes(&mut self) {
        self.processes = apply_filter_sort(
            &self.tree,
            &self.filter,
            &self.search,
            &self.sort_column,
            self.sort_ascending,
        );

        // Clamp cursor
        if !self.processes.is_empty() && self.cursor_index >= self.processes.len() {
            self.cursor_index = self.processes.len() - 1;
        }
    }

    pub fn cursor_process(&self) -> Option<&ProcessInfo> {
        self.processes.get(self.cursor_index)
    }

    /// Returns processes that are either multi-selected or (if none selected) the cursor process
    pub fn effective_targets(&self) -> Vec<&ProcessInfo> {
        if !self.selected_pids.is_empty() {
            self.processes
                .iter()
                .filter(|p| self.selected_pids.contains(&p.pid))
                .collect()
        } else {
            self.cursor_process().into_iter().collect()
        }
    }

    pub fn move_cursor(&mut self, delta: i64) {
        let len = self.processes.len();
        if len == 0 {
            return;
        }
        let new_idx = (self.cursor_index as i64 + delta)
            .clamp(0, (len - 1) as i64) as usize;
        self.cursor_index = new_idx;
        self.focused_pid = self.processes.get(self.cursor_index).map(|p| p.pid);
    }
}
