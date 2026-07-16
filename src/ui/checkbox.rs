pub use crate::ui::theme::RgbColor;
use crate::ui::theme::{
    COLOR_BORDER, COLOR_DISABLED_BORDER, COLOR_DISABLED_SURFACE, COLOR_DISABLED_TEXT,
    COLOR_PRIMARY, COLOR_PRIMARY_HOVER, COLOR_PRIMARY_TEXT, COLOR_SURFACE, COLOR_SURFACE_SUBTLE,
    COLOR_TEXT, FOCUS_RING_INSET,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheckboxVisualState {
    pub checked: bool,
    pub hot: bool,
    pub disabled: bool,
    pub focused: bool,
}

impl CheckboxVisualState {
    pub fn normal() -> Self {
        Self {
            checked: false,
            hot: false,
            disabled: false,
            focused: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheckboxPalette {
    pub box_background: RgbColor,
    pub box_border: RgbColor,
    pub check: RgbColor,
    pub text: RgbColor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheckboxBoxRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

pub const CHECKBOX_BOX_SIZE: i32 = 18;
pub const CHECKBOX_TEXT_GAP: i32 = 8;

pub fn checkbox_palette(state: CheckboxVisualState) -> CheckboxPalette {
    if state.disabled {
        return CheckboxPalette {
            box_background: COLOR_DISABLED_SURFACE,
            box_border: COLOR_DISABLED_BORDER,
            check: COLOR_DISABLED_TEXT,
            text: COLOR_DISABLED_TEXT,
        };
    }

    if state.checked {
        let box_background = if state.hot {
            COLOR_PRIMARY_HOVER
        } else {
            COLOR_PRIMARY
        };
        return CheckboxPalette {
            box_background,
            box_border: box_background,
            check: COLOR_PRIMARY_TEXT,
            text: COLOR_TEXT,
        };
    }

    CheckboxPalette {
        box_background: if state.hot || state.focused {
            COLOR_SURFACE_SUBTLE
        } else {
            COLOR_SURFACE
        },
        box_border: if state.hot || state.focused {
            COLOR_PRIMARY
        } else {
            COLOR_BORDER
        },
        check: COLOR_PRIMARY_TEXT,
        text: COLOR_TEXT,
    }
}

pub fn checkbox_box_rect(control_height: i32) -> CheckboxBoxRect {
    let top = ((control_height - CHECKBOX_BOX_SIZE) / 2).max(0);
    CheckboxBoxRect {
        left: 0,
        top,
        right: CHECKBOX_BOX_SIZE,
        bottom: top + CHECKBOX_BOX_SIZE,
    }
}

pub fn checkbox_text_left() -> i32 {
    CHECKBOX_BOX_SIZE + CHECKBOX_TEXT_GAP
}

pub fn is_modern_checkbox(id: usize) -> bool {
    id == 3117
}

pub fn checkbox_uses_native_border(id: usize) -> bool {
    !is_modern_checkbox(id)
}

pub fn checkbox_toggled_state(checked: bool) -> bool {
    !checked
}

#[cfg(windows)]
pub fn set_owner_draw_checkbox_checked(hwnd: windows::Win32::Foundation::HWND, checked: bool) {
    owner_draw_checkbox_states()
        .lock()
        .unwrap()
        .insert(hwnd.0 as isize, checked);
}

#[cfg(windows)]
pub fn owner_draw_checkbox_checked(hwnd: windows::Win32::Foundation::HWND) -> bool {
    owner_draw_checkbox_states()
        .lock()
        .unwrap()
        .get(&(hwnd.0 as isize))
        .copied()
        .unwrap_or(false)
}

#[cfg(windows)]
fn clear_owner_draw_checkbox_state(hwnd: windows::Win32::Foundation::HWND) {
    owner_draw_checkbox_states()
        .lock()
        .unwrap()
        .remove(&(hwnd.0 as isize));
}

#[cfg(windows)]
fn owner_draw_checkbox_states() -> &'static std::sync::Mutex<std::collections::HashMap<isize, bool>>
{
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};

    static STATES: OnceLock<Mutex<HashMap<isize, bool>>> = OnceLock::new();
    STATES.get_or_init(|| Mutex::new(HashMap::new()))
}

#[cfg(windows)]
pub unsafe fn draw_owner_draw_checkbox(
    draw_item: *const windows::Win32::UI::Controls::DRAWITEMSTRUCT,
) -> bool {
    use windows::Win32::Graphics::Gdi::{
        BACKGROUND_MODE, CreatePen, CreateRoundRectRgn, CreateSolidBrush, DT_LEFT, DT_SINGLELINE,
        DT_VCENTER, DeleteObject, DrawTextW, FillRect, GetBkMode, GetStockObject, GetTextColor,
        HGDIOBJ, NULL_BRUSH, PS_SOLID, RoundRect, SelectClipRgn, SelectObject, SetBkMode,
        SetTextColor, TRANSPARENT,
    };
    use windows::Win32::UI::Controls::{ODS_CHECKED, ODS_DISABLED, ODS_FOCUS, ODS_HOTLIGHT};

    let Some(draw_item) = (unsafe { draw_item.as_ref() }) else {
        return false;
    };
    if !is_modern_checkbox(draw_item.CtlID as usize) {
        return false;
    }

    let state = CheckboxVisualState {
        checked: owner_draw_checkbox_checked(draw_item.hwndItem)
            || (draw_item.itemState.0 & ODS_CHECKED.0) != 0,
        hot: is_checkbox_hot(draw_item.hwndItem) || (draw_item.itemState.0 & ODS_HOTLIGHT.0) != 0,
        disabled: (draw_item.itemState.0 & ODS_DISABLED.0) != 0,
        focused: (draw_item.itemState.0 & ODS_FOCUS.0) != 0,
    };
    let palette = checkbox_palette(state);
    let hdc = draw_item.hDC;
    let rect = draw_item.rcItem;

    let background = unsafe {
        windows::Win32::Graphics::Gdi::HBRUSH(
            GetStockObject(windows::Win32::Graphics::Gdi::WHITE_BRUSH).0,
        )
    };
    unsafe {
        let _ = FillRect(hdc, &rect, background);
    }

    let box_size = crate::ui::theme::scale(CHECKBOX_BOX_SIZE);
    let box_top = (((rect.bottom - rect.top) - box_size) / 2).max(0);
    let box_frame = CheckboxBoxRect {
        left: 0,
        top: box_top,
        right: box_size,
        bottom: box_top + box_size,
    };
    let box_rect = windows::Win32::Foundation::RECT {
        left: rect.left + box_frame.left,
        top: rect.top + box_frame.top,
        right: rect.left + box_frame.right,
        bottom: rect.top + box_frame.bottom,
    };

    let box_background = unsafe { CreateSolidBrush(palette.box_background.colorref()) };
    let clip_region = unsafe {
        CreateRoundRectRgn(
            box_rect.left,
            box_rect.top,
            box_rect.right + 1,
            box_rect.bottom + 1,
            5,
            5,
        )
    };
    if !clip_region.is_invalid() {
        unsafe {
            let _ = SelectClipRgn(hdc, Some(clip_region));
        }
    }
    if !box_background.is_invalid() {
        unsafe {
            let _ = FillRect(hdc, &box_rect, box_background);
        }
    }
    if !clip_region.is_invalid() {
        unsafe {
            let _ = SelectClipRgn(hdc, None);
            let _ = DeleteObject(clip_region.into());
        }
    }

    let border_pen = unsafe { CreatePen(PS_SOLID, 1, palette.box_border.colorref()) };
    if !border_pen.is_invalid() {
        let old_pen = unsafe { SelectObject(hdc, border_pen.into()) };
        let old_brush = unsafe { SelectObject(hdc, GetStockObject(NULL_BRUSH)) };
        unsafe {
            let _ = RoundRect(
                hdc,
                box_rect.left,
                box_rect.top,
                box_rect.right,
                box_rect.bottom,
                5,
                5,
            );
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
            let _ = DeleteObject(border_pen.into());
        }
    }

    if state.checked {
        let check_pen = unsafe { CreatePen(PS_SOLID, 2, palette.check.colorref()) };
        if !check_pen.is_invalid() {
            let old_pen: HGDIOBJ = unsafe { SelectObject(hdc, check_pen.into()) };
            unsafe {
                let _ = windows::Win32::Graphics::Gdi::MoveToEx(
                    hdc,
                    box_rect.left + crate::ui::theme::scale(4),
                    box_rect.top + crate::ui::theme::scale(9),
                    None,
                );
                let _ = windows::Win32::Graphics::Gdi::LineTo(
                    hdc,
                    box_rect.left + crate::ui::theme::scale(8),
                    box_rect.top + crate::ui::theme::scale(13),
                );
                let _ = windows::Win32::Graphics::Gdi::LineTo(
                    hdc,
                    box_rect.left + crate::ui::theme::scale(14),
                    box_rect.top + crate::ui::theme::scale(5),
                );
            }
            if !old_pen.is_invalid() {
                unsafe {
                    let _ = SelectObject(hdc, old_pen);
                }
            }
            unsafe {
                let _ = DeleteObject(check_pen.into());
            }
        }
    }

    let mut text = checkbox_text(draw_item.hwndItem);
    let mut text_rect = windows::Win32::Foundation::RECT {
        left: rect.left + crate::ui::theme::scale(CHECKBOX_BOX_SIZE + CHECKBOX_TEXT_GAP),
        top: rect.top,
        right: rect.right,
        bottom: rect.bottom,
    };
    let old_bk_mode = unsafe { GetBkMode(hdc) };
    let old_text_color = unsafe { GetTextColor(hdc) };
    unsafe {
        let _ = SetBkMode(hdc, TRANSPARENT);
        let _ = SetTextColor(hdc, palette.text.colorref());
        let _ = DrawTextW(
            hdc,
            &mut text,
            &mut text_rect,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );
        let _ = SetTextColor(hdc, old_text_color);
        let _ = SetBkMode(hdc, BACKGROUND_MODE(old_bk_mode as u32));
    }

    if state.focused && !state.disabled {
        let focus_pen = unsafe { CreatePen(PS_SOLID, 1, COLOR_PRIMARY.colorref()) };
        if !focus_pen.is_invalid() {
            let old_pen = unsafe { SelectObject(hdc, focus_pen.into()) };
            let old_brush = unsafe { SelectObject(hdc, GetStockObject(NULL_BRUSH)) };
            unsafe {
                let _ = RoundRect(
                    hdc,
                    rect.left + FOCUS_RING_INSET,
                    rect.top + FOCUS_RING_INSET,
                    rect.right - FOCUS_RING_INSET,
                    rect.bottom - FOCUS_RING_INSET,
                    5,
                    5,
                );
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
                let _ = DeleteObject(focus_pen.into());
            }
        }
    }

    if !box_background.is_invalid() {
        unsafe {
            let _ = DeleteObject(box_background.into());
        }
    }
    true
}

#[cfg(windows)]
fn checkbox_text(hwnd: windows::Win32::Foundation::HWND) -> Vec<u16> {
    use windows::Win32::UI::WindowsAndMessaging::{GetWindowTextLengthW, GetWindowTextW};

    let len = unsafe { GetWindowTextLengthW(hwnd) };
    let mut text = vec![0u16; len as usize + 1];
    if len > 0 {
        let copied = unsafe { GetWindowTextW(hwnd, &mut text) };
        text.truncate(copied as usize);
    } else {
        text.clear();
    }
    text
}

#[cfg(windows)]
pub fn install_owner_draw_checkbox_hover(
    hwnd: windows::Win32::Foundation::HWND,
) -> crate::error::Result<()> {
    use windows::Win32::UI::Shell::SetWindowSubclass;

    unsafe {
        if SetWindowSubclass(hwnd, Some(owner_draw_checkbox_subclass_proc), 1, 0).as_bool() {
            Ok(())
        } else {
            Err(crate::error::AppError::Windows(
                "安装复选框悬停处理失败".to_string(),
            ))
        }
    }
}

#[cfg(windows)]
fn is_checkbox_hot(hwnd: windows::Win32::Foundation::HWND) -> bool {
    let hot = hot_checkbox().lock().unwrap();
    *hot == hwnd.0 as isize
}

#[cfg(windows)]
fn set_checkbox_hot(hwnd: windows::Win32::Foundation::HWND, hot: bool) {
    let mut current = hot_checkbox().lock().unwrap();
    if hot {
        *current = hwnd.0 as isize;
    } else if *current == hwnd.0 as isize {
        *current = 0;
    }
}

#[cfg(windows)]
fn hot_checkbox() -> &'static std::sync::Mutex<isize> {
    use std::sync::{Mutex, OnceLock};

    static HOT_CHECKBOX: OnceLock<Mutex<isize>> = OnceLock::new();
    HOT_CHECKBOX.get_or_init(|| Mutex::new(0))
}

#[cfg(windows)]
unsafe extern "system" fn owner_draw_checkbox_subclass_proc(
    hwnd: windows::Win32::Foundation::HWND,
    msg: u32,
    wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
    subclass_id: usize,
    _ref_data: usize,
) -> windows::Win32::Foundation::LRESULT {
    use windows::Win32::Graphics::Gdi::InvalidateRect;
    use windows::Win32::UI::Controls::WM_MOUSELEAVE;
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        TME_LEAVE, TRACKMOUSEEVENT, TrackMouseEvent,
    };
    use windows::Win32::UI::Shell::{DefSubclassProc, RemoveWindowSubclass};
    use windows::Win32::UI::WindowsAndMessaging::{
        WM_ENABLE, WM_KILLFOCUS, WM_MOUSEMOVE, WM_NCDESTROY, WM_SETFOCUS,
    };

    if msg == WM_MOUSEMOVE {
        if !is_checkbox_hot(hwnd) {
            set_checkbox_hot(hwnd, true);
            let mut event = TRACKMOUSEEVENT {
                cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                dwFlags: TME_LEAVE,
                hwndTrack: hwnd,
                dwHoverTime: 0,
            };
            unsafe {
                let _ = TrackMouseEvent(&mut event);
                let _ = InvalidateRect(Some(hwnd), None, true);
            }
        }
    } else if msg == WM_MOUSELEAVE {
        set_checkbox_hot(hwnd, false);
        unsafe {
            let _ = InvalidateRect(Some(hwnd), None, true);
        }
    } else if msg == WM_SETFOCUS || msg == WM_KILLFOCUS || msg == WM_ENABLE {
        unsafe {
            let _ = InvalidateRect(Some(hwnd), None, true);
        }
    } else if msg == WM_NCDESTROY {
        set_checkbox_hot(hwnd, false);
        clear_owner_draw_checkbox_state(hwnd);
        unsafe {
            let _ =
                RemoveWindowSubclass(hwnd, Some(owner_draw_checkbox_subclass_proc), subclass_id);
            return DefSubclassProc(hwnd, msg, wparam, lparam);
        }
    }

    unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) }
}

