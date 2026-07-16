pub use crate::ui::theme::RgbColor;
use crate::ui::theme::{
    COLOR_BORDER, COLOR_DISABLED_BORDER, COLOR_DISABLED_SURFACE, COLOR_DISABLED_TEXT,
    COLOR_FOCUS_SOFT, COLOR_FOCUS_TEXT, COLOR_PRIMARY, COLOR_SURFACE, COLOR_SURFACE_HOVER,
    COLOR_TEXT, CONTROL_RADIUS, LIST_ITEM_HEIGHT,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ListBoxVisualState {
    pub focused: bool,
    pub disabled: bool,
}

impl ListBoxVisualState {
    pub fn normal() -> Self {
        Self {
            focused: false,
            disabled: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ListBoxItemVisualState {
    pub selected: bool,
    pub hot: bool,
    pub disabled: bool,
}

impl ListBoxItemVisualState {
    pub fn normal() -> Self {
        Self {
            selected: false,
            hot: false,
            disabled: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ListBoxPalette {
    pub background: RgbColor,
    pub border: RgbColor,
    pub text: RgbColor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ListBoxItemPalette {
    pub background: RgbColor,
    pub text: RgbColor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ListBoxTextRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

pub const MODERN_LISTBOX_ITEM_HEIGHT: u32 = LIST_ITEM_HEIGHT;
pub const MODERN_LISTBOX_TEXT_PADDING: i32 = 10;
pub const MODERN_LISTBOX_RADIUS: i32 = CONTROL_RADIUS;

pub fn listbox_palette(state: ListBoxVisualState) -> ListBoxPalette {
    if state.disabled {
        return ListBoxPalette {
            background: COLOR_DISABLED_SURFACE,
            border: COLOR_DISABLED_BORDER,
            text: COLOR_DISABLED_TEXT,
        };
    }

    ListBoxPalette {
        background: COLOR_SURFACE,
        border: if state.focused {
            COLOR_PRIMARY
        } else {
            COLOR_BORDER
        },
        text: COLOR_TEXT,
    }
}

pub fn listbox_item_palette(state: ListBoxItemVisualState) -> ListBoxItemPalette {
    if state.disabled {
        return ListBoxItemPalette {
            background: COLOR_DISABLED_SURFACE,
            text: COLOR_DISABLED_TEXT,
        };
    }

    if state.selected {
        ListBoxItemPalette {
            background: COLOR_FOCUS_SOFT,
            text: COLOR_FOCUS_TEXT,
        }
    } else if state.hot {
        ListBoxItemPalette {
            background: COLOR_SURFACE_HOVER,
            text: COLOR_TEXT,
        }
    } else {
        ListBoxItemPalette {
            background: COLOR_SURFACE,
            text: COLOR_TEXT,
        }
    }
}

pub fn is_modern_listbox(id: usize) -> bool {
    id == 3101
}

pub fn listbox_uses_native_border(id: usize) -> bool {
    !is_modern_listbox(id)
}

pub fn modern_listbox_text_rect(rect: ListBoxTextRect) -> ListBoxTextRect {
    ListBoxTextRect {
        left: rect.left + MODERN_LISTBOX_TEXT_PADDING,
        top: rect.top,
        right: (rect.right - MODERN_LISTBOX_TEXT_PADDING)
            .max(rect.left + MODERN_LISTBOX_TEXT_PADDING),
        bottom: rect.bottom,
    }
}

#[cfg(windows)]
pub fn install_modern_listbox_tracking(
    hwnd: windows::Win32::Foundation::HWND,
) -> crate::error::Result<()> {
    use windows::Win32::UI::Shell::SetWindowSubclass;

    unsafe {
        if SetWindowSubclass(
            hwnd,
            Some(modern_listbox_subclass_proc),
            MODERN_LISTBOX_SUBCLASS_ID,
            0,
        )
        .as_bool()
        {
            Ok(())
        } else {
            Err(crate::error::AppError::Windows(
                "安装列表框绘制处理失败".to_string(),
            ))
        }
    }
}

#[cfg(windows)]
pub unsafe fn measure_owner_draw_listbox_item(
    measure_item: *mut windows::Win32::UI::Controls::MEASUREITEMSTRUCT,
) -> bool {
    use windows::Win32::UI::Controls::ODT_LISTBOX;

    let Some(measure_item) = (unsafe { measure_item.as_mut() }) else {
        return false;
    };
    if !is_modern_listbox(measure_item.CtlID as usize) || measure_item.CtlType != ODT_LISTBOX {
        return false;
    }

    measure_item.itemHeight = crate::ui::theme::scale(MODERN_LISTBOX_ITEM_HEIGHT as i32) as u32;
    true
}

#[cfg(windows)]
pub unsafe fn draw_owner_draw_listbox_item(
    draw_item: *const windows::Win32::UI::Controls::DRAWITEMSTRUCT,
) -> bool {
    use windows::Win32::Foundation::{LPARAM, RECT, WPARAM};
    use windows::Win32::Graphics::Gdi::{
        BACKGROUND_MODE, CreateSolidBrush, DT_END_ELLIPSIS, DT_SINGLELINE, DT_VCENTER,
        DeleteObject, DrawTextW, FillRect, GetBkMode, GetTextColor, SetBkMode, SetTextColor,
        TRANSPARENT,
    };
    use windows::Win32::UI::Controls::{ODS_DISABLED, ODS_SELECTED, ODT_LISTBOX};
    use windows::Win32::UI::WindowsAndMessaging::{LB_GETTEXT, LB_GETTEXTLEN, SendMessageW};

    let Some(draw_item) = (unsafe { draw_item.as_ref() }) else {
        return false;
    };
    if !is_modern_listbox(draw_item.CtlID as usize) || draw_item.CtlType != ODT_LISTBOX {
        return false;
    }
    if draw_item.itemID == u32::MAX {
        return true;
    }

    let state = ListBoxItemVisualState {
        selected: (draw_item.itemState.0 & ODS_SELECTED.0) != 0,
        hot: hot_listbox_item(draw_item.hwndItem) == Some(draw_item.itemID as usize),
        disabled: (draw_item.itemState.0 & ODS_DISABLED.0) != 0,
    };
    let palette = listbox_item_palette(state);
    let hdc = draw_item.hDC;
    let rect = draw_item.rcItem;

    let background = unsafe { CreateSolidBrush(palette.background.colorref()) };
    if !background.is_invalid() {
        unsafe {
            let _ = FillRect(hdc, &rect, background);
        }
    }

    let len = unsafe {
        SendMessageW(
            draw_item.hwndItem,
            LB_GETTEXTLEN,
            Some(WPARAM(draw_item.itemID as usize)),
            Some(LPARAM(0)),
        )
        .0
    };
    if len >= 0 {
        let mut text = vec![0u16; len as usize + 1];
        unsafe {
            let _ = SendMessageW(
                draw_item.hwndItem,
                LB_GETTEXT,
                Some(WPARAM(draw_item.itemID as usize)),
                Some(LPARAM(text.as_mut_ptr() as isize)),
            );
        }
        text.truncate(len as usize);

        let padding = crate::ui::theme::scale(MODERN_LISTBOX_TEXT_PADDING);
        let text_frame = ListBoxTextRect {
            left: rect.left + padding,
            top: rect.top,
            right: (rect.right - padding).max(rect.left + padding),
            bottom: rect.bottom,
        };
        let mut text_rect = RECT {
            left: text_frame.left,
            top: text_frame.top,
            right: text_frame.right,
            bottom: text_frame.bottom,
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
                DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS,
            );
            let _ = SetTextColor(hdc, old_text_color);
            let _ = SetBkMode(hdc, BACKGROUND_MODE(old_bk_mode as u32));
        }
    }

    if !background.is_invalid() {
        unsafe {
            let _ = DeleteObject(background.into());
        }
    }
    unsafe {
        paint_modern_listbox_border(draw_item.hwndItem);
    }
    true
}

#[cfg(windows)]
unsafe fn paint_modern_listbox_border(hwnd: windows::Win32::Foundation::HWND) {
    use windows::Win32::Graphics::Gdi::{
        CreatePen, DeleteObject, GetDC, GetStockObject, NULL_BRUSH, PS_SOLID, ReleaseDC, RoundRect,
        SelectObject,
    };
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetFocus, IsWindowEnabled};
    use windows::Win32::UI::WindowsAndMessaging::GetClientRect;

    let state = ListBoxVisualState {
        focused: unsafe { GetFocus() } == hwnd,
        disabled: !unsafe { IsWindowEnabled(hwnd).as_bool() },
    };
    let palette = listbox_palette(state);
    let mut rect = windows::Win32::Foundation::RECT::default();
    if unsafe { GetClientRect(hwnd, &mut rect).is_err() } {
        return;
    }
    if rect.right <= rect.left || rect.bottom <= rect.top {
        return;
    }

    let hdc = unsafe { GetDC(Some(hwnd)) };
    if hdc.is_invalid() {
        return;
    }

    let pen = unsafe { CreatePen(PS_SOLID, 1, palette.border.colorref()) };
    if pen.is_invalid() {
        unsafe {
            let _ = ReleaseDC(Some(hwnd), hdc);
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
            MODERN_LISTBOX_RADIUS,
            MODERN_LISTBOX_RADIUS,
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
        let _ = ReleaseDC(Some(hwnd), hdc);
    }
}

#[cfg(windows)]
fn hot_listbox_items() -> &'static std::sync::Mutex<std::collections::HashMap<isize, Option<usize>>>
{
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};
    static HOT_ITEMS: OnceLock<Mutex<HashMap<isize, Option<usize>>>> = OnceLock::new();
    HOT_ITEMS.get_or_init(|| Mutex::new(HashMap::new()))
}

#[cfg(windows)]
fn hot_listbox_item(hwnd: windows::Win32::Foundation::HWND) -> Option<usize> {
    hot_listbox_items()
        .lock()
        .unwrap()
        .get(&(hwnd.0 as isize))
        .copied()
        .flatten()
}

#[cfg(windows)]
fn set_hot_listbox_item(hwnd: windows::Win32::Foundation::HWND, item: Option<usize>) -> bool {
    let mut items = hot_listbox_items().lock().unwrap();
    let key = hwnd.0 as isize;
    let changed = items.get(&key).copied().flatten() != item;
    items.insert(key, item);
    changed
}

#[cfg(windows)]
fn clear_hot_listbox_item(hwnd: windows::Win32::Foundation::HWND) {
    hot_listbox_items()
        .lock()
        .unwrap()
        .remove(&(hwnd.0 as isize));
}

#[cfg(windows)]
const MODERN_LISTBOX_SUBCLASS_ID: usize = 5;

#[cfg(windows)]
unsafe extern "system" fn modern_listbox_subclass_proc(
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
        LB_ITEMFROMPOINT, SendMessageW, WM_ENABLE, WM_KILLFOCUS, WM_MOUSEMOVE, WM_NCDESTROY,
        WM_PAINT, WM_SETFOCUS,
    };

    if msg == WM_MOUSEMOVE {
        let result = unsafe { SendMessageW(hwnd, LB_ITEMFROMPOINT, None, Some(lparam)) }.0 as u32;
        let item = if result >> 16 == 0 {
            Some((result & 0xffff) as usize)
        } else {
            None
        };
        if set_hot_listbox_item(hwnd, item) {
            unsafe {
                let _ = InvalidateRect(Some(hwnd), None, false);
            }
        }
        let mut event = TRACKMOUSEEVENT {
            cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
            dwFlags: TME_LEAVE,
            hwndTrack: hwnd,
            dwHoverTime: 0,
        };
        unsafe {
            let _ = TrackMouseEvent(&mut event);
        }
    } else if msg == WM_MOUSELEAVE && set_hot_listbox_item(hwnd, None) {
        unsafe {
            let _ = InvalidateRect(Some(hwnd), None, false);
        }
    }

    if msg == WM_SETFOCUS || msg == WM_KILLFOCUS || msg == WM_ENABLE {
        unsafe {
            let _ = InvalidateRect(Some(hwnd), None, false);
        }
    }

    if msg == WM_PAINT {
        let result = unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) };
        unsafe {
            paint_modern_listbox_border(hwnd);
        }
        return result;
    }

    if msg == WM_NCDESTROY {
        clear_hot_listbox_item(hwnd);
        unsafe {
            let _ = RemoveWindowSubclass(hwnd, Some(modern_listbox_subclass_proc), subclass_id);
            return DefSubclassProc(hwnd, msg, wparam, lparam);
        }
    }

    unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) }
}

#[cfg(test)]
mod tests {
    use super::{
        ListBoxItemVisualState, ListBoxTextRect, ListBoxVisualState, RgbColor,
        listbox_item_palette, listbox_palette, listbox_uses_native_border,
        modern_listbox_text_rect,
    };

    #[test]
    fn maps_settings_profile_listbox() {
        assert!(super::is_modern_listbox(3101));
        assert!(!listbox_uses_native_border(3101));
    }

    #[test]
    fn ignores_unknown_controls() {
        assert!(!super::is_modern_listbox(9999));
        assert!(listbox_uses_native_border(9999));
    }

    #[test]
    fn normal_listbox_uses_white_surface() {
        let palette = listbox_palette(ListBoxVisualState::normal());

        assert_eq!(palette.background, RgbColor::new(255, 255, 255));
        assert_eq!(palette.border, RgbColor::new(203, 213, 225));
        assert_eq!(palette.text, RgbColor::new(31, 41, 55));
    }

    #[test]
    fn focused_listbox_uses_active_border() {
        let palette = listbox_palette(ListBoxVisualState {
            focused: true,
            ..ListBoxVisualState::normal()
        });

        assert_eq!(palette.border, RgbColor::new(37, 99, 235));
    }

    #[test]
    fn selected_item_matches_combo_dropdown_selection() {
        let palette = listbox_item_palette(ListBoxItemVisualState {
            selected: true,
            ..ListBoxItemVisualState::normal()
        });

        assert_eq!(palette.background, RgbColor::new(219, 234, 254));
        assert_eq!(palette.text, RgbColor::new(30, 64, 175));
    }

    #[test]
    fn text_rect_adds_horizontal_padding() {
        let rect = modern_listbox_text_rect(ListBoxTextRect {
            left: 0,
            top: 0,
            right: 220,
            bottom: 32,
        });

        assert_eq!(rect.left, 10);
        assert_eq!(rect.right, 210);
    }
}
