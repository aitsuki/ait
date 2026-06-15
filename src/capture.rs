use crate::error::{AppError, Result};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapturedText {
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureErrorKind {
    NoText,
    ClipboardUnavailable,
    CopyFailed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureError {
    pub kind: CaptureErrorKind,
    pub message: String,
}

impl std::fmt::Display for CaptureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for CaptureError {}

pub trait ClipboardBackend {
    fn read_text(&self) -> Result<Option<String>>;
    fn write_text(&self, text: &str) -> Result<()>;
    fn sequence_number(&self) -> Result<u32>;
}

pub trait SelectionBackend {
    fn read_selected_text(&self) -> Result<Option<String>>;
}

pub struct NoSelectionBackend;

impl SelectionBackend for NoSelectionBackend {
    fn read_selected_text(&self) -> Result<Option<String>> {
        Ok(None)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CopyAction {
    ReleaseCtrl,
    ReleaseAlt,
    ReleaseShift,
    ReleaseWin,
    ReleaseTab,
    ReleaseEscape,
    ReleaseCapsLock,
    ReleaseC,
    PressCtrl,
    PressC,
}

pub trait CopyBackend {
    fn send_copy(&self) -> Result<()>;
}

pub struct NoCopyBackend;

impl CopyBackend for NoCopyBackend {
    fn send_copy(&self) -> Result<()> {
        Ok(())
    }
}

pub struct CaptureService<B, S = NoSelectionBackend, C = NoCopyBackend> {
    backend: B,
    selection: S,
    copy: C,
    copy_wait: Duration,
}

impl<B: ClipboardBackend> CaptureService<B> {
    pub fn new(backend: B, copy_wait: Duration) -> Self {
        Self {
            backend,
            selection: NoSelectionBackend,
            copy: NoCopyBackend,
            copy_wait,
        }
    }
}

impl<B, S, C> CaptureService<B, S, C>
where
    B: ClipboardBackend,
    S: SelectionBackend,
    C: CopyBackend,
{
    pub fn with_selection<NextSelection>(
        self,
        selection: NextSelection,
    ) -> CaptureService<B, NextSelection, C>
    where
        NextSelection: SelectionBackend,
    {
        CaptureService {
            backend: self.backend,
            selection,
            copy: self.copy,
            copy_wait: self.copy_wait,
        }
    }

    pub fn with_copy<NextCopy>(self, copy: NextCopy) -> CaptureService<B, S, NextCopy>
    where
        NextCopy: CopyBackend,
    {
        CaptureService {
            backend: self.backend,
            selection: self.selection,
            copy,
            copy_wait: self.copy_wait,
        }
    }

    pub fn backend(&self) -> &B {
        &self.backend
    }

    pub fn copy_backend(&self) -> &C {
        &self.copy
    }

    pub fn capture_selected_text(&self) -> std::result::Result<CapturedText, CaptureError> {
        match self.selection.read_selected_text() {
            Ok(Some(text)) if !text.trim().is_empty() => {
                tracing::debug!(
                    strategy = "uia_focused_selection",
                    text_len = text.chars().count(),
                    "captured selected text"
                );
                return Ok(CapturedText { text });
            }
            Ok(_) => {
                tracing::debug!(
                    strategy = "uia_focused_selection",
                    "selection backend returned no text"
                );
            }
            Err(err) => {
                tracing::debug!(
                    strategy = "uia_focused_selection",
                    error = %err,
                    "selection backend failed; falling back to clipboard copy"
                );
            }
        }

        let previous = self.read_clipboard_with_retry()?;
        if previous.is_some() {
            self.backend.write_text("").map_err(to_capture_error)?;
        }
        let sequence_before = self.backend.sequence_number().map_err(to_capture_error)?;
        self.copy.send_copy().map_err(|err| CaptureError {
            kind: CaptureErrorKind::CopyFailed,
            message: err.to_string(),
        })?;

        let sequence_changed = self.wait_for_clipboard_sequence_change(sequence_before)?;
        let copied = if sequence_changed {
            self.read_clipboard_with_retry()?
        } else {
            None
        };
        if let Some(old) = previous {
            let _ = self.backend.write_text(&old);
        }

        if !sequence_changed {
            return Err(CaptureError {
                kind: CaptureErrorKind::CopyFailed,
                message: "复制后剪贴板没有变化".to_string(),
            });
        }

        let text = copied.unwrap_or_default();
        if text.trim().is_empty() {
            return Err(CaptureError {
                kind: CaptureErrorKind::NoText,
                message: "没有取到选中文本".to_string(),
            });
        }

        tracing::debug!(
            strategy = "clipboard_copy",
            text_len = text.chars().count(),
            "captured selected text"
        );
        Ok(CapturedText { text })
    }

    fn read_clipboard_with_retry(&self) -> std::result::Result<Option<String>, CaptureError> {
        let deadline = Instant::now() + self.copy_wait;
        loop {
            match self.backend.read_text() {
                Ok(text) => return Ok(text),
                Err(err) if Instant::now() >= deadline => return Err(to_capture_error(err)),
                Err(_) => {}
            }
            if Instant::now() >= deadline {
                return Err(to_capture_error(AppError::Capture(
                    "读取剪贴板失败".to_string(),
                )));
            }
            thread::sleep(Duration::from_millis(10));
        }
    }

    fn wait_for_clipboard_sequence_change(
        &self,
        previous_sequence: u32,
    ) -> std::result::Result<bool, CaptureError> {
        let deadline = Instant::now() + self.copy_wait;
        loop {
            match self.backend.sequence_number() {
                Ok(sequence) if sequence != previous_sequence => return Ok(true),
                Ok(_) => {}
                Err(err) => {
                    tracing::debug!(
                        error = %err,
                        "clipboard sequence read failed while waiting for copied text"
                    );
                }
            }
            if Instant::now() >= deadline {
                return Ok(false);
            }
            thread::sleep(Duration::from_millis(10));
        }
    }
}

fn to_capture_error(err: AppError) -> CaptureError {
    CaptureError {
        kind: CaptureErrorKind::ClipboardUnavailable,
        message: err.to_string(),
    }
}

#[cfg(windows)]
pub struct WindowsClipboardBackend;

#[cfg(windows)]
impl ClipboardBackend for WindowsClipboardBackend {
    fn read_text(&self) -> Result<Option<String>> {
        use windows::Win32::Foundation::HGLOBAL;
        use windows::Win32::System::DataExchange::{
            CloseClipboard, GetClipboardData, IsClipboardFormatAvailable, OpenClipboard,
        };
        use windows::Win32::System::Memory::{GlobalLock, GlobalUnlock};

        const CF_UNICODETEXT_FORMAT: u32 = 13;

        unsafe {
            if IsClipboardFormatAvailable(CF_UNICODETEXT_FORMAT).is_err() {
                return Ok(None);
            }
            OpenClipboard(None)
                .map_err(|err| AppError::Capture(format!("打开剪贴板失败: {err}")))?;
            let handle = GetClipboardData(CF_UNICODETEXT_FORMAT).map_err(|err| {
                let _ = CloseClipboard();
                AppError::Capture(format!("读取剪贴板失败: {err}"))
            })?;
            let global = HGLOBAL(handle.0);
            let ptr = GlobalLock(global);
            if ptr.is_null() {
                let _ = CloseClipboard();
                return Err(AppError::Capture("锁定剪贴板数据失败".to_string()));
            }
            let mut len = 0usize;
            let wide = ptr as *const u16;
            while *wide.add(len) != 0 {
                len += 1;
            }
            let text = String::from_utf16_lossy(std::slice::from_raw_parts(wide, len));
            let _ = GlobalUnlock(global);
            let _ = CloseClipboard();
            Ok(Some(text))
        }
    }

    fn write_text(&self, text: &str) -> Result<()> {
        use windows::Win32::Foundation::HANDLE;
        use windows::Win32::System::DataExchange::{
            CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData,
        };
        use windows::Win32::System::Memory::{
            GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalUnlock,
        };

        const CF_UNICODETEXT_FORMAT: u32 = 13;
        let mut wide: Vec<u16> = text.encode_utf16().chain(Some(0)).collect();
        unsafe {
            OpenClipboard(None)
                .map_err(|err| AppError::Capture(format!("打开剪贴板失败: {err}")))?;
            EmptyClipboard().map_err(|err| {
                let _ = CloseClipboard();
                AppError::Capture(format!("清空剪贴板失败: {err}"))
            })?;
            let bytes = wide.len() * std::mem::size_of::<u16>();
            let handle = GlobalAlloc(GMEM_MOVEABLE, bytes).map_err(|err| {
                let _ = CloseClipboard();
                AppError::Capture(format!("分配剪贴板内存失败: {err}"))
            })?;
            let ptr = GlobalLock(handle);
            std::ptr::copy_nonoverlapping(wide.as_mut_ptr() as *const u8, ptr as *mut u8, bytes);
            let _ = GlobalUnlock(handle);
            SetClipboardData(CF_UNICODETEXT_FORMAT, Some(HANDLE(handle.0))).map_err(|err| {
                let _ = CloseClipboard();
                AppError::Capture(format!("写入剪贴板失败: {err}"))
            })?;
            let _ = CloseClipboard();
            Ok(())
        }
    }

    fn sequence_number(&self) -> Result<u32> {
        use windows::Win32::System::DataExchange::GetClipboardSequenceNumber;

        unsafe { Ok(GetClipboardSequenceNumber()) }
    }
}

#[cfg(windows)]
pub struct WindowsCopyBackend;

#[cfg(windows)]
impl CopyBackend for WindowsCopyBackend {
    fn send_copy(&self) -> Result<()> {
        use windows::Win32::UI::Input::KeyboardAndMouse::{
            INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, SendInput, VIRTUAL_KEY,
            VK_CONTROL, VK_ESCAPE, VK_LWIN, VK_MENU, VK_SHIFT, VK_TAB,
        };

        unsafe {
            let inputs = [
                key_input(VK_CONTROL, true),
                key_input(VK_MENU, true),
                key_input(VK_SHIFT, true),
                key_input(VK_LWIN, true),
                key_input(VK_TAB, true),
                key_input(VK_ESCAPE, true),
                key_input(VIRTUAL_KEY(0x14), true),
                key_input(VIRTUAL_KEY(b'C' as u16), true),
                key_input(VK_CONTROL, false),
                key_input(VIRTUAL_KEY(b'C' as u16), false),
                key_input(VIRTUAL_KEY(b'C' as u16), true),
                key_input(VK_CONTROL, true),
            ];
            let sent = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
            if sent != inputs.len() as u32 {
                return Err(AppError::Capture("发送复制快捷键失败".to_string()));
            }
            return Ok(());
        }

        unsafe fn key_input(key: VIRTUAL_KEY, key_up: bool) -> INPUT {
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: key,
                        wScan: 0,
                        dwFlags: if key_up {
                            KEYEVENTF_KEYUP
                        } else {
                            Default::default()
                        },
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            }
        }
    }
}

#[cfg(windows)]
pub struct WindowsSelectionBackend;

#[cfg(windows)]
impl SelectionBackend for WindowsSelectionBackend {
    fn read_selected_text(&self) -> Result<Option<String>> {
        use windows::Win32::System::Com::{
            CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
        };
        use windows::Win32::UI::Accessibility::{
            CUIAutomation, IUIAutomation, IUIAutomationTextPattern, UIA_TextPatternId,
        };
        use windows::core::Interface;

        unsafe {
            let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
            let automation: IUIAutomation =
                CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER).map_err(|err| {
                    AppError::Capture(format!("初始化 UI Automation 失败: {err}"))
                })?;
            let element = automation
                .GetFocusedElement()
                .map_err(|err| AppError::Capture(format!("读取焦点控件失败: {err}")))?;
            let pattern = element
                .GetCurrentPattern(UIA_TextPatternId)
                .map_err(|_| AppError::Capture("焦点控件不支持 UIA TextPattern".to_string()))?;
            let text_pattern: IUIAutomationTextPattern = pattern
                .cast()
                .map_err(|err| AppError::Capture(format!("转换 UIA TextPattern 失败: {err}")))?;
            let ranges = text_pattern
                .GetSelection()
                .map_err(|err| AppError::Capture(format!("读取 UIA 选区失败: {err}")))?;
            let length = ranges
                .Length()
                .map_err(|err| AppError::Capture(format!("读取 UIA 选区数量失败: {err}")))?;
            let mut collected = String::new();
            for index in 0..length {
                let range = ranges
                    .GetElement(index)
                    .map_err(|err| AppError::Capture(format!("读取 UIA 选区范围失败: {err}")))?;
                let text = range
                    .GetText(-1)
                    .map_err(|err| AppError::Capture(format!("读取 UIA 选中文本失败: {err}")))?;
                collected.push_str(&text.to_string());
            }
            if collected.trim().is_empty() {
                Ok(None)
            } else {
                Ok(Some(collected))
            }
        }
    }
}
