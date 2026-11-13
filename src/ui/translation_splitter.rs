use crate::error::{AppError, Result};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreateSolidBrush, DeleteObject, EndPaint, FillRect, InvalidateRect, PAINTSTRUCT,
};
use windows::Win32::UI::Controls::WM_MOUSELEAVE;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    ReleaseCapture, SetCapture, TME_LEAVE, TRACKMOUSEEVENT, TrackMouseEvent,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CS_DBLCLKS, CreateWindowExW, DefWindowProcW, GWLP_USERDATA, GetClientRect, GetCursorPos,
    GetParent, GetWindowLongPtrW, HMENU, IDC_SIZENS, LoadCursorW, RegisterClassW,
    SetWindowLongPtrW, WINDOW_EX_STYLE, WM_CANCELMODE, WM_CAPTURECHANGED, WM_CREATE, WM_ERASEBKGND,
    WM_LBUTTONDBLCLK, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE, WM_NCDESTROY, WM_PAINT,
    WNDCLASSW, WS_CHILD,
};
use windows::core::PCWSTR;

use super::translate_window::{
    current_window_layout, notify_source_ratio_changed, set_window_source_ratio,
};

#[derive(Clone, Copy, Default)]
struct SplitterState {
    hovered: bool,
    dragging: bool,
    start_cursor_y: i32,
    start_source_height: i32,
    combined_height: i32,
}

pub(super) fn create(parent: HWND, id: isize) -> Result<()> {
    let class_name: Vec<u16> = "ait_translation_splitter\0".encode_utf16().collect();
    unsafe {
        let class = WNDCLASSW {
            style: CS_DBLCLKS,
            lpfnWndProc: Some(window_proc),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            hCursor: LoadCursorW(None, IDC_SIZENS).unwrap_or_default(),
            ..Default::default()
        };
        if RegisterClassW(&class) == 0 {
            return Err(AppError::Windows("注册分隔条窗口类失败".to_string()));
        }
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR::null(),
            WS_CHILD,
            0,
            0,
            1,
            1,
            Some(parent),
            Some(HMENU(id as _)),
            None,
            None,
        )
        .map_err(|err| AppError::Windows(format!("创建分隔条失败: {err}")))?;
    }
    Ok(())
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_CREATE {
        let state = Box::into_raw(Box::new(SplitterState::default()));
        unsafe {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, state as isize);
        }
        return LRESULT(0);
    }
    let ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *mut SplitterState;
    if msg == WM_NCDESTROY {
        if !ptr.is_null() {
            unsafe {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                drop(Box::from_raw(ptr));
            }
        }
        return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
    }
    if ptr.is_null() {
        return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
    }
    // Copy state so calls that synchronously dispatch another window message
    // never retain a mutable reference to the same allocation.
    let mut state = unsafe { *ptr };
    let Ok(parent) = (unsafe { GetParent(hwnd) }) else {
        return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
    };

    match msg {
        WM_LBUTTONDOWN => {
            let mut cursor = POINT::default();
            if let Ok(layout) = current_window_layout(parent)
                && unsafe { GetCursorPos(&mut cursor) }.is_ok()
            {
                state.dragging = true;
                state.start_cursor_y = cursor.y;
                state.start_source_height = layout.source_edit.height;
                state.combined_height = layout.source_edit.height + layout.translated_edit.height;
                unsafe {
                    *ptr = state;
                    SetCapture(hwnd);
                    let _ = InvalidateRect(Some(hwnd), None, false);
                }
            }
            LRESULT(0)
        }
        WM_MOUSEMOVE => {
            if !state.hovered {
                state.hovered = true;
                unsafe {
                    *ptr = state;
                    let mut event = TRACKMOUSEEVENT {
                        cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                        dwFlags: TME_LEAVE,
                        hwndTrack: hwnd,
                        dwHoverTime: 0,
                    };
                    let _ = TrackMouseEvent(&mut event);
                    let _ = InvalidateRect(Some(hwnd), None, false);
                }
            }
            if state.dragging {
                let mut cursor = POINT::default();
                if unsafe { GetCursorPos(&mut cursor) }.is_ok() {
                    let total = state.combined_height.max(2);
                    let minimum = super::theme::scale(48).min(total / 2).max(1);
                    let height = (state.start_source_height + cursor.y - state.start_cursor_y)
                        .clamp(minimum, total - minimum);
                    let ratio = (i64::from(height) * i64::from(crate::config::PANE_RATIO_SCALE)
                        / i64::from(total)) as u32;
                    if let Err(err) = set_window_source_ratio(parent, ratio) {
                        tracing::warn!(error = %err, "drag translation splitter failed");
                    }
                }
            }
            LRESULT(0)
        }
        WM_LBUTTONUP | WM_CAPTURECHANGED => {
            if state.dragging {
                state.dragging = false;
                unsafe {
                    *ptr = state;
                    if msg == WM_LBUTTONUP {
                        let _ = ReleaseCapture();
                    }
                    let _ = InvalidateRect(Some(hwnd), None, false);
                }
                notify_source_ratio_changed(parent);
            }
            LRESULT(0)
        }
        WM_CANCELMODE => {
            if state.dragging {
                unsafe {
                    let _ = ReleaseCapture();
                }
            }
            LRESULT(0)
        }
        WM_LBUTTONDBLCLK => {
            match set_window_source_ratio(parent, crate::config::PANE_RATIO_SCALE / 2) {
                Ok(()) => notify_source_ratio_changed(parent),
                Err(err) => tracing::warn!(error = %err, "reset translation splitter failed"),
            }
            LRESULT(0)
        }
        WM_MOUSELEAVE => {
            state.hovered = false;
            unsafe {
                *ptr = state;
                let _ = InvalidateRect(Some(hwnd), None, false);
            }
            LRESULT(0)
        }
        WM_PAINT => {
            unsafe {
                paint(hwnd, state.hovered || state.dragging);
            }
            LRESULT(0)
        }
        WM_ERASEBKGND => LRESULT(1),
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

unsafe fn paint(hwnd: HWND, active: bool) {
    unsafe {
        let mut ps = PAINTSTRUCT::default();
        let hdc = BeginPaint(hwnd, &mut ps);
        let mut rect = RECT::default();
        let _ = GetClientRect(hwnd, &mut rect);
        FillRect(hdc, &rect, super::theme::surface_brush());
        let width = super::theme::scale(64).min(rect.right).max(1);
        let height = super::theme::scale(2).min(rect.bottom).max(1);
        let left = (rect.right - width) / 2;
        let top = (rect.bottom - height) / 2;
        let grip = RECT {
            left,
            top,
            right: left + width,
            bottom: top + height,
        };
        let color = if active {
            super::theme::colors().primary
        } else {
            super::theme::colors().border
        };
        let brush = CreateSolidBrush(color.colorref());
        FillRect(hdc, &grip, brush);
        let _ = DeleteObject(brush.into());
        let _ = EndPaint(hwnd, &ps);
    }
}
