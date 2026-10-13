pub type Pid = u32;

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: Pid,
    pub ppid: Pid,
    pub name: String,
    pub cmdline: String,
    pub status: ProcessStatus,
    pub cpu_pct: f32,
    pub mem_bytes: u64,
    pub mem_pct: f32,
    pub threads: u32,
    pub priority: i64,
    pub nice: i64,
    pub uid: u32,
    pub username: String,
    pub starttime_ticks: u64,
    pub io_read_bytes: u64,
    pub io_write_bytes: u64,
    pub is_kernel: bool,
    pub children: Vec<Pid>,
}

impl ProcessInfo {
    pub fn status_char(&self) -> char {
        match &self.status {
            ProcessStatus::Running => 'R',
            ProcessStatus::Sleeping => 'S',
            ProcessStatus::DiskSleep => 'D',
            ProcessStatus::Stopped => 'T',
            ProcessStatus::Zombie => 'Z',
            ProcessStatus::Idle => 'I',
            ProcessStatus::Dead => 'X',
            ProcessStatus::Unknown(c) => *c,
        }
    }

    pub fn status_str(&self) -> &'static str {
        match &self.status {
            ProcessStatus::Running => "Running",
            ProcessStatus::Sleeping => "Sleeping",
            ProcessStatus::DiskSleep => "DiskSleep",
            ProcessStatus::Stopped => "Stopped",
            ProcessStatus::Zombie => "Zombie",
            ProcessStatus::Idle => "Idle",
            ProcessStatus::Dead => "Dead",
            ProcessStatus::Unknown(_) => "Unknown",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ProcessStatus {
    Running,
    Sleeping,
    DiskSleep,
    Stopped,
    Zombie,
    Idle,
    Dead,
    Unknown(char),
}

impl From<char> for ProcessStatus {
    fn from(c: char) -> Self {
        match c {
            'R' => ProcessStatus::Running,
            'S' => ProcessStatus::Sleeping,
            'D' => ProcessStatus::DiskSleep,
            'T' => ProcessStatus::Stopped,
            'Z' => ProcessStatus::Zombie,
            'I' => ProcessStatus::Idle,
            'X' => ProcessStatus::Dead,
            other => ProcessStatus::Unknown(other),
        }
    }
}
