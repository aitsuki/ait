# Hide Copy Wait Setting Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. Do not use superpowers:subagent-driven-development in this repository.

**Goal:** Remove the copy wait time control from the settings window while preserving the underlying `copy_wait_ms` configuration behavior.

**Architecture:** This is a scoped UI cleanup. The settings window stops creating and reading the copy wait edit control, while `AppSettings.clipboard_capture.copy_wait_ms` remains part of the config model and continues to drive capture timing.

**Tech Stack:** Rust, Win32 UI via `windows`, Cargo tests.

---

## File Structure

- Modify `tests/settings_window_tests.rs`: update behavior tests so saving settings preserves an existing copy wait value and layout no longer exposes a copy wait rectangle.
- Modify `src/ui/settings_window.rs`: remove the copy wait control from Win32 settings UI creation and save flow; update layout model accordingly.
- Leave `src/config.rs`, `src/capture.rs`, and `src/app.rs` unchanged so runtime behavior and config compatibility remain intact.

### Task 1: Update Settings Window Tests

**Files:**
- Modify: `tests/settings_window_tests.rs`

- [ ] **Step 1: Write the failing save-behavior test**

Change `settings_detail_update_saves_selected_profile_fields` so it proves saving detail fields preserves the existing copy wait value instead of updating it from user input.

```rust
#[test]
fn settings_detail_update_saves_selected_profile_fields() {
    let mut settings = AppSettings::default();
    settings.clipboard_capture.copy_wait_ms = 425;
    let id = settings.add_custom_profile().id;
    settings.profile_by_id_mut(&id).unwrap().provider = TranslatorProvider::OpenAi;

    apply_settings_detail_update(
        &mut settings,
        SettingsProfileDetailUpdate {
            id: id.clone(),
            name: "Work GPT".to_string(),
            provider: TranslatorProvider::OpenAi,
            base_url: "https://example.test/v1".to_string(),
            model: "gpt-test".to_string(),
            api_key: Some("secret".to_string()),
            timeout_secs: 45,
            hotkey: "Ctrl+Alt+T".to_string(),
            copy_wait_ms: settings.clipboard_capture.copy_wait_ms,
        },
    )
    .unwrap();

    let profile = settings.profile_by_id(&id).unwrap();
    assert_eq!(profile.name, "Work GPT");
    assert_eq!(profile.provider, TranslatorProvider::OpenAi);
    assert_eq!(profile.base_url, "https://example.test/v1");
    assert_eq!(profile.model, "gpt-test");
    assert_eq!(profile.encrypted_api_key.as_deref(), Some("secret"));
    assert_eq!(profile.timeout_secs, 45);
    assert_eq!(settings.hotkey, "Ctrl+Alt+T");
    assert_eq!(settings.clipboard_capture.copy_wait_ms, 425);
}
```

- [ ] **Step 2: Write the failing layout test**

Change `settings_window_layout_places_global_settings_above_profiles` so it no longer refers to `layout.copy_wait`.

```rust
#[test]
fn settings_window_layout_places_global_settings_above_profiles() {
    let layout = settings_window_layout();

    assert!(layout.hotkey.y < layout.separator.y);
    assert!(layout.profile_list.y > layout.separator.y);
    assert!(layout.name.y > layout.separator.y);
}
```

- [ ] **Step 3: Run tests to verify RED**

Run:

```powershell
cargo test settings_window
```

Expected: compilation fails because `SettingsWindowLayout` still exposes or tests still depend on the old copy wait layout shape, or the save behavior still treats copy wait as an editable field.

### Task 2: Remove Copy Wait UI Surface

**Files:**
- Modify: `src/ui/settings_window.rs`

- [ ] **Step 1: Remove copy wait control creation**

In `SettingsWindow::open`, delete this block:

```rust
create_static(hwnd, "复制等待毫秒", 318, 20, 100, 22)?;
create_edit(
    hwnd,
    &view_model.copy_wait_ms.to_string(),
    430,
    18,
    90,
    24,
    false,
    ID_COPY_WAIT,
)?;
```

- [ ] **Step 2: Preserve copy wait on save**

In `save_settings_from_window`, replace the `copy_wait_ms` field assignment with the existing settings value:

```rust
copy_wait_ms: settings.clipboard_capture.copy_wait_ms,
```

- [ ] **Step 3: Stop refreshing the removed control**

In `load_profile_into_window`, delete:

```rust
set_control_text(hwnd, ID_COPY_WAIT, &vm.copy_wait_ms.to_string())?;
```

- [ ] **Step 4: Remove copy wait from layout model**

Change `SettingsWindowLayout` to remove the field:

```rust
pub struct SettingsWindowLayout {
    pub hotkey: SettingsControlRect,
    pub separator: SettingsControlRect,
    pub profile_list: SettingsControlRect,
    pub name: SettingsControlRect,
}
```

Change `settings_window_layout()` to remove the `copy_wait` initializer and leave the existing `hotkey`, `separator`, `profile_list`, and `name` rectangles unchanged.

- [ ] **Step 5: Run focused tests to verify GREEN**

Run:

```powershell
cargo test settings_window
```

Expected: all settings window tests pass.

### Task 3: Full Verification

**Files:**
- No code changes unless verification exposes a regression.

- [ ] **Step 1: Run all tests**

Run:

```powershell
cargo test
```

Expected: all tests pass.

- [ ] **Step 2: Review remaining references**

Run:

```powershell
rg -n "复制等待|ID_COPY_WAIT|copy_wait" src tests
```

Expected: no settings-window UI references to `复制等待` or `ID_COPY_WAIT`; `copy_wait_ms` may remain in config, app, capture, and tests that verify config compatibility.

## Self-Review

- Spec coverage: The plan removes the UI control, preserves the config value on save, leaves runtime capture behavior unchanged, and updates tests.
- Placeholder scan: No placeholders remain.
- Type consistency: All referenced Rust types and functions already exist in `src/ui/settings_window.rs` and `tests/settings_window_tests.rs`.
