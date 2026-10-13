pub const HISTORY_LEN: usize = 300; // 5 min at 1 s/tick

#[derive(Debug, Clone, Default)]
pub struct SystemMetrics {
    pub timestamp_ms: u64,
    pub cpu: CpuMetrics,
    pub memory: MemoryMetrics,
    pub disks: Vec<DiskMetrics>,
    pub networks: Vec<NetworkMetrics>,
    pub temperatures: Vec<TempSensor>,
    pub battery: Option<BatteryMetrics>,
    pub gpu: Option<GpuMetrics>,
}

#[derive(Debug, Clone, Default)]
pub struct CpuMetrics {
    pub global_pct: f32,
    pub per_core_pct: Vec<f32>,
    pub freq_mhz: Vec<u64>,
    pub load_1m: f64,
    pub load_5m: f64,
    pub load_15m: f64,
    pub uptime_secs: u64,
}

#[derive(Debug, Clone, Default)]
pub struct MemoryMetrics {
    /// All values in bytes
    pub total: u64,
    pub used: u64,
    pub available: u64,
    pub cached: u64,
    pub buffers: u64,
    pub swap_total: u64,
    pub swap_used: u64,
}

impl MemoryMetrics {
    pub fn used_pct(&self) -> f32 {
        if self.total == 0 {
            return 0.0;
        }
        self.used as f32 / self.total as f32 * 100.0
    }

    pub fn swap_pct(&self) -> f32 {
        if self.swap_total == 0 {
            return 0.0;
        }
        self.swap_used as f32 / self.swap_total as f32 * 100.0
    }
}

#[derive(Debug, Clone, Default)]
pub struct NetworkMetrics {
    pub interface: String,
    pub rx_bytes_per_sec: u64,
    pub tx_bytes_per_sec: u64,
    pub rx_total: u64,
    pub tx_total: u64,
    pub rx_errors: u64,
    pub tx_errors: u64,
}

#[derive(Debug, Clone, Default)]
pub struct DiskMetrics {
    pub name: String,
    pub mount: Option<String>,
    pub read_bytes_per_sec: u64,
    pub write_bytes_per_sec: u64,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub is_rotational: bool,
}

impl DiskMetrics {
    pub fn used_pct(&self) -> f32 {
        if self.total_bytes == 0 {
            return 0.0;
        }
        self.used_bytes as f32 / self.total_bytes as f32 * 100.0
    }
}

#[derive(Debug, Clone)]
pub struct TempSensor {
    pub label: String,
    pub device: String,
    pub temp_celsius: f32,
    pub max_celsius: Option<f32>,
    pub crit_celsius: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct BatteryMetrics {
    pub name: String,
    pub capacity_pct: u8,
    pub status: BatteryStatus,
    pub power_uw: Option<u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BatteryStatus {
    Charging,
    Discharging,
    Full,
    Unknown(String),
}

impl std::fmt::Display for BatteryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BatteryStatus::Charging => write!(f, "Charging"),
            BatteryStatus::Discharging => write!(f, "Discharging"),
            BatteryStatus::Full => write!(f, "Full"),
            BatteryStatus::Unknown(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GpuMetrics {
    pub name: String,
    pub vendor: GpuVendor,
    pub utilization_pct: Option<f32>,
    pub mem_used_bytes: Option<u64>,
    pub mem_total_bytes: Option<u64>,
    pub temp_celsius: Option<f32>,
    pub freq_mhz: Option<u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GpuVendor {
    Nvidia,
    Amd,
    Intel,
    Unknown,
}

impl std::fmt::Display for GpuVendor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GpuVendor::Nvidia => write!(f, "NVIDIA"),
            GpuVendor::Amd => write!(f, "AMD"),
            GpuVendor::Intel => write!(f, "Intel"),
            GpuVendor::Unknown => write!(f, "GPU"),
        }
    }
}
