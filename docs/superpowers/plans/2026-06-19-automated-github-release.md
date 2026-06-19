# Automated GitHub Release Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Do not use superpowers:subagent-driven-development because `AGENTS.md` forbids it. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build an automated GitHub Release flow that publishes both a portable Windows exe and an Inno Setup installer.

**Architecture:** The release process lives in GitHub Actions and runs on `windows-latest`. The installer definition lives in a dedicated Inno Setup script under `installer/`, while release usage instructions live in `README.md`.

**Tech Stack:** Rust/Cargo, GitHub Actions, PowerShell, Inno Setup, GitHub CLI.

---

## File Structure

- Create `.github/workflows/release.yml`
  - Owns the release automation.
  - Supports manual `workflow_dispatch` and `v*.*.*` tag pushes.
  - Validates version strings, runs tests, builds release exe, builds installer, checks that the Release does not already exist, and uploads Release assets.

- Create `installer/ait.iss`
  - Owns the Windows installer definition.
  - Installs `ait.exe`, creates a Start Menu shortcut, creates a standard uninstaller entry, and offers to launch the app after install.

- Modify `README.md`
  - Documents which Release asset ordinary users should download.
  - Documents maintainer release steps using either the GitHub Actions button or a git tag.

No Rust source files need to change.

---

### Task 1: Add Inno Setup Installer Script

**Files:**
- Create: `installer/ait.iss`

- [ ] **Step 1: Create the installer directory**

Run:

```powershell
New-Item -ItemType Directory -Force -Path installer
```

Expected: PowerShell creates `installer` if it does not already exist.

- [ ] **Step 2: Add `installer/ait.iss`**

Create `installer/ait.iss` with this exact content:

```ini
#define MyAppName "ait"
#define MyAppVersion GetEnv("AIT_VERSION")
#define MyAppExeName "ait.exe"
#define MyAppSourceExe GetEnv("AIT_SOURCE_EXE")
#define MyAppOutputDir GetEnv("AIT_OUTPUT_DIR")
#define MyAppOutputBase GetEnv("AIT_OUTPUT_BASE")

[Setup]
AppId={{8F5939A4-77A7-4EE9-9E1F-A01E7E728437}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher=ait
DefaultDirName={autopf}\ait
DefaultGroupName=ait
DisableProgramGroupPage=yes
OutputDir={#MyAppOutputDir}
OutputBaseFilename={#MyAppOutputBase}
Compression=lzma
SolidCompression=yes
WizardStyle=modern
ArchitecturesInstallIn64BitMode=x64
PrivilegesRequired=lowest
UninstallDisplayIcon={app}\{#MyAppExeName}

[Files]
Source: "{#MyAppSourceExe}"; DestDir: "{app}"; DestName: "{#MyAppExeName}"; Flags: ignoreversion

[Icons]
Name: "{group}\ait"; Filename: "{app}\{#MyAppExeName}"

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "Launch ait"; Flags: nowait postinstall skipifsilent
```

- [ ] **Step 3: Verify installer script has no unresolved environment variable names**

Run:

```powershell
rg -n "AIT_VERSION|AIT_SOURCE_EXE|AIT_OUTPUT_DIR|AIT_OUTPUT_BASE" installer\ait.iss
```

Expected: Output shows only the four `GetEnv(...)` lines near the top of `installer/ait.iss`.

- [ ] **Step 4: Commit installer script**

Run:

```powershell
git add installer\ait.iss
git commit -m "build: add windows installer script"
```

Expected: Commit succeeds with one new file.

---

### Task 2: Add GitHub Release Workflow

**Files:**
- Create: `.github/workflows/release.yml`

- [ ] **Step 1: Create the workflow directory**

Run:

```powershell
New-Item -ItemType Directory -Force -Path .github\workflows
```

Expected: PowerShell creates `.github/workflows` if it does not already exist.

- [ ] **Step 2: Add `.github/workflows/release.yml`**

Create `.github/workflows/release.yml` with this exact content:

