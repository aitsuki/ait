use ait::capture::{CaptureErrorKind, CaptureService, ClipboardBackend};
use std::cell::RefCell;
use std::time::Duration;

#[derive(Default)]
struct FakeClipboard {
    current: RefCell<Option<String>>,
    copied: RefCell<Option<String>>,
}

impl ClipboardBackend for FakeClipboard {
    fn read_text(&self) -> ait::error::Result<Option<String>> {
        Ok(self.current.borrow().clone())
    }

    fn write_text(&self, text: &str) -> ait::error::Result<()> {
        *self.current.borrow_mut() = Some(text.to_string());
        Ok(())
    }

    fn send_copy(&self) -> ait::error::Result<()> {
        if let Some(text) = self.copied.borrow().clone() {
            *self.current.borrow_mut() = Some(text);
        }
        Ok(())
    }
}

#[test]
fn capture_restores_previous_text_clipboard() {
    let fake = FakeClipboard::default();
    *fake.current.borrow_mut() = Some("old clipboard".to_string());
    *fake.copied.borrow_mut() = Some("selected text".to_string());
    let service = CaptureService::new(fake, Duration::from_millis(1));

    let captured = service.capture_selected_text().unwrap();

    assert_eq!(captured.text, "selected text");
    assert_eq!(
        service.backend().read_text().unwrap(),
        Some("old clipboard".to_string())
    );
}

#[test]
fn capture_returns_empty_when_copy_produces_no_text() {
    let fake = FakeClipboard::default();
    *fake.current.borrow_mut() = Some("old clipboard".to_string());
    let service = CaptureService::new(fake, Duration::from_millis(1));

    let err = service.capture_selected_text().unwrap_err();

    assert_eq!(err.kind, CaptureErrorKind::NoText);
    assert_eq!(
        service.backend().read_text().unwrap(),
        Some("old clipboard".to_string())
    );
}
