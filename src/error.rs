use thiserror::Error;

#[derive(Error, Debug)]
pub enum VirtuosoError {
    #[error("connection failed: {0}")]
    Connection(String),

    #[error("execution failed: {0}")]
    Execution(String),

    #[error("ssh error: {0}")]
    Ssh(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("timeout after {0}s")]
    Timeout(u64),

    #[error("daemon not ready: {0}")]
    DaemonNotReady(String),

    #[error("config error: {0}")]
    Config(String),
}

pub type Result<T> = std::result::Result<T, VirtuosoError>;
