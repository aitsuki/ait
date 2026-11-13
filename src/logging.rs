use crate::error::Result;
use std::path::PathBuf;

pub fn log_dir() -> Result<PathBuf> {
    Ok(crate::paths::data_dir()?.join("logs"))
}

pub fn init_logging() -> Result<PathBuf> {
    let log_dir = log_dir()?;
    std::fs::create_dir_all(&log_dir)?;

    let file_appender = tracing_appender::rolling::daily(&log_dir, "ait.log");
    tracing_subscriber::fmt()
        .with_writer(file_appender)
        .with_ansi(false)
        .with_target(false)
        .init();

    Ok(log_dir)
}
