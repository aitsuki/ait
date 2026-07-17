use crate::error::{AppError, Result};

#[cfg(windows)]
pub const WM_TRAY_COMMAND: u32 = windows::Win32::UI::WindowsAndMessaging::WM_APP + 20;
#[cfg(windows)]
const WM_TRAY_ICON: u32 = windows::Win32::UI::WindowsAndMessaging::WM_APP + 21;
#[cfg(windows)]
pub const TRAY_WINDOW_CLASS_NAME: &str = "ait_tray_window";
#[cfg(windows)]
pub const MENU_SHOW_TRANSLATION_WINDOW: usize = 1001;
#[cfg(windows)]
pub const MENU_SETTINGS: usize = 1002;
#[cfg(windows)]
pub const MENU_OPEN_LOG_DIRECTORY: usize = 1005;
#[cfg(windows)]
pub const MENU_OPEN_LATEST_RELEASE: usize = 1006;
#[cfg(windows)]
pub const MENU_EXIT: usize = 1004;

#[cfg(windows)]
pub struct TrayIcon {
    hwnd: windows::Win32::Foundation::HWND,
    id: u32,
}

#[cfg(windows)]
impl TrayIcon {
    pub fn create() -> Result<Self> {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::UI::Shell::{
            NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NOTIFYICONDATAW, Shell_NotifyIconW,
        };
        use windows::Win32::UI::WindowsAndMessaging::{
            CreateWindowExW, RegisterClassW, WINDOW_EX_STYLE, WINDOW_STYLE, WNDCLASSW,
        };
        use windows::core::PCWSTR;

        let class_name = wide(TRAY_WINDOW_CLASS_NAME);
        unsafe {
            let class = WNDCLASSW {
                lpfnWndProc: Some(tray_wnd_proc),
                lpszClassName: PCWSTR(class_name.as_ptr()),
                ..Default::default()
            };
            let atom = RegisterClassW(&class);
            if atom == 0 {
                return Err(AppError::Windows("注册托盘窗口类失败".to_string()));
            }

            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                PCWSTR(class_name.as_ptr()),
                PCWSTR(wide("ait").as_ptr()),
                WINDOW_STYLE::default(),
                0,
                0,
                0,
                0,
                Some(HWND::default()),
                None,
                None,
                None,
            )
            .map_err(|err| AppError::Windows(format!("创建托盘窗口失败: {err}")))?;

            let icon = load_tray_icon();
            let mut data = NOTIFYICONDATAW {
                cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: hwnd,
                uID: 1,
                uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
                uCallbackMessage: WM_TRAY_ICON,
                hIcon: icon,
                ..Default::default()
            };
            fill_wide_buf(&mut data.szTip, "ait 选区翻译");
            if !Shell_NotifyIconW(NIM_ADD, &data).as_bool() {
                return Err(AppError::Windows("创建托盘图标失败".to_string()));
            }

            tracing::info!("tray icon created");
            Ok(Self { hwnd, id: 1 })
        }
    }
}

#[cfg(windows)]
impl Drop for TrayIcon {
    fn drop(&mut self) {
        use windows::Win32::UI::Shell::{NIM_DELETE, NOTIFYICONDATAW, Shell_NotifyIconW};

        unsafe {
            let data = NOTIFYICONDATAW {
                cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: self.hwnd,
                uID: self.id,
                ..Default::default()
            };
            let _ = Shell_NotifyIconW(NIM_DELETE, &data);
        }
    }
}

#[cfg(windows)]
unsafe extern "system" fn tray_wnd_proc(
    hwnd: windows::Win32::Foundation::HWND,
    msg: u32,
    wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::LRESULT {
    use windows::Win32::Foundation::{LPARAM, LRESULT, POINT, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        AppendMenuW, CreatePopupMenu, DefWindowProcW, DestroyMenu, GetCursorPos, MF_SEPARATOR,
        MF_STRING, PostMessageW, PostQuitMessage, SetForegroundWindow, TPM_RETURNCMD,
        TrackPopupMenu, WM_CLOSE, WM_LBUTTONUP, WM_RBUTTONUP,
    };
    use windows::core::PCWSTR;

    if msg == WM_CLOSE {
        tracing::info!("tray window close requested");
        unsafe {
            PostQuitMessage(0);
        }
        return LRESULT(0);
    }

    if msg == WM_TRAY_ICON && lparam.0 as u32 == WM_LBUTTONUP {
        unsafe {
            let _ = PostMessageW(
                Some(hwnd),
                WM_TRAY_COMMAND,
                WPARAM(MENU_SHOW_TRANSLATION_WINDOW),
                LPARAM(0),
            );
        }
        return LRESULT(0);
    }

    if msg == WM_TRAY_ICON && lparam.0 as u32 == WM_RBUTTONUP {
        let menu = match unsafe { CreatePopupMenu() } {
            Ok(menu) => menu,
            Err(_) => return LRESULT(0),
        };
        unsafe {
            let _ = AppendMenuW(
                menu,
                MF_STRING,
                MENU_SHOW_TRANSLATION_WINDOW,
                PCWSTR(wide("显示翻译窗口").as_ptr()),
            );
            let _ = AppendMenuW(
                menu,
                MF_STRING,
                MENU_SETTINGS,
                PCWSTR(wide("设置").as_ptr()),
            );
            let _ = AppendMenuW(
                menu,
                MF_STRING,
                MENU_OPEN_LOG_DIRECTORY,
                PCWSTR(wide("打开日志目录").as_ptr()),
            );
            let _ = AppendMenuW(
                menu,
                MF_STRING,
                MENU_OPEN_LATEST_RELEASE,
                PCWSTR(wide("打开最新版本页面").as_ptr()),
            );
            let _ = AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null());
            let _ = AppendMenuW(menu, MF_STRING, MENU_EXIT, PCWSTR(wide("退出").as_ptr()));

            let mut point = POINT::default();
            let _ = GetCursorPos(&mut point);
            let _ = SetForegroundWindow(hwnd);
            let selected = TrackPopupMenu(menu, TPM_RETURNCMD, point.x, point.y, None, hwnd, None);
            if selected.0 != 0 {
                let _ = PostMessageW(
                    Some(hwnd),
                    WM_TRAY_COMMAND,
                    WPARAM(selected.0 as usize),
                    LPARAM(0),
                );
            }
            let _ = DestroyMenu(menu);
        }
        return LRESULT(0);
    }

    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

#[cfg(windows)]
unsafe fn load_tray_icon() -> windows::Win32::UI::WindowsAndMessaging::HICON {
    use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSMICON, SM_CYSMICON};

    unsafe {
        crate::ui::icon::load_app_icon(GetSystemMetrics(SM_CXSMICON), GetSystemMetrics(SM_CYSMICON))
    }
}

#[cfg(windows)]
fn fill_wide_buf<const N: usize>(buf: &mut [u16; N], text: &str) {
    let wide = wide(text);
    for (idx, ch) in wide.into_iter().take(N).enumerate() {
        buf[idx] = ch;
    }
}

#[cfg(windows)]
fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(Some(0)).collect()
}

#[cfg(not(windows))]
pub struct TrayIcon;

#[cfg(not(windows))]
impl TrayIcon {
    pub fn create() -> Result<Self> {
        Ok(Self)
    }
}
