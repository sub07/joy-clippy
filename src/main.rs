use app::App;
use tray::create_tray;

mod app;
mod clipboard;
mod tray;
mod utils;
mod window;

const JOY_CLIPPY_ICON: &[u8] = include_bytes!("../icon.ico");

fn main() {
    let _tray = create_tray();

    iced::daemon("Joy clippy", App::update, App::view)
        .subscription(App::subscription)
        .run_with(App::new)
        .unwrap();
}
