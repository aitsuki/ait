# v0.1.3 Diagnostics And Errors Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:executing-plans` to implement this plan task-by-task. Do not use `superpowers:subagent-driven-development`; this repository's `AGENTS.md` forbids it. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add small, user-triggered diagnostic tools and make common runtime errors show actionable user-facing summaries.

**Architecture:** Keep diagnostic text generation pure and testable in a new `diagnostics` module. Reuse the existing logging directory rules from `logging.rs`, the existing Windows clipboard backend for copying diagnostics, and the existing tray/settings window patterns for native UI actions. Display concise error summaries in the translation window while preserving detailed errors in logs.

**Tech Stack:** Rust 2024, `windows` crate 0.62, Win32 tray/menu/window controls, existing `cargo test` integration tests.

---

## File Structure

- Modify `src/logging.rs`
  - Add `log_dir() -> Result<PathBuf>` and make `init_logging()` call it.
- Create `src/diagnostics.rs`
  - Own `DiagnosticInfo`, diagnostic collection from saved settings, and safe clipboard text formatting.
- Modify `src/lib.rs`
  - Export the new `diagnostics` module.
- Create `tests/diagnostics_tests.rs`
  - Test diagnostic text contents and sensitive-data exclusion.
- Modify `tests/logging_tests.rs`
  - Test reusable log directory behavior.
- Modify `src/error.rs`
  - Refine `AppError::user_summary()` for API key and common translation failures.
- Modify `src/ui/translate_window.rs`
  - Use `user_summary()` for errors shown in the translation window.
- Modify `tests/workflow_tests.rs`
  - Add pure state-level tests for translated error summaries and tray action mapping.
- Modify `src/ui/tray.rs`
  - Add the `打开日志目录` menu item and stable menu ID.
- Modify `src/app.rs`
  - Add `TrayAction::OpenLogDirectory`, map the menu ID, and open the log directory with Windows Shell.
- Modify `src/ui/settings_window.rs`
  - Add the `复制诊断信息` button and wire it to diagnostic collection plus clipboard copy.
- Modify `tests/settings_window_tests.rs`
  - Add layout test for the diagnostic button.
- Modify `Cargo.toml` and `Cargo.lock`
  - Bump package version to `0.1.3`.
- Modify `README.md`
  - Update download/release examples to `v0.1.3` and add a troubleshooting note for opening logs and copying diagnostics.

---

### Task 1: Reusable Log Directory And Diagnostic Text

**Files:**
- Modify: `src/logging.rs`
- Modify: `src/lib.rs`
- Create: `src/diagnostics.rs`
- Modify: `tests/logging_tests.rs`
- Create: `tests/diagnostics_tests.rs`

- [ ] **Step 1: Write failing logging and diagnostics tests**

Append to `tests/logging_tests.rs`:

```rust
#[test]
fn log_dir_uses_logs_subdirectory() {
    let dir = ait::logging::log_dir().unwrap();

    assert_eq!(dir.file_name().and_then(|name| name.to_str()), Some("logs"));
}
```

Create `tests/diagnostics_tests.rs`:

```rust
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
```

- [ ] **Step 2: Run tests to verify RED**

Run:

```powershell
cargo test --test logging_tests log_dir_uses_logs_subdirectory
cargo test --test diagnostics_tests
```

Expected: compile failure because `logging::log_dir`, `diagnostics`, and `DiagnosticInfo` do not exist.

- [ ] **Step 3: Add reusable log directory helper**

Modify `src/logging.rs`:

```rust
use crate::error::{AppError, Result};
use std::path::PathBuf;

pub fn log_dir() -> Result<PathBuf> {
    let project_dirs = directories::ProjectDirs::from("dev", "aitsu", "ait")
        .ok_or_else(|| AppError::Config("无法定位日志目录".to_string()))?;
    Ok(project_dirs.data_local_dir().join("logs"))
}

pub fn init_logging() -> Result<PathBuf> {
    let log_dir = log_dir()?;
    std::fs::create_dir_all(&log_dir)?;

    let file_appender = tracing_appender::rolling::daily(&log_dir, "ait.log");
    tracing_subscriber::fmt()
        .with_writer(file_appender)
        .with_ansi(false)
        .with_target(false)
        .init();

    Ok(log_dir)
}

pub fn safe_text_len(text: &str) -> usize {
    text.chars().count()
}
```

