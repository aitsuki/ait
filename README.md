# ait

Windows-only lightweight selection translator.

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

## Important Limitations

- No UI Automation capture in MVP.
- No OCR in MVP.
- No history in MVP.
- No streaming output in MVP.
- Built-in Google no-key translation is not Google Cloud Translation and may break or be rate-limited.
