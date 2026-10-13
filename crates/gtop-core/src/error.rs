use thiserror::Error;

pub type Pid = u32;

#[derive(Error, Debug)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Process {0} not found")]
    ProcessNotFound(Pid),

    #[error("Permission denied for PID {0}")]
    PermissionDenied(Pid),

    #[error("Nix error: {0}")]
    Nix(#[from] nix::errno::Errno),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Proc error: {0}")]
    Proc(String),

    #[error("Config error: {0}")]
    Config(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<procfs::ProcError> for Error {
    fn from(e: procfs::ProcError) -> Self {
        Error::Proc(e.to_string())
    }
}