- [ ] **Step 4: Export and implement diagnostics module**

Modify `src/lib.rs`:

```rust
pub mod app;
pub mod capture;
pub mod command;
pub mod config;
pub mod diagnostics;
pub mod error;
pub mod hotkey;
pub mod logging;
pub mod secret;
pub mod startup;
pub mod translator;
pub mod ui;
```

Create `src/diagnostics.rs`:

```rust
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
            .or_else(|_| settings.translator_profiles.first().ok_or_else(|| {
                crate::error::AppError::Config("没有可用的翻译配置".to_string())
            }))
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
```

- [ ] **Step 5: Run focused tests to verify GREEN**

Run:

```powershell
cargo test --test logging_tests log_dir_uses_logs_subdirectory
cargo test --test diagnostics_tests
```

Expected: all focused tests pass.

- [ ] **Step 6: Commit Task 1**

```powershell
git add src/lib.rs src/logging.rs src/diagnostics.rs tests/logging_tests.rs tests/diagnostics_tests.rs
git commit -m "feat: add diagnostic info text"
```

---

### Task 2: User-Facing Error Summaries

**Files:**
- Modify: `src/error.rs`
- Modify: `src/ui/translate_window.rs`
- Modify: `tests/workflow_tests.rs`

- [ ] **Step 1: Add failing error summary tests**

Append to `tests/workflow_tests.rs`:

```rust
#[test]
fn app_error_user_summaries_are_actionable() {
    assert_eq!(
        ait::error::AppError::Capture("clipboard busy".to_string()).user_summary(),
        "没有取到选中文本，可以手动粘贴文本后重试。"
    );
    assert_eq!(
        ait::error::AppError::Network("timeout".to_string()).user_summary(),
        "网络连接失败，请检查网络或代理设置后重试。"
    );
    assert_eq!(
        ait::error::AppError::Secret("decrypt failed".to_string()).user_summary(),
        "API Key 读取失败，请重新保存接口配置。"
    );
    assert_eq!(
        ait::error::AppError::Translate("API Key 缺失".to_string()).user_summary(),
        "翻译失败：API Key 缺失，请在设置中填写 API Key。"
    );
}

#[test]
fn translation_window_state_uses_user_summary_for_app_error() {
    let state = TranslationWindowState {
        source_text: "hello".to_string(),
        translated_text: String::new(),
        loading: true,
        error: None,
    };

    let next = state.with_app_error(&ait::error::AppError::Network("timeout".to_string()));

    assert!(!next.loading);
    assert_eq!(
        next.error.as_deref(),
        Some("网络连接失败，请检查网络或代理设置后重试。")
    );
}
```

- [ ] **Step 2: Run tests to verify RED**

Run:

```powershell
cargo test --test workflow_tests app_error_user_summaries_are_actionable translation_window_state_uses_user_summary_for_app_error
```

Expected: `translation_window_state_uses_user_summary_for_app_error` fails to compile because `with_app_error` does not exist; the network/API key summary assertions may also fail.

- [ ] **Step 3: Refine `AppError::user_summary()`**

Modify `src/error.rs`:

```rust
impl AppError {
    pub fn user_summary(&self) -> String {
        match self {
            AppError::Hotkey(_) => "快捷键注册失败，请在设置中更换快捷键。".to_string(),
            AppError::Capture(_) => "没有取到选中文本，可以手动粘贴文本后重试。".to_string(),
            AppError::Translate(msg) if msg.contains("API Key 缺失") => {
                "翻译失败：API Key 缺失，请在设置中填写 API Key。".to_string()
            }
            AppError::Translate(msg) => format!("翻译失败：{msg}"),
            AppError::Network(_) => "网络连接失败，请检查网络或代理设置后重试。".to_string(),
            AppError::Secret(_) => "API Key 读取失败，请重新保存接口配置。".to_string(),
            AppError::Config(_) => "配置读取失败，已尝试恢复默认配置。".to_string(),
            AppError::Windows(_) | AppError::Io(_) | AppError::Json(_) => self.to_string(),
        }
    }
}
```

- [ ] **Step 4: Add state helper and use summaries in the window**

Modify `impl TranslationWindowState` in `src/ui/translate_window.rs`:

```rust
    pub fn with_app_error(mut self, err: &crate::error::AppError) -> Self {
        self.loading = false;
        self.error = Some(err.user_summary());
        self
    }
```

Modify `finish_translation_result` in `src/ui/translate_window.rs`:

```rust
    pub fn finish_translation_result(
        &mut self,
        result: crate::error::Result<crate::app::TranslationWorkflowResult>,
    ) -> Result<()> {
        match result {
            Ok(result) => self.show_result(&result),
            Err(err) => {
                tracing::warn!(error = %err, "show translation error summary");
                self.show_error(err.user_summary())
            }
        }
    }
```

- [ ] **Step 5: Run focused tests**

Run:

```powershell
cargo test --test workflow_tests app_error_user_summaries_are_actionable translation_window_state_uses_user_summary_for_app_error
```

Expected: both tests pass.

- [ ] **Step 6: Commit Task 2**

```powershell
git add src/error.rs src/ui/translate_window.rs tests/workflow_tests.rs
git commit -m "fix: show actionable runtime errors"
```

---

### Task 3: Tray Menu Opens Log Directory

**Files:**
- Modify: `src/ui/tray.rs`
- Modify: `src/app.rs`
- Modify: `tests/workflow_tests.rs`

- [ ] **Step 1: Add failing tray action test**

Append to `tests/workflow_tests.rs`:

```rust
#[test]
fn tray_open_logs_menu_id_maps_to_open_log_directory_action() {
    assert_eq!(
        ait::app::tray_action_from_menu_id(ait::ui::tray::MENU_OPEN_LOG_DIRECTORY),
        ait::app::TrayAction::OpenLogDirectory
    );
}
```

Replace the old `removed_logs_menu_id_is_not_actionable` expectation with:

```rust
#[test]
fn legacy_logs_menu_id_is_not_reused() {
    assert_eq!(
        ait::app::tray_action_from_menu_id(1003),
        ait::app::TrayAction::Unknown
    );
}
```

- [ ] **Step 2: Run test to verify RED**

Run:

```powershell
cargo test --test workflow_tests tray_open_logs_menu_id_maps_to_open_log_directory_action legacy_logs_menu_id_is_not_reused
```

Expected: compile failure because `MENU_OPEN_LOG_DIRECTORY` and `TrayAction::OpenLogDirectory` do not exist.

- [ ] **Step 3: Add tray menu ID and menu item**

Modify `src/ui/tray.rs` near the existing IDs:

```rust
#[cfg(windows)]
pub const MENU_SHOW_TRANSLATION_WINDOW: usize = 1001;
#[cfg(windows)]
pub const MENU_SETTINGS: usize = 1002;
#[cfg(windows)]
pub const MENU_OPEN_LOG_DIRECTORY: usize = 1005;
#[cfg(windows)]
pub const MENU_EXIT: usize = 1004;
```

In `tray_wnd_proc`, after the `设置` item and before the separator, add:

```rust
            let _ = AppendMenuW(
                menu,
                MF_STRING,
                MENU_OPEN_LOG_DIRECTORY,
                PCWSTR(wide("打开日志目录").as_ptr()),
            );
```

- [ ] **Step 4: Map tray action and handle it**

Modify `TrayAction` in `src/app.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayAction {
    ShowTranslationWindow,
    OpenSettings,
    OpenLogDirectory,
    Exit,
    Unknown,
}
```

Modify `tray_action_from_menu_id`:

```rust
pub fn tray_action_from_menu_id(menu_id: usize) -> TrayAction {
    match menu_id {
        crate::ui::tray::MENU_SHOW_TRANSLATION_WINDOW => TrayAction::ShowTranslationWindow,
        crate::ui::tray::MENU_SETTINGS => TrayAction::OpenSettings,
        crate::ui::tray::MENU_OPEN_LOG_DIRECTORY => TrayAction::OpenLogDirectory,
        crate::ui::tray::MENU_EXIT => TrayAction::Exit,
        _ => TrayAction::Unknown,
    }
}
```

Add this Windows helper near `show_runtime_message`:

