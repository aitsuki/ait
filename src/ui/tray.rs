use crate::error::Result;

#[cfg(windows)]
pub struct TrayIcon;

#[cfg(windows)]
impl TrayIcon {
    pub fn create() -> Result<Self> {
        tracing::info!("tray icon placeholder created");
        Ok(Self)
    }
}

#[cfg(not(windows))]
pub struct TrayIcon;

#[cfg(not(windows))]
impl TrayIcon {
    pub fn create() -> Result<Self> {
        Ok(Self)
    }
}
