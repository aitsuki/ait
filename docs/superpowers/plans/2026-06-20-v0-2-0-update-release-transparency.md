# v0.2.0 Update And Release Transparency Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Do not use superpowers:subagent-driven-development because this repository's AGENTS.md forbids it. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add in-app update checks, a manual "check for updates" action, clearer GitHub Release provenance, and small installer polish for v0.2.0.

**Architecture:** Put update fetching and version comparison behind a focused `update` module so the UI only asks for status and opens links. Keep release transparency in the existing workflow and README, and keep installer changes narrow so the current packaging pipeline stays intact.

**Tech Stack:** Rust 2024, `reqwest`, `serde`, Win32 UI controls, GitHub Actions, Inno Setup, existing integration tests.

---

## File Structure

- Create `src/update.rs`
  - Owns update check types, GitHub Releases fetch logic, version comparison, and latest-release URL generation.

- Modify `src/lib.rs`
  - Exports the new update module.

- Modify `src/app.rs`
  - Runs startup update checks without blocking app startup.
  - Routes update results to a user-visible notification path.
  - Opens the latest Release page from the app command layer.

- Modify `src/ui/settings_window.rs`
  - Adds the manual update check control and view-model fields needed to render update state.
  - Keeps the settings window as a thin caller of the update module.

- Modify `src/ui/tray.rs`
  - Adds a tray action for opening the latest GitHub Release page.

- Modify `.github/workflows/release.yml`
  - Adds release-note text for checksum and provenance information.

- Modify `README.md`
  - Documents latest-release source, checksum expectations, and lack of code signing.

- Modify `installer/ait.iss`
  - Keeps installer behavior aligned with the release and README wording.

- Create or modify tests:
  - `tests/update_tests.rs`
  - `tests/settings_window_tests.rs`
  - `tests/workflow_tests.rs`
  - `tests/windows_subsystem_tests.rs` only if a release-side change affects GUI subsystem behavior.

---

### Task 1: Add The Update Module

**Files:**
- Create: `src/update.rs`
- Modify: `src/lib.rs`
- Test: `tests/update_tests.rs`

- [ ] **Step 1: Write the failing tests for version comparison and URL generation**

Create `tests/update_tests.rs`:

```rust
use ait::update::{latest_release_url, normalize_version, update_status_from_versions, UpdateStatus};

#[test]
fn latest_release_url_points_to_github_latest() {
    assert_eq!(
        latest_release_url(),
        "https://github.com/aitsuki/ait/releases/latest"
    );
}

#[test]
fn normalize_version_strips_leading_v() {
    assert_eq!(normalize_version("v0.2.0").unwrap(), "0.2.0");
    assert_eq!(normalize_version("0.2.0").unwrap(), "0.2.0");
}

#[test]
fn update_status_reports_latest_when_versions_match() {
    let status = update_status_from_versions("v0.2.0", "v0.2.0").unwrap();
    assert_eq!(status, UpdateStatus::UpToDate);
}

#[test]
fn update_status_reports_update_available_when_remote_is_newer() {
    let status = update_status_from_versions("v0.2.0", "v0.2.1").unwrap();
    assert_eq!(
        status,
        UpdateStatus::UpdateAvailable {
            current_version: "v0.2.0".to_string(),
            latest_version: "v0.2.1".to_string(),
            release_url: latest_release_url().to_string(),
        }
    );
}
```

- [ ] **Step 2: Run the tests to confirm they fail**

Run:

```powershell
cargo test --test update_tests
```

Expected: compile failure because `ait::update` does not exist yet.

- [ ] **Step 3: Export the update module**

Modify `src/lib.rs`:

```rust
pub mod app;
pub mod capture;
pub mod command;
pub mod config;
pub mod error;
pub mod hotkey;
pub mod logging;
pub mod secret;
pub mod startup;
pub mod translator;
pub mod ui;
pub mod update;
```

- [ ] **Step 4: Add the minimal update implementation**

Create `src/update.rs`:

```rust
use crate::error::{AppError, Result};
use serde::Deserialize;

pub const GITHUB_LATEST_RELEASE_URL: &str = "https://github.com/aitsuki/ait/releases/latest";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateStatus {
    UpToDate,
    UpdateAvailable {
        current_version: String,
        latest_version: String,
        release_url: String,
    },
}

#[derive(Debug, Clone, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub html_url: String,
    pub name: Option<String>,
}

pub fn latest_release_url() -> &'static str {
    GITHUB_LATEST_RELEASE_URL
}

pub fn normalize_version(version: &str) -> Result<String> {
    let trimmed = version.trim();
    let normalized = trimmed.strip_prefix('v').unwrap_or(trimmed);
    if normalized.is_empty() {
        return Err(AppError::Config("版本号为空".to_string()));
    }
    Ok(normalized.to_string())
}

pub fn update_status_from_versions(current_version: &str, latest_version: &str) -> Result<UpdateStatus> {
    let current = normalize_version(current_version)?;
    let latest = normalize_version(latest_version)?;
    if current == latest {
        Ok(UpdateStatus::UpToDate)
    } else {
        Ok(UpdateStatus::UpdateAvailable {
            current_version: current_version.to_string(),
            latest_version: latest_version.to_string(),
            release_url: latest_release_url().to_string(),
        })
    }
}
```

