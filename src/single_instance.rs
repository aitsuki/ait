use crate::error::{AppError, Result};
use windows::Win32::Foundation::{CloseHandle, ERROR_ALREADY_EXISTS, GetLastError, HANDLE};
use windows::Win32::System::Threading::CreateMutexW;
use windows::core::w;

/// Keeps the session-local instance marker alive until the application exits.
pub(crate) struct SingleInstance {
    handle: HANDLE,
}

impl SingleInstance {
    pub(crate) fn acquire() -> Result<Option<Self>> {
        unsafe {
            let handle = CreateMutexW(None, false, w!("Local\\ait.SingleInstance"))
                .map_err(|err| AppError::Windows(format!("创建单实例互斥锁失败: {err}")))?;
            let already_running = GetLastError() == ERROR_ALREADY_EXISTS;
            let instance = Self { handle };
            if already_running {
                return Ok(None);
            }
            Ok(Some(instance))
        }
    }
}

impl Drop for SingleInstance {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.handle);
        }
    }
}
