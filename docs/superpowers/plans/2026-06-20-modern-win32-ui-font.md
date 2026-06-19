# Modern Win32 UI Font Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. Do not use `superpowers:subagent-driven-development`; this repository explicitly forbids it.

**Goal:** Apply a clearer modern Windows UI font to all native controls in the translation and settings windows.

**Architecture:** Add a small Windows-only font helper under `src/ui/font.rs`. The helper owns one reusable `HFONT`, computes the 9pt logical height from DPI, and applies the font to Win32 controls through `WM_SETFONT`.

**Tech Stack:** Rust 2024, `windows` crate Win32 bindings, GDI `CreateFontW`, Win32 `WM_SETFONT`, existing `cargo test` suite.

---

## File Structure

- Create `src/ui/font.rs`: Windows-only font helper plus pure testable point-to-logical-height conversion.
- Modify `src/ui/mod.rs`: expose the new `font` module on Windows.
- Modify `src/ui/translate_window.rs`: apply the shared UI font inside the existing `create_control` helper after each control is created.
- Modify `src/ui/settings_window.rs`: apply the shared UI font inside the existing `create_control` helper after each control is created.

---

### Task 1: Add Shared Win32 UI Font Helper

**Files:**
- Create: `src/ui/font.rs`
- Modify: `src/ui/mod.rs`

- [ ] **Step 1: Write the failing test**

Create `src/ui/font.rs` with the pure helper and tests first:

```rust
#[cfg(windows)]
use crate::error::{AppError, Result};

pub fn point_size_to_logical_height(point_size: i32, dpi: i32) -> i32 {
    -((point_size * dpi) / 72)
}

#[cfg(test)]
mod tests {
    use super::point_size_to_logical_height;

    #[test]
    fn point_size_to_logical_height_uses_negative_logical_height() {
        assert_eq!(point_size_to_logical_height(9, 96), -12);
        assert_eq!(point_size_to_logical_height(9, 144), -18);
    }
}
```

Modify `src/ui/mod.rs`:

```rust
#[cfg(windows)]
pub mod font;
pub mod settings_window;
pub mod translate_window;
pub mod tray;
```

- [ ] **Step 2: Run the focused test**

Run:

```powershell
cargo test point_size_to_logical_height_uses_negative_logical_height
```

Expected: PASS. This confirms the pure conversion behavior before adding Win32 API calls.

- [ ] **Step 3: Add the Win32 implementation**

Replace `src/ui/font.rs` with:

```rust
#[cfg(windows)]
use crate::error::{AppError, Result};

pub fn point_size_to_logical_height(point_size: i32, dpi: i32) -> i32 {
    -((point_size * dpi) / 72)
}

#[cfg(windows)]
pub fn apply_ui_font(hwnd: windows::Win32::Foundation::HWND) -> Result<()> {
    use windows::Win32::Foundation::{LPARAM, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{SendMessageW, WM_SETFONT};

    let font = ui_font()?;
    unsafe {
        let _ = SendMessageW(
            hwnd,
            WM_SETFONT,
            Some(WPARAM(font.0 as usize)),
            Some(LPARAM(1)),
        );
    }
    Ok(())
}

#[cfg(windows)]
fn ui_font() -> Result<windows::Win32::Graphics::Gdi::HFONT> {
    use std::sync::OnceLock;
    use windows::Win32::Graphics::Gdi::HFONT;

    static FONT: OnceLock<HFONT> = OnceLock::new();
    if let Some(font) = FONT.get() {
        return Ok(*font);
    }

    let font = create_ui_font()?;
    let _ = FONT.set(font);
    Ok(*FONT.get().unwrap_or(&font))
}

#[cfg(windows)]
fn create_ui_font() -> Result<windows::Win32::Graphics::Gdi::HFONT> {
    use windows::Win32::Graphics::Gdi::{
        CLIP_DEFAULT_PRECIS, CreateFontW, DEFAULT_CHARSET, DEFAULT_PITCH, FF_DONTCARE,
        GetDeviceCaps, HDC, OUT_DEFAULT_PRECIS, PROOF_QUALITY, VERTRES,
    };
    use windows::Win32::UI::WindowsAndMessaging::{GetDC, ReleaseDC};
    use windows::core::PCWSTR;

    let hdc = unsafe { GetDC(None) };
    let dpi = if hdc == HDC::default() {
        96
    } else {
        let dpi = unsafe { GetDeviceCaps(hdc, VERTRES) };
        unsafe {
            let _ = ReleaseDC(None, hdc);
        }
        dpi.max(1)
    };
    let face = wide("Microsoft YaHei UI");
    let font = unsafe {
        CreateFontW(
            point_size_to_logical_height(9, dpi),
            0,
            0,
            0,
            400,
            0,
            0,
            0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            PROOF_QUALITY,
            DEFAULT_PITCH | FF_DONTCARE,
            PCWSTR(face.as_ptr()),
        )
    };

    if font == windows::Win32::Graphics::Gdi::HFONT::default() {
        Err(AppError::Windows("创建 UI 字体失败".to_string()))
    } else {
        Ok(font)
    }
}

#[cfg(windows)]
fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(Some(0)).collect()
}

#[cfg(test)]
mod tests {
    use super::point_size_to_logical_height;

    #[test]
    fn point_size_to_logical_height_uses_negative_logical_height() {
        assert_eq!(point_size_to_logical_height(9, 96), -12);
        assert_eq!(point_size_to_logical_height(9, 144), -18);
    }
}
```