```yaml
name: Release

on:
  workflow_dispatch:
    inputs:
      version:
        description: "Release version, for example v0.1.0"
        required: true
        type: string
  push:
    tags:
      - "v*.*.*"

permissions:
  contents: write

jobs:
  release:
    name: Build and publish release
    runs-on: windows-latest

    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Resolve version
        id: version
        shell: pwsh
        run: |
          if ("${{ github.event_name }}" -eq "workflow_dispatch") {
            $version = "${{ inputs.version }}"
          } else {
            $version = "${{ github.ref_name }}"
          }

          if ($version -notmatch '^v\d+\.\d+\.\d+$') {
            Write-Error "Version must use vX.Y.Z format, got '$version'."
            exit 1
          }

          $plainVersion = $version.Substring(1)
          "version=$version" >> $env:GITHUB_OUTPUT
          "plain_version=$plainVersion" >> $env:GITHUB_OUTPUT
          "portable_name=ait-$version-windows.exe" >> $env:GITHUB_OUTPUT
          "setup_name=ait-$version-setup.exe" >> $env:GITHUB_OUTPUT

      - name: Test
        run: cargo test

      - name: Build release executable
        run: cargo build --release

      - name: Prepare portable executable
        shell: pwsh
        run: |
          New-Item -ItemType Directory -Force -Path dist | Out-Null
          Copy-Item target\release\ait.exe "dist\${{ steps.version.outputs.portable_name }}"

      - name: Install Inno Setup
        shell: pwsh
        run: choco install innosetup --no-progress -y

      - name: Build installer
        shell: pwsh
        run: |
          $env:AIT_VERSION = "${{ steps.version.outputs.plain_version }}"
          $env:AIT_SOURCE_EXE = (Resolve-Path "target\release\ait.exe").Path
          $env:AIT_OUTPUT_DIR = (Resolve-Path "dist").Path
          $env:AIT_OUTPUT_BASE = "ait-${{ steps.version.outputs.version }}-setup"
          & "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe" installer\ait.iss

      - name: Write release notes
        shell: pwsh
        run: |
          @"
          # ait ${{ steps.version.outputs.version }}

          Windows-only lightweight selection translator.

          ## Download

          - `${{ steps.version.outputs.setup_name }}`: recommended for most users.
          - `${{ steps.version.outputs.portable_name }}`: portable single-file executable.

          ## Notes

          - Windows only.
          - The built-in no-key Google translation provider may be rate-limited or break.
          - OpenAI-compatible providers can be configured in settings.
          "@ | Set-Content -Path release-notes.md -Encoding UTF8

      - name: Publish GitHub Release
        shell: pwsh
        env:
          GH_TOKEN: ${{ github.token }}
        run: |
          $version = "${{ steps.version.outputs.version }}"

          gh release view $version --json tagName 2>$null
          if ($LASTEXITCODE -eq 0) {
            Write-Error "Release '$version' already exists."
            exit 1
          }

          gh release create $version `
            "dist/${{ steps.version.outputs.portable_name }}#${{ steps.version.outputs.portable_name }}" `
            "dist/${{ steps.version.outputs.setup_name }}#${{ steps.version.outputs.setup_name }}" `
            --title "ait $version" `
            --notes-file release-notes.md `
            --target "${{ github.sha }}"
```

- [ ] **Step 3: Verify workflow contains both release triggers**

Run:

```powershell
rg -n "workflow_dispatch|tags:|v\\*\\.\\*\\.\\*" .github\workflows\release.yml
```

Expected: Output includes `workflow_dispatch`, `tags:`, and `"v*.*.*"`.

- [ ] **Step 4: Verify workflow uploads both assets**

Run:

```powershell
rg -n "portable_name|setup_name|gh release create|gh release view" .github\workflows\release.yml
```

Expected: Output shows both `portable_name` and `setup_name` definitions, the existing Release check, and the GitHub CLI publish command.

- [ ] **Step 5: Commit release workflow**

Run:

```powershell
git add .github\workflows\release.yml
git commit -m "ci: add automated release workflow"
```

Expected: Commit succeeds with one new workflow file.

---

### Task 3: Document Release Downloads and Maintainer Flow

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Replace README with updated user and maintainer instructions**

Replace `README.md` with this exact content:

```markdown
# ait

Windows-only lightweight selection translator.

## Download

Download the latest version from the GitHub Releases page.

- `ait-vX.Y.Z-setup.exe`: recommended for most users. Installs ait and adds a Start Menu shortcut.
- `ait-vX.Y.Z-windows.exe`: portable single-file executable. Download and run it directly.

No zip extraction is required.

## MVP Behavior

- Tray app, no main window.
- Default hotkey: `Ctrl+Alt+E`.
- Text capture uses clipboard copy and only promises to restore text clipboard content.
- Default translation provider is an unofficial no-key Google Translate endpoint.
- OpenAI-compatible APIs can be configured as an optional provider.
- API keys are protected with Windows DPAPI.

## Build

```powershell
cargo build
```

## Run

```powershell
cargo run
```

## Tests

```powershell
cargo test
```

## Release

There are two supported release paths.

Manual GitHub Actions release:

1. Open the GitHub repository.
2. Go to Actions.
3. Select the Release workflow.
4. Click Run workflow.
5. Enter a version such as `v0.1.0`.

Tag-based release:

```powershell
git tag v0.1.0
git push origin v0.1.0
```