```rust
#[cfg(windows)]
fn open_directory(path: &std::path::Path) -> Result<()> {
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
    use windows::core::PCWSTR;

    let operation = wide("open");
    let file = wide(&path.to_string_lossy());
    let result = unsafe {
        ShellExecuteW(
            None,
            PCWSTR(operation.as_ptr()),
            PCWSTR(file.as_ptr()),
            PCWSTR::null(),
            PCWSTR::null(),
            SW_SHOWNORMAL,
        )
    };
    if result.0 as isize <= 32 {
        return Err(crate::error::AppError::Windows(
            "打开日志目录失败".to_string(),
        ));
    }
    Ok(())
}
```

In the `WM_TRAY_COMMAND` match in `src/app.rs`, add:

```rust
                    TrayAction::OpenLogDirectory => {
                        match crate::logging::log_dir().and_then(|dir| {
                            std::fs::create_dir_all(&dir)?;
                            open_directory(&dir)
                        }) {
                            Ok(()) => {}
                            Err(err) => {
                                tracing::warn!(error = %err, "open log directory failed");
                                show_runtime_message(
                                    translation_window.hwnd(),
                                    "打开失败",
                                    "无法打开日志目录，请稍后重试。",
                                );
                            }
                        }
                    }
```

- [ ] **Step 5: Run focused tests and compile check**

Run:

```powershell
cargo test --test workflow_tests tray_open_logs_menu_id_maps_to_open_log_directory_action legacy_logs_menu_id_is_not_reused
cargo check
```

Expected: tests pass and compile succeeds.

- [ ] **Step 6: Commit Task 3**

```powershell
git add src/ui/tray.rs src/app.rs tests/workflow_tests.rs
git commit -m "feat: open logs from tray"
```

---

### Task 4: Settings Button Copies Diagnostics

**Files:**
- Modify: `src/ui/settings_window.rs`
- Modify: `tests/settings_window_tests.rs`

- [ ] **Step 1: Add failing layout test**

Append to `tests/settings_window_tests.rs`:

```rust
#[test]
fn settings_window_layout_places_diagnostics_button_near_save_actions() {
    let layout = settings_window_layout();

    assert!(layout.diagnostics.x > layout.version.x);
    assert_eq!(layout.diagnostics.y, 382);
    assert!(layout.diagnostics.width >= 110);
}
```

- [ ] **Step 2: Run test to verify RED**

Run:

```powershell
cargo test --test settings_window_tests settings_window_layout_places_diagnostics_button_near_save_actions
```

Expected: compile failure because `SettingsWindowLayout` has no `diagnostics` field.

- [ ] **Step 3: Add layout field and button ID**

Modify `src/ui/settings_window.rs` near the IDs:

```rust
#[cfg(windows)]
const ID_COPY_DIAGNOSTICS: isize = 3006;
```

Modify `SettingsWindowLayout`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingsWindowLayout {
    pub hotkey: SettingsControlRect,
    pub auto_start: SettingsControlRect,
    pub separator: SettingsControlRect,
    pub profile_list: SettingsControlRect,
    pub name: SettingsControlRect,
    pub version: SettingsControlRect,
    pub diagnostics: SettingsControlRect,
}
```

Modify `settings_window_layout()` by adding:

```rust
        diagnostics: SettingsControlRect {
            x: 398,
            y: 382,
            width: 120,
            height: 28,
        },
```

- [ ] **Step 4: Create the button**

In `SettingsWindow::open`, before the existing `保存` and `取消` buttons, add:

```rust
            create_button(
                hwnd,
                "复制诊断信息",
                layout.diagnostics.x,
                layout.diagnostics.y,
                layout.diagnostics.width,
                layout.diagnostics.height,
                ID_COPY_DIAGNOSTICS,
            )?;
```

- [ ] **Step 5: Wire button click to clipboard copy**

In `default_wnd_proc`, before the `ID_SAVE` branch, add:

```rust
        if command == ID_COPY_DIAGNOSTICS as usize {
            match unsafe { copy_diagnostics_from_window(hwnd) } {
                Ok(()) => unsafe { show_message(hwnd, "已复制", "诊断信息已复制。") },
                Err(err) => {
                    tracing::warn!(error = %err, "copy diagnostics failed");
                    unsafe {
                        show_message(
                            hwnd,
                            "复制失败",
                            "复制诊断信息失败，请打开日志目录后反馈日志文件。",
                        )
                    };
                }
            }
            return LRESULT(0);
        }