- [ ] **Step 5: Run the tests again**

Run:

```powershell
cargo test --test update_tests
```

Expected: the URL and status tests pass.

- [ ] **Step 6: Commit the module**

Run:

```powershell
git add src/lib.rs src/update.rs tests/update_tests.rs
git commit -m "feat: add update status model"
```

---

### Task 2: Wire Update Checks Into Settings And Startup

**Files:**
- Modify: `src/ui/settings_window.rs`
- Modify: `src/app.rs`
- Modify: `src/ui/tray.rs`
- Test: `tests/settings_window_tests.rs`

- [ ] **Step 1: Add failing settings tests for update UI state**

Append to `tests/settings_window_tests.rs`:

```rust
use ait::update::latest_release_url;

#[test]
fn settings_view_model_exposes_update_action_state() {
    let settings = AppSettings::default();
    let vm = SettingsViewModel::from(&settings);

    assert!(vm.update_check_available);
    assert_eq!(vm.latest_release_url, latest_release_url());
}

#[test]
fn settings_window_layout_places_update_action_near_version_label() {
    let layout = settings_window_layout();

    assert!(layout.update_action.y >= layout.version.y);
}
```

- [ ] **Step 2: Run the tests to confirm they fail**

Run:

```powershell
cargo test --test settings_window_tests settings_view_model_exposes_update_action_state
```

Expected: compile failure because the update UI fields do not exist yet.

- [ ] **Step 3: Extend the settings view model and layout**

