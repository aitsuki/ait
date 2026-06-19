# Edit Hotkey Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:executing-plans` to implement this plan task-by-task. `superpowers:subagent-driven-development` is prohibited by this repository's `AGENTS.md`. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a safe editable global hotkey flow: users press a supported key combination in settings, the app saves a normalized shortcut, and the running app switches to it immediately with clear errors and rollback.

**Architecture:** Keep `src/hotkey.rs` as the parsing, display, and validation boundary. Add small pure helpers in `src/ui/settings_window.rs` for key capture and save normalization so unit tests can cover most behavior without a Windows UI. Add a pure runtime hotkey transition helper in `src/app.rs`; the Windows message loop uses it to register the new hotkey, keep the old registration on failure, and roll back persisted config.

**Tech Stack:** Rust, Win32 via the `windows` crate, existing `cargo test` integration tests.

---

## File Structure

- Modify `src/hotkey.rs`: add modifier presence checks and make numeric keys part of the documented supported key range.
- Modify `tests/hotkey_tests.rs`: add parse/display tests for numeric keys and rejection tests for shortcuts without modifiers.
- Modify `src/ui/settings_window.rs`: add pure capture helpers, normalize hotkey during settings update, make the Windows hotkey edit control read-only, and subclass it to capture key presses.
- Modify `tests/settings_window_tests.rs`: test capture helper behavior and save-time normalization/rejection.
- Modify `src/app.rs`: add pure runtime hotkey transition helper and wire Windows `WM_SETTINGS_SAVED` to re-register hotkeys with rollback and user-visible errors.
- Modify `tests/workflow_tests.rs`: test the runtime transition helper for success, no-op, and failed registration rollback.

---

### Task 1: Harden `Hotkey` Parsing

**Files:**
- Modify: `src/hotkey.rs`
- Test: `tests/hotkey_tests.rs`

- [ ] **Step 1: Add failing hotkey parser tests**

Add these tests to `tests/hotkey_tests.rs`:

```rust
#[test]
fn parses_numeric_hotkey() {
    let hotkey = "Ctrl+Shift+1".parse::<Hotkey>().unwrap();

    assert_eq!(
        hotkey.modifiers,
        Modifiers {
            ctrl: true,
            alt: false,
            shift: true,
            win: false
        }
    );
    assert_eq!(hotkey.key, KeyCode::Char('1'));
    assert_eq!(hotkey.to_string(), "Ctrl+Shift+1");
}

#[test]
fn rejects_shortcut_without_modifier() {
    let err = "E".parse::<Hotkey>().unwrap_err().to_string();

    assert!(err.contains("至少包含一个修饰键"));
}
```

- [ ] **Step 2: Run tests to verify failure**

Run:

```powershell
cargo test --test hotkey_tests
```

Expected: `rejects_shortcut_without_modifier` fails because `E` is currently accepted.

- [ ] **Step 3: Add modifier helper and reject unmodified shortcuts**

In `src/hotkey.rs`, add an inherent impl near the `Modifiers` definition:

```rust
impl Modifiers {
    pub fn any(self) -> bool {
        self.ctrl || self.alt || self.shift || self.win
    }
}
```

Then in `impl FromStr for Hotkey`, after the existing `key` extraction and before `Ok(Self { modifiers, key })`, add:

```rust
        if !modifiers.any() {
            return Err(AppError::Hotkey(
                "快捷键必须至少包含一个修饰键".to_string(),
            ));
        }
```

- [ ] **Step 4: Run tests to verify pass**

Run:

```powershell
cargo test --test hotkey_tests
```

Expected: all `hotkey_tests` pass.

- [ ] **Step 5: Commit**

```powershell
git add src/hotkey.rs tests/hotkey_tests.rs
git commit -m "fix: require hotkey modifiers"
```

---

### Task 2: Add Settings Hotkey Capture Helpers

**Files:**
- Modify: `src/ui/settings_window.rs`
- Test: `tests/settings_window_tests.rs`

- [ ] **Step 1: Add failing capture helper tests**

Update the import block in `tests/settings_window_tests.rs` to include `hotkey_capture_text`.

Add these tests:

