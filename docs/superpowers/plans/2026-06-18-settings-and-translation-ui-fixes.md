# Settings And Translation UI Fixes Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. `superpowers:subagent-driven-development` is forbidden by this repository's `AGENTS.md`.

**Goal:** Fix the settings window interaction problems, restore the translation profile dropdown, and move blocking translation work off the UI thread.

**Architecture:** Keep the existing Rust/Win32 structure and add small, testable view-model/action helpers before changing platform-specific UI code. Settings window logic remains in `src/ui/settings_window.rs`; translation window layout remains in `src/ui/translate_window.rs`; async translation orchestration is added to `src/app.rs` with UI updates delivered back to the main message loop.

**Tech Stack:** Rust, Cargo tests, Win32 API through the `windows` crate, existing `AppSettings`, `TranslationWorkflow`, and translator abstractions.

---

## File Structure

- Modify `src/ui/settings_window.rs`: settings view-model fields, profile detail flags, list labels, save behavior, Win32 control layout, delete button enablement, Google-only visibility.
- Modify `tests/settings_window_tests.rs`: tests for labels, Google readonly/network visibility flags, save action semantics, provider preservation.
- Modify `src/ui/translate_window.rs`: combobox layout model, combobox move height, optional loading state helper for async results.
- Modify `tests/workflow_tests.rs`: layout tests for combobox dropdown height and async translation action/state helpers.
- Modify `src/app.rs`: introduce async translation request/result types and spawn blocking workflow on a background thread; handle completion messages in the main loop.

## Task 1: Settings View Model Flags And Labels

**Files:**
- Modify: `src/ui/settings_window.rs`
- Test: `tests/settings_window_tests.rs`

- [ ] **Step 1: Write failing tests for list labels and detail flags**

Add these tests to `tests/settings_window_tests.rs`:

```rust
#[test]
fn settings_view_model_does_not_show_builtin_label() {
    let settings = AppSettings::default();

    let vm = SettingsViewModel::from(&settings);

    let google = vm.profiles.iter().find(|item| item.id == "google").unwrap();
    assert_eq!(google.label, "Google（默认）");
    let openai = vm.profiles.iter().find(|item| item.id == "openai").unwrap();
    assert_eq!(openai.label, "OpenAI");
    assert!(!vm.profiles.iter().any(|item| item.label.contains("内置")));
}

#[test]
fn google_profile_detail_is_readonly_and_hides_network_fields() {
    let settings = AppSettings::default();

    let vm = SettingsViewModel::from_settings_with_selected(&settings, "google");

    assert_eq!(vm.selected_profile.id, "google");
    assert!(!vm.selected_profile.can_delete);
    assert!(!vm.selected_profile.name_editable);
    assert!(!vm.selected_profile.network_fields_visible);
    assert!(vm.selected_profile.google_notice_visible);
}

#[test]
fn non_google_profile_detail_is_editable_and_shows_network_fields() {
    let settings = AppSettings::default();

    let vm = SettingsViewModel::from_settings_with_selected(&settings, "openai");

    assert_eq!(vm.selected_profile.id, "openai");
    assert!(!vm.selected_profile.can_delete);
    assert!(vm.selected_profile.name_editable);
    assert!(vm.selected_profile.network_fields_visible);
    assert!(!vm.selected_profile.google_notice_visible);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```powershell
cargo test --test settings_window_tests settings_view_model_does_not_show_builtin_label google_profile_detail_is_readonly_and_hides_network_fields non_google_profile_detail_is_editable_and_shows_network_fields
```

Expected: tests fail to compile because `name_editable`, `network_fields_visible`, and `google_notice_visible` do not exist, and the old label still contains `内置`.

- [ ] **Step 3: Update `SettingsProfileDetail` and view-model construction**

In `src/ui/settings_window.rs`, change `SettingsProfileDetail` to include the new flags and keep `network_fields_enabled` temporarily for compatibility until UI code is updated:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsProfileDetail {
    pub id: String,
    pub name: String,
    pub provider: TranslatorProvider,
    pub base_url: String,
    pub model: String,
    pub has_api_key: bool,
    pub timeout_secs: u64,
    pub built_in: bool,
    pub can_delete: bool,
    pub name_editable: bool,
    pub network_fields_visible: bool,
    pub network_fields_enabled: bool,
    pub google_notice_visible: bool,
}
```

