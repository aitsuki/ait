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
    pub arrow: RgbColor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComboListItemVisualState {
    pub selected: bool,
    pub disabled: bool,
}

impl ComboListItemVisualState {
    pub fn normal() -> Self {
        Self {
            selected: false,
            disabled: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComboListItemPalette {
    pub background: RgbColor,
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
            arrow: RgbColor::new(156, 163, 175),
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
        arrow: RgbColor::new(31, 41, 55),
    }
}

pub fn combo_list_item_palette(state: ComboListItemVisualState) -> ComboListItemPalette {
    if state.disabled {
        return ComboListItemPalette {
            background: RgbColor::new(243, 244, 246),
            text: RgbColor::new(156, 163, 175),
        };
    }

    if state.selected {
        ComboListItemPalette {
            background: RgbColor::new(219, 234, 254),
            text: RgbColor::new(30, 64, 175),
        }
    } else {
        ComboListItemPalette {
            background: RgbColor::new(255, 255, 255),
            text: RgbColor::new(31, 41, 55),
        }
    }
}

pub fn combo_owner_draws_list_item(is_combo_edit_area: bool) -> bool {
    !is_combo_edit_area
}

pub fn modern_combo_dropdown_item_fill_rect(rect: ComboTextRect) -> ComboTextRect {
    ComboTextRect {
        left: rect.left + 1,
        top: rect.top,
        right: (rect.right - 1).max(rect.left + 1),
        bottom: rect.bottom,
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
        left,
        top,
        right,
        bottom: top + MODERN_COMBO_VISIBLE_HEIGHT,
    }
}

pub fn modern_combo_child_rect(id: usize, x: i32, y: i32, width: i32, height: i32) -> ComboRect {
    let _ = id;
    ComboRect {
        x,
        y,
        width,
        height,
    }
}
pub const MODERN_COMBO_VISIBLE_HEIGHT: i32 = 26;
pub const MODERN_COMBO_LIST_ITEM_HEIGHT: u32 = 28;
pub const MODERN_COMBO_ARROW_WIDTH: i32 = 28;
pub const MODERN_COMBO_TEXT_PADDING: i32 = 9;
pub const MODERN_COMBO_LIST_TEXT_PADDING: i32 = 10;
pub const MODERN_COMBO_DROPDOWN_RADIUS: i32 = 7;

pub fn modern_combo_text_rect(left: i32, top: i32, right: i32, bottom: i32) -> ComboTextRect {
    ComboTextRect {
        left: left + MODERN_COMBO_TEXT_PADDING,
        top,
        right: (right - MODERN_COMBO_ARROW_WIDTH).max(left + MODERN_COMBO_TEXT_PADDING),
        bottom,
    }
}

pub fn modern_combo_visible_region_rect(width: i32) -> ComboRect {
    ComboRect {
        x: 0,
        y: 0,
        width,
        height: MODERN_COMBO_VISIBLE_HEIGHT,
    }
}

pub fn combo_dropdown_region_needs_update(
    previous: Option<(i32, i32)>,
    width: i32,
    height: i32,
) -> bool {
    width > 0 && height > 0 && previous != Some((width, height))
}

pub fn modern_combo_dropdown_border_rect(width: i32, height: i32) -> ComboTextRect {
    ComboTextRect {
        left: 0,
        top: 0,
        right: (width - 1).max(0),
        bottom: (height - 1).max(0),
    }
}

pub fn modern_combo_dropdown_window_style(style: u32, native_border_style: u32) -> u32 {
    style & !native_border_style
}

pub fn modern_combo_dropdown_border_color() -> RgbColor {
    RgbColor::new(37, 99, 235)
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
pub unsafe fn paint_modern_combo(hwnd: windows::Win32::Foundation::HWND) {
    use windows::Win32::Graphics::Gdi::{
        BACKGROUND_MODE, BeginPaint, CreatePen, CreateRoundRectRgn, CreateSolidBrush, DeleteObject,
        DrawTextW, EndPaint, FillRect, GetStockObject, HGDIOBJ, LineTo, MoveToEx, NULL_BRUSH,
        PS_SOLID, RoundRect, SelectClipRgn, SelectObject, SetBkMode, SetTextColor, TRANSPARENT,
    };
    use windows::Win32::UI::WindowsAndMessaging::GetClientRect;

    let state = unsafe { combo_visual_state_for_child(hwnd) };
    let palette = combo_palette(state);
    let mut rect = windows::Win32::Foundation::RECT::default();
    if unsafe { GetClientRect(hwnd, &mut rect).is_err() } {
        return;
    }
    let frame = modern_combo_frame_rect(rect.left, rect.top, rect.right, rect.bottom);
    rect.left = frame.left;
    rect.top = frame.top;
    rect.right = frame.right;
    rect.bottom = frame.bottom;

    let mut paint = windows::Win32::Graphics::Gdi::PAINTSTRUCT::default();
    let hdc = unsafe { BeginPaint(hwnd, &mut paint) };

    let background = unsafe { CreateSolidBrush(palette.background.colorref()) };
    let clip_region =
        unsafe { CreateRoundRectRgn(rect.left, rect.top, rect.right + 1, rect.bottom + 1, 7, 7) };
    if !clip_region.is_invalid() {
        unsafe {
            let _ = SelectClipRgn(hdc, Some(clip_region));
        }
    }
    if !background.is_invalid() {
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

    let mut text = combo_text(hwnd);
    let text_frame = modern_combo_text_rect(rect.left, rect.top, rect.right, rect.bottom);
    let mut text_rect = windows::Win32::Foundation::RECT {
        left: text_frame.left,
        top: text_frame.top,
        right: text_frame.right,
        bottom: text_frame.bottom,
    };
    unsafe {
        let _ = SetBkMode(hdc, TRANSPARENT);
        let _ = SetTextColor(hdc, palette.text.colorref());
        let _ = DrawTextW(
            hdc,
            &mut text,
            &mut text_rect,
            windows::Win32::Graphics::Gdi::DT_SINGLELINE
                | windows::Win32::Graphics::Gdi::DT_VCENTER
                | windows::Win32::Graphics::Gdi::DT_END_ELLIPSIS,
        );
        let _ = SetBkMode(hdc, BACKGROUND_MODE(1));
    }

    let arrow_pen = unsafe { CreatePen(PS_SOLID, 2, palette.arrow.colorref()) };
    let old_arrow_pen = if arrow_pen.is_invalid() {
        HGDIOBJ::default()
    } else {
        unsafe { SelectObject(hdc, arrow_pen.into()) }
    };
    let arrow_center_x = rect.right - MODERN_COMBO_ARROW_WIDTH / 2;
    let arrow_center_y = rect.top + (rect.bottom - rect.top) / 2 + 1;
    unsafe {
        let _ = MoveToEx(hdc, arrow_center_x - 5, arrow_center_y - 2, None);
        let _ = LineTo(hdc, arrow_center_x, arrow_center_y + 3);
        let _ = LineTo(hdc, arrow_center_x + 5, arrow_center_y - 2);
    }
    if !old_arrow_pen.is_invalid() {
        unsafe {
            let _ = SelectObject(hdc, old_arrow_pen);
        }
    }
    if !arrow_pen.is_invalid() {
        unsafe {
            let _ = DeleteObject(arrow_pen.into());
        }
    }

    let pen = unsafe { CreatePen(PS_SOLID, 1, palette.border.colorref()) };
    if !pen.is_invalid() {
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
        }
    }

    if !background.is_invalid() {
        unsafe {
            let _ = DeleteObject(background.into());
        }
    }
    unsafe {
        let _ = EndPaint(hwnd, &paint);
    }
}

#[cfg(windows)]
pub unsafe fn measure_owner_draw_combo_item(
    measure_item: *mut windows::Win32::UI::Controls::MEASUREITEMSTRUCT,
) -> bool {
    use windows::Win32::UI::Controls::ODT_COMBOBOX;

    let Some(measure_item) = (unsafe { measure_item.as_mut() }) else {
        return false;
    };
    if !is_modern_combo(measure_item.CtlID as usize) || measure_item.CtlType != ODT_COMBOBOX {
        return false;
    }

    measure_item.itemHeight = MODERN_COMBO_LIST_ITEM_HEIGHT;
    true
}

#[cfg(windows)]
pub unsafe fn draw_owner_draw_combo_item(
    draw_item: *const windows::Win32::UI::Controls::DRAWITEMSTRUCT,
) -> bool {
    use windows::Win32::Foundation::{LPARAM, RECT, WPARAM};
    use windows::Win32::Graphics::Gdi::{
        BACKGROUND_MODE, CreateSolidBrush, DT_END_ELLIPSIS, DT_SINGLELINE, DT_VCENTER,
        DeleteObject, DrawTextW, FillRect, GetBkMode, GetStockObject, GetTextColor, SetBkMode,
        SetTextColor, TRANSPARENT, WHITE_BRUSH,
    };
    use windows::Win32::UI::Controls::{
        ODS_COMBOBOXEDIT, ODS_DISABLED, ODS_SELECTED, ODT_COMBOBOX,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        CB_ERR, CB_GETLBTEXT, CB_GETLBTEXTLEN, SendMessageW,
    };

    let Some(draw_item) = (unsafe { draw_item.as_ref() }) else {
        return false;
    };
    if !is_modern_combo(draw_item.CtlID as usize) || draw_item.CtlType != ODT_COMBOBOX {
        return false;
    }
    if !combo_owner_draws_list_item((draw_item.itemState.0 & ODS_COMBOBOXEDIT.0) != 0) {
        return true;
    }

    let selected = (draw_item.itemState.0 & ODS_SELECTED.0) != 0;
    let disabled = (draw_item.itemState.0 & ODS_DISABLED.0) != 0;
    let palette = combo_list_item_palette(ComboListItemVisualState { selected, disabled });
    let hdc = draw_item.hDC;
    let rect = draw_item.rcItem;
    let fill_rect = modern_combo_dropdown_item_fill_rect(ComboTextRect {
        left: rect.left,
        top: rect.top,
        right: rect.right,
        bottom: rect.bottom,
    });
    let fill_rect = RECT {
        left: fill_rect.left,
        top: fill_rect.top,
        right: fill_rect.right,
        bottom: fill_rect.bottom,
    };

    let background = unsafe { CreateSolidBrush(palette.background.colorref()) };
    if background.is_invalid() {
        unsafe {
            let _ = FillRect(
                hdc,
                &fill_rect,
                windows::Win32::Graphics::Gdi::HBRUSH(GetStockObject(WHITE_BRUSH).0),
            );
        }
    } else {
        unsafe {
            let _ = FillRect(hdc, &fill_rect, background);
        }
    }

    if draw_item.itemID != u32::MAX {
        let len = unsafe {
            SendMessageW(
                draw_item.hwndItem,
                CB_GETLBTEXTLEN,
                Some(WPARAM(draw_item.itemID as usize)),
                Some(LPARAM(0)),
            )
            .0
        };
        if len != CB_ERR as isize {
            let mut text = vec![0u16; len as usize + 1];
            unsafe {
                let _ = SendMessageW(
                    draw_item.hwndItem,
                    CB_GETLBTEXT,
                    Some(WPARAM(draw_item.itemID as usize)),
                    Some(LPARAM(text.as_mut_ptr() as isize)),
                );
            }
            text.truncate(len as usize);

            let mut text_rect = RECT {
                left: rect.left + MODERN_COMBO_LIST_TEXT_PADDING,
                top: rect.top,
                right: rect.right - MODERN_COMBO_LIST_TEXT_PADDING,
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
                    DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS,
                );
                let _ = SetTextColor(hdc, old_text_color);
                let _ = SetBkMode(hdc, BACKGROUND_MODE(old_bk_mode as u32));
            }
        }
    }

    if !background.is_invalid() {
        unsafe {
            let _ = DeleteObject(background.into());
        }
    }
    if let Some(list) = unsafe { combo_list_hwnd(draw_item.hwndItem) } {
        unsafe {
            paint_modern_combo_dropdown_border(list);
        }
    }
    true
}

#[cfg(windows)]
pub unsafe fn prepare_modern_combo_dropdown(hwnd: windows::Win32::Foundation::HWND) {
    let Some(list) = (unsafe { combo_list_hwnd(hwnd) }) else {
        return;
    };
    unsafe {
        remove_modern_combo_dropdown_native_border(list);
        apply_modern_combo_dropdown_region(list);
        let _ = install_modern_combo_dropdown_tracking(list);
        redraw_modern_combo_dropdown(list);
    }
}

#[cfg(windows)]
unsafe fn combo_list_hwnd(
    hwnd: windows::Win32::Foundation::HWND,
) -> Option<windows::Win32::Foundation::HWND> {
    use windows::Win32::UI::Controls::{COMBOBOXINFO, GetComboBoxInfo};

    let mut info = COMBOBOXINFO {
        cbSize: std::mem::size_of::<COMBOBOXINFO>() as u32,
        ..Default::default()
    };
    if unsafe { GetComboBoxInfo(hwnd, &mut info).is_err() } || info.hwndList.is_invalid() {
        None
    } else {
        Some(info.hwndList)
    }
}

#[cfg(windows)]
fn installed_dropdown_lists() -> &'static std::sync::Mutex<std::collections::HashSet<isize>> {
    use std::collections::HashSet;
    use std::sync::{Mutex, OnceLock};

    static INSTALLED: OnceLock<Mutex<HashSet<isize>>> = OnceLock::new();
    INSTALLED.get_or_init(|| Mutex::new(HashSet::new()))
}

#[cfg(windows)]
fn dropdown_list_regions() -> &'static std::sync::Mutex<std::collections::HashMap<isize, (i32, i32)>>
{
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};

