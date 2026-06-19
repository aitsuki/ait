#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() -> ait::error::Result<()> {
    ait::app::run()
}