In `SettingsViewModel::from_settings_with_selected`, compute flags:

```rust
let is_google = selected.provider == TranslatorProvider::Google;
```

Then construct `selected_profile` with:

```rust
selected_profile: SettingsProfileDetail {
    id: selected.id.clone(),
    name: selected.name.clone(),
    provider: selected.provider,
    base_url: selected.base_url.clone(),
    model: selected.model.clone(),
    has_api_key: selected.encrypted_api_key.is_some(),
    timeout_secs: selected.timeout_secs,
    built_in: selected.built_in,
    can_delete: !selected.built_in,
    name_editable: !is_google,
    network_fields_visible: !is_google,
    network_fields_enabled: !is_google,
    google_notice_visible: is_google,
},
```

- [ ] **Step 4: Remove built-in text from profile labels**

Replace `profile_list_label` in `src/ui/settings_window.rs` with:

```rust
fn profile_list_label(profile: &crate::config::TranslatorProfile, is_default: bool) -> String {
    if is_default {
        format!("{}（默认）", profile.name)
    } else {
        profile.name.clone()
    }
}
```

- [ ] **Step 5: Update existing label tests**

In `tests/settings_window_tests.rs`, update old assertions that expected `内置`:

```rust
assert!(vm.profiles.iter().any(|item| item.label == "Google（默认）"));
assert!(vm.profiles.iter().any(|item| item.label == "DeepSeek"));
```

And:

```rust
assert_eq!(google.label, "Google（默认）");
assert_eq!(openai.label, "OpenAI");
```

- [ ] **Step 6: Run settings tests**

Run:

```powershell
cargo test --test settings_window_tests
```

Expected: all settings window tests pass.

- [ ] **Step 7: Commit**

```powershell
git add src\ui\settings_window.rs tests\settings_window_tests.rs
git commit -m "fix: simplify settings profile labels"
```

## Task 2: Preserve Provider During Settings Detail Save

**Files:**
- Modify: `src/ui/settings_window.rs`
- Test: `tests/settings_window_tests.rs`

- [ ] **Step 1: Write failing test for provider preservation**

Add this test to `tests/settings_window_tests.rs`:

```rust
#[test]
fn settings_detail_update_preserves_existing_provider() {
    let mut settings = AppSettings::default();
    let id = settings.add_custom_profile().id;
    settings.profile_by_id_mut(&id).unwrap().provider = TranslatorProvider::DeepSeek;

    apply_settings_detail_update(
        &mut settings,
        SettingsProfileDetailUpdate {
            id: id.clone(),
            name: "DeepSeek Work".to_string(),
            provider: TranslatorProvider::Google,
            base_url: "https://api.deepseek.com/v1".to_string(),
            model: "deepseek-chat".to_string(),
            api_key: None,
            timeout_secs: 30,
            hotkey: "Ctrl+Alt+E".to_string(),
            copy_wait_ms: 300,
        },
    )
    .unwrap();

    assert_eq!(
        settings.profile_by_id(&id).unwrap().provider,
        TranslatorProvider::DeepSeek
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test settings_window_tests settings_detail_update_preserves_existing_provider
```

Expected: test fails because `apply_settings_detail_update` currently assigns `profile.provider = update.provider`.

- [ ] **Step 3: Stop assigning provider from the UI update**

In `src/ui/settings_window.rs`, remove this line from `apply_settings_detail_update`:

```rust
profile.provider = update.provider;
```

Leave `SettingsProfileDetailUpdate.provider` in place for now to keep the change small; it will no longer be trusted by save logic.

- [ ] **Step 4: Ensure fallback name uses existing provider**

