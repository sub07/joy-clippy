use std::{collections::HashMap, fmt::Debug, thread, time::Duration};

use clipboard_rs::{Clipboard, ClipboardContext};
use iced::{
    advanced::graphics::image::image_rs::load_from_memory,
    event::{self, Status},
    futures::{SinkExt, Stream},
    keyboard::{
        key::{self, Code, Physical},
        Key, Modifiers,
    },
    stream,
    widget::horizontal_space,
    window::{close_events, Level, Position, Settings},
    Element, Size, Subscription, Task,
};
use joy_impl_ignore::debug::DebugImplIgnore;
use sea_orm::DatabaseConnection;
use tokio::{sync::mpsc, time::sleep};

use crate::{
    clipboard::ClipboardListener,
    db::{get_db, repo},
    tray::subscribe_tray_menu_event,
    utils::{self, iced_event_to_shortcut, ASYNC_CHANNEL_SIZE},
    window::{self, Window},
    JOY_CLIPPY_ICON,
};

const DEFAULT_TOGGLE_MODIFIERS: iced::keyboard::Modifiers = Modifiers::ALT;
const DEFAULT_TOGGLE_PHYSICAL_KEY: iced::keyboard::key::Physical = Physical::Code(Code::F9);
const DEFAULT_TOGGLE_LOGICAL_KEY: iced::keyboard::Key = Key::Named(key::Named::F9);

#[derive(Debug, Clone)]
pub struct Shortcut {
    pub modifiers: Modifiers,
    pub logical_key: iced::keyboard::Key,
    pub iced_physical_key: iced::keyboard::key::Physical,
    pub rdev_key: rdev::Key,
}

pub struct App {
    clipboard_context: DebugImplIgnore<ClipboardContext>,
    windows: HashMap<iced::window::Id, Window>,
    db: DatabaseConnection,
    toggle_shortcut: Shortcut,
}

#[derive(Debug, Clone)]
pub enum Message {
    GlobalEvent(Modifiers, rdev::Event),
    AppEvent(iced::window::Id, iced::Event),

    // Window general
    RequestWindowClose(iced::window::Id),
    WindowClose(iced::window::Id),
    LooseFocus(iced::window::Id),
    Panic(String),
    ExitApp,

    // Clipboard
    ClipboardEvent,
    RequestPaste(entity::entry::Model),
    SetClipboardItem(entity::entry::Model),
    SimulatePaste,

    // History window
    RequestOpenHistoryWindow,
    RequestCloseHistoryWindow,
    HistoryWindowLoaded(iced::window::Id, Vec<entity::entry::Model>),
    HistoryWindowEvent(iced::window::Id, window::history::Message),

    // Settings window
    OpenSettingsWindow,
    SettingsWindowEvent(iced::window::Id, window::settings::Message),

    // Async events
    DbConnection(DatabaseConnection),

