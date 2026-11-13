use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TranslatorProvider {
    Google,
    Custom,
}

impl TranslatorProvider {
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Google => "Google",
            Self::Custom => "自定义",
        }
    }

    pub fn as_log_name(self) -> &'static str {
        match self {
            Self::Google => "google",
            Self::Custom => "custom",
        }
    }
}

pub type ProviderKind = TranslatorProvider;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TranslatorProfile {
    pub id: String,
    pub name: String,
    pub provider: TranslatorProvider,
    pub base_url: String,
    pub model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encrypted_api_key: Option<String>,
    pub timeout_secs: u64,
}

impl TranslatorProfile {
    pub fn has_api_key(&self) -> bool {
        self.api_key.is_some() || self.encrypted_api_key.is_some()
    }

    pub fn read_api_key(&self) -> Result<Option<String>> {
        if let Some(key) = &self.api_key {
            return Ok(Some(key.clone()));
        }
        self.encrypted_api_key
            .as_deref()
            .map(|key| self.secret_store().unprotect(key))
            .transpose()
    }

    pub fn set_api_key(&mut self, key: Option<&str>) -> Result<()> {
        let key = key.filter(|key| !key.trim().is_empty());
        if crate::paths::is_portable() {
            self.api_key = key.map(str::to_string);
            self.encrypted_api_key = None;
        } else {
            let encrypted = key
                .map(|key| self.secret_store().protect(key))
                .transpose()?;
            self.encrypted_api_key = encrypted;
            self.api_key = None;
        }
        Ok(())
    }

