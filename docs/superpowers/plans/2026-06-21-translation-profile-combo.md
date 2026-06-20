# Translation Profile Combo Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. Do not use subagent-driven development in this repository.

**Goal:** Modernize only the translation window's top profile `COMBOBOX` while preserving native ComboBox behavior.

**Architecture:** Add a focused `src/ui/combo.rs` module that mirrors the existing `button` and `edit` style modules: pure palette/mapping helpers are testable on every platform, while Windows-only helpers install a subclass and paint the parent-owned border. `src/ui/translate_window.rs` remains responsible for creating the actual Win32 control, forwarding ComboBox notifications, and painting the modern border during `WM_PAINT`.

**Tech Stack:** Rust, Win32 via the `windows` crate, GDI drawing, existing Cargo test suite.

---

## File Structure

- Create `src/ui/combo.rs`: ComboBox visual state, palette, control mapping, native-border policy, parent-frame rect helper, Windows subclass/install/paint helpers, and unit tests.
- Modify `src/ui/mod.rs`: export `combo` behind `#[cfg(windows)]`, matching the current `button` and `edit` modules.
- Modify `src/ui/translate_window.rs`: make `ID_PROFILE_COMBO` visible to `ui::combo`, remove native border for modern ComboBox creation, install ComboBox tracking, paint the border, and handle `CBN_DROPDOWN` / `CBN_CLOSEUP`.
- Test with `cargo test`.

## Task 1: Add Pure ComboBox Style Model

**Files:**
- Create: `src/ui/combo.rs`

- [ ] **Step 1: Write the failing tests and pure model skeleton**

Create `src/ui/combo.rs` with this initial content:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComboVisualState {
    pub focused: bool,
    pub dropped: bool,
    pub disabled: bool,
}

