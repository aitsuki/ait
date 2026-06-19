# v0.1.2 Startup And Version Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:executing-plans` to implement this plan task-by-task. Do not use `superpowers:subagent-driven-development`; this repository's `AGENTS.md` forbids it. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a settings checkbox named `开启自启` that syncs the current user's Windows Run startup entry on save, and show the app version in the settings window.

**Architecture:** Add a focused `startup` module that owns startup-entry naming, command generation, and Windows Run registry access. Keep startup state out of `settings.json`; settings UI reads the current system state when opening and writes the registry only when the user saves. Extend the settings view model and native settings window with an auto-start checkbox and static version label.

**Tech Stack:** Rust 2024, `windows` crate 0.62, Win32 registry APIs, Win32 native controls, existing Rust integration tests.

---

## File Structure

- Modify `Cargo.toml`
  - Add the `Win32_System_Registry` feature to the existing `windows` dependency.
- Modify `src/lib.rs`
  - Export the new `startup` module.
- Create `src/startup.rs`
  - Own startup entry constants, command formatting, testable store logic, and Windows Run registry implementation.
- Create `tests/startup_tests.rs`
  - Test entry name, command generation, enable/disable behavior through an in-memory store, and non-Windows/default behavior.
- Modify `tests/config_tests.rs`
  - Add a regression test proving `settings.json` does not contain an auto-start field.
- Modify `src/ui/settings_window.rs`
  - Add `auto_start_enabled` and `version_text` to `SettingsViewModel`.
  - Add `auto_start_enabled` to `SettingsProfileDetailUpdate`.
  - Add layout rectangles for the checkbox and version label.
  - Add Win32 checkbox creation, checkbox read/write helpers, and registry sync on save.
- Modify `tests/settings_window_tests.rs`
  - Add view-model, update-struct, layout, and version-label tests.
- Modify `Cargo.toml`, `Cargo.lock`, `README.md`
  - Bump package/documentation version to `0.1.2` after feature behavior is implemented and verified.

---

### Task 1: Startup Module Pure Behavior

**Files:**
- Create: `src/startup.rs`
- Modify: `src/lib.rs`
- Test: `tests/startup_tests.rs`

- [ ] **Step 1: Write failing startup tests**

Create `tests/startup_tests.rs`:

```rust
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
```

- [ ] **Step 2: Run startup tests to verify RED**

Run:

```powershell
cargo test --test startup_tests
```

Expected: compile failure because `ait::startup` does not exist.

- [ ] **Step 3: Export the startup module**

Modify `src/lib.rs` to include:

```rust
pub mod app;
pub mod capture;
pub mod command;
pub mod config;
pub mod error;
pub mod hotkey;
pub mod logging;
pub mod secret;
pub mod startup;
pub mod translator;
pub mod ui;
```

- [ ] **Step 4: Add minimal testable startup implementation**

Create `src/startup.rs`:

```rust
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
```

- [ ] **Step 5: Run startup tests to verify GREEN**

Run:

```powershell
cargo test --test startup_tests
```

Expected: all 6 tests pass.

- [ ] **Step 6: Commit Task 1**

```powershell
git add src/lib.rs src/startup.rs tests/startup_tests.rs
git commit -m "feat: add startup setting model"
```

---

