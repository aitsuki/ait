use crate::error::{AppError, Result};
use std::path::PathBuf;

pub fn log_dir() -> Result<PathBuf> {
    let project_dirs = directories::ProjectDirs::from("dev", "aitsu", "ait")
        .ok_or_else(|| AppError::Config("无法定位日志目录".to_string()))?;
    Ok(project_dirs.data_local_dir().join("logs"))
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

pub fn safe_text_len(text: &str) -> usize {
    text.chars().count()
}
