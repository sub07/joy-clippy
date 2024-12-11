use app::App;
use tray::create_tray;

mod app;
mod clipboard;
mod db;
mod tray;
mod utils;
mod window;

const JOY_CLIPPY_ICON: &[u8] = include_bytes!("../icon.ico");

#[cfg(debug_assertions)]
const APPLICATION: &str = "Clippy Dev";

#[cfg(not(debug_assertions))]
const APPLICATION: &str = "Clippy";

const ORGANIZATION: &str = "Joy";
const QUALIFIER: &str = "me.mpardo";

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::ERROR)
        .with_test_writer()
        .init();

    let _tray = create_tray();

    iced::daemon("Joy clippy", App::update, App::view)
        .subscription(App::subscription)
        .run_with(App::new)
        .unwrap();
}
