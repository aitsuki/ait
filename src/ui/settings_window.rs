use crate::config::{AppSettings, TranslatorProvider};
use crate::error::{AppError, Result};

const SETTINGS_WINDOW_WIDTH: i32 = 720;
const SETTINGS_WINDOW_HEIGHT: i32 = 460;
const GOOGLE_NOTICE_TEXT: &str = "Google 使用内置免 Key 翻译，无需填写 Base URL、模型或 API Key。";
const API_KEY_PLACEHOLDER_TEXT: &str = "********";

#[cfg(windows)]
const ID_PROFILE_LIST: i32 = 3101;
#[cfg(windows)]
const ID_NAME: i32 = 3102;
#[cfg(windows)]
const ID_BASE_URL: i32 = 3104;
#[cfg(windows)]
const ID_MODEL: i32 = 3105;
#[cfg(windows)]
const ID_API_KEY: i32 = 3106;
#[cfg(windows)]
const ID_TIMEOUT: i32 = 3107;
#[cfg(windows)]
const ID_HOTKEY: i32 = 3108;
#[cfg(windows)]
const ID_GOOGLE_NOTICE: i32 = 3110;
#[cfg(windows)]
const ID_BASE_URL_LABEL: i32 = 3111;
#[cfg(windows)]
const ID_MODEL_LABEL: i32 = 3112;
#[cfg(windows)]
const ID_API_KEY_LABEL: i32 = 3113;
#[cfg(windows)]
const ID_TIMEOUT_LABEL: i32 = 3114;
#[cfg(windows)]
const ID_NAME_LABEL: i32 = 3115;
#[cfg(windows)]
const ID_API_KEY_VISIBILITY: isize = 3116;
#[cfg(windows)]
const EM_SET_PASSWORD_CHAR: u32 = 0x00CC;
#[cfg(windows)]
const ID_NEW_PROFILE: isize = 3001;
#[cfg(windows)]
const ID_DELETE_PROFILE: isize = 3002;
#[cfg(windows)]
const ID_SET_DEFAULT: isize = 3003;
#[cfg(windows)]
const ID_SAVE: isize = 3004;
#[cfg(windows)]
const ID_CANCEL: isize = 3005;
#[cfg(windows)]
pub const WM_SETTINGS_SAVED: u32 = windows::Win32::UI::WindowsAndMessaging::WM_APP + 40;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsViewModel {
    pub profiles: Vec<SettingsProfileListItem>,
    pub selected_profile: SettingsProfileDetail,
    pub hotkey: String,
    pub clipboard_capture_enabled: bool,
    pub copy_wait_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsProfileListItem {
    pub id: String,
    pub label: String,
    pub selected: bool,
    pub default: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsProfileDetail {
    pub id: String,
    pub name: String,
    pub provider: TranslatorProvider,
    pub base_url: String,
    pub model: String,
    pub has_api_key: bool,
    pub timeout_secs: u64,
    pub built_in: bool,
    pub can_delete: bool,
    pub name_editable: bool,
    pub network_fields_visible: bool,
    pub network_fields_enabled: bool,
    pub google_notice_visible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsProfileDetailUpdate {
    pub id: String,
    pub name: String,
    pub provider: TranslatorProvider,
    pub base_url: String,
    pub model: String,
    pub api_key: SettingsApiKeyUpdate,
    pub timeout_secs: u64,
    pub hotkey: String,
    pub copy_wait_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsApiKeyUpdate {
    Preserve,
    Clear,
    Replace(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsProfileDetailControl {
    NameLabel,
    NameInput,
    BaseUrlLabel,
    BaseUrlInput,
    ModelLabel,
    ModelInput,
    ApiKeyLabel,
    ApiKeyInput,
    TimeoutLabel,
    TimeoutInput,
    GoogleNotice,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingsProfileDetailControlState {
    pub control: SettingsProfileDetailControl,
    pub visible: bool,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsEditAction {
    NewProfile,
    DeleteProfile(String),
    SetDefault(String),
    SelectProfile(String),
}

pub fn api_key_placeholder_text() -> &'static str {
    API_KEY_PLACEHOLDER_TEXT
}

pub fn settings_api_key_input_text(has_api_key: bool) -> &'static str {
    if has_api_key {
        API_KEY_PLACEHOLDER_TEXT
    } else {
        ""
    }
}

pub fn settings_api_key_update_from_input(
    existing_encrypted_api_key: Option<String>,
    input: &str,
) -> SettingsApiKeyUpdate {
    if input == API_KEY_PLACEHOLDER_TEXT && existing_encrypted_api_key.is_some() {
        SettingsApiKeyUpdate::Preserve
    } else if input.trim().is_empty() {
        SettingsApiKeyUpdate::Clear
    } else {
        SettingsApiKeyUpdate::Replace(input.to_string())
    }
}

pub fn settings_profile_detail_control_states(
    profile: &SettingsProfileDetail,
) -> Vec<SettingsProfileDetailControlState> {
    let network_visible = profile.network_fields_visible;
    let network_enabled = profile.network_fields_enabled;
    vec![
        SettingsProfileDetailControlState {
            control: SettingsProfileDetailControl::NameLabel,
            visible: profile.name_editable,
            enabled: profile.name_editable,
        },
        SettingsProfileDetailControlState {
            control: SettingsProfileDetailControl::NameInput,
            visible: profile.name_editable,
            enabled: profile.name_editable,
        },
        SettingsProfileDetailControlState {
            control: SettingsProfileDetailControl::BaseUrlLabel,
            visible: network_visible,
            enabled: network_enabled,
        },
        SettingsProfileDetailControlState {
            control: SettingsProfileDetailControl::BaseUrlInput,
            visible: network_visible,
            enabled: network_enabled,
        },
        SettingsProfileDetailControlState {
            control: SettingsProfileDetailControl::ModelLabel,
            visible: network_visible,
            enabled: network_enabled,
        },
        SettingsProfileDetailControlState {
            control: SettingsProfileDetailControl::ModelInput,
            visible: network_visible,
            enabled: network_enabled,
        },
        SettingsProfileDetailControlState {
            control: SettingsProfileDetailControl::ApiKeyLabel,
            visible: network_visible,
            enabled: network_enabled,
        },
        SettingsProfileDetailControlState {
            control: SettingsProfileDetailControl::ApiKeyInput,
            visible: network_visible,
            enabled: network_enabled,
        },
        SettingsProfileDetailControlState {
            control: SettingsProfileDetailControl::TimeoutLabel,
            visible: network_visible,
            enabled: network_enabled,
        },
        SettingsProfileDetailControlState {
            control: SettingsProfileDetailControl::TimeoutInput,
            visible: network_visible,
            enabled: network_enabled,
        },
        SettingsProfileDetailControlState {
            control: SettingsProfileDetailControl::GoogleNotice,
            visible: profile.google_notice_visible,
            enabled: true,
        },
    ]
}

pub fn settings_profile_detail_control_rect(
    control: SettingsProfileDetailControl,
) -> SettingsControlRect {
    match control {
        SettingsProfileDetailControl::NameLabel => SettingsControlRect {
            x: 266,
            y: 102,
            width: 90,
            height: 22,
        },
        SettingsProfileDetailControl::NameInput => SettingsControlRect {
            x: 370,
            y: 100,
            width: 240,
            height: 24,
        },
        SettingsProfileDetailControl::BaseUrlLabel => SettingsControlRect {
            x: 266,
            y: 136,
            width: 90,
            height: 22,
        },
        SettingsProfileDetailControl::BaseUrlInput => SettingsControlRect {
            x: 370,
            y: 134,
            width: 300,
            height: 24,
        },
        SettingsProfileDetailControl::ModelLabel => SettingsControlRect {
            x: 266,
            y: 170,
            width: 90,
            height: 22,
        },
        SettingsProfileDetailControl::ModelInput => SettingsControlRect {
            x: 370,
            y: 168,
            width: 240,
            height: 24,
        },
        SettingsProfileDetailControl::ApiKeyLabel => SettingsControlRect {
            x: 266,
            y: 204,
            width: 90,
            height: 22,
        },
        SettingsProfileDetailControl::ApiKeyInput => SettingsControlRect {
            x: 370,
            y: 202,
            width: 240,
            height: 24,
        },
        SettingsProfileDetailControl::TimeoutLabel => SettingsControlRect {
            x: 266,
            y: 238,
            width: 90,
            height: 22,
        },
        SettingsProfileDetailControl::TimeoutInput => SettingsControlRect {
            x: 370,
            y: 236,
            width: 90,
            height: 24,
        },
        SettingsProfileDetailControl::GoogleNotice => SettingsControlRect {
            x: 266,
            y: 100,
            width: 420,
            height: 44,
        },
    }
}

pub fn settings_profile_google_notice_text() -> &'static str {
    GOOGLE_NOTICE_TEXT
}

pub fn settings_profile_detail_hidden_rect() -> SettingsControlRect {
    SettingsControlRect {
        x: -32000,
        y: -32000,
        width: 0,
        height: 0,
    }
}

pub fn apply_settings_edit_action(
    settings: &mut AppSettings,
    action: SettingsEditAction,
) -> Result<String> {
    match action {
        SettingsEditAction::NewProfile => Ok(settings.add_custom_profile().id),
        SettingsEditAction::DeleteProfile(id) => {
            settings.delete_profile(&id)?;
            Ok(settings.default_profile_id.clone())
        }
        SettingsEditAction::SetDefault(id) => {
            settings.set_default_profile(&id)?;
            Ok(id)
        }
        SettingsEditAction::SelectProfile(id) => {
            if settings.profile_by_id(&id).is_none() {
                return Err(AppError::Config("翻译配置不存在".to_string()));
            }
            Ok(id)
        }
    }
}

pub fn apply_settings_detail_update(
    settings: &mut AppSettings,
    update: SettingsProfileDetailUpdate,
) -> Result<()> {
    settings.hotkey = update.hotkey;

    let profile = settings
        .profile_by_id_mut(&update.id)
        .ok_or_else(|| AppError::Config("翻译配置不存在".to_string()))?;
    profile.name = update.name.trim().to_string();
    if profile.name.is_empty() {
        profile.name = profile.provider.display_name().to_string();
    }
    if profile.provider == TranslatorProvider::Google {
        profile.base_url.clear();
        profile.model.clear();
        profile.encrypted_api_key = None;
        profile.timeout_secs = 0;
    } else {
        profile.base_url = update.base_url;
        profile.model = update.model;
        match update.api_key {
            SettingsApiKeyUpdate::Preserve => {}
            SettingsApiKeyUpdate::Clear => profile.encrypted_api_key = None,
            SettingsApiKeyUpdate::Replace(api_key) => {
                if api_key.trim().is_empty() {
                    profile.encrypted_api_key = None;
                } else {
                    profile.encrypted_api_key = Some(api_key);
                }
            }
        }
        profile.timeout_secs = update.timeout_secs.max(1);
    }
    Ok(())
}

impl SettingsViewModel {
    pub fn from_settings_with_selected(settings: &AppSettings, selected_profile_id: &str) -> Self {
        let selected = settings
            .profile_by_id(selected_profile_id)
            .or_else(|| settings.profile_by_id(&settings.default_profile_id))
            .or_else(|| settings.translator_profiles.first())
            .expect("settings always contain profiles after normalization");
        let is_google = selected.provider == TranslatorProvider::Google;
        let (base_url, model, has_api_key, timeout_secs) = if is_google {
            (String::new(), String::new(), false, 0)
        } else {
            (
                selected.base_url.clone(),
                selected.model.clone(),
                selected.encrypted_api_key.is_some(),
                selected.timeout_secs,
            )
        };
        Self {
            profiles: settings
                .translator_profiles
                .iter()
                .map(|profile| SettingsProfileListItem {
                    id: profile.id.clone(),
                    label: profile_list_label(profile, profile.id == settings.default_profile_id),
                    selected: profile.id == selected.id,
                    default: profile.id == settings.default_profile_id,
                })
                .collect(),
            selected_profile: SettingsProfileDetail {
                id: selected.id.clone(),
                name: selected.name.clone(),
                provider: selected.provider,
                base_url,
                model,
                has_api_key,
                timeout_secs,
                built_in: selected.built_in,
                can_delete: !selected.built_in,
                name_editable: !is_google,
                network_fields_visible: !is_google,
                network_fields_enabled: !is_google,
                google_notice_visible: is_google,
            },
            hotkey: settings.hotkey.clone(),
            clipboard_capture_enabled: settings.clipboard_capture.enabled,
            copy_wait_ms: settings.clipboard_capture.copy_wait_ms,
        }
    }
}

fn profile_list_label(profile: &crate::config::TranslatorProfile, is_default: bool) -> String {
    if is_default {
        format!("{}（默认）", profile.name)
    } else {
        profile.name.clone()
    }
}

impl From<&AppSettings> for SettingsViewModel {
    fn from(settings: &AppSettings) -> Self {
        Self::from_settings_with_selected(settings, &settings.default_profile_id)
    }
}

#[cfg(windows)]
pub struct SettingsWindow;

#[cfg(windows)]
impl SettingsWindow {
    pub fn open(
        settings: &AppSettings,
        owner_hwnd: windows::Win32::Foundation::HWND,
    ) -> Result<()> {
        use windows::Win32::Foundation::GetLastError;
        use windows::Win32::Graphics::Gdi::{
            COLOR_WINDOW, GetMonitorInfoW, GetSysColorBrush, MONITOR_DEFAULTTONEAREST, MONITORINFO,
            MonitorFromPoint,
        };
        use windows::Win32::UI::WindowsAndMessaging::{
            CreateWindowExW, GWLP_USERDATA, GetCursorPos, IDC_ARROW, LoadCursorW, RegisterClassW,
            SW_SHOW, SetWindowLongPtrW, ShowWindow, WINDOW_EX_STYLE, WNDCLASSW, WS_CAPTION,
            WS_OVERLAPPED, WS_SYSMENU,
        };
        use windows::core::PCWSTR;

        let view_model = SettingsViewModel::from(settings);
        let class_name = wide("ait_settings_window");
        unsafe {
            let class = WNDCLASSW {
                lpfnWndProc: Some(default_wnd_proc),
                lpszClassName: PCWSTR(class_name.as_ptr()),
                hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
                hbrBackground: GetSysColorBrush(COLOR_WINDOW),
                ..Default::default()
            };
            let atom = RegisterClassW(&class);
            if !can_continue_after_register_class(atom, GetLastError()) {
                return Err(AppError::Windows("注册设置窗口类失败".to_string()));
            }

            let mut cursor = windows::Win32::Foundation::POINT::default();
            let _ = GetCursorPos(&mut cursor);
            let monitor = MonitorFromPoint(cursor, MONITOR_DEFAULTTONEAREST);
            let mut monitor_info = MONITORINFO {
                cbSize: std::mem::size_of::<MONITORINFO>() as u32,
                ..Default::default()
            };
            let _ = GetMonitorInfoW(monitor, &mut monitor_info);
            let work = monitor_info.rcWork;
            let (x, y) = settings_window_center_position(
                (work.left, work.top, work.right, work.bottom),
                (SETTINGS_WINDOW_WIDTH, SETTINGS_WINDOW_HEIGHT),
            );

            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                PCWSTR(class_name.as_ptr()),
                PCWSTR(wide("ait 设置").as_ptr()),
                WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU,
                x,
                y,
                SETTINGS_WINDOW_WIDTH,
                SETTINGS_WINDOW_HEIGHT,
                Some(owner_hwnd),
                None,
                None,
                None,
            )
            .map_err(|err| AppError::Windows(format!("创建设置窗口失败: {err}")))?;
            let settings_ptr = Box::into_raw(Box::new(settings.clone()));
            let _ = SetWindowLongPtrW(hwnd, GWLP_USERDATA, settings_ptr as isize);

            create_static(hwnd, "快捷键", 18, 20, 90, 22)?;
            create_edit(hwnd, &view_model.hotkey, 118, 18, 180, 24, false, ID_HOTKEY)?;
            create_static(hwnd, "", 18, 62, 668, 1)?;

            create_static(hwnd, "翻译配置", 18, 74, 120, 22)?;
            let profile_list = create_listbox(hwnd, 18, 100, 220, 228, ID_PROFILE_LIST)?;
            reset_profile_list(profile_list, &view_model)?;
            create_button(hwnd, "新增", 18, 342, 64, 28, ID_NEW_PROFILE)?;
            let delete_button = create_button(hwnd, "删除", 90, 342, 64, 28, ID_DELETE_PROFILE)?;
            create_button(hwnd, "设为默认", 162, 342, 76, 28, ID_SET_DEFAULT)?;

            create_static_with_id(hwnd, "名称", 266, 102, 90, 22, ID_NAME_LABEL)?;
            create_edit(
                hwnd,
                &view_model.selected_profile.name,
                370,
                100,
                240,
                24,
                false,
                ID_NAME,
            )?;
            create_static_with_id(hwnd, "Base URL", 266, 136, 90, 22, ID_BASE_URL_LABEL)?;
            create_edit(
                hwnd,
                &view_model.selected_profile.base_url,
                370,
                134,
                300,
                24,
                false,
                ID_BASE_URL,
            )?;
            create_static_with_id(hwnd, "模型", 266, 170, 90, 22, ID_MODEL_LABEL)?;
            create_edit(
                hwnd,
                &view_model.selected_profile.model,
                370,
                168,
                240,
                24,
                false,
                ID_MODEL,
            )?;
            create_static_with_id(hwnd, "API Key", 266, 204, 90, 22, ID_API_KEY_LABEL)?;
            create_edit(
                hwnd,
                settings_api_key_input_text(view_model.selected_profile.has_api_key),
                370,
                202,
                240,
                24,
                true,
                ID_API_KEY,
            )?;
            let api_key_visibility_button =
                create_button(hwnd, "显示", 618, 200, 52, 28, ID_API_KEY_VISIBILITY)?;
            create_static_with_id(hwnd, "超时秒数", 266, 238, 90, 22, ID_TIMEOUT_LABEL)?;
            create_edit(
                hwnd,
                &view_model.selected_profile.timeout_secs.to_string(),
                370,
                236,
                90,
                24,
                false,
                ID_TIMEOUT,
            )?;
            create_static_with_id(
                hwnd,
                GOOGLE_NOTICE_TEXT,
                266,
                278,
                390,
                36,
                ID_GOOGLE_NOTICE,
            )?;
            apply_profile_detail_ui_state(hwnd, &view_model.selected_profile);
            let _ = windows::Win32::UI::Input::KeyboardAndMouse::EnableWindow(
                delete_button,
                view_model.selected_profile.can_delete,
            );
            let _ = windows::Win32::UI::Input::KeyboardAndMouse::EnableWindow(
                api_key_visibility_button,
                view_model.selected_profile.has_api_key,
            );
            create_button(hwnd, "保存", 534, 382, 72, 28, ID_SAVE)?;
            create_button(hwnd, "取消", 614, 382, 72, 28, ID_CANCEL)?;
            let _ = ShowWindow(hwnd, SW_SHOW);
        }

        tracing::info!(?view_model, "open settings window");
        Ok(())
    }
}

pub fn settings_window_center_position(
    work_area: (i32, i32, i32, i32),
    window_size: (i32, i32),
) -> (i32, i32) {
    let (left, top, right, bottom) = work_area;
    let (width, height) = window_size;
    let x = left + ((right - left - width) / 2).max(0);
    let y = top + ((bottom - top - height) / 2).max(0);
    (x, y)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsSaveOutcome {
    KeepOpen,
}

pub fn settings_save_outcome_after_success() -> SettingsSaveOutcome {
    SettingsSaveOutcome::KeepOpen
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingsControlRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingsWindowLayout {
    pub hotkey: SettingsControlRect,
    pub separator: SettingsControlRect,
    pub profile_list: SettingsControlRect,
    pub name: SettingsControlRect,
}

pub fn settings_window_layout() -> SettingsWindowLayout {
    SettingsWindowLayout {
        hotkey: SettingsControlRect {
            x: 118,
            y: 18,
            width: 180,
            height: 24,
        },
        separator: SettingsControlRect {
            x: 18,
            y: 62,
            width: 668,
            height: 1,
        },
        profile_list: SettingsControlRect {
            x: 18,
            y: 100,
            width: 220,
            height: 228,
        },
        name: SettingsControlRect {
            x: 370,
            y: 100,
            width: 240,
            height: 24,
        },
    }
}

pub fn settings_window_uses_background_brush() -> bool {
    true
}

pub fn settings_static_controls_have_border() -> bool {
    false
}

#[cfg(windows)]
pub fn can_continue_after_register_class(
    atom: u16,
    last_error: windows::Win32::Foundation::WIN32_ERROR,
) -> bool {
    use windows::Win32::Foundation::ERROR_CLASS_ALREADY_EXISTS;

    atom != 0 || last_error == ERROR_CLASS_ALREADY_EXISTS
}

#[cfg(windows)]
unsafe extern "system" fn default_wnd_proc(
    hwnd: windows::Win32::Foundation::HWND,
    msg: u32,
    wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::LRESULT {
    use windows::Win32::Foundation::LRESULT;
    use windows::Win32::UI::WindowsAndMessaging::{
        DefWindowProcW, DestroyWindow, GWLP_USERDATA, GetWindowLongPtrW, SetWindowLongPtrW,
        WM_CLOSE, WM_COMMAND, WM_NCDESTROY,
    };

    if msg == WM_CLOSE {
        unsafe {
            let _ = DestroyWindow(hwnd);
        }
        return LRESULT(0);
    }
    if msg == WM_COMMAND {
        let command = wparam.0 & 0xffff;
        let notification = (wparam.0 >> 16) & 0xffff;
        if command == ID_PROFILE_LIST as usize
            && notification == windows::Win32::UI::WindowsAndMessaging::LBN_SELCHANGE as usize
        {
            if let Err(err) = unsafe { load_selected_profile_into_window(hwnd) } {
                tracing::warn!(error = %err, "select settings profile failed");
            }
            return LRESULT(0);
        }
        if command == ID_NEW_PROFILE as usize {
            if let Err(err) =
                unsafe { edit_settings_profiles(hwnd, SettingsEditAction::NewProfile) }
            {
                tracing::warn!(error = %err, "create settings profile failed");
            }
            return LRESULT(0);
        }
        if command == ID_DELETE_PROFILE as usize {
            match selected_profile_id(hwnd).and_then(|id| unsafe {
                edit_settings_profiles(hwnd, SettingsEditAction::DeleteProfile(id))
            }) {
                Ok(_) => {}
                Err(err) => unsafe {
                    show_message(hwnd, "删除失败", &err.to_string());
                },
            }
            return LRESULT(0);
        }
        if command == ID_SET_DEFAULT as usize {
            match selected_profile_id(hwnd).and_then(|id| unsafe {
                edit_settings_profiles(hwnd, SettingsEditAction::SetDefault(id))
            }) {
                Ok(_) => {}
                Err(err) => unsafe {
                    show_message(hwnd, "设置失败", &err.to_string());
                },
            }
            return LRESULT(0);
        }
        if command == ID_API_KEY_VISIBILITY as usize {
            if let Err(err) = unsafe { toggle_api_key_visibility(hwnd) } {
                tracing::warn!(error = %err, "toggle api key visibility failed");
                unsafe {
                    show_message(hwnd, "读取失败", &err.user_summary());
                }
            }
            return LRESULT(0);
        }
        if command == ID_SAVE as usize {
            match unsafe { save_settings_from_window(hwnd) } {
                Ok(_) => unsafe {
                    if let Some(owner) = get_owner_hwnd(hwnd) {
                        let _ = windows::Win32::UI::WindowsAndMessaging::PostMessageW(
                            Some(owner),
                            WM_SETTINGS_SAVED,
                            windows::Win32::Foundation::WPARAM(0),
                            windows::Win32::Foundation::LPARAM(0),
                        );
                    }
                },
                Err(err) => {
                    tracing::warn!(error = %err, "save settings failed");
                    unsafe {
                        show_message(hwnd, "保存失败", &err.to_string());
                    }
                }
            }
            return LRESULT(0);
        }
        if command == ID_CANCEL as usize {
            unsafe {
                let _ = DestroyWindow(hwnd);
            }
            return LRESULT(0);
        }
    }
    if msg == WM_NCDESTROY {
        let ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) };
        if ptr != 0 {
            unsafe {
                drop(Box::from_raw(ptr as *mut AppSettings));
                let _ = SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            }
        }
    }

    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

#[cfg(windows)]
unsafe fn save_settings_from_window(hwnd: windows::Win32::Foundation::HWND) -> Result<()> {
    use windows::Win32::UI::WindowsAndMessaging::{GWLP_USERDATA, GetWindowLongPtrW};

    let ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) };
    if ptr == 0 {
        return Err(AppError::Config("设置窗口状态缺失".to_string()));
    }
    let settings = unsafe { &mut *(ptr as *mut AppSettings) };
    let profile_id = selected_profile_id(hwnd)?;
    let existing_provider = settings
        .profile_by_id(&profile_id)
        .map(|profile| profile.provider)
        .ok_or_else(|| AppError::Config("翻译配置不存在".to_string()))?;
    let existing_encrypted_api_key = settings
        .profile_by_id(&profile_id)
        .and_then(|profile| profile.encrypted_api_key.clone());
    let api_key = read_control_text(hwnd, ID_API_KEY)?;
    let api_key_update =
        match settings_api_key_update_from_input(existing_encrypted_api_key, &api_key) {
            SettingsApiKeyUpdate::Replace(api_key) => SettingsApiKeyUpdate::Replace(
                crate::secret::SecretStore::new(&format!("ait-translator-profile-{profile_id}"))
                    .protect(&api_key)?,
            ),
            update => update,
        };
    apply_settings_detail_update(
        settings,
        SettingsProfileDetailUpdate {
            id: profile_id,
            name: read_control_text(hwnd, ID_NAME)?,
            provider: existing_provider,
            base_url: read_control_text(hwnd, ID_BASE_URL)?,
            model: read_control_text(hwnd, ID_MODEL)?,
            api_key: api_key_update,
            timeout_secs: read_control_text(hwnd, ID_TIMEOUT)?
                .parse::<u64>()
                .unwrap_or(30),
            hotkey: read_control_text(hwnd, ID_HOTKEY)?,
            copy_wait_ms: settings.clipboard_capture.copy_wait_ms,
        },
    )?;
    refresh_profile_list(hwnd, settings)?;

    let dir = crate::config::SettingsStore::default_dir()?;
    crate::config::SettingsStore::new(dir).save(settings)?;
    tracing::info!(
        default_profile_id = %settings.default_profile_id,
        "settings saved"
    );
    Ok(())
}

#[cfg(windows)]
unsafe fn edit_settings_profiles(
    hwnd: windows::Win32::Foundation::HWND,
    action: SettingsEditAction,
) -> Result<()> {
    use windows::Win32::UI::WindowsAndMessaging::{GWLP_USERDATA, GetWindowLongPtrW};

    let ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) };
    if ptr == 0 {
        return Err(AppError::Config("设置窗口状态缺失".to_string()));
    }
    let settings = unsafe { &mut *(ptr as *mut AppSettings) };
    let selected_id = apply_settings_edit_action(settings, action)?;
    refresh_profile_list_with_selected(hwnd, settings, &selected_id)?;
    load_profile_into_window(hwnd, settings, &selected_id)?;
    Ok(())
}

#[cfg(windows)]
unsafe fn load_selected_profile_into_window(hwnd: windows::Win32::Foundation::HWND) -> Result<()> {
    use windows::Win32::UI::WindowsAndMessaging::{GWLP_USERDATA, GetWindowLongPtrW};

    let ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) };
    if ptr == 0 {
        return Err(AppError::Config("设置窗口状态缺失".to_string()));
    }
    let settings = unsafe { &*(ptr as *const AppSettings) };
    let profile_id = selected_profile_id(hwnd)?;
    load_profile_into_window(hwnd, settings, &profile_id)
}

