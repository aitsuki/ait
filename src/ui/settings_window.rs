use crate::config::{AppSettings, TranslatorProvider};
use crate::error::{AppError, Result};

const SETTINGS_WINDOW_WIDTH: i32 = 720;
const SETTINGS_WINDOW_HEIGHT: i32 = 460;

#[cfg(windows)]
const ID_PROFILE_LIST: i32 = 3101;
#[cfg(windows)]
const ID_NAME: i32 = 3102;
#[cfg(windows)]
const ID_PROVIDER: i32 = 3103;
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
const ID_COPY_WAIT: i32 = 3109;
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

const PROVIDER_OPTIONS: [TranslatorProvider; 6] = [
    TranslatorProvider::Google,
    TranslatorProvider::OpenAi,
    TranslatorProvider::Claude,
    TranslatorProvider::Gemini,
    TranslatorProvider::DeepSeek,
    TranslatorProvider::Custom,
];

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
    pub api_key: Option<String>,
    pub timeout_secs: u64,
    pub hotkey: String,
    pub copy_wait_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsEditAction {
    NewProfile,
    DeleteProfile(String),
    SetDefault(String),
    SelectProfile(String),
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
    settings.clipboard_capture.copy_wait_ms = update.copy_wait_ms;

    let profile = settings
        .profile_by_id_mut(&update.id)
        .ok_or_else(|| AppError::Config("翻译配置不存在".to_string()))?;
    profile.provider = update.provider;
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
        if let Some(api_key) = update.api_key.filter(|value| !value.trim().is_empty()) {
            profile.encrypted_api_key = Some(api_key);
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
                base_url: selected.base_url.clone(),
                model: selected.model.clone(),
                has_api_key: selected.encrypted_api_key.is_some(),
                timeout_secs: selected.timeout_secs,
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
            GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromPoint,
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

            create_static(hwnd, "翻译配置", 18, 18, 120, 22)?;
            let profile_list = create_listbox(hwnd, 18, 44, 220, 284, ID_PROFILE_LIST)?;
            reset_profile_list(profile_list, &view_model)?;
            create_button(hwnd, "新增", 18, 342, 64, 28, ID_NEW_PROFILE)?;
            create_button(hwnd, "删除", 90, 342, 64, 28, ID_DELETE_PROFILE)?;
            create_button(hwnd, "设为默认", 162, 342, 76, 28, ID_SET_DEFAULT)?;

            create_static(hwnd, "名称", 266, 20, 90, 22)?;
            create_edit(hwnd, &view_model.selected_profile.name, 370, 18, 240, 24, false, ID_NAME)?;
            create_static(hwnd, "供应商", 266, 54, 90, 22)?;
            let provider_combo = create_provider_combo(hwnd, 370, 52, 180, 180, ID_PROVIDER)?;
            select_provider(provider_combo, view_model.selected_profile.provider)?;
            create_static(hwnd, "Base URL", 266, 88, 90, 22)?;
            create_edit(hwnd, &view_model.selected_profile.base_url, 370, 86, 300, 24, false, ID_BASE_URL)?;
            create_static(hwnd, "模型", 266, 122, 90, 22)?;
            create_edit(hwnd, &view_model.selected_profile.model, 370, 120, 240, 24, false, ID_MODEL)?;
            create_static(hwnd, "API Key", 266, 156, 90, 22)?;
            create_edit(
                hwnd,
                if view_model.selected_profile.has_api_key { "已保存" } else { "" },
                370,
                154,
                240,
                24,
                true,
                ID_API_KEY,
            )?;
            create_static(hwnd, "超时秒数", 266, 190, 90, 22)?;
            create_edit(
                hwnd,
                &view_model.selected_profile.timeout_secs.to_string(),
                370,
                188,
                90,
                24,
                false,
                ID_TIMEOUT,
            )?;
            create_static(hwnd, "快捷键", 266, 236, 90, 22)?;
            create_edit(hwnd, &view_model.hotkey, 370, 234, 180, 24, false, ID_HOTKEY)?;
            create_static(hwnd, "复制等待毫秒", 266, 270, 90, 22)?;
            create_edit(hwnd, &view_model.copy_wait_ms.to_string(), 370, 268, 90, 24, false, ID_COPY_WAIT)?;
            create_static(hwnd, "Google 配置使用免 Key 翻译，网络字段不会保存。", 266, 314, 390, 36)?;
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
            if let Err(err) = unsafe { edit_settings_profiles(hwnd, SettingsEditAction::NewProfile) }
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
                    let _ = DestroyWindow(hwnd);
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
    let api_key = read_control_text(hwnd, ID_API_KEY)?;
    let encrypted_api_key = if !api_key.trim().is_empty() && api_key != "已保存" {
        Some(
            crate::secret::SecretStore::new(&format!("ait-translator-profile-{profile_id}"))
                .protect(&api_key)?,
        )
    } else {
        None
    };
    apply_settings_detail_update(
        settings,
        SettingsProfileDetailUpdate {
            id: profile_id,
            name: read_control_text(hwnd, ID_NAME)?,
            provider: selected_provider(hwnd)?,
            base_url: read_control_text(hwnd, ID_BASE_URL)?,
            model: read_control_text(hwnd, ID_MODEL)?,
            api_key: encrypted_api_key,
            timeout_secs: read_control_text(hwnd, ID_TIMEOUT)?.parse::<u64>().unwrap_or(30),
            hotkey: read_control_text(hwnd, ID_HOTKEY)?,
            copy_wait_ms: read_control_text(hwnd, ID_COPY_WAIT)?
                .parse::<u64>()
                .unwrap_or(settings.clipboard_capture.copy_wait_ms),
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
    select_provider(control(hwnd, ID_PROVIDER)?, profile.provider)?;
    set_control_text(hwnd, ID_BASE_URL, &profile.base_url)?;
    set_control_text(hwnd, ID_MODEL, &profile.model)?;
    set_control_text(
        hwnd,
        ID_API_KEY,
        if profile.has_api_key { "已保存" } else { "" },
    )?;
    set_control_text(hwnd, ID_TIMEOUT, &profile.timeout_secs.to_string())?;
    set_control_text(hwnd, ID_HOTKEY, &vm.hotkey)?;
    set_control_text(hwnd, ID_COPY_WAIT, &vm.copy_wait_ms.to_string())?;
    set_network_fields_enabled(hwnd, profile.network_fields_enabled);
    Ok(())
}

#[cfg(windows)]
fn refresh_profile_list(hwnd: windows::Win32::Foundation::HWND, settings: &AppSettings) -> Result<()> {
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
    use windows::Win32::UI::WindowsAndMessaging::{SendMessageW, LB_GETCURSEL};

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
fn selected_provider(hwnd: windows::Win32::Foundation::HWND) -> Result<TranslatorProvider> {
    use windows::Win32::Foundation::{LPARAM, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{SendMessageW, CB_GETCURSEL};

    let combo = control(hwnd, ID_PROVIDER)?;
    let index = unsafe { SendMessageW(combo, CB_GETCURSEL, Some(WPARAM(0)), Some(LPARAM(0))) }.0;
    PROVIDER_OPTIONS
        .get(index as usize)
        .copied()
        .ok_or_else(|| AppError::Config("未选择供应商".to_string()))
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
fn set_control_text(
    hwnd: windows::Win32::Foundation::HWND,
    id: i32,
    text: &str,
) -> Result<()> {
    use windows::Win32::UI::WindowsAndMessaging::SetWindowTextW;

    let child = control(hwnd, id)?;
    unsafe {
        SetWindowTextW(child, windows::core::PCWSTR(wide(text).as_ptr()))
            .map_err(|err| AppError::Windows(format!("设置控件文本失败: {err}")))?;
    }
    Ok(())
}

#[cfg(windows)]
fn set_network_fields_enabled(hwnd: windows::Win32::Foundation::HWND, enabled: bool) {
    use windows::Win32::UI::Input::KeyboardAndMouse::EnableWindow;

    for id in [ID_BASE_URL, ID_MODEL, ID_API_KEY, ID_TIMEOUT] {
        if let Ok(child) = control(hwnd, id) {
            unsafe {
                let _ = EnableWindow(child, enabled);
            }
        }
    }
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
        SendMessageW, LB_ADDSTRING, LB_RESETCONTENT, LB_SETCURSEL,
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
fn select_provider(
    combo: windows::Win32::Foundation::HWND,
    provider: TranslatorProvider,
) -> Result<()> {
    use windows::Win32::Foundation::{LPARAM, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{SendMessageW, CB_SETCURSEL};

    let index = PROVIDER_OPTIONS
        .iter()
        .position(|item| *item == provider)
        .ok_or_else(|| AppError::Config("供应商不存在".to_string()))?;
    unsafe {
        let _ = SendMessageW(combo, CB_SETCURSEL, Some(WPARAM(index)), Some(LPARAM(0)));
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
    )
}

#[cfg(windows)]
fn create_provider_combo(
    parent: windows::Win32::Foundation::HWND,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    id: i32,
) -> Result<windows::Win32::Foundation::HWND> {
    use windows::Win32::Foundation::{LPARAM, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        SendMessageW, CB_ADDSTRING, CBS_DROPDOWNLIST, WINDOW_STYLE,
    };

    let combo = create_control(
        parent,
        "COMBOBOX",
        "",
        x,
        y,
        width,
        height,
        id as isize,
        WINDOW_STYLE(CBS_DROPDOWNLIST as u32),
    )?;
    unsafe {
        for provider in PROVIDER_OPTIONS {
            let label = wide(provider.display_name());
            let _ = SendMessageW(
                combo,
                CB_ADDSTRING,
                Some(WPARAM(0)),
                Some(LPARAM(label.as_ptr() as isize)),
            );
        }
    }
    Ok(combo)
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
        .map_err(|err| AppError::Windows(format!("创建控件失败: {err}")))
    }
}

#[cfg(windows)]
fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(Some(0)).collect()
}
