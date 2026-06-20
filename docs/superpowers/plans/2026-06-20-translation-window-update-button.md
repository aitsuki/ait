# Translation Window Update Button Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. Do not use superpowers:subagent-driven-development because `AGENTS.md` forbids it.

**Goal:** Stop showing the update dialog automatically at startup and show a top-level `有新版本` button in the translation window when an update is available.

**Architecture:** Keep update checking in `src/app.rs` and version comparison/message formatting in `src/update.rs`. Add structured update state and a hidden-by-default update button to `src/ui/translate_window.rs`; `src/app.rs` stores update availability into the window and only shows the existing dialog when the user clicks the button.

**Tech Stack:** Rust, Win32 via the `windows` crate, Cargo integration tests.

---

## File Structure

- Modify `src/ui/translate_window.rs`: add update button ID/message, add update button layout, store `Option<UpdateStatus>`, show/hide the button, expose update status for app-layer click handling.
- Modify `src/app.rs`: add a pure update-check action helper for test coverage; route startup `UpdateAvailable` to `TranslationWindow::show_update_available`; handle the update button click by showing the existing update dialog.
- Modify `tests/workflow_tests.rs`: add tests for update button layout/state and the app-layer update-check action helper.

---

### Task 1: Add Translation Window Update State Tests

**Files:**
- Modify: `tests/workflow_tests.rs`
- Modify: `src/ui/translate_window.rs`

- [ ] **Step 1: Write the failing tests**

In `tests/workflow_tests.rs`, extend the imports:

```rust
use ait::update::{UpdateStatus, latest_release_url};
```

In the `ait::ui::translate_window` import list, add `translation_window_update_button_visible`:

```rust
    translation_profile_combo_dropdown_height, translation_window_layout,
    translation_window_min_client_size, translation_window_update_button_visible, window_z_order,
```

Add these tests near the existing translation window layout/state tests:

```rust
#[test]
fn translation_window_update_button_is_hidden_without_update_status() {
    assert!(!translation_window_update_button_visible(None));
}

#[test]
fn translation_window_update_button_is_visible_when_update_is_available() {
    let status = UpdateStatus::UpdateAvailable {
        current_version: "v0.1.4".to_string(),
        latest_version: "v0.1.5".to_string(),
        release_url: latest_release_url().to_string(),
    };

    assert!(translation_window_update_button_visible(Some(&status)));
}
```

- [ ] **Step 2: Run tests to verify RED**

Run:

```powershell
cargo test --test workflow_tests translation_window_update_button
```

Expected: compile failure because `translation_window_update_button_visible` does not exist.

- [ ] **Step 3: Write minimal implementation**

In `src/ui/translate_window.rs`, add this function near `translation_profile_combo_dropdown_height`:

```rust
pub fn translation_window_update_button_visible(
    status: Option<&crate::update::UpdateStatus>,
) -> bool {
    matches!(status, Some(crate::update::UpdateStatus::UpdateAvailable { .. }))
}
```

- [ ] **Step 4: Run tests to verify GREEN**

Run:

```powershell
cargo test --test workflow_tests translation_window_update_button
```

Expected: PASS.

- [ ] **Step 5: Commit**

```powershell
git add tests/workflow_tests.rs src/ui/translate_window.rs
git commit -m "test: cover translation update button visibility"
```

---

### Task 2: Add Update Button Layout

**Files:**
- Modify: `tests/workflow_tests.rs`
- Modify: `src/ui/translate_window.rs`

- [ ] **Step 1: Write the failing test**

In `tests/workflow_tests.rs`, add:

```rust
#[test]
fn translation_window_layout_places_update_button_before_profile_combo() {
    let layout = translation_window_layout(620, 420);

    assert_eq!(layout.update_button.height, layout.profile_combo.height);
    assert!(layout.update_button.y <= layout.profile_combo.y + 2);
    assert!(layout.update_button.x + layout.update_button.width < layout.profile_combo.x);
    assert!(layout.update_button.x > layout.source_label.x + layout.source_label.width);
}
```

- [ ] **Step 2: Run test to verify RED**

Run:

```powershell
cargo test --test workflow_tests translation_window_layout_places_update_button_before_profile_combo
```

Expected: compile failure because `TranslationWindowLayout` has no `update_button` field.

- [ ] **Step 3: Write minimal implementation**

In `src/ui/translate_window.rs`, add constants in `translation_window_layout`:

```rust
    const UPDATE_BUTTON_WIDTH: i32 = 86;
```

Add `update_button` to `TranslationWindowLayout`:

```rust
    pub update_button: ControlRect,
```

After `profile_combo` is computed, add:

```rust
    let update_button_width = UPDATE_BUTTON_WIDTH
        .min((profile_combo.x - content_x - GAP).max(1))
        .max(1);
    let update_button_x = (profile_combo.x - GAP - update_button_width)
        .max(content_x + source_label.width + GAP)
        .min(usable_width - update_button_width);
    let update_button = ControlRect {
        x: update_button_x,
        y: profile_combo.y,
        width: update_button_width,
        height: combo_height,
    };
```

