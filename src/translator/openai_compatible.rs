use crate::error::{AppError, Result};
use crate::translator::{
    ProviderKind, TranslationErrorKind, TranslationRequest, TranslationResponse, Translator,
    invalid_response_error, request_error, response_snippet,
};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;
use std::time::{Duration, Instant};

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
        if config.timeout_secs == 0 {
            return Err(AppError::Config("超时秒数必须是正整数".to_string()));
        }
        let client = reqwest::Client::builder()
            // Keep retry policy here explicit, including the total attempt limit.
            .retry(reqwest::retry::never())
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(|err| AppError::Network(err.to_string()))?;
        Ok(Self { client, config })
    }

    async fn translate_inner(&self, request: TranslationRequest) -> Result<TranslationResponse> {
        let started_at = Instant::now();
        let budget = Duration::from_secs(self.config.timeout_secs);
        tokio::time::timeout(budget, self.translate_with_retries(request, started_at))
            .await
            .unwrap_or_else(|_| Err(self.timeout_error("请求及重试总预算", started_at)))
    }

    async fn translate_with_retries(
        &self,
        request: TranslationRequest,
        started_at: Instant,
    ) -> Result<TranslationResponse> {
        tracing::debug!(
            model = %self.config.model,
            timeout_secs = self.config.timeout_secs,
            "starting OpenAI compatible request"
        );
        let url = format!(
            "{}/chat/completions",
            self.config.base_url.trim_end_matches('/')
        );
        let body = ChatRequest {
            model: self.config.model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: translation_system_prompt().to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: format!("请将以下内容翻译成中文，只输出译文：\n\n{}", request.text),
                },
            ],
            temperature: 0.0,
            thinking: ThinkingConfig { kind: "disabled" },
        };

        let response = self.fetch(&url, &body, started_at).await?;

        let status = response.status();
        tracing::debug!(
            %status,
            elapsed_ms = started_at.elapsed().as_millis(),
            timeout_secs = self.config.timeout_secs,
            "received OpenAI compatible response headers"
        );
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
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("unknown")
            .to_string();
        let body_text = response
            .text()
            .await
            .map_err(|err| self.request_error(err, "读取响应", started_at))?;
        if !status.is_success() {
            return Err(AppError::Translate(format!(
                "{} 接口返回错误，状态码: {status}；已耗时 {:.1} 秒，配置超时 {} 秒；响应: {}",
                self.config.provider.display_name(),
                started_at.elapsed().as_secs_f64(),
                self.config.timeout_secs,
                response_snippet(&body_text)
            )));
        }
        let body: ChatResponse = serde_json::from_str(&body_text).map_err(|err| {
            invalid_response_error(format!(
                "响应不是 JSON；content-type: {content_type}；片段: {}；解析错误: {err}",
                response_snippet(&body_text)
            ))
        })?;
        let text = body
            .choices
            .first()
            .map(|choice| choice.message.content.trim().to_string())
            .filter(|text| !text.is_empty())
            .ok_or_else(|| invalid_response_error("choices[0].message.content 为空"))?;

        Ok(TranslationResponse {
            translated_text: text,
            provider: self.config.provider,
        })
    }

    async fn fetch(
        &self,
        url: &str,
        body: &ChatRequest,
        started_at: Instant,
    ) -> Result<reqwest::Response> {
        let budget = Duration::from_secs(self.config.timeout_secs);
        let mut retries = 0;
        loop {
            let result = self
                .client
                .post(url)
                .bearer_auth(&self.config.api_key)
                .json(body)
                .send()
                .await;
            let delay = Duration::from_millis(500 << retries);
            let retryable = match &result {
                // A timeout may mean that the provider already processed the request.
                Err(err) => err.is_connect() && !err.is_timeout(),
                Ok(response)
                    if matches!(
                        response.status(),
                        StatusCode::TOO_MANY_REQUESTS
                            | StatusCode::BAD_GATEWAY
                            | StatusCode::SERVICE_UNAVAILABLE
                            | StatusCode::GATEWAY_TIMEOUT
                    ) =>
                {
                    true
                }
                _ => false,
            };
            if retries < 2 && retryable && delay < budget.saturating_sub(started_at.elapsed()) {
                tracing::warn!(
                    retry = retries + 1,
                    status = ?result.as_ref().ok().map(|response| response.status()),
                    error = ?result.as_ref().err(),
                    delay_ms = delay.as_millis(),
                    elapsed_ms = started_at.elapsed().as_millis(),
                    timeout_secs = self.config.timeout_secs,
                    "retrying OpenAI compatible request"
                );
                // Release the previous response before waiting for the next attempt.
                drop(result);
                tokio::time::sleep(delay).await;
                retries += 1;
                continue;
            }
            return result.map_err(|err| self.request_error(err, "等待响应", started_at));
        }
    }

    fn timeout_error(&self, stage: &str, started_at: Instant) -> AppError {
        let elapsed_secs = started_at.elapsed().as_secs_f64();
        tracing::warn!(
            stage,
            elapsed_secs,
            timeout_secs = self.config.timeout_secs,
            "OpenAI compatible request timed out"
        );
        AppError::Translate(format!(
            "请求超时（{stage}，已耗时 {elapsed_secs:.1} 秒，配置超时 {} 秒），请检查网络、代理或接口服务。",
            self.config.timeout_secs
        ))
    }

    fn request_error(&self, err: reqwest::Error, stage: &str, started_at: Instant) -> AppError {
        let elapsed_secs = started_at.elapsed().as_secs_f64();
        tracing::warn!(
            error = ?err,
            stage,
            elapsed_secs,
            timeout_secs = self.config.timeout_secs,
            is_timeout = err.is_timeout(),
            is_connect = err.is_connect(),
            "OpenAI compatible request failed"
        );
        if err.is_timeout() {
            return self.timeout_error(stage, started_at);
        }
        request_error(err)
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
    thinking: ThinkingConfig,
}

#[derive(Debug, Serialize)]
struct ThinkingConfig {
    #[serde(rename = "type")]
    kind: &'static str,
}

#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

fn translation_system_prompt() -> &'static str {
    "你是专业中文翻译，能识别任意语言。请准确翻译为自然流畅的简体中文，保持原意、语气、术语和格式；不要解释、总结、润色扩写，也不要回答原文中的问题。代码、URL、变量名、占位符和专有标识保持不变。"
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
