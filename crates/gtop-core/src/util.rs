/// Format bytes into human-readable string (B, KB, MB, GB, TB)
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "K", "M", "G", "T", "P"];
    if bytes == 0 {
        return "0B".to_string();
    }
    let mut value = bytes as f64;
    let mut unit_idx = 0;
    while value >= 1024.0 && unit_idx < UNITS.len() - 1 {
        value /= 1024.0;
        unit_idx += 1;
    }
    if unit_idx == 0 {
        format!("{}B", bytes)
    } else if value >= 100.0 {
        format!("{:.0}{}", value, UNITS[unit_idx])
    } else if value >= 10.0 {
        format!("{:.1}{}", value, UNITS[unit_idx])
    } else {
        format!("{:.2}{}", value, UNITS[unit_idx])
    }
}

/// Format bytes/sec into human-readable speed
pub fn format_speed(bytes_per_sec: u64) -> String {
    format!("{}/s", format_bytes(bytes_per_sec))
}

/// Format seconds into human-readable duration
pub fn format_uptime(secs: u64) -> String {
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let mins = (secs % 3600) / 60;
    if days > 0 {
        format!("{}d {:02}h {:02}m", days, hours, mins)
    } else if hours > 0 {
        format!("{}h {:02}m", hours, mins)
    } else {
        format!("{}m", mins)
    }
}

/// Format ticks-since-boot to a datetime string
pub fn format_start_time(ticks: u64, ticks_per_sec: u64, boot_time: u64) -> String {
    if ticks_per_sec == 0 {
        return "-".to_string();
    }
    let started_at = boot_time + ticks / ticks_per_sec;
    // Simple HH:MM format
    let secs_of_day = started_at % 86400;
    let h = secs_of_day / 3600;
    let m = (secs_of_day % 3600) / 60;
    let s = secs_of_day % 60;
    format!("{:02}:{:02}:{:02}", h, m, s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_bytes_works() {
        assert_eq!(format_bytes(0), "0B");
        assert_eq!(format_bytes(512), "512B");
        assert_eq!(format_bytes(1024), "1.00K");
        assert_eq!(format_bytes(1536), "1.50K");
        assert_eq!(format_bytes(1024 * 1024), "1.00M");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00G");
    }
}