```

Add this helper near `save_settings_from_window`:

```rust
#[cfg(windows)]
unsafe fn copy_diagnostics_from_window(hwnd: windows::Win32::Foundation::HWND) -> Result<()> {
    use windows::Win32::UI::WindowsAndMessaging::{GWLP_USERDATA, GetWindowLongPtrW};

    let ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) };
    if ptr == 0 {
        return Err(AppError::Config("设置窗口状态缺失".to_string()));
    }
    let settings = unsafe { &*(ptr as *const AppSettings) };
    let text = crate::diagnostics::DiagnosticInfo::collect(settings).to_clipboard_text();
    crate::capture::ClipboardBackend::write_text(&crate::capture::WindowsClipboardBackend, &text)
}
```

- [ ] **Step 6: Run focused tests and compile check**

Run:

```powershell
cargo test --test settings_window_tests settings_window_layout_places_diagnostics_button_near_save_actions
cargo check
```

Expected: focused test passes and compile succeeds.

- [ ] **Step 7: Commit Task 4**

```powershell
git add src/ui/settings_window.rs tests/settings_window_tests.rs
git commit -m "feat: copy diagnostics from settings"
```

---

### Task 5: Version Bump And Troubleshooting Docs

**Files:**
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`
- Modify: `README.md`
- Modify: `tests/settings_window_tests.rs`

- [ ] **Step 1: Update version expectation test**

In `tests/settings_window_tests.rs`, replace the existing `app_version_text_uses_v0_1_2` test with:

```rust
#[test]
fn app_version_text_uses_v0_1_3() {
    assert_eq!(app_version_text(), "ait v0.1.3");
}
```

- [ ] **Step 2: Run version test to verify RED**

Run:

```powershell
cargo test --test settings_window_tests app_version_text_uses_v0_1_3
```

Expected: fails because `Cargo.toml` still says `0.1.2`.

- [ ] **Step 3: Bump Cargo package version**

Modify `Cargo.toml`:

```toml
[package]
name = "ait"
version = "0.1.3"
edition = "2024"
```

Modify the `ait` package entry in `Cargo.lock`:

```toml
[[package]]
name = "ait"
version = "0.1.3"
```

- [ ] **Step 4: Update README download and release examples**

Replace `v0.1.2` examples in `README.md` with `v0.1.3`:

```text
ait-v0.1.3-setup.exe
ait-v0.1.3-windows.exe
```

Update the workflow and tag examples:

```powershell
git tag v0.1.3
git push origin v0.1.3
```

- [ ] **Step 5: Update README troubleshooting section**

Add this FAQ entry after the default Google failure question in `README.md`:

```markdown
### 遇到问题时怎么反馈？

可以先在托盘菜单点击 `打开日志目录`，找到最近的日志文件。

也可以打开 `设置`，点击 `复制诊断信息`，把复制出来的内容和日志一起反馈。诊断信息不会包含 API Key、原文或译文。
```

- [ ] **Step 6: Run version test**

Run:

```powershell
cargo test --test settings_window_tests app_version_text_uses_v0_1_3
```

Expected: test passes.

- [ ] **Step 7: Commit Task 5**

```powershell
git add Cargo.toml Cargo.lock README.md tests/settings_window_tests.rs
git commit -m "chore: bump version to 0.1.3"
```

---

### Task 6: Final Verification

**Files:**
- No planned source edits.

- [ ] **Step 1: Run full test suite**

Run:

```powershell
cargo test
```

Expected: all tests pass.

- [ ] **Step 2: Run release build**

Run:

```powershell
cargo build --release
```

Expected: release build succeeds.

- [ ] **Step 3: Manual Windows smoke test**

Run:

```powershell
.\target\release\ait.exe
```

Manual checks:

- Right-click the tray icon and confirm `打开日志目录` appears under `设置`.
- Click `打开日志目录`; File Explorer opens the logs directory.
- Open `设置`; confirm `复制诊断信息` appears near the bottom actions.
- Click `复制诊断信息`; a success message appears.
- Paste into Notepad and confirm the text contains version, config directory, log directory, default provider, hotkey, and startup state.
- Confirm the pasted text does not contain API Key, encrypted API Key, source text, or translated text.
- Trigger a network/API-key failure and confirm the translation window shows a concise user-facing message while details are still written to the log.

- [ ] **Step 4: Inspect git status**

```powershell
git status --short --branch
```

Expected: clean working tree on `main`, ahead by the implementation commits.
