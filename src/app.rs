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
    Ok(())
}
