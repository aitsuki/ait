use crate::error::{AppError, Result};

#[cfg(windows)]
const ID_SOURCE_EDIT: isize = 2101;
#[cfg(windows)]
const ID_TRANSLATED_EDIT: isize = 2102;
#[cfg(windows)]
const ID_STATUS_TEXT: isize = 2105;
#[cfg(windows)]
const ID_SPLITTER: isize = 2107;
#[cfg(windows)]
const LAYOUT_MODE_SLOT: windows::Win32::UI::WindowsAndMessaging::WINDOW_LONG_PTR_INDEX =
    windows::Win32::UI::WindowsAndMessaging::WINDOW_LONG_PTR_INDEX(0);
#[cfg(windows)]
const SOURCE_RATIO_SLOT: windows::Win32::UI::WindowsAndMessaging::WINDOW_LONG_PTR_INDEX =
    windows::Win32::UI::WindowsAndMessaging::WINDOW_LONG_PTR_INDEX(
        std::mem::size_of::<isize>() as i32
    );
#[cfg(windows)]
pub(crate) const ID_PROFILE_COMBO: isize = 2106;
#[cfg(windows)]
const ID_TRANSLATE: usize = 2001;
#[cfg(windows)]
const ID_UPDATE_BUTTON: usize = 2002;
#[cfg(windows)]
pub const WM_TRANSLATE_WINDOW_SOURCE: u32 = windows::Win32::UI::WindowsAndMessaging::WM_APP + 30;
#[cfg(windows)]
pub const WM_TRANSLATE_WINDOW_PROFILE_CHANGED: u32 =
    windows::Win32::UI::WindowsAndMessaging::WM_APP + 31;
#[cfg(windows)]
pub const WM_TRANSLATE_WINDOW_UPDATE_CLICKED: u32 =
    windows::Win32::UI::WindowsAndMessaging::WM_APP + 32;
#[cfg(windows)]
pub const WM_TRANSLATE_WINDOW_RATIO_CHANGED: u32 =
    windows::Win32::UI::WindowsAndMessaging::WM_APP + 33;
#[cfg(windows)]
const WM_TRANSLATION_TASK_FINISHED: u32 = windows::Win32::UI::WindowsAndMessaging::WM_APP + 60;
#[cfg(windows)]
const WM_TRANSLATION_SOURCE_CAPTURED: u32 = windows::Win32::UI::WindowsAndMessaging::WM_APP + 61;

#[cfg(windows)]
pub(crate) struct TranslationTaskMessage {
    pub request_id: usize,
    pub profile_id: String,
    pub result: crate::error::Result<crate::app::TranslationWorkflowResult>,
}

#[cfg(windows)]
pub(crate) struct TranslationSourceMessage {
    pub request_id: usize,
    pub source_text: String,
}

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
}

pub fn translation_message_is_current(message_request_id: usize, active_request_id: usize) -> bool {
    message_request_id != 0 && message_request_id == active_request_id
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
    (420, 276)
}

pub fn translation_profile_combo_dropdown_height() -> i32 {
    220
}

pub fn translation_static_text_uses_window_background(control_id: usize) -> bool {
    control_id == 2105
}

pub fn translation_window_update_button_visible(
    status: Option<&crate::update::UpdateStatus>,
) -> bool {
    matches!(
        status,
        Some(crate::update::UpdateStatus::UpdateAvailable { .. })
    )
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
    pub update_button: ControlRect,
    pub source_edit: ControlRect,
    pub splitter: ControlRect,
    pub translated_edit: ControlRect,
    pub status_text: ControlRect,
    pub translate_button: ControlRect,
}

#[cfg(windows)]
fn translation_window_layout_for_system_dpi(
    client_width: i32,
    client_height: i32,
    mode: crate::config::TranslationWindowLayout,
    source_ratio: u32,
) -> TranslationWindowLayout {
    translation_window_layout_for_dpi(
        client_width,
        client_height,
        crate::ui::theme::system_dpi(),
        mode,
        source_ratio,
    )
}

