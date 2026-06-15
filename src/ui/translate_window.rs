use crate::error::Result;

#[derive(Debug, Clone)]
pub struct TranslationWindowState {
    pub source_text: String,
    pub translated_text: String,
    pub loading: bool,
    pub error: Option<String>,
}

#[cfg(windows)]
pub struct TranslationWindow {
    state: TranslationWindowState,
}

#[cfg(windows)]
impl TranslationWindow {
    pub fn new() -> Result<Self> {
        Ok(Self {
            state: TranslationWindowState {
                source_text: String::new(),
                translated_text: String::new(),
                loading: false,
                error: None,
            },
        })
    }

    pub fn show_loading(&mut self, source_text: String) -> Result<()> {
        self.state.source_text = source_text;
        self.state.translated_text.clear();
        self.state.loading = true;
        self.state.error = None;
        tracing::info!("show translation window loading state");
        Ok(())
    }

    pub fn show_result(&mut self, translated_text: String) -> Result<()> {
        self.state.translated_text = translated_text;
        self.state.loading = false;
        self.state.error = None;
        tracing::info!("show translation window result");
        Ok(())
    }

    pub fn show_error(&mut self, message: String) -> Result<()> {
        self.state.loading = false;
        self.state.error = Some(message);
        tracing::info!("show translation window error");
        Ok(())
    }
}