impl ComboVisualState {
    pub fn normal() -> Self {
        Self {
            focused: false,
            dropped: false,
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
pub struct ComboPalette {
    pub background: RgbColor,
    pub border: RgbColor,
    pub text: RgbColor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComboTextRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

pub fn combo_palette(_state: ComboVisualState) -> ComboPalette {
    unimplemented!("combo palette")
}

pub fn is_modern_combo(_id: usize) -> bool {
    unimplemented!("modern combo mapping")
}

pub fn combo_uses_native_border(_id: usize) -> bool {
    unimplemented!("combo native border policy")
}

pub fn modern_combo_frame_rect(left: i32, top: i32, right: i32, bottom: i32) -> ComboTextRect {
    unimplemented!("modern combo frame rect")
}

#[cfg(test)]
mod tests {
    use super::{
        ComboVisualState, RgbColor, combo_palette, combo_uses_native_border, is_modern_combo,
        modern_combo_frame_rect,
    };

    #[test]
    fn maps_translation_profile_combo() {
        assert!(is_modern_combo(2106));
    }

    #[test]
    fn ignores_unknown_controls() {
        assert!(!is_modern_combo(9999));
        assert!(combo_uses_native_border(9999));
    }

    #[test]
    fn modern_combo_does_not_use_native_border() {
        assert!(!combo_uses_native_border(2106));
    }

    #[test]
    fn normal_combo_uses_white_surface() {
        let palette = combo_palette(ComboVisualState::normal());
        assert_eq!(palette.background, RgbColor::new(255, 255, 255));
        assert_eq!(palette.border, RgbColor::new(203, 213, 225));
        assert_eq!(palette.text, RgbColor::new(31, 41, 55));
    }

    #[test]
    fn focused_combo_uses_blue_border() {
        let palette = combo_palette(ComboVisualState {
            focused: true,
            ..ComboVisualState::normal()
        });
        assert_eq!(palette.border, RgbColor::new(37, 99, 235));
    }

    #[test]
    fn dropped_combo_uses_active_border() {
        let palette = combo_palette(ComboVisualState {
            dropped: true,
            ..ComboVisualState::normal()
        });
        assert_eq!(palette.border, RgbColor::new(37, 99, 235));
    }

    #[test]
    fn disabled_combo_uses_muted_colors() {
        let palette = combo_palette(ComboVisualState {
            disabled: true,
            ..ComboVisualState::normal()
        });
        assert_eq!(palette.background, RgbColor::new(243, 244, 246));
        assert_eq!(palette.border, RgbColor::new(209, 213, 219));
        assert_eq!(palette.text, RgbColor::new(156, 163, 175));
    }

    #[test]
    fn frame_rect_matches_control_bounds() {
        let rect = modern_combo_frame_rect(408, 12, 588, 38);
        assert_eq!(rect.left, 408);
        assert_eq!(rect.top, 12);
        assert_eq!(rect.right, 588);
        assert_eq!(rect.bottom, 38);
    }
}
```

- [ ] **Step 2: Run the focused test to verify it fails**

Run:

```powershell
cargo test ui::combo
```

Expected: compile/test failure because `combo.rs` is not exported yet or because the functions contain `unimplemented!`.

- [ ] **Step 3: Export the module for tests**

Modify `src/ui/mod.rs` to include:

```rust
#[cfg(windows)]
pub mod button;
#[cfg(windows)]
pub mod combo;
#[cfg(windows)]
pub mod edit;
#[cfg(windows)]
pub mod font;
pub mod settings_window;
pub mod translate_window;
pub mod tray;
```

- [ ] **Step 4: Implement the minimal pure logic**

Replace the unimplemented functions in `src/ui/combo.rs` with:

```rust
pub fn combo_palette(state: ComboVisualState) -> ComboPalette {
    if state.disabled {
        return ComboPalette {
            background: RgbColor::new(243, 244, 246),
            border: RgbColor::new(209, 213, 219),
            text: RgbColor::new(156, 163, 175),
        };
    }

    ComboPalette {
        background: RgbColor::new(255, 255, 255),
        border: if state.focused || state.dropped {
            RgbColor::new(37, 99, 235)
        } else {
            RgbColor::new(203, 213, 225)
        },
        text: RgbColor::new(31, 41, 55),
    }
}

pub fn is_modern_combo(id: usize) -> bool {
    id == 2106
}

pub fn combo_uses_native_border(id: usize) -> bool {
    !is_modern_combo(id)
}

pub fn modern_combo_frame_rect(left: i32, top: i32, right: i32, bottom: i32) -> ComboTextRect {
    ComboTextRect {
        left,
        top,
        right,
        bottom,
    }
}
```

- [ ] **Step 5: Run the focused test to verify it passes**

Run:

```powershell
cargo test ui::combo
```

Expected: all `ui::combo` tests pass.

- [ ] **Step 6: Commit**

Run:

```powershell
git add src\ui\mod.rs src\ui\combo.rs
git commit -m "test: add combo style model"
```

## Task 2: Add Windows ComboBox Tracking and Border Painting Helpers

**Files:**
- Modify: `src/ui/combo.rs`

- [ ] **Step 1: Add Windows color conversion and dropped-state storage**

Append this Windows-only support code above the test module in `src/ui/combo.rs`:

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
fn dropped_combo() -> &'static std::sync::Mutex<isize> {
    use std::sync::{Mutex, OnceLock};

    static DROPPED_COMBO: OnceLock<Mutex<isize>> = OnceLock::new();
    DROPPED_COMBO.get_or_init(|| Mutex::new(0))
}

#[cfg(windows)]
pub fn set_combo_dropped(hwnd: windows::Win32::Foundation::HWND, dropped: bool) {
    let mut current = dropped_combo().lock().unwrap();
    if dropped {
        *current = hwnd.0 as isize;
    } else if *current == hwnd.0 as isize {
        *current = 0;
    }
}

#[cfg(windows)]
fn is_combo_dropped(hwnd: windows::Win32::Foundation::HWND) -> bool {
    let current = dropped_combo().lock().unwrap();
    *current == hwnd.0 as isize
}
```

- [ ] **Step 2: Add border invalidation and painting helpers**

Append:

```rust
#[cfg(windows)]
pub unsafe fn paint_modern_combo_border(parent: windows::Win32::Foundation::HWND, control_id: i32) {
    use windows::Win32::Graphics::Gdi::MapWindowPoints;
    use windows::Win32::Graphics::Gdi::{
        CreatePen, DeleteObject, GetDC, GetStockObject, NULL_BRUSH, PS_SOLID, ReleaseDC, RoundRect,
        SelectObject,
    };
    use windows::Win32::UI::WindowsAndMessaging::{GetDlgItem, GetWindowRect};

    let Ok(child) = (unsafe { GetDlgItem(Some(parent), control_id) }) else {
        return;
    };
    let state = unsafe { combo_visual_state_for_child(child) };
    let palette = combo_palette(state);
    let mut rect = windows::Win32::Foundation::RECT::default();
    if unsafe { GetWindowRect(child, &mut rect).is_err() } {
        return;
    }
    let mut points = [
        windows::Win32::Foundation::POINT {
            x: rect.left,
            y: rect.top,
        },
        windows::Win32::Foundation::POINT {
            x: rect.right,
            y: rect.bottom,
        },
    ];
    unsafe {
        let _ = MapWindowPoints(None, Some(parent), &mut points);
    }
    let frame = modern_combo_frame_rect(points[0].x, points[0].y, points[1].x, points[1].y);
    rect.left = frame.left;
    rect.top = frame.top;
    rect.right = frame.right;
    rect.bottom = frame.bottom;

    let hdc = unsafe { GetDC(Some(parent)) };
    if hdc.is_invalid() {
        return;
    }

    let pen = unsafe { CreatePen(PS_SOLID, 1, palette.border.colorref()) };
    if pen.is_invalid() {
        unsafe {
            let _ = ReleaseDC(Some(parent), hdc);
        }
        return;
    }
    let old_pen = unsafe { SelectObject(hdc, pen.into()) };
    let old_brush = unsafe { SelectObject(hdc, GetStockObject(NULL_BRUSH)) };
    unsafe {
        let _ = RoundRect(hdc, rect.left, rect.top, rect.right, rect.bottom, 7, 7);
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
        let _ = ReleaseDC(Some(parent), hdc);
    }
}

#[cfg(windows)]
unsafe fn combo_visual_state_for_child(hwnd: windows::Win32::Foundation::HWND) -> ComboVisualState {
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetFocus, IsWindowEnabled};

    ComboVisualState {
        focused: unsafe { GetFocus() } == hwnd,
        dropped: is_combo_dropped(hwnd),
        disabled: !unsafe { IsWindowEnabled(hwnd).as_bool() },
    }
}

#[cfg(windows)]
pub unsafe fn invalidate_modern_combo_for_child(
    parent: windows::Win32::Foundation::HWND,
    child: windows::Win32::Foundation::HWND,
) {
    use windows::Win32::Graphics::Gdi::InvalidateRect;
    use windows::Win32::Graphics::Gdi::MapWindowPoints;
    use windows::Win32::UI::WindowsAndMessaging::GetWindowRect;

    let mut rect = windows::Win32::Foundation::RECT::default();
    if unsafe { GetWindowRect(child, &mut rect).is_ok() } {
        let mut points = [
            windows::Win32::Foundation::POINT {
                x: rect.left,
                y: rect.top,
            },
            windows::Win32::Foundation::POINT {
                x: rect.right,
                y: rect.bottom,
            },
        ];
        unsafe {
            let _ = MapWindowPoints(None, Some(parent), &mut points);
        }
        let frame = modern_combo_frame_rect(points[0].x, points[0].y, points[1].x, points[1].y);
        rect.left = frame.left;
        rect.top = frame.top;
        rect.right = frame.right;
        rect.bottom = frame.bottom;
        unsafe {
            let _ = InvalidateRect(Some(parent), Some(&rect), false);
        }
    }
}
```

- [ ] **Step 3: Add subclass installation**

Append:

```rust
#[cfg(windows)]
pub fn install_modern_combo_tracking(
    hwnd: windows::Win32::Foundation::HWND,
) -> crate::error::Result<()> {
    use windows::Win32::UI::Shell::SetWindowSubclass;

    unsafe {
        if SetWindowSubclass(
            hwnd,
            Some(modern_combo_subclass_proc),
            MODERN_COMBO_SUBCLASS_ID,
            0,
        )
        .as_bool()
        {
            Ok(())
        } else {
            Err(crate::error::AppError::Windows(
                "安装下拉框焦点处理失败".to_string(),
            ))
        }
    }
}

#[cfg(windows)]
const MODERN_COMBO_SUBCLASS_ID: usize = 3;

#[cfg(windows)]
unsafe extern "system" fn modern_combo_subclass_proc(
    hwnd: windows::Win32::Foundation::HWND,
    msg: u32,
    wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
    subclass_id: usize,
    _ref_data: usize,
) -> windows::Win32::Foundation::LRESULT {
    use windows::Win32::UI::Shell::{DefSubclassProc, RemoveWindowSubclass};
    use windows::Win32::UI::WindowsAndMessaging::{
        GetParent, WM_ENABLE, WM_KILLFOCUS, WM_NCDESTROY, WM_SETFOCUS,
    };

    if msg == WM_SETFOCUS || msg == WM_KILLFOCUS || msg == WM_ENABLE {
        if let Ok(parent) = unsafe { GetParent(hwnd) } {
            unsafe {
                invalidate_modern_combo_for_child(parent, hwnd);
            }
        }
    }

    if msg == WM_NCDESTROY {
        set_combo_dropped(hwnd, false);
        unsafe {
            let _ = RemoveWindowSubclass(hwnd, Some(modern_combo_subclass_proc), subclass_id);
            return DefSubclassProc(hwnd, msg, wparam, lparam);
        }
    }

    unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) }
}
```

- [ ] **Step 4: Add Windows-only dropped-state round-trip test**

Inside the existing `#[cfg(test)] mod tests`, add:

```rust
    #[cfg(windows)]
    #[test]
    fn dropped_state_round_trips_for_combo_hwnd() {
        let hwnd = windows::Win32::Foundation::HWND(2106 as *mut core::ffi::c_void);

        super::set_combo_dropped(hwnd, false);
        super::set_combo_dropped(hwnd, true);
        assert!(super::is_combo_dropped(hwnd));

        super::set_combo_dropped(hwnd, false);
        assert!(!super::is_combo_dropped(hwnd));
    }
```

- [ ] **Step 5: Run tests**

Run:

```powershell
cargo test ui::combo
```

Expected: all combo tests pass.

- [ ] **Step 6: Commit**

Run:

```powershell
git add src\ui\combo.rs
git commit -m "feat: add combo border helpers"
```

## Task 3: Wire Modern ComboBox Into the Translation Window

**Files:**
- Modify: `src/ui/translate_window.rs`

- [ ] **Step 1: Make the profile ComboBox id visible to `ui::combo`**

Change the id definition near the top of `src/ui/translate_window.rs` from:

```rust
const ID_PROFILE_COMBO: isize = 2106;
```

to:

```rust
pub(crate) const ID_PROFILE_COMBO: isize = 2106;
```

- [ ] **Step 2: Update `create_combo` to use the modern border policy and install tracking**

Replace the current `create_combo` function with:

```rust
#[cfg(windows)]
fn create_combo(
    parent: windows::Win32::Foundation::HWND,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    id: isize,
) -> Result<windows::Win32::Foundation::HWND> {
    use windows::Win32::UI::WindowsAndMessaging::{CBS_DROPDOWNLIST, WINDOW_STYLE, WS_VSCROLL};
    let hwnd = create_control(
        parent,
        "COMBOBOX",
        "",
        x,
        y,
        width,
        height,
        id,
        WINDOW_STYLE(CBS_DROPDOWNLIST as u32 | WS_VSCROLL.0),
        crate::ui::combo::combo_uses_native_border(id as usize),
    )?;
    if crate::ui::combo::is_modern_combo(id as usize) {
        crate::ui::combo::install_modern_combo_tracking(hwnd)?;
    }
    Ok(hwnd)
}
```

- [ ] **Step 3: Paint the ComboBox border during `WM_PAINT`**

In the `WM_PAINT` block, after the two existing edit border calls:

```rust
    if msg == WM_PAINT {
        let result = unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
        unsafe {
            crate::ui::edit::paint_modern_edit_border(hwnd, ID_SOURCE_EDIT as i32);
            crate::ui::edit::paint_modern_edit_border(hwnd, ID_TRANSLATED_EDIT as i32);
            crate::ui::combo::paint_modern_combo_border(hwnd, ID_PROFILE_COMBO as i32);
        }
        return result;
    }
```

- [ ] **Step 4: Handle dropdown open/close notifications without changing selection behavior**

In the existing `WM_COMMAND` match, add these arms before the `CBN_SELCHANGE` arm:

```rust
            command
                if command == ID_PROFILE_COMBO as usize
                    && notification
                        == windows::Win32::UI::WindowsAndMessaging::CBN_DROPDOWN as usize =>
            {
                if let Ok(combo) = unsafe {
                    windows::Win32::UI::WindowsAndMessaging::GetDlgItem(
                        Some(hwnd),
                        ID_PROFILE_COMBO as i32,
                    )
                } {
                    crate::ui::combo::set_combo_dropped(combo, true);
                    unsafe {
                        crate::ui::combo::invalidate_modern_combo_for_child(hwnd, combo);
                    }
                }
                return LRESULT(0);
            }
            command
                if command == ID_PROFILE_COMBO as usize
                    && notification
                        == windows::Win32::UI::WindowsAndMessaging::CBN_CLOSEUP as usize =>
            {
                if let Ok(combo) = unsafe {
                    windows::Win32::UI::WindowsAndMessaging::GetDlgItem(
                        Some(hwnd),
                        ID_PROFILE_COMBO as i32,
                    )
                } {
                    crate::ui::combo::set_combo_dropped(combo, false);
                    unsafe {
                        crate::ui::combo::invalidate_modern_combo_for_child(hwnd, combo);
                    }
                }
                return LRESULT(0);
            }
```

- [ ] **Step 5: Run the full test suite**

Run:

```powershell
cargo test
```

Expected: all tests pass.

- [ ] **Step 6: Commit**

Run:

```powershell
git add src\ui\translate_window.rs
git commit -m "feat: modernize translation profile combo"
```

## Task 4: Verify Formatting, Tests, and Worktree State

**Files:**
- Modify only if formatting reports changes.

- [ ] **Step 1: Format the code**

Run:

```powershell
cargo fmt
```

Expected: command exits successfully. If files change, review with `git diff`.

- [ ] **Step 2: Run all tests again**

Run:

```powershell
cargo test
```

Expected: all tests pass.

- [ ] **Step 3: Check final diff**

Run:

```powershell
git status --short
git diff
```

Expected: no unexpected files. If `cargo fmt` changed tracked files, commit those formatting changes:

```powershell
git add src\ui\combo.rs src\ui\mod.rs src\ui\translate_window.rs
git commit -m "style: format combo modernization"
```

- [ ] **Step 4: Manual verification on Windows**

Run the app using the normal project workflow, open the translation window, and verify:

- The top profile ComboBox has a rounded modern border.
- Focus and opened dropdown use the active blue border.
- The dropdown list opens and selection still works.
- Switching profile still posts the existing profile-changed workflow.
- Resizing the translation window does not leave border artifacts.

If manual verification finds a behavior bug, fix it in the smallest affected task area, rerun `cargo fmt` and `cargo test`, then commit with a precise message.

## Self-Review

- Spec coverage: the plan covers a dedicated `ui::combo` module, the single `ID_PROFILE_COMBO` scope, native behavior preservation, modern border painting, dropdown open/close state, tests, and manual verification.
- Placeholder scan: no placeholders are intentionally left in the implementation steps.
- Type consistency: `ComboVisualState`, `ComboPalette`, `RgbColor`, `ComboTextRect`, `is_modern_combo`, `combo_uses_native_border`, `modern_combo_frame_rect`, `install_modern_combo_tracking`, `paint_modern_combo_border`, `set_combo_dropped`, and `invalidate_modern_combo_for_child` are introduced before they are used by `translate_window.rs`.
