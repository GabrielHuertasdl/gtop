use ringbuffer::{AllocRingBuffer, RingBuffer};

use crate::metrics::model::{SystemMetrics, HISTORY_LEN};

#[derive(Clone)]
pub struct MetricsHistory {
    pub cpu_global: AllocRingBuffer<f32>,
    pub cpu_per_core: Vec<AllocRingBuffer<f32>>,
    pub mem_used: AllocRingBuffer<u64>,
    pub swap_used: AllocRingBuffer<u64>,
    pub net_rx: std::collections::HashMap<String, AllocRingBuffer<u64>>,
    pub net_tx: std::collections::HashMap<String, AllocRingBuffer<u64>>,
    pub disk_read: std::collections::HashMap<String, AllocRingBuffer<u64>>,
    pub disk_write: std::collections::HashMap<String, AllocRingBuffer<u64>>,
    pub timestamps_ms: AllocRingBuffer<u64>,
    pub core_count: usize,
}

impl MetricsHistory {
    pub fn new(core_count: usize) -> Self {
        let cpu_per_core = (0..core_count)
            .map(|_| AllocRingBuffer::new(HISTORY_LEN))
            .collect();
        MetricsHistory {
            cpu_global: AllocRingBuffer::new(HISTORY_LEN),
            cpu_per_core,
            mem_used: AllocRingBuffer::new(HISTORY_LEN),
            swap_used: AllocRingBuffer::new(HISTORY_LEN),
            net_rx: std::collections::HashMap::new(),
            net_tx: std::collections::HashMap::new(),
            disk_read: std::collections::HashMap::new(),
            disk_write: std::collections::HashMap::new(),
            timestamps_ms: AllocRingBuffer::new(HISTORY_LEN),
            core_count,
        }
    }

    pub fn push(&mut self, metrics: &SystemMetrics) {
        self.cpu_global.push(metrics.cpu.global_pct);
        self.mem_used.push(metrics.memory.used);
        self.swap_used.push(metrics.memory.swap_used);
        self.timestamps_ms.push(metrics.timestamp_ms);

        // Grow per_core if core count increased
        while self.cpu_per_core.len() < metrics.cpu.per_core_pct.len() {
            self.cpu_per_core.push(AllocRingBuffer::new(HISTORY_LEN));
        }
        for (i, &pct) in metrics.cpu.per_core_pct.iter().enumerate() {
            if i < self.cpu_per_core.len() {
                self.cpu_per_core[i].push(pct);
            }
        }

        for net in &metrics.networks {
            self.net_rx
                .entry(net.interface.clone())
                .or_insert_with(|| AllocRingBuffer::new(HISTORY_LEN))
                .push(net.rx_bytes_per_sec);
            self.net_tx
                .entry(net.interface.clone())
                .or_insert_with(|| AllocRingBuffer::new(HISTORY_LEN))
                .push(net.tx_bytes_per_sec);
        }

        for disk in &metrics.disks {
            self.disk_read
                .entry(disk.name.clone())
                .or_insert_with(|| AllocRingBuffer::new(HISTORY_LEN))
                .push(disk.read_bytes_per_sec);
            self.disk_write
                .entry(disk.name.clone())
                .or_insert_with(|| AllocRingBuffer::new(HISTORY_LEN))
                .push(disk.write_bytes_per_sec);
        }
    }

    pub fn cpu_vec(&self) -> Vec<f64> {
        self.cpu_global.iter().map(|&v| v as f64).collect()
    }

    pub fn mem_used_vec(&self) -> Vec<u64> {
        self.mem_used.iter().copied().collect()
    }

    /// Returns a sparkline string of `width` chars from global CPU history.
    /// Normalized to 100% (absolute), not relative to max.
    pub fn cpu_sparkline(&self, width: usize, ascii: bool) -> String {
        const BLOCKS: [char; 9] = [' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
        const ASCHARS: [char; 9] = [' ', '.', ':', '-', '=', '+', '*', '#', '@'];
        let chars = if ascii { &ASCHARS } else { &BLOCKS };

        let data = self.cpu_vec();
        if data.is_empty() {
            return " ".repeat(width);
        }
        let available = data.len().min(width);
        let padding = width.saturating_sub(available);
        let slice = &data[data.len().saturating_sub(available)..];

        let bar: String = slice.iter().map(|&v| {
            let idx = ((v / 100.0) * 8.0).clamp(0.0, 8.0) as usize;
            chars[idx.min(8)]
        }).collect();

        format!("{}{}", " ".repeat(padding), bar)
    }

    /// Returns a sparkline string for a specific core's history.
    pub fn core_sparkline(&self, core_idx: usize, width: usize, ascii: bool) -> String {
        const BLOCKS: [char; 9] = [' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
        const ASCHARS: [char; 9] = [' ', '.', ':', '-', '=', '+', '*', '#', '@'];
        let chars = if ascii { &ASCHARS } else { &BLOCKS };

        let Some(buf) = self.cpu_per_core.get(core_idx) else {
            return " ".repeat(width);
        };
        let data: Vec<f32> = buf.iter().copied().collect();
        if data.is_empty() {
            return " ".repeat(width);
        }
        let available = data.len().min(width);
        let padding = width.saturating_sub(available);
        let slice = &data[data.len().saturating_sub(available)..];

        let bar: String = slice.iter().map(|&v| {
            let idx = ((v / 100.0) * 8.0).clamp(0.0, 8.0) as usize;
            chars[idx.min(8)]
        }).collect();

        format!("{}{}", " ".repeat(padding), bar)
    }
}
