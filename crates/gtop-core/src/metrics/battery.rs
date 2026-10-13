use crate::metrics::model::{BatteryMetrics, BatteryStatus};

pub fn collect() -> Vec<BatteryMetrics> {
    let mut batteries = Vec::new();

    let Ok(entries) = std::fs::read_dir("/sys/class/power_supply") else {
        return batteries;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        // Only process batteries, not AC adapters
        let type_val = read_str(path.join("type")).unwrap_or_default();
        if type_val.trim() != "Battery" {
            continue;
        }

        let capacity_pct = read_u64(path.join("capacity"))
            .map(|v| v.min(100) as u8)
            .unwrap_or(0);

        let status_str = read_str(path.join("status")).unwrap_or_default();
        let status = match status_str.trim() {
            "Charging" => BatteryStatus::Charging,
            "Discharging" => BatteryStatus::Discharging,
            "Full" => BatteryStatus::Full,
            other => BatteryStatus::Unknown(other.to_string()),
        };

        let power_uw = read_u64(path.join("power_now"));

        batteries.push(BatteryMetrics {
            name,
            capacity_pct,
            status,
            power_uw,
        });
    }

    batteries
}

fn read_str(path: impl AsRef<std::path::Path>) -> Option<String> {
    std::fs::read_to_string(path).ok()
}

fn read_u64(path: impl AsRef<std::path::Path>) -> Option<u64> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
}
