use crate::metrics::model::{GpuMetrics, GpuVendor};

pub trait GpuBackend: Send + Sync {
    fn name(&self) -> &str;
    fn collect(&self) -> Option<GpuMetrics>;
}

pub struct NoGpu;
impl GpuBackend for NoGpu {
    fn name(&self) -> &str {
        "none"
    }
    fn collect(&self) -> Option<GpuMetrics> {
        None
    }
}

pub struct IntelGpu {
    pub card: String,
    pub name: String,
}

impl GpuBackend for IntelGpu {
    fn name(&self) -> &str {
        &self.name
    }

    fn collect(&self) -> Option<GpuMetrics> {
        let base = format!("/sys/class/drm/{}/device", self.card);
        let freq_mhz = read_u64(format!("{}/drm/{}/gt/gt0/rps_cur_freq_mhz", base, self.card));

        Some(GpuMetrics {
            name: self.name.clone(),
            vendor: GpuVendor::Intel,
            utilization_pct: None, // Needs CAP_PERFMON
            mem_used_bytes: None,
            mem_total_bytes: None,
            temp_celsius: None,
            freq_mhz,
        })
    }
}

pub struct AmdGpu {
    pub card: String,
    pub name: String,
}

impl GpuBackend for AmdGpu {
    fn name(&self) -> &str {
        &self.name
    }

    fn collect(&self) -> Option<GpuMetrics> {
        let base = format!("/sys/class/drm/{}/device", self.card);

        let util_pct = read_u64(format!("{}/gpu_busy_percent", base))
            .map(|v| v.min(100) as f32);

        let mem_used = read_u64(format!("{}/mem_info_vram_used", base));
        let mem_total = read_u64(format!("{}/mem_info_vram_total", base));

        let freq_mhz = read_u64(format!("{}/pp_dpm_sclk", base)).or_else(|| {
            // Try the hwmon approach
            None
        });

        Some(GpuMetrics {
            name: self.name.clone(),
            vendor: GpuVendor::Amd,
            utilization_pct: util_pct,
            mem_used_bytes: mem_used,
            mem_total_bytes: mem_total,
            temp_celsius: None,
            freq_mhz,
        })
    }
}

pub fn detect_gpu() -> Box<dyn GpuBackend + Send + Sync> {
    // Check for DRM cards
    let Ok(entries) = std::fs::read_dir("/sys/class/drm") else {
        return Box::new(NoGpu);
    };

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("card") || name.contains('-') {
            continue;
        }

        let vendor_path = entry.path().join("device/vendor");
        let vendor_id = read_str(vendor_path).unwrap_or_default();
        let vendor_id = vendor_id.trim();

        if vendor_id == "0x8086" {
            let gpu_name = read_str(entry.path().join("device/product_name"))
                .unwrap_or_else(|| "Intel GPU".to_string());
            return Box::new(IntelGpu {
                card: name,
                name: gpu_name.trim().to_string(),
            });
        } else if vendor_id == "0x1002" {
            let gpu_name = read_str(entry.path().join("device/product_name"))
                .unwrap_or_else(|| "AMD GPU".to_string());
            return Box::new(AmdGpu {
                card: name,
                name: gpu_name.trim().to_string(),
            });
        }
    }

    Box::new(NoGpu)
}

fn read_u64(path: impl AsRef<std::path::Path>) -> Option<u64> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
}

fn read_str(path: impl AsRef<std::path::Path>) -> Option<String> {
    std::fs::read_to_string(path).ok()
}
