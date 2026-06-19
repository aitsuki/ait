use ait::app::{
    HotkeyAction, HotkeyRegistrationUpdate, TranslationObserver, TranslationRequestKind,
    TranslationWorkflow, TranslationWorkflowResult, WorkflowCapture, WorkflowTranslator,
    hotkey_action, hotkey_registration_update, run_translation_request_with_observer,
    translation_task_action,
};
use ait::capture::CapturedText;
use ait::config::AppSettings;
use ait::translator::{ProviderKind, TranslationRequest, TranslationResponse};
use ait::ui::translate_window::{
    EditCharAction, EditShortcutAction, ProfileSelectionAction, ShowAction, ShowMode,
    TranslationProfileOption, TranslationWindowState, WindowZOrder, edit_char_action,
    edit_display_text, edit_shortcut_action, is_third_click_after_double_click,
    paragraph_selection_range_utf16, profile_selection_action, show_action,
    show_window_needs_topmost_raise, show_window_needs_topmost_reset, show_window_z_order,
    translation_profile_combo_dropdown_height, translation_window_layout,
    translation_window_min_client_size, window_z_order,
};
use std::cell::RefCell;

struct FakeCapture;

impl WorkflowCapture for FakeCapture {
    fn capture(&self) -> ait::error::Result<CapturedText> {
        Ok(CapturedText {
            text: "hello".to_string(),
        })
    }
}

struct FakeTranslator;

impl WorkflowTranslator for FakeTranslator {
    fn translate_blocking(
        &self,
        request: TranslationRequest,
    ) -> ait::error::Result<TranslationResponse> {
        assert_eq!(request.text, "hello");
        Ok(TranslationResponse {
            translated_text: "你好".to_string(),
            provider: ProviderKind::GoogleFree,
        })
    }
}

struct FormattingCapture;

impl WorkflowCapture for FormattingCapture {
    fn capture(&self) -> ait::error::Result<CapturedText> {
        Ok(CapturedText {
            text: "\nfirst paragraph\n\n\nsecond paragraph\n".to_string(),
        })
    }
}

struct FormattingTranslator;

impl WorkflowTranslator for FormattingTranslator {
    fn translate_blocking(
        &self,
        request: TranslationRequest,
    ) -> ait::error::Result<TranslationResponse> {
        assert_eq!(request.text, "\nfirst paragraph\n\n\nsecond paragraph\n");
        Ok(TranslationResponse {
            translated_text: "translated".to_string(),
            provider: ProviderKind::GoogleFree,
        })
    }
}

struct PanicCapture;

impl WorkflowCapture for PanicCapture {
    fn capture(&self) -> ait::error::Result<CapturedText> {
        panic!("capture must not run for direct text translation");
    }
}

struct RecordingCapture<'a> {
    events: &'a RefCell<Vec<&'static str>>,
}

impl WorkflowCapture for RecordingCapture<'_> {
    fn capture(&self) -> ait::error::Result<CapturedText> {
        self.events.borrow_mut().push("capture");
        Ok(CapturedText {
            text: "hello".to_string(),
        })
    }
}

struct RecordingTranslator<'a> {
    events: &'a RefCell<Vec<&'static str>>,
}

impl WorkflowTranslator for RecordingTranslator<'_> {
    fn translate_blocking(
        &self,
        request: TranslationRequest,
    ) -> ait::error::Result<TranslationResponse> {
        assert_eq!(request.text, "hello");
        self.events.borrow_mut().push("translate");
        Ok(TranslationResponse {
            translated_text: "你好".to_string(),
            provider: ProviderKind::GoogleFree,
        })
    }
}

struct RecordingObserver<'a> {
    events: &'a RefCell<Vec<&'static str>>,
}

impl TranslationObserver for RecordingObserver<'_> {
    fn translation_started(&mut self) -> ait::error::Result<()> {
        self.events.borrow_mut().push("started");
        Ok(())
    }

    fn source_captured(&mut self, source_text: &str) -> ait::error::Result<()> {
        assert_eq!(source_text, "hello");
        self.events.borrow_mut().push("source");
        Ok(())
    }

    fn translation_succeeded(
        &mut self,
        result: &TranslationWorkflowResult,
    ) -> ait::error::Result<()> {
        assert_eq!(result.translated_text, "你好");
        self.events.borrow_mut().push("result");
        Ok(())
    }
}

