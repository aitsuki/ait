pub use crate::ui::theme::RgbColor;
use crate::ui::theme::{CONTROL_RADIUS, FOCUS_RING_INSET, colors};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonRole {
    Primary,
    Secondary,
    Danger,
    Ghost,
    SelectedTab,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ButtonVisualState {
    pub pressed: bool,
    pub hot: bool,
    pub disabled: bool,
    pub focused: bool,
}

impl ButtonVisualState {
    pub fn normal() -> Self {
        Self {
            pressed: false,
            hot: false,
            disabled: false,
            focused: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ButtonPalette {
    pub background: RgbColor,
    pub border: RgbColor,
    pub text: RgbColor,
    pub focus: RgbColor,
}

pub fn button_palette(role: ButtonRole, state: ButtonVisualState) -> ButtonPalette {
    let c = colors();
    if state.disabled {
        return ButtonPalette {
            background: c.disabled_surface,
            border: c.disabled_border,
            text: c.disabled_text,
            focus: c.primary,
        };
    }

    let surface = if state.pressed {
        c.surface_pressed
    } else if state.hot {
        c.surface_hover
    } else {
        c.surface
    };
    match role {
        ButtonRole::Primary => {
            let background = if state.pressed {
                c.primary_pressed
            } else if state.hot {
                c.primary_hover
            } else {
                c.primary
            };
            ButtonPalette {
                background,
                border: background,
                text: c.primary_text,
                focus: c.focus_soft,
            }
        }
        ButtonRole::Secondary => ButtonPalette {
            background: surface,
            border: if state.hot || state.pressed {
                c.border_strong
            } else {
                c.border
            },
            text: c.text,
            focus: c.primary,
        },
        ButtonRole::Danger => ButtonPalette {
            background: if state.pressed || state.hot {
                c.danger_soft
            } else {
                c.surface
            },
            border: if state.hot || state.pressed {
                c.danger_hover
            } else {
                c.border
            },
            text: c.danger,
            focus: c.danger,
        },
        ButtonRole::SelectedTab => ButtonPalette {
            background: c.focus_soft,
            border: c.focus_soft,
            text: c.focus_text,
            focus: c.primary,
        },
        ButtonRole::Ghost => ButtonPalette {
            background: surface,
            border: if state.focused { c.primary } else { c.surface },
            text: c.text,
            focus: c.primary,
        },
    }
}

pub fn is_owner_draw_button(id: usize) -> bool {
    button_role_for_control(id).is_some()
}

pub fn button_uses_native_border(id: usize) -> bool {
    !is_owner_draw_button(id)
}

pub fn button_draws_inner_focus_ring(state: ButtonVisualState) -> bool {
    state.focused && !state.disabled
}

pub fn button_role_for_control(id: usize) -> Option<ButtonRole> {
    match id {
        2001 | 3004 => Some(ButtonRole::Primary),
        2002 | 3001 | 3002 | 3003 | 3005 | 3116 | 3119 | 3121 | 3124 => Some(ButtonRole::Secondary),
        3126 | 3135 => Some(ButtonRole::Ghost),
        _ => None,
    }
}

#[cfg(windows)]
pub(crate) unsafe fn draw_owner_draw_button(
    draw_item: *const windows::Win32::UI::Controls::DRAWITEMSTRUCT,
) -> bool {
    use windows::Win32::Graphics::Gdi::{
        BACKGROUND_MODE, CreatePen, CreateSolidBrush, DT_CENTER, DT_SINGLELINE, DT_VCENTER,
        DeleteObject, DrawTextW, FillRect, GetBkMode, GetStockObject, GetTextColor, HGDIOBJ,
        NULL_BRUSH, PS_SOLID, RoundRect, SelectObject, SetBkMode, SetTextColor, TRANSPARENT,
    };
    use windows::Win32::UI::Controls::{ODS_DISABLED, ODS_FOCUS, ODS_HOTLIGHT, ODS_SELECTED};

    let Some(draw_item) = (unsafe { draw_item.as_ref() }) else {
        return false;
    };
    let Some(role) = button_role_for_window(draw_item.hwndItem)
        .or_else(|| button_role_for_control(draw_item.CtlID as usize))
    else {
        return false;
    };

    let state = ButtonVisualState {
        pressed: (draw_item.itemState.0 & ODS_SELECTED.0) != 0,
        hot: is_button_hot(draw_item.hwndItem) || (draw_item.itemState.0 & ODS_HOTLIGHT.0) != 0,
        disabled: (draw_item.itemState.0 & ODS_DISABLED.0) != 0,
        focused: (draw_item.itemState.0 & ODS_FOCUS.0) != 0,
    };
    let palette = button_palette(role, state);
    let radius = crate::ui::theme::scale(CONTROL_RADIUS);
    let focus_inset = crate::ui::theme::scale(FOCUS_RING_INSET);
    let hdc = draw_item.hDC;
    let rect = draw_item.rcItem;

    // Owner-drawn buttons can retain native pixels outside the rounded shape.
    // Clear all four corners on every repaint, including after a theme change.
    unsafe {
        let _ = FillRect(hdc, &rect, crate::ui::theme::surface_brush());
    }
    let background = unsafe { CreateSolidBrush(palette.background.colorref()) };

    let pen = unsafe { CreatePen(PS_SOLID, 1, palette.border.colorref()) };
    let old_pen = if pen.is_invalid() {
        HGDIOBJ::default()
    } else {
        unsafe { SelectObject(hdc, pen.into()) }
    };
    let fill_brush = if background.is_invalid() {
        crate::ui::theme::surface_brush()
    } else {
        background
    };
    let old_brush = unsafe { SelectObject(hdc, fill_brush.into()) };
    unsafe {
        let _ = RoundRect(
            hdc,
            rect.left,
            rect.top,
            rect.right,
            rect.bottom,
            radius,
            radius,
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

    if button_draws_inner_focus_ring(state) {
        let focus_pen = unsafe { CreatePen(PS_SOLID, 2, palette.focus.colorref()) };
        if !focus_pen.is_invalid() {
            let old_focus_pen = unsafe { SelectObject(hdc, focus_pen.into()) };
            let old_focus_brush = unsafe { SelectObject(hdc, GetStockObject(NULL_BRUSH)) };
            unsafe {
                let _ = RoundRect(
                    hdc,
                    rect.left + focus_inset,
                    rect.top + focus_inset,
                    rect.right - focus_inset,
                    rect.bottom - focus_inset,
                    (radius - 1).max(1),
                    (radius - 1).max(1),
                );
            }
            if !old_focus_brush.is_invalid() {
                unsafe {
                    let _ = SelectObject(hdc, old_focus_brush);
                }
            }
            if !old_focus_pen.is_invalid() {
                unsafe {
                    let _ = SelectObject(hdc, old_focus_pen);
                }
            }
            unsafe {
                let _ = DeleteObject(focus_pen.into());
            }
        }
    }

    let mut text = button_text(draw_item.hwndItem);
    let mut text_rect = rect;
    let old_bk_mode = unsafe { GetBkMode(hdc) };
    let old_text_color = unsafe { GetTextColor(hdc) };
    unsafe {
        let _ = SetBkMode(hdc, TRANSPARENT);
        let _ = SetTextColor(hdc, palette.text.colorref());
        let _ = DrawTextW(
            hdc,
            &mut text,
            &mut text_rect,
            DT_CENTER | DT_VCENTER | DT_SINGLELINE,
        );
        let _ = SetTextColor(hdc, old_text_color);
        let _ = SetBkMode(hdc, BACKGROUND_MODE(old_bk_mode as u32));
    }

    if !pen.is_invalid() {
        unsafe {
            let _ = DeleteObject(pen.into());
        }
    }
    if !background.is_invalid() {
        unsafe {
            let _ = DeleteObject(background.into());
        }
    }
    true
}

#[cfg(windows)]
fn button_text(hwnd: windows::Win32::Foundation::HWND) -> Vec<u16> {
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
pub fn install_owner_draw_button_hover(
    hwnd: windows::Win32::Foundation::HWND,
    role: ButtonRole,
) -> crate::error::Result<()> {
    use windows::Win32::UI::Shell::SetWindowSubclass;

    button_roles().lock().unwrap().insert(hwnd.0 as isize, role);
    unsafe {
        if SetWindowSubclass(hwnd, Some(owner_draw_button_subclass_proc), 1, 0).as_bool() {
            Ok(())
        } else {
            button_roles().lock().unwrap().remove(&(hwnd.0 as isize));
            Err(crate::error::AppError::Windows(
                "安装按钮悬停处理失败".to_string(),
            ))
        }
    }
}

#[cfg(windows)]
pub fn set_button_role(hwnd: windows::Win32::Foundation::HWND, role: ButtonRole) {
    button_roles().lock().unwrap().insert(hwnd.0 as isize, role);
    unsafe {
        let _ = windows::Win32::Graphics::Gdi::InvalidateRect(Some(hwnd), None, true);
    }
}

#[cfg(windows)]
fn button_roles() -> &'static std::sync::Mutex<std::collections::HashMap<isize, ButtonRole>> {
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};
    static ROLES: OnceLock<Mutex<HashMap<isize, ButtonRole>>> = OnceLock::new();
    ROLES.get_or_init(|| Mutex::new(HashMap::new()))
}

#[cfg(windows)]
fn button_role_for_window(hwnd: windows::Win32::Foundation::HWND) -> Option<ButtonRole> {
    button_roles()
        .lock()
        .unwrap()
        .get(&(hwnd.0 as isize))
        .copied()
}

#[cfg(windows)]
fn is_button_hot(hwnd: windows::Win32::Foundation::HWND) -> bool {
    let hot = hot_button().lock().unwrap();
    *hot == hwnd.0 as isize
}

#[cfg(windows)]
fn set_button_hot(hwnd: windows::Win32::Foundation::HWND, hot: bool) {
    let mut current = hot_button().lock().unwrap();
    if hot {
        *current = hwnd.0 as isize;
    } else if *current == hwnd.0 as isize {
        *current = 0;
    }
}

#[cfg(windows)]
fn hot_button() -> &'static std::sync::Mutex<isize> {
    use std::sync::{Mutex, OnceLock};

    static HOT_BUTTON: OnceLock<Mutex<isize>> = OnceLock::new();
    HOT_BUTTON.get_or_init(|| Mutex::new(0))
}

#[cfg(windows)]
unsafe extern "system" fn owner_draw_button_subclass_proc(
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
        if !is_button_hot(hwnd) {
            set_button_hot(hwnd, true);
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
        set_button_hot(hwnd, false);
        unsafe {
            let _ = InvalidateRect(Some(hwnd), None, true);
        }
    } else if msg == WM_SETFOCUS || msg == WM_KILLFOCUS || msg == WM_ENABLE {
        unsafe {
            let _ = InvalidateRect(Some(hwnd), None, true);
        }
    } else if msg == WM_NCDESTROY {
        set_button_hot(hwnd, false);
        button_roles().lock().unwrap().remove(&(hwnd.0 as isize));
        unsafe {
            let _ = RemoveWindowSubclass(hwnd, Some(owner_draw_button_subclass_proc), subclass_id);
            return DefSubclassProc(hwnd, msg, wparam, lparam);
        }
    }

    unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) }
}
