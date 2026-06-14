use crate::capture::CapturedText;
use crate::error::Result;
use crate::translator::{ProviderKind, TranslationRequest, TranslationResponse};

pub trait WorkflowCapture {
    fn capture(&self) -> Result<CapturedText>;
}

pub trait WorkflowTranslator {
    fn translate_blocking(&self, request: TranslationRequest) -> Result<TranslationResponse>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranslationWorkflowResult {
    pub source_text: String,
    pub translated_text: String,
    pub provider: ProviderKind,
}

pub struct TranslationWorkflow<C, T> {
    capture: C,
    translator: T,
}

impl<C, T> TranslationWorkflow<C, T>
where
    C: WorkflowCapture,
    T: WorkflowTranslator,
{
    pub fn new(capture: C, translator: T) -> Self {
        Self {
            capture,
            translator,
        }
    }

    pub fn translate_selection(&self, target_lang: &str) -> Result<TranslationWorkflowResult> {
        let captured = self.capture.capture()?;
        let response = self.translator.translate_blocking(TranslationRequest {
            text: captured.text.clone(),
            source_lang: "auto".to_string(),
            target_lang: target_lang.to_string(),
        })?;

        Ok(TranslationWorkflowResult {
            source_text: captured.text,
            translated_text: response.translated_text,
            provider: response.provider,
        })
    }
}

pub fn run() -> Result<()> {
    crate::logging::init_logging()?;
    run_platform()
}

#[cfg(not(windows))]
fn run_platform() -> Result<()> {
    tracing::warn!("ait MVP currently supports Windows only");
    Ok(())
}

#[cfg(windows)]
fn run_platform() -> Result<()> {
    use crate::config::{AppSettings, SettingsStore};
    use crate::hotkey::{Hotkey, RegisteredHotkey};
    use crate::ui::tray::TrayIcon;
    use windows::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, GetMessageW, TranslateMessage, MSG, WM_HOTKEY,
    };

    let settings_dir = SettingsStore::default_dir()?;
    let settings = SettingsStore::new(settings_dir)
        .load()
        .unwrap_or_else(|_| AppSettings::default());
    let hotkey = settings.hotkey.parse::<Hotkey>()?;
    let _tray = TrayIcon::create()?;
    let _registered = RegisteredHotkey::register(1, hotkey)?;

    tracing::info!("registered hotkey {}", hotkey);

    unsafe {
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            if msg.message == WM_HOTKEY {
                tracing::info!("TranslateSelection requested");
            }
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    Ok(())
}