#[test]
fn translate_selection_captures_then_translates() {
    let workflow = TranslationWorkflow::new(FakeCapture, FakeTranslator);

    let result = workflow.translate_selection("zh-CN").unwrap();

    assert_eq!(result.source_text, "hello");
    assert_eq!(result.translated_text, "你好");
    assert_eq!(result.provider, ProviderKind::GoogleFree);
}

#[test]
fn translate_text_translates_without_capture() {
    let workflow = TranslationWorkflow::new(PanicCapture, FakeTranslator);

    let result = workflow.translate_text("hello", "zh-CN").unwrap();

    assert_eq!(result.source_text, "hello");
    assert_eq!(result.translated_text, "你好");
    assert_eq!(result.provider, ProviderKind::GoogleFree);
}

#[test]
fn translate_text_rejects_empty_source() {
    let workflow = TranslationWorkflow::new(PanicCapture, FakeTranslator);

    let err = workflow.translate_text("   ", "zh-CN").unwrap_err();

    assert!(err.to_string().contains("原文为空"));
}

#[test]
fn translate_selection_preserves_captured_paragraph_spacing() {
    let workflow = TranslationWorkflow::new(FormattingCapture, FormattingTranslator);

    let result = workflow.translate_selection("zh-CN").unwrap();

    assert_eq!(
        result.source_text,
        "\nfirst paragraph\n\n\nsecond paragraph\n"
    );
}

#[test]
fn translate_selection_captures_before_notifying_started() {
    let events = RefCell::new(Vec::new());
    let workflow = TranslationWorkflow::new(
        RecordingCapture { events: &events },
        RecordingTranslator { events: &events },
    );
    let mut observer = RecordingObserver { events: &events };

    let result = workflow
        .translate_selection_with_observer("zh-CN", &mut observer)
        .unwrap();

    assert_eq!(result.translated_text, "你好");
    assert_eq!(
        events.into_inner(),
        vec!["capture", "started", "source", "translate", "result"]
    );
}

#[test]
fn selection_translation_task_reports_source_before_translating() {
    let events = RefCell::new(Vec::new());
    let workflow = TranslationWorkflow::new(
        RecordingCapture { events: &events },
        RecordingTranslator { events: &events },
    );
    let mut observer = RecordingObserver { events: &events };

    let result = run_translation_request_with_observer(
        &workflow,
        TranslationRequestKind::Selection,
        "zh-CN",
        &mut observer,
    )
    .unwrap();

    assert_eq!(result.translated_text, "你好");
    assert_eq!(
        events.into_inner(),
        vec!["capture", "started", "source", "translate", "result"]
    );
}

#[test]
fn translation_starting_window_does_not_take_focus() {
    assert!(!ShowMode::Starting.activates_window());
    assert!(ShowMode::Loading.activates_window());
    assert!(ShowMode::Result.activates_window());
    assert!(ShowMode::Error.activates_window());
}

#[test]
fn translation_starting_window_is_temporarily_shown_above_foreground_app() {
    assert_eq!(
        show_window_z_order(ShowMode::Starting),
        WindowZOrder::TopmostNoActivate
    );
    assert_eq!(
        show_window_z_order(ShowMode::Result),
        WindowZOrder::NotTopmost
    );
}

#[test]
fn visible_translation_window_clears_temporary_topmost_after_starting_state() {
    assert!(!show_window_needs_topmost_reset(
        ShowMode::Starting,
        ShowAction::KeepPosition
    ));
    assert!(show_window_needs_topmost_reset(
        ShowMode::Loading,
        ShowAction::KeepPosition
    ));
    assert!(show_window_needs_topmost_reset(
        ShowMode::Result,
        ShowAction::ActivateOnly
    ));
    assert!(show_window_needs_topmost_reset(
        ShowMode::Error,
        ShowAction::ActivateOnly
    ));
}

#[test]
fn starting_window_is_raised_without_activation_when_already_visible() {
    assert!(show_window_needs_topmost_raise(
        ShowMode::Starting,
        ShowAction::ActivateOnly
    ));
    assert!(!show_window_needs_topmost_raise(
        ShowMode::Starting,
        ShowAction::KeepPosition
    ));
    assert!(!show_window_needs_topmost_raise(
        ShowMode::Loading,
        ShowAction::ActivateOnly
    ));
}