#[cfg(test)]
mod tests {
    use super::{
        CheckboxVisualState, RgbColor, checkbox_box_rect, checkbox_palette, checkbox_text_left,
        checkbox_toggled_state, checkbox_uses_native_border, is_modern_checkbox,
    };

    #[test]
    fn maps_settings_auto_start_checkbox() {
        assert!(is_modern_checkbox(3117));
        assert!(!checkbox_uses_native_border(3117));
    }

    #[test]
    fn ignores_unknown_controls() {
        assert!(!is_modern_checkbox(9999));
        assert!(checkbox_uses_native_border(9999));
    }

    #[test]
    fn unchecked_checkbox_uses_white_surface() {
        let palette = checkbox_palette(CheckboxVisualState::normal());

        assert_eq!(palette.box_background, RgbColor::new(255, 255, 255));
        assert_eq!(palette.box_border, RgbColor::new(203, 213, 225));
        assert_eq!(palette.text, RgbColor::new(31, 41, 55));
    }

    #[test]
    fn checked_checkbox_uses_primary_fill() {
        let palette = checkbox_palette(CheckboxVisualState {
            checked: true,
            ..CheckboxVisualState::normal()
        });

        assert_eq!(palette.box_background, RgbColor::new(37, 99, 235));
        assert_eq!(palette.check, RgbColor::new(255, 255, 255));
    }