    // Business
    UpdateToggleShortcut(Shortcut),
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                clipboard_context: ClipboardContext::new()
                    .expect("Retrieval of system clipboard")
                    .into(),
                windows: Default::default(),
                db: DatabaseConnection::Disconnected,
                toggle_shortcut: Shortcut {
                    modifiers: DEFAULT_TOGGLE_MODIFIERS,
                    logical_key: DEFAULT_TOGGLE_LOGICAL_KEY,
                    iced_physical_key: DEFAULT_TOGGLE_PHYSICAL_KEY,
                    rdev_key: utils::iced_key_to_rdev(DEFAULT_TOGGLE_PHYSICAL_KEY),
                },
            },
            Task::perform(get_db(), |res| match res {
                Ok(db) => Message::DbConnection(db),
                Err(e) => Message::Panic(format!("{e:?}")),
            })
            .chain(Task::done(Message::RequestOpenHistoryWindow))
            .chain(Task::done(Message::RequestCloseHistoryWindow)),
        )
    }

    fn get_icon() -> iced::window::Icon {
        let icon_data = load_from_memory(JOY_CLIPPY_ICON).expect("Icon loading");
        let (width, height) = (icon_data.width(), icon_data.height());
        iced::window::icon::from_rgba(icon_data.into_bytes(), width, height)
            .expect("Icon from rgba")
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::HistoryWindowLoaded(id, items) => {
                if matches!(
                    self.windows.get(&id),
                    Some(Window::History(window::history::State::Loading))
                ) {
                    if let Some(Window::History(state)) = self.windows.get_mut(&id) {
                        *state = window::history::State::Loaded {
                            selected_item_cursor: 0,
                            items,
                        }
                    }
                }

                Task::none()
            }
            Message::OpenSettingsWindow => {
                let (id, open_task) = iced::window::open(Settings {
                    size: Size::new(500., 300.),
                    resizable: true,
                    icon: Some(Self::get_icon()),
                    ..Default::default()
                });

                self.windows.insert(
                    id,
                    Window::Settings(window::settings::State::new(self.toggle_shortcut.clone())),
                );

                open_task.chain(iced::window::gain_focus(id)).discard()
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
                let db = self.db.clone();
                if let Ok(text) = self.clipboard_context.get_text() {
                    Task::future(async move { crate::db::repo::add_item(&db, text).await })
                        .discard()
                } else {
                    Task::none()
                }
            }
            Message::RequestPaste(item) => Task::done(Message::RequestCloseHistoryWindow)
                .chain(Task::done(Message::SetClipboardItem(item)))
                .chain(Task::done(Message::SimulatePaste)),
            Message::SetClipboardItem(item) => {
                self.clipboard_context
                    .set_text(item.data.clone())
                    .expect("Setting system clipboard value");
                let db = self.db.clone();
                Task::future(async move { repo::delete(&db, &item).await }).discard()
            }
            Message::SimulatePaste => Task::future(async {
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
            })
            .discard(),
            Message::HistoryWindowEvent(window_id, message) => {
                if let Some(Window::History(state)) = self.windows.get_mut(&window_id) {
                    state.update(message)
                } else {
                    Task::none()
                }
            }
            Message::SettingsWindowEvent(window_id, message) => {
                if let Some(Window::Settings(state)) = self.windows.get_mut(&window_id) {
                    state.update(message)
                } else {
                    Task::none()
                }
            }
            Message::RequestCloseHistoryWindow => {
                if let Some(id) = self.get_history_window_id() {
                    Task::done(Message::RequestWindowClose(id))
                } else {
                    Task::none()
                }
            }
            Message::DbConnection(db) => {
                self.db = db;
                Task::none()
            }
            Message::RequestOpenHistoryWindow => {
                let (id, open_task) = iced::window::open(Settings {
                    decorations: false,
                    level: Level::AlwaysOnTop,
                    position: Position::Centered,
                    size: Size::new(200., 450.),
                    exit_on_close_request: false,
                    icon: Some(Self::get_icon()),
                    ..Default::default()
                });
                self.windows
                    .insert(id, Window::History(window::history::State::Loading));

                let db = self.db.clone();
                open_task
                    .chain(iced::window::gain_focus(id))
                    .discard()
                    .chain(Task::perform(
                        async move { crate::db::repo::get_items(&db).await },
                        move |items| {
                            Message::HistoryWindowLoaded(
                                id,
                                items.expect("Retreiving history item"),
                            )
                        },
                    ))
            }
            Message::Panic(message) => {
                tracing::error!("A fatal error occured\n{message}");
                Task::done(Message::ExitApp)
            }
            Message::GlobalEvent(modifiers, event) => {
                let Shortcut {
                    modifiers: toggle_modifiers,
                    rdev_key,
                    ..
                } = &self.toggle_shortcut;
                if matches!(event.event_type, rdev::EventType::KeyPress(key) if &key == rdev_key && toggle_modifiers == &modifiers)
                {
                    Task::done(Message::RequestOpenHistoryWindow)
                } else {
                    Task::none()
                }
            }
            Message::AppEvent(id, event) => match self.windows.get(&id) {
                Some(window) => match window {
                    Window::History(_) => {
                        let Shortcut {
                            modifiers: toggle_modifiers,
                            iced_physical_key,
                            ..
                        } = &self.toggle_shortcut;
                        if matches!(&event, iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                            key: _,
                            modified_key: _,
                            physical_key,
                            location: _,
                            modifiers,
                            text: _,
                        }) if (physical_key == iced_physical_key && modifiers == toggle_modifiers))
                        {
                            Task::done(Message::RequestCloseHistoryWindow)
                        } else {
                            match event {
                                iced::Event::Window(iced::window::Event::Unfocused) => {
                                    Task::done(Message::LooseFocus(id))
                                }
                                iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                                    key: _,
                                    modified_key: _,
                                    physical_key,
                                    location: _,
                                    modifiers: _,
                                    text: _,
                                }) => match physical_key {
                                    key::Physical::Code(Code::ArrowDown) => {
                                        Task::done(Message::HistoryWindowEvent(
                                            id,
                                            window::history::Message::MoveHistoryCursor(1),
                                        ))
                                    }
                                    key::Physical::Code(Code::ArrowUp) => {
                                        Task::done(Message::HistoryWindowEvent(
                                            id,
                                            window::history::Message::MoveHistoryCursor(-1),
                                        ))
                                    }
                                    key::Physical::Code(Code::Escape) => {
                                        Task::done(Message::RequestCloseHistoryWindow)
                                    }
                                    key::Physical::Code(Code::Enter) => {
                                        Task::done(Message::HistoryWindowEvent(
                                            id,
                                            window::history::Message::Paste,
                                        ))
                                    }
                                    _ => Task::none(),
                                },
                                _ => Task::none(),
                            }
                        }
                    }
                    Window::Settings(_) => {
                        if let Some(shortcut) = iced_event_to_shortcut(event) {
                            Task::done(Message::SettingsWindowEvent(
                                id,
                                window::settings::Message::NewShortcutInput(shortcut),
                            ))
                        } else {
                            Task::none()
                        }
                    }
                },
                None => Task::none(),
            },
            Message::UpdateToggleShortcut(shortcut) => {
                self.toggle_shortcut = shortcut;
                Task::none()
            }
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let clipboard_event_subscription = Subscription::run(ClipboardListener::subscribe);
        let global_event_subscription = Subscription::run(Self::subscribe_global_event);
        let tray_menu_event_subscription = Subscription::run(subscribe_tray_menu_event);
        let iced_event_subscription = event::listen_with(|event, status, id| {
            if let Status::Captured = status {
                return None;
            }

            Some(Message::AppEvent(id, event))
        });

        let window_close_event_subscription = close_events().map(Message::WindowClose);

        Subscription::batch([
            clipboard_event_subscription,
            global_event_subscription,
            tray_menu_event_subscription,
            iced_event_subscription,
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
                sender
                    .send(Message::GlobalEvent(modifiers, event))
                    .await
                    .unwrap()
            }
        })
    }

    pub fn view(&self, id: iced::window::Id) -> Element<Message> {
        match self.windows.get(&id) {
            Some(Window::History(state)) => state
                .view()
                .map(move |message| Message::HistoryWindowEvent(id, message)),
            Some(Window::Settings(state)) => state
                .view()
                .map(move |message| Message::SettingsWindowEvent(id, message)),
            None => horizontal_space().into(),
        }
    }
}