#[test]
fn translation_window_completion_keeps_source_text() {
    let mut state = TranslationWindowState {
        source_text: String::new(),
        translated_text: String::new(),
        loading: true,
        error: Some("old error".to_string()),
    };
    let result = TranslationWorkflowResult {
        source_text: "hello".to_string(),
        translated_text: "你好".to_string(),
        provider: ProviderKind::GoogleFree,
    };

    state.apply_translation_result(&result);

    assert_eq!(state.source_text, "hello");
    assert_eq!(state.translated_text, "你好");
    assert!(!state.loading);
    assert_eq!(state.error, None);
}

#[test]
fn translation_starting_state_preserves_existing_text() {
    let mut state = TranslationWindowState {
        source_text: "previous source".to_string(),
        translated_text: "previous translation".to_string(),
        loading: false,
        error: Some("old error".to_string()),
    };

    state.mark_starting();

    assert_eq!(state.source_text, "previous source");
    assert_eq!(state.translated_text, "previous translation");
    assert!(state.loading);
    assert_eq!(state.error, None);
}

#[test]
fn active_translation_window_stays_where_user_put_it() {
    assert_eq!(show_action(false, false), ShowAction::PositionAndActivate);
    assert_eq!(show_action(true, true), ShowAction::KeepPosition);
}

#[test]
fn visible_background_translation_window_stays_where_user_put_it() {
    assert_eq!(show_action(true, false), ShowAction::ActivateOnly);
}

#[test]
fn global_hotkey_retranslates_visible_background_translation_window() {
    assert_eq!(hotkey_action(false), HotkeyAction::TranslateSelection);
}

#[test]
fn global_hotkey_is_ignored_while_translation_window_is_foreground() {
    assert_eq!(hotkey_action(true), HotkeyAction::Ignore);
    assert_eq!(hotkey_action(false), HotkeyAction::TranslateSelection);
}

#[test]
fn hotkey_translation_runs_as_selection_task() {
    assert_eq!(
        translation_task_action(true, ""),
        TranslationRequestKind::Selection
    );
}

#[test]
fn hotkey_registration_update_noops_when_hotkey_is_unchanged() {
    assert_eq!(
        hotkey_registration_update("Ctrl+Alt+E", "Ctrl+Alt+E", Ok(())),
        HotkeyRegistrationUpdate::Unchanged
    );
}

#[test]
fn hotkey_registration_update_accepts_changed_registered_hotkey() {
    assert_eq!(
        hotkey_registration_update("Ctrl+Alt+E", "Ctrl+Alt+T", Ok(())),
        HotkeyRegistrationUpdate::Changed {
            hotkey: "Ctrl+Alt+T".to_string()
        }
    );
}

#[test]
fn hotkey_registration_update_keeps_old_hotkey_when_registration_fails() {
    assert_eq!(
        hotkey_registration_update(
            "Ctrl+Alt+E",
            "Ctrl+Alt+T",
            Err("注册快捷键失败: already registered".to_string())
        ),
        HotkeyRegistrationUpdate::Rejected {
            rollback_hotkey: "Ctrl+Alt+E".to_string(),
            message: "快捷键注册失败，请换一个组合键；当前仍使用原来的快捷键。注册快捷键失败: already registered"
                .to_string()
        }
    );
}

#[test]
fn window_translation_runs_as_text_task() {
    assert_eq!(
        translation_task_action(false, "hello"),
        TranslationRequestKind::WindowText {
            source_text: "hello".to_string()
        }
    );
}

#[test]
fn translation_window_is_not_topmost_without_pin_feature() {
    assert_eq!(window_z_order(), WindowZOrder::NotTopmost);
}

#[test]
fn translation_window_layout_resizes_content_with_client_area() {
    let small = translation_window_layout(620, 420);
    let large = translation_window_layout(820, 900);

    assert_eq!(small.source_edit.height, small.translated_edit.height);
    assert!(large.source_edit.width > small.source_edit.width);
    assert!(large.translated_edit.width > small.translated_edit.width);
    assert!(large.source_edit.height > small.source_edit.height);
    assert!(large.translated_edit.height > small.translated_edit.height);
    assert!(large.source_edit.height < large.translated_edit.height);
    assert!(large.status_text.y > small.status_text.y);
    assert!(large.translate_button.x > small.translate_button.x);
}

