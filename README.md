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
