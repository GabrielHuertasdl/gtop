use std::collections::HashMap;
use std::time::Instant;

use crate::metrics::model::DiskMetrics;

#[derive(Clone)]
struct DiskSnapshot {
    read_sectors: u64,
    write_sectors: u64,
    at: Instant,
}

#[derive(Default)]
pub struct DiskCollector {
    prev: HashMap<String, DiskSnapshot>,
    mounts: HashMap<String, String>, // basename → mountpoint
}

impl DiskCollector {
    pub fn collect(&mut self) -> Vec<DiskMetrics> {
        self.mounts = read_mounts();

        let Ok(diskstats) = procfs::diskstats() else {
            return Vec::new();
        };

        let now = Instant::now();
        let mut result = Vec::new();

        const SECTOR_SIZE: u64 = 512;

        for disk in &diskstats {
            let name = &disk.name;
            if name.starts_with("loop")
                || name.starts_with("ram")
                || name.starts_with("dm-")
                || name.starts_with("sr")
                || name.starts_with("zram")
            {
                continue;
            }
            // Skip partitions
            if (name.starts_with("sd") || name.starts_with("hd") || name.starts_with("vd"))
                && name.chars().last().map(|c| c.is_ascii_digit()).unwrap_or(false)
            {
                continue;
            }

            let (read_bps, write_bps) = if let Some(prev) = self.prev.get(name) {
                let elapsed = now.duration_since(prev.at).as_secs_f64().max(0.001);
                let rd = disk.sectors_read.saturating_sub(prev.read_sectors) * SECTOR_SIZE;
                let wd = disk.sectors_written.saturating_sub(prev.write_sectors) * SECTOR_SIZE;
                (
                    (rd as f64 / elapsed) as u64,
                    (wd as f64 / elapsed) as u64,
                )
            } else {
                (0, 0)
            };

            self.prev.insert(
                name.clone(),
                DiskSnapshot {
                    read_sectors: disk.sectors_read,
                    write_sectors: disk.sectors_written,
                    at: now,
                },
            );

            let mount = self.mounts.get(name).cloned();
            let (total_bytes, used_bytes) = mount
                .as_deref()
                .and_then(statvfs_usage)
                .unwrap_or((0, 0));

            let is_rotational = read_rotational(name);

            result.push(DiskMetrics {
                name: name.clone(),
                mount,
                read_bytes_per_sec: read_bps,
                write_bytes_per_sec: write_bps,
                total_bytes,
                used_bytes,
                is_rotational,
            });
        }

        result.sort_by(|a, b| a.name.cmp(&b.name));
        result
    }
}

fn read_mounts() -> HashMap<String, String> {
    let mut map = HashMap::new();
    let Ok(content) = std::fs::read_to_string("/proc/mounts") else {
        return map;
    };
    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 && parts[0].starts_with("/dev/") {
            let dev = parts[0].trim_start_matches("/dev/").to_string();
            map.insert(dev, parts[1].to_string());
        }
    }
    map
}

fn statvfs_usage(path: &str) -> Option<(u64, u64)> {
    use nix::sys::statvfs::statvfs;
    let stat = statvfs(path).ok()?;
    let bsize = stat.fragment_size() as u64;
    let total = stat.blocks() as u64 * bsize;
    let avail = stat.blocks_available() as u64 * bsize;
    Some((total, total.saturating_sub(avail)))
}

fn read_rotational(name: &str) -> bool {
    let path = format!("/sys/block/{}/queue/rotational", name);
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| s.trim().parse::<u8>().ok())
        .map(|v| v == 1)
        .unwrap_or(false)
}
