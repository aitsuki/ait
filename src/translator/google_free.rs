use crate::error::{AppError, Result};
use crate::translator::{
    ProviderKind, TranslationErrorKind, TranslationRequest, TranslationResponse, Translator,
};
use reqwest::StatusCode;
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

pub struct GoogleFreeTranslator {
    client: reqwest::Client,
    base_url: String,
}

impl Default for GoogleFreeTranslator {
    fn default() -> Self {
        Self::new()
    }
}

impl GoogleFreeTranslator {
    pub fn new() -> Self {
        Self::with_base_url("https://translate.googleapis.com".to_string())
    }

    pub fn with_base_url(base_url: String) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(15))
                .user_agent("ait/0.1")
                .build()
                .expect("reqwest client"),
            base_url,
        }
    }

    async fn translate_inner(&self, request: TranslationRequest) -> Result<TranslationResponse> {
        let url = format!(
            "{}/translate_a/single?client=gtx&sl={}&tl={}&dt=t&q={}",
            self.base_url.trim_end_matches('/'),
            urlencoding::encode(&request.source_lang),
            urlencoding::encode(&request.target_lang),
            urlencoding::encode(&request.text),
        );
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|err| AppError::Network(err.to_string()))?;

        let status = response.status();
        if status == StatusCode::TOO_MANY_REQUESTS {
            return Err(AppError::Translate(
                TranslationErrorKind::RateLimited.user_message().to_string(),
            ));
        }
        if status == StatusCode::FORBIDDEN {
            return Err(AppError::Translate(
                TranslationErrorKind::ProviderChanged
                    .user_message()
                    .to_string(),
            ));
        }
        if !status.is_success() {
            return Err(AppError::Translate(format!(
                "内置 Google 翻译失败，状态码: {status}"
            )));
        }

        let json: Value = response.json().await.map_err(|_| {
            AppError::Translate(
                TranslationErrorKind::InvalidResponse
                    .user_message()
                    .to_string(),
            )
        })?;
        let translated = parse_google_response(&json)?;

        Ok(TranslationResponse {
            translated_text: translated,
            provider: ProviderKind::Google,
        })
    }
}

impl Translator for GoogleFreeTranslator {
    fn translate<'a>(
        &'a self,
        request: TranslationRequest,
    ) -> Pin<Box<dyn Future<Output = Result<TranslationResponse>> + Send + 'a>> {
        Box::pin(self.translate_inner(request))
    }
}

fn parse_google_response(json: &Value) -> Result<String> {
    let segments = json.get(0).and_then(Value::as_array).ok_or_else(|| {
        AppError::Translate(
            TranslationErrorKind::InvalidResponse
                .user_message()
                .to_string(),
        )
    })?;

    let mut out = String::new();
    for segment in segments {
        let text = segment.get(0).and_then(Value::as_str).ok_or_else(|| {
            AppError::Translate(
                TranslationErrorKind::InvalidResponse
                    .user_message()
                    .to_string(),
            )
        })?;
        out.push_str(text);
    }

    if out.trim().is_empty() {
        return Err(AppError::Translate(
            TranslationErrorKind::InvalidResponse
                .user_message()
                .to_string(),
        ));
    }
    Ok(out)
}
