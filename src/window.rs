use std::{fmt::Debug, thread, time::Duration};

use clipboard_rs::{Clipboard, ClipboardContext};
use iced::{
    futures::{SinkExt, Stream},
    keyboard::Modifiers,
    stream,
    widget::{self, button, column, container, scrollable, text},
    window::{self, Level, Position, Settings},
    Color, Element, Length, Size, Subscription, Task,
};
use tokio::{sync::mpsc, time::sleep};

use crate::{
    clipboard::ClipboardListener, tray_icon::subscribe_tray_menu_event, utils::ColorUtils,
};

pub struct App {
    history: Vec<String>,
    clipboard_context: ClipboardContext,
    window: Option<window::Id>,
}

impl Debug for App {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JoyClippyApp")
            .field("history", &self.history)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub enum AppMessage {
    ClipboardEvent,
    ToggleWindow,
    GlobalMouseLeftClick((f64, f64)),
    CleanWindow,
    Quit,
    Paste,
}

impl App {
    pub fn new() -> (Self, Task<AppMessage>) {
        (
            Self {
                history: Default::default(),
                clipboard_context: ClipboardContext::new().unwrap(),
                window: None,
            },
            Task::none(),
        )
    }

    pub fn update(&mut self, message: AppMessage) -> Task<AppMessage> {
        match message {
            AppMessage::ClipboardEvent => {
                if let Ok(text) = self.clipboard_context.get_text() {
                    self.history.push(text);
                }
                Task::none()
            }
            AppMessage::ToggleWindow => {
                if let Some(window) = self.window.take() {
                    window::close(window)
                } else {
                    let (id, open_task) = window::open(Settings {
                        decorations: false,
                        level: Level::AlwaysOnTop,
                        position: Position::Centered,
                        size: Size::new(200., 450.),
                        exit_on_close_request: false,
                        ..Default::default()
                    });
                    self.window = Some(id);
                    open_task.discard()
                }
            }
            AppMessage::GlobalMouseLeftClick((x, y)) => {
                let (x, y) = (x as f32, y as f32);
                if let Some(window) = self.window {
                    window::get_position(window)
                        .and_then(move |position| {
                            window::get_size(window).map(move |size| (position, size))
                        })
                        .then(move |(position, size)| {
                            if !(x > position.x
                                && x < position.x + size.width
                                && y > position.y
                                && y < position.x + size.height)
                            {
                                window::close(window).chain(Task::done(AppMessage::CleanWindow))
                            } else {
                                // self.window = Some(window);
                                Task::none()
                            }
                        })
                } else {
                    Task::none()
                }
            }
            AppMessage::CleanWindow => {
                self.window = None;
                Task::none()
            }
            AppMessage::Quit => iced::exit(),
            AppMessage::Paste => Task::done(AppMessage::ToggleWindow)
                .chain(Task::perform(Self::paste(), |_| {}).discard()),
        }
    }

    pub async fn paste() {
        println!("paste");

        async fn simulate(event: rdev::EventType) {
            sleep(Duration::from_millis(20)).await;
            rdev::simulate(&event).unwrap();
            sleep(Duration::from_millis(20)).await;
        }

        simulate(rdev::EventType::KeyPress(rdev::Key::ControlLeft)).await;
        simulate(rdev::EventType::KeyPress(rdev::Key::KeyV)).await;
        simulate(rdev::EventType::KeyRelease(rdev::Key::KeyV)).await;
        simulate(rdev::EventType::KeyRelease(rdev::Key::ControlLeft)).await;
    }

    pub fn subscription(&self) -> Subscription<AppMessage> {
        let clipboard_subscription = Subscription::run(ClipboardListener::subscribe);
        let global_event_subscription = Subscription::run(Self::subscribe_global_event);
        let tray_menu_event_subscription = Subscription::run(subscribe_tray_menu_event);

        Subscription::batch([
            clipboard_subscription,
            global_event_subscription,
            tray_menu_event_subscription,
        ])
    }

    fn subscribe_global_event() -> impl Stream<Item = AppMessage> {
        stream::channel(100, |mut sender| async move {
            let (tx, mut rx) = mpsc::channel(100);
            thread::spawn(move || {
                rdev::listen(move |event| {
                    tx.blocking_send(event).unwrap();
                })
            });

            let mut modifiers = Modifiers::empty();
            let mut mouse_position = (0., 0.);

            loop {
                let event = rx.recv().await.unwrap();
                match event.event_type {
                    rdev::EventType::KeyPress(key) => match key {
                        rdev::Key::ControlLeft | rdev::Key::ControlRight => {
                            modifiers.insert(Modifiers::CTRL);
                        }
                        rdev::Key::Alt => {
                            modifiers.insert(Modifiers::ALT);
                        }
                        rdev::Key::ShiftLeft | rdev::Key::ShiftRight => {
                            modifiers.insert(Modifiers::SHIFT);
                        }

                        rdev::Key::MetaLeft | rdev::Key::MetaRight => {
                            modifiers.insert(Modifiers::LOGO);
                        }
                        rdev::Key::F9 => {
                            if modifiers.alt() {
                                sender.send(AppMessage::ToggleWindow).await.unwrap();
                            }
                        }
                        _ => {}
                    },
                    rdev::EventType::KeyRelease(key) => match key {
                        rdev::Key::ControlLeft | rdev::Key::ControlRight => {
                            modifiers.remove(Modifiers::CTRL);
                        }
                        rdev::Key::Alt => {
                            modifiers.remove(Modifiers::ALT);
                        }
                        rdev::Key::ShiftLeft | rdev::Key::ShiftRight => {
                            modifiers.remove(Modifiers::SHIFT);
                        }
                        rdev::Key::MetaLeft | rdev::Key::MetaRight => {
                            modifiers.remove(Modifiers::LOGO);
                        }
                        _ => {}
                    },
                    rdev::EventType::ButtonPress(_) => {
                        sender
                            .send(AppMessage::GlobalMouseLeftClick(mouse_position))
                            .await
                            .unwrap();
                    }
                    rdev::EventType::ButtonRelease(_) => {}
                    rdev::EventType::MouseMove { x, y } => {
                        mouse_position = (x, y);
                    }
                    rdev::EventType::Wheel {
                        delta_x: _,
                        delta_y: _,
                    } => {}
                }
            }
        })
    }

    pub fn view(&self, _: window::Id) -> Element<AppMessage> {
        column![
            button(text!("Paste")).on_press(AppMessage::Paste),
            scrollable(
                widget::Column::from_iter(self.history.iter().rev().enumerate().map(
                    |(index, entry)| {
                        container(text!("{entry}").size(13))
                            .style(move |theme: &iced::Theme| {
                                widget::container::background(if index & 1 == 0 {
                                    Color::TRANSPARENT
                                } else if theme.extended_palette().is_dark {
                                    theme.palette().background.lighten(0.2)
                                } else {
                                    theme.palette().background.darken(0.2)
                                })
                            })
                            .padding(8)
                            .width(Length::Fill)
                            .into()
                    },
                ))
                .spacing(4),
            )
        ]
        .into()
    }
}
