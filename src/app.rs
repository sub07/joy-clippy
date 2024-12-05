use std::{collections::HashMap, fmt::Debug, thread, time::Duration};

use clipboard_rs::{Clipboard, ClipboardContext};
use iced::{
    event::{self, Status},
    futures::{SinkExt, Stream},
    keyboard::{key, Key, Modifiers},
    stream,
    widget::horizontal_space,
    window::{close_events, Level, Position, Settings},
    Element, Size, Subscription, Task,
};
use joy_impl_ignore::debug::DebugImplIgnore;
use tokio::{sync::mpsc, time::sleep};

use crate::{
    clipboard::ClipboardListener,
    tray::subscribe_tray_menu_event,
    utils::ASYNC_CHANNEL_SIZE,
    window::{self, Window},
};

pub struct App {
    history: Vec<String>,
    clipboard_context: DebugImplIgnore<ClipboardContext>,
    windows: HashMap<iced::window::Id, Window>,
}

#[derive(Debug, Clone)]
pub enum Message {
    // Window general
    OpenHistoryWindow,
    OpenSettingsWindow,
    RequestHistoryWindowClose,
    RequestWindowClose(iced::window::Id),
    WindowClose(iced::window::Id),
    LooseFocus(iced::window::Id),
    ExitApp,

    // Clipboard
    ClipboardEvent,
    RequestPaste(usize),
    SetClipboardItem(usize),
    SimulatePaste,

    // Window specific
    HistoryWindow(iced::window::Id, window::history::Message),
    ConfigWindow(iced::window::Id, window::settings::Message),
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                history: Default::default(),
                clipboard_context: ClipboardContext::new()
                    .expect("Retrieval of system clipboard")
                    .into(),
                windows: Default::default(),
            },
            Task::none(),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        println!("{:?}", message);
        match message {
            Message::OpenHistoryWindow => {
                let (id, open_task) = iced::window::open(Settings {
                    decorations: false,
                    level: Level::AlwaysOnTop,
                    position: Position::Centered,
                    size: Size::new(200., 450.),
                    exit_on_close_request: false,
                    ..Default::default()
                });
                self.windows.insert(
                    id,
                    Window::History(window::history::State::new(self.history.clone())),
                );
                open_task.chain(iced::window::gain_focus(id)).discard()
            }
            Message::OpenSettingsWindow => {
                let (id, open_task) = iced::window::open(Settings {
                    size: Size::new(500., 300.),
                    resizable: true,
                    ..Default::default()
                });
                let close_task = Task::batch(
                    self.windows
                        .keys()
                        .map(|id| Task::done(Message::RequestWindowClose(*id))),
                );

                self.windows
                    .insert(id, Window::Settings(window::settings::State::new()));

                close_task
                    .chain(open_task.discard())
                    .chain(iced::window::gain_focus(id))
                    .discard()
            }
            Message::RequestWindowClose(id) => iced::window::close(id),
            Message::WindowClose(id) => {
                self.windows.remove(&id);
                Task::none()
            }
            Message::LooseFocus(id) => {
                if let Some(Window::History(_)) = self.windows.get(&id) {
                    Task::done(Message::RequestWindowClose(id))
                } else {
                    Task::none()
                }
            }
            Message::ExitApp => iced::exit(),
            Message::ClipboardEvent => {
                if let Ok(text) = self.clipboard_context.get_text() {
                    self.history.push(text);
                }
                Task::none()
            }
            Message::RequestPaste(item_index) => {
                let close_task = if let Some(id) = self.get_history_window_id() {
                    Task::done(Message::RequestWindowClose(id))
                } else {
                    Task::none()
                };

                close_task
                    .chain(Task::done(Message::SetClipboardItem(item_index)))
                    .chain(Task::done(Message::SimulatePaste))
            }
            Message::SetClipboardItem(index) => {
                self.clipboard_context
                    .set_text(self.history.remove(index))
                    .expect("Setting system clipboard value");
                Task::none()
            }
            Message::SimulatePaste => Task::perform(
                async {
                    async fn simulate(event: rdev::EventType) {
                        sleep(Duration::from_millis(20)).await;
                        rdev::simulate(&event).unwrap();
                        sleep(Duration::from_millis(20)).await;
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
            Message::HistoryWindow(window_id, message) => {
                if let Some(Window::History(state)) = self.windows.get_mut(&window_id) {
                    state.update(message)
                } else {
                    Task::none()
                }
            }
            Message::ConfigWindow(window_id, message) => {
                if let Some(Window::Settings(state)) = self.windows.get_mut(&window_id) {
                    state.update(message)
                } else {
                    Task::none()
                }
            }
            Message::RequestHistoryWindowClose => {
                if let Some(id) = self.get_history_window_id() {
                    Task::done(Message::RequestWindowClose(id))
                } else {
                    Task::none()
                }
            }
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let clipboard_event_subscription = Subscription::run(ClipboardListener::subscribe);
        let global_event_subscription = Subscription::run(Self::subscribe_global_event);
        let tray_menu_event_subscription = Subscription::run(subscribe_tray_menu_event);
        let loose_focus_event_subscription = event::listen_with(|event, status, id| {
            if let Status::Captured = status {
                return None;
            }

            match event {
                iced::Event::Window(iced::window::Event::Unfocused) => {
                    Some(Message::LooseFocus(id))
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
                        Some(Message::RequestHistoryWindowClose)
                    }
                    Key::Named(key::Named::ArrowDown) => Some(Message::HistoryWindow(
                        id,
                        window::history::Message::MoveHistoryCursor(1),
                    )),
                    Key::Named(key::Named::ArrowUp) => Some(Message::HistoryWindow(
                        id,
                        window::history::Message::MoveHistoryCursor(-1),
                    )),
                    Key::Named(key::Named::Escape) => Some(Message::RequestHistoryWindowClose),
                    Key::Named(key::Named::Enter) => {
                        Some(Message::HistoryWindow(id, window::history::Message::Paste))
                    }
                    _ => None,
                },
                _ => None,
            }
        });

        let window_close_event_subscription = close_events().map(Message::WindowClose);

        Subscription::batch([
            clipboard_event_subscription,
            global_event_subscription,
            tray_menu_event_subscription,
            loose_focus_event_subscription,
            window_close_event_subscription,
        ])
    }

    fn get_history_window_id(&self) -> Option<iced::window::Id> {
        self.windows
            .iter()
            .find(|(_, window)| matches!(window, Window::History(_)))
            .map(|(id, _)| *id)
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
                                sender.send(Message::OpenHistoryWindow).await.unwrap();
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
        match self.windows.get(&id) {
            Some(Window::History(state)) => state
                .view()
                .map(move |message| Message::HistoryWindow(id, message)),
            Some(Window::Settings(state)) => state
                .view()
                .map(move |message| Message::ConfigWindow(id, message)),
            None => horizontal_space().into(),
        }
    }
}
