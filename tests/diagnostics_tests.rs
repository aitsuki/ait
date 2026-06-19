use std::path::PathBuf;

use ait::config::{AppSettings, TranslatorProvider};
use ait::diagnostics::DiagnosticInfo;

#[test]
fn diagnostic_text_contains_useful_context() {
    let settings = AppSettings::default();
    let info = DiagnosticInfo::from_parts(
        &settings,
        PathBuf::from(r"C:\Users\tester\AppData\Roaming\ait"),
        PathBuf::from(r"C:\Users\tester\AppData\Local\ait\logs"),
        Ok(true),
    );

    let text = info.to_clipboard_text();

    assert!(text.contains("ait 诊断信息"));
    assert!(text.contains("版本: ait v"));
    assert!(text.contains("操作系统: Windows"));
    assert!(text.contains(r"配置目录: C:\Users\tester\AppData\Roaming\ait"));
    assert!(text.contains(r"日志目录: C:\Users\tester\AppData\Local\ait\logs"));
    assert!(text.contains("默认翻译配置: Google (google)"));
    assert!(text.contains("快捷键: Ctrl+Alt+E"));
    assert!(text.contains("开机自启: 开启"));
}

#[test]
fn diagnostic_text_does_not_include_secrets_or_translation_content() {
    let mut settings = AppSettings::default();
    let profile = settings.profile_by_id_mut("openai").unwrap();
    profile.provider = TranslatorProvider::OpenAi;
    profile.name = "Private OpenAI".to_string();
    profile.encrypted_api_key = Some("SECRET_ENCRYPTED_API_KEY".to_string());
    settings.default_profile_id = "openai".to_string();

    let info = DiagnosticInfo::from_parts(
        &settings,
        PathBuf::from(r"C:\config"),
        PathBuf::from(r"C:\logs"),
        Err("registry denied".to_string()),
    );

    let text = info.to_clipboard_text();

    assert!(text.contains("默认翻译配置: Private OpenAI (openai)"));
    assert!(text.contains("开机自启: 读取失败"));
    assert!(!text.contains("SECRET_ENCRYPTED_API_KEY"));
    assert!(!text.to_lowercase().contains("api key"));
    assert!(!text.contains("原文"));
    assert!(!text.contains("译文"));
}