```rust
#[test]
fn hotkey_capture_text_formats_supported_combinations() {
    let ctrl_alt = ait::hotkey::Modifiers {
        ctrl: true,
        alt: true,
        shift: false,
        win: false,
    };
    let ctrl_shift = ait::hotkey::Modifiers {
        ctrl: true,
        alt: false,
        shift: true,
        win: false,
    };

    assert_eq!(hotkey_capture_text(0x54, ctrl_alt).as_deref(), Some("Ctrl+Alt+T"));
    assert_eq!(hotkey_capture_text(0x31, ctrl_shift).as_deref(), Some("Ctrl+Shift+1"));
    assert_eq!(hotkey_capture_text(0x70, ctrl_alt).as_deref(), Some("Ctrl+Alt+F1"));
    assert_eq!(hotkey_capture_text(0x87, ctrl_alt).as_deref(), Some("Ctrl+Alt+F24"));
}

#[test]
fn hotkey_capture_text_ignores_incomplete_or_unsupported_keys() {
    let none = ait::hotkey::Modifiers {
        ctrl: false,
        alt: false,
        shift: false,
        win: false,
    };
    let ctrl = ait::hotkey::Modifiers {
        ctrl: true,
        alt: false,
        shift: false,
        win: false,
    };

    assert_eq!(hotkey_capture_text(0x54, none), None);
    assert_eq!(hotkey_capture_text(0x11, ctrl), None);
    assert_eq!(hotkey_capture_text(0x1B, ctrl), None);
    assert_eq!(hotkey_capture_text(0xBA, ctrl), None);
}
```

- [ ] **Step 2: Run tests to verify failure**

Run:

```powershell
cargo test --test settings_window_tests hotkey_capture_text
```

Expected: compile failure because `hotkey_capture_text` does not exist.

- [ ] **Step 3: Implement pure capture helper**

In `src/ui/settings_window.rs`, add this public helper near the other public settings helpers:

```rust
pub fn hotkey_capture_text(vk: u32, modifiers: crate::hotkey::Modifiers) -> Option<String> {
    if !modifiers.any() {
        return None;
    }

    let key = match vk {
        0x30..=0x39 => crate::hotkey::KeyCode::Char(char::from_u32(vk)?),
        0x41..=0x5A => crate::hotkey::KeyCode::Char(char::from_u32(vk)?),
        0x70..=0x87 => crate::hotkey::KeyCode::Function((vk - 0x70 + 1) as u8),
        0x10 | 0x11 | 0x12 | 0x1B | 0x5B | 0x5C => return None,
        _ => return None,
    };

    Some(
        crate::hotkey::Hotkey {
            modifiers,
            key,
        }
        .to_string(),
    )
}
```

- [ ] **Step 4: Run tests to verify pass**

Run:

```powershell
cargo test --test settings_window_tests hotkey_capture_text
```

Expected: both new capture helper tests pass.

- [ ] **Step 5: Wire the Windows edit control as read-only**

In `src/ui/settings_window.rs`, change the hotkey control creation in `SettingsWindow::open` from:

```rust
create_edit(hwnd, &view_model.hotkey, 118, 18, 180, 24, false, ID_HOTKEY)?;
```

to:

```rust
let hotkey_edit = create_edit(hwnd, &view_model.hotkey, 118, 18, 180, 24, false, ID_HOTKEY)?;
set_hotkey_capture_mode(hotkey_edit)?;
```

Add a Windows-only helper near `set_api_key_password_mode`:

```rust
#[cfg(windows)]
fn set_hotkey_capture_mode(edit: windows::Win32::Foundation::HWND) -> Result<()> {
    use windows::Win32::Foundation::{LPARAM, WPARAM};
    use windows::Win32::UI::Controls::SetWindowSubclass;
    use windows::Win32::UI::WindowsAndMessaging::{SendMessageW, EM_SETREADONLY};

    unsafe {
        SendMessageW(edit, EM_SETREADONLY, Some(WPARAM(1)), Some(LPARAM(0)));
        SetWindowSubclass(edit, Some(hotkey_edit_subclass_proc), 1, 0)
            .map_err(|err| AppError::Windows(format!("安装快捷键捕获失败: {err}")))?;
    }
    Ok(())
}
```

This step prevents free typing and installs the subclass hook that Task 3 fills in.

- [ ] **Step 6: Run focused tests**

Run:

```powershell
cargo test --test settings_window_tests hotkey_capture_text
```

Expected: tests still pass.

- [ ] **Step 7: Commit**

```powershell
git add src/ui/settings_window.rs tests/settings_window_tests.rs
git commit -m "feat: add settings hotkey capture helper"
```

---

### Task 3: Normalize and Validate Hotkey on Settings Save

**Files:**
- Modify: `src/ui/settings_window.rs`
- Test: `tests/settings_window_tests.rs`

- [ ] **Step 1: Add failing save normalization tests**

