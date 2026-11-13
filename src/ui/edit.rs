pub use crate::ui::theme::RgbColor;
use crate::ui::theme::{CONTROL_RADIUS, colors};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditKind {
    SingleLine,
    MultiLine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditVisualState {
    pub focused: bool,
    pub hot: bool,
    pub readonly: bool,
    pub disabled: bool,
}

impl EditVisualState {
    pub fn normal() -> Self {
        Self {
            focused: false,
            hot: false,
            readonly: false,
            disabled: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditPalette {
    pub background: RgbColor,
    pub border: RgbColor,
    pub text: RgbColor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

pub fn edit_palette(state: EditVisualState) -> EditPalette {
    let c = colors();
    if state.disabled {
        return EditPalette {
            background: c.disabled_surface,
            border: c.disabled_border,
            text: c.disabled_text,
        };
    }

    EditPalette {
        background: if state.readonly {
            c.surface_subtle
        } else {
            c.surface
        },
        border: if state.focused {
            c.primary
        } else if state.hot {
            c.border_strong
        } else {
            c.border
        },
        text: c.text,
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

pub fn single_line_third_click_selects_all(
    last_double_click_time: Option<u32>,
    current_time: u32,
    double_click_time: u32,
) -> bool {
    last_double_click_time
        .map(|last| current_time.wrapping_sub(last) <= double_click_time)
        .unwrap_or(false)
}

pub const EDIT_FRAME_GUTTER: i32 = 4;
pub const MULTILINE_EDIT_HORIZONTAL_PADDING: i32 = 8;
pub const MULTILINE_EDIT_VERTICAL_PADDING: i32 = 6;

pub fn modern_edit_child_rect(x: i32, y: i32, width: i32, height: i32) -> EditRect {
    EditRect {
        x: x + EDIT_FRAME_GUTTER,
        y: y + EDIT_FRAME_GUTTER,
        width: (width - EDIT_FRAME_GUTTER * 2).max(1),
        height: (height - EDIT_FRAME_GUTTER * 2).max(1),
    }
}

#[cfg(windows)]
pub fn install_modern_edit_focus_tracking(
    hwnd: windows::Win32::Foundation::HWND,
) -> crate::error::Result<()> {
    use windows::Win32::UI::Shell::SetWindowSubclass;

    let state = Box::into_raw(Box::new(ModernEditSubclassState::default())) as usize;
    unsafe {
        if SetWindowSubclass(
            hwnd,
            Some(modern_edit_subclass_proc),
            MODERN_EDIT_SUBCLASS_ID,
            state,
        )
        .as_bool()
        {
            Ok(())
        } else {
            drop(Box::from_raw(state as *mut ModernEditSubclassState));
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
    crate::ui::theme::background_brush(state.readonly, state.disabled)
}

#[cfg(windows)]
pub(crate) unsafe fn paint_modern_edit_border(
    parent: windows::Win32::Foundation::HWND,
    control_id: i32,
) {
    use windows::Win32::Graphics::Gdi::MapWindowPoints;
    use windows::Win32::Graphics::Gdi::{
        CreatePen, DeleteObject, GetDC, GetStockObject, NULL_BRUSH, PS_SOLID, ReleaseDC, RoundRect,
        SelectObject,
    };
    use windows::Win32::UI::WindowsAndMessaging::{GetDlgItem, GetWindowRect};

    let Ok(child) = (unsafe { GetDlgItem(Some(parent), control_id) }) else {
        return;
    };
    let state = unsafe { edit_visual_state_for_child(child) };
    let palette = edit_palette(state);
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
    let gutter = crate::ui::theme::scale(EDIT_FRAME_GUTTER);
    rect.left = points[0].x - gutter;
    rect.top = points[0].y - gutter;
    rect.right = points[1].x + gutter;
    rect.bottom = points[1].y + gutter;

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
        let _ = RoundRect(
            hdc,
            rect.left,
            rect.top,
            rect.right,
            rect.bottom,
            crate::ui::theme::scale(CONTROL_RADIUS),
            crate::ui::theme::scale(CONTROL_RADIUS),
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
        let _ = DeleteObject(pen.into());
        let _ = ReleaseDC(Some(parent), hdc);
    }
}

#[cfg(windows)]
pub(crate) unsafe fn handle_modern_edit_color(
    _parent: windows::Win32::Foundation::HWND,
    wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
    readonly: bool,
) -> Option<windows::Win32::Foundation::LRESULT> {
    use windows::Win32::Graphics::Gdi::{SetBkColor, SetTextColor};
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetFocus, IsWindowEnabled};
    use windows::Win32::UI::WindowsAndMessaging::GetDlgCtrlID;

    let child = windows::Win32::Foundation::HWND(lparam.0 as *mut core::ffi::c_void);
    let id = unsafe { GetDlgCtrlID(child) };
    if !is_modern_edit(id as usize) {
        return None;
    }

    let state = EditVisualState {
        focused: unsafe { GetFocus() } == child,
        hot: is_edit_hot(child),
        readonly,
        disabled: !unsafe { IsWindowEnabled(child).as_bool() },
    };
    let palette = edit_palette(state);
    let hdc = windows::Win32::Graphics::Gdi::HDC(wparam.0 as *mut core::ffi::c_void);
    unsafe {
        let _ = SetTextColor(hdc, palette.text.colorref());
        let _ = SetBkColor(hdc, palette.background.colorref());
    }
    Some(windows::Win32::Foundation::LRESULT(
        modern_edit_brush_for_state(state).0 as isize,
    ))
}

#[cfg(windows)]
unsafe fn edit_visual_state_for_child(hwnd: windows::Win32::Foundation::HWND) -> EditVisualState {
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetFocus, IsWindowEnabled};
    use windows::Win32::UI::WindowsAndMessaging::{ES_READONLY, GWL_STYLE, GetWindowLongW};

    let style = unsafe { GetWindowLongW(hwnd, GWL_STYLE) } as u32;
    EditVisualState {
        focused: unsafe { GetFocus() } == hwnd,
        hot: is_edit_hot(hwnd),
        readonly: (style & ES_READONLY as u32) != 0,
        disabled: !unsafe { IsWindowEnabled(hwnd).as_bool() },
    }
}

#[cfg(windows)]
pub(crate) unsafe fn invalidate_modern_edit_for_child(
    parent: windows::Win32::Foundation::HWND,
    child: windows::Win32::Foundation::HWND,
) {
    unsafe {
        invalidate_edit_border(parent, child);
    }
}

#[cfg(windows)]
unsafe fn invalidate_edit_border(
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
        let gutter = crate::ui::theme::scale(EDIT_FRAME_GUTTER);
        rect.left = points[0].x - gutter;
        rect.top = points[0].y - gutter;
        rect.right = points[1].x + gutter;
        rect.bottom = points[1].y + gutter;
        unsafe {
            let _ = InvalidateRect(Some(parent), Some(&rect), false);
        }
    }
}

#[cfg(windows)]
fn hot_edit() -> &'static std::sync::Mutex<isize> {
    use std::sync::{Mutex, OnceLock};
    static HOT_EDIT: OnceLock<Mutex<isize>> = OnceLock::new();
    HOT_EDIT.get_or_init(|| Mutex::new(0))
}

#[cfg(windows)]
fn is_edit_hot(hwnd: windows::Win32::Foundation::HWND) -> bool {
    *hot_edit().lock().unwrap() == hwnd.0 as isize
}

#[cfg(windows)]
fn set_edit_hot(hwnd: windows::Win32::Foundation::HWND, hot: bool) {
    let mut current = hot_edit().lock().unwrap();
    if hot {
        *current = hwnd.0 as isize;
    } else if *current == hwnd.0 as isize {
        *current = 0;
    }
}

#[cfg(windows)]
#[derive(Default)]
struct ModernEditSubclassState {
    last_double_click_time: Option<u32>,
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
    use windows::Win32::Foundation::{LPARAM, WPARAM};
    use windows::Win32::UI::Controls::{EM_SETSEL, WM_MOUSELEAVE};
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        GetDoubleClickTime, TME_LEAVE, TRACKMOUSEEVENT, TrackMouseEvent,
    };
    use windows::Win32::UI::Shell::{DefSubclassProc, RemoveWindowSubclass};
    use windows::Win32::UI::WindowsAndMessaging::{
        GetDlgCtrlID, GetMessageTime, GetParent, SendMessageW, WM_ENABLE, WM_KILLFOCUS,
        WM_LBUTTONDBLCLK, WM_LBUTTONDOWN, WM_MOUSEMOVE, WM_NCDESTROY, WM_SETFOCUS,
    };

    let state_ptr = ref_data as *mut ModernEditSubclassState;

    if msg == WM_MOUSEMOVE && !is_edit_hot(hwnd) {
        set_edit_hot(hwnd, true);
        let mut event = TRACKMOUSEEVENT {
            cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
            dwFlags: TME_LEAVE,
            hwndTrack: hwnd,
            dwHoverTime: 0,
        };
        unsafe {
            let _ = TrackMouseEvent(&mut event);
        }
        if let Ok(parent) = unsafe { GetParent(hwnd) } {
            unsafe {
                invalidate_edit_border(parent, hwnd);
            }
        }
    } else if msg == WM_MOUSELEAVE {
        set_edit_hot(hwnd, false);
        if let Ok(parent) = unsafe { GetParent(hwnd) } {
            unsafe {
                invalidate_edit_border(parent, hwnd);
            }
        }
    }

    if msg == WM_LBUTTONDBLCLK && !state_ptr.is_null() {
        let state = unsafe { &mut *state_ptr };
        state.last_double_click_time = Some(unsafe { GetMessageTime() } as u32);
    } else if msg == WM_LBUTTONDOWN
        && !state_ptr.is_null()
        && edit_kind_for_control(unsafe { GetDlgCtrlID(hwnd) } as usize)
            == Some(EditKind::SingleLine)
    {
        let state = unsafe { &mut *state_ptr };
        if single_line_third_click_selects_all(
            state.last_double_click_time,
            unsafe { GetMessageTime() } as u32,
            unsafe { GetDoubleClickTime() },
        ) {
            state.last_double_click_time = None;
            unsafe {
                let _ = SendMessageW(hwnd, EM_SETSEL, Some(WPARAM(0)), Some(LPARAM(-1)));
            }
            return windows::Win32::Foundation::LRESULT(0);
        }
        state.last_double_click_time = None;
    }

    if (msg == WM_SETFOCUS || msg == WM_KILLFOCUS || msg == WM_ENABLE)
        && let Ok(parent) = unsafe { GetParent(hwnd) }
    {
        unsafe {
            invalidate_edit_border(parent, hwnd);
        }
    }

    if msg == WM_NCDESTROY {
        set_edit_hot(hwnd, false);
        if !state_ptr.is_null() {
            unsafe {
                drop(Box::from_raw(state_ptr));
            }
        }
        unsafe {
            let _ = RemoveWindowSubclass(hwnd, Some(modern_edit_subclass_proc), subclass_id);
            return DefSubclassProc(hwnd, msg, wparam, lparam);
        }
    }

    unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) }
}