    static REGIONS: OnceLock<Mutex<HashMap<isize, (i32, i32)>>> = OnceLock::new();
    REGIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

#[cfg(windows)]
unsafe fn install_modern_combo_dropdown_tracking(
    hwnd: windows::Win32::Foundation::HWND,
) -> crate::error::Result<()> {
    use windows::Win32::UI::Shell::SetWindowSubclass;

    {
        let mut installed = installed_dropdown_lists().lock().unwrap();
        if !installed.insert(hwnd.0 as isize) {
            return Ok(());
        }
    }

    unsafe {
        if SetWindowSubclass(
            hwnd,
            Some(modern_combo_dropdown_subclass_proc),
            MODERN_COMBO_DROPDOWN_SUBCLASS_ID,
            0,
        )
        .as_bool()
        {
            Ok(())
        } else {
            installed_dropdown_lists()
                .lock()
                .unwrap()
                .remove(&(hwnd.0 as isize));
            Err(crate::error::AppError::Windows(
                "安装下拉列表绘制处理失败".to_string(),
            ))
        }
    }
}

#[cfg(windows)]
unsafe fn remove_modern_combo_dropdown_native_border(hwnd: windows::Win32::Foundation::HWND) {
    use windows::Win32::UI::WindowsAndMessaging::{
        GWL_STYLE, GetWindowLongW, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOOWNERZORDER,
        SWP_NOSIZE, SWP_NOZORDER, SetWindowLongW, SetWindowPos, WS_BORDER,
    };

    let style = unsafe { GetWindowLongW(hwnd, GWL_STYLE) } as u32;
    let next_style = modern_combo_dropdown_window_style(style, WS_BORDER.0);
    if next_style == style {
        return;
    }

    unsafe {
        let _ = SetWindowLongW(hwnd, GWL_STYLE, next_style as i32);
        let _ = SetWindowPos(
            hwnd,
            None,
            0,
            0,
            0,
            0,
            SWP_NOMOVE
                | SWP_NOSIZE
                | SWP_NOZORDER
                | SWP_NOACTIVATE
                | SWP_NOOWNERZORDER
                | SWP_FRAMECHANGED,
        );
    }
}

#[cfg(windows)]
unsafe fn redraw_modern_combo_dropdown(hwnd: windows::Win32::Foundation::HWND) {
    use windows::Win32::Graphics::Gdi::{
        RDW_FRAME, RDW_INVALIDATE, RDW_NOERASE, RDW_UPDATENOW, RedrawWindow,
    };

    unsafe {
        let _ = RedrawWindow(
            Some(hwnd),
            None,
            None,
            RDW_INVALIDATE | RDW_FRAME | RDW_UPDATENOW | RDW_NOERASE,
        );
        paint_modern_combo_dropdown_border(hwnd);
    }
}

#[cfg(windows)]
unsafe fn apply_modern_combo_dropdown_region(hwnd: windows::Win32::Foundation::HWND) {
    use windows::Win32::Foundation::RECT;
    use windows::Win32::Graphics::Gdi::{CreateRoundRectRgn, SetWindowRgn};
    use windows::Win32::UI::WindowsAndMessaging::GetWindowRect;

    let mut rect = RECT::default();
    if unsafe { GetWindowRect(hwnd, &mut rect).is_err() } {
        return;
    }
    let width = rect.right - rect.left;
    let height = rect.bottom - rect.top;
    if width <= 0 || height <= 0 {
        return;
    }

    {
        let mut regions = dropdown_list_regions().lock().unwrap();
        let size = (width, height);
        if !combo_dropdown_region_needs_update(
            regions.get(&(hwnd.0 as isize)).copied(),
            width,
            height,
        ) {
            return;
        }
        regions.insert(hwnd.0 as isize, size);
    }

    let region = unsafe {
        CreateRoundRectRgn(
            0,
            0,
            width + 1,
            height + 1,
            MODERN_COMBO_DROPDOWN_RADIUS,
            MODERN_COMBO_DROPDOWN_RADIUS,
        )
    };
    if !region.is_invalid() {
        unsafe {
            let _ = SetWindowRgn(hwnd, Some(region), false);
        }
    }
}

#[cfg(windows)]
unsafe fn paint_modern_combo_dropdown_border(hwnd: windows::Win32::Foundation::HWND) {
    use windows::Win32::Foundation::RECT;
    use windows::Win32::Graphics::Gdi::{
        CreatePen, DeleteObject, GetStockObject, HGDIOBJ, NULL_BRUSH, PS_SOLID, RoundRect,
        SelectObject,
    };
    use windows::Win32::UI::WindowsAndMessaging::GetClientRect;

    let mut rect = RECT::default();
    if unsafe { GetClientRect(hwnd, &mut rect).is_err() } {
        return;
    }
    if rect.right <= rect.left || rect.bottom <= rect.top {
        return;
    }
    let border_rect =
        modern_combo_dropdown_border_rect(rect.right - rect.left, rect.bottom - rect.top);

    let hdc = unsafe { windows::Win32::Graphics::Gdi::GetWindowDC(Some(hwnd)) };
    if hdc.is_invalid() {
        return;
    }

    let old_brush = unsafe { SelectObject(hdc, GetStockObject(NULL_BRUSH)) };
    let pen = unsafe { CreatePen(PS_SOLID, 1, modern_combo_dropdown_border_color().colorref()) };
    let old_pen = if pen.is_invalid() {
        HGDIOBJ::default()
    } else {
        unsafe { SelectObject(hdc, pen.into()) }
    };
    unsafe {
        let _ = RoundRect(
            hdc,
            border_rect.left,
            border_rect.top,
            border_rect.right,
            border_rect.bottom,
            MODERN_COMBO_DROPDOWN_RADIUS,
            MODERN_COMBO_DROPDOWN_RADIUS,
        );
    }
    if !old_pen.is_invalid() {
        unsafe {
            let _ = SelectObject(hdc, old_pen);
        }
    }
    if !old_brush.is_invalid() {
        unsafe {
            let _ = SelectObject(hdc, old_brush);
        }
    }
    if !pen.is_invalid() {
        unsafe {
            let _ = DeleteObject(pen.into());
        }
    }
    unsafe {
        let _ = windows::Win32::Graphics::Gdi::ReleaseDC(Some(hwnd), hdc);
    }
}

#[cfg(windows)]
fn combo_text(hwnd: windows::Win32::Foundation::HWND) -> Vec<u16> {
    let len = unsafe { windows::Win32::UI::WindowsAndMessaging::GetWindowTextLengthW(hwnd) };
    let mut text = vec![0u16; len as usize + 1];
    if len > 0 {
        let copied =
            unsafe { windows::Win32::UI::WindowsAndMessaging::GetWindowTextW(hwnd, &mut text) };
        text.truncate(copied as usize);
    } else {
        text.clear();
    }
    text
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
    _parent: windows::Win32::Foundation::HWND,
    child: windows::Win32::Foundation::HWND,
) {
    use windows::Win32::Graphics::Gdi::InvalidateRect;

    let rect = windows::Win32::Foundation::RECT {
        left: 0,
        top: 0,
        right: 32000,
        bottom: MODERN_COMBO_VISIBLE_HEIGHT,
    };
    unsafe {
        let _ = InvalidateRect(Some(child), Some(&rect), false);
    }
}

#[cfg(windows)]
pub fn install_modern_combo_tracking(
    hwnd: windows::Win32::Foundation::HWND,
) -> crate::error::Result<()> {
    use windows::Win32::UI::Shell::SetWindowSubclass;

    unsafe {
        apply_modern_combo_visible_region(hwnd);
        if SetWindowSubclass(
            hwnd,
            Some(modern_combo_subclass_proc),
            MODERN_COMBO_SUBCLASS_ID,
            0,
        )
        .as_bool()
        {
            prepare_modern_combo_dropdown(hwnd);
            Ok(())
        } else {
            Err(crate::error::AppError::Windows(
                "安装下拉框焦点处理失败".to_string(),
            ))
        }
    }
}

#[cfg(windows)]
unsafe fn apply_modern_combo_visible_region(hwnd: windows::Win32::Foundation::HWND) {
    use windows::Win32::Foundation::RECT;
    use windows::Win32::Graphics::Gdi::{CreateRoundRectRgn, SetWindowRgn};
    use windows::Win32::UI::WindowsAndMessaging::GetClientRect;

    let mut rect = RECT::default();
    if unsafe { GetClientRect(hwnd, &mut rect).is_err() } {
        return;
    }
    let region_rect = modern_combo_visible_region_rect(rect.right - rect.left);
    if region_rect.width <= 0 || region_rect.height <= 0 {
        return;
    }
    let region = unsafe {
        CreateRoundRectRgn(
            region_rect.x,
            region_rect.y,
            region_rect.width + 1,
            region_rect.height + 1,
            MODERN_COMBO_DROPDOWN_RADIUS,
            MODERN_COMBO_DROPDOWN_RADIUS,
        )
    };
    if !region.is_invalid() {
        unsafe {
            let _ = SetWindowRgn(hwnd, Some(region), true);
        }
    }
}

#[cfg(windows)]
const MODERN_COMBO_SUBCLASS_ID: usize = 3;
#[cfg(windows)]
const MODERN_COMBO_DROPDOWN_SUBCLASS_ID: usize = 4;

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
        GetParent, WM_ENABLE, WM_ERASEBKGND, WM_KILLFOCUS, WM_NCDESTROY, WM_PAINT, WM_SETFOCUS,
        WM_SIZE,
    };

    if msg == WM_PAINT {
        unsafe {
            paint_modern_combo(hwnd);
        }
        return windows::Win32::Foundation::LRESULT(0);
    }

    if msg == WM_ERASEBKGND {
        return windows::Win32::Foundation::LRESULT(1);
    }

    if msg == WM_SETFOCUS || msg == WM_KILLFOCUS || msg == WM_ENABLE {
        if let Ok(parent) = unsafe { GetParent(hwnd) } {
            unsafe {
                invalidate_modern_combo_for_child(parent, hwnd);
            }
        }
    }

    if msg == WM_SIZE {
        unsafe {
            apply_modern_combo_visible_region(hwnd);
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

#[cfg(windows)]
unsafe extern "system" fn modern_combo_dropdown_subclass_proc(
    hwnd: windows::Win32::Foundation::HWND,
    msg: u32,
    wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
    subclass_id: usize,
    _ref_data: usize,
) -> windows::Win32::Foundation::LRESULT {
    use windows::Win32::UI::Shell::{DefSubclassProc, RemoveWindowSubclass};
    use windows::Win32::UI::WindowsAndMessaging::{
        WM_NCDESTROY, WM_NCPAINT, WM_PAINT, WM_SHOWWINDOW, WM_WINDOWPOSCHANGED,
    };

    if msg == WM_SHOWWINDOW || msg == WM_WINDOWPOSCHANGED {
        let result = unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) };
        unsafe {
            apply_modern_combo_dropdown_region(hwnd);
            paint_modern_combo_dropdown_border(hwnd);
        }
        return result;
    }

    if msg == WM_PAINT || msg == WM_NCPAINT {
        let result = unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) };
        unsafe {
            paint_modern_combo_dropdown_border(hwnd);
        }
        return result;
    }

    if msg == WM_NCDESTROY {
        installed_dropdown_lists()
            .lock()
            .unwrap()
            .remove(&(hwnd.0 as isize));
        dropdown_list_regions()
            .lock()
            .unwrap()
            .remove(&(hwnd.0 as isize));
        unsafe {
            let _ =
                RemoveWindowSubclass(hwnd, Some(modern_combo_dropdown_subclass_proc), subclass_id);
            return DefSubclassProc(hwnd, msg, wparam, lparam);
        }
    }

    unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) }
}

