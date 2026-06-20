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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditTextRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditBorderPaintTarget {
    ParentFrame,
}

pub fn edit_border_paint_target() -> EditBorderPaintTarget {
    EditBorderPaintTarget::ParentFrame
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

pub fn modern_edit_color_query_invalidates_border() -> bool {
    false
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

pub fn modern_edit_frame_rect(left: i32, top: i32, right: i32, bottom: i32) -> EditTextRect {
    EditTextRect {
        left: left - EDIT_FRAME_GUTTER,
        top: top - EDIT_FRAME_GUTTER,
        right: right + EDIT_FRAME_GUTTER,
        bottom: bottom + EDIT_FRAME_GUTTER,
    }
}

pub fn multiline_edit_text_rect(width: i32, height: i32) -> EditTextRect {
    EditTextRect {
        left: MULTILINE_EDIT_HORIZONTAL_PADDING,
        top: MULTILINE_EDIT_VERTICAL_PADDING,
        right: (width - MULTILINE_EDIT_HORIZONTAL_PADDING).max(MULTILINE_EDIT_HORIZONTAL_PADDING),
        bottom: (height - MULTILINE_EDIT_VERTICAL_PADDING).max(MULTILINE_EDIT_VERTICAL_PADDING),
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
pub fn install_modern_edit_focus_tracking(
    hwnd: windows::Win32::Foundation::HWND,
) -> crate::error::Result<()> {
    use windows::Win32::UI::Shell::SetWindowSubclass;

    unsafe {
        if SetWindowSubclass(
            hwnd,
            Some(modern_edit_subclass_proc),
            MODERN_EDIT_SUBCLASS_ID,
            0,
        )
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
pub unsafe fn paint_modern_edit_border(parent: windows::Win32::Foundation::HWND, control_id: i32) {
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
    let frame = modern_edit_frame_rect(points[0].x, points[0].y, points[1].x, points[1].y);
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
pub unsafe fn handle_modern_edit_color(
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
        readonly: (style & ES_READONLY as u32) != 0,
        disabled: !unsafe { IsWindowEnabled(hwnd).as_bool() },
    }
}

#[cfg(windows)]
pub unsafe fn invalidate_modern_edit_for_child(
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
        let frame = modern_edit_frame_rect(points[0].x, points[0].y, points[1].x, points[1].y);
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
const MODERN_EDIT_SUBCLASS_ID: usize = 2;

#[cfg(windows)]
unsafe extern "system" fn modern_edit_subclass_proc(
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

#[cfg(test)]
mod tests {
    use super::{
        EditBorderPaintTarget, EditKind, EditVisualState, RgbColor, edit_border_paint_target,
        edit_kind_for_control, edit_palette, edit_uses_native_border, is_modern_edit,
        modern_edit_child_rect, modern_edit_color_query_invalidates_border,
        multiline_edit_text_rect,
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

    #[test]
    fn edit_border_is_painted_on_control_window() {
        assert_eq!(
            edit_border_paint_target(),
            EditBorderPaintTarget::ParentFrame
        );
    }

    #[test]
    fn color_query_does_not_schedule_border_repaint() {
        assert!(!modern_edit_color_query_invalidates_border());
    }

    #[test]
    fn modern_multiline_edit_leaves_space_for_parent_drawn_rounded_border() {
        let rect = modern_edit_child_rect(16, 38, 572, 96);

        assert!(rect.x > 16);
        assert!(rect.y > 38);
        assert!(rect.x - 16 >= 4);
        assert!(rect.y - 38 >= 4);
        assert_eq!(rect.width, 572 - (rect.x - 16) * 2);
        assert_eq!(rect.height, 96 - (rect.y - 38) * 2);
    }

    #[test]
    fn multiline_edit_text_rect_adds_readable_inner_padding() {
        let rect = multiline_edit_text_rect(564, 88);

        assert_eq!(rect.left, 8);
        assert_eq!(rect.top, 6);
        assert_eq!(rect.right, 564 - 8);
        assert_eq!(rect.bottom, 88 - 6);
    }
}