The workflow runs tests, builds the release executable, builds the installer, creates a GitHub Release, and uploads:

- `ait-v0.1.0-setup.exe`
- `ait-v0.1.0-windows.exe`

## Important Limitations

- Windows only.
- No UI Automation capture in MVP.
- No OCR in MVP.
- No history in MVP.
- No streaming output in MVP.
- Built-in Google no-key translation is not Google Cloud Translation and may break or be rate-limited.
- Release artifacts are not code-signed.
```

- [ ] **Step 2: Verify README mentions no zip extraction**

Run:

```powershell
rg -n "No zip extraction|setup.exe|windows.exe|Release workflow" README.md
```

Expected: Output includes all four phrases.

- [ ] **Step 3: Commit README update**

Run:

```powershell
git add README.md
git commit -m "docs: document release downloads"
```

Expected: Commit succeeds with README changes.

---

### Task 4: Local Verification

**Files:**
- Read: `installer/ait.iss`
- Read: `.github/workflows/release.yml`
- Read: `README.md`

- [ ] **Step 1: Verify git worktree only contains planned files**

Run:

```powershell
git status --short
```

Expected: No output after the previous commits.

- [ ] **Step 2: Run tests**

Run:

```powershell
cargo test
```

Expected: All tests pass.

- [ ] **Step 3: Build release executable**

Run:

```powershell
cargo build --release
```

Expected: Command succeeds and `target\release\ait.exe` exists.

- [ ] **Step 4: Check release executable exists**

Run:

```powershell
Test-Path target\release\ait.exe
```

Expected:

```text
True
```

- [ ] **Step 5: Validate version regex behavior locally**

Run:

```powershell
$valid = "v0.1.0" -match '^v\d+\.\d+\.\d+$'
$invalid = "0.1.0" -match '^v\d+\.\d+\.\d+$'
"valid=$valid invalid=$invalid"
```

Expected:

```text
valid=True invalid=False
```

- [ ] **Step 6: Verify the workflow does not create zip assets**

Run:

```powershell
rg -n "zip|Compress-Archive" .github\workflows\release.yml installer\ait.iss README.md
```

Expected: No output, except README's user-facing sentence `No zip extraction is required.` if the search pattern matches lowercase `zip`.

- [ ] **Step 7: Commit verification note if any file changed during verification**

Run:

```powershell
git status --short
```

Expected: No output. If output exists, inspect the files and commit only intentional changes.

---

### Task 5: Remote Release Smoke Test

**Files:**
- Read: `.github/workflows/release.yml`
- Read: GitHub Actions run logs in the browser

- [ ] **Step 1: Push the implementation commits**

Run:

```powershell
git push
```

Expected: Commits are pushed to the GitHub repository.

- [ ] **Step 2: Trigger a manual test release**

In GitHub:

1. Open the repository.
2. Open Actions.
3. Select `Release`.
4. Click `Run workflow`.
5. Enter `v0.1.0`.
6. Start the workflow.

Expected: The workflow starts on `windows-latest`.

- [ ] **Step 3: Confirm the workflow succeeds**

In GitHub Actions, open the run log.

Expected completed steps:

- `Test`
- `Build release executable`
- `Prepare portable executable`
- `Install Inno Setup`
- `Build installer`
- `Publish GitHub Release`

- [ ] **Step 4: Confirm Release assets**

Open the created GitHub Release.

Expected assets:

```text
ait-v0.1.0-setup.exe
ait-v0.1.0-windows.exe
```

There must be no `.zip` asset.

- [ ] **Step 5: Confirm setup installer can start**

Download `ait-v0.1.0-setup.exe` on Windows and run it.

Expected:

- Installer opens.
- It offers to install `ait`.
- It creates a Start Menu shortcut.
- Windows shows a normal uninstall entry after install.

- [ ] **Step 6: Confirm portable exe can start**

Download `ait-v0.1.0-windows.exe` on Windows and run it.

Expected:

- App starts.
- Tray icon appears.
- The app can exit from its tray menu.

---

## Self-Review Notes

- Spec coverage:
  - Manual GitHub Actions publishing is covered in Task 2 and Task 5.
  - Tag-triggered publishing is covered in Task 2.
  - `cargo test` and `cargo build --release` are covered in Task 2 and Task 4.
  - Portable exe asset is covered in Task 2 and Task 5.
  - Inno Setup installer asset is covered in Task 1, Task 2, and Task 5.
  - No zip asset is covered in Task 3 and Task 4.
  - README maintainer guidance is covered in Task 3.
- Scope:
  - No code signing, auto-update, startup registration, desktop shortcut, or multi-platform publishing is included.
- Execution constraint:
  - This plan intentionally excludes `superpowers:subagent-driven-development` because repository instructions forbid it.
