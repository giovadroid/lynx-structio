pub mod traits;
pub mod core;
pub mod file_monitor;

pub type MonitorResult<T> = Result<T, anyhow::Error>;
