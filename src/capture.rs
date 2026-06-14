use crate::error::{AppError, Result};
use std::thread;
use std::time::Duration;

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
    fn send_copy(&self) -> Result<()>;
}

pub struct CaptureService<B> {
    backend: B,
    copy_wait: Duration,
}

impl<B: ClipboardBackend> CaptureService<B> {
    pub fn new(backend: B, copy_wait: Duration) -> Self {
        Self { backend, copy_wait }
    }

    pub fn backend(&self) -> &B {
        &self.backend
    }

    pub fn capture_selected_text(&self) -> std::result::Result<CapturedText, CaptureError> {
        let previous = self.backend.read_text().map_err(to_capture_error)?;
        if previous.is_some() {
            self.backend.write_text("").map_err(to_capture_error)?;
        }
        self.backend.send_copy().map_err(|err| CaptureError {
            kind: CaptureErrorKind::CopyFailed,
            message: err.to_string(),
        })?;
        thread::sleep(self.copy_wait);

        let copied = self.backend.read_text().map_err(to_capture_error)?;
        if let Some(old) = previous {
            let _ = self.backend.write_text(&old);
        }

        let text = copied.unwrap_or_default();
        if text.trim().is_empty() {
            return Err(CaptureError {
                kind: CaptureErrorKind::NoText,
                message: "没有取到选中文本".to_string(),
            });
        }

        Ok(CapturedText { text })
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

    fn send_copy(&self) -> Result<()> {
        use windows::Win32::UI::Input::KeyboardAndMouse::{
            INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, SendInput, VIRTUAL_KEY,
            VK_CONTROL,
        };

        unsafe {
            let inputs = [
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