fn translation_window_layout_for_dpi(
    client_width: i32,
    client_height: i32,
    dpi: u32,
    mode: crate::config::TranslationWindowLayout,
    source_ratio: u32,
) -> TranslationWindowLayout {
    let scale = |value| crate::ui::theme::scale_for_dpi(value, dpi);
    let margin = scale(16);
    let gap = scale(10);
    let status_height_value = scale(22);
    let button_width_value = scale(crate::ui::theme::BUTTON_MIN_WIDTH);
    let button_height_value = scale(crate::ui::theme::PRIMARY_BUTTON_HEIGHT);
    let update_button_width_value = scale(86);
    let source_line_height = 20;
    let source_edit_height = scale(
        3 * source_line_height
            + 2 * crate::ui::edit::MULTILINE_EDIT_VERTICAL_PADDING
            + 2 * crate::ui::edit::EDIT_FRAME_GUTTER,
    );

    let usable_width = client_width.max(1);
    let usable_height = client_height.max(1);
    let content_x = margin.min(usable_width - 1);
    let content_width = (usable_width - content_x - margin).max(1);
    let combo_width = scale(180).min(content_width);
    let combo_height = scale(crate::ui::theme::CONTROL_HEIGHT).min(usable_height);
    let button_width = button_width_value.min(content_width);
    let button_height = button_height_value.min(usable_height);
    let button_x = (usable_width - margin - button_width).clamp(0, usable_width - button_width);
    let bottom_y = (usable_height - margin - button_height).clamp(0, usable_height - button_height);
    let status_width = (button_x - margin - gap).max(1);
    let status_height = status_height_value.min(usable_height);
    let profile_combo = ControlRect {
        x: (usable_width - margin - combo_width).clamp(0, usable_width - combo_width),
        y: scale(12).min(usable_height - combo_height),
        width: combo_width,
        height: combo_height,
    };
    let update_button_width = update_button_width_value
        .min((profile_combo.x - content_x - gap).max(1))
        .max(1);
    let update_button_x = (profile_combo.x - gap - update_button_width)
        .max(content_x)
        .min(usable_width - update_button_width);
    let update_button = ControlRect {
        x: update_button_x,
        y: profile_combo.y,
        width: update_button_width,
        height: combo_height,
    };
    let top_row_bottom = profile_combo
        .y
        .max(update_button.y)
        .saturating_add(profile_combo.height.max(update_button.height));
    let source_edit_y = (top_row_bottom + scale(6)).min(usable_height - 1);
    let source_edit_height = if mode != crate::config::TranslationWindowLayout::FixedSource {
        let available = (bottom_y - source_edit_y - 2 * gap).max(2);
        let minimum = scale(48).min(available / 2).max(1);
        let ratio = source_ratio.clamp(100, crate::config::PANE_RATIO_SCALE - 100);
        let height = (i64::from(available) * i64::from(ratio)
            / i64::from(crate::config::PANE_RATIO_SCALE)) as i32;
        height.clamp(minimum, available - minimum)
    } else {
        source_edit_height
    };
    let source_edit_height = source_edit_height.min(usable_height - source_edit_y);
    let translated_edit_y = (source_edit_y + source_edit_height + gap).min(usable_height - 1);
    let translated_edit_height = (bottom_y - translated_edit_y - gap)
        .max(1)
        .min(usable_height - translated_edit_y);

    TranslationWindowLayout {
        profile_combo,
        update_button,
        source_edit: ControlRect {
            x: content_x,
            y: source_edit_y,
            width: content_width,
            height: source_edit_height,
        },
        splitter: ControlRect {
            x: content_x,
            y: source_edit_y + source_edit_height,
            width: content_width,
            height: gap,
        },
        translated_edit: ControlRect {
            x: content_x,
            y: translated_edit_y,
            width: content_width,
            height: translated_edit_height,
        },
        status_text: ControlRect {
            x: content_x,
            y: (bottom_y + scale(3)).min(usable_height - status_height),
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
    Translate,
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
    const VK_RETURN: u32 = 0x0D;

    if ctrl_down && vk == VK_A {
        EditShortcutAction::SelectAll
    } else if ctrl_down && vk == VK_RETURN {
        EditShortcutAction::Translate
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
    source_edit: windows::Win32::Foundation::HWND,
    translated_edit: windows::Win32::Foundation::HWND,
    status_text: windows::Win32::Foundation::HWND,
    #[cfg(test)]
    translate_button: windows::Win32::Foundation::HWND,
    profile_combo: windows::Win32::Foundation::HWND,
    update_button: windows::Win32::Foundation::HWND,
    profile_options: Vec<TranslationProfileOption>,
    state: TranslationWindowState,
    update_status: Option<crate::update::UpdateStatus>,
}

#[cfg(windows)]
impl TranslationWindow {
    pub fn new() -> Result<Self> {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::Graphics::Gdi::{COLOR_WINDOW, GetSysColorBrush};
        use windows::Win32::UI::WindowsAndMessaging::{
            CW_USEDEFAULT, CreateWindowExW, GetSystemMetrics, IDC_ARROW, LoadCursorW,
            RegisterClassW, SM_CXICON, SM_CYICON, WINDOW_EX_STYLE, WNDCLASSW,
        };
        use windows::core::PCWSTR;

        let class_name = wide("ait_translation_window");
        unsafe {
            let class = WNDCLASSW {
                lpfnWndProc: Some(default_wnd_proc),
                cbWndExtra: (2 * std::mem::size_of::<isize>()) as i32,
                lpszClassName: PCWSTR(class_name.as_ptr()),
                hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
                hIcon: crate::ui::icon::load_app_icon(
                    GetSystemMetrics(SM_CXICON),
                    GetSystemMetrics(SM_CYICON),
                ),
                hbrBackground: GetSysColorBrush(COLOR_WINDOW),
                ..Default::default()
            };
            let atom = RegisterClassW(&class);
            if atom == 0 {
                return Err(AppError::Windows("注册翻译窗口类失败".to_string()));
            }

            let (width, height) = translation_window_outer_size(420, 336)?;
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                PCWSTR(class_name.as_ptr()),
                PCWSTR(wide("ait 翻译").as_ptr()),
                translation_window_style(),
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                width,
                height,
                Some(HWND::default()),
                None,
                None,
                None,
            )
            .map_err(|err| AppError::Windows(format!("创建翻译窗口失败: {err}")))?;

            let source_edit = create_edit(hwnd, 16, 38, 572, 96, ID_SOURCE_EDIT, false)?;
            let translated_edit = create_edit(hwnd, 16, 170, 572, 140, ID_TRANSLATED_EDIT, true)?;
            let status_text = create_static(hwnd, "", 16, 324, 360, 22, ID_STATUS_TEXT)?;
            let _translate_button =
                create_button(hwnd, "翻译", 534, 322, 52, 28, ID_TRANSLATE as isize)?;
            let profile_combo = create_combo(hwnd, 408, 12, 180, 220, ID_PROFILE_COMBO)?;
            let update_button =
                create_button(hwnd, "有新版本", 314, 12, 86, 26, ID_UPDATE_BUTTON as isize)?;
            hide_window(update_button);
            crate::ui::translation_splitter::create(hwnd, ID_SPLITTER)?;
            apply_multiline_edit_padding(source_edit)?;
            apply_multiline_edit_padding(translated_edit)?;
            install_edit_subclass(source_edit)?;
            install_edit_subclass(translated_edit)?;

            let this = Self {
                hwnd,
                source_edit,
                translated_edit,
                status_text,
                #[cfg(test)]
                translate_button: _translate_button,
                profile_combo,
                update_button,
                profile_options: Vec::new(),
                state: TranslationWindowState {
                    source_text: String::new(),
                    translated_text: String::new(),
                    loading: false,
                    error: None,
                },
                update_status: None,
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
        self.state.source_text = get_text(self.source_edit)?;
        self.state.translated_text = get_text(self.translated_edit)?;
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
        set_text(self.status_text, result.provider)?;
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

    pub fn begin_selection_translation(&mut self, request_id: usize) -> Result<()> {
        set_active_request_id(self.hwnd, request_id);
        self.show_starting()
    }

    pub fn begin_window_text_translation(
        &mut self,
        request_id: usize,
        source_text: String,
    ) -> Result<()> {
        set_active_request_id(self.hwnd, request_id);
        self.show_loading(source_text)
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

    pub fn show_update_available(&mut self, status: crate::update::UpdateStatus) -> Result<()> {
        self.update_status = Some(status);
        set_window_visible(
            self.update_button,
            translation_window_update_button_visible(self.update_status.as_ref()),
        );
        Ok(())
    }

    pub fn update_status(&self) -> Option<&crate::update::UpdateStatus> {
        self.update_status.as_ref()
    }

    pub fn hwnd(&self) -> windows::Win32::Foundation::HWND {
        self.hwnd
    }

    pub fn handle_dialog_message(
        &self,
        msg: &mut windows::Win32::UI::WindowsAndMessaging::MSG,
    ) -> bool {
        use windows::Win32::Foundation::{LPARAM, WPARAM};
        use windows::Win32::UI::Input::KeyboardAndMouse::VK_ESCAPE;
        use windows::Win32::UI::WindowsAndMessaging::{
            IsChild, IsDialogMessageW, PostMessageW, WM_CLOSE, WM_KEYDOWN,
        };

        let belongs_to_window =
            msg.hwnd == self.hwnd || unsafe { IsChild(self.hwnd, msg.hwnd).as_bool() };
        if belongs_to_window && msg.message == WM_KEYDOWN && msg.wParam.0 == VK_ESCAPE.0 as usize {
            unsafe {
                let _ = PostMessageW(Some(self.hwnd), WM_CLOSE, WPARAM(0), LPARAM(0));
            }
            return true;
        }
        unsafe { IsDialogMessageW(self.hwnd, msg).as_bool() }
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

    pub fn source_ratio(&self) -> u32 {
        window_source_ratio(self.hwnd)
    }

    pub fn set_layout(&self, settings: &crate::config::WindowSettings) -> Result<()> {
        // Extra window bytes keep the layout available to WM_SIZE without using
        // GWLP_USERDATA, which stores the active translation request ID.
        unsafe {
            windows::Win32::UI::WindowsAndMessaging::SetWindowLongPtrW(
                self.hwnd,
                LAYOUT_MODE_SLOT,
                settings.layout.index() as isize,
            );
        }
        set_window_source_ratio(self.hwnd, settings.source_ratio)
    }

    fn apply_layout(&self) -> Result<()> {
        resize_translation_window(self.hwnd)
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
        DefWindowProcW, PostMessageW, SW_HIDE, ShowWindow, WM_CLOSE, WM_COMMAND, WM_CTLCOLOREDIT,
        WM_CTLCOLORSTATIC, WM_DRAWITEM, WM_GETMINMAXINFO, WM_KEYDOWN, WM_MEASUREITEM, WM_PAINT,
        WM_SIZE,
    };

    if msg == WM_TRANSLATION_SOURCE_CAPTURED {
        if lparam.0 == 0 {
            tracing::warn!("translation source message had no payload");
            return LRESULT(0);
        }
        let message = unsafe { Box::from_raw(lparam.0 as *mut TranslationSourceMessage) };
        if translation_message_is_current(message.request_id, active_request_id(hwnd)) {
            if let Err(err) = show_loading_for_hwnd(hwnd, message.source_text) {
                tracing::warn!(error = %err, request_id = message.request_id, "show translation source failed");
            }
        } else {
            tracing::debug!(
                request_id = message.request_id,
                active_request_id = active_request_id(hwnd),
                "ignore stale translation source"
            );
        }
        return LRESULT(0);
    }
    if msg == WM_TRANSLATION_TASK_FINISHED {
        if lparam.0 == 0 {
            tracing::warn!("translation result message had no payload");
            return LRESULT(0);
        }
        let message = unsafe { Box::from_raw(lparam.0 as *mut TranslationTaskMessage) };
        if !translation_message_is_current(message.request_id, active_request_id(hwnd)) {
            tracing::info!(
                request_id = message.request_id,
                active_request_id = active_request_id(hwnd),
                "ignore stale translation result"
            );
            return LRESULT(0);
        }
        set_active_request_id(hwnd, 0);
        match message.result {
            Ok(result) => {
                tracing::info!(
                    request_id = message.request_id,
                    provider = result.provider,
                    profile_id = message.profile_id,
                    source_len = result.source_text.chars().count(),
                    translated_len = result.translated_text.chars().count(),
                    "translation completed"
                );
                if let Err(err) = show_result_for_hwnd(hwnd, &result) {
                    tracing::warn!(error = %err, request_id = message.request_id, "show translation result failed");
                }
            }
            Err(err) => {
                tracing::warn!(
                    error = %err,
                    request_id = message.request_id,
                    profile_id = message.profile_id,
                    "translation failed"
                );
                if let Err(show_err) = show_error_for_hwnd(hwnd, &err.user_summary()) {
                    tracing::warn!(error = %show_err, request_id = message.request_id, "show translation error failed");
                }
            }
        }
        return LRESULT(0);
    }
    if msg == WM_CLOSE
        || (msg == WM_KEYDOWN
            && edit_shortcut_action(wparam.0 as u32, false) == EditShortcutAction::HideWindow)
    {
        unsafe {
            use windows::Win32::UI::WindowsAndMessaging::{
                GetDlgItem, SendMessageW, WM_CANCELMODE,
            };
            if let Ok(splitter) = GetDlgItem(Some(hwnd), ID_SPLITTER as i32) {
                let _ = SendMessageW(splitter, WM_CANCELMODE, None, None);
            }
            let _ = ShowWindow(hwnd, SW_HIDE);
        }
        return LRESULT(0);
    }
    if msg == WM_COMMAND {
        let command = wparam.0 & 0xffff;
        let notification = (wparam.0 >> 16) & 0xffff;
        if command == ID_SOURCE_EDIT as usize
            && notification == windows::Win32::UI::WindowsAndMessaging::EN_CHANGE as usize
        {
            // Native EDIT can draw changes without WM_PAINT; repaint the whole
            // source pane so the placeholder appears and disappears reliably.
            unsafe {
                let source = windows::Win32::Foundation::HWND(lparam.0 as _);
                let _ = InvalidateRect(Some(source), None, true);
            }
            return LRESULT(0);
        }
        match command {
            ID_TRANSLATE => unsafe {
                let _ = PostMessageW(Some(hwnd), WM_TRANSLATE_WINDOW_SOURCE, WPARAM(0), LPARAM(0));
                return LRESULT(0);
            },
            ID_UPDATE_BUTTON => unsafe {
                let _ = PostMessageW(
                    Some(hwnd),
                    WM_TRANSLATE_WINDOW_UPDATE_CLICKED,
                    WPARAM(0),
                    LPARAM(0),
                );
                return LRESULT(0);
            },
            command
                if command == ID_PROFILE_COMBO as usize
                    && notification
                        == windows::Win32::UI::WindowsAndMessaging::CBN_DROPDOWN as usize =>
            {
                if let Ok(combo) = unsafe {
                    windows::Win32::UI::WindowsAndMessaging::GetDlgItem(
                        Some(hwnd),
                        ID_PROFILE_COMBO as i32,
                    )
                } {
                    crate::ui::combo::set_combo_dropped(combo, true);
                    unsafe {
                        crate::ui::combo::prepare_modern_combo_dropdown(combo);
                        crate::ui::combo::invalidate_modern_combo_for_child(hwnd, combo);
                    }
                }
                return LRESULT(0);
            }
            command
                if command == ID_PROFILE_COMBO as usize
                    && notification
                        == windows::Win32::UI::WindowsAndMessaging::CBN_CLOSEUP as usize =>
            {
                if let Ok(combo) = unsafe {
                    windows::Win32::UI::WindowsAndMessaging::GetDlgItem(
                        Some(hwnd),
                        ID_PROFILE_COMBO as i32,
                    )
                } {
                    crate::ui::combo::set_combo_dropped(combo, false);
                    unsafe {
                        crate::ui::combo::invalidate_modern_combo_for_child(hwnd, combo);
                    }
                }
                return LRESULT(0);
            }
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
    if msg == WM_CTLCOLOREDIT
        && let Some(result) =
            unsafe { crate::ui::edit::handle_modern_edit_color(hwnd, wparam, lparam, false) }
    {
        return result;
    }
    if msg == WM_CTLCOLORSTATIC {
        let child = windows::Win32::Foundation::HWND(lparam.0 as *mut core::ffi::c_void);
        let id = unsafe { windows::Win32::UI::WindowsAndMessaging::GetDlgCtrlID(child) };
        if crate::ui::edit::is_modern_edit(id as usize)
            && let Some(result) =
                unsafe { crate::ui::edit::handle_modern_edit_color(hwnd, wparam, lparam, true) }
        {
            return result;
        }
        if translation_static_text_uses_window_background(id as usize) {
            return unsafe { handle_static_text_color(wparam) };
        }
    }
    if msg == WM_PAINT {
        let result = unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
        unsafe {
            crate::ui::edit::paint_modern_edit_border(hwnd, ID_SOURCE_EDIT as i32);
            crate::ui::edit::paint_modern_edit_border(hwnd, ID_TRANSLATED_EDIT as i32);
        }
        return result;
    }
    if msg == WM_DRAWITEM {
        if unsafe { crate::ui::combo::draw_owner_draw_combo_item(lparam.0 as _) } {
            return LRESULT(1);
        }
        if unsafe { crate::ui::button::draw_owner_draw_button(lparam.0 as _) } {
            return LRESULT(1);
        }
    }
    if msg == WM_MEASUREITEM
        && unsafe { crate::ui::combo::measure_owner_draw_combo_item(lparam.0 as _) }
    {
        return LRESULT(1);
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
pub(crate) fn post_translation_source(
    hwnd: windows::Win32::Foundation::HWND,
    message: TranslationSourceMessage,
) {
    post_translation_message(hwnd, WM_TRANSLATION_SOURCE_CAPTURED, message);
}

#[cfg(windows)]
pub(crate) fn post_translation_result(
    hwnd: windows::Win32::Foundation::HWND,
    message: TranslationTaskMessage,
) {
    post_translation_message(hwnd, WM_TRANSLATION_TASK_FINISHED, message);
}

#[cfg(windows)]
fn post_translation_message<T>(
    hwnd: windows::Win32::Foundation::HWND,
    message_id: u32,
    message: T,
) {
    let message = Box::into_raw(Box::new(message));
    unsafe {
        if let Err(err) = windows::Win32::UI::WindowsAndMessaging::PostMessageW(
            Some(hwnd),
            message_id,
            windows::Win32::Foundation::WPARAM(0),
            windows::Win32::Foundation::LPARAM(message as isize),
        ) {
            drop(Box::from_raw(message));
            tracing::warn!(error = %err, message_id, "post translation window message failed");
        }
    }
}

#[cfg(windows)]
fn active_request_id(hwnd: windows::Win32::Foundation::HWND) -> usize {
    unsafe {
        windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(
            hwnd,
            windows::Win32::UI::WindowsAndMessaging::GWLP_USERDATA,
        ) as usize
    }
}

#[cfg(windows)]
fn set_active_request_id(hwnd: windows::Win32::Foundation::HWND, request_id: usize) {
    unsafe {
        windows::Win32::UI::WindowsAndMessaging::SetWindowLongPtrW(
            hwnd,
            windows::Win32::UI::WindowsAndMessaging::GWLP_USERDATA,
            request_id as isize,
        );
    }
}

#[cfg(windows)]
fn show_loading_for_hwnd(
    hwnd: windows::Win32::Foundation::HWND,
    source_text: String,
) -> Result<()> {
    set_child_text(hwnd, ID_SOURCE_EDIT, &source_text)?;
    set_child_text(hwnd, ID_TRANSLATED_EDIT, "")?;
    set_child_text(hwnd, ID_STATUS_TEXT, "正在翻译...")?;
    show_window_at_cursor(hwnd, ShowMode::Loading);
    tracing::info!("show translation window loading state");
    Ok(())
}

#[cfg(windows)]
fn show_result_for_hwnd(
    hwnd: windows::Win32::Foundation::HWND,
    result: &crate::app::TranslationWorkflowResult,
) -> Result<()> {
    set_child_text(hwnd, ID_SOURCE_EDIT, &result.source_text)?;
    set_child_text(hwnd, ID_TRANSLATED_EDIT, &result.translated_text)?;
    set_child_text(hwnd, ID_STATUS_TEXT, result.provider)?;
    show_window_at_cursor(hwnd, ShowMode::Result);
    tracing::info!("show translation window result");
    Ok(())
}

#[cfg(windows)]
fn show_error_for_hwnd(hwnd: windows::Win32::Foundation::HWND, message: &str) -> Result<()> {
    set_child_text(hwnd, ID_STATUS_TEXT, message)?;
    show_window_at_cursor(hwnd, ShowMode::Error);
    tracing::info!("show translation window error");
    Ok(())
}

#[cfg(windows)]
fn set_child_text(
    hwnd: windows::Win32::Foundation::HWND,
    control_id: isize,
    text: &str,
) -> Result<()> {
    let child = unsafe {
        windows::Win32::UI::WindowsAndMessaging::GetDlgItem(Some(hwnd), control_id as i32)
            .map_err(|err| AppError::Windows(format!("获取翻译窗口控件失败: {err}")))?
    };
    set_text(child, text)
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
        GetDlgCtrlID, GetMessageTime, GetParent, PostMessageW, SendMessageW, WM_CHAR, WM_CLOSE,
        WM_KEYDOWN, WM_KILLFOCUS, WM_LBUTTONDBLCLK, WM_LBUTTONDOWN, WM_NCDESTROY, WM_PAINT,
        WM_SETFOCUS, WM_SETTEXT,
    };

    let state_ptr = ref_data as *mut EditSubclassState;
    let is_source = unsafe { GetDlgCtrlID(hwnd) } == ID_SOURCE_EDIT as i32;

    if msg == WM_PAINT && is_source {
        let result = unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) };
        unsafe {
            paint_source_placeholder(hwnd);
        }
        return result;
    }
    if msg == WM_SETTEXT && is_source {
        // Multiline EDIT does not send EN_CHANGE for WM_SETTEXT.
        let result = unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) };
        unsafe {
            let _ = windows::Win32::Graphics::Gdi::InvalidateRect(Some(hwnd), None, true);
        }
        return result;
    }
    if is_source && (msg == WM_SETFOCUS || msg == WM_KILLFOCUS) {
        unsafe {
            let _ = windows::Win32::Graphics::Gdi::InvalidateRect(Some(hwnd), None, true);
        }
    }

    if (msg == WM_SETFOCUS || msg == WM_KILLFOCUS)
        && let Ok(parent) = unsafe { GetParent(hwnd) }
    {
        unsafe {
            crate::ui::edit::invalidate_modern_edit_for_child(parent, hwnd);
        }
    }
    if msg == WM_KEYDOWN {
        let ctrl_down = unsafe { GetKeyState(VK_CONTROL.0 as i32) } < 0;
        match edit_shortcut_action(wparam.0 as u32, ctrl_down) {
            EditShortcutAction::SelectAll => {
                unsafe {
                    let _ = SendMessageW(hwnd, EM_SETSEL, Some(WPARAM(0)), Some(LPARAM(-1)));
                }
                return LRESULT(0);
            }
            EditShortcutAction::Translate => {
                unsafe {
                    if let Ok(parent) = GetParent(hwnd) {
                        let _ = PostMessageW(
                            Some(parent),
                            WM_TRANSLATE_WINDOW_SOURCE,
                            WPARAM(0),
                            LPARAM(0),
                        );
                    }
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
    if msg == WM_CHAR {
        let ctrl_down = unsafe { GetKeyState(VK_CONTROL.0 as i32) } < 0;
        if edit_char_action(wparam.0 as u32) == EditCharAction::Swallow
            || (ctrl_down && wparam.0 as u32 == 0x0D)
        {
            return LRESULT(0);
        }
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
unsafe fn paint_source_placeholder(hwnd: windows::Win32::Foundation::HWND) {
    use windows::Win32::Foundation::{LPARAM, RECT};
    use windows::Win32::Graphics::Gdi::{
        DT_END_ELLIPSIS, DT_LEFT, DT_NOPREFIX, DT_SINGLELINE, DT_TOP, DrawTextW, GetDC, HGDIOBJ,
        ReleaseDC, RestoreDC, SaveDC, SelectObject, SetBkMode, SetTextColor, TRANSPARENT,
    };
    use windows::Win32::UI::Controls::EM_GETRECT;
    use windows::Win32::UI::Input::KeyboardAndMouse::GetFocus;
    use windows::Win32::UI::WindowsAndMessaging::{
        GetWindowTextLengthW, HideCaret, SendMessageW, ShowCaret, WM_GETFONT,
    };

    unsafe {
        if GetWindowTextLengthW(hwnd) != 0 {
            return;
        }
        let hdc = GetDC(Some(hwnd));
        if hdc.is_invalid() {
            return;
        }
        let saved = SaveDC(hdc);
        if saved == 0 {
            let _ = ReleaseDC(Some(hwnd), hdc);
            return;
        }
        let caret_hidden = GetFocus() == hwnd && HideCaret(Some(hwnd)).is_ok();
        let mut rect = RECT::default();
        let _ = SendMessageW(
            hwnd,
            EM_GETRECT,
            None,
            Some(LPARAM(&mut rect as *mut _ as isize)),
        );
        let font = SendMessageW(hwnd, WM_GETFONT, None, None);
        if font.0 != 0 {
            SelectObject(hdc, HGDIOBJ(font.0 as _));
        }
        SetBkMode(hdc, TRANSPARENT);
        SetTextColor(hdc, crate::ui::theme::COLOR_PLACEHOLDER.colorref());
        let mut text = wide("输入待翻译文本");
        DrawTextW(
            hdc,
            &mut text,
            &mut rect,
            DT_LEFT | DT_TOP | DT_SINGLELINE | DT_NOPREFIX | DT_END_ELLIPSIS,
        );
        let _ = RestoreDC(hdc, saved);
        let _ = ReleaseDC(Some(hwnd), hdc);
        if caret_hidden {
            let _ = ShowCaret(Some(hwnd));
        }
    }
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
        false,
    )
}

#[cfg(windows)]
unsafe fn handle_static_text_color(
    wparam: windows::Win32::Foundation::WPARAM,
) -> windows::Win32::Foundation::LRESULT {
    use windows::Win32::Graphics::Gdi::{
        COLOR_WINDOW, GetSysColorBrush, HDC, SetBkMode, TRANSPARENT,
    };

    let hdc = HDC(wparam.0 as *mut core::ffi::c_void);
    unsafe {
        let _ = SetBkMode(hdc, TRANSPARENT);
    }
    windows::Win32::Foundation::LRESULT(unsafe { GetSysColorBrush(COLOR_WINDOW) }.0 as isize)
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
    use windows::Win32::UI::WindowsAndMessaging::{BS_OWNERDRAW, BS_PUSHBUTTON, WINDOW_STYLE};
    let owner_draw = crate::ui::button::is_owner_draw_button(id as usize);
    let style = if owner_draw {
        WINDOW_STYLE((BS_PUSHBUTTON | BS_OWNERDRAW) as u32)
    } else {
        WINDOW_STYLE(BS_PUSHBUTTON as u32)
    };
    let hwnd = create_control(
        parent,
        "BUTTON",
        text,
        x,
        y,
        width,
        height,
        id,
        style,
        crate::ui::button::button_uses_native_border(id as usize),
    )?;
    if owner_draw {
        crate::ui::button::install_owner_draw_button_hover(
            hwnd,
            crate::ui::button::button_role_for_control(id as usize)
                .expect("owner-draw button must have a semantic role"),
        )?;
    }
    Ok(hwnd)
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
    let rect = crate::ui::edit::modern_edit_child_rect(x, y, width, height);
    let hwnd = create_control(
        parent,
        "EDIT",
        "",
        rect.x,
        rect.y,
        rect.width,
        rect.height,
        id,
        style,
        crate::ui::edit::edit_uses_native_border(id as usize),
    )?;
    if crate::ui::edit::is_modern_edit(id as usize) {
        crate::ui::edit::install_modern_edit_focus_tracking(hwnd)?;
    }
    Ok(hwnd)
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
    use windows::Win32::UI::WindowsAndMessaging::{
        CBS_DROPDOWNLIST, CBS_HASSTRINGS, CBS_OWNERDRAWFIXED, WINDOW_STYLE, WS_VSCROLL,
    };
    let mut style_bits = CBS_DROPDOWNLIST as u32 | WS_VSCROLL.0;
    if crate::ui::combo::is_modern_combo(id as usize) {
        style_bits |= (CBS_OWNERDRAWFIXED | CBS_HASSTRINGS) as u32;
    }
    let hwnd = create_control(
        parent,
        "COMBOBOX",
        "",
        x,
        y,
        width,
        height,
        id,
        WINDOW_STYLE(style_bits),
        crate::ui::combo::combo_uses_native_border(id as usize),
    )?;
    if crate::ui::combo::is_modern_combo(id as usize) {
        crate::ui::combo::install_modern_combo_tracking(hwnd)?;
    }
    Ok(hwnd)
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
    bordered: bool,
) -> Result<windows::Win32::Foundation::HWND> {
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, HMENU, WINDOW_EX_STYLE, WS_BORDER, WS_CHILD, WS_TABSTOP, WS_VISIBLE,
    };
    use windows::core::PCWSTR;

    let hwnd = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(wide(class_name).as_ptr()),
            PCWSTR(wide(text).as_ptr()),
            WS_CHILD
                | WS_VISIBLE
                | if id != 0 && class_name != "STATIC" {
                    WS_TABSTOP
                } else {
                    windows::Win32::UI::WindowsAndMessaging::WINDOW_STYLE(0)
                }
                | if bordered {
                    WS_BORDER
                } else {
                    windows::Win32::UI::WindowsAndMessaging::WINDOW_STYLE(0)
                }
                | extra_style,
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
fn edit_content_rect(rect: ControlRect) -> ControlRect {
    let gutter = crate::ui::theme::scale(crate::ui::edit::EDIT_FRAME_GUTTER);
    ControlRect {
        x: rect.x + gutter,
        y: rect.y + gutter,
        width: (rect.width - gutter * 2).max(1),
        height: (rect.height - gutter * 2).max(1),
    }
}

#[cfg(windows)]
fn apply_multiline_edit_padding(hwnd: windows::Win32::Foundation::HWND) -> Result<()> {
    use windows::Win32::Foundation::{LPARAM, RECT, WPARAM};
    use windows::Win32::Graphics::Gdi::InvalidateRect;
    use windows::Win32::UI::WindowsAndMessaging::{GetClientRect, SendMessageW};

    const EM_SETRECT: u32 = 0x00B3;
    const EM_SETMARGINS: u32 = 0x00D3;
    const EC_LEFTMARGIN: usize = 0x0001;
    const EC_RIGHTMARGIN: usize = 0x0002;

    unsafe {
        let mut client = RECT::default();
        GetClientRect(hwnd, &mut client)
            .map_err(|err| AppError::Windows(format!("获取编辑框尺寸失败: {err}")))?;
        let horizontal_padding =
            crate::ui::theme::scale(crate::ui::edit::MULTILINE_EDIT_HORIZONTAL_PADDING);
        let vertical_padding =
            crate::ui::theme::scale(crate::ui::edit::MULTILINE_EDIT_VERTICAL_PADDING);
        let mut rect = RECT {
            left: horizontal_padding,
            top: vertical_padding,
            right: (client.right - horizontal_padding).max(horizontal_padding),
            bottom: (client.bottom - vertical_padding).max(vertical_padding),
        };
        let margin = horizontal_padding as u16 as u32;
        let packed_margins = margin | (margin << 16);
        let _ = SendMessageW(
            hwnd,
            EM_SETMARGINS,
            Some(WPARAM(EC_LEFTMARGIN | EC_RIGHTMARGIN)),
            Some(LPARAM(packed_margins as isize)),
        );
        let _ = SendMessageW(
            hwnd,
            EM_SETRECT,
            Some(WPARAM(0)),
            Some(LPARAM((&mut rect as *mut RECT) as isize)),
        );
        let _ = InvalidateRect(Some(hwnd), None, true);
    }
    Ok(())
}

#[cfg(windows)]
fn translation_window_style() -> windows::Win32::UI::WindowsAndMessaging::WINDOW_STYLE {
    use windows::Win32::UI::WindowsAndMessaging::{
        WS_CAPTION, WS_CLIPCHILDREN, WS_OVERLAPPED, WS_SYSMENU, WS_THICKFRAME,
    };

    WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_THICKFRAME | WS_CLIPCHILDREN
}

#[cfg(windows)]
fn translation_window_outer_size(client_width: i32, client_height: i32) -> Result<(i32, i32)> {
    use windows::Win32::Foundation::RECT;
    use windows::Win32::UI::WindowsAndMessaging::{AdjustWindowRectEx, WINDOW_EX_STYLE};

    let mut rect = RECT {
        left: 0,
        top: 0,
        right: crate::ui::theme::scale(client_width),
        bottom: crate::ui::theme::scale(client_height),
    };
    unsafe {
        AdjustWindowRectEx(
            &mut rect,
            translation_window_style(),
            false,
            WINDOW_EX_STYLE::default(),
        )
        .map_err(|err| AppError::Windows(format!("计算翻译窗口尺寸失败: {err}")))?;
    }
    Ok((rect.right - rect.left, rect.bottom - rect.top))
}

#[cfg(windows)]
unsafe fn apply_min_track_size(lparam: windows::Win32::Foundation::LPARAM) {
    use windows::Win32::UI::WindowsAndMessaging::MINMAXINFO;

    let info = lparam.0 as *mut MINMAXINFO;
    if info.is_null() {
        return;
    }

    let (min_width, min_height) = translation_window_min_client_size();
    if let Ok((width, height)) = translation_window_outer_size(min_width, min_height) {
        unsafe {
            (*info).ptMinTrackSize.x = width;
            (*info).ptMinTrackSize.y = height;
        }
    }
}

#[cfg(windows)]
fn window_layout_mode(
    hwnd: windows::Win32::Foundation::HWND,
) -> crate::config::TranslationWindowLayout {
    let index = unsafe {
        windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(hwnd, LAYOUT_MODE_SLOT)
    };
    crate::config::TranslationWindowLayout::from_index(index as usize)
}

#[cfg(windows)]
fn window_source_ratio(hwnd: windows::Win32::Foundation::HWND) -> u32 {
    let ratio = unsafe {
        windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(hwnd, SOURCE_RATIO_SLOT)
    } as u32;
    if ratio == 0 {
        crate::config::PANE_RATIO_SCALE / 2
    } else {
        ratio
    }
}

#[cfg(windows)]
pub(super) fn current_window_layout(
    hwnd: windows::Win32::Foundation::HWND,
) -> Result<TranslationWindowLayout> {
    use windows::Win32::Foundation::RECT;
    use windows::Win32::UI::WindowsAndMessaging::GetClientRect;

    let mut rect = RECT::default();
    unsafe { GetClientRect(hwnd, &mut rect) }
        .map_err(|err| AppError::Windows(format!("获取翻译窗口尺寸失败: {err}")))?;
    Ok(translation_window_layout_for_system_dpi(
        rect.right - rect.left,
        rect.bottom - rect.top,
        window_layout_mode(hwnd),
        window_source_ratio(hwnd),
    ))
}

#[cfg(windows)]
pub(super) fn set_window_source_ratio(
    hwnd: windows::Win32::Foundation::HWND,
    ratio: u32,
) -> Result<()> {
    unsafe {
        windows::Win32::UI::WindowsAndMessaging::SetWindowLongPtrW(
            hwnd,
            SOURCE_RATIO_SLOT,
            ratio.clamp(100, crate::config::PANE_RATIO_SCALE - 100) as isize,
        );
    }
    resize_translation_window(hwnd)?;
    unsafe {
        let _ = windows::Win32::Graphics::Gdi::InvalidateRect(Some(hwnd), None, true);
    }
    Ok(())
}

#[cfg(windows)]
pub(super) fn notify_source_ratio_changed(hwnd: windows::Win32::Foundation::HWND) {
    unsafe {
        let _ = windows::Win32::UI::WindowsAndMessaging::PostMessageW(
            Some(hwnd),
            WM_TRANSLATE_WINDOW_RATIO_CHANGED,
            windows::Win32::Foundation::WPARAM(0),
            windows::Win32::Foundation::LPARAM(0),
        );
    }
}

#[cfg(windows)]
fn resize_translation_window(hwnd: windows::Win32::Foundation::HWND) -> Result<()> {
    unsafe {
        let layout = current_window_layout(hwnd)?;
        let splitter =
            windows::Win32::UI::WindowsAndMessaging::GetDlgItem(Some(hwnd), ID_SPLITTER as i32)
                .map_err(|err| AppError::Windows(format!("获取分隔条失败: {err}")))?;
        move_window(splitter, layout.splitter)?;
        set_window_visible(
            splitter,
            window_layout_mode(hwnd) != crate::config::TranslationWindowLayout::FixedSource,
        );
        let profile_combo = windows::Win32::UI::WindowsAndMessaging::GetDlgItem(
            Some(hwnd),
            ID_PROFILE_COMBO as i32,
        )
        .map_err(|err| AppError::Windows(format!("获取配置下拉框失败: {err}")))?;
        let source_edit =
            windows::Win32::UI::WindowsAndMessaging::GetDlgItem(Some(hwnd), ID_SOURCE_EDIT as i32)
                .map_err(|err| AppError::Windows(format!("获取原文输入框失败: {err}")))?;
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
        let update_button = windows::Win32::UI::WindowsAndMessaging::GetDlgItem(
            Some(hwnd),
            ID_UPDATE_BUTTON as i32,
        )
        .map_err(|err| AppError::Windows(format!("获取更新按钮失败: {err}")))?;
        move_window(
            profile_combo,
            ControlRect {
                height: crate::ui::theme::scale(translation_profile_combo_dropdown_height()),
                ..layout.profile_combo
            },
        )?;
        move_window(source_edit, edit_content_rect(layout.source_edit))?;
        apply_multiline_edit_padding(source_edit)?;
        move_window(translated_edit, edit_content_rect(layout.translated_edit))?;
        apply_multiline_edit_padding(translated_edit)?;
        move_window(status_text, layout.status_text)?;
        move_window(translate_button, layout.translate_button)?;
        move_window(update_button, layout.update_button)?;
        Ok(())
    }
}

#[cfg(windows)]
fn hide_window(hwnd: windows::Win32::Foundation::HWND) {
    unsafe {
        let _ = windows::Win32::UI::WindowsAndMessaging::ShowWindow(
            hwnd,
            windows::Win32::UI::WindowsAndMessaging::SW_HIDE,
        );
    }
}

#[cfg(windows)]
fn set_window_visible(hwnd: windows::Win32::Foundation::HWND, visible: bool) {
    let command = if visible {
        windows::Win32::UI::WindowsAndMessaging::SW_SHOW
    } else {
        windows::Win32::UI::WindowsAndMessaging::SW_HIDE
    };
    unsafe {
        let _ = windows::Win32::UI::WindowsAndMessaging::ShowWindow(hwnd, command);
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

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn escape_closes_window_from_every_focus_target() {
        use windows::Win32::Foundation::WPARAM;
        use windows::Win32::UI::Input::KeyboardAndMouse::VK_ESCAPE;
        use windows::Win32::UI::WindowsAndMessaging::{
            DestroyWindow, MSG, PM_REMOVE, PeekMessageW, WM_CLOSE, WM_KEYDOWN,
        };

        let window = TranslationWindow::new().unwrap();
        for hwnd in [
            window.hwnd,
            window.source_edit,
            window.translated_edit,
            window.translate_button,
            window.profile_combo,
            window.update_button,
        ] {
            let mut msg = MSG {
                hwnd,
                message: WM_KEYDOWN,
                wParam: WPARAM(VK_ESCAPE.0 as usize),
                ..Default::default()
            };
            assert!(window.handle_dialog_message(&mut msg));
            let mut close = MSG::default();
            assert!(unsafe {
                PeekMessageW(&mut close, Some(window.hwnd), WM_CLOSE, WM_CLOSE, PM_REMOVE).as_bool()
            });
        }
        let mut unrelated = MSG {
            message: WM_KEYDOWN,
            wParam: WPARAM(VK_ESCAPE.0 as usize),
            ..Default::default()
        };
        assert!(!window.handle_dialog_message(&mut unrelated));
        unsafe { DestroyWindow(window.hwnd).unwrap() };
    }
}