#[test]
fn translation_window_layout_keeps_controls_inside_small_client_area() {
    let layout = translation_window_layout(180, 160);

    for rect in [
        layout.profile_combo,
        layout.source_label,
        layout.source_edit,
        layout.translated_label,
        layout.translated_edit,
        layout.status_text,
        layout.translate_button,
    ] {
        assert!(rect.width > 0);
        assert!(rect.height > 0);
        assert!(rect.x + rect.width <= 180);
        assert!(rect.y + rect.height <= 160);
    }
}

#[test]
fn translation_profile_combo_keeps_dropdown_height() {
    let layout = translation_window_layout(620, 420);

    assert_eq!(layout.profile_combo.height, 26);
    assert_eq!(translation_profile_combo_dropdown_height(), 220);
}

#[test]
fn selecting_profile_with_source_requests_save_and_retranslate() {
    assert_eq!(
        profile_selection_action("openai", "hello"),
        ProfileSelectionAction::SaveDefaultAndRetranslate {
            profile_id: "openai".to_string()
        }
    );
}

#[test]
fn selecting_profile_with_empty_source_only_saves_default() {
    assert_eq!(
        profile_selection_action("deepseek", "  "),
        ProfileSelectionAction::SaveDefaultOnly {
            profile_id: "deepseek".to_string()
        }
    );
}

#[test]
fn profile_options_mark_active_profile() {
    let settings = AppSettings::default();
    let options = TranslationProfileOption::from_settings(&settings, "google");

    assert_eq!(options[0].id, "google");
    assert_eq!(options[0].label, "Google");
    assert!(options[0].active);
    assert!(options.iter().any(|option| option.label == "DeepSeek"));
}

#[test]
fn runtime_select_profile_updates_default_profile() {
    let mut state = ait::app::AppRuntimeState::new(AppSettings::default());

    state.select_profile("deepseek").unwrap();

    assert_eq!(state.active_profile_id(), "deepseek");
    assert_eq!(state.settings().default_profile_id, "deepseek");
}

#[test]
fn profile_switch_action_preserves_source_on_error() {
    let state = TranslationWindowState {
        source_text: "hello".to_string(),
        translated_text: "old translation".to_string(),
        loading: false,
        error: None,
    };

    let next = state
        .clone()
        .with_profile_switch_error("API Key 缺失".to_string());

    assert_eq!(next.source_text, "hello");
    assert_eq!(next.translated_text, "old translation");
    assert_eq!(next.error.as_deref(), Some("API Key 缺失"));
    assert!(!next.loading);
}

#[test]
fn app_error_user_summaries_are_actionable() {
    assert_eq!(
        ait::error::AppError::Capture("clipboard busy".to_string()).user_summary(),
        "没有取到选中文本，可以手动粘贴文本后重试。"
    );
    assert_eq!(
        ait::error::AppError::Network("timeout".to_string()).user_summary(),
        "网络连接失败，请检查网络或代理设置后重试。"
    );
    assert_eq!(
        ait::error::AppError::Secret("decrypt failed".to_string()).user_summary(),
        "API Key 读取失败，请重新保存接口配置。"
    );
    assert_eq!(
        ait::error::AppError::Translate("API Key 缺失".to_string()).user_summary(),
        "翻译失败：API Key 缺失，请在设置中填写 API Key。"
    );
    assert_eq!(
        ait::error::AppError::Translate(
            "翻译服务返回了无法识别的数据。详情: 响应不是 JSON。片段: <html>blocked</html>".to_string()
        )
        .user_summary(),
        "翻译服务返回了无法识别的数据。"
    );
}

#[test]
fn translation_window_state_uses_user_summary_for_app_error() {
    let state = TranslationWindowState {
        source_text: "hello".to_string(),
        translated_text: String::new(),
        loading: true,
        error: None,
    };

    let next = state.with_app_error(&ait::error::AppError::Network("timeout".to_string()));

    assert!(!next.loading);
    assert_eq!(
        next.error.as_deref(),
        Some("网络连接失败，请检查网络或代理设置后重试。")
    );
}

