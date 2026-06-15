use crate::error::{AppError, Result};

#[cfg(windows)]
const ID_SOURCE_EDIT: isize = 2101;
#[cfg(windows)]
const ID_TRANSLATED_EDIT: isize = 2102;
#[cfg(windows)]
const ID_COPY: usize = 2001;
#[cfg(windows)]
const ID_RETRY: usize = 2002;
#[cfg(windows)]
const ID_SETTINGS: usize = 2003;
#[cfg(windows)]
const ID_CLOSE: usize = 2004;

#[derive(Debug, Clone)]
pub struct TranslationWindowState {
    pub source_text: String,
    pub translated_text: String,
    pub loading: bool,
    pub error: Option<String>,
}

#[cfg(windows)]
pub struct TranslationWindow {
    hwnd: windows::Win32::Foundation::HWND,
    source_edit: windows::Win32::Foundation::HWND,
    translated_edit: windows::Win32::Foundation::HWND,
    status_text: windows::Win32::Foundation::HWND,
    state: TranslationWindowState,
}

#[cfg(windows)]
impl TranslationWindow {
    pub fn new() -> Result<Self> {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::UI::WindowsAndMessaging::{
            CW_USEDEFAULT, CreateWindowExW, IDC_ARROW, LoadCursorW, RegisterClassW,
            WINDOW_EX_STYLE, WNDCLASSW, WS_CAPTION, WS_OVERLAPPED, WS_SYSMENU, WS_THICKFRAME,
        };
        use windows::core::PCWSTR;

        let class_name = wide("ait_translation_window");
        unsafe {
            let class = WNDCLASSW {
                lpfnWndProc: Some(default_wnd_proc),
                lpszClassName: PCWSTR(class_name.as_ptr()),
                hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
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
                WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_THICKFRAME,
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

            create_static(hwnd, "原文", 16, 14, 80, 20)?;
            let source_edit = create_edit(hwnd, 16, 38, 572, 96, ID_SOURCE_EDIT, false)?;
            create_static(hwnd, "译文", 16, 146, 80, 20)?;
            let translated_edit = create_edit(hwnd, 16, 170, 572, 140, ID_TRANSLATED_EDIT, true)?;
            let status_text = create_static(hwnd, "", 16, 324, 360, 22)?;
            create_button(hwnd, "复制译文", 388, 322, 82, 28, ID_COPY as isize)?;
            create_button(hwnd, "重试", 476, 322, 52, 28, ID_RETRY as isize)?;
            create_button(hwnd, "设置", 534, 322, 52, 28, ID_SETTINGS as isize)?;
            create_button(hwnd, "关闭", 534, 354, 52, 28, ID_CLOSE as isize)?;

            Ok(Self {
                hwnd,
                source_edit,
                translated_edit,
                status_text,
                state: TranslationWindowState {
                    source_text: String::new(),
                    translated_text: String::new(),
                    loading: false,
                    error: None,
                },
            })
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
        show_window_at_cursor_and_raise(self.hwnd);
        tracing::info!("show translation window loading state");
        Ok(())
    }

    pub fn show_result(&mut self, translated_text: String) -> Result<()> {
        self.state.translated_text = translated_text;
        self.state.loading = false;
        self.state.error = None;
        set_text(self.translated_edit, &self.state.translated_text)?;
        set_text(self.status_text, "翻译完成")?;
        show_window_at_cursor_and_raise(self.hwnd);
        tracing::info!("show translation window result");
        Ok(())
    }

    pub fn show_error(&mut self, message: String) -> Result<()> {
        self.state.loading = false;
        self.state.error = Some(message);
        let message = self.state.error.as_deref().unwrap_or("翻译失败");
        set_text(self.status_text, message)?;
        show_window_at_cursor_and_raise(self.hwnd);
        tracing::info!("show translation window error");
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
    use crate::capture::ClipboardBackend;
    use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        DefWindowProcW, GetDlgItemTextW, PostMessageW, SW_HIDE, ShowWindow, WM_CLOSE, WM_COMMAND,
    };

    if msg == WM_CLOSE {
        unsafe {
            let _ = ShowWindow(hwnd, SW_HIDE);
        }
        return LRESULT(0);
    }
    if msg == WM_COMMAND {
        let command = wparam.0 & 0xffff;
        match command {
            ID_CLOSE => unsafe {
                let _ = ShowWindow(hwnd, SW_HIDE);
                return LRESULT(0);
            },
            ID_COPY => {
                let mut buf = [0u16; 8192];
                let len =
                    unsafe { GetDlgItemTextW(hwnd, ID_TRANSLATED_EDIT as i32, &mut buf) } as usize;
                let text = String::from_utf16_lossy(&buf[..len]);
                if !text.trim().is_empty() {
                    let backend = crate::capture::WindowsClipboardBackend;
                    if let Err(err) = backend.write_text(&text) {
                        tracing::warn!(error = %err, "copy translated text failed");
                    }
                }
                return LRESULT(0);
            }
            ID_RETRY => unsafe {
                let _ = PostMessageW(
                    Some(hwnd),
                    crate::ui::tray::WM_TRAY_COMMAND,
                    WPARAM(crate::ui::tray::MENU_TRANSLATE_SELECTION),
                    LPARAM(0),
                );
                return LRESULT(0);
            },
            ID_SETTINGS => unsafe {
                let _ = PostMessageW(
                    Some(hwnd),
                    crate::ui::tray::WM_TRAY_COMMAND,
                    WPARAM(crate::ui::tray::MENU_SETTINGS),
                    LPARAM(0),
                );
                return LRESULT(0);
            },
            _ => {}
        }
    }

    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
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
fn set_text(hwnd: windows::Win32::Foundation::HWND, text: &str) -> Result<()> {
    use windows::Win32::UI::WindowsAndMessaging::SetWindowTextW;
    use windows::core::PCWSTR;

    unsafe {
        SetWindowTextW(hwnd, PCWSTR(wide(text).as_ptr()))
            .map_err(|err| AppError::Windows(format!("设置窗口文本失败: {err}")))
    }
}

#[cfg(windows)]
fn show_window_at_cursor_and_raise(hwnd: windows::Win32::Foundation::HWND) {
    use windows::Win32::Foundation::{POINT, RECT};
    use windows::Win32::Graphics::Gdi::{
        GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromPoint,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        GetCursorPos, GetWindowRect, HWND_TOPMOST, SET_WINDOW_POS_FLAGS, SW_SHOW, SWP_SHOWWINDOW,
        SetForegroundWindow, SetWindowPos, ShowWindow,
    };

    unsafe {
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
        let _ = SetWindowPos(
            hwnd,
            Some(HWND_TOPMOST),
            x,
            y,
            width,
            height,
            SET_WINDOW_POS_FLAGS(SWP_SHOWWINDOW.0),
        );
        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = SetForegroundWindow(hwnd);
    }
}

#[cfg(windows)]
fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(Some(0)).collect()
}
