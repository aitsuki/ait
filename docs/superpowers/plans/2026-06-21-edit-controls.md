# Edit Controls Modernization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. Do not use `superpowers:subagent-driven-development`; this repository's `AGENTS.md` forbids it.

**Goal:** Modernize Win32 `EDIT` controls in the settings window and translation window without taking over native text rendering.

**Architecture:** Add a focused `src/ui/edit.rs` style module with pure state/palette/control-id logic plus Windows-only helpers for background brushes, focus tracking, and border painting. Wire settings and translation windows to remove the old native edit border, handle `WM_CTLCOLOREDIT` / `WM_CTLCOLORSTATIC`, and paint a lightweight outer border while preserving existing text, selection, scrolling, IME, API key, hotkey, and multiline edit behavior.

**Tech Stack:** Rust, `windows-rs`, Win32 `EDIT`, GDI, existing `cargo test` suite.

---

## File Structure

- Create `src/ui/edit.rs`
  - Owns modern `EDIT` visual state, palette selection, known control id mapping, background brush cache, focus tracking, subclass installation, and border painting helpers.
  - Keeps pure logic testable without Windows.

- Modify `src/ui/mod.rs`
  - Exposes `edit` on Windows next to `button` and `font`.

- Modify `src/ui/settings_window.rs`
  - Routes setting-window `EDIT` controls through the new module.
  - Handles edit color messages and border painting in the parent window.
  - Installs focus tracking for single-line settings edits while preserving hotkey/API key behavior.

- Modify `src/ui/translate_window.rs`
  - Routes source/translated multiline edits through the new module.
  - Extends the existing multiline edit subclass path with focus repaint support.
  - Handles edit color messages and border painting in the parent window.

- Tests live with the modules already used by the project:
  - Pure unit tests in `src/ui/edit.rs`.
  - Existing integration tests in `tests/settings_window_tests.rs` and `tests/workflow_tests.rs` remain the regression net for behavior that must not change.

---

### Task 1: Add Pure Edit Style Module

**Files:**
- Create: `src/ui/edit.rs`
- Modify: `src/ui/mod.rs`

- [ ] **Step 1: Write the failing pure logic module and tests**

