#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonRole {
    Primary,
    Secondary,
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
pub struct ButtonPalette {
    pub background: RgbColor,
    pub border: RgbColor,
    pub text: RgbColor,
    pub focus: RgbColor,
}

pub fn button_palette(role: ButtonRole, state: ButtonVisualState) -> ButtonPalette {
    let focus = RgbColor::new(37, 99, 235);
    if state.disabled {
        return ButtonPalette {
            background: RgbColor::new(243, 244, 246),
            border: RgbColor::new(209, 213, 219),
            text: RgbColor::new(156, 163, 175),
            focus,
        };
    }

    match role {
        ButtonRole::Primary => {
            let focus = RgbColor::new(219, 234, 254);
            let background = if state.pressed {
                RgbColor::new(29, 78, 216)
            } else if state.hot {
                RgbColor::new(30, 90, 224)
            } else {
                RgbColor::new(37, 99, 235)
            };
            ButtonPalette {
                background,
                border: background,
                text: RgbColor::new(255, 255, 255),
                focus,
            }
        }
        ButtonRole::Secondary => ButtonPalette {
            background: if state.pressed {
                RgbColor::new(226, 232, 240)
            } else if state.hot {
                RgbColor::new(241, 245, 249)
            } else {
                RgbColor::new(255, 255, 255)
            },
            border: if state.hot || state.pressed {
                RgbColor::new(148, 163, 184)
            } else {
                RgbColor::new(203, 213, 225)
            },
            text: RgbColor::new(31, 41, 55),
            focus,
        },
    }
}

pub fn is_owner_draw_button(id: usize) -> bool {
    button_role_for_control(id).is_some()
}

pub fn button_uses_native_border(id: usize) -> bool {
    !is_owner_draw_button(id)
}

pub fn button_draws_inner_focus_ring(_state: ButtonVisualState) -> bool {
    false
}

pub fn button_role_for_control(id: usize) -> Option<ButtonRole> {
    match id {
        2001 | 3004 => Some(ButtonRole::Primary),
        2002 | 3001 | 3002 | 3003 | 3005 | 3116 | 3119 => Some(ButtonRole::Secondary),
        _ => None,
    }
}

#[cfg(windows)]
impl RgbColor {
    fn colorref(self) -> windows::Win32::Foundation::COLORREF {
        windows::Win32::Foundation::COLORREF(
            self.r as u32 | ((self.g as u32) << 8) | ((self.b as u32) << 16),
        )
    }
}

#[cfg(windows)]
pub unsafe fn draw_owner_draw_button(
    draw_item: *const windows::Win32::UI::Controls::DRAWITEMSTRUCT,
) -> bool {
    use windows::Win32::Graphics::Gdi::{
        BACKGROUND_MODE, CreatePen, CreateRoundRectRgn, CreateSolidBrush, DT_CENTER, DT_SINGLELINE,
        DT_VCENTER, DeleteObject, DrawTextW, FillRect, GetBkMode, GetStockObject, GetTextColor,
        HGDIOBJ, NULL_BRUSH, PS_SOLID, RoundRect, SelectClipRgn, SelectObject, SetBkMode,
        SetTextColor, TRANSPARENT,
    };
    use windows::Win32::UI::Controls::{ODS_DISABLED, ODS_FOCUS, ODS_HOTLIGHT, ODS_SELECTED};

    let Some(draw_item) = (unsafe { draw_item.as_ref() }) else {
        return false;
    };
    let Some(role) = button_role_for_control(draw_item.CtlID as usize) else {
        return false;
    };

    let state = ButtonVisualState {
        pressed: (draw_item.itemState.0 & ODS_SELECTED.0) != 0,
        hot: is_button_hot(draw_item.hwndItem) || (draw_item.itemState.0 & ODS_HOTLIGHT.0) != 0,
        disabled: (draw_item.itemState.0 & ODS_DISABLED.0) != 0,
        focused: (draw_item.itemState.0 & ODS_FOCUS.0) != 0,
    };
    let palette = button_palette(role, state);
    let hdc = draw_item.hDC;
    let rect = draw_item.rcItem;

    let background = unsafe { CreateSolidBrush(palette.background.colorref()) };
    let clip_region =
        unsafe { CreateRoundRectRgn(rect.left, rect.top, rect.right + 1, rect.bottom + 1, 7, 7) };
    if !clip_region.is_invalid() {
        unsafe {
            let _ = SelectClipRgn(hdc, Some(clip_region));
        }
    }
    if background.is_invalid() {
        unsafe {
            let _ = FillRect(
                hdc,
                &rect,
                windows::Win32::Graphics::Gdi::HBRUSH(
                    GetStockObject(windows::Win32::Graphics::Gdi::WHITE_BRUSH).0,
                ),
            );
        }
    } else {
        unsafe {
            let _ = FillRect(hdc, &rect, background);
        }
    }
    if !clip_region.is_invalid() {
        unsafe {
            let _ = SelectClipRgn(hdc, None);
            let _ = DeleteObject(clip_region.into());
        }
    }

    let pen = unsafe { CreatePen(PS_SOLID, 1, palette.border.colorref()) };
    let old_pen = if pen.is_invalid() {
        HGDIOBJ::default()
    } else {
        unsafe { SelectObject(hdc, pen.into()) }
    };
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
) -> crate::error::Result<()> {
    use windows::Win32::UI::Shell::SetWindowSubclass;

    unsafe {
        if SetWindowSubclass(hwnd, Some(owner_draw_button_subclass_proc), 1, 0).as_bool() {
            Ok(())
        } else {
            Err(crate::error::AppError::Windows(
                "安装按钮悬停处理失败".to_string(),
            ))
        }
    }
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
    use windows::Win32::UI::WindowsAndMessaging::{WM_MOUSEMOVE, WM_NCDESTROY};

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
    } else if msg == WM_NCDESTROY {
        set_button_hot(hwnd, false);
        unsafe {
            let _ = RemoveWindowSubclass(hwnd, Some(owner_draw_button_subclass_proc), subclass_id);
            return DefSubclassProc(hwnd, msg, wparam, lparam);
        }
    }

    unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) }
}