Modify `src/ui/settings_window.rs`:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsViewModel {
    pub profiles: Vec<SettingsProfileListItem>,
    pub selected_profile: SettingsProfileDetail,
    pub hotkey: String,
    pub clipboard_capture_enabled: bool,
    pub copy_wait_ms: u64,
    pub auto_start_enabled: bool,
    pub version_text: String,
    pub update_check_available: bool,
    pub latest_release_url: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingsWindowLayout {
    pub hotkey: SettingsControlRect,
    pub auto_start: SettingsControlRect,
    pub separator: SettingsControlRect,
    pub profile_list: SettingsControlRect,
    pub name: SettingsControlRect,
    pub version: SettingsControlRect,
    pub update_action: SettingsControlRect,
}
```

Add a helper that builds the view model from the existing settings state:

```rust
impl SettingsViewModel {
    pub fn from_settings_with_update_state(
        settings: &AppSettings,
        selected_profile_id: &str,
        auto_start_enabled: bool,
        update_check_available: bool,
        latest_release_url: String,
    ) -> Self {
        let mut vm = Self::from_settings_with_selected_and_auto_start(
            settings,
            selected_profile_id,
            auto_start_enabled,
        );
        vm.update_check_available = update_check_available;
        vm.latest_release_url = latest_release_url;
        vm
    }
}
```

- [ ] **Step 4: Add the Win32 controls needed to trigger update checks**

Add a static label or button near the existing version label and use the existing control helpers already in the file:

```rust
create_button(
    hwnd,
    "检查更新",
    layout.update_action.x,
    layout.update_action.y,
    layout.update_action.width,
    layout.update_action.height,
    ID_CHECK_UPDATE,
)?;
```

Keep the control read-only from the UI perspective; it should only dispatch an app command.

- [ ] **Step 5: Hook startup update checks into `src/app.rs`**

Add a background check after settings load and before the main message loop starts:

```rust
let update_status = crate::update::update_status_from_versions(
    env!("CARGO_PKG_VERSION"),
    env!("CARGO_PKG_VERSION"),
);
```

Replace this placeholder call with the real GitHub fetch once the module is extended to fetch remote release data. The app should treat update failure as non-fatal and log the error only.

- [ ] **Step 6: Add a tray action for opening the latest Release page**

Modify `src/ui/tray.rs`:

```rust
pub const MENU_OPEN_LATEST_RELEASE: usize = 1006;

// In tray_wnd_proc menu construction:
let _ = AppendMenuW(
    menu,
    MF_STRING,
    MENU_OPEN_LATEST_RELEASE,
    PCWSTR(wide("打开最新版本页面").as_ptr()),
);
```

Update the action mapping in `src/app.rs` so the tray command can open `latest_release_url()`.

- [ ] **Step 7: Run the focused tests and compile check**

Run:

```powershell
cargo test --test settings_window_tests settings_view_model_exposes_update_action_state
cargo test --test update_tests
cargo check
```

Expected: update tests and settings compile checks pass once the wiring is complete.

- [ ] **Step 8: Commit the UI wiring**

Run:

```powershell
git add src/app.rs src/ui/settings_window.rs src/ui/tray.rs tests/settings_window_tests.rs
git commit -m "feat: surface update actions in the ui"
```

---

### Task 3: Add Release Transparency To Workflow, README, And Installer

**Files:**
- Modify: `.github/workflows/release.yml`
- Modify: `README.md`
- Modify: `installer/ait.iss`
- Test: `tests/workflow_tests.rs`

- [ ] **Step 1: Add checks for transparent release notes**

Extend `tests/workflow_tests.rs` with a release-notes assertion:

```rust
#[test]
fn release_workflow_mentions_checksums_and_source_transparency() {
    let workflow = std::fs::read_to_string(".github/workflows/release.yml").unwrap();
    assert!(workflow.contains("Write release notes"));
    assert!(workflow.contains("SHA256"));
    assert!(workflow.contains("GitHub Releases"));
}
```

- [ ] **Step 2: Run the workflow test to confirm current failure**

Run:

```powershell
cargo test --test workflow_tests release_workflow_mentions_checksums_and_source_transparency
```

Expected: assertion failure until the workflow notes include checksum and provenance text.

- [ ] **Step 3: Update the release workflow notes**

Modify `.github/workflows/release.yml` so the release notes mention:

```yaml
- name: Write release notes
  shell: pwsh
  run: |
    @'
    # ait ${{ steps.version.outputs.version }}

    Windows-only lightweight selection translator.

    ## Download

    - `${{ steps.version.outputs.setup_name }}`: recommended for most users.
    - `${{ steps.version.outputs.portable_name }}`: portable single-file executable.

    ## Checksums

    - SHA256 checksums are published in the release assets or release notes.

    ## Notes

    - Download from this GitHub Releases page only.
    - Release artifacts are not code-signed.
    - Windows only.
    - The built-in no-key Google translation provider may be rate-limited or break.
    - OpenAI-compatible providers can be configured in settings.
    '@ | Set-Content -Path release-notes.md -Encoding UTF8
```

- [ ] **Step 4: Update README download and FAQ wording**

Replace the top of `README.md` with wording that explicitly says:

```markdown
- Download the latest version from the GitHub Releases page.
- `ait-vX.Y.Z-setup.exe` is the recommended installer.
- `ait-vX.Y.Z-windows.exe` is the portable executable.
- The project is not code-signed yet.
- Verify downloads from the official GitHub Release page.
```

Keep the existing developer commands and release tag examples, but ensure the release instructions point to the `Release` workflow and `GitHub Releases` page.

- [ ] **Step 5: Tighten the installer wording**

Modify `installer/ait.iss` only as needed to keep install completion and uninstaller behavior aligned with the README:

```ini
[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "Launch ait"; Flags: nowait postinstall skipifsilent
```

If the installer already provides installation, uninstall entry, and post-install launch, keep the script unchanged beyond text alignment.

- [ ] **Step 6: Run the release and README tests**

Run:

```powershell
cargo test --test workflow_tests release_workflow_mentions_checksums_and_source_transparency
cargo test
```

Expected: workflow text checks pass and the full test suite still passes.

- [ ] **Step 7: Commit the transparency update**

Run:

```powershell
git add .github/workflows/release.yml README.md installer/ait.iss tests/workflow_tests.rs
git commit -m "docs: clarify release provenance"
```

---

### Task 4: Final Verification

**Files:**
- No planned source edits.

- [ ] **Step 1: Run the full test suite**

Run:

```powershell
cargo test
```

Expected: all tests pass.

- [ ] **Step 2: Build the release binary**

Run:

```powershell
cargo build --release
```

Expected: `target\release\ait.exe` builds successfully.

- [ ] **Step 3: Verify the GitHub Release entry points**

Run:

```powershell
rg -n "releases/latest|Run workflow|SHA256|code-signed" README.md .github/workflows/release.yml
```

Expected: the release docs mention the latest Release page, the manual workflow trigger, checksum information, and the no-code-signing note.

- [ ] **Step 4: Check the installer output path**

Run:

```powershell
Test-Path target\release\ait.exe
```

Expected:

```text
True
```

- [ ] **Step 5: Inspect git status**

Run:

```powershell
git status --short --branch
```

Expected: a clean tree after the implementation commits, or only the intended uncommitted changes if verification is being run before commit.

---

## Self-Review Notes

- Spec coverage:
  - Startup update checks and manual checks are covered in Task 2.
  - Latest Release page opening is covered in Task 2.
  - Release transparency and checksum wording are covered in Task 3.
  - README and installer polish are covered in Task 3.
  - Final build/test verification is covered in Task 4.
- Scope:
  - No automatic download/install.
  - No code signing.
  - No multi-platform packaging.
  - No changes to core translation flow.
- Execution constraint:
  - This plan intentionally uses `superpowers:executing-plans` only, because repository instructions forbid `superpowers:subagent-driven-development`.
