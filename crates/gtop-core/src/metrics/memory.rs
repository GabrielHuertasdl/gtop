use crate::error::Result;
use crate::metrics::model::MemoryMetrics;

pub fn collect() -> Result<MemoryMetrics> {
    let content = std::fs::read_to_string("/proc/meminfo")
        .map_err(crate::error::Error::Io)?;

    let mut fields = std::collections::HashMap::new();
    for line in content.lines() {
        let mut parts = line.split_whitespace();
        if let (Some(key), Some(val)) = (parts.next(), parts.next()) {
            if let Ok(v) = val.parse::<u64>() {
                let key = key.trim_end_matches(':');
                fields.insert(key.to_string(), v * 1024); // kB → bytes
            }
        }
    }

    let get = |k: &str| fields.get(k).copied().unwrap_or(0);

    let total = get("MemTotal");
    let available = get("MemAvailable").max(get("MemFree"));
    let used = total.saturating_sub(available);

    Ok(MemoryMetrics {
        total,
        used,
        available,
        cached: get("Cached"),
        buffers: get("Buffers"),
        swap_total: get("SwapTotal"),
        swap_used: get("SwapTotal").saturating_sub(get("SwapFree")),
    })
}
