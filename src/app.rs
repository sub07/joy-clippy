use std::{fmt::Debug, thread, time::Duration};

use clipboard_rs::{Clipboard, ClipboardContext};
use iced::{
    event::{self, Status},
    futures::{SinkExt, Stream},
    keyboard::{key, Key, Modifiers},
    stream,
    widget::{self, button, column, container, scrollable, text},
    window::{self, Level, Position, Settings},
    Color, Element, Length, Size, Subscription, Task,
};
use tokio::{sync::mpsc, time::sleep};

use crate::{
    clipboard::ClipboardListener,
    tray_icon::subscribe_tray_menu_event,
    utils::{ColorUtils, ASYNC_CHANNEL_SIZE},
};

pub struct App {
    history: Vec<String>,
    selected_history_item_index: i32,
    clipboard_context: ClipboardContext,
    clippy_window: Option<window::Id>,
}

impl Debug for App {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JoyClippyApp")
            .field("history", &self.history)
            .finish()
    }
}

pub enum ClippyWindowEvent {
    Close,
    Paste,
    LooseFocus,
    MoveHistoryCursor,
}

#[derive(Debug, Clone)]
pub enum AppMessage {
    ClipboardEvent,
    ToggleWindow,
    OpenClippyWindow,
    CloseClippyWindow,
    ExitApp,
    Paste,
    LooseFocus { id: window::Id },
    MoveHistoryItem(i32),
}

impl App {
    pub fn new() -> (Self, Task<AppMessage>) {
        (
            Self {
                history: Default::default(),
                selected_history_item_index: 0,
                clipboard_context: ClipboardContext::new().unwrap(),
                clippy_window: None,
            },
            Task::none(),
        )
    }

    pub fn selected_history_index(&self) -> usize {
        self.history.len() - self.selected_history_item_index as usize - 1
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
                if self.clippy_window.is_some() {
                    Task::done(AppMessage::CloseClippyWindow)
                } else {
                    Task::done(AppMessage::OpenClippyWindow)
                }
            }
            AppMessage::ExitApp => iced::exit(),
            AppMessage::Paste => Task::done(AppMessage::ToggleWindow)
                .chain(Task::perform(Self::paste(), |_| {}).discard()),
            AppMessage::OpenClippyWindow => {
                let (id, open_task) = window::open(Settings {
                    decorations: false,
                    level: Level::AlwaysOnTop,
                    position: Position::Centered,
                    size: Size::new(200., 450.),
                    exit_on_close_request: false,
                    ..Default::default()
                });
                self.clippy_window = Some(id);
                self.selected_history_item_index = 0;
                open_task.chain(window::gain_focus(id)).discard()
            }
            AppMessage::CloseClippyWindow => {
                if let Some(window) = self.clippy_window.take() {
                    window::close(window)
                } else {
                    Task::none()
                }
            }
            AppMessage::LooseFocus { id } => {
                if self
                    .clippy_window
                    .is_some_and(|clippy_window_id| clippy_window_id == id)
                {
                    Task::done(AppMessage::CloseClippyWindow)
                } else {
                    Task::none()
                }
            }
            AppMessage::MoveHistoryItem(direction) => {
                self.selected_history_item_index += direction;
                if self.selected_history_item_index < 0 {
                    self.selected_history_item_index = 0
                }
                if self.selected_history_item_index >= self.history.len() as i32 {
                    self.selected_history_item_index = self.history.len() as i32 - 1;
                }
                Task::none()
            }
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
        let loose_focus_event_handler = event::listen_with(|event, status, id| {
            if let Status::Captured = status {
                return None;
            }

            match event {
                iced::Event::Window(window::Event::Unfocused) => {
                    Some(AppMessage::LooseFocus { id })
                }
                iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                    key,
                    modified_key: _,
                    physical_key: _,
                    location: _,
                    modifiers,
                    text: _,
                }) => match key {
                    Key::Named(key::Named::F9) if modifiers.alt() => {
                        Some(AppMessage::CloseClippyWindow)
                    }
                    Key::Named(key::Named::ArrowDown) => Some(AppMessage::MoveHistoryItem(1)),
                    Key::Named(key::Named::ArrowUp) => Some(AppMessage::MoveHistoryItem(-1)),
                    Key::Named(key::Named::Escape) => Some(AppMessage::CloseClippyWindow),
                    Key::Named(key::Named::Enter) => Some(AppMessage::Paste),
                    _ => None,
                },
                _ => None,
            }
        });

        Subscription::batch([
            clipboard_subscription,
            global_event_subscription,
            tray_menu_event_subscription,
            loose_focus_event_handler,
        ])
    }

    fn subscribe_global_event() -> impl Stream<Item = AppMessage> {
        stream::channel(ASYNC_CHANNEL_SIZE, |mut sender| async move {
            let (tx, mut rx) = mpsc::channel(ASYNC_CHANNEL_SIZE);
            thread::spawn(move || {
                rdev::listen(move |event| {
                    tx.blocking_send(event).unwrap();
                })
            });

            let mut modifiers = Modifiers::empty();
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
                                sender.send(AppMessage::OpenClippyWindow).await.unwrap();
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
                    _ => {}
                }
            }
        })
    }

    pub fn view(&self, _: window::Id) -> Element<AppMessage> {
        fn row_bg_color(theme: &iced::Theme, row_index: usize, selected: bool) -> container::Style {
            let bg_color = if row_index & 1 == 0 {
                theme.palette().background
            } else if theme.extended_palette().is_dark {
                theme.palette().background.lighten(0.1)
            } else {
                theme.palette().background.darken(0.1)
            };

            let bg_color = if selected {
                if theme.extended_palette().is_dark {
                    bg_color.lighten(0.4)
                } else {
                    bg_color.darken(0.4)
                }
            } else {
                bg_color
            };

            widget::container::background(bg_color)
        }

        column![
            button(text!("Paste")).on_press(AppMessage::Paste),
            scrollable(
                widget::Column::from_iter(self.history.iter().rev().enumerate().map(
                    |(index, entry)| {
                        container(text!("{entry}").size(13))
                            .style(move |theme: &iced::Theme| {
                                row_bg_color(
                                    theme,
                                    index,
                                    index == self.selected_history_item_index as usize,
                                )
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
