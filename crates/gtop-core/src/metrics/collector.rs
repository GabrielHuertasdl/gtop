use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::Result;
use crate::metrics::battery;
use crate::metrics::cpu::CpuCollector;
use crate::metrics::disk::DiskCollector;
use crate::metrics::gpu::{detect_gpu, GpuBackend};
use crate::metrics::history::MetricsHistory;
use crate::metrics::memory;
use crate::metrics::model::SystemMetrics;
use crate::metrics::network::NetworkCollector;
use crate::metrics::temperature;

pub struct MetricsCollector {
    pub cpu: CpuCollector,
    pub net: NetworkCollector,
    pub disk: DiskCollector,
    pub gpu: Box<dyn GpuBackend + Send + Sync>,
    pub history: MetricsHistory,
}

impl MetricsCollector {
    pub fn new() -> Result<Self> {
        let core_count = num_cpus();
        Ok(MetricsCollector {
            cpu: CpuCollector::default(),
            net: NetworkCollector::default(),
            disk: DiskCollector::default(),
            gpu: detect_gpu(),
            history: MetricsHistory::new(core_count),
        })
    }

    pub fn collect(&mut self) -> Result<SystemMetrics> {
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        let cpu = self.cpu.collect().unwrap_or_default();
        let memory = memory::collect().unwrap_or_default();
        let networks = self.net.collect();
        let disks = self.disk.collect();
        let temperatures = temperature::collect();
        let batteries = battery::collect();
        let battery = batteries.into_iter().next();
        let gpu = self.gpu.collect();

        let metrics = SystemMetrics {
            timestamp_ms,
            cpu,
            memory,
            disks,
            networks,
            temperatures,
            battery,
            gpu,
        };

        self.history.push(&metrics);
        Ok(metrics)
    }
}

fn num_cpus() -> usize {
    std::fs::read_dir("/sys/devices/system/cpu")
        .map(|d| {
            d.flatten()
                .filter(|e| {
                    let n = e.file_name();
                    let s = n.to_string_lossy();
                    s.starts_with("cpu") && s[3..].chars().all(|c| c.is_ascii_digit())
                })
                .count()
        })
        .unwrap_or(1)
        .max(1)
}