#[test]
fn translation_window_has_minimum_resizable_client_area() {
    assert_eq!(translation_window_min_client_size(), (420, 300));
}

#[test]
fn tray_show_window_menu_id_maps_to_show_window_action() {
    assert_eq!(
        ait::app::tray_action_from_menu_id(ait::ui::tray::MENU_SHOW_TRANSLATION_WINDOW),
        ait::app::TrayAction::ShowTranslationWindow
    );
}

#[test]
fn tray_open_logs_menu_id_maps_to_open_log_directory_action() {
    assert_eq!(
        ait::app::tray_action_from_menu_id(ait::ui::tray::MENU_OPEN_LOG_DIRECTORY),
        ait::app::TrayAction::OpenLogDirectory
    );
}

#[test]
fn legacy_logs_menu_id_is_not_reused() {
    assert_eq!(
        ait::app::tray_action_from_menu_id(1003),
        ait::app::TrayAction::Unknown
    );
}

#[test]
fn release_workflow_mentions_checksums_and_source_transparency() {
    let workflow = std::fs::read_to_string(".github/workflows/release.yml").unwrap();
    assert!(workflow.contains("Write release notes"));
    assert!(workflow.contains("SHA256"));
    assert!(workflow.contains("GitHub Releases"));
    assert!(workflow.contains("Release artifacts are not code-signed."));
}

#[test]
fn edit_shortcut_action_handles_ctrl_a_and_escape() {
    assert_eq!(
        edit_shortcut_action(0x41, true),
        EditShortcutAction::SelectAll
    );
    assert_eq!(
        edit_shortcut_action(0x1B, false),
        EditShortcutAction::HideWindow
    );
    assert_eq!(edit_shortcut_action(0x42, false), EditShortcutAction::None);
}

#[test]
fn edit_char_action_swallows_ctrl_a_control_character() {
    assert_eq!(edit_char_action(0x01), EditCharAction::Swallow);
    assert_eq!(edit_char_action('a' as u32), EditCharAction::Default);
}

#[test]
fn edit_display_text_normalizes_newlines_for_windows_multiline_edit() {
    assert_eq!(edit_display_text("one\ntwo"), "one\r\ntwo");
    assert_eq!(edit_display_text("one\r\ntwo"), "one\r\ntwo");
    assert_eq!(
        edit_display_text("one\r\ntwo\n\nthree"),
        "one\r\ntwo\r\n\r\nthree"
    );
    assert_eq!(edit_display_text("one\u{2028}two"), "one\r\ntwo");
    assert_eq!(edit_display_text("one\u{2029}two"), "one\r\n\r\ntwo");
    assert_eq!(edit_display_text("one\u{000B}two"), "one\r\ntwo");
    assert_eq!(edit_display_text("one\u{000C}two"), "one\r\ntwo");
    assert_eq!(edit_display_text("one\u{0085}two"), "one\r\ntwo");
}

#[test]
fn paragraph_selection_range_selects_current_paragraph() {
    let text: Vec<u16> = "first paragraph\r\nsecond paragraph\nthird"
        .encode_utf16()
        .collect();

    assert_eq!(paragraph_selection_range_utf16(&text, 2), (0, 15));
    assert_eq!(paragraph_selection_range_utf16(&text, 18), (17, 33));
    assert_eq!(paragraph_selection_range_utf16(&text, 35), (34, 39));
}

#[test]
fn paragraph_selection_range_handles_empty_and_out_of_bounds() {
    let empty: Vec<u16> = Vec::new();
    assert_eq!(paragraph_selection_range_utf16(&empty, 12), (0, 0));

    let text: Vec<u16> = "alpha\nbeta".encode_utf16().collect();
    assert_eq!(paragraph_selection_range_utf16(&text, 200), (6, 10));
}

#[test]
fn third_click_is_detected_after_recent_double_click() {
    assert!(is_third_click_after_double_click(Some(100), 250, 500));
    assert!(!is_third_click_after_double_click(Some(100), 700, 500));
    assert!(!is_third_click_after_double_click(None, 250, 500));
}
