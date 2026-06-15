use crate::capture::CapturedText;
use crate::error::Result;
use crate::translator::{ProviderKind, TranslationRequest, TranslationResponse};

pub trait WorkflowCapture {
    fn capture(&self) -> Result<CapturedText>;
}

pub trait WorkflowTranslator {
    fn translate_blocking(&self, request: TranslationRequest) -> Result<TranslationResponse>;
}

impl<T: WorkflowTranslator + ?Sized> WorkflowTranslator for Box<T> {
    fn translate_blocking(&self, request: TranslationRequest) -> Result<TranslationResponse> {
        (**self).translate_blocking(request)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranslationWorkflowResult {
    pub source_text: String,
    pub translated_text: String,
    pub provider: ProviderKind,
}

pub trait TranslationObserver {
    fn translation_started(&mut self) -> Result<()> {
        Ok(())
    }

    fn source_captured(&mut self, _source_text: &str) -> Result<()> {
        Ok(())
    }

    fn translation_succeeded(&mut self, _result: &TranslationWorkflowResult) -> Result<()> {
        Ok(())
    }
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
        self.translate_selection_with_observer(target_lang, &mut ())
    }

    pub fn translate_selection_with_observer<O>(
        &self,
        target_lang: &str,
        observer: &mut O,
    ) -> Result<TranslationWorkflowResult>
    where
        O: TranslationObserver,
    {
        observer.translation_started()?;
        let captured = self.capture.capture()?;
        observer.source_captured(&captured.text)?;
        let response = self.translator.translate_blocking(TranslationRequest {
            text: captured.text.clone(),
            source_lang: "auto".to_string(),
            target_lang: target_lang.to_string(),
        })?;

        let result = TranslationWorkflowResult {
            source_text: captured.text,
            translated_text: response.translated_text,
            provider: response.provider,
        };
        observer.translation_succeeded(&result)?;
        Ok(result)
    }
}

impl TranslationObserver for () {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyAction {
    Ignore,
    TranslateSelection,
}

pub fn hotkey_action(is_translation_window_foreground: bool) -> HotkeyAction {
    if is_translation_window_foreground {
        HotkeyAction::Ignore
    } else {
        HotkeyAction::TranslateSelection
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
    use crate::ui::translate_window::TranslationWindow;
    use crate::ui::tray::TrayIcon;
    use windows::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, GetMessageW, MSG, PostQuitMessage, TranslateMessage, WM_HOTKEY,
    };

    let settings_dir = SettingsStore::default_dir()?;
    let settings = SettingsStore::new(settings_dir)
        .load()
        .unwrap_or_else(|_| AppSettings::default());
    let hotkey = settings.hotkey.parse::<Hotkey>()?;
    let _tray = TrayIcon::create()?;
    let _registered = RegisteredHotkey::register(1, hotkey)?;
    let workflow = TranslationWorkflow::new(
        WindowsWorkflowCapture {
            wait_ms: settings.clipboard_capture.copy_wait_ms,
        },
        build_workflow_translator(&settings)?,
    );
    let mut translation_window = TranslationWindow::new()?;

    tracing::info!("registered hotkey {}", hotkey);

    unsafe {
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            if msg.message == WM_HOTKEY {
                match hotkey_action(translation_window.is_foreground()) {
                    HotkeyAction::Ignore => {
                        tracing::info!("ignore hotkey while translation window is foreground");
                    }
                    HotkeyAction::TranslateSelection => {
                        tracing::info!("TranslateSelection requested");
                        let _ = perform_translation(&workflow, &settings, &mut translation_window);
                    }
                }
            } else if msg.message == crate::ui::tray::WM_TRAY_COMMAND {
                match msg.wParam.0 {
                    crate::ui::tray::MENU_TRANSLATE_SELECTION => {
                        let _ = perform_translation(&workflow, &settings, &mut translation_window);
                    }
                    crate::ui::tray::MENU_SETTINGS => {
                        let _ =
                            handle_app_command(crate::command::AppCommand::OpenSettings, &settings);
                    }
                    crate::ui::tray::MENU_OPEN_LOGS => {
                        tracing::info!("OpenLogs requested");
                    }
                    crate::ui::tray::MENU_EXIT => {
                        if handle_app_command(crate::command::AppCommand::Exit, &settings)? {
                            PostQuitMessage(0);
                        }
                    }
                    _ => {}
                }
            }
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    Ok(())
}

#[cfg(windows)]
fn perform_translation<C, T>(
    workflow: &TranslationWorkflow<C, T>,
    settings: &crate::config::AppSettings,
    window: &mut crate::ui::translate_window::TranslationWindow,
) -> Result<()>
where
    C: WorkflowCapture,
    T: WorkflowTranslator,
{
    match workflow.translate_selection_with_observer(&settings.target_language, window) {
        Ok(result) => {
            tracing::info!(
                provider = result.provider.as_log_name(),
                source_len = result.source_text.chars().count(),
                translated_len = result.translated_text.chars().count(),
                "translation completed"
            );
        }
        Err(err) => {
            let _ = window.show_error(err.to_string());
            tracing::warn!(error = %err, "translation failed");
        }
    }
    Ok(())
}

#[cfg(windows)]
fn handle_app_command(
    command: crate::command::AppCommand,
    settings: &crate::config::AppSettings,
) -> Result<bool> {
    match command {
        crate::command::AppCommand::OpenSettings => {
            crate::ui::settings_window::SettingsWindow::open(settings)?;
            Ok(false)
        }
        crate::command::AppCommand::Exit => Ok(true),
        _ => Ok(false),
    }
}

#[cfg(windows)]
struct WindowsWorkflowCapture {
    wait_ms: u64,
}

#[cfg(windows)]
impl WorkflowCapture for WindowsWorkflowCapture {
    fn capture(&self) -> Result<crate::capture::CapturedText> {
        let service = crate::capture::CaptureService::new(
            crate::capture::WindowsClipboardBackend,
            std::time::Duration::from_millis(self.wait_ms),
        )
        .with_selection(crate::capture::WindowsSelectionBackend)
        .with_copy(crate::capture::WindowsCopyBackend);
        service
            .capture_selected_text()
            .map_err(|err| crate::error::AppError::Capture(err.to_string()))
    }
}

#[cfg(windows)]
struct BlockingGoogleTranslator(crate::translator::google_free::GoogleFreeTranslator);

#[cfg(windows)]
impl WorkflowTranslator for BlockingGoogleTranslator {
    fn translate_blocking(
        &self,
        request: crate::translator::TranslationRequest,
    ) -> Result<crate::translator::TranslationResponse> {
        crate::translator::translate_blocking(&self.0, request)
    }
}

#[cfg(windows)]
fn build_workflow_translator(
    settings: &crate::config::AppSettings,
) -> Result<Box<dyn WorkflowTranslator>> {
    match settings.default_provider {
        crate::config::ProviderKind::GoogleFree => Ok(Box::new(BlockingGoogleTranslator(
            crate::translator::google_free::GoogleFreeTranslator::new(),
        ))),
        crate::config::ProviderKind::OpenAiCompatible => {
            let encrypted = settings
                .openai
                .encrypted_api_key
                .as_ref()
                .ok_or_else(|| crate::error::AppError::Translate("API Key 缺失".to_string()))?;
            let api_key =
                crate::secret::SecretStore::new("ait-openai-api-key").unprotect(encrypted)?;
            let translator = crate::translator::openai_compatible::OpenAiCompatibleTranslator::new(
                crate::translator::openai_compatible::OpenAiCompatibleConfig {
                    base_url: settings.openai.base_url.clone(),
                    api_key,
                    model: settings.openai.model.clone(),
                    timeout_secs: settings.openai.timeout_secs,
                },
            )?;
            Ok(Box::new(BlockingOpenAiTranslator(translator)))
        }
    }
}

#[cfg(windows)]
struct BlockingOpenAiTranslator(crate::translator::openai_compatible::OpenAiCompatibleTranslator);

#[cfg(windows)]
impl WorkflowTranslator for BlockingOpenAiTranslator {
    fn translate_blocking(
        &self,
        request: crate::translator::TranslationRequest,
    ) -> Result<crate::translator::TranslationResponse> {
        crate::translator::translate_blocking(&self.0, request)
    }
}