Create `src/ui/edit.rs` with this initial content:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditKind {
    SingleLine,
    MultiLine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditVisualState {
    pub focused: bool,
    pub readonly: bool,
    pub disabled: bool,
}

impl EditVisualState {
    pub fn normal() -> Self {
        Self {
            focused: false,
            readonly: false,
            disabled: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RgbColor {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditPalette {
    pub background: RgbColor,
    pub border: RgbColor,
    pub text: RgbColor,
}

pub fn edit_palette(state: EditVisualState) -> EditPalette {
    if state.disabled {
        return EditPalette {
            background: RgbColor::new(243, 244, 246),
            border: RgbColor::new(209, 213, 219),
            text: RgbColor::new(156, 163, 175),
        };
    }

    if state.readonly {
        return EditPalette {
            background: RgbColor::new(248, 250, 252),
            border: RgbColor::new(203, 213, 225),
            text: RgbColor::new(31, 41, 55),
        };
    }

    EditPalette {
        background: RgbColor::new(255, 255, 255),
        border: if state.focused {
            RgbColor::new(37, 99, 235)
        } else {
            RgbColor::new(203, 213, 225)
        },
        text: RgbColor::new(31, 41, 55),
    }
}

pub fn edit_kind_for_control(id: usize) -> Option<EditKind> {
    match id {
        2101 | 2102 => Some(EditKind::MultiLine),
        3102 | 3104 | 3105 | 3106 | 3107 | 3108 => Some(EditKind::SingleLine),
        _ => None,
    }
}

pub fn is_modern_edit(id: usize) -> bool {
    edit_kind_for_control(id).is_some()
}

pub fn edit_uses_native_border(id: usize) -> bool {
    !is_modern_edit(id)
}

#[cfg(test)]
mod tests {
    use super::{
        EditKind, EditVisualState, RgbColor, edit_kind_for_control, edit_palette,
        edit_uses_native_border, is_modern_edit,
    };

    #[test]
    fn maps_translation_multiline_edits() {
        assert_eq!(edit_kind_for_control(2101), Some(EditKind::MultiLine));
        assert_eq!(edit_kind_for_control(2102), Some(EditKind::MultiLine));
    }

    #[test]
    fn maps_settings_single_line_edits() {
        for id in [3102, 3104, 3105, 3106, 3107, 3108] {
            assert_eq!(edit_kind_for_control(id), Some(EditKind::SingleLine));
            assert!(is_modern_edit(id));
        }
    }

    #[test]
    fn ignores_unknown_controls() {
        assert_eq!(edit_kind_for_control(9999), None);
        assert!(!is_modern_edit(9999));
        assert!(edit_uses_native_border(9999));
    }

    #[test]
    fn normal_edit_uses_white_surface() {
        let palette = edit_palette(EditVisualState::normal());
        assert_eq!(palette.background, RgbColor::new(255, 255, 255));
        assert_eq!(palette.border, RgbColor::new(203, 213, 225));
        assert_eq!(palette.text, RgbColor::new(31, 41, 55));
    }

    #[test]
    fn focused_edit_uses_blue_border() {
        let palette = edit_palette(EditVisualState {
            focused: true,
            ..EditVisualState::normal()
        });
        assert_eq!(palette.border, RgbColor::new(37, 99, 235));
    }

    #[test]
    fn readonly_edit_is_distinct_from_disabled() {
        let readonly = edit_palette(EditVisualState {
            readonly: true,
            ..EditVisualState::normal()
        });
        let disabled = edit_palette(EditVisualState {
            disabled: true,
            ..EditVisualState::normal()
        });

        assert_eq!(readonly.background, RgbColor::new(248, 250, 252));
        assert_eq!(readonly.text, RgbColor::new(31, 41, 55));
        assert_eq!(disabled.background, RgbColor::new(243, 244, 246));
        assert_eq!(disabled.text, RgbColor::new(156, 163, 175));
        assert_ne!(readonly, disabled);
    }
}
```

Modify `src/ui/mod.rs`:

```rust
#[cfg(windows)]
pub mod button;
#[cfg(windows)]
pub mod edit;
#[cfg(windows)]
pub mod font;
pub mod settings_window;
pub mod translate_window;
pub mod tray;
```

- [ ] **Step 2: Run tests for the new module**

Run:

```powershell
cargo test ui::edit
```

Expected: all `src/ui/edit.rs` tests pass.

- [ ] **Step 3: Run a quick compile check**

Run:

```powershell
cargo test --no-run
```

Expected: compile succeeds.

- [ ] **Step 4: Commit**

```powershell
git add src/ui/edit.rs src/ui/mod.rs
git commit -m "test: add edit control style logic"
```

---

### Task 2: Add Windows Edit Drawing Helpers

**Files:**
- Modify: `src/ui/edit.rs`

- [ ] **Step 1: Extend `src/ui/edit.rs` with Windows helpers**

Append this Windows-only code before the `#[cfg(test)] mod tests` block:

```rust
#[cfg(windows)]
impl RgbColor {
    fn colorref(self) -> windows::Win32::Foundation::COLORREF {
        windows::Win32::Foundation::COLORREF(
            self.r as u32 | ((self.g as u32) << 8) | ((self.b as u32) << 16),
        )
    }
}

#[cfg(windows)]
pub fn install_modern_edit_focus_tracking(
    hwnd: windows::Win32::Foundation::HWND,
) -> crate::error::Result<()> {
    use windows::Win32::UI::Shell::SetWindowSubclass;

    unsafe {
        if SetWindowSubclass(hwnd, Some(modern_edit_subclass_proc), MODERN_EDIT_SUBCLASS_ID, 0)
            .as_bool()
        {
            Ok(())
        } else {
            Err(crate::error::AppError::Windows(
                "安装输入框焦点处理失败".to_string(),
            ))
        }
    }
}

#[cfg(windows)]
pub fn modern_edit_brush_for_state(
    state: EditVisualState,
) -> windows::Win32::Graphics::Gdi::HBRUSH {
    use std::sync::OnceLock;
    use windows::Win32::Graphics::Gdi::{CreateSolidBrush, HBRUSH};

    static NORMAL_BRUSH: OnceLock<isize> = OnceLock::new();
    static READONLY_BRUSH: OnceLock<isize> = OnceLock::new();
    static DISABLED_BRUSH: OnceLock<isize> = OnceLock::new();

    let slot = if state.disabled {
        &DISABLED_BRUSH
    } else if state.readonly {
        &READONLY_BRUSH
    } else {
        &NORMAL_BRUSH
    };
    let palette = edit_palette(state);
    HBRUSH(
        *slot.get_or_init(|| unsafe { CreateSolidBrush(palette.background.colorref()).0 as isize })
            as *mut core::ffi::c_void,
    )
}

#[cfg(windows)]
pub unsafe fn paint_modern_edit_border(
    parent: windows::Win32::Foundation::HWND,
    control_id: i32,
    readonly: bool,
    hdc: windows::Win32::Graphics::Gdi::HDC,
) {
    use windows::Win32::Graphics::Gdi::{
        CreatePen, DeleteObject, GetStockObject, HGDIOBJ, NULL_BRUSH, PS_SOLID, Rectangle,
        SelectObject,
    };
    use windows::Win32::UI::Input::KeyboardAndMouse::IsWindowEnabled;
    use windows::Win32::UI::WindowsAndMessaging::{GetDlgItem, GetFocus, GetWindowRect, MapWindowPoints};

    let Ok(child) = unsafe { GetDlgItem(Some(parent), control_id) } else {
        return;
    };
    let state = EditVisualState {
        focused: unsafe { GetFocus() } == child,
        readonly,
        disabled: !unsafe { IsWindowEnabled(child).as_bool() },
    };
    let palette = edit_palette(state);
    let mut rect = windows::Win32::Foundation::RECT::default();
    if unsafe { GetWindowRect(child, &mut rect).is_err() } {
        return;
    }
    unsafe {
        let _ = MapWindowPoints(
            None,
            Some(parent),
            &mut rect as *mut _ as *mut windows::Win32::Foundation::POINT,
            2,
        );
    }

    let pen = unsafe { CreatePen(PS_SOLID, 1, palette.border.colorref()) };
    if pen.is_invalid() {
        return;
    }
    let old_pen = unsafe { SelectObject(hdc, pen.into()) };
    let old_brush = unsafe { SelectObject(hdc, GetStockObject(NULL_BRUSH)) };
    unsafe {
        let _ = Rectangle(hdc, rect.left, rect.top, rect.right, rect.bottom);
    }
    if !old_brush.is_invalid() {
        unsafe {
            let _ = SelectObject(hdc, old_brush);
        }
    }
    if !old_pen.is_invalid() {
        unsafe {
            let _ = SelectObject(hdc, old_pen);
        }
    }
    unsafe {
        let _ = DeleteObject(pen.into());
    }
}

#[cfg(windows)]
pub unsafe fn handle_modern_edit_color(
    parent: windows::Win32::Foundation::HWND,
    wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
    readonly: bool,
) -> Option<windows::Win32::Foundation::LRESULT> {
    use windows::Win32::Graphics::Gdi::{SetBkColor, SetTextColor};
    use windows::Win32::UI::Input::KeyboardAndMouse::IsWindowEnabled;
    use windows::Win32::UI::WindowsAndMessaging::{GetDlgCtrlID, GetFocus};

    let child = windows::Win32::Foundation::HWND(lparam.0 as *mut core::ffi::c_void);
    let id = unsafe { GetDlgCtrlID(child) };
    if !is_modern_edit(id as usize) {
        return None;
    }

    let state = EditVisualState {
        focused: unsafe { GetFocus() } == child,
        readonly,
        disabled: !unsafe { IsWindowEnabled(child).as_bool() },
    };
    let palette = edit_palette(state);
    let hdc = windows::Win32::Graphics::Gdi::HDC(wparam.0 as *mut core::ffi::c_void);
    unsafe {
        let _ = SetTextColor(hdc, palette.text.colorref());
        let _ = SetBkColor(hdc, palette.background.colorref());
        invalidate_edit_border(parent, child);
    }
    Some(windows::Win32::Foundation::LRESULT(
        modern_edit_brush_for_state(state).0 as isize,
    ))
}

#[cfg(windows)]
unsafe fn invalidate_edit_border(
    parent: windows::Win32::Foundation::HWND,
    child: windows::Win32::Foundation::HWND,
) {
    use windows::Win32::Graphics::Gdi::InvalidateRect;
    use windows::Win32::UI::WindowsAndMessaging::{GetWindowRect, MapWindowPoints};

    let mut rect = windows::Win32::Foundation::RECT::default();
    if unsafe { GetWindowRect(child, &mut rect).is_ok() } {
        unsafe {
            let _ = MapWindowPoints(
                None,
                Some(parent),
                &mut rect as *mut _ as *mut windows::Win32::Foundation::POINT,
                2,
            );
            let _ = InvalidateRect(Some(parent), Some(&rect), false);
        }
    }
}

#[cfg(windows)]
const MODERN_EDIT_SUBCLASS_ID: usize = 2;

#[cfg(windows)]
unsafe extern "system" fn modern_edit_subclass_proc(
    hwnd: windows::Win32::Foundation::HWND,
    msg: u32,
    wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
    subclass_id: usize,
    ref_data: usize,
) -> windows::Win32::Foundation::LRESULT {
    use windows::Win32::UI::Shell::{DefSubclassProc, RemoveWindowSubclass};
    use windows::Win32::UI::WindowsAndMessaging::{
        GetParent, WM_ENABLE, WM_KILLFOCUS, WM_NCDESTROY, WM_SETFOCUS,
    };

    if msg == WM_SETFOCUS || msg == WM_KILLFOCUS || msg == WM_ENABLE {
        if let Ok(parent) = unsafe { GetParent(hwnd) } {
            unsafe {
                invalidate_edit_border(parent, hwnd);
            }
        }
    }

    if msg == WM_NCDESTROY {
        unsafe {
            let _ = RemoveWindowSubclass(hwnd, Some(modern_edit_subclass_proc), subclass_id);
            return DefSubclassProc(hwnd, msg, wparam, lparam);
        }
    }

    unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) }
}
```

Remove unused imports if the compiler reports any. If `IsWindowEnabled` resolves under `WindowsAndMessaging` in the local `windows` crate version, import it from there instead of `UI::Input::KeyboardAndMouse`.

- [ ] **Step 2: Compile-check helper APIs**

Run:

```powershell
cargo test --no-run
```

Expected: compile succeeds. If the local `windows-rs` type signatures differ, adjust imports/casts only; do not change the public helper names.

- [ ] **Step 3: Run edit logic tests**

Run:

```powershell
cargo test ui::edit
```

Expected: all edit tests pass.

- [ ] **Step 4: Commit**

```powershell
git add src/ui/edit.rs
git commit -m "feat: add edit control drawing helpers"
```

---

### Task 3: Wire Settings Window Single-Line Edits

**Files:**
- Modify: `src/ui/settings_window.rs`
- Test: `tests/settings_window_tests.rs`

- [ ] **Step 1: Add a focused test for native-border opt-out**

Append to `tests/settings_window_tests.rs`:

```rust
#[test]
fn settings_edit_controls_use_modern_border() {
    for id in [3102, 3104, 3105, 3106, 3107, 3108] {
        assert!(!ait::ui::edit::edit_uses_native_border(id));
    }
}
```

- [ ] **Step 2: Run the new test and confirm failure if `ui::edit` is not accessible**

Run:

```powershell
cargo test settings_edit_controls_use_modern_border
```

Expected: PASS if Task 1 exposed `ui::edit`; otherwise fail with an import/access error and fix `src/ui/mod.rs` as shown in Task 1.

- [ ] **Step 3: Update settings `create_edit` to remove native borders and install focus tracking**

In `src/ui/settings_window.rs`, change `create_edit` so the final argument to `create_control` uses the new border decision and then installs tracking:

```rust
let hwnd = create_control(
    parent,
    "EDIT",
    text,
    x,
    y,
    width,
    height,
    id as isize,
    WINDOW_STYLE(style as u32),
    crate::ui::edit::edit_uses_native_border(id as usize),
)?;
if crate::ui::edit::is_modern_edit(id as usize) {
    crate::ui::edit::install_modern_edit_focus_tracking(hwnd)?;
}
Ok(hwnd)
```

Keep the existing `ES_AUTOHSCROLL` / `ES_PASSWORD` logic unchanged.

- [ ] **Step 4: Handle edit color messages in settings window proc**

In `default_wnd_proc`, extend imports to include:

```rust
WM_CTLCOLOREDIT, WM_CTLCOLORSTATIC, WM_PAINT, BeginPaint, EndPaint, PAINTSTRUCT
```

Then add these branches before `WM_DRAWITEM`:

```rust
if msg == WM_CTLCOLOREDIT {
    if let Some(result) =
        unsafe { crate::ui::edit::handle_modern_edit_color(hwnd, wparam, lparam, false) }
    {
        return result;
    }
}

if msg == WM_CTLCOLORSTATIC {
    let child = windows::Win32::Foundation::HWND(lparam.0 as *mut core::ffi::c_void);
    let id = unsafe { windows::Win32::UI::WindowsAndMessaging::GetDlgCtrlID(child) };
    let readonly = crate::ui::edit::is_modern_edit(id as usize);
    if readonly {
        if let Some(result) =
            unsafe { crate::ui::edit::handle_modern_edit_color(hwnd, wparam, lparam, true) }
        {
            return result;
        }
    }
}

if msg == WM_PAINT {
    let mut ps = PAINTSTRUCT::default();
    let hdc = unsafe { BeginPaint(hwnd, &mut ps) };
    for (id, readonly) in [
        (ID_NAME, false),
        (ID_BASE_URL, false),
        (ID_MODEL, false),
        (ID_API_KEY, false),
        (ID_TIMEOUT, false),
        (ID_HOTKEY, true),
    ] {
        unsafe {
            crate::ui::edit::paint_modern_edit_border(hwnd, id, readonly, hdc);
        }
    }
    unsafe {
        let _ = EndPaint(hwnd, &ps);
    }
    return LRESULT(0);
}
```

If direct `WM_PAINT` handling stops background erasure for hidden Google fields, instead call `DefWindowProcW` first for `WM_PAINT`, then draw borders using `GetDC` / `ReleaseDC`. Keep the existing `settings_window_uses_background_brush()` behavior intact.

- [ ] **Step 5: Run settings tests**

Run:

```powershell
cargo test --test settings_window_tests
```

Expected: all settings window tests pass.

- [ ] **Step 6: Run compile check**

Run:

```powershell
cargo test --no-run
```

Expected: compile succeeds.

- [ ] **Step 7: Commit**

```powershell
git add src/ui/settings_window.rs tests/settings_window_tests.rs
git commit -m "feat: modernize settings edit controls"
```

---

### Task 4: Wire Translation Window Multiline Edits

**Files:**
- Modify: `src/ui/translate_window.rs`
- Test: `tests/workflow_tests.rs`

- [ ] **Step 1: Add a focused test for translation edit coverage**

Append to `tests/workflow_tests.rs`:

```rust
#[test]
fn translation_multiline_edit_controls_use_modern_border() {
    assert!(!ait::ui::edit::edit_uses_native_border(2101));
    assert!(!ait::ui::edit::edit_uses_native_border(2102));
}
```

- [ ] **Step 2: Run the focused test**

Run:

```powershell
cargo test translation_multiline_edit_controls_use_modern_border
```

Expected: PASS.

- [ ] **Step 3: Update translation `create_edit` to remove native borders**

In `src/ui/translate_window.rs`, replace the existing `create_control(..., true)` call in `create_edit` with:

```rust
create_control(
    parent,
    "EDIT",
    "",
    x,
    y,
    width,
    height,
    id,
    style,
    crate::ui::edit::edit_uses_native_border(id as usize),
)
```

Do not change `ES_MULTILINE`, `ES_AUTOVSCROLL`, `ES_WANTRETURN`, `ES_READONLY`, or `WS_VSCROLL`.

- [ ] **Step 4: Extend the existing translation edit subclass for focus repaint**

In `edit_subclass_proc`, extend the message imports with:

```rust
GetParent, WM_ENABLE, WM_KILLFOCUS, WM_SETFOCUS
```

Near the top of the function, after the `state_ptr` calculation, add:

```rust
if msg == WM_SETFOCUS || msg == WM_KILLFOCUS || msg == WM_ENABLE {
    if let Ok(parent) = unsafe { GetParent(hwnd) } {
        unsafe {
            crate::ui::edit::invalidate_modern_edit_for_child(parent, hwnd);
        }
    }
}
```

If `invalidate_modern_edit_for_child` is not public yet, expose a public wrapper in `src/ui/edit.rs`:

```rust
#[cfg(windows)]
pub unsafe fn invalidate_modern_edit_for_child(
    parent: windows::Win32::Foundation::HWND,
    child: windows::Win32::Foundation::HWND,
) {
    unsafe {
        invalidate_edit_border(parent, child);
    }
}
```

Keep the existing `Ctrl+A`, `Escape`, double-click, third-click, and `WM_NCDESTROY` behavior unchanged.

- [ ] **Step 5: Handle edit color and border painting in translation window proc**

In `default_wnd_proc`, extend imports to include:

```rust
WM_CTLCOLOREDIT, WM_CTLCOLORSTATIC, WM_PAINT, BeginPaint, EndPaint, PAINTSTRUCT
```

Add these branches before `WM_DRAWITEM`:

```rust
if msg == WM_CTLCOLOREDIT {
    if let Some(result) =
        unsafe { crate::ui::edit::handle_modern_edit_color(hwnd, wparam, lparam, false) }
    {
        return result;
    }
}

if msg == WM_CTLCOLORSTATIC {
    if let Some(result) =
        unsafe { crate::ui::edit::handle_modern_edit_color(hwnd, wparam, lparam, true) }
    {
        return result;
    }
}

if msg == WM_PAINT {
    let mut ps = PAINTSTRUCT::default();
    let hdc = unsafe { BeginPaint(hwnd, &mut ps) };
    unsafe {
        crate::ui::edit::paint_modern_edit_border(hwnd, ID_SOURCE_EDIT as i32, false, hdc);
        crate::ui::edit::paint_modern_edit_border(hwnd, ID_TRANSLATED_EDIT as i32, true, hdc);
        let _ = EndPaint(hwnd, &ps);
    }
    return LRESULT(0);
}
```

If `WM_PAINT` handling causes the white window background not to erase, switch to the same `DefWindowProcW`-then-`GetDC` approach chosen in Task 3.

- [ ] **Step 6: Run translation behavior tests**

Run:

```powershell
cargo test --test workflow_tests edit_display_text edit_shortcut_action edit_char_action paragraph_selection_range third_click translation_multiline_edit_controls_use_modern_border
```

Expected: all targeted workflow tests pass.

- [ ] **Step 7: Run compile check**

Run:

```powershell
cargo test --no-run
```

Expected: compile succeeds.

- [ ] **Step 8: Commit**

```powershell
git add src/ui/translate_window.rs src/ui/edit.rs tests/workflow_tests.rs
git commit -m "feat: modernize translation edit controls"
```

---

### Task 5: Full Verification and Manual Check Notes

**Files:**
- No required source files.
- Optionally modify: `docs/superpowers/specs/2026-06-21-edit-controls-design.md` only if implementation reveals a spec correction is necessary.

- [ ] **Step 1: Format**

Run:

```powershell
cargo fmt --check
```

Expected: success. If it fails, run `cargo fmt`, inspect `git diff`, and include formatting changes in the final verification commit if any.

- [ ] **Step 2: Run full tests**

Run:

```powershell
cargo test
```

Expected: all tests pass.

- [ ] **Step 3: Run compile-only check for final artifact**

Run:

```powershell
cargo test --no-run
```

Expected: compile succeeds.

- [ ] **Step 4: Manual Windows UI verification**

Run the app:

```powershell
cargo run
```

Check these items manually:

- Settings window single-line edits have a modern border.
- Focus on settings edits changes only the border, not layout.
- API Key show/hide keeps password mode behavior.
- Google profile hides network fields without stale pixels.
- Translation source and translated multiline edits have matching modern borders.
- Source multiline edit still accepts Chinese input, selection, copy, paste, scrolling, `Ctrl+A`, and `Escape`.
- Translated multiline edit is readonly but still allows selection, copy, and scrolling.
- Multiline scrollbars are not covered by the border.

- [ ] **Step 5: Commit final polish if needed**

If formatting or small fixes were needed:

```powershell
git add src/ui/edit.rs src/ui/settings_window.rs src/ui/translate_window.rs tests/settings_window_tests.rs tests/workflow_tests.rs
git commit -m "fix: polish edit control modernization"
```

If no files changed, skip this commit.

---

## Self-Review

- Spec coverage: the plan covers pure state/palette logic, known control mapping, settings single-line edits, translation multiline edits, native text rendering preservation, color message handling, focus repaint, regression tests, and manual verification.
- Placeholder scan: no placeholder markers are intentionally left in the implementation steps.
- Type consistency: public helpers used later are defined in `src/ui/edit.rs`: `edit_uses_native_border`, `is_modern_edit`, `handle_modern_edit_color`, `paint_modern_edit_border`, `install_modern_edit_focus_tracking`, and `invalidate_modern_edit_for_child`.
