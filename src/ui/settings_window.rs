use crate::config::{AppSettings, TranslatorProvider};
use crate::error::{AppError, Result};

const SETTINGS_WINDOW_WIDTH: i32 = 520;
const SETTINGS_WINDOW_HEIGHT: i32 = 360;

#[cfg(windows)]
const ID_PROVIDER: i32 = 3101;
#[cfg(windows)]
const ID_HOTKEY: i32 = 3102;
#[cfg(windows)]
const ID_BASE_URL: i32 = 3103;
#[cfg(windows)]
const ID_MODEL: i32 = 3104;
#[cfg(windows)]
const ID_API_KEY: i32 = 3105;
#[cfg(windows)]
const ID_COPY_WAIT: i32 = 3106;
#[cfg(windows)]
const ID_SAVE: isize = 3001;
#[cfg(windows)]
const ID_CANCEL: isize = 3002;
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
    pub network_fields_enabled: bool,
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

impl SettingsViewModel {
    pub fn from_settings_with_selected(settings: &AppSettings, selected_profile_id: &str) -> Self {
        let selected = settings
            .profile_by_id(selected_profile_id)
            .or_else(|| settings.profile_by_id(&settings.default_profile_id))
            .or_else(|| settings.translator_profiles.first())
            .expect("settings always contain profiles after normalization");
        Self {
            profiles: settings
                .translator_profiles
                .iter()
                .map(|profile| SettingsProfileListItem {
                    id: profile.id.clone(),
                    label: profile.name.clone(),
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
                network_fields_enabled: selected.provider != TranslatorProvider::Google,
            },
            hotkey: settings.hotkey.clone(),
            clipboard_capture_enabled: settings.clipboard_capture.enabled,
            copy_wait_ms: settings.clipboard_capture.copy_wait_ms,
        }
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

            create_static(hwnd, "默认配置 ID", 18, 20, 120, 22)?;
            create_edit(
                hwnd,
                &view_model.selected_profile.id,
                150,
                18,
                230,
                24,
                false,
                ID_PROVIDER,
            )?;
            create_static(hwnd, "快捷键", 18, 54, 120, 22)?;
            create_edit(hwnd, &view_model.hotkey, 150, 52, 230, 24, false, ID_HOTKEY)?;
            create_static(hwnd, "OpenAI Base URL", 18, 88, 120, 22)?;
            create_edit(
                hwnd,
                &view_model.selected_profile.base_url,
                150,
                86,
                320,
                24,
                false,
                ID_BASE_URL,
            )?;
            create_static(hwnd, "OpenAI Model", 18, 122, 120, 22)?;
            create_edit(
                hwnd,
                &view_model.selected_profile.model,
                150,
                120,
                230,
                24,
                false,
                ID_MODEL,
            )?;
            create_static(hwnd, "API Key", 18, 156, 120, 22)?;
            create_edit(
                hwnd,
                if view_model.selected_profile.has_api_key {
                    "已保存"
                } else {
                    ""
                },
                150,
                154,
                230,
                24,
                true,
                ID_API_KEY,
            )?;
            create_static(hwnd, "复制等待毫秒", 18, 190, 120, 22)?;
            create_edit(
                hwnd,
                &view_model.copy_wait_ms.to_string(),
                150,
                188,
                120,
                24,
                false,
                ID_COPY_WAIT,
            )?;
            create_static(
                hwnd,
                "Google 非官方免 Key 翻译不是 Google Cloud Translation API，可能失效或限流。",
                18,
                228,
                460,
                42,
            )?;
            create_button(hwnd, "保存", 318, 282, 72, 28, ID_SAVE)?;
            create_button(hwnd, "取消", 398, 282, 72, 28, ID_CANCEL)?;
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
                        let text = wide(&err.to_string());
                        let caption = wide("保存失败");
                        let _ = windows::Win32::UI::WindowsAndMessaging::MessageBoxW(
                            Some(hwnd),
                            windows::core::PCWSTR(text.as_ptr()),
                            windows::core::PCWSTR(caption.as_ptr()),
                            windows::Win32::UI::WindowsAndMessaging::MB_OK,
                        );
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
    let profile_id = read_control_text(hwnd, ID_PROVIDER)?;
    settings.hotkey = read_control_text(hwnd, ID_HOTKEY)?;
    settings.clipboard_capture.copy_wait_ms = read_control_text(hwnd, ID_COPY_WAIT)?
        .parse::<u64>()
        .unwrap_or(settings.clipboard_capture.copy_wait_ms);

    let profile = settings
        .profile_by_id_mut(&profile_id)
        .ok_or_else(|| AppError::Config("翻译配置不存在".to_string()))?;
    if profile.provider == TranslatorProvider::Google {
        profile.base_url.clear();
        profile.model.clear();
        profile.encrypted_api_key = None;
        profile.timeout_secs = 0;
    } else {
        profile.base_url = read_control_text(hwnd, ID_BASE_URL)?;
        profile.model = read_control_text(hwnd, ID_MODEL)?;
        let api_key = read_control_text(hwnd, ID_API_KEY)?;
        if !api_key.trim().is_empty() && api_key != "已保存" {
            profile.encrypted_api_key = Some(
                crate::secret::SecretStore::new(&format!("ait-translator-profile-{}", profile.id))
                    .protect(&api_key)?,
            );
        }
    }

    let dir = crate::config::SettingsStore::default_dir()?;
    crate::config::SettingsStore::new(dir).save(settings)?;
    tracing::info!(
        default_profile_id = %settings.default_profile_id,
        "settings saved"
    );
    Ok(())
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
