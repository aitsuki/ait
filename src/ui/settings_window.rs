use crate::config::{AppSettings, ProviderKind};
use crate::error::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsViewModel {
    pub default_provider: ProviderKind,
    pub hotkey: String,
    pub openai_base_url: String,
    pub openai_model: String,
    pub has_openai_key: bool,
    pub clipboard_capture_enabled: bool,
    pub copy_wait_ms: u64,
}

impl From<&AppSettings> for SettingsViewModel {
    fn from(settings: &AppSettings) -> Self {
        Self {
            default_provider: settings.default_provider,
            hotkey: settings.hotkey.clone(),
            openai_base_url: settings.openai.base_url.clone(),
            openai_model: settings.openai.model.clone(),
            has_openai_key: settings.openai.encrypted_api_key.is_some(),
            clipboard_capture_enabled: settings.clipboard_capture.enabled,
            copy_wait_ms: settings.clipboard_capture.copy_wait_ms,
        }
    }
}

#[cfg(windows)]
pub struct SettingsWindow;

#[cfg(windows)]
impl SettingsWindow {
    pub fn open(settings: &AppSettings) -> Result<()> {
        let view_model = SettingsViewModel::from(settings);
        tracing::info!(?view_model, "open settings window placeholder");
        Ok(())
    }
}
