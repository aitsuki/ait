use crate::error::{AppError, Result};
use crate::translator::{
    ProviderKind, TranslationErrorKind, TranslationRequest, TranslationResponse, Translator,
};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct OpenAiCompatibleConfig {
    pub provider: ProviderKind,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub timeout_secs: u64,
}

pub struct OpenAiCompatibleTranslator {
    client: reqwest::Client,
    config: OpenAiCompatibleConfig,
}

impl OpenAiCompatibleTranslator {
    pub fn new(config: OpenAiCompatibleConfig) -> Result<Self> {
        if config.api_key.trim().is_empty() {
            return Err(AppError::Translate("API Key 缺失".to_string()));
        }
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(|err| AppError::Network(err.to_string()))?;
        Ok(Self { client, config })
    }

    async fn translate_inner(&self, request: TranslationRequest) -> Result<TranslationResponse> {
        let url = format!(
            "{}/chat/completions",
            self.config.base_url.trim_end_matches('/')
        );
        let body = ChatRequest {
            model: self.config.model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: format!(
                        "Translate the user's text into {}. Return only the translation.",
                        request.target_lang
                    ),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: request.text,
                },
            ],
            temperature: 0.2,
        };

        let response = self
            .client
            .post(url)
            .bearer_auth(&self.config.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|err| AppError::Network(err.to_string()))?;

        let status = response.status();
        if status == StatusCode::UNAUTHORIZED {
            return Err(AppError::Translate(
                TranslationErrorKind::Unauthorized
                    .user_message()
                    .to_string(),
            ));
        }
        if status == StatusCode::TOO_MANY_REQUESTS {
            return Err(AppError::Translate(
                TranslationErrorKind::RateLimited.user_message().to_string(),
            ));
        }
        if !status.is_success() {
            return Err(AppError::Translate(format!(
                "OpenAI 兼容接口失败，状态码: {status}"
            )));
        }

        let body: ChatResponse = response.json().await.map_err(|_| {
            AppError::Translate(
                TranslationErrorKind::InvalidResponse
                    .user_message()
                    .to_string(),
            )
        })?;
        let text = body
            .choices
            .first()
            .map(|choice| choice.message.content.trim().to_string())
            .filter(|text| !text.is_empty())
            .ok_or_else(|| {
                AppError::Translate(
                    TranslationErrorKind::InvalidResponse
                        .user_message()
                        .to_string(),
                )
            })?;

        Ok(TranslationResponse {
            translated_text: text,
            provider: self.config.provider,
        })
    }
}

impl Translator for OpenAiCompatibleTranslator {
    fn translate<'a>(
        &'a self,
        request: TranslationRequest,
    ) -> Pin<Box<dyn Future<Output = Result<TranslationResponse>> + Send + 'a>> {
        Box::pin(self.translate_inner(request))
    }
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
}

#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatChoiceMessage,
}

#[derive(Debug, Deserialize)]
struct ChatChoiceMessage {
    content: String,
}
