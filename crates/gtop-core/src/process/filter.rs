use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};

use crate::process::model::{ProcessInfo, ProcessStatus};
use crate::process::tree::ProcessTree;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FilterConfig {
    pub by_user: Option<String>,
    pub min_cpu_pct: Option<f32>,
    pub min_mem_pct: Option<f32>,
    pub by_status: Option<ProcessStatus>,
    pub hide_kernel: bool,
    pub hide_threads: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum SortColumn {
    Pid,
    Name,
    User,
    #[default]
    CpuPct,
    MemPct,
    MemBytes,
    Priority,
    Nice,
    Status,
    Threads,
    IoRead,
    IoWrite,
}

impl SortColumn {
    pub fn label(&self) -> &'static str {
        match self {
            SortColumn::Pid => "PID",
            SortColumn::Name => "NAME",
            SortColumn::User => "USER",
            SortColumn::CpuPct => "CPU%",
            SortColumn::MemPct => "MEM%",
            SortColumn::MemBytes => "RSS",
            SortColumn::Priority => "PRI",
            SortColumn::Nice => "NI",
            SortColumn::Status => "STATUS",
            SortColumn::Threads => "THR",
            SortColumn::IoRead => "R/s",
            SortColumn::IoWrite => "W/s",
        }
    }
}

pub fn apply_filter_sort(
    tree: &ProcessTree,
    filter: &FilterConfig,
    search: &str,
    sort_col: &SortColumn,
    ascending: bool,
) -> Vec<ProcessInfo> {
    let matcher = SkimMatcherV2::default();

    let mut result: Vec<ProcessInfo> = tree
        .by_pid
        .values()
        .filter(|p| {
            if filter.hide_kernel && p.is_kernel {
                return false;
            }
            if let Some(ref user) = filter.by_user {
                if &p.username != user {
                    return false;
                }
            }
            if let Some(min_cpu) = filter.min_cpu_pct {
                if p.cpu_pct < min_cpu {
                    return false;
                }
            }
            if let Some(min_mem) = filter.min_mem_pct {
                if p.mem_pct < min_mem {
                    return false;
                }
            }
            if let Some(ref status) = filter.by_status {
                if &p.status != status {
                    return false;
                }
            }
            if !search.is_empty() {
                let name_match = matcher.fuzzy_match(&p.name, search).is_some();
                let cmd_match = matcher.fuzzy_match(&p.cmdline, search).is_some();
                let pid_match = p.pid.to_string().contains(search);
                if !name_match && !cmd_match && !pid_match {
                    return false;
                }
            }
            true
        })
        .cloned()
        .collect();

    result.sort_unstable_by(|a, b| {
        let ord = match sort_col {
            SortColumn::Pid => a.pid.cmp(&b.pid),
            SortColumn::Name => a.name.cmp(&b.name),
            SortColumn::User => a.username.cmp(&b.username),
            SortColumn::CpuPct => a
                .cpu_pct
                .partial_cmp(&b.cpu_pct)
                .unwrap_or(std::cmp::Ordering::Equal),
            SortColumn::MemPct => a
                .mem_pct
                .partial_cmp(&b.mem_pct)
                .unwrap_or(std::cmp::Ordering::Equal),
            SortColumn::MemBytes => a.mem_bytes.cmp(&b.mem_bytes),
            SortColumn::Priority => a.priority.cmp(&b.priority),
            SortColumn::Nice => a.nice.cmp(&b.nice),
            SortColumn::Status => a.status_char().cmp(&b.status_char()),
            SortColumn::Threads => a.threads.cmp(&b.threads),
            SortColumn::IoRead => a.io_read_bytes.cmp(&b.io_read_bytes),
            SortColumn::IoWrite => a.io_write_bytes.cmp(&b.io_write_bytes),
        };
        if ascending { ord } else { ord.reverse() }
    });

    result
}