- [ ] **Step 4: Run the focused test again**

Run:

```powershell
cargo test point_size_to_logical_height_uses_negative_logical_height
```

Expected: PASS.

- [ ] **Step 5: Commit**

Run:

```powershell
git add src\ui\font.rs src\ui\mod.rs
git commit -m "feat: add shared win32 ui font"
```

Expected: commit succeeds.

---

### Task 2: Apply Font to Translation Window Controls

**Files:**
- Modify: `src/ui/translate_window.rs`

- [ ] **Step 1: Update `create_control`**

In `src/ui/translate_window.rs`, replace the body of `create_control` with this implementation:

```rust
#[cfg(windows)]
// Mirrors CreateWindowExW inputs so call sites remain explicit about control layout and style.
#[allow(clippy::too_many_arguments)]
fn create_control(
    parent: windows::Win32::Foundation::HWND,
    class_name: &str,
    text: &str,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    id: isize,
    extra_style: windows::Win32::UI::WindowsAndMessaging::WINDOW_STYLE,
) -> Result<windows::Win32::Foundation::HWND> {
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, HMENU, WINDOW_EX_STYLE, WS_BORDER, WS_CHILD, WS_VISIBLE,
    };
    use windows::core::PCWSTR;

    let hwnd = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(wide(class_name).as_ptr()),
            PCWSTR(wide(text).as_ptr()),
            WS_CHILD | WS_VISIBLE | WS_BORDER | extra_style,
            x,
            y,
            width,
            height,
            Some(parent),
            if id == 0 { None } else { Some(HMENU(id as _)) },
            None,
            None,
        )
        .map_err(|err| AppError::Windows(format!("创建控件失败: {err}")))?
    };
    crate::ui::font::apply_ui_font(hwnd)?;
    Ok(hwnd)
}
```

- [ ] **Step 2: Run translation-window related tests**

Run:

```powershell
cargo test translation_window workflow_tests
```

Expected: all selected tests pass.

- [ ] **Step 3: Commit**

Run:

```powershell
git add src\ui\translate_window.rs
git commit -m "feat: apply ui font to translation window"
```

Expected: commit succeeds.

---

### Task 3: Apply Font to Settings Window Controls and Verify

**Files:**
- Modify: `src/ui/settings_window.rs`

- [ ] **Step 1: Update `create_control`**

In `src/ui/settings_window.rs`, replace the body of `create_control` with this implementation:

```rust
#[cfg(windows)]
// Mirrors CreateWindowExW inputs so call sites remain explicit about control layout and style.
#[allow(clippy::too_many_arguments)]
fn create_control(
    parent: windows::Win32::Foundation::HWND,
    class_name: &str,
    text: &str,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    id: isize,
    extra_style: windows::Win32::UI::WindowsAndMessaging::WINDOW_STYLE,
    bordered: bool,
) -> Result<windows::Win32::Foundation::HWND> {
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, HMENU, WINDOW_EX_STYLE, WS_BORDER, WS_CHILD, WS_VISIBLE,
    };
    use windows::core::PCWSTR;

    let hwnd = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(wide(class_name).as_ptr()),
            PCWSTR(wide(text).as_ptr()),
            WS_CHILD
                | WS_VISIBLE
                | if bordered {
                    WS_BORDER
                } else {
                    windows::Win32::UI::WindowsAndMessaging::WINDOW_STYLE(0)
                }
                | extra_style,
            x,
            y,
            width,
            height,
            Some(parent),
            if id == 0 { None } else { Some(HMENU(id as _)) },
            None,
            None,
        )
        .map_err(|err| AppError::Windows(format!("创建控件失败: {err}")))?
    };
    crate::ui::font::apply_ui_font(hwnd)?;
    Ok(hwnd)
}
```

- [ ] **Step 2: Run settings-window related tests**

Run:

```powershell
cargo test settings_window
```

Expected: all selected tests pass.

- [ ] **Step 3: Run formatting and full tests**

Run:

```powershell
cargo fmt --check
cargo test
```

Expected: both commands pass.

- [ ] **Step 4: Commit**

Run:

```powershell
git add src\ui\settings_window.rs
git commit -m "feat: apply ui font to settings window"
```

Expected: commit succeeds.

---

## Self-Review

- Spec coverage: Task 1 creates the shared Windows-only font helper and DPI-based 9pt conversion. Task 2 applies it to the translation window. Task 3 applies it to the settings window and runs full verification.
- Placeholder scan: no placeholder steps remain; each code-changing step includes concrete code.
- Type consistency: `apply_ui_font` returns the repository `Result<()>`; both window `create_control` helpers already return that same `Result` type, so `?` composes with existing error handling.