    fn secret_store(&self) -> crate::secret::SecretStore {
        crate::secret::SecretStore::new(format!("ait-translator-profile-{}", self.id))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClipboardCaptureSettings {
    pub enabled: bool,
    pub open_manual_input_on_failure: bool,
    pub copy_wait_ms: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TranslationWindowLayout {
    #[default]
    FixedSource,
    Free,
    // Retained for loading configurations written before the draggable splitter.
    Equal,
    ThreeToTwo,
    FourToThree,
}

impl TranslationWindowLayout {
    pub const OPTIONS: [Self; 2] = [Self::FixedSource, Self::Free];

    pub fn label(self) -> &'static str {
        match self {
            Self::FixedSource => "原文固定高度",
            _ => "自由调整（拖动分隔条）",
        }
    }

    pub fn index(self) -> usize {
        usize::from(self != Self::FixedSource)
    }

    pub fn from_index(index: usize) -> Self {
        Self::OPTIONS.get(index).copied().unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowSettings {
    pub width: i32,
    pub height: i32,
    #[serde(default)]
    pub layout: TranslationWindowLayout,
    /// Original pane's share of the combined text pane height, in basis points.
    #[serde(default = "default_source_ratio")]
    pub source_ratio: u32,
}

pub const PANE_RATIO_SCALE: u32 = 10_000;

fn default_source_ratio() -> u32 {
    PANE_RATIO_SCALE / 2
}

impl WindowSettings {
    fn normalize(&mut self) {
        match self.layout {
            TranslationWindowLayout::Equal => self.source_ratio = 5_000,
            TranslationWindowLayout::ThreeToTwo => self.source_ratio = 6_000,
            TranslationWindowLayout::FourToThree => self.source_ratio = 5_714,
            _ => {}
        }
        if self.layout != TranslationWindowLayout::FixedSource {
            self.layout = TranslationWindowLayout::Free;
        }
        self.source_ratio = self.source_ratio.clamp(100, PANE_RATIO_SCALE - 100);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarkdownSettings {
    pub render_enabled: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeMode {
    #[default]
    System,
    Light,
    Dark,
}

impl ThemeMode {
    pub const OPTIONS: [Self; 3] = [Self::System, Self::Light, Self::Dark];

    pub fn label(self) -> &'static str {
        match self {
            Self::System => "跟随系统",
            Self::Light => "浅色",
            Self::Dark => "深色",
        }
    }

    pub fn index(self) -> usize {
        Self::OPTIONS
            .iter()
            .position(|&mode| mode == self)
            .unwrap_or(0)
    }

    pub fn from_index(index: usize) -> Self {
        Self::OPTIONS.get(index).copied().unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(default)]
    pub theme: ThemeMode,
    pub default_profile_id: String,
    pub translator_profiles: Vec<TranslatorProfile>,
    pub hotkey: String,
    pub target_language: String,
    #[serde(default)]
    pub dictionary_enabled: bool,
    pub clipboard_capture: ClipboardCaptureSettings,
    pub window: WindowSettings,
    pub markdown: MarkdownSettings,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: ThemeMode::default(),
            default_profile_id: "google".to_string(),
            translator_profiles: builtin_translator_profiles(),
            hotkey: "Ctrl+Alt+E".to_string(),
            target_language: "zh-CN".to_string(),
            dictionary_enabled: false,
            clipboard_capture: ClipboardCaptureSettings {
                enabled: true,
                open_manual_input_on_failure: true,
                copy_wait_ms: 300,
            },
            window: WindowSettings {
                width: 620,
                height: 420,
                layout: TranslationWindowLayout::default(),
                source_ratio: default_source_ratio(),
            },
            markdown: MarkdownSettings {
                render_enabled: false,
            },
        }
    }
}

pub fn builtin_translator_profiles() -> Vec<TranslatorProfile> {
    vec![TranslatorProfile {
        id: "google".to_string(),
        name: "Google".to_string(),
        provider: TranslatorProvider::Google,
        base_url: String::new(),
        model: String::new(),
        api_key: None,
        encrypted_api_key: None,
        timeout_secs: 0,
    }]
}

impl AppSettings {
    pub fn profile_by_id(&self, id: &str) -> Option<&TranslatorProfile> {
        self.translator_profiles
            .iter()
            .find(|profile| profile.id == id)
    }

    pub fn profile_by_id_mut(&mut self, id: &str) -> Option<&mut TranslatorProfile> {
        self.translator_profiles
            .iter_mut()
            .find(|profile| profile.id == id)
    }

    pub fn default_profile(&self) -> Result<&TranslatorProfile> {
        self.profile_by_id(&self.default_profile_id)
            .or_else(|| self.translator_profiles.first())
            .ok_or_else(|| AppError::Config("没有可用的翻译配置".to_string()))
    }

    pub fn normalized(mut self) -> Self {
        self.window.normalize();
        for builtin in builtin_translator_profiles() {
            if self.profile_by_id(&builtin.id).is_none() {
                self.translator_profiles.push(builtin);
            }
        }
        if self.profile_by_id(&self.default_profile_id).is_none() {
            self.default_profile_id = self
                .translator_profiles
                .first()
                .map(|profile| profile.id.clone())
                .unwrap_or_else(|| "google".to_string());
        }
        self
    }

    pub fn add_custom_profile(&mut self) -> TranslatorProfile {
        let id = self.next_custom_profile_id();
        let profile = TranslatorProfile {
            id,
            name: "自定义配置".to_string(),
            provider: TranslatorProvider::Custom,
            base_url: String::new(),
            model: String::new(),
            api_key: None,
            encrypted_api_key: None,
            timeout_secs: 30,
        };
        self.translator_profiles.push(profile.clone());
        profile
    }

    pub fn delete_profile(&mut self, id: &str) -> Result<()> {
        let profile = self
            .profile_by_id(id)
            .ok_or_else(|| AppError::Config("翻译配置不存在".to_string()))?;
        if profile.provider == TranslatorProvider::Google {
            return Err(AppError::Config("Google 翻译配置不能删除".to_string()));
        }
        self.translator_profiles.retain(|profile| profile.id != id);
        if self.default_profile_id == id {
            self.default_profile_id = self
                .translator_profiles
                .first()
                .map(|profile| profile.id.clone())
                .unwrap_or_else(|| "google".to_string());
        }
        Ok(())
    }

    pub fn set_default_profile(&mut self, id: &str) -> Result<()> {
        if self.profile_by_id(id).is_none() {
            return Err(AppError::Config("翻译配置不存在".to_string()));
        }
        self.default_profile_id = id.to_string();
        Ok(())
    }

    fn next_custom_profile_id(&self) -> String {
        let mut index = 1;
        loop {
            let id = if index == 1 {
                "custom".to_string()
            } else {
                format!("custom-{index}")
            };
            if self.profile_by_id(&id).is_none() {
                return id;
            }
            index += 1;
        }
    }
}

pub struct SettingsStore {
    dir: PathBuf,
}

impl SettingsStore {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    pub fn default_dir() -> Result<PathBuf> {
        crate::paths::config_dir()
    }

    pub fn path(&self) -> PathBuf {
        self.dir.join("settings.json")
    }

    pub fn load(&self) -> Result<AppSettings> {
        let path = self.path();
        if !path.exists() {
            return Ok(AppSettings::default());
        }

        let raw = fs::read_to_string(&path)?;
        if let Ok(settings) = serde_json::from_str::<AppSettings>(&raw) {
            return Ok(settings.normalized());
        }
        self.backup_corrupt_file(&path)?;
        Ok(AppSettings::default())
    }

    pub fn save(&self, settings: &AppSettings) -> Result<()> {
        let mut settings = settings.clone();
        for profile in &mut settings.translator_profiles {
            // Convert only when moving between encrypted and portable storage.
            if (crate::paths::is_portable() && profile.encrypted_api_key.is_some())
                || (!crate::paths::is_portable() && profile.api_key.is_some())
            {
                let key = profile.read_api_key()?;
                profile.set_api_key(key.as_deref())?;
            }
        }
        let raw = serde_json::to_string_pretty(&settings)?;
        let path = self.path();
        match fs::read(&path) {
            Ok(existing) if existing == raw.as_bytes() => return Ok(()),
            Ok(_) => {}
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => return Err(err.into()),
        }
        fs::create_dir_all(&self.dir)?;
        fs::write(path, raw)?;
        Ok(())
    }

    fn backup_corrupt_file(&self, path: &Path) -> Result<()> {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|err| AppError::Config(err.to_string()))?
            .as_secs();
        let backup = self.dir.join(format!("settings.json.bak.{ts}"));
        fs::rename(path, backup)?;
        Ok(())
    }
}
