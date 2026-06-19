use std::path::PathBuf;

use crate::config::AppSettings;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticInfo {
    pub app_version: String,
    pub os: String,
    pub config_dir: PathBuf,
    pub log_dir: PathBuf,
    pub default_profile: String,
    pub default_provider: String,
    pub hotkey: String,
    pub auto_start: String,
}

impl DiagnosticInfo {
    pub fn from_parts(
        settings: &AppSettings,
        config_dir: PathBuf,
        log_dir: PathBuf,
        auto_start: std::result::Result<bool, String>,
    ) -> Self {
        let profile = settings
            .default_profile()
            .or_else(|_| {
                settings.translator_profiles.first().ok_or_else(|| {
                    crate::error::AppError::Config("没有可用的翻译配置".to_string())
                })
            })
            .expect("default settings always contain profiles");

        Self {
            app_version: format!("ait v{}", env!("CARGO_PKG_VERSION")),
            os: "Windows".to_string(),
            config_dir,
            log_dir,
            default_profile: profile.name.clone(),
            default_provider: profile.provider.as_log_name().to_string(),
            hotkey: settings.hotkey.clone(),
            auto_start: match auto_start {
                Ok(true) => "开启".to_string(),
                Ok(false) => "关闭".to_string(),
                Err(_) => "读取失败".to_string(),
            },
        }
    }

    pub fn collect(settings: &AppSettings) -> Self {
        let config_dir = crate::config::SettingsStore::default_dir()
            .unwrap_or_else(|_| PathBuf::from("读取失败"));
        let log_dir = crate::logging::log_dir().unwrap_or_else(|_| PathBuf::from("读取失败"));
        let auto_start = crate::startup::is_auto_start_enabled().map_err(|err| err.to_string());
        Self::from_parts(settings, config_dir, log_dir, auto_start)
    }

    pub fn to_clipboard_text(&self) -> String {
        format!(
            "ait 诊断信息\n版本: {}\n操作系统: {}\n配置目录: {}\n日志目录: {}\n默认翻译配置: {} ({})\n快捷键: {}\n开机自启: {}",
            self.app_version,
            self.os,
            self.config_dir.display(),
            self.log_dir.display(),
            self.default_profile,
            self.default_provider,
            self.hotkey,
            self.auto_start
        )
    }
}
