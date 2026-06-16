use crate::error::{AppError, Result};

#[cfg(windows)]
const ID_SOURCE_EDIT: isize = 2101;
#[cfg(windows)]
const ID_TRANSLATED_EDIT: isize = 2102;
#[cfg(windows)]
const ID_TRANSLATE: usize = 2001;
#[cfg(windows)]
pub const WM_TRANSLATE_WINDOW_SOURCE: u32 = windows::Win32::UI::WindowsAndMessaging::WM_APP + 30;

#[derive(Debug, Clone)]
pub struct TranslationWindowState {
    pub source_text: String,
    pub translated_text: String,
    pub loading: bool,
    pub error: Option<String>,
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
}

pub fn window_z_order() -> WindowZOrder {
    WindowZOrder::NotTopmost
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
            create_button(hwnd, "翻译", 534, 322, 52, 28, ID_TRANSLATE as isize)?;
            install_edit_subclass(source_edit)?;
            install_edit_subclass(translated_edit)?;

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
        show_window_at_cursor(self.hwnd, ShowMode::Loading);
        tracing::info!("show translation window loading state");
        Ok(())
    }

    pub fn show_starting(&mut self) -> Result<()> {
        self.state.source_text.clear();
        self.state.translated_text.clear();
        self.state.loading = true;
        self.state.error = None;
        set_text(self.source_edit, "")?;
        set_text(self.translated_edit, "")?;
        set_text(self.status_text, "正在取词...")?;
        show_window_at_cursor(self.hwnd, ShowMode::Starting);
        tracing::info!("show translation window starting state");
        Ok(())
    }

    pub fn show_result(&mut self, translated_text: String) -> Result<()> {
        self.state.translated_text = translated_text;
        self.state.loading = false;
        self.state.error = None;
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
        self.show_result(result.translated_text.clone())
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
    use windows::Win32::UI::WindowsAndMessaging::{
        DefWindowProcW, PostMessageW, SW_HIDE, ShowWindow, WM_CLOSE, WM_COMMAND, WM_KEYDOWN,
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
        match command {
            ID_TRANSLATE => unsafe {
                let _ = PostMessageW(Some(hwnd), WM_TRANSLATE_WINDOW_SOURCE, WPARAM(0), LPARAM(0));
                return LRESULT(0);
            },
            _ => {}
        }
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
    _ref_data: usize,
) -> windows::Win32::Foundation::LRESULT {
    use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
    use windows::Win32::UI::Controls::EM_SETSEL;
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetKeyState, VK_CONTROL};
    use windows::Win32::UI::Shell::DefSubclassProc;
    use windows::Win32::UI::WindowsAndMessaging::{
        GetParent, PostMessageW, SendMessageW, WM_CHAR, WM_CLOSE, WM_KEYDOWN,
    };

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

    unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) }
}

#[cfg(windows)]
fn install_edit_subclass(hwnd: windows::Win32::Foundation::HWND) -> Result<()> {
    use windows::Win32::UI::Shell::SetWindowSubclass;

    unsafe {
        if SetWindowSubclass(hwnd, Some(edit_subclass_proc), 1, 0).as_bool() {
            Ok(())
        } else {
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
fn show_window_at_cursor(hwnd: windows::Win32::Foundation::HWND, mode: ShowMode) {
    use windows::Win32::Foundation::{POINT, RECT};
    use windows::Win32::Graphics::Gdi::{
        GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromPoint,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        GetCursorPos, GetWindowRect, HWND_NOTOPMOST, SET_WINDOW_POS_FLAGS, SW_SHOW,
        SW_SHOWNOACTIVATE, SWP_NOACTIVATE, SWP_SHOWWINDOW, SetForegroundWindow, SetWindowPos,
        ShowWindow,
    };

    unsafe {
        match show_action(is_window_visible(hwnd), is_foreground_window(hwnd)) {
            ShowAction::PositionAndActivate => {}
            ShowAction::ActivateOnly => {
                if mode.activates_window() {
                    let _ = ShowWindow(hwnd, SW_SHOW);
                    let _ = SetForegroundWindow(hwnd);
                } else {
                    let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
                }
                return;
            }
            ShowAction::KeepPosition => {
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
        let _ = SetWindowPos(
            hwnd,
            Some(HWND_NOTOPMOST),
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
