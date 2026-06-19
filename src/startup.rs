use std::path::Path;

use crate::error::Result;

const AUTO_START_ENTRY_NAME: &str = "ait";

pub trait AutoStartStore {
    fn read_entry(&self, name: &str) -> Result<Option<String>>;
    fn write_entry(&self, name: &str, value: &str) -> Result<()>;
    fn delete_entry(&self, name: &str) -> Result<()>;
}

pub fn auto_start_entry_name() -> &'static str {
    AUTO_START_ENTRY_NAME
}

pub fn auto_start_command_for_exe(exe_path: &Path) -> String {
    format!("\"{}\"", exe_path.display())
}

pub fn is_auto_start_enabled_in_store(store: &impl AutoStartStore) -> Result<bool> {
    Ok(store.read_entry(AUTO_START_ENTRY_NAME)?.is_some())
}

pub fn set_auto_start_enabled_in_store(
    store: &impl AutoStartStore,
    enabled: bool,
    exe_path: &Path,
) -> Result<()> {
    if enabled {
        store.write_entry(AUTO_START_ENTRY_NAME, &auto_start_command_for_exe(exe_path))
    } else {
        store.delete_entry(AUTO_START_ENTRY_NAME)
    }
}

#[cfg(not(windows))]
pub fn is_auto_start_enabled() -> Result<bool> {
    Ok(false)
}

#[cfg(not(windows))]
pub fn set_auto_start_enabled(_enabled: bool) -> Result<()> {
    Ok(())
}

#[cfg(windows)]
const RUN_KEY_PATH: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";

#[cfg(windows)]
pub fn registry_string_to_bytes(value: &str) -> Vec<u8> {
    value
        .encode_utf16()
        .chain(Some(0))
        .flat_map(u16::to_le_bytes)
        .collect()
}

#[cfg(windows)]
pub fn registry_string_from_bytes(bytes: &[u8]) -> Result<String> {
    let mut units = Vec::with_capacity(bytes.len() / 2);
    for chunk in bytes.chunks_exact(2) {
        let unit = u16::from_le_bytes([chunk[0], chunk[1]]);
        if unit == 0 {
            break;
        }
        units.push(unit);
    }
    String::from_utf16(&units).map_err(|err| crate::error::AppError::Config(err.to_string()))
}

#[cfg(windows)]
struct WindowsRunRegistry;

#[cfg(windows)]
struct RegistryKey(windows::Win32::System::Registry::HKEY);

#[cfg(windows)]
impl Drop for RegistryKey {
    fn drop(&mut self) {
        unsafe {
            let _ = windows::Win32::System::Registry::RegCloseKey(self.0);
        }
    }
}

#[cfg(windows)]
impl WindowsRunRegistry {
    fn open_read() -> Result<Option<RegistryKey>> {
        use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
        use windows::Win32::System::Registry::{HKEY, HKEY_CURRENT_USER, KEY_READ, RegOpenKeyExW};
        use windows::core::PCWSTR;

        let mut key = HKEY::default();
        let path = wide(RUN_KEY_PATH);
        let status = unsafe {
            RegOpenKeyExW(
                HKEY_CURRENT_USER,
                PCWSTR(path.as_ptr()),
                None,
                KEY_READ,
                &mut key,
            )
        };
        if status == ERROR_FILE_NOT_FOUND {
            return Ok(None);
        }
        if status != ERROR_SUCCESS {
            return Err(crate::error::AppError::Windows(format!(
                "打开自启注册表失败: {}",
                std::io::Error::from_raw_os_error(status.0 as i32)
            )));
        }
        Ok(Some(RegistryKey(key)))
    }

