use tray_icon::create_tray;
use window::App;

mod clipboard;
mod tray_icon;
mod utils;
mod window;

fn main() {
    let _tray = create_tray();

    iced::daemon("Joy Clippy", App::update, App::view)
        .subscription(App::subscription)
        .run_with(App::new)
        .unwrap();
}
