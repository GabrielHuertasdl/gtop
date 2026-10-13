use std::collections::{HashMap, VecDeque};

use crate::process::model::{Pid, ProcessInfo};

const SPARKLINE_BLOCKS: &[char] = &[' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
const SPARKLINE_ASCII: &[char] = &[' ', '.', ':', '-', '=', '+', '*', '#', '@'];

pub struct ProcessHistory {
    pub cpu_samples: HashMap<Pid, VecDeque<f32>>,
    pub history_len: usize,
}

impl ProcessHistory {
    pub fn new(history_len: usize) -> Self {
        ProcessHistory {
            cpu_samples: HashMap::new(),
            history_len,
        }
    }

    pub fn update(&mut self, processes: &[ProcessInfo]) {
        let active: std::collections::HashSet<Pid> =
            processes.iter().map(|p| p.pid).collect();

        // Evict dead pids
        self.cpu_samples.retain(|pid, _| active.contains(pid));

        for proc in processes {
            let buf = self
                .cpu_samples
                .entry(proc.pid)
                .or_insert_with(|| VecDeque::with_capacity(self.history_len + 1));
            buf.push_back(proc.cpu_pct);
            while buf.len() > self.history_len {
                buf.pop_front();
            }
        }
    }

    /// Returns a string of `width` sparkline chars for the given pid.
    /// `ascii` = use ASCII fallback chars.
    pub fn sparkline(&self, pid: Pid, width: usize, ascii: bool) -> String {
        let chars = if ascii { SPARKLINE_ASCII } else { SPARKLINE_BLOCKS };

        let Some(samples) = self.cpu_samples.get(&pid) else {
            return " ".repeat(width);
        };

        if samples.is_empty() {
            return " ".repeat(width);
        }

        let available = samples.len().min(width);
        let padding = width.saturating_sub(available);

        // Take the last `available` samples
        let data: Vec<f32> = samples
            .iter()
            .copied()
            .rev()
            .take(available)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();

        // Normalize to max in window (min 1% so silent process shows blanks)
        let max = data.iter().copied().fold(1.0f32, f32::max);

        let bar: String = data
            .iter()
            .map(|&v| {
                let idx = ((v / max) * (chars.len() - 1) as f32)
                    .clamp(0.0, (chars.len() - 1) as f32) as usize;
                chars[idx]
            })
            .collect();

        format!("{}{}", " ".repeat(padding), bar)
    }

    /// Returns raw f32 samples for a pid (for graph rendering)
    pub fn samples(&self, pid: Pid) -> Vec<f32> {
        self.cpu_samples
            .get(&pid)
            .map(|d| d.iter().copied().collect())
            .unwrap_or_default()
    }
}

impl Clone for ProcessHistory {
    fn clone(&self) -> Self {
        ProcessHistory {
            cpu_samples: self.cpu_samples
                .iter()
                .map(|(&k, v)| (k, v.clone()))
                .collect(),
            history_len: self.history_len,
        }
    }
}
