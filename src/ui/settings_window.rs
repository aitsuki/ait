use crate::config::{AppSettings, ProviderKind};
use crate::error::{AppError, Result};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsViewModel {
    pub default_provider: ProviderKind,
    pub hotkey: String,
    pub openai_base_url: String,
    pub openai_model: String,
    pub has_openai_key: bool,
    pub clipboard_capture_enabled: bool,
    pub copy_wait_ms: u64,
}

impl From<&AppSettings> for SettingsViewModel {
    fn from(settings: &AppSettings) -> Self {
        Self {
            default_provider: settings.default_provider,
            hotkey: settings.hotkey.clone(),
            openai_base_url: settings.openai.base_url.clone(),
            openai_model: settings.openai.model.clone(),
            has_openai_key: settings.openai.encrypted_api_key.is_some(),
            clipboard_capture_enabled: settings.clipboard_capture.enabled,
            copy_wait_ms: settings.clipboard_capture.copy_wait_ms,
        }
    }
}

#[cfg(windows)]
pub struct SettingsWindow;

#[cfg(windows)]
impl SettingsWindow {
    pub fn open(settings: &AppSettings) -> Result<()> {
        use windows::core::PCWSTR;
        use windows::Win32::Foundation::HWND;
        use windows::Win32::UI::WindowsAndMessaging::{
            CreateWindowExW, LoadCursorW, RegisterClassW, SetWindowLongPtrW, ShowWindow,
            CW_USEDEFAULT, GWLP_USERDATA, IDC_ARROW, SW_SHOW, WNDCLASSW, WINDOW_EX_STYLE,
            WS_CAPTION, WS_OVERLAPPED, WS_SYSMENU,
        };

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
            if atom == 0 {
                return Err(AppError::Windows("注册设置窗口类失败".to_string()));
            }

            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                PCWSTR(class_name.as_ptr()),
                PCWSTR(wide("ait 设置").as_ptr()),
                WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                520,
                360,
                Some(HWND::default()),
                None,
                None,
                None,
            )
            .map_err(|err| AppError::Windows(format!("创建设置窗口失败: {err}")))?;
            let settings_ptr = Box::into_raw(Box::new(settings.clone()));
            let _ = SetWindowLongPtrW(hwnd, GWLP_USERDATA, settings_ptr as isize);

            let provider = match view_model.default_provider {
                ProviderKind::GoogleFree => "google_free",
                ProviderKind::OpenAiCompatible => "openai_compatible",
            };
            create_static(hwnd, "默认提供方", 18, 20, 120, 22)?;
            create_edit(hwnd, provider, 150, 18, 230, 24, false, ID_PROVIDER)?;
            create_static(hwnd, "快捷键", 18, 54, 120, 22)?;
            create_edit(hwnd, &view_model.hotkey, 150, 52, 230, 24, false, ID_HOTKEY)?;
            create_static(hwnd, "OpenAI Base URL", 18, 88, 120, 22)?;
            create_edit(
                hwnd,
                &view_model.openai_base_url,
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
                &view_model.openai_model,
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
                if view_model.has_openai_key { "已保存" } else { "" },
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

#[cfg(windows)]
unsafe extern "system" fn default_wnd_proc(
    hwnd: windows::Win32::Foundation::HWND,
    msg: u32,
    wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::LRESULT {
    use windows::Win32::Foundation::LRESULT;
    use windows::Win32::UI::WindowsAndMessaging::{
        DefWindowProcW, DestroyWindow, GetWindowLongPtrW, SetWindowLongPtrW, GWLP_USERDATA,
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
            if let Err(err) = unsafe { save_settings_from_window(hwnd) } {
                tracing::warn!(error = %err, "save settings failed");
            }
            unsafe {
                let _ = DestroyWindow(hwnd);
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
    use windows::Win32::UI::WindowsAndMessaging::{GetWindowLongPtrW, GWLP_USERDATA};

    let ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) };
    if ptr == 0 {
        return Err(AppError::Config("设置窗口状态缺失".to_string()));
    }
    let settings = unsafe { &mut *(ptr as *mut AppSettings) };
    settings.default_provider = match read_control_text(hwnd, ID_PROVIDER)?.as_str() {
        "openai_compatible" => ProviderKind::OpenAiCompatible,
        _ => ProviderKind::GoogleFree,
    };
    settings.hotkey = read_control_text(hwnd, ID_HOTKEY)?;
    settings.openai.base_url = read_control_text(hwnd, ID_BASE_URL)?;
    settings.openai.model = read_control_text(hwnd, ID_MODEL)?;
    settings.clipboard_capture.copy_wait_ms = read_control_text(hwnd, ID_COPY_WAIT)?
        .parse::<u64>()
        .unwrap_or(settings.clipboard_capture.copy_wait_ms);

    let api_key = read_control_text(hwnd, ID_API_KEY)?;
    if !api_key.trim().is_empty() && api_key != "已保存" {
        settings.openai.encrypted_api_key =
            Some(crate::secret::SecretStore::new("ait-openai-api-key").protect(&api_key)?);
    }

    let dir = crate::config::SettingsStore::default_dir()?;
    crate::config::SettingsStore::new(dir).save(settings)?;
    tracing::info!(
        provider = ?settings.default_provider,
        has_openai_key = settings.openai.encrypted_api_key.is_some(),
        "settings saved"
    );
    Ok(())
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
    create_control(parent, "STATIC", text, x, y, width, height, 0, Default::default())
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
    use windows::Win32::UI::WindowsAndMessaging::{WINDOW_STYLE, BS_PUSHBUTTON};
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
    use windows::core::PCWSTR;
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, HMENU, WINDOW_EX_STYLE, WS_BORDER, WS_CHILD, WS_VISIBLE,
    };

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