#[cfg(windows)]
fn load_profile_into_window(
    hwnd: windows::Win32::Foundation::HWND,
    settings: &AppSettings,
    profile_id: &str,
) -> Result<()> {
    let vm = SettingsViewModel::from_settings_with_selected(settings, profile_id);
    let profile = &vm.selected_profile;
    set_control_text(hwnd, ID_NAME, &profile.name)?;
    set_control_text(hwnd, ID_BASE_URL, &profile.base_url)?;
    set_control_text(hwnd, ID_MODEL, &profile.model)?;
    set_control_text(
        hwnd,
        ID_API_KEY,
        settings_api_key_input_text(profile.has_api_key),
    )?;
    set_control_text(hwnd, ID_API_KEY_VISIBILITY as i32, "显示")?;
    set_api_key_password_mode(hwnd, true)?;
    set_control_text(hwnd, ID_TIMEOUT, &profile.timeout_secs.to_string())?;
    set_control_text(
        hwnd,
        ID_GOOGLE_NOTICE,
        if profile.google_notice_visible {
            GOOGLE_NOTICE_TEXT
        } else {
            ""
        },
    )?;
    set_control_text(hwnd, ID_HOTKEY, &vm.hotkey)?;
    apply_profile_detail_ui_state(hwnd, profile);
    Ok(())
}

