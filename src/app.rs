use std::{fmt::Debug, thread, time::Duration};

use clipboard_rs::{Clipboard, ClipboardContext};
use iced::{
    event::{self, Status},
    futures::{SinkExt, Stream},
    keyboard::{key, Key, Modifiers},
    stream,
    widget::horizontal_space,
    window::{Level, Position, Settings},
    Element, Size, Subscription, Task,
};
use joy_impl_ignore::debug::DebugImplIgnore;
use tokio::{sync::mpsc, time::sleep};

use crate::{
    clipboard::ClipboardListener,
    tray_icon::subscribe_tray_menu_event,
    utils::ASYNC_CHANNEL_SIZE,
    window::{self, Window},
};

pub struct App {
    history: Vec<String>,
    clipboard_context: DebugImplIgnore<ClipboardContext>,
    current_window: Option<(iced::window::Id, Window)>,
}

#[derive(Debug, Clone)]
pub enum Message {
    ClipboardEvent,
    ToggleWindow,
    OpenMainWindow,
    CloseMainWindow,
    ExitApp,
    Paste(usize),
    SimulatePasting,
    SetClipboardItem(usize),
    LooseFocus { id: iced::window::Id },
    MainWindow(window::main::Message),
    ConfigWindow(window::config::Message),
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                history: Default::default(),
                clipboard_context: ClipboardContext::new().unwrap().into(),
                current_window: None,
            },
            Task::none(),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        println!("{:?}", message);
        match message {
            Message::ClipboardEvent => {
                if let Ok(text) = self.clipboard_context.get_text() {
                    self.history.push(text);
                }
                Task::none()
            }
            Message::ToggleWindow => {
                if self.current_window.is_some() {
                    Task::done(Message::CloseMainWindow)
                } else {
                    Task::done(Message::OpenMainWindow)
                }
            }
            Message::ExitApp => iced::exit(),
            Message::Paste(index) => Task::done(Message::ToggleWindow)
                .chain(Task::done(Message::SetClipboardItem(index)))
                .chain(Task::done(Message::SimulatePasting)),
            Message::OpenMainWindow => {
                let (id, open_task) = iced::window::open(Settings {
                    decorations: false,
                    level: Level::AlwaysOnTop,
                    position: Position::Centered,
                    size: Size::new(200., 450.),
                    exit_on_close_request: false,
                    ..Default::default()
                });
                self.current_window = Some((
                    id,
                    window::Window::Main(window::main::State::new(self.history.clone())),
                ));
                open_task.chain(iced::window::gain_focus(id)).discard()
            }
            Message::CloseMainWindow => {
                if let Some((id, _)) = self.current_window.take() {
                    iced::window::close(id)
                } else {
                    Task::none()
                }
            }
            Message::LooseFocus { id } => {
                if self
                    .current_window
                    .as_ref()
                    .is_some_and(|(current_id, window)| {
                        *current_id == id && matches!(window, window::Window::Main(_))
                    })
                {
                    Task::done(Message::CloseMainWindow)
                } else {
                    Task::none()
                }
            }
            Message::MainWindow(message) => {
                if let Some((_, Window::Main(state))) = self.current_window.as_mut() {
                    state.update(message)
                } else {
                    Task::none()
                }
            }
            Message::ConfigWindow(message) => {
                if let Some((_, Window::Config(state))) = self.current_window.as_mut() {
                    state.update(message)
                } else {
                    Task::none()
                }
            }
            Message::SetClipboardItem(index) => {
                self.clipboard_context
                    .set_text(self.history.remove(index))
                    .unwrap();
                Task::none()
            }
            Message::SimulatePasting => Task::perform(
                async {
                    async fn simulate(event: rdev::EventType) {
                        sleep(Duration::from_millis(20)).await;
                        rdev::simulate(&event).unwrap();
                        sleep(Duration::from_millis(20)).await;
                    }

                    simulate(rdev::EventType::KeyPress(rdev::Key::ControlLeft)).await;
                    simulate(rdev::EventType::KeyPress(rdev::Key::KeyV)).await;
                    simulate(rdev::EventType::KeyRelease(rdev::Key::KeyV)).await;
                    simulate(rdev::EventType::KeyRelease(rdev::Key::ControlLeft)).await;
                },
                |_| {},
            )
            .discard(),
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let clipboard_subscription = Subscription::run(ClipboardListener::subscribe);
        let global_event_subscription = Subscription::run(Self::subscribe_global_event);
        let tray_menu_event_subscription = Subscription::run(subscribe_tray_menu_event);
        let loose_focus_event_handler = event::listen_with(|event, status, id| {
            if let Status::Captured = status {
                return None;
            }

            match event {
                iced::Event::Window(iced::window::Event::Unfocused) => {
                    Some(Message::LooseFocus { id })
                }
                iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                    key,
                    modified_key: _,
                    physical_key: _,
                    location: _,
                    modifiers,
                    text: _,
                }) => match key {
                    Key::Named(key::Named::F9) if modifiers.alt() => Some(Message::CloseMainWindow),
                    Key::Named(key::Named::ArrowDown) => Some(Message::MainWindow(
                        window::main::Message::MoveHistoryCursor(1),
                    )),
                    Key::Named(key::Named::ArrowUp) => Some(Message::MainWindow(
                        window::main::Message::MoveHistoryCursor(-1),
                    )),
                    Key::Named(key::Named::Escape) => Some(Message::CloseMainWindow), // TODO filtrer
                    Key::Named(key::Named::Enter) => {
                        Some(Message::MainWindow(window::main::Message::Paste))
                    }
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

    fn subscribe_global_event() -> impl Stream<Item = Message> {
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
                                sender.send(Message::OpenMainWindow).await.unwrap();
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

    pub fn view(&self, id: iced::window::Id) -> Element<Message> {
        match self
            .current_window
            .as_ref()
            .filter(|(current_window_id, _)| *current_window_id == id)
        {
            Some((_, Window::Main(state))) => state.view().map(Message::MainWindow),
            Some((_, Window::Config(state))) => state.view().map(Message::ConfigWindow),
            _ => horizontal_space().into(),
        }
    }
}
