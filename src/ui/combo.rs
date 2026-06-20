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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComboRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

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

pub fn modern_combo_frame_rect(left: i32, top: i32, right: i32, _bottom: i32) -> ComboTextRect {
    ComboTextRect {
        left: left - MODERN_COMBO_FRAME_GUTTER,
        top: top - MODERN_COMBO_FRAME_GUTTER,
        right: right + MODERN_COMBO_FRAME_GUTTER,
        bottom: top - MODERN_COMBO_FRAME_GUTTER + MODERN_COMBO_VISIBLE_HEIGHT,
    }
}

pub fn modern_combo_child_rect(id: usize, x: i32, y: i32, width: i32, height: i32) -> ComboRect {
    if is_modern_combo(id) {
        return ComboRect {
            x: x + MODERN_COMBO_FRAME_GUTTER,
            y: y + MODERN_COMBO_FRAME_GUTTER,
            width: (width - MODERN_COMBO_FRAME_GUTTER * 2).max(1),
            height,
        };
    }

    ComboRect {
        x,
        y,
        width,
        height,
    }
}

pub const MODERN_COMBO_FRAME_GUTTER: i32 = 2;
pub const MODERN_COMBO_VISIBLE_HEIGHT: i32 = 26;

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

#[cfg(test)]
mod tests {
    use super::{
        ComboVisualState, RgbColor, combo_palette, combo_uses_native_border, is_modern_combo,
        modern_combo_child_rect, modern_combo_frame_rect,
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
        let rect = modern_combo_frame_rect(410, 14, 586, 234);
        assert_eq!(rect.left, 408);
        assert_eq!(rect.top, 12);
        assert_eq!(rect.right, 588);
        assert_eq!(rect.bottom, 38);
    }

    #[test]
    fn modern_combo_child_rect_leaves_room_for_parent_drawn_visible_frame() {
        let rect = modern_combo_child_rect(2106, 408, 12, 180, 220);

        assert!(rect.x > 408);
        assert!(rect.y > 12);
        assert!(rect.width < 180);
        assert_eq!(rect.height, 220);
    }

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
}