Keep this code in `apply_settings_detail_update`:

```rust
if profile.name.is_empty() {
    profile.name = profile.provider.display_name().to_string();
}
```

- [ ] **Step 5: Run settings tests**

Run:

```powershell
cargo test --test settings_window_tests
```

Expected: all settings window tests pass.

- [ ] **Step 6: Commit**

```powershell
git add src\ui\settings_window.rs tests\settings_window_tests.rs
git commit -m "fix: preserve translator provider in settings save"
```

## Task 3: Settings Save Action And Window Layout Helpers

**Files:**
- Modify: `src/ui/settings_window.rs`
- Test: `tests/settings_window_tests.rs`

- [ ] **Step 1: Write failing tests for save action and section layout**

Add these types imports and tests to `tests/settings_window_tests.rs`:

```rust
use ait::ui::settings_window::{
    SettingsSaveOutcome, settings_save_outcome_after_success, settings_window_layout,
};

#[test]
fn successful_settings_save_keeps_window_open() {
    assert_eq!(
        settings_save_outcome_after_success(),
        SettingsSaveOutcome::KeepOpen
    );
}

#[test]
fn settings_window_layout_places_global_settings_above_profiles() {
    let layout = settings_window_layout();

    assert!(layout.hotkey.y < layout.separator.y);
    assert!(layout.copy_wait.y < layout.separator.y);
    assert!(layout.profile_list.y > layout.separator.y);
    assert!(layout.name.y > layout.separator.y);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```powershell
cargo test --test settings_window_tests successful_settings_save_keeps_window_open settings_window_layout_places_global_settings_above_profiles
```

Expected: tests fail to compile because the helper types and functions do not exist.

- [ ] **Step 3: Add layout/action helper structs**

In `src/ui/settings_window.rs`, add these non-Windows-gated types near `settings_window_center_position`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsSaveOutcome {
    KeepOpen,
}

pub fn settings_save_outcome_after_success() -> SettingsSaveOutcome {
    SettingsSaveOutcome::KeepOpen
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingsControlRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingsWindowLayout {
    pub hotkey: SettingsControlRect,
    pub copy_wait: SettingsControlRect,
    pub separator: SettingsControlRect,
    pub profile_list: SettingsControlRect,
    pub name: SettingsControlRect,
}

pub fn settings_window_layout() -> SettingsWindowLayout {
    SettingsWindowLayout {
        hotkey: SettingsControlRect {
            x: 118,
            y: 18,
            width: 180,
            height: 24,
        },
        copy_wait: SettingsControlRect {
            x: 430,
            y: 18,
            width: 90,
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
    }
}
```

- [ ] **Step 4: Run tests**

Run:

```powershell
cargo test --test settings_window_tests successful_settings_save_keeps_window_open settings_window_layout_places_global_settings_above_profiles
```

Expected: both tests pass.

- [ ] **Step 5: Commit helper changes**

```powershell
git add src\ui\settings_window.rs tests\settings_window_tests.rs
git commit -m "test: capture settings window save and layout rules"
```

## Task 4: Settings Window Win32 UI Behavior

**Files:**
- Modify: `src/ui/settings_window.rs`

- [ ] **Step 1: Remove provider combobox from window creation**

In `SettingsWindow::open`, delete the `create_static(hwnd, "供应商", ...)`, `create_provider_combo(...)`, and `select_provider(...)` calls.

Keep `ID_PROVIDER` and provider helper code until the end of this task, then remove unused constants/functions after compilation identifies them.

- [ ] **Step 2: Move global controls to the top section**

In `SettingsWindow::open`, create the top controls before profile controls:

```rust
create_static(hwnd, "快捷键", 18, 20, 90, 22)?;
create_edit(hwnd, &view_model.hotkey, 118, 18, 180, 24, false, ID_HOTKEY)?;
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
create_static(hwnd, "", 18, 62, 668, 1)?;
```

- [ ] **Step 3: Move profile controls below the separator**

Use these coordinates for the lower section:

```rust
create_static(hwnd, "翻译配置", 18, 74, 120, 22)?;
let profile_list = create_listbox(hwnd, 18, 100, 220, 228, ID_PROFILE_LIST)?;
create_button(hwnd, "新增", 18, 342, 64, 28, ID_NEW_PROFILE)?;
let delete_button = create_button(hwnd, "删除", 90, 342, 64, 28, ID_DELETE_PROFILE)?;
create_button(hwnd, "设为默认", 162, 342, 76, 28, ID_SET_DEFAULT)?;
```

Use detail coordinates:

```rust
create_static(hwnd, "名称", 266, 102, 90, 22)?;
create_edit(hwnd, &view_model.selected_profile.name, 370, 100, 240, 24, false, ID_NAME)?;
create_static(hwnd, "Base URL", 266, 136, 90, 22)?;
create_edit(hwnd, &view_model.selected_profile.base_url, 370, 134, 300, 24, false, ID_BASE_URL)?;
create_static(hwnd, "模型", 266, 170, 90, 22)?;
create_edit(hwnd, &view_model.selected_profile.model, 370, 168, 240, 24, false, ID_MODEL)?;
create_static(hwnd, "API Key", 266, 204, 90, 22)?;
create_edit(
    hwnd,
    if view_model.selected_profile.has_api_key { "已保存" } else { "" },
    370,
    202,
    240,
    24,
    true,
    ID_API_KEY,
)?;
create_static(hwnd, "超时秒数", 266, 238, 90, 22)?;
create_edit(
    hwnd,
    &view_model.selected_profile.timeout_secs.to_string(),
    370,
    236,
    90,
    24,
    false,
    ID_TIMEOUT,
)?;
create_static(hwnd, "Google 配置使用免 Key 翻译。", 266, 278, 390, 36)?;
```

- [ ] **Step 4: Do not destroy the settings window after save**

In the `ID_SAVE` branch of `default_wnd_proc`, replace the success body with:

```rust
Ok(_) => unsafe {
    if let Some(owner) = get_owner_hwnd(hwnd) {
        let _ = windows::Win32::UI::WindowsAndMessaging::PostMessageW(
            Some(owner),
            WM_SETTINGS_SAVED,
            windows::Win32::Foundation::WPARAM(0),
            windows::Win32::Foundation::LPARAM(0),
        );
    }
},
```

- [ ] **Step 5: Preserve provider when saving from controls**

In `save_settings_from_window`, get the selected profile's existing provider from settings:

```rust
let existing_provider = settings
    .profile_by_id(&profile_id)
    .map(|profile| profile.provider)
    .ok_or_else(|| AppError::Config("翻译配置不存在".to_string()))?;
```

Pass `provider: existing_provider` in `SettingsProfileDetailUpdate`.

- [ ] **Step 6: Update profile loading to stop selecting provider**

In `load_profile_into_window`, remove:

```rust
select_provider(control(hwnd, ID_PROVIDER)?, profile.provider)?;
```

Keep setting detail text fields, then call updated UI state helpers from Step 7.

- [ ] **Step 7: Add delete enablement and Google visibility helpers**

Add this Windows-only helper:

```rust
#[cfg(windows)]
fn apply_profile_detail_ui_state(
    hwnd: windows::Win32::Foundation::HWND,
    profile: &SettingsProfileDetail,
) {
    use windows::Win32::UI::Input::KeyboardAndMouse::EnableWindow;
    use windows::Win32::UI::WindowsAndMessaging::ShowWindow;
    use windows::Win32::UI::WindowsAndMessaging::{SW_HIDE, SW_SHOW};

    if let Ok(delete_button) = control(hwnd, ID_DELETE_PROFILE as i32) {
        unsafe {
            let _ = EnableWindow(delete_button, profile.can_delete);
        }
    }
    for id in [ID_NAME, ID_BASE_URL, ID_MODEL, ID_API_KEY, ID_TIMEOUT] {
        if let Ok(child) = control(hwnd, id) {
            let visible = if id == ID_NAME {
                true
            } else {
                profile.network_fields_visible
            };
            unsafe {
                let _ = ShowWindow(child, if visible { SW_SHOW } else { SW_HIDE });
                let _ = EnableWindow(child, id != ID_NAME || profile.name_editable);
            }
        }
    }
}
```

