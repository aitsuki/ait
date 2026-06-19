#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppCommand {
    TranslateSelection,
    TranslateClipboard,
    OpenSettings,
    OpenLatestRelease,
    RetryTranslation,
    CopyTranslation,
    Exit,
}