Add these tests to `tests/settings_window_tests.rs`:

```rust
#[test]
fn settings_detail_update_normalizes_hotkey_before_saving() {
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
            hotkey: " shift + ctrl + 1 ".to_string(),
            copy_wait_ms: 300,
        },
    )
    .unwrap();

    assert_eq!(settings.hotkey, "Ctrl+Shift+1");
}

#[test]
fn settings_detail_update_rejects_invalid_hotkey() {
    let mut settings = AppSettings::default();

    let err = apply_settings_detail_update(
        &mut settings,
        SettingsProfileDetailUpdate {
            id: "google".to_string(),
            name: "Google".to_string(),
            provider: TranslatorProvider::Google,
            base_url: String::new(),
            model: String::new(),
            api_key: SettingsApiKeyUpdate::Preserve,
            timeout_secs: 0,
            hotkey: "not-a-hotkey".to_string(),
            copy_wait_ms: 300,
        },
    )
    .unwrap_err();

    assert!(err.to_string().contains("快捷键"));
    assert_eq!(settings.hotkey, "Ctrl+Alt+E");
}
```

- [ ] **Step 2: Run tests to verify failure**

Run:

```powershell
cargo test --test settings_window_tests settings_detail_update_normalizes_hotkey_before_saving settings_detail_update_rejects_invalid_hotkey
```

Expected: normalization test fails because the raw string is saved.

- [ ] **Step 3: Normalize before mutating settings**

In `src/ui/settings_window.rs`, change the top of `apply_settings_detail_update` from:

```rust
pub fn apply_settings_detail_update(
    settings: &mut AppSettings,
    update: SettingsProfileDetailUpdate,
) -> Result<()> {
    settings.hotkey = update.hotkey;
```

to:

```rust
pub fn apply_settings_detail_update(
    settings: &mut AppSettings,
    update: SettingsProfileDetailUpdate,
) -> Result<()> {
    let hotkey = update.hotkey.parse::<crate::hotkey::Hotkey>()?.to_string();
    settings.hotkey = hotkey;
```

This keeps invalid hotkeys from partially mutating `settings.hotkey`.

- [ ] **Step 4: Run tests to verify pass**

Run:

```powershell
cargo test --test settings_window_tests settings_detail_update_normalizes_hotkey_before_saving settings_detail_update_rejects_invalid_hotkey
```

Expected: both tests pass.

- [ ] **Step 5: Add Windows keydown capture in the hotkey edit subclass**

In `src/ui/settings_window.rs`, add the edit-control subclass procedure used by `set_hotkey_capture_mode`. The parent window will not reliably receive `WM_KEYDOWN` for the child edit control, so capture must happen in the child control's message path.

Add this near `set_hotkey_capture_mode`:

```rust
#[cfg(windows)]
unsafe extern "system" fn hotkey_edit_subclass_proc(
    hwnd: windows::Win32::Foundation::HWND,
    msg: u32,
    wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
    subclass_id: usize,
    ref_data: usize,
) -> windows::Win32::Foundation::LRESULT {
    use windows::Win32::Foundation::LRESULT;
    use windows::Win32::UI::Input::KeyboardAndMouse::GetKeyState;
    use windows::Win32::UI::WindowsAndMessaging::{
        WM_KEYDOWN, WM_NCDESTROY, WM_SYSKEYDOWN, WM_CHAR, WM_PASTE, WM_CLEAR, WM_CUT,
    };
    use windows::Win32::UI::Controls::{DefSubclassProc, RemoveWindowSubclass};

    if msg == WM_NCDESTROY {
        let _ = RemoveWindowSubclass(hwnd, Some(hotkey_edit_subclass_proc), subclass_id);
        return DefSubclassProc(hwnd, msg, wparam, lparam);
    }

    if msg == WM_CHAR || msg == WM_PASTE || msg == WM_CLEAR || msg == WM_CUT {
        return LRESULT(0);
    }

    if msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN {
        let modifiers = crate::hotkey::Modifiers {
            ctrl: GetKeyState(0x11) < 0,
            alt: GetKeyState(0x12) < 0,
            shift: GetKeyState(0x10) < 0,
            win: GetKeyState(0x5B) < 0 || GetKeyState(0x5C) < 0,
        };

        if let Some(text) = hotkey_capture_text(wparam.0 as u32, modifiers) {
            let text = wide(&text);
            let _ = windows::Win32::UI::WindowsAndMessaging::SetWindowTextW(
                hwnd,
                windows::core::PCWSTR(text.as_ptr()),
            );
            return LRESULT(0);
        }
    }

    DefSubclassProc(hwnd, msg, wparam, lparam)
}
```

