use crate::error::{AppError, Result};

#[cfg(windows)]
const ID_SOURCE_EDIT: isize = 2101;
#[cfg(windows)]
const ID_TRANSLATED_EDIT: isize = 2102;
#[cfg(windows)]
const ID_SOURCE_LABEL: isize = 2103;
#[cfg(windows)]
const ID_TRANSLATED_LABEL: isize = 2104;
#[cfg(windows)]
const ID_STATUS_TEXT: isize = 2105;
#[cfg(windows)]
const ID_PROFILE_COMBO: isize = 2106;
#[cfg(windows)]
const ID_TRANSLATE: usize = 2001;
#[cfg(windows)]
pub const WM_TRANSLATE_WINDOW_SOURCE: u32 = windows::Win32::UI::WindowsAndMessaging::WM_APP + 30;
#[cfg(windows)]
pub const WM_TRANSLATE_WINDOW_PROFILE_CHANGED: u32 =
    windows::Win32::UI::WindowsAndMessaging::WM_APP + 31;

#[derive(Debug, Clone)]
pub struct TranslationWindowState {
    pub source_text: String,
    pub translated_text: String,
    pub loading: bool,
    pub error: Option<String>,
}

impl TranslationWindowState {
    pub fn mark_starting(&mut self) {
        self.loading = true;
        self.error = None;
    }

    pub fn apply_translation_result(&mut self, result: &crate::app::TranslationWorkflowResult) {
        self.source_text = result.source_text.clone();
        self.translated_text = result.translated_text.clone();
        self.loading = false;
        self.error = None;
    }

    pub fn with_profile_switch_error(mut self, message: String) -> Self {
        self.loading = false;
        self.error = Some(message);
        self
    }