    #[test]
    fn focused_unchecked_checkbox_uses_active_border() {
        let palette = checkbox_palette(CheckboxVisualState {
            focused: true,
            ..CheckboxVisualState::normal()
        });

        assert_eq!(palette.box_border, RgbColor::new(37, 99, 235));
    }

    #[test]
    fn disabled_checkbox_uses_muted_colors() {
        let palette = checkbox_palette(CheckboxVisualState {
            disabled: true,
            ..CheckboxVisualState::normal()
        });

        assert_eq!(palette.box_background, RgbColor::new(243, 244, 246));
        assert_eq!(palette.text, RgbColor::new(156, 163, 175));
    }

    #[test]
    fn checkbox_box_is_vertically_centered() {
        let rect = checkbox_box_rect(28);

        assert_eq!(rect.left, 0);
        assert_eq!(rect.top, 5);
        assert_eq!(rect.right, 18);
        assert_eq!(rect.bottom, 23);
    }

    #[test]
    fn checkbox_text_leaves_gap_after_box() {
        assert_eq!(checkbox_text_left(), 26);
    }

    #[test]
    fn checkbox_click_toggles_state() {
        assert!(checkbox_toggled_state(false));
        assert!(!checkbox_toggled_state(true));
    }

    #[cfg(windows)]
    #[test]
    fn hover_state_round_trips_for_checkbox_hwnd() {
        let hwnd = windows::Win32::Foundation::HWND(3117 as *mut core::ffi::c_void);

        super::set_checkbox_hot(hwnd, false);
        super::set_checkbox_hot(hwnd, true);
        assert!(super::is_checkbox_hot(hwnd));

        super::set_checkbox_hot(hwnd, false);
        assert!(!super::is_checkbox_hot(hwnd));
    }
}