Then include it in the returned layout:

```rust
        update_button,
```

- [ ] **Step 4: Run test to verify GREEN**

Run:

```powershell
cargo test --test workflow_tests translation_window_layout_places_update_button_before_profile_combo
```

Expected: PASS.

- [ ] **Step 5: Commit**

```powershell
git add tests/workflow_tests.rs src/ui/translate_window.rs
git commit -m "feat: add translation update button layout"
```

---

### Task 3: Add App-Layer Update Check Action Helper

**Files:**
- Modify: `tests/workflow_tests.rs`
- Modify: `src/app.rs`

- [ ] **Step 1: Write the failing tests**

In `tests/workflow_tests.rs`, extend the `ait::app` import list:

```rust
    UpdateCheckAction, update_check_action,
```

Add:

```rust
#[test]
fn silent_update_check_shows_button_instead_of_dialog_when_update_available() {
    let status = UpdateStatus::UpdateAvailable {
        current_version: "v0.1.4".to_string(),
        latest_version: "v0.1.5".to_string(),
        release_url: latest_release_url().to_string(),
    };

    assert_eq!(
        update_check_action(Ok(status), false),
        UpdateCheckAction::ShowUpdateButton
    );
}

#[test]
fn explicit_update_check_still_shows_dialog_when_update_available() {
    let status = UpdateStatus::UpdateAvailable {
        current_version: "v0.1.4".to_string(),
        latest_version: "v0.1.5".to_string(),
        release_url: latest_release_url().to_string(),
    };

    assert_eq!(
        update_check_action(Ok(status), true),
        UpdateCheckAction::ShowDialog
    );
}

#[test]
fn silent_update_check_ignores_up_to_date_and_errors() {
    assert_eq!(
        update_check_action(Ok(UpdateStatus::UpToDate), false),
        UpdateCheckAction::Ignore
    );
    assert_eq!(
        update_check_action(Err("network".to_string()), false),
        UpdateCheckAction::Ignore
    );
}
```

- [ ] **Step 2: Run tests to verify RED**

Run:

```powershell
cargo test --test workflow_tests update_check
```

Expected: compile failure because `UpdateCheckAction` and `update_check_action` do not exist.

- [ ] **Step 3: Write minimal implementation**

In `src/app.rs`, add after `UpdateCheckDisplayMode`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateCheckAction {
    Ignore,
    ShowDialog,
    ShowUpdateButton,
}

pub fn update_check_action(
    result: std::result::Result<UpdateStatus, String>,
    show_all: bool,
) -> UpdateCheckAction {
    match result {
        Ok(UpdateStatus::UpdateAvailable { .. }) if show_all => UpdateCheckAction::ShowDialog,
        Ok(UpdateStatus::UpdateAvailable { .. }) => UpdateCheckAction::ShowUpdateButton,
        Ok(UpdateStatus::UpToDate) if show_all => UpdateCheckAction::ShowDialog,
        Ok(UpdateStatus::UpToDate) => UpdateCheckAction::Ignore,
        Err(_) if show_all => UpdateCheckAction::ShowDialog,
        Err(_) => UpdateCheckAction::Ignore,
    }
}
```

- [ ] **Step 4: Run tests to verify GREEN**

Run:

```powershell
cargo test --test workflow_tests update_check
```

Expected: PASS.

- [ ] **Step 5: Commit**

```powershell
git add tests/workflow_tests.rs src/app.rs
git commit -m "test: cover update check display action"
```

---

### Task 4: Wire Update Button into Windows Translation Window

**Files:**
- Modify: `src/ui/translate_window.rs`

- [ ] **Step 1: Add Windows constants and state fields**

In `src/ui/translate_window.rs`, add:

```rust
#[cfg(windows)]
const ID_UPDATE_BUTTON: usize = 2002;
#[cfg(windows)]
pub const WM_TRANSLATE_WINDOW_UPDATE_CLICKED: u32 =
    windows::Win32::UI::WindowsAndMessaging::WM_APP + 32;
```

Add to `TranslationWindow`:

```rust
    update_button: windows::Win32::Foundation::HWND,
    update_status: Option<crate::update::UpdateStatus>,
```

- [ ] **Step 2: Create hidden button**

In `TranslationWindow::new`, after creating `profile_combo`, add:

```rust
            let update_button =
                create_button(hwnd, "有新版本", 314, 12, 86, 26, ID_UPDATE_BUTTON as isize)?;
            hide_window(update_button);
```

Include it in `Self`:

```rust
                update_button,
                update_status: None,
