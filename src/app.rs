use crate::capture::CapturedText;
use crate::error::{AppError, Result};
use crate::translator::{ProviderKind, TranslationRequest, TranslationResponse};
use crate::update::{UpdateStatus, check_for_updates, latest_release_url, update_status_message};

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

    pub fn translate_text(
        &self,
        source_text: &str,
        target_lang: &str,
    ) -> Result<TranslationWorkflowResult> {
        self.translate_text_with_observer(source_text, target_lang, &mut ())
    }

    pub fn translate_text_with_observer<O>(
        &self,
        source_text: &str,
        target_lang: &str,
        observer: &mut O,
    ) -> Result<TranslationWorkflowResult>
    where
        O: TranslationObserver,
    {
        if source_text.trim().is_empty() {
            return Err(AppError::Translate("原文为空".to_string()));
        }

        observer.translation_started()?;
        self.translate_captured_text_with_observer(source_text, target_lang, observer)
    }

    pub fn translate_selection_with_observer<O>(
        &self,
        target_lang: &str,
        observer: &mut O,
    ) -> Result<TranslationWorkflowResult>
    where
        O: TranslationObserver,
    {
        let captured = self.capture.capture()?;
        observer.translation_started()?;
        self.translate_captured_text_with_observer(&captured.text, target_lang, observer)
    }

    fn translate_captured_text_with_observer<O>(
        &self,
        source_text: &str,
        target_lang: &str,
        observer: &mut O,
    ) -> Result<TranslationWorkflowResult>
    where
        O: TranslationObserver,
    {
        if source_text.trim().is_empty() {
            return Err(AppError::Translate("原文为空".to_string()));
        }

        observer.source_captured(source_text)?;
        let response = self.translator.translate_blocking(TranslationRequest {
            text: source_text.to_string(),
            source_lang: "auto".to_string(),
            target_lang: target_lang.to_string(),
        })?;

        let result = TranslationWorkflowResult {
            source_text: source_text.to_string(),
            translated_text: response.translated_text,
            provider: response.provider,
        };
        observer.translation_succeeded(&result)?;
        Ok(result)
    }
}

impl TranslationObserver for () {}

#[derive(Debug, Clone)]
pub struct AppRuntimeState {
    settings: crate::config::AppSettings,
    active_profile_id: String,
}

impl AppRuntimeState {
    pub fn new(settings: crate::config::AppSettings) -> Self {
        let active_profile_id = settings
            .default_profile()
            .map(|profile| profile.id.clone())
            .unwrap_or_else(|_| "google".to_string());
        Self {
            settings,
            active_profile_id,
        }
    }

    pub fn settings(&self) -> &crate::config::AppSettings {
        &self.settings
    }

    pub fn active_profile_id(&self) -> &str {
        &self.active_profile_id
    }

    pub fn active_profile(&self) -> Result<&crate::config::TranslatorProfile> {
        self.settings
            .profile_by_id(&self.active_profile_id)
            .or_else(|| {
                self.settings
                    .profile_by_id(&self.settings.default_profile_id)
            })
            .or_else(|| self.settings.translator_profiles.first())
            .ok_or_else(|| AppError::Config("没有可用的翻译配置".to_string()))
    }

    pub fn select_profile(&mut self, profile_id: &str) -> Result<()> {
        if self.settings.profile_by_id(profile_id).is_none() {
            return Err(AppError::Config("翻译配置不存在".to_string()));
        }
        self.active_profile_id = profile_id.to_string();
        self.settings.default_profile_id = profile_id.to_string();
        Ok(())
    }

