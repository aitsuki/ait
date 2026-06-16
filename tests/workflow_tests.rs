use ait::app::{
    HotkeyAction, TranslationObserver, TranslationWorkflow, TranslationWorkflowResult,
    WorkflowCapture, WorkflowTranslator, hotkey_action,
};
use ait::capture::CapturedText;
use ait::translator::{ProviderKind, TranslationRequest, TranslationResponse};
use ait::ui::translate_window::{
    EditCharAction, EditShortcutAction, ShowAction, ShowMode, TranslationWindowState, WindowZOrder,
    edit_char_action, edit_display_text, edit_shortcut_action, is_third_click_after_double_click,
    paragraph_selection_range_utf16, show_action, translation_window_layout,
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
fn translate_selection_notifies_started_before_capture() {
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
        vec!["started", "capture", "source", "translate", "result"]
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