```

Add helper near `set_text`:

```rust
#[cfg(windows)]
fn hide_window(hwnd: windows::Win32::Foundation::HWND) {
    unsafe {
        let _ = windows::Win32::UI::WindowsAndMessaging::ShowWindow(
            hwnd,
            windows::Win32::UI::WindowsAndMessaging::SW_HIDE,
        );
    }
}
```

Add helper:

```rust
#[cfg(windows)]
fn set_window_visible(hwnd: windows::Win32::Foundation::HWND, visible: bool) {
    let command = if visible {
        windows::Win32::UI::WindowsAndMessaging::SW_SHOW
    } else {
        windows::Win32::UI::WindowsAndMessaging::SW_HIDE
    };
    unsafe {
        let _ = windows::Win32::UI::WindowsAndMessaging::ShowWindow(hwnd, command);
    }
}
```

- [ ] **Step 3: Add methods for update status**

In the `impl TranslationWindow` block, add:

```rust
    pub fn show_update_available(&mut self, status: crate::update::UpdateStatus) -> Result<()> {
        self.update_status = Some(status);
        set_window_visible(
            self.update_button,
            translation_window_update_button_visible(self.update_status.as_ref()),
        );
        Ok(())
    }

    pub fn update_status(&self) -> Option<&crate::update::UpdateStatus> {
        self.update_status.as_ref()
    }
```

- [ ] **Step 4: Apply layout and route click message**

In `apply_layout`, add:

```rust
            move_window(self.update_button, layout.update_button)?;
```

In `resize_translation_window`, fetch and move the button:

```rust
        let update_button =
            windows::Win32::UI::WindowsAndMessaging::GetDlgItem(Some(hwnd), ID_UPDATE_BUTTON as i32)
                .map_err(|err| AppError::Windows(format!("获取更新按钮失败: {err}")))?;
```

Then move it:

```rust
        move_window(update_button, layout.update_button)?;
```

In `default_wnd_proc`, add a command branch before `_ => {}`:

```rust
            ID_UPDATE_BUTTON => unsafe {
                let _ = PostMessageW(
                    Some(hwnd),
                    WM_TRANSLATE_WINDOW_UPDATE_CLICKED,
                    WPARAM(0),
                    LPARAM(0),
                );
                return LRESULT(0);
            },
```

- [ ] **Step 5: Run focused tests and compile**

Run:

```powershell
cargo test --test workflow_tests translation_window_update_button
cargo test --test workflow_tests translation_window_layout_places_update_button_before_profile_combo
cargo check
```

Expected: PASS and `cargo check` succeeds.

- [ ] **Step 6: Commit**

```powershell
git add src/ui/translate_window.rs
git commit -m "feat: add translation update button control"
```

---

### Task 5: Route Update Checks and Button Clicks in App Loop

**Files:**
- Modify: `src/app.rs`

- [ ] **Step 1: Replace automatic startup dialog with button state**

In the `WM_UPDATE_CHECK_FINISHED` branch, keep the existing `match message.result` shape but change the `UpdateAvailable` branch to:

```rust
                        Ok(UpdateStatus::UpdateAvailable {
                            current_version,
                            latest_version,
                            release_url,
                        }) => {
                            let status = UpdateStatus::UpdateAvailable {
                                current_version,
                                latest_version,
                                release_url,
                            };
                            if matches!(message.display_mode, UpdateCheckDisplayMode::ShowAll) {
                                show_runtime_message(
                                    translation_window.hwnd(),
                                    "发现新版本",
                                    &update_status_message(&message.current_version, &status),
                                );
                            } else if let Err(err) = translation_window.show_update_available(status)
                            {
                                tracing::warn!(error = %err, "show update button failed");
                            }
                        }
```

- [ ] **Step 2: Handle update button click**

In the message loop, add an `else if` branch before settings saved handling:

```rust
            } else if msg.message
                == crate::ui::translate_window::WM_TRANSLATE_WINDOW_UPDATE_CLICKED
            {
                if let Some(status) = translation_window.update_status() {
                    show_runtime_message(
                        translation_window.hwnd(),
                        "发现新版本",
                        &update_status_message(env!("CARGO_PKG_VERSION"), status),
                    );
                }
```

- [ ] **Step 3: Run update and workflow tests**

Run:

```powershell
cargo test --test workflow_tests update_check
cargo test --test release_tests
cargo check
```

Expected: PASS and `cargo check` succeeds.

- [ ] **Step 4: Commit**

```powershell
git add src/app.rs
git commit -m "fix: show startup updates in translation window"
```

---

### Task 6: Final Verification

**Files:**
- No planned source edits.

- [ ] **Step 1: Run full test suite**

Run:

```powershell
cargo test
```

Expected: PASS.

- [ ] **Step 2: Run formatting check**

Run:

```powershell
cargo fmt -- --check
```

Expected: PASS. If it fails, run `cargo fmt`, inspect the diff, and commit the formatting-only changes with:

```powershell
git add src/app.rs src/ui/translate_window.rs tests/workflow_tests.rs
git commit -m "style: format update button changes"
```

- [ ] **Step 3: Inspect final diff**

Run:

```powershell
git status --short
git log --oneline -5
```

Expected: clean working tree after commits; recent commits include the spec, plan, tests, UI control, and app routing changes.

