use crate::error::{AppError, Result};
use std::path::PathBuf;

pub fn is_portable() -> bool {
    cfg!(feature = "portable")
}

fn portable_dir() -> Result<PathBuf> {
    let exe = std::env::current_exe()?;
    let dir = exe
        .parent()
        .ok_or_else(|| AppError::Config("无法定位程序目录".to_string()))?;
    Ok(dir.join("data"))
}

fn project_dirs() -> Result<directories::ProjectDirs> {
    directories::ProjectDirs::from("dev", "aitsu", "ait")
        .ok_or_else(|| AppError::Config("无法定位数据目录".to_string()))
}

pub fn config_dir() -> Result<PathBuf> {
    if is_portable() {
        portable_dir()
    } else {
        Ok(project_dirs()?.config_dir().to_path_buf())
    }
}

pub fn data_dir() -> Result<PathBuf> {
    if is_portable() {
        portable_dir()
    } else {
        Ok(project_dirs()?.data_local_dir().to_path_buf())
    }
}
