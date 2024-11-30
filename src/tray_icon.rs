use std::{str::FromStr, thread};

use iced::{
    advanced::graphics::image::image_rs::load_from_memory,
    futures::{SinkExt, Stream},
    stream,
};
use joy_macro::DisplayFromDebug;
use tokio::sync::mpsc;
use tray_icon::{
    menu::{Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem},
    Icon, TrayIcon, TrayIconBuilder,
};

use crate::{app::AppMessage, utils::ASYNC_CHANNEL_SIZE};

const TRAY_ICON: &[u8] = include_bytes!("../icon.ico");

#[derive(Debug, DisplayFromDebug)]
enum MenuEntry {
    Open,
    Quit,
}

impl FromStr for MenuEntry {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Open" => Ok(MenuEntry::Open),
            "Quit" => Ok(MenuEntry::Quit),
            _ => Err(()),
        }
    }
}

pub fn create_tray() -> TrayIcon {
    let icon_data = load_from_memory(TRAY_ICON).unwrap();
    let (width, height) = (icon_data.width(), icon_data.height());
    let icon = Icon::from_rgba(icon_data.into_bytes(), width, height).unwrap();

    TrayIconBuilder::new()
        .with_tooltip("Joy Clippy")
        .with_menu(Box::new(
            Menu::with_items(&[
                &MenuItem::with_id(MenuEntry::Open.to_string(), "Open", true, None),
                &PredefinedMenuItem::separator(),
                &MenuItem::with_id(MenuEntry::Quit.to_string(), "Quit", true, None),
            ])
            .unwrap(),
        ))
        .with_icon(icon)
        .build()
        .unwrap()
}

pub fn subscribe_tray_menu_event() -> impl Stream<Item = AppMessage> {
    stream::channel(ASYNC_CHANNEL_SIZE, |mut output| async move {
        let (tx, mut rx) = mpsc::channel(ASYNC_CHANNEL_SIZE);

        thread::spawn(move || loop {
            if let Ok(menu_event) = MenuEvent::receiver().recv() {
                tx.blocking_send(menu_event).unwrap()
            }
        });

        loop {
            if let Some(menu_entry) = rx
                .recv()
                .await
                .and_then(|MenuEvent { id: MenuId(id) }| MenuEntry::from_str(id.as_str()).ok())
            {
                let message = match menu_entry {
                    MenuEntry::Open => AppMessage::ToggleWindow,
                    MenuEntry::Quit => AppMessage::ExitApp,
                };
                output.send(message).await.unwrap();
            }
        }
    })
}
