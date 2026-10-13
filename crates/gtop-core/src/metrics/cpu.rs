use crate::error::Result;
use crate::metrics::model::CpuMetrics;

#[derive(Default, Clone)]
struct CpuSnapshot {
    total: u64,
    idle: u64,
    per_core: Vec<(u64, u64)>, // (total, idle)
}

#[derive(Default)]
pub struct CpuCollector {
    prev: Option<CpuSnapshot>,
}

impl CpuCollector {
    pub fn collect(&mut self) -> Result<CpuMetrics> {
        let current = read_cpu_snapshot()?;

        let (global_pct, per_core_pct): (f32, Vec<f32>) = if let Some(ref prev) = self.prev {
            let total_d = current.total.saturating_sub(prev.total);
            let idle_d = current.idle.saturating_sub(prev.idle);
            let gp = if total_d > 0 {
                (total_d - idle_d) as f32 / total_d as f32 * 100.0
            } else {
                0.0
            };
            let cores: Vec<f32> = current
                .per_core
                .iter()
                .zip(prev.per_core.iter())
                .map(|((ct, ci), (pt, pi))| {
                    let td = ct.saturating_sub(*pt);
                    let id = ci.saturating_sub(*pi);
                    if td > 0 {
                        (td - id) as f32 / td as f32 * 100.0
                    } else {
                        0.0
                    }
                })
                .collect();
            (gp, cores)
        } else {
            (0.0, vec![0.0; current.per_core.len()])
        };

        self.prev = Some(current);

        let uptime = read_uptime();
        let (load_1m, load_5m, load_15m) = read_loadavg();

        Ok(CpuMetrics {
            global_pct: global_pct.clamp(0.0, 100.0),
            per_core_pct: per_core_pct
                .into_iter()
                .map(|v| v.clamp(0.0, 100.0))
                .collect(),
            freq_mhz: read_cpu_freqs(),
            load_1m,
            load_5m,
            load_15m,
            uptime_secs: uptime,
        })
    }
}

fn read_cpu_snapshot() -> Result<CpuSnapshot> {
    let content = std::fs::read_to_string("/proc/stat")
        .map_err(|e| crate::error::Error::Io(e))?;

    let mut total = 0u64;
    let mut idle_val = 0u64;
    let mut per_core: Vec<(u64, u64)> = Vec::new();

    for line in content.lines() {
        if line.starts_with("cpu") {
            let parts: Vec<u64> = line
                .split_whitespace()
                .skip(1) // skip "cpu" or "cpuN"
                .filter_map(|v| v.parse().ok())
                .collect();

            if parts.len() < 4 {
                continue;
            }

            // user nice system idle iowait irq softirq steal
            let idle = parts.get(3).copied().unwrap_or(0)
                + parts.get(4).copied().unwrap_or(0); // idle + iowait
            let t: u64 = parts.iter().sum();

            let name = line.split_whitespace().next().unwrap_or("");
            if name == "cpu" {
                total = t;
                idle_val = idle;
            } else {
                per_core.push((t, idle));
            }
        }
    }

    Ok(CpuSnapshot {
        total,
        idle: idle_val,
        per_core,
    })
}

fn read_uptime() -> u64 {
    std::fs::read_to_string("/proc/uptime")
        .ok()
        .and_then(|s| {
            s.split_whitespace()
                .next()
                .and_then(|v| v.parse::<f64>().ok())
        })
        .map(|v| v as u64)
        .unwrap_or(0)
}

fn read_loadavg() -> (f64, f64, f64) {
    let content = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    let mut parts = content.split_whitespace();
    let l1 = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0.0);
    let l5 = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0.0);
    let l15 = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0.0);
    (l1, l5, l15)
}

fn read_cpu_freqs() -> Vec<u64> {
    let mut freqs = Vec::new();
    for i in 0..512 {
        let path = format!(
            "/sys/devices/system/cpu/cpu{}/cpufreq/scaling_cur_freq",
            i
        );
        match std::fs::read_to_string(&path) {
            Ok(content) => {
                if let Ok(khz) = content.trim().parse::<u64>() {
                    freqs.push(khz / 1000);
                } else {
                    break;
                }
            }
            Err(_) => break,
        }
    }
    freqs
}