    pub fn replace_settings(&mut self, settings: crate::config::AppSettings) {
        self.settings = settings;
        if self
            .settings
            .profile_by_id(&self.settings.default_profile_id)
            .is_some()
        {
            self.active_profile_id = self.settings.default_profile_id.clone();
        } else if let Some(profile) = self.settings.translator_profiles.first() {
            self.active_profile_id = profile.id.clone();
            self.settings.default_profile_id = profile.id.clone();
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyAction {
    Ignore,
    TranslateSelection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HotkeyRegistrationUpdate {
    Unchanged,
    Changed {
        hotkey: String,
    },
    Rejected {
        rollback_hotkey: String,
        message: String,
    },
}

pub fn hotkey_registration_update(
    current_hotkey: &str,
    next_hotkey: &str,
    registration_result: std::result::Result<(), String>,
) -> HotkeyRegistrationUpdate {
    if current_hotkey == next_hotkey {
        return HotkeyRegistrationUpdate::Unchanged;
    }

    match registration_result {
        Ok(()) => HotkeyRegistrationUpdate::Changed {
            hotkey: next_hotkey.to_string(),
        },
        Err(error) => HotkeyRegistrationUpdate::Rejected {
            rollback_hotkey: current_hotkey.to_string(),
            message: format!("快捷键注册失败，请换一个组合键；当前仍使用原来的快捷键。{error}"),
        },
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TranslationRequestKind {
    Selection,
    WindowText { source_text: String },
}

pub fn translation_task_action(
    selection_requested: bool,
    source_text: &str,
) -> TranslationRequestKind {
    if selection_requested {
        TranslationRequestKind::Selection
    } else {
        TranslationRequestKind::WindowText {
            source_text: source_text.to_string(),
        }
    }
}

pub fn run_translation_request_with_observer<C, T, O>(
    workflow: &TranslationWorkflow<C, T>,
    request: TranslationRequestKind,
    target_lang: &str,
    observer: &mut O,
) -> Result<TranslationWorkflowResult>
where
    C: WorkflowCapture,
    T: WorkflowTranslator,
    O: TranslationObserver,
{
    match request {
        TranslationRequestKind::Selection => {
            workflow.translate_selection_with_observer(target_lang, observer)
        }
        TranslationRequestKind::WindowText { source_text } => {
            workflow.translate_text_with_observer(&source_text, target_lang, observer)
        }
    }
}

pub fn hotkey_action(is_translation_window_foreground: bool) -> HotkeyAction {
    if is_translation_window_foreground {
        HotkeyAction::Ignore
    } else {
        HotkeyAction::TranslateSelection
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateCheckAction {
    Ignore,
    ShowDialog,
    ShowUpdateButton,
}

pub fn update_check_action(
    result: std::result::Result<UpdateStatus, String>,
    show_all: bool,
) -> UpdateCheckAction {
    match result {
        Ok(UpdateStatus::UpdateAvailable { .. }) if show_all => UpdateCheckAction::ShowDialog,
        Ok(UpdateStatus::UpdateAvailable { .. }) => UpdateCheckAction::ShowUpdateButton,
        Ok(UpdateStatus::UpToDate) if show_all => UpdateCheckAction::ShowDialog,
        Ok(UpdateStatus::UpToDate) => UpdateCheckAction::Ignore,
        Err(_) if show_all => UpdateCheckAction::ShowDialog,
        Err(_) => UpdateCheckAction::Ignore,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayAction {
    ShowTranslationWindow,
    OpenSettings,
    OpenLogDirectory,
    OpenLatestRelease,
    Exit,
    Unknown,
}

#[cfg(windows)]
pub fn tray_action_from_menu_id(menu_id: usize) -> TrayAction {
    match menu_id {
        crate::ui::tray::MENU_SHOW_TRANSLATION_WINDOW => TrayAction::ShowTranslationWindow,
        crate::ui::tray::MENU_SETTINGS => TrayAction::OpenSettings,
        crate::ui::tray::MENU_OPEN_LOG_DIRECTORY => TrayAction::OpenLogDirectory,
        crate::ui::tray::MENU_OPEN_LATEST_RELEASE => TrayAction::OpenLatestRelease,
        crate::ui::tray::MENU_EXIT => TrayAction::Exit,
        _ => TrayAction::Unknown,
    }
}

#[cfg(windows)]
const WM_TRANSLATION_TASK_FINISHED: u32 = windows::Win32::UI::WindowsAndMessaging::WM_APP + 60;

#[cfg(windows)]
const WM_TRANSLATION_SOURCE_CAPTURED: u32 = windows::Win32::UI::WindowsAndMessaging::WM_APP + 61;

#[cfg(windows)]
pub const WM_UPDATE_CHECK_FINISHED: u32 = windows::Win32::UI::WindowsAndMessaging::WM_APP + 62;

#[cfg(windows)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateCheckDisplayMode {
    SilentWhenUpToDate,
    ShowAll,
}

#[cfg(windows)]
struct TranslationTaskMessage {
    result: Result<TranslationWorkflowResult>,
}

#[cfg(windows)]
struct TranslationSourceMessage {
    source_text: String,
}

#[cfg(windows)]
struct UpdateCheckMessage {
    result: Result<UpdateStatus>,
    current_version: String,
    display_mode: UpdateCheckDisplayMode,
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
    let store = SettingsStore::new(settings_dir.clone());
    let settings = store.load().unwrap_or_else(|_| AppSettings::default());
    let hotkey = settings.hotkey.parse::<Hotkey>()?;
    let mut runtime_state = AppRuntimeState::new(settings);
    let _tray = TrayIcon::create()?;
    let mut _registered_hotkey = RegisteredHotkey::register(1, hotkey)?;
    let mut registered_hotkey_id = 1;
    let mut registered_hotkey_text = hotkey.to_string();
    let mut translation_window = TranslationWindow::new()?;
    translation_window
        .refresh_profiles(runtime_state.settings(), runtime_state.active_profile_id())?;
    spawn_update_check_task(
        translation_window.hwnd(),
        env!("CARGO_PKG_VERSION").to_string(),
        UpdateCheckDisplayMode::SilentWhenUpToDate,
    );

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
                        let _ = translation_window.begin_selection_translation();
                        spawn_translation_task(
                            runtime_state.clone(),
                            TranslationRequestKind::Selection,
                            translation_window.hwnd(),
                        );
                    }
                }
            } else if msg.message == crate::ui::tray::WM_TRAY_COMMAND {
                match tray_action_from_menu_id(msg.wParam.0) {
                    TrayAction::ShowTranslationWindow => {
                        let _ = translation_window.show_window();
                    }
                    TrayAction::OpenSettings => {
                        let _ = handle_app_command(
                            crate::command::AppCommand::OpenSettings,
                            runtime_state.settings(),
                            translation_window.hwnd(),
                        );
                    }
                    TrayAction::OpenLogDirectory => {
                        match crate::logging::log_dir().and_then(|dir| {
                            std::fs::create_dir_all(&dir)?;
                            open_directory(&dir)
                        }) {
                            Ok(()) => {}
                            Err(err) => {
                                tracing::warn!(error = %err, "open log directory failed");
                                show_runtime_message(
                                    translation_window.hwnd(),
                                    "打开失败",
                                    "无法打开日志目录，请稍后重试。",
                                );
                            }
                        }
                    }
                    TrayAction::OpenLatestRelease => {
                        let _ = handle_app_command(
                            crate::command::AppCommand::OpenLatestRelease,
                            runtime_state.settings(),
                            translation_window.hwnd(),
                        );
                    }
                    TrayAction::Exit => {
                        if handle_app_command(
                            crate::command::AppCommand::Exit,
                            runtime_state.settings(),
                            translation_window.hwnd(),
                        )? {
                            PostQuitMessage(0);
                        }
                    }
                    TrayAction::Unknown => {}
                }
            } else if msg.message == crate::ui::translate_window::WM_TRANSLATE_WINDOW_SOURCE {
                match translation_window.source_text() {
                    Ok(source_text) => {
                        let _ =
                            translation_window.begin_window_text_translation(source_text.clone());
                        spawn_translation_task(
                            runtime_state.clone(),
                            TranslationRequestKind::WindowText { source_text },
                            translation_window.hwnd(),
                        );
                    }
                    Err(err) => {
                        let _ = translation_window.show_error(err.to_string());
                    }
                }
            } else if msg.message == WM_UPDATE_CHECK_FINISHED {
                let ptr = msg.lParam.0 as *mut UpdateCheckMessage;
                if !ptr.is_null() {
                    let message = Box::from_raw(ptr);
                    match message.result {
                        Ok(UpdateStatus::UpToDate) => {
                            if matches!(message.display_mode, UpdateCheckDisplayMode::ShowAll) {
                                show_runtime_message(
                                    translation_window.hwnd(),
                                    "检查更新",
                                    &update_status_message(
                                        &message.current_version,
                                        &UpdateStatus::UpToDate,
                                    ),
                                );
                            } else {
                                tracing::info!("update check: already latest");
                            }
                        }
                        Ok(UpdateStatus::UpdateAvailable {
                            current_version,
                            latest_version,
                            release_url,
                        }) => {
                            let status = UpdateStatus::UpdateAvailable {
                                current_version,
                                latest_version,
                                release_url,
                            };
                            if matches!(message.display_mode, UpdateCheckDisplayMode::ShowAll) {
                                show_update_available_message(
                                    translation_window.hwnd(),
                                    &message.current_version,
                                    &status,
                                );
                            } else if let Err(err) =
                                translation_window.show_update_available(status)
                            {
                                tracing::warn!(error = %err, "show update button failed");
                            }
                        }
                        Err(err) => {
                            tracing::warn!(error = %err, "update check failed");
                            if matches!(message.display_mode, UpdateCheckDisplayMode::ShowAll) {
                                show_runtime_message(
                                    translation_window.hwnd(),
                                    "检查更新失败",
                                    "暂时无法检查更新，请稍后重试。",
                                );
                            }
                        }
                    }
                }
            } else if msg.message == crate::ui::translate_window::WM_TRANSLATE_WINDOW_UPDATE_CLICKED
            {
                if let Some(status) = translation_window.update_status() {
                    show_update_available_message(
                        translation_window.hwnd(),
                        env!("CARGO_PKG_VERSION"),
                        status,
                    );
                }
            } else if msg.message == crate::ui::settings_window::WM_SETTINGS_SAVED {
                match SettingsStore::new(settings_dir.clone()).load() {
                    Ok(mut settings) => {
                        let next_hotkey_text = settings.hotkey.clone();
                        if next_hotkey_text != registered_hotkey_text {
                            match next_hotkey_text.parse::<Hotkey>() {
                                Ok(next_hotkey) => {
                                    let next_hotkey_id =
                                        if registered_hotkey_id == 1 { 2 } else { 1 };
                                    match RegisteredHotkey::register(next_hotkey_id, next_hotkey) {
                                        Ok(next_registered) => {
                                            _registered_hotkey = next_registered;
                                            registered_hotkey_id = next_hotkey_id;
                                            registered_hotkey_text = next_hotkey.to_string();
                                            settings.hotkey = registered_hotkey_text.clone();
                                            runtime_state.replace_settings(settings);
                                        }
                                        Err(err) => {
                                            settings.hotkey = registered_hotkey_text.clone();
                                            if let Err(save_err) =
                                                SettingsStore::new(settings_dir.clone())
                                                    .save(&settings)
                                            {
                                                tracing::warn!(error = %save_err, "rollback hotkey save failed");
                                            }
                                            show_runtime_message(
                                                translation_window.hwnd(),
                                                "快捷键注册失败",
                                                &format!(
                                                    "快捷键注册失败，请换一个组合键；当前仍使用原来的快捷键。{err}"
                                                ),
                                            );
                                            runtime_state.replace_settings(settings);
                                        }
                                    }
                                }
                                Err(err) => {
                                    settings.hotkey = registered_hotkey_text.clone();
                                    if let Err(save_err) =
                                        SettingsStore::new(settings_dir.clone()).save(&settings)
                                    {
                                        tracing::warn!(error = %save_err, "rollback invalid hotkey save failed");
                                    }
                                    show_runtime_message(
                                        translation_window.hwnd(),
                                        "快捷键注册失败",
                                        &format!("快捷键无效，当前仍使用原来的快捷键。{err}"),
                                    );
                                    runtime_state.replace_settings(settings);
                                }
                            }
                        } else {
                            runtime_state.replace_settings(settings);
                        }
                        let _ = translation_window.refresh_profiles(
                            runtime_state.settings(),
                            runtime_state.active_profile_id(),
                        );
                    }
                    Err(err) => tracing::warn!(error = %err, "reload settings failed"),
                }
            } else if msg.message
                == crate::ui::translate_window::WM_TRANSLATE_WINDOW_PROFILE_CHANGED
            {
                if let Some(profile_id) = translation_window.selected_profile_id() {
                    let source_text = translation_window.source_text().unwrap_or_default();
                    match crate::ui::translate_window::profile_selection_action(
                        &profile_id,
                        &source_text,
                    ) {
                        crate::ui::translate_window::ProfileSelectionAction::SaveDefaultOnly {
                            profile_id,
                        } => {
                            if let Err(err) = save_default_profile_selection(
                                &settings_dir,
                                &mut runtime_state,
                                &profile_id,
                                &mut translation_window,
                            ) {
                                let _ = translation_window.show_error(err.to_string());
                            }
                        }
                        crate::ui::translate_window::ProfileSelectionAction::SaveDefaultAndRetranslate {
                            profile_id,
                        } => {
                            match save_default_profile_selection(
                                &settings_dir,
                                &mut runtime_state,
                                &profile_id,
                                &mut translation_window,
                            ) {
                                Ok(()) => {
                                    let source_text =
                                        translation_window.source_text().unwrap_or_default();
                                    let _ = translation_window
                                        .begin_window_text_translation(source_text.clone());
                                    spawn_translation_task(
                                        runtime_state.clone(),
                                        TranslationRequestKind::WindowText { source_text },
                                        translation_window.hwnd(),
                                    );
                                }
                                Err(err) => {
                                    let _ = translation_window.show_error(err.to_string());
                                }
                            }
                        }
                    }
                }
            } else if msg.message == WM_TRANSLATION_SOURCE_CAPTURED {
                let ptr = msg.lParam.0 as *mut TranslationSourceMessage;
                if !ptr.is_null() {
                    let message = Box::from_raw(ptr);
                    let _ = translation_window.show_loading(message.source_text);
                }
            } else if msg.message == WM_TRANSLATION_TASK_FINISHED {
                let ptr = msg.lParam.0 as *mut TranslationTaskMessage;
                if !ptr.is_null() {
                    let message = Box::from_raw(ptr);
                    match message.result {
                        Ok(result) => {
                            tracing::info!(
                                provider = result.provider.as_log_name(),
                                profile_id = runtime_state.active_profile_id(),
                                source_len = result.source_text.chars().count(),
                                translated_len = result.translated_text.chars().count(),
                                "translation completed"
                            );
                            let _ = translation_window.finish_translation_result(Ok(result));
                        }
                        Err(err) => {
                            tracing::warn!(error = %err, profile_id = runtime_state.active_profile_id(), "translation failed");
                            let _ = translation_window.finish_translation_result(Err(err));
                        }
                    }
                }
            }
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    Ok(())
}

#[cfg(windows)]
fn build_workflow(
    state: &AppRuntimeState,
) -> Result<TranslationWorkflow<WindowsWorkflowCapture, Box<dyn WorkflowTranslator>>> {
    Ok(TranslationWorkflow::new(
        WindowsWorkflowCapture {
            wait_ms: state.settings().clipboard_capture.copy_wait_ms,
        },
        build_workflow_translator_for_profile(state.active_profile()?)?,
    ))
}

#[cfg(windows)]
fn spawn_translation_task(
    state: AppRuntimeState,
    request: TranslationRequestKind,
    notify_hwnd: windows::Win32::Foundation::HWND,
) {
    let notify_hwnd = notify_hwnd.0 as isize;
    std::thread::spawn(move || {
        let result = run_translation_task(&state, request, notify_hwnd);
        let message = Box::into_raw(Box::new(TranslationTaskMessage { result }));
        let notify_hwnd = windows::Win32::Foundation::HWND(notify_hwnd as *mut core::ffi::c_void);
        unsafe {
            let posted = windows::Win32::UI::WindowsAndMessaging::PostMessageW(
                Some(notify_hwnd),
                WM_TRANSLATION_TASK_FINISHED,
                windows::Win32::Foundation::WPARAM(0),
                windows::Win32::Foundation::LPARAM(message as isize),
            );
            if posted.is_err() {
                drop(Box::from_raw(message));
            }
        }
    });
}

#[cfg(windows)]
fn run_translation_task(
    state: &AppRuntimeState,
    request: TranslationRequestKind,
    notify_hwnd: isize,
) -> Result<TranslationWorkflowResult> {
    let workflow = build_workflow(state)?;
    match request {
        TranslationRequestKind::Selection => {
            let mut observer = TranslationProgressObserver { notify_hwnd };
            run_translation_request_with_observer(
                &workflow,
                TranslationRequestKind::Selection,
                &state.settings().target_language,
                &mut observer,
            )
        }
        TranslationRequestKind::WindowText { source_text } => {
            run_translation_request_with_observer(
                &workflow,
                TranslationRequestKind::WindowText { source_text },
                &state.settings().target_language,
                &mut (),
            )
        }
    }
}

#[cfg(windows)]
struct TranslationProgressObserver {
    notify_hwnd: isize,
}

#[cfg(windows)]
impl TranslationObserver for TranslationProgressObserver {
    fn source_captured(&mut self, source_text: &str) -> Result<()> {
        let message = Box::into_raw(Box::new(TranslationSourceMessage {
            source_text: source_text.to_string(),
        }));
        let notify_hwnd =
            windows::Win32::Foundation::HWND(self.notify_hwnd as *mut core::ffi::c_void);
        unsafe {
            let posted = windows::Win32::UI::WindowsAndMessaging::PostMessageW(
                Some(notify_hwnd),
                WM_TRANSLATION_SOURCE_CAPTURED,
                windows::Win32::Foundation::WPARAM(0),
                windows::Win32::Foundation::LPARAM(message as isize),
            );
            if posted.is_err() {
                drop(Box::from_raw(message));
            }
        }
        Ok(())
    }
}

#[cfg(windows)]
fn save_default_profile_selection(
    settings_dir: &std::path::Path,
    state: &mut AppRuntimeState,
    profile_id: &str,
    window: &mut crate::ui::translate_window::TranslationWindow,
) -> Result<()> {
    state.select_profile(profile_id)?;
    crate::config::SettingsStore::new(settings_dir.to_path_buf()).save(state.settings())?;
    window.refresh_profiles(state.settings(), state.active_profile_id())?;
    Ok(())
}

#[cfg(windows)]
fn handle_app_command(
    command: crate::command::AppCommand,
    settings: &crate::config::AppSettings,
    owner_hwnd: windows::Win32::Foundation::HWND,
) -> Result<bool> {
    match command {
        crate::command::AppCommand::OpenSettings => {
            crate::ui::settings_window::SettingsWindow::open(settings, owner_hwnd)?;
            Ok(false)
        }
        crate::command::AppCommand::OpenLatestRelease => {
            open_url(owner_hwnd, latest_release_url())?;
            Ok(false)
        }
        crate::command::AppCommand::Exit => Ok(true),
        _ => Ok(false),
    }
}

#[cfg(windows)]
pub fn spawn_update_check_task(
    notify_hwnd: windows::Win32::Foundation::HWND,
    current_version: String,
    display_mode: UpdateCheckDisplayMode,
) {
    let notify_hwnd = notify_hwnd.0 as isize;
    std::thread::spawn(move || {
        let result = tokio::runtime::Runtime::new()
            .map_err(|err| AppError::Translate(format!("启动更新检查运行时失败: {err}")))
            .and_then(|runtime| runtime.block_on(check_for_updates(&current_version)));
        let message = Box::into_raw(Box::new(UpdateCheckMessage {
            result,
            current_version,
            display_mode,
        }));
        let notify_hwnd = windows::Win32::Foundation::HWND(notify_hwnd as *mut core::ffi::c_void);
        unsafe {
            let posted = windows::Win32::UI::WindowsAndMessaging::PostMessageW(
                Some(notify_hwnd),
                WM_UPDATE_CHECK_FINISHED,
                windows::Win32::Foundation::WPARAM(0),
                windows::Win32::Foundation::LPARAM(message as isize),
            );
            if posted.is_err() {
                drop(Box::from_raw(message));
            }
        }
    });
}

#[cfg(windows)]
fn show_runtime_message(
    owner_hwnd: windows::Win32::Foundation::HWND,
    caption: &str,
    text: &str,
) -> bool {
    let caption = wide(caption);
    let text = wide(text);
    let result = unsafe {
        windows::Win32::UI::WindowsAndMessaging::MessageBoxW(
            Some(owner_hwnd),
            windows::core::PCWSTR(text.as_ptr()),
            windows::core::PCWSTR(caption.as_ptr()),
            windows::Win32::UI::WindowsAndMessaging::MB_OK,
        )
    };
    result == windows::Win32::UI::WindowsAndMessaging::IDOK
}

#[cfg(windows)]
fn show_update_available_message(
    owner_hwnd: windows::Win32::Foundation::HWND,
    current_version: &str,
    status: &UpdateStatus,
) {
    let release_url = match status {
        UpdateStatus::UpdateAvailable { release_url, .. } => release_url.as_str(),
        UpdateStatus::UpToDate => {
            let _ = show_runtime_message(
                owner_hwnd,
                "检查更新",
                &update_status_message(current_version, status),
            );
            return;
        }
    };

    if show_runtime_message(
        owner_hwnd,
        "发现新版本",
        &update_status_message(current_version, status),
    ) {
        if let Err(err) = open_url(owner_hwnd, release_url) {
            tracing::warn!(error = %err, "open release page after update confirmation failed");
            let _ = show_runtime_message(
                owner_hwnd,
                "打开失败",
                "无法打开 Release 页面，请稍后重试。",
            );
        }
    }
}

#[cfg(windows)]
fn open_directory(path: &std::path::Path) -> Result<()> {
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
    use windows::core::PCWSTR;

    let operation = wide("open");
    let file = wide(&path.to_string_lossy());
    let result = unsafe {
        ShellExecuteW(
            None,
            PCWSTR(operation.as_ptr()),
            PCWSTR(file.as_ptr()),
            PCWSTR::null(),
            PCWSTR::null(),
            SW_SHOWNORMAL,
        )
    };
    if result.0 as isize <= 32 {
        return Err(crate::error::AppError::Windows(
            "打开日志目录失败".to_string(),
        ));
    }
    Ok(())
}

#[cfg(windows)]
fn open_url(owner_hwnd: windows::Win32::Foundation::HWND, url: &str) -> Result<()> {
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
    use windows::core::PCWSTR;

    let operation = wide("open");
    let file = wide(url);
    let result = unsafe {
        ShellExecuteW(
            Some(owner_hwnd),
            PCWSTR(operation.as_ptr()),
            PCWSTR(file.as_ptr()),
            PCWSTR::null(),
            PCWSTR::null(),
            SW_SHOWNORMAL,
        )
    };
    if result.0 as isize <= 32 {
        return Err(AppError::Windows("打开链接失败".to_string()));
    }
    Ok(())
}

#[cfg(windows)]
fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(Some(0)).collect()
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
fn build_workflow_translator_for_profile(
    profile: &crate::config::TranslatorProfile,
) -> Result<Box<dyn WorkflowTranslator>> {
    match profile.provider {
        crate::config::TranslatorProvider::Google => Ok(Box::new(BlockingGoogleTranslator(
            crate::translator::google_free::GoogleFreeTranslator::new(),
        ))),
        crate::config::TranslatorProvider::OpenAi
        | crate::config::TranslatorProvider::Claude
        | crate::config::TranslatorProvider::Gemini
        | crate::config::TranslatorProvider::DeepSeek
        | crate::config::TranslatorProvider::Custom => {
            let encrypted = profile
                .encrypted_api_key
                .as_ref()
                .ok_or_else(|| crate::error::AppError::Translate("API Key 缺失".to_string()))?;
            let api_key =
                crate::secret::SecretStore::new(format!("ait-translator-profile-{}", profile.id))
                    .unprotect(encrypted)?;
            let translator = crate::translator::openai_compatible::OpenAiCompatibleTranslator::new(
                crate::translator::openai_compatible::OpenAiCompatibleConfig {
                    provider: profile.provider,
                    base_url: profile.base_url.clone(),
                    api_key,
                    model: profile.model.clone(),
                    timeout_secs: profile.timeout_secs,
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
