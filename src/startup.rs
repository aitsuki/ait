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