    pub fn with_app_error(mut self, err: &crate::error::AppError) -> Self {
        self.loading = false;
        self.error = Some(err.user_summary());
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShowMode {
    Starting,
    Loading,
    Result,
    Error,
}

impl ShowMode {
    pub fn activates_window(self) -> bool {
        !matches!(self, Self::Starting)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowZOrder {
    NotTopmost,
    TopmostNoActivate,
}

pub fn window_z_order() -> WindowZOrder {
    WindowZOrder::NotTopmost
}

pub fn show_window_z_order(mode: ShowMode) -> WindowZOrder {
    match mode {
        ShowMode::Starting => WindowZOrder::TopmostNoActivate,
        ShowMode::Loading | ShowMode::Result | ShowMode::Error => WindowZOrder::NotTopmost,
    }
}

pub fn show_window_needs_topmost_reset(mode: ShowMode, action: ShowAction) -> bool {
    !matches!(mode, ShowMode::Starting)
        && matches!(action, ShowAction::ActivateOnly | ShowAction::KeepPosition)
}

pub fn show_window_needs_topmost_raise(mode: ShowMode, action: ShowAction) -> bool {
    matches!(mode, ShowMode::Starting) && matches!(action, ShowAction::ActivateOnly)
}

pub fn translation_window_min_client_size() -> (i32, i32) {
    (420, 300)
}

pub fn translation_profile_combo_dropdown_height() -> i32 {
    220
}

pub fn translation_window_update_button_visible(
    status: Option<&crate::update::UpdateStatus>,
) -> bool {
    matches!(status, Some(crate::update::UpdateStatus::UpdateAvailable { .. }))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranslationProfileOption {
    pub id: String,
    pub label: String,
    pub active: bool,
}

impl TranslationProfileOption {
    pub fn from_settings(
        settings: &crate::config::AppSettings,
        active_profile_id: &str,
    ) -> Vec<Self> {
        settings
            .translator_profiles
            .iter()
            .map(|profile| Self {
                id: profile.id.clone(),
                label: profile.name.clone(),
                active: profile.id == active_profile_id,
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileSelectionAction {
    SaveDefaultOnly { profile_id: String },
    SaveDefaultAndRetranslate { profile_id: String },
}

pub fn profile_selection_action(profile_id: &str, source_text: &str) -> ProfileSelectionAction {
    if source_text.trim().is_empty() {
        ProfileSelectionAction::SaveDefaultOnly {
            profile_id: profile_id.to_string(),
        }
    } else {
        ProfileSelectionAction::SaveDefaultAndRetranslate {
            profile_id: profile_id.to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ControlRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TranslationWindowLayout {
    pub profile_combo: ControlRect,
    pub source_label: ControlRect,
    pub source_edit: ControlRect,
    pub translated_label: ControlRect,
    pub translated_edit: ControlRect,
    pub status_text: ControlRect,
    pub translate_button: ControlRect,
}

pub fn translation_window_layout(client_width: i32, client_height: i32) -> TranslationWindowLayout {
    const MARGIN: i32 = 16;
    const GAP: i32 = 10;
    const LABEL_HEIGHT: i32 = 20;
    const STATUS_HEIGHT: i32 = 22;
    const BUTTON_WIDTH: i32 = 52;
    const BUTTON_HEIGHT: i32 = 28;
    const MIN_EDIT_HEIGHT: i32 = 64;
    const MAX_SOURCE_EDIT_HEIGHT: i32 = 200;

    let usable_width = client_width.max(1);
    let usable_height = client_height.max(1);
    let content_x = MARGIN.min(usable_width - 1);
    let content_width = (usable_width - content_x - MARGIN).max(1);
    let combo_width = 180.min(content_width);
    let combo_height = 26.min(usable_height);
    let button_width = BUTTON_WIDTH.min(content_width);
    let button_height = BUTTON_HEIGHT.min(usable_height);
    let button_x = (usable_width - MARGIN - button_width).clamp(0, usable_width - button_width);
    let bottom_y = (usable_height - MARGIN - button_height).clamp(0, usable_height - button_height);
    let status_width = (button_x - MARGIN - GAP).max(1);
    let label_height = LABEL_HEIGHT.min(usable_height);
    let status_height = STATUS_HEIGHT.min(usable_height);
    let profile_combo = ControlRect {
        x: (usable_width - MARGIN - combo_width).clamp(0, usable_width - combo_width),
        y: 12.min(usable_height - combo_height),
        width: combo_width,
        height: combo_height,
    };

    let source_label = ControlRect {
        x: content_x,
        y: 14.min(usable_height - label_height),
        width: 80.min(content_width),
        height: label_height,
    };
    let source_edit_y = (source_label.y + label_height + 4).min(usable_height - 1);
    let edit_area_bottom = (bottom_y - GAP).max(source_edit_y + 1);
    let fixed_between_edits = GAP + label_height + 4;
    let available_edit_height = (edit_area_bottom - source_edit_y - fixed_between_edits).max(2);
    let half_edit_height = available_edit_height / 2;
    let source_edit_height = half_edit_height
        .min(MAX_SOURCE_EDIT_HEIGHT)
        .max(MIN_EDIT_HEIGHT.min(half_edit_height.max(1)))
        .min(usable_height - source_edit_y);
    let translated_label_y = source_edit_y + source_edit_height + GAP;
    let translated_label_y = translated_label_y.min(usable_height - 1);
    let translated_edit_y = (translated_label_y + label_height + 4).min(usable_height - 1);
    let translated_edit_height = (bottom_y - translated_edit_y - GAP)
        .max(1)
        .min(usable_height - translated_edit_y);

    TranslationWindowLayout {
        profile_combo,
        source_label,
        source_edit: ControlRect {
            x: content_x,
            y: source_edit_y,
            width: content_width,
            height: source_edit_height,
        },
        translated_label: ControlRect {
            x: content_x,
            y: translated_label_y,
            width: 80.min(content_width),
            height: label_height.min(usable_height - translated_label_y),
        },
        translated_edit: ControlRect {
            x: content_x,
            y: translated_edit_y,
            width: content_width,
            height: translated_edit_height,
        },
        status_text: ControlRect {
            x: content_x,
            y: (bottom_y + 3).min(usable_height - status_height),
            width: status_width,
            height: status_height,
        },
        translate_button: ControlRect {
            x: button_x,
            y: bottom_y,
            width: button_width,
            height: button_height,
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShowAction {
    PositionAndActivate,
    ActivateOnly,
    KeepPosition,
}

pub fn show_action(is_visible: bool, is_foreground: bool) -> ShowAction {
    if !is_visible {
        ShowAction::PositionAndActivate
    } else if !is_foreground {
        ShowAction::ActivateOnly
    } else {
        ShowAction::KeepPosition
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditShortcutAction {
    None,
    SelectAll,
    HideWindow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditCharAction {
    Default,
    Swallow,
}

pub fn edit_shortcut_action(vk: u32, ctrl_down: bool) -> EditShortcutAction {
    const VK_A: u32 = 0x41;
    const VK_ESCAPE: u32 = 0x1B;

    if ctrl_down && vk == VK_A {
        EditShortcutAction::SelectAll
    } else if vk == VK_ESCAPE {
        EditShortcutAction::HideWindow
    } else {
        EditShortcutAction::None
    }
}

pub fn edit_char_action(ch: u32) -> EditCharAction {
    const CTRL_A: u32 = 0x01;

    if ch == CTRL_A {
        EditCharAction::Swallow
    } else {
        EditCharAction::Default
    }
}

pub fn edit_display_text(text: &str) -> String {
    let mut normalized = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '\r' => {
                normalized.push('\r');
                normalized.push('\n');
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
            }
            '\n' => {
                normalized.push('\r');
                normalized.push('\n');
            }
            '\u{000B}' | '\u{000C}' | '\u{0085}' | '\u{2028}' => {
                normalized.push('\r');
                normalized.push('\n');
            }
            '\u{2029}' => {
                normalized.push('\r');
                normalized.push('\n');
                normalized.push('\r');
                normalized.push('\n');
            }
            _ => normalized.push(ch),
        }
    }
    normalized
}

pub fn paragraph_selection_range_utf16(text: &[u16], char_index: usize) -> (usize, usize) {
    if text.is_empty() {
        return (0, 0);
    }

    let index = char_index.min(text.len().saturating_sub(1));
    let mut start = index;
    while start > 0 && !is_newline_utf16(text[start - 1]) {
        start -= 1;
    }
    while start < text.len() && is_newline_utf16(text[start]) {
        start += 1;
    }

    let mut end = index;
    while end < text.len() && !is_newline_utf16(text[end]) {
        end += 1;
    }

    if end < start {
        (start, start)
    } else {
        (start, end)
    }
}

fn is_newline_utf16(value: u16) -> bool {
    value == b'\r' as u16 || value == b'\n' as u16
}

pub fn is_third_click_after_double_click(
    last_double_click_time: Option<u32>,
    current_time: u32,
    double_click_time: u32,
) -> bool {
    last_double_click_time
        .map(|last| current_time.wrapping_sub(last) <= double_click_time)
        .unwrap_or(false)
}

#[cfg(windows)]
pub struct TranslationWindow {
    hwnd: windows::Win32::Foundation::HWND,
    source_label: windows::Win32::Foundation::HWND,
    source_edit: windows::Win32::Foundation::HWND,
    translated_label: windows::Win32::Foundation::HWND,
    translated_edit: windows::Win32::Foundation::HWND,
    status_text: windows::Win32::Foundation::HWND,
    translate_button: windows::Win32::Foundation::HWND,
    profile_combo: windows::Win32::Foundation::HWND,
    profile_options: Vec<TranslationProfileOption>,
    state: TranslationWindowState,
}

#[cfg(windows)]
impl TranslationWindow {
    pub fn new() -> Result<Self> {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::Graphics::Gdi::{COLOR_WINDOW, GetSysColorBrush};
        use windows::Win32::UI::WindowsAndMessaging::{
            CW_USEDEFAULT, CreateWindowExW, IDC_ARROW, LoadCursorW, RegisterClassW,
            WINDOW_EX_STYLE, WNDCLASSW,
        };
        use windows::core::PCWSTR;

        let class_name = wide("ait_translation_window");
        unsafe {
            let class = WNDCLASSW {
                lpfnWndProc: Some(default_wnd_proc),
                lpszClassName: PCWSTR(class_name.as_ptr()),
                hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
                hbrBackground: GetSysColorBrush(COLOR_WINDOW),
                ..Default::default()
            };
            let atom = RegisterClassW(&class);
            if atom == 0 {
                return Err(AppError::Windows("注册翻译窗口类失败".to_string()));
            }

            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                PCWSTR(class_name.as_ptr()),
                PCWSTR(wide("ait 翻译").as_ptr()),
                translation_window_style(),
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                620,
                420,
                Some(HWND::default()),
                None,
                None,
                None,
            )
            .map_err(|err| AppError::Windows(format!("创建翻译窗口失败: {err}")))?;

            let source_label = create_static(hwnd, "原文", 16, 14, 80, 20, ID_SOURCE_LABEL)?;
            let source_edit = create_edit(hwnd, 16, 38, 572, 96, ID_SOURCE_EDIT, false)?;
            let translated_label =
                create_static(hwnd, "译文", 16, 146, 80, 20, ID_TRANSLATED_LABEL)?;
            let translated_edit = create_edit(hwnd, 16, 170, 572, 140, ID_TRANSLATED_EDIT, true)?;
            let status_text = create_static(hwnd, "", 16, 324, 360, 22, ID_STATUS_TEXT)?;
            let translate_button =
                create_button(hwnd, "翻译", 534, 322, 52, 28, ID_TRANSLATE as isize)?;
            let profile_combo = create_combo(hwnd, 408, 12, 180, 220, ID_PROFILE_COMBO)?;
            install_edit_subclass(source_edit)?;
            install_edit_subclass(translated_edit)?;

            let this = Self {
                hwnd,
                source_label,
                source_edit,
                translated_label,
                translated_edit,
                status_text,
                translate_button,
                profile_combo,
                profile_options: Vec::new(),
                state: TranslationWindowState {
                    source_text: String::new(),
                    translated_text: String::new(),
                    loading: false,
                    error: None,
                },
            };
            this.apply_layout()?;

            Ok(this)
        }
    }

    pub fn show_loading(&mut self, source_text: String) -> Result<()> {
        self.state.source_text = source_text;
        self.state.translated_text.clear();
        self.state.loading = true;
        self.state.error = None;
        set_text(self.source_edit, &self.state.source_text)?;
        set_text(self.translated_edit, "")?;
        set_text(self.status_text, "正在翻译...")?;
        show_window_at_cursor(self.hwnd, ShowMode::Loading);
        tracing::info!("show translation window loading state");
        Ok(())
    }

    pub fn show_starting(&mut self) -> Result<()> {
        self.state.mark_starting();
        set_text(self.source_edit, &self.state.source_text)?;
        set_text(self.translated_edit, &self.state.translated_text)?;
        set_text(self.status_text, "正在取词...")?;
        show_window_at_cursor(self.hwnd, ShowMode::Starting);
        tracing::info!("show translation window starting state");
        Ok(())
    }

    pub fn show_result(&mut self, result: &crate::app::TranslationWorkflowResult) -> Result<()> {
        self.state.apply_translation_result(result);
        set_text(self.source_edit, &self.state.source_text)?;
        set_text(self.translated_edit, &self.state.translated_text)?;
        set_text(self.status_text, "翻译完成")?;
        show_window_at_cursor(self.hwnd, ShowMode::Result);
        tracing::info!("show translation window result");
        Ok(())
    }

    pub fn show_error(&mut self, message: String) -> Result<()> {
        self.state.loading = false;
        self.state.error = Some(message);
        let message = self.state.error.as_deref().unwrap_or("翻译失败");
        set_text(self.status_text, message)?;
        show_window_at_cursor(self.hwnd, ShowMode::Error);
        tracing::info!("show translation window error");
        Ok(())
    }

    pub fn begin_selection_translation(&mut self) -> Result<()> {
        self.show_starting()
    }

    pub fn begin_window_text_translation(&mut self, source_text: String) -> Result<()> {
        self.show_loading(source_text)
    }

    pub fn finish_translation_result(
        &mut self,
        result: crate::error::Result<crate::app::TranslationWorkflowResult>,
    ) -> Result<()> {
        match result {
            Ok(result) => self.show_result(&result),
            Err(err) => {
                tracing::warn!(error = %err, "show translation error summary");
                self.show_error(err.user_summary())
            }
        }
    }

    pub fn show_window(&mut self) -> Result<()> {
        show_window_at_cursor(self.hwnd, ShowMode::Result);
        tracing::info!("show translation window");
        Ok(())
    }

    pub fn is_foreground(&self) -> bool {
        is_foreground_window(self.hwnd)
    }

    pub fn is_visible(&self) -> bool {
        is_window_visible(self.hwnd)
    }

    pub fn source_text(&self) -> Result<String> {
        get_text(self.source_edit)
    }

    pub fn hwnd(&self) -> windows::Win32::Foundation::HWND {
        self.hwnd
    }

    pub fn refresh_profiles(
        &mut self,
        settings: &crate::config::AppSettings,
        active_profile_id: &str,
    ) -> Result<()> {
        self.profile_options = TranslationProfileOption::from_settings(settings, active_profile_id);
        reset_combo_items(self.profile_combo, &self.profile_options)?;
        Ok(())
    }

    pub fn selected_profile_id(&self) -> Option<String> {
        selected_combo_index(self.profile_combo).and_then(|index| {
            self.profile_options
                .get(index)
                .map(|option| option.id.clone())
        })
    }

    fn apply_layout(&self) -> Result<()> {
        use windows::Win32::Foundation::RECT;
        use windows::Win32::UI::WindowsAndMessaging::GetClientRect;

        unsafe {
            let mut rect = RECT::default();
            let _ = GetClientRect(self.hwnd, &mut rect);
            let layout = translation_window_layout(rect.right - rect.left, rect.bottom - rect.top);
            move_window(
                self.profile_combo,
                ControlRect {
                    height: translation_profile_combo_dropdown_height(),
                    ..layout.profile_combo
                },
            )?;
            move_window(self.source_label, layout.source_label)?;
            move_window(self.source_edit, layout.source_edit)?;
            move_window(self.translated_label, layout.translated_label)?;
            move_window(self.translated_edit, layout.translated_edit)?;
            move_window(self.status_text, layout.status_text)?;
            move_window(self.translate_button, layout.translate_button)?;
        }
        Ok(())
    }
}

#[cfg(windows)]
impl crate::app::TranslationObserver for TranslationWindow {
    fn translation_started(&mut self) -> Result<()> {
        self.show_starting()
    }

    fn source_captured(&mut self, source_text: &str) -> Result<()> {
        self.show_loading(source_text.to_string())
    }

    fn translation_succeeded(
        &mut self,
        result: &crate::app::TranslationWorkflowResult,
    ) -> Result<()> {
        self.show_result(result)
    }
}

#[cfg(windows)]
unsafe extern "system" fn default_wnd_proc(
    hwnd: windows::Win32::Foundation::HWND,
    msg: u32,
    wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::LRESULT {
    use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
    use windows::Win32::Graphics::Gdi::InvalidateRect;
    use windows::Win32::UI::WindowsAndMessaging::{
        DefWindowProcW, PostMessageW, SW_HIDE, ShowWindow, WM_CLOSE, WM_COMMAND, WM_GETMINMAXINFO,
        WM_KEYDOWN, WM_SIZE,
    };

    if msg == WM_CLOSE {
        unsafe {
            let _ = ShowWindow(hwnd, SW_HIDE);
        }
        return LRESULT(0);
    }
    if msg == WM_KEYDOWN
        && edit_shortcut_action(wparam.0 as u32, false) == EditShortcutAction::HideWindow
    {
        unsafe {
            let _ = ShowWindow(hwnd, SW_HIDE);
        }
        return LRESULT(0);
    }
    if msg == WM_COMMAND {
        let command = wparam.0 & 0xffff;
        let notification = (wparam.0 >> 16) & 0xffff;
        match command {
            ID_TRANSLATE => unsafe {
                let _ = PostMessageW(Some(hwnd), WM_TRANSLATE_WINDOW_SOURCE, WPARAM(0), LPARAM(0));
                return LRESULT(0);
            },
            command
                if command == ID_PROFILE_COMBO as usize
                    && notification
                        == windows::Win32::UI::WindowsAndMessaging::CBN_SELCHANGE as usize =>
            {
                unsafe {
                    let _ = PostMessageW(
                        Some(hwnd),
                        WM_TRANSLATE_WINDOW_PROFILE_CHANGED,
                        WPARAM(0),
                        LPARAM(0),
                    );
                }
                return LRESULT(0);
            }
            _ => {}
        }
    }
    if msg == WM_SIZE {
        let _ = resize_translation_window(hwnd);
        unsafe {
            let _ = InvalidateRect(Some(hwnd), None, true);
        }
    }
    if msg == WM_GETMINMAXINFO {
        unsafe {
            apply_min_track_size(lparam);
        }
        return LRESULT(0);
    }

    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

#[cfg(windows)]
unsafe extern "system" fn edit_subclass_proc(
    hwnd: windows::Win32::Foundation::HWND,
    msg: u32,
    wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
    _id_subclass: usize,
    ref_data: usize,
) -> windows::Win32::Foundation::LRESULT {
    use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
    use windows::Win32::UI::Controls::EM_SETSEL;
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        GetDoubleClickTime, GetKeyState, VK_CONTROL,
    };
    use windows::Win32::UI::Shell::{DefSubclassProc, RemoveWindowSubclass};
    use windows::Win32::UI::WindowsAndMessaging::{
        GetMessageTime, GetParent, PostMessageW, SendMessageW, WM_CHAR, WM_CLOSE, WM_KEYDOWN,
        WM_LBUTTONDBLCLK, WM_LBUTTONDOWN, WM_NCDESTROY,
    };

    let state_ptr = ref_data as *mut EditSubclassState;

    if msg == WM_KEYDOWN {
        let ctrl_down = unsafe { GetKeyState(VK_CONTROL.0 as i32) } < 0;
        match edit_shortcut_action(wparam.0 as u32, ctrl_down) {
            EditShortcutAction::SelectAll => {
                unsafe {
                    let _ = SendMessageW(hwnd, EM_SETSEL, Some(WPARAM(0)), Some(LPARAM(-1)));
                }
                return LRESULT(0);
            }
            EditShortcutAction::HideWindow => {
                unsafe {
                    if let Ok(parent) = GetParent(hwnd) {
                        let _ = PostMessageW(Some(parent), WM_CLOSE, WPARAM(0), LPARAM(0));
                    }
                }
                return LRESULT(0);
            }
            EditShortcutAction::None => {}
        }
    }
    if msg == WM_CHAR && edit_char_action(wparam.0 as u32) == EditCharAction::Swallow {
        return LRESULT(0);
    }
    if msg == WM_LBUTTONDBLCLK && !state_ptr.is_null() {
        let state = unsafe { &mut *state_ptr };
        state.last_double_click_time = Some(unsafe { GetMessageTime() } as u32);
    }
    if msg == WM_LBUTTONDOWN && !state_ptr.is_null() {
        let state = unsafe { &mut *state_ptr };
        let current_time = unsafe { GetMessageTime() } as u32;
        if is_third_click_after_double_click(state.last_double_click_time, current_time, unsafe {
            GetDoubleClickTime()
        }) {
            state.last_double_click_time = None;
            unsafe {
                select_paragraph_at_point(hwnd, lparam);
            }
            return LRESULT(0);
        }
    }
    if msg == WM_NCDESTROY && ref_data != 0 {
        unsafe {
            let _ = RemoveWindowSubclass(hwnd, Some(edit_subclass_proc), EDIT_SUBCLASS_ID);
            drop(Box::from_raw(ref_data as *mut EditSubclassState));
        }
    }

    unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) }
}

#[cfg(windows)]
#[derive(Default)]
struct EditSubclassState {
    last_double_click_time: Option<u32>,
}

#[cfg(windows)]
const EDIT_SUBCLASS_ID: usize = 1;

#[cfg(windows)]
unsafe fn select_paragraph_at_point(
    hwnd: windows::Win32::Foundation::HWND,
    point: windows::Win32::Foundation::LPARAM,
) {
    use windows::Win32::Foundation::{LPARAM, WPARAM};
    use windows::Win32::UI::Controls::{EM_CHARFROMPOS, EM_SETSEL};
    use windows::Win32::UI::WindowsAndMessaging::SendMessageW;

    let char_from_pos = unsafe { SendMessageW(hwnd, EM_CHARFROMPOS, None, Some(point)) }.0 as usize;
    let char_index = char_from_pos & 0xffff;
    let text = unsafe { get_text_utf16(hwnd) };
    let (start, end) = paragraph_selection_range_utf16(&text, char_index);
    let _ = unsafe {
        SendMessageW(
            hwnd,
            EM_SETSEL,
            Some(WPARAM(start)),
            Some(LPARAM(end as isize)),
        )
    };
}

#[cfg(windows)]
fn install_edit_subclass(hwnd: windows::Win32::Foundation::HWND) -> Result<()> {
    use windows::Win32::UI::Shell::SetWindowSubclass;

    let state = Box::into_raw(Box::new(EditSubclassState::default())) as usize;
    unsafe {
        if SetWindowSubclass(hwnd, Some(edit_subclass_proc), EDIT_SUBCLASS_ID, state).as_bool() {
            Ok(())
        } else {
            drop(Box::from_raw(state as *mut EditSubclassState));
            Err(AppError::Windows("安装编辑框快捷键处理失败".to_string()))
        }
    }
}

#[cfg(windows)]
fn create_static(
    parent: windows::Win32::Foundation::HWND,
    text: &str,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    id: isize,
) -> Result<windows::Win32::Foundation::HWND> {
    create_control(
        parent,
        "STATIC",
        text,
        x,
        y,
        width,
        height,
        id,
        Default::default(),
    )
}

#[cfg(windows)]
fn create_button(
    parent: windows::Win32::Foundation::HWND,
    text: &str,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    id: isize,
) -> Result<windows::Win32::Foundation::HWND> {
    use windows::Win32::UI::WindowsAndMessaging::{BS_PUSHBUTTON, WINDOW_STYLE};
    create_control(
        parent,
        "BUTTON",
        text,
        x,
        y,
        width,
        height,
        id,
        WINDOW_STYLE(BS_PUSHBUTTON as u32),
    )
}

#[cfg(windows)]
fn create_edit(
    parent: windows::Win32::Foundation::HWND,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    id: isize,
    readonly: bool,
) -> Result<windows::Win32::Foundation::HWND> {
    use windows::Win32::UI::WindowsAndMessaging::{
        ES_AUTOVSCROLL, ES_LEFT, ES_MULTILINE, ES_READONLY, ES_WANTRETURN, WINDOW_STYLE, WS_VSCROLL,
    };
    let mut style_bits =
        (ES_LEFT | ES_MULTILINE | ES_AUTOVSCROLL | ES_WANTRETURN) as u32 | WS_VSCROLL.0;
    if readonly {
        style_bits |= ES_READONLY as u32;
    }
    let style = WINDOW_STYLE(style_bits);
    create_control(parent, "EDIT", "", x, y, width, height, id, style)
}

#[cfg(windows)]
fn create_combo(
    parent: windows::Win32::Foundation::HWND,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    id: isize,
) -> Result<windows::Win32::Foundation::HWND> {
    use windows::Win32::UI::WindowsAndMessaging::{CBS_DROPDOWNLIST, WINDOW_STYLE, WS_VSCROLL};
    create_control(
        parent,
        "COMBOBOX",
        "",
        x,
        y,
        width,
        height,
        id,
        WINDOW_STYLE(CBS_DROPDOWNLIST as u32 | WS_VSCROLL.0),
    )
}

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

#[cfg(windows)]
fn reset_combo_items(
    hwnd: windows::Win32::Foundation::HWND,
    options: &[TranslationProfileOption],
) -> Result<()> {
    use windows::Win32::Foundation::{LPARAM, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        CB_ADDSTRING, CB_RESETCONTENT, CB_SETCURSEL, SendMessageW,
    };

    unsafe {
        let _ = SendMessageW(hwnd, CB_RESETCONTENT, Some(WPARAM(0)), Some(LPARAM(0)));
        let mut active_index = 0usize;
        for (index, option) in options.iter().enumerate() {
            if option.active {
                active_index = index;
            }
            let label = wide(&option.label);
            let _ = SendMessageW(
                hwnd,
                CB_ADDSTRING,
                Some(WPARAM(0)),
                Some(LPARAM(label.as_ptr() as isize)),
            );
        }
        if !options.is_empty() {
            let _ = SendMessageW(
                hwnd,
                CB_SETCURSEL,
                Some(WPARAM(active_index)),
                Some(LPARAM(0)),
            );
        }
    }
    Ok(())
}

#[cfg(windows)]
fn selected_combo_index(hwnd: windows::Win32::Foundation::HWND) -> Option<usize> {
    use windows::Win32::Foundation::{LPARAM, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{CB_ERR, CB_GETCURSEL, SendMessageW};

    let index = unsafe { SendMessageW(hwnd, CB_GETCURSEL, Some(WPARAM(0)), Some(LPARAM(0))).0 };
    if index == CB_ERR as isize {
        None
    } else {
        Some(index as usize)
    }
}

#[cfg(windows)]
fn move_window(hwnd: windows::Win32::Foundation::HWND, rect: ControlRect) -> Result<()> {
    use windows::Win32::UI::WindowsAndMessaging::MoveWindow;

    unsafe {
        MoveWindow(hwnd, rect.x, rect.y, rect.width, rect.height, true)
            .map_err(|err| AppError::Windows(format!("调整控件位置失败: {err}")))
    }
}

#[cfg(windows)]
fn translation_window_style() -> windows::Win32::UI::WindowsAndMessaging::WINDOW_STYLE {
    use windows::Win32::UI::WindowsAndMessaging::{
        WS_CAPTION, WS_CLIPCHILDREN, WS_OVERLAPPED, WS_SYSMENU, WS_THICKFRAME,
    };

    WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_THICKFRAME | WS_CLIPCHILDREN
}

#[cfg(windows)]
unsafe fn apply_min_track_size(lparam: windows::Win32::Foundation::LPARAM) {
    use windows::Win32::Foundation::RECT;
    use windows::Win32::UI::WindowsAndMessaging::{
        AdjustWindowRectEx, MINMAXINFO, WINDOW_EX_STYLE,
    };

    let info = lparam.0 as *mut MINMAXINFO;
    if info.is_null() {
        return;
    }

    let (min_width, min_height) = translation_window_min_client_size();
    let mut rect = RECT {
        left: 0,
        top: 0,
        right: min_width,
        bottom: min_height,
    };
    if unsafe {
        AdjustWindowRectEx(
            &mut rect,
            translation_window_style(),
            false,
            WINDOW_EX_STYLE::default(),
        )
        .is_ok()
    } {
        unsafe {
            (*info).ptMinTrackSize.x = rect.right - rect.left;
            (*info).ptMinTrackSize.y = rect.bottom - rect.top;
        }
    }
}

#[cfg(windows)]
fn resize_translation_window(hwnd: windows::Win32::Foundation::HWND) -> Result<()> {
    use windows::Win32::Foundation::RECT;
    use windows::Win32::UI::WindowsAndMessaging::GetClientRect;

    unsafe {
        let mut rect = RECT::default();
        GetClientRect(hwnd, &mut rect)
            .map_err(|err| AppError::Windows(format!("获取翻译窗口尺寸失败: {err}")))?;
        let layout = translation_window_layout(rect.right - rect.left, rect.bottom - rect.top);
        let source_label =
            windows::Win32::UI::WindowsAndMessaging::GetDlgItem(Some(hwnd), ID_SOURCE_LABEL as i32)
                .map_err(|err| AppError::Windows(format!("获取原文标签失败: {err}")))?;
        let profile_combo = windows::Win32::UI::WindowsAndMessaging::GetDlgItem(
            Some(hwnd),
            ID_PROFILE_COMBO as i32,
        )
        .map_err(|err| AppError::Windows(format!("获取配置下拉框失败: {err}")))?;
        let source_edit =
            windows::Win32::UI::WindowsAndMessaging::GetDlgItem(Some(hwnd), ID_SOURCE_EDIT as i32)
                .map_err(|err| AppError::Windows(format!("获取原文输入框失败: {err}")))?;
        let translated_label = windows::Win32::UI::WindowsAndMessaging::GetDlgItem(
            Some(hwnd),
            ID_TRANSLATED_LABEL as i32,
        )
        .map_err(|err| AppError::Windows(format!("获取译文标签失败: {err}")))?;
        let translated_edit = windows::Win32::UI::WindowsAndMessaging::GetDlgItem(
            Some(hwnd),
            ID_TRANSLATED_EDIT as i32,
        )
        .map_err(|err| AppError::Windows(format!("获取译文输入框失败: {err}")))?;
        let status_text =
            windows::Win32::UI::WindowsAndMessaging::GetDlgItem(Some(hwnd), ID_STATUS_TEXT as i32)
                .map_err(|err| AppError::Windows(format!("获取状态文本失败: {err}")))?;
        let translate_button =
            windows::Win32::UI::WindowsAndMessaging::GetDlgItem(Some(hwnd), ID_TRANSLATE as i32)
                .map_err(|err| AppError::Windows(format!("获取翻译按钮失败: {err}")))?;
        move_window(
            profile_combo,
            ControlRect {
                height: translation_profile_combo_dropdown_height(),
                ..layout.profile_combo
            },
        )?;
        move_window(source_label, layout.source_label)?;
        move_window(source_edit, layout.source_edit)?;
        move_window(translated_label, layout.translated_label)?;
        move_window(translated_edit, layout.translated_edit)?;
        move_window(status_text, layout.status_text)?;
        move_window(translate_button, layout.translate_button)?;
        Ok(())
    }
}

#[cfg(windows)]
fn set_text(hwnd: windows::Win32::Foundation::HWND, text: &str) -> Result<()> {
    use windows::Win32::UI::WindowsAndMessaging::SetWindowTextW;
    use windows::core::PCWSTR;

    let text = edit_display_text(text);
    unsafe {
        SetWindowTextW(hwnd, PCWSTR(wide(&text).as_ptr()))
            .map_err(|err| AppError::Windows(format!("设置窗口文本失败: {err}")))
    }
}

#[cfg(windows)]
fn get_text(hwnd: windows::Win32::Foundation::HWND) -> Result<String> {
    use windows::Win32::UI::WindowsAndMessaging::{GetWindowTextLengthW, GetWindowTextW};

    unsafe {
        let len = GetWindowTextLengthW(hwnd);
        if len == 0 {
            return Ok(String::new());
        }
        let mut buf = vec![0u16; len as usize + 1];
        let copied = GetWindowTextW(hwnd, &mut buf);
        Ok(String::from_utf16_lossy(&buf[..copied as usize]))
    }
}

#[cfg(windows)]
unsafe fn get_text_utf16(hwnd: windows::Win32::Foundation::HWND) -> Vec<u16> {
    use windows::Win32::UI::WindowsAndMessaging::{GetWindowTextLengthW, GetWindowTextW};

    let len = unsafe { GetWindowTextLengthW(hwnd) };
    if len == 0 {
        return Vec::new();
    }
    let mut buf = vec![0u16; len as usize + 1];
    let copied = unsafe { GetWindowTextW(hwnd, &mut buf) };
    buf.truncate(copied as usize);
    buf
}

#[cfg(windows)]
fn show_window_at_cursor(hwnd: windows::Win32::Foundation::HWND, mode: ShowMode) {
    use windows::Win32::Foundation::{POINT, RECT};
    use windows::Win32::Graphics::Gdi::{
        GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromPoint,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        GetCursorPos, GetWindowRect, HWND_NOTOPMOST, HWND_TOPMOST, SET_WINDOW_POS_FLAGS, SW_SHOW,
        SW_SHOWNOACTIVATE, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW,
        SetForegroundWindow, SetWindowPos, ShowWindow,
    };

    unsafe {
        let action = show_action(is_window_visible(hwnd), is_foreground_window(hwnd));
        match action {
            ShowAction::PositionAndActivate => {}
            ShowAction::ActivateOnly => {
                if show_window_needs_topmost_reset(mode, action) {
                    let _ = SetWindowPos(
                        hwnd,
                        Some(HWND_NOTOPMOST),
                        0,
                        0,
                        0,
                        0,
                        SET_WINDOW_POS_FLAGS(
                            SWP_NOMOVE.0 | SWP_NOSIZE.0 | SWP_NOACTIVATE.0 | SWP_SHOWWINDOW.0,
                        ),
                    );
                }
                if show_window_needs_topmost_raise(mode, action) {
                    let _ = SetWindowPos(
                        hwnd,
                        Some(HWND_TOPMOST),
                        0,
                        0,
                        0,
                        0,
                        SET_WINDOW_POS_FLAGS(
                            SWP_NOMOVE.0 | SWP_NOSIZE.0 | SWP_NOACTIVATE.0 | SWP_SHOWWINDOW.0,
                        ),
                    );
                }
                if mode.activates_window() {
                    let _ = ShowWindow(hwnd, SW_SHOW);
                    let _ = SetForegroundWindow(hwnd);
                }
                return;
            }
            ShowAction::KeepPosition => {
                if show_window_needs_topmost_reset(mode, action) {
                    let _ = SetWindowPos(
                        hwnd,
                        Some(HWND_NOTOPMOST),
                        0,
                        0,
                        0,
                        0,
                        SET_WINDOW_POS_FLAGS(
                            SWP_NOMOVE.0 | SWP_NOSIZE.0 | SWP_NOACTIVATE.0 | SWP_SHOWWINDOW.0,
                        ),
                    );
                }
                let _ = ShowWindow(hwnd, SW_SHOW);
                return;
            }
        }

        let mut cursor = POINT::default();
        let _ = GetCursorPos(&mut cursor);
        let mut rect = RECT::default();
        let _ = GetWindowRect(hwnd, &mut rect);
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;
        let monitor = MonitorFromPoint(cursor, MONITOR_DEFAULTTONEAREST);
        let mut monitor_info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        let _ = GetMonitorInfoW(monitor, &mut monitor_info);
        let work = monitor_info.rcWork;
        let x = (cursor.x + 12).clamp(work.left, work.right - width);
        let y = (cursor.y + 12).clamp(work.top, work.bottom - height);
        let mut flags = SWP_SHOWWINDOW.0;
        if !mode.activates_window() {
            flags |= SWP_NOACTIVATE.0;
        }
        let z_order = match show_window_z_order(mode) {
            WindowZOrder::NotTopmost => HWND_NOTOPMOST,
            WindowZOrder::TopmostNoActivate => HWND_TOPMOST,
        };
        let _ = SetWindowPos(
            hwnd,
            Some(z_order),
            x,
            y,
            width,
            height,
            SET_WINDOW_POS_FLAGS(flags),
        );
        if mode.activates_window() {
            let _ = ShowWindow(hwnd, SW_SHOW);
            let _ = SetForegroundWindow(hwnd);
        } else {
            let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
        }
    }
}

#[cfg(windows)]
fn is_foreground_window(hwnd: windows::Win32::Foundation::HWND) -> bool {
    use windows::Win32::UI::WindowsAndMessaging::{GA_ROOT, GetAncestor, GetForegroundWindow};

    unsafe {
        let foreground = GetForegroundWindow();
        foreground == hwnd || GetAncestor(foreground, GA_ROOT) == GetAncestor(hwnd, GA_ROOT)
    }
}

#[cfg(windows)]
fn is_window_visible(hwnd: windows::Win32::Foundation::HWND) -> bool {
    use windows::Win32::UI::WindowsAndMessaging::IsWindowVisible;

    unsafe { IsWindowVisible(hwnd).as_bool() }
}

#[cfg(windows)]
fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(Some(0)).collect()
}