    fn open_write() -> Result<RegistryKey> {
        use windows::Win32::Foundation::ERROR_SUCCESS;
        use windows::Win32::System::Registry::{
            HKEY, HKEY_CURRENT_USER, KEY_SET_VALUE, REG_OPEN_CREATE_OPTIONS, RegCreateKeyExW,
        };
        use windows::core::PCWSTR;

        let mut key = HKEY::default();
        let path = wide(RUN_KEY_PATH);
        let status = unsafe {
            RegCreateKeyExW(
                HKEY_CURRENT_USER,
                PCWSTR(path.as_ptr()),
                None,
                windows::core::PWSTR::null(),
                REG_OPEN_CREATE_OPTIONS(0),
                KEY_SET_VALUE,
                None,
                &mut key,
                None,
            )
        };
        if status != ERROR_SUCCESS {
            return Err(crate::error::AppError::Windows(format!(
                "创建自启注册表失败: {}",
                std::io::Error::from_raw_os_error(status.0 as i32)
            )));
        }
        Ok(RegistryKey(key))
    }
}

#[cfg(windows)]
impl AutoStartStore for WindowsRunRegistry {
    fn read_entry(&self, name: &str) -> Result<Option<String>> {
        use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
        use windows::Win32::System::Registry::{REG_SZ, REG_VALUE_TYPE, RegQueryValueExW};
        use windows::core::PCWSTR;

        let Some(key) = Self::open_read()? else {
            return Ok(None);
        };
        let name = wide(name);
        let mut value_type = REG_VALUE_TYPE::default();
        let mut len = 0u32;
        let status = unsafe {
            RegQueryValueExW(
                key.0,
                PCWSTR(name.as_ptr()),
                None,
                Some(&mut value_type),
                None,
                Some(&mut len),
            )
        };
        if status == ERROR_FILE_NOT_FOUND {
            return Ok(None);
        }
        if status != ERROR_SUCCESS {
            return Err(crate::error::AppError::Windows(format!(
                "读取自启注册表失败: {}",
                std::io::Error::from_raw_os_error(status.0 as i32)
            )));
        }
        if value_type != REG_SZ {
            return Ok(None);
        }

        let mut bytes = vec![0u8; len as usize];
        let status = unsafe {
            RegQueryValueExW(
                key.0,
                PCWSTR(name.as_ptr()),
                None,
                Some(&mut value_type),
                Some(bytes.as_mut_ptr()),
                Some(&mut len),
            )
        };
        if status != ERROR_SUCCESS {
            return Err(crate::error::AppError::Windows(format!(
                "读取自启注册表失败: {}",
                std::io::Error::from_raw_os_error(status.0 as i32)
            )));
        }
        Ok(Some(registry_string_from_bytes(&bytes)?))
    }

    fn write_entry(&self, name: &str, value: &str) -> Result<()> {
        use windows::Win32::Foundation::ERROR_SUCCESS;
        use windows::Win32::System::Registry::{REG_SZ, RegSetValueExW};
        use windows::core::PCWSTR;

        let key = Self::open_write()?;
        let name = wide(name);
        let bytes = registry_string_to_bytes(value);
        let status =
            unsafe { RegSetValueExW(key.0, PCWSTR(name.as_ptr()), None, REG_SZ, Some(&bytes)) };
        if status != ERROR_SUCCESS {
            return Err(crate::error::AppError::Windows(format!(
                "写入自启注册表失败: {}",
                std::io::Error::from_raw_os_error(status.0 as i32)
            )));
        }
        Ok(())
    }

    fn delete_entry(&self, name: &str) -> Result<()> {
        use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
        use windows::Win32::System::Registry::RegDeleteValueW;
        use windows::core::PCWSTR;

        let key = Self::open_write()?;
        let name = wide(name);
        let status = unsafe { RegDeleteValueW(key.0, PCWSTR(name.as_ptr())) };
        if status == ERROR_FILE_NOT_FOUND {
            return Ok(());
        }
        if status != ERROR_SUCCESS {
            return Err(crate::error::AppError::Windows(format!(
                "删除自启注册表失败: {}",
                std::io::Error::from_raw_os_error(status.0 as i32)
            )));
        }
        Ok(())
    }
}

#[cfg(windows)]
pub fn is_auto_start_enabled() -> Result<bool> {
    is_auto_start_enabled_in_store(&WindowsRunRegistry)
}

#[cfg(windows)]
pub fn set_auto_start_enabled(enabled: bool) -> Result<()> {
    let exe = std::env::current_exe()?;
    set_auto_start_enabled_in_store(&WindowsRunRegistry, enabled, &exe)
}

#[cfg(windows)]
fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(Some(0)).collect()
}