Call it after initial control creation and at the end of `load_profile_into_window`:

```rust
apply_profile_detail_ui_state(hwnd, &view_model.selected_profile);
```

- [ ] **Step 8: Remove provider combobox helpers**

Remove `PROVIDER_OPTIONS`, `ID_PROVIDER`, `selected_provider`, `select_provider`, and `create_provider_combo` if no references remain.

- [ ] **Step 9: Run checks**

Run:

```powershell
cargo test --test settings_window_tests
cargo check
```

Expected: tests pass and `cargo check` has no errors.

- [ ] **Step 10: Commit**

```powershell
git add src\ui\settings_window.rs
git commit -m "fix: update settings window behavior"
```

## Task 5: Translation Window Combo Dropdown Height

**Files:**
- Modify: `src/ui/translate_window.rs`
- Test: `tests/workflow_tests.rs`

- [ ] **Step 1: Write failing layout test**

In `tests/workflow_tests.rs`, update the import list to include `translation_profile_combo_dropdown_height`, then add:

```rust
#[test]
fn translation_profile_combo_keeps_dropdown_height() {
    let layout = translation_window_layout(620, 420);

    assert_eq!(layout.profile_combo.height, 26);
    assert_eq!(translation_profile_combo_dropdown_height(), 220);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test workflow_tests translation_profile_combo_keeps_dropdown_height
```

Expected: test fails to compile because `translation_profile_combo_dropdown_height` does not exist.

- [ ] **Step 3: Add dropdown height helper**

In `src/ui/translate_window.rs`, add:

```rust
pub fn translation_profile_combo_dropdown_height() -> i32 {
    220
}
```

- [ ] **Step 4: Use dropdown height when moving the Win32 combobox**

In `TranslationWindow::apply_layout`, replace the profile combo move call with:

```rust
move_window(
    self.profile_combo,
    ControlRect {
        height: translation_profile_combo_dropdown_height(),
        ..layout.profile_combo
    },
)?;
```

In `resize_translation_window`, replace the profile combo move call with the same `ControlRect` expression.

- [ ] **Step 5: Run workflow tests**

Run:

```powershell
cargo test --test workflow_tests translation_profile_combo_keeps_dropdown_height translation_window_layout_resizes_content_with_client_area translation_window_layout_keeps_controls_inside_small_client_area
```

Expected: tests pass.

- [ ] **Step 6: Commit**

```powershell
git add src\ui\translate_window.rs tests\workflow_tests.rs
git commit -m "fix: preserve translation profile dropdown height"
```

## Task 6: Async Translation Planning Types

**Files:**
- Modify: `src/app.rs`
- Test: `tests/workflow_tests.rs`

- [ ] **Step 1: Write failing tests for translation task action**

Add these imports and tests to `tests/workflow_tests.rs`:

```rust
use ait::app::{TranslationRequestKind, translation_task_action};

#[test]
fn hotkey_translation_runs_as_selection_task() {
    assert_eq!(
        translation_task_action(true, ""),
        TranslationRequestKind::Selection
    );
}

#[test]
fn window_translation_runs_as_text_task() {
    assert_eq!(
        translation_task_action(false, "hello"),
        TranslationRequestKind::WindowText {
            source_text: "hello".to_string()
        }
    );
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```powershell
cargo test --test workflow_tests hotkey_translation_runs_as_selection_task window_translation_runs_as_text_task
```

Expected: tests fail to compile because `TranslationRequestKind` and `translation_task_action` do not exist.

- [ ] **Step 3: Add request kind helper to `src/app.rs`**

Add near `HotkeyAction`:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TranslationRequestKind {
    Selection,
    WindowText { source_text: String },
}

pub fn translation_task_action(
    selection_requested: bool,
    source_text: &str,
) -> TranslationRequestKind {
    if selection_requested {
        TranslationRequestKind::Selection
    } else {
        TranslationRequestKind::WindowText {
            source_text: source_text.to_string(),
        }
    }
}
```