#[cfg(windows)]
unsafe fn toggle_api_key_visibility(hwnd: windows::Win32::Foundation::HWND) -> Result<()> {
    use windows::Win32::UI::WindowsAndMessaging::{GWLP_USERDATA, GetWindowLongPtrW};

    let button_text = read_control_text(hwnd, ID_API_KEY_VISIBILITY as i32)?;
    if button_text == "隐藏" {
        set_control_text(hwnd, ID_API_KEY, API_KEY_PLACEHOLDER_TEXT)?;
        set_api_key_password_mode(hwnd, true)?;
        set_control_text(hwnd, ID_API_KEY_VISIBILITY as i32, "显示")?;
        return Ok(());
    }

    let ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) };
    if ptr == 0 {
        return Err(AppError::Config("设置窗口状态缺失".to_string()));
    }
    let settings = unsafe { &*(ptr as *const AppSettings) };
    let profile_id = selected_profile_id(hwnd)?;
    let encrypted = settings
        .profile_by_id(&profile_id)
        .and_then(|profile| profile.encrypted_api_key.as_ref())
        .ok_or_else(|| AppError::Secret("API Key 未保存".to_string()))?;
    let api_key = crate::secret::SecretStore::new(&format!("ait-translator-profile-{profile_id}"))
        .unprotect(encrypted)?;

    set_api_key_password_mode(hwnd, false)?;
    set_control_text(hwnd, ID_API_KEY, &api_key)?;
    set_control_text(hwnd, ID_API_KEY_VISIBILITY as i32, "隐藏")?;
    Ok(())
}

