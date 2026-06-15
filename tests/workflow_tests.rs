use ait::app::{
    HotkeyAction, TranslationObserver, TranslationWorkflow, TranslationWorkflowResult,
    WorkflowCapture, WorkflowTranslator, hotkey_action,
};
use ait::capture::CapturedText;
use ait::translator::{ProviderKind, TranslationRequest, TranslationResponse};
use ait::ui::translate_window::{ShowAction, ShowMode, WindowZOrder, show_action, window_z_order};
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
