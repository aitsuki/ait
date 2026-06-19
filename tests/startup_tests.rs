use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::Path;

use ait::startup::{
    AutoStartStore, auto_start_command_for_exe, auto_start_entry_name,
    is_auto_start_enabled_in_store, set_auto_start_enabled_in_store,
};

#[derive(Default)]
struct MemoryAutoStartStore {
    values: RefCell<BTreeMap<String, String>>,
}

impl AutoStartStore for MemoryAutoStartStore {
    fn read_entry(&self, name: &str) -> ait::error::Result<Option<String>> {
        Ok(self.values.borrow().get(name).cloned())
    }

    fn write_entry(&self, name: &str, value: &str) -> ait::error::Result<()> {
        self.values
            .borrow_mut()
            .insert(name.to_string(), value.to_string());
        Ok(())
    }

    fn delete_entry(&self, name: &str) -> ait::error::Result<()> {
        self.values.borrow_mut().remove(name);
        Ok(())
    }
}

#[test]
fn startup_entry_name_is_stable() {
    assert_eq!(auto_start_entry_name(), "ait");
}

#[test]
fn startup_command_quotes_exe_path() {
    let command = auto_start_command_for_exe(Path::new(r"C:\Program Files\ait\ait.exe"));

    assert_eq!(command, r#""C:\Program Files\ait\ait.exe""#);
}

#[test]
fn startup_store_reports_enabled_when_entry_exists() {
    let store = MemoryAutoStartStore::default();
    store.write_entry("ait", r#""C:\ait\ait.exe""#).unwrap();

    assert!(is_auto_start_enabled_in_store(&store).unwrap());
}

#[test]
fn startup_store_reports_disabled_when_entry_is_missing() {
    let store = MemoryAutoStartStore::default();

    assert!(!is_auto_start_enabled_in_store(&store).unwrap());
}

#[test]
fn enabling_startup_writes_current_exe_command() {
    let store = MemoryAutoStartStore::default();

    set_auto_start_enabled_in_store(&store, true, Path::new(r"C:\Tools\ait.exe")).unwrap();

    assert_eq!(
        store.values.borrow().get("ait").map(String::as_str),
        Some(r#""C:\Tools\ait.exe""#)
    );
}

#[test]
fn disabling_startup_deletes_existing_entry() {
    let store = MemoryAutoStartStore::default();
    store.write_entry("ait", r#""C:\ait\ait.exe""#).unwrap();

    set_auto_start_enabled_in_store(&store, false, Path::new(r"C:\ignored\ait.exe")).unwrap();

    assert_eq!(store.values.borrow().get("ait"), None);
}