- [ ] **Step 6: Run settings tests**

Run:

```powershell
cargo test --test settings_window_tests
```

Expected: all `settings_window_tests` pass.

- [ ] **Step 7: Commit**

```powershell
git add src/ui/settings_window.rs tests/settings_window_tests.rs
git commit -m "feat: validate settings hotkey saves"
```

---

### Task 4: Re-register Hotkey at Runtime With Rollback

**Files:**
- Modify: `src/app.rs`
- Test: `tests/workflow_tests.rs`

- [ ] **Step 1: Add failing pure transition tests**

Update the `use ait::app::{ ... }` import in `tests/workflow_tests.rs` to include `HotkeyRegistrationUpdate` and `hotkey_registration_update`.

Add these tests near the existing hotkey tests:

```rust
#[test]
fn hotkey_registration_update_noops_when_hotkey_is_unchanged() {
    assert_eq!(
        hotkey_registration_update("Ctrl+Alt+E", "Ctrl+Alt+E", Ok(())),
        HotkeyRegistrationUpdate::Unchanged
    );
}

#[test]
fn hotkey_registration_update_accepts_changed_registered_hotkey() {
    assert_eq!(
        hotkey_registration_update("Ctrl+Alt+E", "Ctrl+Alt+T", Ok(())),
        HotkeyRegistrationUpdate::Changed {
            hotkey: "Ctrl+Alt+T".to_string()
        }
    );
}

#[test]
fn hotkey_registration_update_keeps_old_hotkey_when_registration_fails() {
    assert_eq!(
        hotkey_registration_update(
            "Ctrl+Alt+E",
            "Ctrl+Alt+T",
            Err("注册快捷键失败: already registered".to_string())
        ),
        HotkeyRegistrationUpdate::Rejected {
            rollback_hotkey: "Ctrl+Alt+E".to_string(),
            message: "快捷键注册失败，请换一个组合键；当前仍使用原来的快捷键。注册快捷键失败: already registered".to_string()
        }
    );
}
```

- [ ] **Step 2: Run tests to verify failure**

Run:

```powershell
cargo test --test workflow_tests hotkey_registration_update
```

Expected: compile failure because the enum and function do not exist.

- [ ] **Step 3: Add pure runtime transition helper**

In `src/app.rs`, near `HotkeyAction`, add:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HotkeyRegistrationUpdate {
    Unchanged,
    Changed { hotkey: String },
    Rejected { rollback_hotkey: String, message: String },
}

pub fn hotkey_registration_update(
    current_hotkey: &str,
    next_hotkey: &str,
    registration_result: std::result::Result<(), String>,
) -> HotkeyRegistrationUpdate {
    if current_hotkey == next_hotkey {
        return HotkeyRegistrationUpdate::Unchanged;
    }

    match registration_result {
        Ok(()) => HotkeyRegistrationUpdate::Changed {
            hotkey: next_hotkey.to_string(),
        },
        Err(error) => HotkeyRegistrationUpdate::Rejected {
            rollback_hotkey: current_hotkey.to_string(),
            message: format!(
                "快捷键注册失败，请换一个组合键；当前仍使用原来的快捷键。{error}"
            ),
        },
    }
}
```

- [ ] **Step 4: Run tests to verify pass**

Run:

```powershell
cargo test --test workflow_tests hotkey_registration_update
```

Expected: the three new tests pass.

- [ ] **Step 5: Wire Windows `WM_SETTINGS_SAVED`**

In `src/app.rs`, change:

```rust
    let _registered = RegisteredHotkey::register(1, hotkey)?;
```

to:

```rust
    let mut _registered_hotkey = RegisteredHotkey::register(1, hotkey)?;
    let mut registered_hotkey_id = 1;
    let mut registered_hotkey_text = hotkey.to_string();