#[cfg(test)]
mod tests {
    use super::{
        ButtonRole, ButtonVisualState, RgbColor, button_draws_inner_focus_ring, button_palette,
        button_role_for_control, button_uses_native_border, is_owner_draw_button,
    };
    #[cfg(windows)]
    use super::{is_button_hot, set_button_hot};

    #[test]
    fn maps_known_primary_buttons() {
        assert_eq!(button_role_for_control(2001), Some(ButtonRole::Primary));
        assert_eq!(button_role_for_control(3004), Some(ButtonRole::Primary));
    }

    #[test]
    fn maps_known_secondary_buttons() {
        for id in [2002, 3001, 3002, 3003, 3005, 3116, 3119] {
            assert_eq!(button_role_for_control(id), Some(ButtonRole::Secondary));
            assert!(is_owner_draw_button(id));
        }
    }

    #[test]
    fn ignores_unknown_controls() {
        assert_eq!(button_role_for_control(9999), None);
        assert!(!is_owner_draw_button(9999));
    }

    #[test]
    fn primary_palette_uses_quiet_blue() {
        let palette = button_palette(ButtonRole::Primary, ButtonVisualState::normal());
        assert_eq!(palette.background, RgbColor::new(37, 99, 235));
        assert_eq!(palette.text, RgbColor::new(255, 255, 255));
    }

    #[test]
    fn secondary_palette_uses_white_surface() {
        let palette = button_palette(ButtonRole::Secondary, ButtonVisualState::normal());
        assert_eq!(palette.background, RgbColor::new(255, 255, 255));
        assert_eq!(palette.border, RgbColor::new(203, 213, 225));
        assert_eq!(palette.text, RgbColor::new(31, 41, 55));
    }

    #[test]
    fn disabled_palette_removes_primary_emphasis() {
        let state = ButtonVisualState {
            disabled: true,
            ..ButtonVisualState::normal()
        };
        let palette = button_palette(ButtonRole::Primary, state);
        assert_eq!(palette.background, RgbColor::new(243, 244, 246));
        assert_eq!(palette.text, RgbColor::new(156, 163, 175));
    }

    #[test]
    fn owner_draw_buttons_do_not_use_native_border() {
        assert!(!button_uses_native_border(2001));
        assert!(!button_uses_native_border(3004));
        assert!(button_uses_native_border(9999));
    }

    #[test]
    fn primary_focus_ring_uses_contrast_color() {
        let palette = button_palette(ButtonRole::Primary, ButtonVisualState::normal());
        assert_eq!(palette.focus, RgbColor::new(219, 234, 254));
    }

    #[test]
    fn focused_button_does_not_draw_inner_focus_ring() {
        let state = ButtonVisualState {
            focused: true,
            ..ButtonVisualState::normal()
        };

        assert!(!button_draws_inner_focus_ring(state));
    }

    #[cfg(windows)]
    #[test]
    fn hover_state_round_trips_for_button_hwnd() {
        let hwnd = windows::Win32::Foundation::HWND(12345 as *mut core::ffi::c_void);

        set_button_hot(hwnd, false);
        set_button_hot(hwnd, true);
        assert!(is_button_hot(hwnd));

        set_button_hot(hwnd, false);
        assert!(!is_button_hot(hwnd));
    }
}
