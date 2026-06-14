use ait::app::{TranslationWorkflow, WorkflowCapture, WorkflowTranslator};
use ait::capture::CapturedText;
use ait::translator::{ProviderKind, TranslationRequest, TranslationResponse};

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

#[test]
fn translate_selection_captures_then_translates() {
    let workflow = TranslationWorkflow::new(FakeCapture, FakeTranslator);

    let result = workflow.translate_selection("zh-CN").unwrap();

    assert_eq!(result.source_text, "hello");
    assert_eq!(result.translated_text, "你好");
    assert_eq!(result.provider, ProviderKind::GoogleFree);
}