#[cfg(windows)]
fn set_api_key_password_mode(hwnd: windows::Win32::Foundation::HWND, password: bool) -> Result<()> {
    use windows::Win32::Foundation::{LPARAM, WPARAM};
    use windows::Win32::Graphics::Gdi::InvalidateRect;
    use windows::Win32::UI::WindowsAndMessaging::SendMessageW;

    let edit = control(hwnd, ID_API_KEY)?;
    let password_char = if password { '*' as usize } else { 0 };
    unsafe {
        let _ = SendMessageW(
            edit,
            EM_SET_PASSWORD_CHAR,
            Some(WPARAM(password_char)),
            Some(LPARAM(0)),
        );
        let _ = InvalidateRect(Some(edit), None, true);
    }
    Ok(())
}

#[cfg(windows)]
fn apply_profile_detail_ui_state(
    hwnd: windows::Win32::Foundation::HWND,
    profile: &SettingsProfileDetail,
) {
    use windows::Win32::Graphics::Gdi::{
        InvalidateRect, RDW_ERASE, RDW_INVALIDATE, RDW_UPDATENOW, RedrawWindow,
    };
    use windows::Win32::UI::Input::KeyboardAndMouse::EnableWindow;
    use windows::Win32::UI::WindowsAndMessaging::{
        MoveWindow, SWP_HIDEWINDOW, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SWP_SHOWWINDOW,
        SetWindowPos,
    };

    if let Ok(delete_button) = control(hwnd, ID_DELETE_PROFILE as i32) {
        unsafe {
            let _ = EnableWindow(delete_button, profile.can_delete);
        }
    }
    if let Ok(api_key_visibility_button) = control(hwnd, ID_API_KEY_VISIBILITY as i32) {
        unsafe {
            let _ = EnableWindow(api_key_visibility_button, profile.has_api_key);
            let rect = if profile.network_fields_visible {
                SettingsControlRect {
                    x: 618,
                    y: 200,
                    width: 52,
                    height: 28,
                }
            } else {
                settings_profile_detail_hidden_rect()
            };
            let _ = MoveWindow(
                api_key_visibility_button,
                rect.x,
                rect.y,
                rect.width,
                rect.height,
                true,
            );
            let visibility_flag = if profile.network_fields_visible {
                SWP_SHOWWINDOW
            } else {
                SWP_HIDEWINDOW
            };
            let _ = SetWindowPos(
                api_key_visibility_button,
                None,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | visibility_flag,
            );
        }
    }
    for state in settings_profile_detail_control_states(profile) {
        let Some(id) = settings_profile_detail_control_id(state.control) else {
            continue;
        };
        if let Ok(child) = control(hwnd, id) {
            unsafe {
                let rect = if state.visible {
                    settings_profile_detail_control_rect(state.control)
                } else {
                    settings_profile_detail_hidden_rect()
                };
                let _ = MoveWindow(child, rect.x, rect.y, rect.width, rect.height, true);
                let visibility_flag = if state.visible {
                    SWP_SHOWWINDOW
                } else {
                    SWP_HIDEWINDOW
                };
                let _ = SetWindowPos(
                    child,
                    None,
                    0,
                    0,
                    0,
                    0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | visibility_flag,
                );
                let _ = EnableWindow(child, state.enabled);
            }
        }
    }

    unsafe {
        let _ = InvalidateRect(Some(hwnd), None, true);
        let _ = RedrawWindow(
            Some(hwnd),
            None,
            None,
            RDW_INVALIDATE | RDW_ERASE | RDW_UPDATENOW,
        );
    }
}

