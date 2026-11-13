use crate::error::{AppError, Result};
use crate::translator::{TranslationRequest, Translator};
use crate::update::{UpdateStatus, check_for_updates, latest_release_url, update_status_message};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranslationWorkflowResult {
    pub source_text: String,
    pub translated_text: String,
    pub provider: &'static str,
}

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

pub fn startup_hotkey_failure_message(hotkey: &str, error: &str) -> String {
    format!(
        "快捷键 {hotkey} 注册失败，可能已被其他程序占用。ait 已继续启动，但全局快捷键暂不可用，请在设置中更换快捷键。{error}"
    )
}

pub fn hotkey_registration_failure_message(current_hotkey: Option<&str>, error: &str) -> String {
    match current_hotkey {
        Some(_) => format!("快捷键注册失败，请换一个组合键；当前仍使用原来的快捷键。{error}"),
        None => format!("快捷键注册失败，请换一个组合键；当前没有可用的全局快捷键。{error}"),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TranslationRequestKind {
    Selection,
    WindowText { source_text: String },
}

pub fn hotkey_action(is_translation_window_foreground: bool) -> HotkeyAction {
    if is_translation_window_foreground {
        HotkeyAction::Ignore
    } else {
        HotkeyAction::TranslateSelection
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayAction {
    ShowTranslationWindow,
    OpenSettings,
    #[cfg(not(feature = "portable"))]
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
        #[cfg(not(feature = "portable"))]
        crate::ui::tray::MENU_OPEN_LOG_DIRECTORY => TrayAction::OpenLogDirectory,
        crate::ui::tray::MENU_OPEN_LATEST_RELEASE => TrayAction::OpenLatestRelease,
        crate::ui::tray::MENU_EXIT => TrayAction::Exit,
        _ => TrayAction::Unknown,
    }
}

#[cfg(windows)]
pub const WM_UPDATE_CHECK_FINISHED: u32 = windows::Win32::UI::WindowsAndMessaging::WM_APP + 62;

#[cfg(windows)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateCheckDisplayMode {
    SilentWhenUpToDate,
    ShowAll,
}

#[cfg(windows)]
struct UpdateCheckMessage {
    result: Result<UpdateStatus>,
    current_version: String,
    display_mode: UpdateCheckDisplayMode,
}

pub fn run() -> Result<()> {
    #[cfg(windows)]
    let Some(_instance) = crate::single_instance::SingleInstance::acquire()? else {
        return Ok(());
    };

    #[cfg(not(feature = "portable"))]
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
        DispatchMessageW, GetMessageW, KillTimer, MSG, PostQuitMessage, SetTimer, TranslateMessage,
        WM_HOTKEY, WM_TIMER,
    };

    let settings_dir = SettingsStore::default_dir()?;
    let store = SettingsStore::new(settings_dir.clone());
    let settings = store.load().unwrap_or_else(|_| AppSettings::default());
    crate::ui::theme::set_mode(settings.theme);
    let configured_hotkey_text = settings.hotkey.clone();
    let mut runtime_state = AppRuntimeState::new(settings);
    let _tray = TrayIcon::create()?;
    let mut translation_window = TranslationWindow::new()?;
    let mut registered_hotkey_id = 1;
    let mut registered_hotkey_text = None;
    let mut startup_hotkey_error = None;
    let startup_registration = configured_hotkey_text.parse::<Hotkey>().and_then(|hotkey| {
        let normalized_hotkey = hotkey.to_string();
        RegisteredHotkey::register(registered_hotkey_id, hotkey)
            .map(|registered| (registered, normalized_hotkey))
    });
    let mut _registered_hotkey = match startup_registration {
        Ok((value, normalized_hotkey)) => {
            registered_hotkey_text = Some(normalized_hotkey);
            Some(value)
        }
        Err(err) => {
            tracing::warn!(
                hotkey = %configured_hotkey_text,
                error = %err,
                "startup hotkey registration failed; continuing without a global hotkey"
            );
            startup_hotkey_error = Some(err.to_string());
            None
        }
    };
    let mut next_translation_request_id = 1_usize;
    translation_window.set_layout(&runtime_state.settings().window)?;
    translation_window
        .refresh_profiles(runtime_state.settings(), runtime_state.active_profile_id())?;

    if let Some(error) = startup_hotkey_error {
        show_runtime_message(
            translation_window.hwnd(),
            "快捷键不可用",
            &startup_hotkey_failure_message(&configured_hotkey_text, &error),
        );
        if let Err(err) = handle_app_command(
            crate::command::AppCommand::OpenSettings,
            runtime_state.settings(),
            translation_window.hwnd(),
        ) {
            tracing::warn!(error = %err, "open settings after hotkey registration failure failed");
        }
    }
    // A thread-message timer reuses the UI loop without a waiting worker thread.
    let update_timer = unsafe { SetTimer(None, 0, 6 * 60 * 60 * 1000, None) };
    if update_timer == 0 {
        tracing::warn!("could not start periodic update timer");
    }
    spawn_update_check_task(
        translation_window.hwnd(),
        env!("CARGO_PKG_VERSION").to_string(),
        UpdateCheckDisplayMode::SilentWhenUpToDate,
    );

    if _registered_hotkey.is_some() {
        tracing::info!("registered hotkey {}", configured_hotkey_text);
    }

    unsafe {
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            if msg.message == WM_TIMER
                && msg.hwnd.is_invalid()
                && update_timer != 0
                && msg.wParam.0 == update_timer
            {
                spawn_update_check_task(
                    translation_window.hwnd(),
                    env!("CARGO_PKG_VERSION").to_string(),
                    UpdateCheckDisplayMode::SilentWhenUpToDate,
                );
                continue;
            }
            if msg.message == WM_HOTKEY {
                match hotkey_action(translation_window.is_foreground()) {
                    HotkeyAction::Ignore => {
                        tracing::info!("ignore hotkey while translation window is foreground");
                    }
                    HotkeyAction::TranslateSelection => {
                        let request_id = take_request_id(&mut next_translation_request_id);
                        tracing::info!("TranslateSelection requested");
                        let _ = translation_window.begin_selection_translation(request_id);
                        spawn_translation_task(
                            runtime_state.clone(),
                            request_id,
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
                    #[cfg(not(feature = "portable"))]
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
                        let request_id = take_request_id(&mut next_translation_request_id);
                        let _ = translation_window
                            .begin_window_text_translation(request_id, source_text.clone());
                        spawn_translation_task(
                            runtime_state.clone(),
                            request_id,
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
                        if registered_hotkey_text.as_deref() != Some(next_hotkey_text.as_str()) {
                            match next_hotkey_text.parse::<Hotkey>() {
                                Ok(next_hotkey) => {
                                    let next_hotkey_id =
                                        if registered_hotkey_id == 1 { 2 } else { 1 };
                                    match RegisteredHotkey::register(next_hotkey_id, next_hotkey) {
                                        Ok(next_registered) => {
                                            _registered_hotkey = Some(next_registered);
                                            registered_hotkey_id = next_hotkey_id;
                                            let normalized_hotkey = next_hotkey.to_string();
                                            registered_hotkey_text =
                                                Some(normalized_hotkey.clone());
                                            settings.hotkey = normalized_hotkey;
                                            if let Err(save_err) =
                                                SettingsStore::new(settings_dir.clone())
                                                    .save(&settings)
                                            {
                                                tracing::warn!(error = %save_err, "save normalized hotkey failed");
                                            }
                                            runtime_state.replace_settings(settings);
                                        }
                                        Err(err) => {
                                            let current_hotkey = registered_hotkey_text.clone();
                                            if let Some(current_hotkey) = current_hotkey.as_ref() {
                                                settings.hotkey = current_hotkey.clone();
                                                if let Err(save_err) =
                                                    SettingsStore::new(settings_dir.clone())
                                                        .save(&settings)
                                                {
                                                    tracing::warn!(error = %save_err, "rollback hotkey save failed");
                                                }
                                            }
                                            show_runtime_message(
                                                translation_window.hwnd(),
                                                "快捷键注册失败",
                                                &hotkey_registration_failure_message(
                                                    current_hotkey.as_deref(),
                                                    &err.to_string(),
                                                ),
                                            );
                                            runtime_state.replace_settings(settings);
                                        }
                                    }
                                }
                                Err(err) => {
                                    let current_hotkey = registered_hotkey_text.clone();
                                    if let Some(current_hotkey) = current_hotkey.as_ref() {
                                        settings.hotkey = current_hotkey.clone();
                                        if let Err(save_err) =
                                            SettingsStore::new(settings_dir.clone()).save(&settings)
                                        {
                                            tracing::warn!(error = %save_err, "rollback invalid hotkey save failed");
                                        }
                                    }
                                    let message = if current_hotkey.is_some() {
                                        format!("快捷键无效，当前仍使用原来的快捷键。{err}")
                                    } else {
                                        format!("快捷键无效，当前没有可用的全局快捷键。{err}")
                                    };
                                    show_runtime_message(
                                        translation_window.hwnd(),
                                        "快捷键注册失败",
                                        &message,
                                    );
                                    runtime_state.replace_settings(settings);
                                }
                            }
                        } else {
                            runtime_state.replace_settings(settings);
                        }
                        crate::ui::theme::set_mode(runtime_state.settings().theme);
                        if let Err(err) =
                            translation_window.set_layout(&runtime_state.settings().window)
                        {
                            tracing::warn!(error = %err, "update translation window layout failed");
                        }
                        let _ = translation_window.refresh_profiles(
                            runtime_state.settings(),
                            runtime_state.active_profile_id(),
                        );
                    }
                    Err(err) => tracing::warn!(error = %err, "reload settings failed"),
                }
            } else if msg.message == crate::ui::translate_window::WM_TRANSLATE_WINDOW_RATIO_CHANGED
            {
                runtime_state.settings.window.source_ratio = translation_window.source_ratio();
                if let Err(err) = crate::config::SettingsStore::new(settings_dir.clone())
                    .save(runtime_state.settings())
                {
                    tracing::warn!(error = %err, "save translation pane ratio failed");
                }
            } else if msg.message
                == crate::ui::translate_window::WM_TRANSLATE_WINDOW_PROFILE_CHANGED
                && let Some(profile_id) = translation_window.selected_profile_id()
            {
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
                                let request_id =
                                    take_request_id(&mut next_translation_request_id);
                                let source_text =
                                    translation_window.source_text().unwrap_or_default();
                                let _ = translation_window.begin_window_text_translation(
                                    request_id,
                                    source_text.clone(),
                                );
                                spawn_translation_task(
                                    runtime_state.clone(),
                                    request_id,
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
            if translation_window.handle_dialog_message(&mut msg)
                || crate::ui::settings_window::handle_dialog_message(&mut msg)
            {
                continue;
            }
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        if update_timer != 0 {
            let _ = KillTimer(None, update_timer);
        }
    }
    Ok(())
}

#[cfg(windows)]
fn spawn_translation_task(
    state: AppRuntimeState,
    request_id: usize,
    request: TranslationRequestKind,
    notify_hwnd: windows::Win32::Foundation::HWND,
) {
    let notify_hwnd = notify_hwnd.0 as isize;
    std::thread::spawn(move || {
        let result = run_translation_task(&state, request_id, request, notify_hwnd);
        let notify_hwnd = windows::Win32::Foundation::HWND(notify_hwnd as *mut core::ffi::c_void);
        crate::ui::translate_window::post_translation_result(
            notify_hwnd,
            crate::ui::translate_window::TranslationTaskMessage {
                request_id,
                profile_id: state.active_profile_id().to_string(),
                result,
            },
        );
    });
}

#[cfg(windows)]
fn run_translation_task(
    state: &AppRuntimeState,
    request_id: usize,
    request: TranslationRequestKind,
    notify_hwnd: isize,
) -> Result<TranslationWorkflowResult> {
    let source_text = match request {
        TranslationRequestKind::Selection => {
            let captured = crate::capture::CaptureService::new(std::time::Duration::from_millis(
                state.settings().clipboard_capture.copy_wait_ms,
            ))
            .capture_selected_text()
            .map_err(|err| AppError::Capture(err.to_string()))?;
            let notify_hwnd =
                windows::Win32::Foundation::HWND(notify_hwnd as *mut core::ffi::c_void);
            crate::ui::translate_window::post_translation_source(
                notify_hwnd,
                crate::ui::translate_window::TranslationSourceMessage {
                    request_id,
                    source_text: captured.text.clone(),
                },
            );
            captured.text
        }
        TranslationRequestKind::WindowText { source_text } => source_text,
    };
    if source_text.trim().is_empty() {
        return Err(AppError::Translate("原文为空".to_string()));
    }
    if state.settings().dictionary_enabled
        && let Some(translated_text) =
            crate::dictionary::lookup(&source_text, &state.settings().target_language)
    {
        return Ok(TranslationWorkflowResult {
            source_text,
            translated_text,
            provider: "本地词典 · ECDICT",
        });
    }
    let translator = build_translator_for_profile(state.active_profile()?)?;
    let response = crate::translator::translate_blocking(
        translator.as_ref(),
        TranslationRequest {
            text: source_text.clone(),
            source_lang: "auto".to_string(),
            target_lang: state.settings().target_language.clone(),
        },
    )?;
    Ok(TranslationWorkflowResult {
        source_text,
        translated_text: response.translated_text,
        provider: response.provider.display_name(),
    })
}

#[cfg(windows)]
fn take_request_id(next_request_id: &mut usize) -> usize {
    let request_id = *next_request_id;
    *next_request_id = next_request_id.wrapping_add(1).max(1);
    request_id
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
    use std::sync::atomic::{AtomicBool, Ordering};

    static CHECK_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

    struct CheckGuard;
    impl Drop for CheckGuard {
        fn drop(&mut self) {
            CHECK_IN_PROGRESS.store(false, Ordering::Release);
        }
    }

    if CHECK_IN_PROGRESS.swap(true, Ordering::AcqRel) {
        return;
    }
    let guard = CheckGuard;
    let notify_hwnd = notify_hwnd.0 as isize;
    let spawned = std::thread::Builder::new()
        .name("update-check".to_string())
        .spawn(move || {
            let _guard = guard;
            let result = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|err| AppError::Translate(format!("启动更新检查运行时失败: {err}")))
                .and_then(|runtime| runtime.block_on(check_for_updates(&current_version)));
            let message = Box::into_raw(Box::new(UpdateCheckMessage {
                result,
                current_version,
                display_mode,
            }));
            let notify_hwnd =
                windows::Win32::Foundation::HWND(notify_hwnd as *mut core::ffi::c_void);
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
    if let Err(err) = spawned {
        tracing::warn!(error = %err, "could not spawn update check thread");
    }
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
    ) && let Err(err) = open_url(owner_hwnd, release_url)
    {
        tracing::warn!(error = %err, "open release page after update confirmation failed");
        let _ = show_runtime_message(
            owner_hwnd,
            "打开失败",
            "无法打开 Release 页面，请稍后重试。",
        );
    }
}

#[cfg(all(windows, not(feature = "portable")))]
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
fn build_translator_for_profile(
    profile: &crate::config::TranslatorProfile,
) -> Result<Box<dyn Translator>> {
    match profile.provider {
        crate::config::TranslatorProvider::Google => Ok(Box::new(
            crate::translator::google_free::GoogleFreeTranslator::new(),
        )),
        crate::config::TranslatorProvider::Custom => {
            let api_key = profile
                .read_api_key()?
                .ok_or_else(|| crate::error::AppError::Translate("API Key 缺失".to_string()))?;
            let translator = crate::translator::openai_compatible::OpenAiCompatibleTranslator::new(
                crate::translator::openai_compatible::OpenAiCompatibleConfig {
                    provider: profile.provider,
                    base_url: profile.base_url.clone(),
                    api_key,
                    model: profile.model.clone(),
                    timeout_secs: profile.timeout_secs,
                },
            )?;
            Ok(Box::new(translator))
        }
    }
}
