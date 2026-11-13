pub mod app;
#[cfg(windows)]
pub mod capture;
pub mod command;
pub mod config;
pub mod dictionary;
pub mod error;
pub mod hotkey;
#[cfg(not(feature = "portable"))]
pub mod logging;
pub mod paths;
pub mod secret;
#[cfg(windows)]
mod single_instance;
pub mod startup;
pub mod translator;
pub mod ui;
pub mod update;