#[cfg(windows)]
fn settings_profile_detail_control_id(control: SettingsProfileDetailControl) -> Option<i32> {
    Some(match control {
        SettingsProfileDetailControl::NameInput => ID_NAME,
        SettingsProfileDetailControl::NameLabel => ID_NAME_LABEL,
        SettingsProfileDetailControl::BaseUrlLabel => ID_BASE_URL_LABEL,
        SettingsProfileDetailControl::BaseUrlInput => ID_BASE_URL,
        SettingsProfileDetailControl::ModelLabel => ID_MODEL_LABEL,
        SettingsProfileDetailControl::ModelInput => ID_MODEL,
        SettingsProfileDetailControl::ApiKeyLabel => ID_API_KEY_LABEL,
        SettingsProfileDetailControl::ApiKeyInput => ID_API_KEY,
        SettingsProfileDetailControl::TimeoutLabel => ID_TIMEOUT_LABEL,
        SettingsProfileDetailControl::TimeoutInput => ID_TIMEOUT,
        SettingsProfileDetailControl::GoogleNotice => ID_GOOGLE_NOTICE,
    })
}

#[cfg(windows)]
fn refresh_profile_list(
    hwnd: windows::Win32::Foundation::HWND,
    settings: &AppSettings,
) -> Result<()> {
    refresh_profile_list_with_selected(hwnd, settings, &settings.default_profile_id)
}