```

Replace the `WM_SETTINGS_SAVED` branch with logic shaped like this:

```rust
            } else if msg.message == crate::ui::settings_window::WM_SETTINGS_SAVED {
                match SettingsStore::new(settings_dir.clone()).load() {
                    Ok(mut settings) => {
                        let next_hotkey_text = settings.hotkey.clone();
                        if next_hotkey_text != registered_hotkey_text {
                            match next_hotkey_text.parse::<Hotkey>() {
                                Ok(next_hotkey) => {
                                    let next_hotkey_id =
                                        if registered_hotkey_id == 1 { 2 } else { 1 };
                                    match RegisteredHotkey::register(next_hotkey_id, next_hotkey) {
                                        Ok(next_registered) => {
                                            _registered_hotkey = next_registered;
                                            registered_hotkey_id = next_hotkey_id;
                                            registered_hotkey_text = next_hotkey.to_string();
                                            settings.hotkey = registered_hotkey_text.clone();
                                            runtime_state.replace_settings(settings);
                                        }
                                        Err(err) => {
                                            settings.hotkey = registered_hotkey_text.clone();
                                            if let Err(save_err) =
                                                SettingsStore::new(settings_dir.clone()).save(&settings)
                                            {
                                                tracing::warn!(error = %save_err, "rollback hotkey save failed");
                                            }
                                            show_runtime_message(
                                                translation_window.hwnd(),
                                                "快捷键注册失败",
                                                &format!(
                                                    "快捷键注册失败，请换一个组合键；当前仍使用原来的快捷键。{err}"
                                                ),
                                            );
                                            runtime_state.replace_settings(settings);
                                        }
                                    }
                                }
                                Err(err) => {
                                    settings.hotkey = registered_hotkey_text.clone();
                                    if let Err(save_err) =
                                        SettingsStore::new(settings_dir.clone()).save(&settings)
                                    {
                                        tracing::warn!(error = %save_err, "rollback invalid hotkey save failed");
                                    }
                                    show_runtime_message(
                                        translation_window.hwnd(),
                                        "快捷键注册失败",
                                        &format!(
                                            "快捷键无效，当前仍使用原来的快捷键。{err}"
                                        ),
                                    );
                                    runtime_state.replace_settings(settings);
                                }
                            }
                        } else {
                            runtime_state.replace_settings(settings);
                        }
                        let _ = translation_window.refresh_profiles(
                            runtime_state.settings(),
                            runtime_state.active_profile_id(),
                        );
                    }
                    Err(err) => tracing::warn!(error = %err, "reload settings failed"),
                }
```

The alternating `1`/`2` IDs matter: the app registers the new shortcut before dropping the old registration, so a failed registration leaves the old shortcut active. The message loop already treats every `WM_HOTKEY` as the translate command, so it does not need to branch on `wParam`.

Add this Windows helper near `handle_app_command`:

```rust
#[cfg(windows)]
fn show_runtime_message(
    owner_hwnd: windows::Win32::Foundation::HWND,
    caption: &str,
    text: &str,
) {
    let caption = wide(caption);
    let text = wide(text);
    unsafe {
        let _ = windows::Win32::UI::WindowsAndMessaging::MessageBoxW(
            Some(owner_hwnd),
            windows::core::PCWSTR(text.as_ptr()),
            windows::core::PCWSTR(caption.as_ptr()),
            windows::Win32::UI::WindowsAndMessaging::MB_OK,
        );
    }
}

#[cfg(windows)]
fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(Some(0)).collect()
}
```

- [ ] **Step 6: Run focused tests**

Run:

```powershell
cargo test --test workflow_tests hotkey_registration_update
```

Expected: the runtime transition tests pass.

- [ ] **Step 7: Run all tests**

Run:

```powershell
cargo test
```

Expected: all tests pass.

- [ ] **Step 8: Commit**

```powershell
git add src/app.rs tests/workflow_tests.rs
git commit -m "feat: reload hotkey after settings save"
```

---

### Task 5: Manual Windows Verification

**Files:**
- No source edits expected.

- [ ] **Step 1: Build the app**

Run:

```powershell
cargo build
```

Expected: build finishes successfully.

- [ ] **Step 2: Launch the app**

Run:

```powershell
cargo run
```

Expected: tray app starts and registers the configured hotkey.

- [ ] **Step 3: Verify capture-only settings field**

Open settings from the tray menu. Click the shortcut field and press `Ctrl+Alt+T`.

Expected: the field displays `Ctrl+Alt+T`. Typing letters without modifiers, pressing Backspace, pressing Delete, and pasting text do not replace the field with arbitrary text.

- [ ] **Step 4: Verify immediate runtime switch**

Save settings. Select text in another application and press `Ctrl+Alt+T`.

Expected: translation starts without restarting the app. The old `Ctrl+Alt+E` no longer triggers translation.

- [ ] **Step 5: Verify conflict error and rollback**

Set the field to a known unavailable or reserved shortcut, save, and observe the message.

Expected: a message says the new shortcut failed and the old shortcut is still used. Reopen settings and confirm the shortcut field still shows the old working shortcut.