### Task 2: Windows Run Registry Implementation

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/startup.rs`
- Test: `tests/startup_tests.rs`

- [ ] **Step 1: Add Windows registry feature**

Modify the existing `windows` dependency in `Cargo.toml` by adding `"Win32_System_Registry"` to the feature list:

```toml
windows = { version = "0.62.2", features = ["Win32_Foundation", "Win32_System_Com", "Win32_System_DataExchange", "Win32_System_Memory", "Win32_System_Registry", "Win32_System_Threading", "Win32_Security_Cryptography", "Win32_UI_Accessibility", "Win32_UI_Controls", "Win32_UI_Input_KeyboardAndMouse", "Win32_UI_Shell", "Win32_UI_WindowsAndMessaging", "Win32_Graphics_Gdi"] }
```

- [ ] **Step 2: Add failing Windows encoding test**

Append to `tests/startup_tests.rs`:

```rust
#[cfg(windows)]
#[test]
fn utf16_registry_value_round_trips() {
    let bytes = ait::startup::registry_string_to_bytes(r#""C:\Program Files\ait\ait.exe""#);
    let decoded = ait::startup::registry_string_from_bytes(&bytes).unwrap();

    assert_eq!(decoded, r#""C:\Program Files\ait\ait.exe""#);
}
```

- [ ] **Step 3: Run test to verify RED**

Run:

```powershell
cargo test --test startup_tests
```

Expected: compile failure because `registry_string_to_bytes` and `registry_string_from_bytes` do not exist.

- [ ] **Step 4: Add Windows registry helpers and implementation**

Append this Windows-specific code to `src/startup.rs`:

```rust
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
```

- [ ] **Step 5: Run startup tests**

Run:

```powershell
cargo test --test startup_tests
```

Expected: all startup tests pass.

- [ ] **Step 6: Run compile check**

Run:

```powershell
cargo check
```

Expected: check succeeds.

- [ ] **Step 7: Commit Task 2**

```powershell
git add Cargo.toml Cargo.lock src/startup.rs tests/startup_tests.rs
git commit -m "feat: manage windows startup entry"
```

---

### Task 3: Settings View Model And Config Compatibility

**Files:**
- Modify: `src/ui/settings_window.rs`
- Modify: `tests/settings_window_tests.rs`
- Modify: `tests/config_tests.rs`

- [ ] **Step 1: Add failing settings view-model tests**

Modify the import in `tests/settings_window_tests.rs` to include `app_version_text`:

```rust
use ait::ui::settings_window::{
    SettingsApiKeyUpdate, SettingsEditAction, SettingsProfileDetailControl,
    SettingsProfileDetailUpdate, SettingsSaveOutcome, SettingsViewModel, api_key_placeholder_text,
    app_version_text, apply_settings_detail_update, apply_settings_edit_action, hotkey_capture_text,
    settings_api_key_input_text, settings_api_key_update_from_input,
    settings_profile_detail_control_rect, settings_profile_detail_control_states,
    settings_profile_detail_hidden_rect, settings_profile_google_notice_text,
    settings_save_outcome_after_success, settings_static_controls_have_border,
    settings_window_center_position, settings_window_layout, settings_window_uses_background_brush,
};
```

Append these tests to `tests/settings_window_tests.rs`:

```rust
#[test]
fn settings_view_model_includes_auto_start_state() {
    let settings = AppSettings::default();

    let disabled = SettingsViewModel::from_settings_with_selected_and_auto_start(
        &settings,
        "google",
        false,
    );
    let enabled = SettingsViewModel::from_settings_with_selected_and_auto_start(
        &settings,
        "google",
        true,
    );

    assert!(!disabled.auto_start_enabled);
    assert!(enabled.auto_start_enabled);
}

#[test]
fn settings_view_model_includes_version_text() {
    let settings = AppSettings::default();

    let vm = SettingsViewModel::from(&settings);

    assert_eq!(vm.version_text, app_version_text());
    assert!(vm.version_text.starts_with("ait v"));
}

#[test]
fn settings_detail_update_carries_auto_start_state_without_storing_in_app_settings() {
    let mut settings = AppSettings::default();

    apply_settings_detail_update(
        &mut settings,
        SettingsProfileDetailUpdate {
            id: "google".to_string(),
            name: "Google".to_string(),
            provider: TranslatorProvider::Google,
            base_url: String::new(),
            model: String::new(),
            api_key: SettingsApiKeyUpdate::Preserve,
            timeout_secs: 0,
            hotkey: "Ctrl+Alt+E".to_string(),
            copy_wait_ms: 300,
            auto_start_enabled: true,
        },
    )
    .unwrap();

    assert_eq!(settings.hotkey, "Ctrl+Alt+E");
}
```

Add this test to `tests/config_tests.rs`:

```rust
#[test]
fn saved_settings_do_not_persist_auto_start_state() {
    let settings = AppSettings::default();
    let raw = serde_json::to_string_pretty(&settings).unwrap();

    assert!(!raw.contains("auto_start"));
    assert!(!raw.contains("startup"));
}
```

- [ ] **Step 2: Run focused tests to verify RED**

Run:

```powershell
cargo test --test settings_window_tests settings_view_model_includes_auto_start_state
```

Expected: compile failure because `from_settings_with_selected_and_auto_start`, `auto_start_enabled`, `version_text`, and `auto_start_enabled` update field do not exist.

- [ ] **Step 3: Extend settings view model and update struct**

Modify `src/ui/settings_window.rs`:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsViewModel {
    pub profiles: Vec<SettingsProfileListItem>,
    pub selected_profile: SettingsProfileDetail,
    pub hotkey: String,
    pub clipboard_capture_enabled: bool,
    pub copy_wait_ms: u64,
    pub auto_start_enabled: bool,
    pub version_text: String,
}
```

Modify `SettingsProfileDetailUpdate`:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsProfileDetailUpdate {
    pub id: String,
    pub name: String,
    pub provider: TranslatorProvider,
    pub base_url: String,
    pub model: String,
    pub api_key: SettingsApiKeyUpdate,
    pub timeout_secs: u64,
    pub hotkey: String,
    pub copy_wait_ms: u64,
    pub auto_start_enabled: bool,
}
```

Add the version helper near `api_key_placeholder_text`:

```rust
pub fn app_version_text() -> String {
    format!("ait v{}", env!("CARGO_PKG_VERSION"))
}
```

Replace the `impl SettingsViewModel` constructor block with:

```rust
impl SettingsViewModel {
    pub fn from_settings_with_selected_and_auto_start(
        settings: &AppSettings,
        selected_profile_id: &str,
        auto_start_enabled: bool,
    ) -> Self {
        let selected = settings
            .profile_by_id(selected_profile_id)
            .or_else(|| settings.profile_by_id(&settings.default_profile_id))
            .or_else(|| settings.translator_profiles.first())
            .expect("settings always contain profiles after normalization");
        let is_google = selected.provider == TranslatorProvider::Google;
        let (base_url, model, has_api_key, timeout_secs) = if is_google {
            (String::new(), String::new(), false, 0)
        } else {
            (
                selected.base_url.clone(),
                selected.model.clone(),
                selected.encrypted_api_key.is_some(),
                selected.timeout_secs,
            )
        };
        Self {
            profiles: settings
                .translator_profiles
                .iter()
                .map(|profile| SettingsProfileListItem {
                    id: profile.id.clone(),
                    label: profile_list_label(profile, profile.id == settings.default_profile_id),
                    selected: profile.id == selected.id,
                    default: profile.id == settings.default_profile_id,
                })
                .collect(),
            selected_profile: SettingsProfileDetail {
                id: selected.id.clone(),
                name: selected.name.clone(),
                provider: selected.provider,
                base_url,
                model,
                has_api_key,
                timeout_secs,
                built_in: selected.built_in,
                can_delete: !selected.built_in,
                name_editable: !is_google,
                network_fields_visible: !is_google,
                network_fields_enabled: !is_google,
                google_notice_visible: is_google,
            },
            hotkey: settings.hotkey.clone(),
            clipboard_capture_enabled: settings.clipboard_capture.enabled,
            copy_wait_ms: settings.clipboard_capture.copy_wait_ms,
            auto_start_enabled,
            version_text: app_version_text(),
        }
    }

    pub fn from_settings_with_selected(settings: &AppSettings, selected_profile_id: &str) -> Self {
        Self::from_settings_with_selected_and_auto_start(settings, selected_profile_id, false)
    }
}
```

Keep `apply_settings_detail_update` from writing to `AppSettings`; the new field is intentionally carried for the UI save pipeline but ignored by config mutation.

- [ ] **Step 4: Update existing tests that construct `SettingsProfileDetailUpdate`**

For every `SettingsProfileDetailUpdate { ... }` literal in `tests/settings_window_tests.rs`, add:

```rust
auto_start_enabled: false,
```

Example:

```rust
SettingsProfileDetailUpdate {
    id: "google".to_string(),
    name: "Google".to_string(),
    provider: TranslatorProvider::Google,
    base_url: String::new(),
    model: String::new(),
    api_key: SettingsApiKeyUpdate::Preserve,
    timeout_secs: 0,
    hotkey: "Ctrl+Alt+E".to_string(),
    copy_wait_ms: 300,
    auto_start_enabled: false,
}
```

- [ ] **Step 5: Run focused settings tests**

Run:

```powershell
cargo test --test settings_window_tests settings_view_model_includes_auto_start_state
cargo test --test settings_window_tests settings_view_model_includes_version_text
cargo test --test config_tests saved_settings_do_not_persist_auto_start_state
```

Expected: all focused tests pass.

- [ ] **Step 6: Commit Task 3**

```powershell
git add src/ui/settings_window.rs tests/settings_window_tests.rs tests/config_tests.rs
git commit -m "feat: add startup state to settings model"
```

---

### Task 4: Settings Window UI Controls

**Files:**
- Modify: `src/ui/settings_window.rs`
- Modify: `tests/settings_window_tests.rs`

- [ ] **Step 1: Add failing layout tests**

Append to `tests/settings_window_tests.rs`:

```rust
#[test]
fn settings_window_layout_places_auto_start_with_global_settings() {
    let layout = settings_window_layout();

    assert!(layout.auto_start.y > layout.hotkey.y);
    assert!(layout.auto_start.y < layout.separator.y);
    assert!(layout.version.y > layout.profile_list.y);
}
```

- [ ] **Step 2: Run layout test to verify RED**

Run:

```powershell
cargo test --test settings_window_tests settings_window_layout_places_auto_start_with_global_settings
```

Expected: compile failure because `SettingsWindowLayout` has no `auto_start` or `version` fields.

- [ ] **Step 3: Add layout fields and control IDs**

Modify `src/ui/settings_window.rs` near the existing IDs:

```rust
#[cfg(windows)]
const ID_AUTO_START: i32 = 3117;
#[cfg(windows)]
const ID_VERSION_LABEL: i32 = 3118;
```

Modify `SettingsWindowLayout`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingsWindowLayout {
    pub hotkey: SettingsControlRect,
    pub auto_start: SettingsControlRect,
    pub separator: SettingsControlRect,
    pub profile_list: SettingsControlRect,
    pub name: SettingsControlRect,
    pub version: SettingsControlRect,
}
```

Modify `settings_window_layout()`:

```rust
pub fn settings_window_layout() -> SettingsWindowLayout {
    SettingsWindowLayout {
        hotkey: SettingsControlRect {
            x: 118,
            y: 18,
            width: 180,
            height: 24,
        },
        auto_start: SettingsControlRect {
            x: 320,
            y: 18,
            width: 100,
            height: 24,
        },
        separator: SettingsControlRect {
            x: 18,
            y: 62,
            width: 668,
            height: 1,
        },
        profile_list: SettingsControlRect {
            x: 18,
            y: 100,
            width: 220,
            height: 228,
        },
        name: SettingsControlRect {
            x: 370,
            y: 100,
            width: 240,
            height: 24,
        },
        version: SettingsControlRect {
            x: 18,
            y: 420,
            width: 160,
            height: 22,
        },
    }
}
```

- [ ] **Step 4: Add checkbox helpers**

Add this helper near `create_button`:

```rust
#[cfg(windows)]
fn create_checkbox(
    parent: windows::Win32::Foundation::HWND,
    text: &str,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    id: i32,
) -> Result<windows::Win32::Foundation::HWND> {
    use windows::Win32::UI::WindowsAndMessaging::{BS_AUTOCHECKBOX, WINDOW_STYLE};
    create_control(
        parent,
        "BUTTON",
        text,
        x,
        y,
        width,
        height,
        id as isize,
        WINDOW_STYLE(BS_AUTOCHECKBOX as u32),
        true,
    )
}
```

Add checkbox read/write helpers near `read_control_text`:

```rust
#[cfg(windows)]
fn set_checkbox_checked(
    hwnd: windows::Win32::Foundation::HWND,
    id: i32,
    checked: bool,
) -> Result<()> {
    use windows::Win32::Foundation::{LPARAM, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{BM_SETCHECK, BST_CHECKED, BST_UNCHECKED, SendMessageW};

    let child = control(hwnd, id)?;
    let state = if checked { BST_CHECKED } else { BST_UNCHECKED };
    unsafe {
        let _ = SendMessageW(child, BM_SETCHECK, Some(WPARAM(state.0 as usize)), Some(LPARAM(0)));
    }
    Ok(())
}

#[cfg(windows)]
fn is_checkbox_checked(hwnd: windows::Win32::Foundation::HWND, id: i32) -> Result<bool> {
    use windows::Win32::Foundation::{LPARAM, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{BM_GETCHECK, BST_CHECKED, SendMessageW};

    let child = control(hwnd, id)?;
    let state = unsafe { SendMessageW(child, BM_GETCHECK, Some(WPARAM(0)), Some(LPARAM(0))) };
    Ok(state.0 as u32 == BST_CHECKED.0)
}
```

- [ ] **Step 5: Create controls in settings window**

In `SettingsWindow::open`, replace:

```rust
let view_model = SettingsViewModel::from(settings);
```

with:

```rust
let auto_start_enabled = crate::startup::is_auto_start_enabled().unwrap_or_else(|err| {
    tracing::warn!(error = %err, "read startup setting failed");
    false
});
let view_model =
    SettingsViewModel::from_settings_with_selected_and_auto_start(settings, &settings.default_profile_id, auto_start_enabled);
let layout = settings_window_layout();
```

Replace the hard-coded hotkey creation block:

```rust
create_static(hwnd, "快捷键", 18, 20, 90, 22)?;
let hotkey_edit =
    create_edit(hwnd, &view_model.hotkey, 118, 18, 180, 24, false, ID_HOTKEY)?;
set_hotkey_capture_mode(hotkey_edit)?;
create_static(hwnd, "", 18, 62, 668, 1)?;
```

with:

```rust
create_static(hwnd, "快捷键", 18, 20, 90, 22)?;
let hotkey_edit = create_edit(
    hwnd,
    &view_model.hotkey,
    layout.hotkey.x,
    layout.hotkey.y,
    layout.hotkey.width,
    layout.hotkey.height,
    false,
    ID_HOTKEY,
)?;
set_hotkey_capture_mode(hotkey_edit)?;
create_checkbox(
    hwnd,
    "开启自启",
    layout.auto_start.x,
    layout.auto_start.y,
    layout.auto_start.width,
    layout.auto_start.height,
    ID_AUTO_START,
)?;
set_checkbox_checked(hwnd, ID_AUTO_START, view_model.auto_start_enabled)?;
create_static(
    hwnd,
    "",
    layout.separator.x,
    layout.separator.y,
    layout.separator.width,
    layout.separator.height,
)?;
```

Before creating the save/cancel buttons, add:

```rust
create_static_with_id(
    hwnd,
    &view_model.version_text,
    layout.version.x,
    layout.version.y,
    layout.version.width,
    layout.version.height,
    ID_VERSION_LABEL,
)?;
```

- [ ] **Step 6: Run settings UI tests and compile check**

Run:

```powershell
cargo test --test settings_window_tests settings_window_layout_places_auto_start_with_global_settings
cargo check
```

Expected: layout test passes and compile succeeds.

- [ ] **Step 7: Commit Task 4**

```powershell
git add src/ui/settings_window.rs tests/settings_window_tests.rs
git commit -m "feat: add startup checkbox to settings window"
```

---

### Task 5: Save Pipeline Syncs Startup On Save

**Files:**
- Modify: `src/ui/settings_window.rs`
- Modify: `tests/settings_window_tests.rs`

- [ ] **Step 1: Add test for save model carrying startup choice**

Append to `tests/settings_window_tests.rs`:

```rust
#[test]
fn settings_detail_update_can_carry_enabled_auto_start_choice() {
    let update = SettingsProfileDetailUpdate {
        id: "google".to_string(),
        name: "Google".to_string(),
        provider: TranslatorProvider::Google,
        base_url: String::new(),
        model: String::new(),
        api_key: SettingsApiKeyUpdate::Preserve,
        timeout_secs: 0,
        hotkey: "Ctrl+Alt+E".to_string(),
        copy_wait_ms: 300,
        auto_start_enabled: true,
    };

    assert!(update.auto_start_enabled);
}
```

- [ ] **Step 2: Run test**

Run:

```powershell
cargo test --test settings_window_tests settings_detail_update_can_carry_enabled_auto_start_choice
```

Expected: pass if Task 3 is complete.

- [ ] **Step 3: Read checkbox and sync registry in save flow**

In `save_settings_from_window`, include `auto_start_enabled` in the update:

```rust
let auto_start_enabled = is_checkbox_checked(hwnd, ID_AUTO_START)?;
apply_settings_detail_update(
    settings,
    SettingsProfileDetailUpdate {
        id: profile_id,
        name: read_control_text(hwnd, ID_NAME)?,
        provider: existing_provider,
        base_url: read_control_text(hwnd, ID_BASE_URL)?,
        model: read_control_text(hwnd, ID_MODEL)?,
        api_key: api_key_update,
        timeout_secs: read_control_text(hwnd, ID_TIMEOUT)?
            .parse::<u64>()
            .unwrap_or(30),
        hotkey: read_control_text(hwnd, ID_HOTKEY)?,
        copy_wait_ms: settings.clipboard_capture.copy_wait_ms,
        auto_start_enabled,
    },
)?;
crate::startup::set_auto_start_enabled(auto_start_enabled)?;
```

Keep `crate::startup::set_auto_start_enabled(auto_start_enabled)?` before saving `settings.json`; this ensures a registry failure surfaces as a save failure and leaves the settings window open through the existing error path.

- [ ] **Step 4: Ensure profile reload preserves auto-start checkbox**

In `load_profile_into_window`, replace:

```rust
let vm = SettingsViewModel::from_settings_with_selected(settings, profile_id);
```

with:

```rust
let auto_start_enabled = crate::startup::is_auto_start_enabled().unwrap_or_else(|err| {
    tracing::warn!(error = %err, "read startup setting failed");
    false
});
let vm = SettingsViewModel::from_settings_with_selected_and_auto_start(
    settings,
    profile_id,
    auto_start_enabled,
);
```

After `set_control_text(hwnd, ID_HOTKEY, &vm.hotkey)?;`, add:

```rust
set_checkbox_checked(hwnd, ID_AUTO_START, vm.auto_start_enabled)?;
```

- [ ] **Step 5: Run focused tests and compile check**

Run:

```powershell
cargo test --test settings_window_tests settings_detail_update_can_carry_enabled_auto_start_choice
cargo check
```

Expected: test passes and compile succeeds.

- [ ] **Step 6: Commit Task 5**

```powershell
git add src/ui/settings_window.rs tests/settings_window_tests.rs
git commit -m "feat: sync startup setting on save"
```

---

### Task 6: Version Bump And Release Text

**Files:**
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`
- Modify: `README.md`

- [ ] **Step 1: Add failing version expectation**

Append to `tests/settings_window_tests.rs`:

```rust
#[test]
fn app_version_text_uses_v0_1_2() {
    assert_eq!(app_version_text(), "ait v0.1.2");
}
```

- [ ] **Step 2: Run test to verify RED**

Run:

```powershell
cargo test --test settings_window_tests app_version_text_uses_v0_1_2
```

Expected: fail while `Cargo.toml` still says `0.1.1`.

- [ ] **Step 3: Bump Cargo package version**

Modify `Cargo.toml`:

```toml
[package]
name = "ait"
version = "0.1.2"
edition = "2024"
```

Modify the `ait` package entry in `Cargo.lock`:

```toml
[[package]]
name = "ait"
version = "0.1.2"
```

- [ ] **Step 4: Update README download and release examples**

Replace `v0.1.1` examples in `README.md` with `v0.1.2`:

```text
ait-v0.1.2-setup.exe
ait-v0.1.2-windows.exe
```

Update the workflow and tag examples:

```powershell
git tag v0.1.2
git push origin v0.1.2
```

- [ ] **Step 5: Run version test**

Run:

```powershell
cargo test --test settings_window_tests app_version_text_uses_v0_1_2
```

Expected: test passes.

- [ ] **Step 6: Commit Task 6**

```powershell
git add Cargo.toml Cargo.lock README.md tests/settings_window_tests.rs
git commit -m "chore: bump version to 0.1.2"
```

---

### Task 7: Final Verification

**Files:**
- No planned source edits.

- [ ] **Step 1: Run full test suite**

Run:

```powershell
cargo test
```

Expected: all tests pass.

- [ ] **Step 2: Run release build**

Run:

```powershell
cargo build --release
```

Expected: release build succeeds.

- [ ] **Step 3: Verify release exe remains GUI subsystem**

Run:

```powershell
$path = 'target\release\ait.exe'
$bytes = [System.IO.File]::ReadAllBytes((Resolve-Path $path))
$peOffset = [BitConverter]::ToInt32($bytes, 0x3c)
$optionalHeaderOffset = $peOffset + 24
$subsystem = [BitConverter]::ToUInt16($bytes, $optionalHeaderOffset + 68)
$name = switch ($subsystem) { 2 { 'Windows GUI' } 3 { 'Windows Console' } default { "Unknown ($subsystem)" } }
"Subsystem=$subsystem ($name)"
```

Expected:

```text
Subsystem=2 (Windows GUI)
```

- [ ] **Step 4: Manual Windows smoke test**

Run the release executable:

```powershell
.\target\release\ait.exe
```

Manual checks:

- Right-click tray icon and open `设置`.
- Confirm the settings window shows `开启自启`.
- Confirm the settings window shows `ait v0.1.2`.
- Check `开启自启`, click `保存`, close settings, reopen settings, and confirm the checkbox is still checked.
- Uncheck `开启自启`, click `保存`, close settings, reopen settings, and confirm the checkbox is unchecked.
- Press `取消` after toggling the checkbox and confirm reopening settings shows the previous saved state.

- [ ] **Step 5: Inspect git status**

Run:

```powershell
git status --short --branch
```

Expected: clean working tree on `main`, ahead of `origin/main` by the implementation commits.