#[cfg(windows)]
fn refresh_profile_list_with_selected(
    hwnd: windows::Win32::Foundation::HWND,
    settings: &AppSettings,
    selected_profile_id: &str,
) -> Result<()> {
    let vm = SettingsViewModel::from_settings_with_selected(settings, selected_profile_id);
    let list = control(hwnd, ID_PROFILE_LIST)?;
    reset_profile_list(list, &vm)
}

#[cfg(windows)]
fn selected_profile_id(hwnd: windows::Win32::Foundation::HWND) -> Result<String> {
    use windows::Win32::Foundation::{LPARAM, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{LB_GETCURSEL, SendMessageW};

    let list = control(hwnd, ID_PROFILE_LIST)?;
    let index = unsafe { SendMessageW(list, LB_GETCURSEL, Some(WPARAM(0)), Some(LPARAM(0))) }.0;
    if index < 0 {
        return Err(AppError::Config("未选择翻译配置".to_string()));
    }
    let ptr = unsafe {
        windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(
            hwnd,
            windows::Win32::UI::WindowsAndMessaging::GWLP_USERDATA,
        )
    };
    if ptr == 0 {
        return Err(AppError::Config("设置窗口状态缺失".to_string()));
    }
    let settings = unsafe { &*(ptr as *const AppSettings) };
    settings
        .translator_profiles
        .get(index as usize)
        .map(|profile| profile.id.clone())
        .ok_or_else(|| AppError::Config("翻译配置不存在".to_string()))
}

#[cfg(windows)]
unsafe fn get_owner_hwnd(
    hwnd: windows::Win32::Foundation::HWND,
) -> Option<windows::Win32::Foundation::HWND> {
    use windows::Win32::UI::WindowsAndMessaging::GW_OWNER;
    use windows::Win32::UI::WindowsAndMessaging::GetWindow;

    unsafe { GetWindow(hwnd, GW_OWNER).ok() }
}

#[cfg(windows)]
fn read_control_text(hwnd: windows::Win32::Foundation::HWND, id: i32) -> Result<String> {
    use windows::Win32::UI::WindowsAndMessaging::GetDlgItemTextW;

    let mut buf = [0u16; 1024];
    let len = unsafe { GetDlgItemTextW(hwnd, id, &mut buf) } as usize;
    Ok(String::from_utf16_lossy(&buf[..len]))
}

#[cfg(windows)]
fn set_control_text(hwnd: windows::Win32::Foundation::HWND, id: i32, text: &str) -> Result<()> {
    use windows::Win32::UI::WindowsAndMessaging::SetDlgItemTextW;

    let text = wide(text);
    unsafe {
        SetDlgItemTextW(hwnd, id, windows::core::PCWSTR(text.as_ptr()))
            .map_err(|err| AppError::Windows(format!("设置控件文本失败: {err}")))?;
    }
    Ok(())
}

#[cfg(windows)]
fn control(
    hwnd: windows::Win32::Foundation::HWND,
    id: i32,
) -> Result<windows::Win32::Foundation::HWND> {
    use windows::Win32::UI::WindowsAndMessaging::GetDlgItem;

    unsafe { GetDlgItem(Some(hwnd), id) }
        .map_err(|err| AppError::Windows(format!("获取控件失败: {err}")))
}

#[cfg(windows)]
unsafe fn show_message(hwnd: windows::Win32::Foundation::HWND, caption: &str, text: &str) {
    let text = wide(text);
    let caption = wide(caption);
    unsafe {
        let _ = windows::Win32::UI::WindowsAndMessaging::MessageBoxW(
            Some(hwnd),
            windows::core::PCWSTR(text.as_ptr()),
            windows::core::PCWSTR(caption.as_ptr()),
            windows::Win32::UI::WindowsAndMessaging::MB_OK,
        );
    }
}

#[cfg(windows)]
fn reset_profile_list(
    list: windows::Win32::Foundation::HWND,
    view_model: &SettingsViewModel,
) -> Result<()> {
    use windows::Win32::Foundation::{LPARAM, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        LB_ADDSTRING, LB_RESETCONTENT, LB_SETCURSEL, SendMessageW,
    };

    unsafe {
        let _ = SendMessageW(list, LB_RESETCONTENT, Some(WPARAM(0)), Some(LPARAM(0)));
        for item in &view_model.profiles {
            let label = wide(&item.label);
            let _ = SendMessageW(
                list,
                LB_ADDSTRING,
                Some(WPARAM(0)),
                Some(LPARAM(label.as_ptr() as isize)),
            );
        }
        let selected_index = view_model
            .profiles
            .iter()
            .position(|item| item.selected)
            .unwrap_or(0);
        let _ = SendMessageW(
            list,
            LB_SETCURSEL,
            Some(WPARAM(selected_index)),
            Some(LPARAM(0)),
        );
    }
    Ok(())
}

#[cfg(windows)]
fn create_static(
    parent: windows::Win32::Foundation::HWND,
    text: &str,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> Result<windows::Win32::Foundation::HWND> {
    create_control(
        parent,
        "STATIC",
        text,
        x,
        y,
        width,
        height,
        0,
        Default::default(),
        false,
    )
}

#[cfg(windows)]
fn create_static_with_id(
    parent: windows::Win32::Foundation::HWND,
    text: &str,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    id: i32,
) -> Result<windows::Win32::Foundation::HWND> {
    create_control(
        parent,
        "STATIC",
        text,
        x,
        y,
        width,
        height,
        id as isize,
        Default::default(),
        false,
    )
}

#[cfg(windows)]
fn create_listbox(
    parent: windows::Win32::Foundation::HWND,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    id: i32,
) -> Result<windows::Win32::Foundation::HWND> {
    use windows::Win32::UI::WindowsAndMessaging::{LBS_NOTIFY, WINDOW_STYLE, WS_VSCROLL};
    create_control(
        parent,
        "LISTBOX",
        "",
        x,
        y,
        width,
        height,
        id as isize,
        WINDOW_STYLE(LBS_NOTIFY as u32) | WS_VSCROLL,
        true,
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
        true,
    )
}

#[cfg(windows)]
fn create_edit(
    parent: windows::Win32::Foundation::HWND,
    text: &str,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    password: bool,
    id: i32,
) -> Result<windows::Win32::Foundation::HWND> {
    use windows::Win32::UI::WindowsAndMessaging::{ES_AUTOHSCROLL, ES_PASSWORD, WINDOW_STYLE};
    let style = if password {
        ES_AUTOHSCROLL | ES_PASSWORD
    } else {
        ES_AUTOHSCROLL
    };
    create_control(
        parent,
        "EDIT",
        text,
        x,
        y,
        width,
        height,
        id as isize,
        WINDOW_STYLE(style as u32),
        true,
    )
}

#[cfg(windows)]
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
        CreateWindowExW, HMENU, WINDOW_EX_STYLE, WS_BORDER, WS_CHILD, WS_VISIBLE,
    };
    use windows::core::PCWSTR;

    unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(wide(class_name).as_ptr()),
            PCWSTR(wide(text).as_ptr()),
            WS_CHILD
                | WS_VISIBLE
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
        .map_err(|err| AppError::Windows(format!("创建控件失败: {err}")))
    }
}

#[cfg(windows)]
fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(Some(0)).collect()
}