- [ ] **Step 4: Run tests**

Run:

```powershell
cargo test --test workflow_tests hotkey_translation_runs_as_selection_task window_translation_runs_as_text_task
```

Expected: tests pass.

- [ ] **Step 5: Commit**

```powershell
git add src\app.rs tests\workflow_tests.rs
git commit -m "test: capture async translation request kinds"
```

## Task 7: Move Blocking Translation To Background Thread

**Files:**
- Modify: `src/app.rs`
- Modify: `src/ui/translate_window.rs`

- [ ] **Step 1: Add Windows completion message and result payload**

In `src/app.rs` inside the Windows section, define:

```rust
#[cfg(windows)]
const WM_TRANSLATION_TASK_FINISHED: u32 =
    windows::Win32::UI::WindowsAndMessaging::WM_APP + 60;

#[cfg(windows)]
struct TranslationTaskMessage {
    result: Result<TranslationWorkflowResult>,
}
```

- [ ] **Step 2: Add background spawn helper**

In `src/app.rs`, add:

```rust
#[cfg(windows)]
fn spawn_translation_task(
    state: AppRuntimeState,
    request: TranslationRequestKind,
    notify_hwnd: windows::Win32::Foundation::HWND,
) {
    std::thread::spawn(move || {
        let result = run_translation_task(&state, request);
        let message = Box::into_raw(Box::new(TranslationTaskMessage { result }));
        unsafe {
            let posted = windows::Win32::UI::WindowsAndMessaging::PostMessageW(
                Some(notify_hwnd),
                WM_TRANSLATION_TASK_FINISHED,
                windows::Win32::Foundation::WPARAM(0),
                windows::Win32::Foundation::LPARAM(message as isize),
            );
            if posted.is_err() {
                drop(Box::from_raw(message));
            }
        }
    });
}
```

- [ ] **Step 3: Add task runner that does not touch UI controls**

In `src/app.rs`, add:

```rust
#[cfg(windows)]
fn run_translation_task(
    state: &AppRuntimeState,
    request: TranslationRequestKind,
) -> Result<TranslationWorkflowResult> {
    let workflow = build_workflow(state)?;
    match request {
        TranslationRequestKind::Selection => {
            workflow.translate_selection(&state.settings().target_language)
        }
        TranslationRequestKind::WindowText { source_text } => {
            workflow.translate_text(&source_text, &state.settings().target_language)
        }
    }
}
```

- [ ] **Step 4: Add UI-only loading helpers**

In `src/ui/translate_window.rs`, add methods:

```rust
#[cfg(windows)]
impl TranslationWindow {
    pub fn begin_selection_translation(&mut self) -> Result<()> {
        self.show_starting()
    }

    pub fn begin_window_text_translation(&mut self, source_text: String) -> Result<()> {
        self.show_loading(source_text)
    }

    pub fn finish_translation_result(
        &mut self,
        result: crate::error::Result<crate::app::TranslationWorkflowResult>,
    ) -> Result<()> {
        match result {
            Ok(result) => self.show_result(result.translated_text),
            Err(err) => self.show_error(err.to_string()),
        }
    }
}
```

- [ ] **Step 5: Replace hotkey synchronous translation call**

In the `WM_HOTKEY` handling branch in `run_platform`, replace:

```rust
let _ = perform_translation(&runtime_state, &mut translation_window);
```

With:

```rust
let _ = translation_window.begin_selection_translation();
spawn_translation_task(
    runtime_state.clone(),
    TranslationRequestKind::Selection,
    translation_window.hwnd(),
);
```

- [ ] **Step 6: Replace window text synchronous translation call**

In the `WM_TRANSLATE_WINDOW_SOURCE` branch, replace:

```rust
let _ = perform_window_text_translation(&runtime_state, &mut translation_window);
```

With:

```rust
match translation_window.source_text() {
    Ok(source_text) => {
        let _ = translation_window.begin_window_text_translation(source_text.clone());
        spawn_translation_task(
            runtime_state.clone(),
            TranslationRequestKind::WindowText { source_text },
            translation_window.hwnd(),
        );
    }
    Err(err) => {
        let _ = translation_window.show_error(err.to_string());
    }
}
```

- [ ] **Step 7: Replace profile-switch retranslate synchronous call**

In the `SaveDefaultAndRetranslate` success branch, replace the direct `perform_window_text_translation` call with the same `source_text` snapshot approach:

```rust
let source_text = translation_window.source_text().unwrap_or_default();
let _ = translation_window.begin_window_text_translation(source_text.clone());
spawn_translation_task(
    runtime_state.clone(),
    TranslationRequestKind::WindowText { source_text },
    translation_window.hwnd(),
);
```

- [ ] **Step 8: Handle task completion message**

In the main message loop, before `TranslateMessage`, add:

```rust
} else if msg.message == WM_TRANSLATION_TASK_FINISHED {
    let ptr = msg.lParam.0 as *mut TranslationTaskMessage;
    if !ptr.is_null() {
        let message = unsafe { Box::from_raw(ptr) };
        let result_for_log = message.result.as_ref().map(|result| {
            (
                result.provider.as_log_name(),
                result.source_text.chars().count(),
                result.translated_text.chars().count(),
            )
        });
        let result = message.result;
        let _ = translation_window.finish_translation_result(result);
        if let Ok((provider, source_len, translated_len)) = result_for_log {
            tracing::info!(
                provider,
                source_len,
                translated_len,
                "translation completed"
            );
        }
    }
```

If the borrow checker rejects moving `message.result` after `as_ref`, compute logging inside a `match` before calling `finish_translation_result`.

- [ ] **Step 9: Remove unused synchronous UI translation functions**

Delete `perform_window_text_translation` and `perform_translation` after all call sites are gone.

- [ ] **Step 10: Run checks**

Run:

```powershell
cargo test
cargo check
```

Expected: all tests pass and `cargo check` has no errors.

- [ ] **Step 11: Commit**

```powershell
git add src\app.rs src\ui\translate_window.rs
git commit -m "fix: run translations off the ui thread"
```

## Task 8: Final Verification

**Files:**
- No planned source edits unless verification reveals a defect.

- [ ] **Step 1: Run full test suite**

Run:

```powershell
cargo test
```

Expected: all tests pass.

- [ ] **Step 2: Run compile check**

Run:

```powershell
cargo check
```

Expected: no errors.

- [ ] **Step 3: Inspect git diff**

Run:

```powershell
git status --short
git diff --stat
```

Expected: only intentional source and test files are modified, or working tree is clean if all task commits were created.

- [ ] **Step 4: Manual Windows verification**

Run the app on Windows:

```powershell
cargo run
```

Verify:

- Settings save keeps the settings window open.
- Settings top section contains shortcut and copy wait.
- Interface section does not show shortcut, copy wait, or provider selector.
- Built-in profiles do not show `内置` in the list.
- Built-in profile delete button is disabled.
- Google shows only the免 Key statement and no network fields.
- Non-Google profiles show editable network fields.
- Translation profile dropdown expands and lists profiles.
- During a slow translation, the translation window can still move, close, and repaint.

## Self-Review

- Spec coverage: Settings save, two-section layout, label simplification, delete enablement, Google readonly view, Google-only notice, provider selector removal, dropdown height, and async translation are each mapped to tasks.
- Placeholder scan: The plan contains no `TBD`, `TODO`, or open-ended "handle later" steps.
- Type consistency: New helper names are introduced before use: `SettingsSaveOutcome`, `settings_window_layout`, `translation_profile_combo_dropdown_height`, `TranslationRequestKind`, and `translation_task_action`.