#[cfg(test)]
mod tests {
    use super::{
        ComboListItemVisualState, ComboTextRect, ComboVisualState, RgbColor,
        combo_dropdown_region_needs_update, combo_list_item_palette, combo_owner_draws_list_item,
        combo_palette, combo_uses_native_border, is_modern_combo, modern_combo_child_rect,
        modern_combo_dropdown_border_color, modern_combo_dropdown_border_rect,
        modern_combo_dropdown_item_fill_rect, modern_combo_dropdown_window_style,
        modern_combo_frame_rect, modern_combo_text_rect, modern_combo_visible_region_rect,
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
        assert_eq!(palette.arrow, RgbColor::new(31, 41, 55));
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
    fn normal_combo_list_item_uses_white_surface() {
        let palette = combo_list_item_palette(ComboListItemVisualState::normal());
        assert_eq!(palette.background, RgbColor::new(255, 255, 255));
        assert_eq!(palette.text, RgbColor::new(31, 41, 55));
    }

    #[test]
    fn selected_combo_list_item_uses_soft_blue() {
        let palette = combo_list_item_palette(ComboListItemVisualState {
            selected: true,
            ..ComboListItemVisualState::normal()
        });
        assert_eq!(palette.background, RgbColor::new(219, 234, 254));
        assert_eq!(palette.text, RgbColor::new(30, 64, 175));
    }

    #[test]
    fn owner_draw_skips_combo_edit_area() {
        assert!(!combo_owner_draws_list_item(true));
        assert!(combo_owner_draws_list_item(false));
    }

    #[test]
    fn frame_rect_matches_control_bounds() {
        let rect = modern_combo_frame_rect(0, 0, 180, 220);
        assert_eq!(rect.left, 0);
        assert_eq!(rect.top, 0);
        assert_eq!(rect.right, 180);
        assert_eq!(rect.bottom, 26);
    }

    #[test]
    fn modern_combo_child_rect_keeps_native_control_bounds() {
        let rect = modern_combo_child_rect(2106, 408, 12, 180, 220);

        assert_eq!(rect.x, 408);
        assert_eq!(rect.y, 12);
        assert_eq!(rect.width, 180);
        assert_eq!(rect.height, 220);
    }

    #[test]
    fn text_rect_leaves_room_for_custom_arrow() {
        let rect = modern_combo_text_rect(0, 0, 180, 26);

        assert_eq!(rect.left, 9);
        assert_eq!(rect.top, 0);
        assert_eq!(rect.right, 152);
        assert_eq!(rect.bottom, 26);
    }

    #[test]
    fn visible_region_clips_native_combo_dropdown_area() {
        let rect = modern_combo_visible_region_rect(180);

        assert_eq!(rect.x, 0);
        assert_eq!(rect.y, 0);
        assert_eq!(rect.width, 180);
        assert_eq!(rect.height, 26);
    }

    #[test]
    fn dropdown_region_updates_only_when_visible_size_changes() {
        assert!(combo_dropdown_region_needs_update(None, 180, 220));
        assert!(!combo_dropdown_region_needs_update(
            Some((180, 220)),
            180,
            220
        ));
        assert!(combo_dropdown_region_needs_update(
            Some((180, 220)),
            181,
            220
        ));
        assert!(combo_dropdown_region_needs_update(
            Some((180, 220)),
            180,
            221
        ));
        assert!(!combo_dropdown_region_needs_update(
            Some((180, 220)),
            0,
            220
        ));
        assert!(!combo_dropdown_region_needs_update(
            Some((180, 220)),
            180,
            0
        ));
    }

    #[test]
    fn dropdown_border_draws_inside_clipped_region() {
        let rect = modern_combo_dropdown_border_rect(180, 220);

        assert_eq!(rect.left, 0);
        assert_eq!(rect.top, 0);
        assert_eq!(rect.right, 179);
        assert_eq!(rect.bottom, 219);
    }

    #[test]
    fn dropdown_border_matches_focused_control_blue() {
        assert_eq!(
            modern_combo_dropdown_border_color(),
            RgbColor::new(37, 99, 235)
        );
    }

    #[test]
    fn dropdown_item_fill_leaves_horizontal_border_pixels() {
        let rect = modern_combo_dropdown_item_fill_rect(ComboTextRect {
            left: 0,
            top: 28,
            right: 180,
            bottom: 56,
        });

        assert_eq!(rect.left, 1);
        assert_eq!(rect.top, 28);
        assert_eq!(rect.right, 179);
        assert_eq!(rect.bottom, 56);
    }

    #[test]
    fn dropdown_window_style_removes_native_border() {
        let ws_border = 0x0080_0000;
        let ws_visible = 0x1000_0000;

        assert_eq!(
            modern_combo_dropdown_window_style(ws_visible | ws_border, ws_border),
            ws_visible
        );
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
