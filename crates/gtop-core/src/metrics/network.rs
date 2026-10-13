use std::collections::HashMap;
use std::time::Instant;

use crate::error::Result;
use crate::metrics::model::NetworkMetrics;

#[derive(Clone)]
struct NetSnapshot {
    rx: u64,
    tx: u64,
    rx_errors: u64,
    tx_errors: u64,
    at: Instant,
}

#[derive(Default)]
pub struct NetworkCollector {
    prev: HashMap<String, NetSnapshot>,
}

impl NetworkCollector {
    pub fn collect(&mut self) -> Vec<NetworkMetrics> {
        let Ok(devs) = procfs::net::dev_status() else {
            return Vec::new();
        };

        let now = Instant::now();
        let mut result = Vec::new();

        for (iface, stats) in &devs {
            if iface == "lo" {
                continue;
            }

            let (rx_per_sec, tx_per_sec) = if let Some(prev) = self.prev.get(iface) {
                let elapsed = now.duration_since(prev.at).as_secs_f64().max(0.001);
                let rx_d = stats.recv_bytes.saturating_sub(prev.rx);
                let tx_d = stats.sent_bytes.saturating_sub(prev.tx);
                (
                    (rx_d as f64 / elapsed) as u64,
                    (tx_d as f64 / elapsed) as u64,
                )
            } else {
                (0, 0)
            };

            let snap = NetSnapshot {
                rx: stats.recv_bytes,
                tx: stats.sent_bytes,
                rx_errors: stats.recv_errs,
                tx_errors: stats.sent_errs,
                at: now,
            };

            result.push(NetworkMetrics {
                interface: iface.clone(),
                rx_bytes_per_sec: rx_per_sec,
                tx_bytes_per_sec: tx_per_sec,
                rx_total: stats.recv_bytes,
                tx_total: stats.sent_bytes,
                rx_errors: stats.recv_errs,
                tx_errors: stats.sent_errs,
            });

            self.prev.insert(iface.clone(), snap);
        }

        result.sort_by(|a, b| a.interface.cmp(&b.interface));
        result
    }
}
