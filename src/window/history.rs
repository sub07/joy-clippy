use iced::{
    widget::{button, center, column, container, horizontal_space, row, scrollable, text, Column},
    Alignment, Element, Length, Task,
};

use crate::{
    app::{self},
    utils::ColorUtils,
};

#[derive(Debug)]
pub enum State {
    Loaded {
        selected_item_cursor: i32,
        items: Vec<entity::entry::Model>,
    },
    Loading,
}

#[derive(Debug, Clone)]
pub enum Message {
    MoveHistoryCursor(i32),
    Paste,
    OpenSettings,
}

impl State {
    pub fn update(&mut self, event: Message) -> Task<app::Message> {
        match event {
            Message::MoveHistoryCursor(direction) => {
                if let Self::Loaded {
                    selected_item_cursor,
                    items,
                } = self
                {
                    *selected_item_cursor += direction;
                    if *selected_item_cursor < 0 {
                        *selected_item_cursor = 0
                    }
                    if *selected_item_cursor >= items.len() as i32 {
                        *selected_item_cursor = items.len() as i32 - 1;
                    }
                }
                Task::none()
            }
            Message::Paste => {
                if let Self::Loaded {
                    selected_item_cursor,
                    items,
                } = self
                {
                    Task::done(app::Message::RequestPaste(
                        items[*selected_item_cursor as usize].clone(),
                    ))
                } else {
                    Task::none()
                }
            }
            Message::OpenSettings => Task::done(app::Message::OpenSettingsWindow),
        }
    }

    pub fn view(&self) -> Element<Message> {
        fn row_bg_color(theme: &iced::Theme, row_index: usize, selected: bool) -> container::Style {
            let other_bg_color = if theme.extended_palette().is_dark {
                theme.palette().background.lighten(0.2)
            } else {
                theme.palette().background.darken(0.2)
            };

            let bg_color = if row_index & 1 == 0 {
                theme.palette().background
            } else {
                other_bg_color
            };

            let bg_color = if selected {
                if theme.extended_palette().is_dark {
                    other_bg_color.lighten(0.4)
                } else {
                    other_bg_color.darken(0.4)
                }
            } else {
                bg_color
            };

            container::background(bg_color)
        }

        match self {
            State::Loaded {
                selected_item_cursor,
                items,
            } => column![
                row![
                    text!("Clippy"),
                    horizontal_space(),
                    button(text!("Settings")).on_press(Message::OpenSettings)
                ]
                .align_y(Alignment::Center)
                .padding(10),
                scrollable(
                    Column::from_iter(items.iter().enumerate().map(|(index, entry)| {
                        container(
                            text!("{}", entry.data)
                                .size(13)
                                .wrapping(text::Wrapping::None),
                        )
                        .style(move |theme: &iced::Theme| {
                            row_bg_color(theme, index, index == *selected_item_cursor as usize)
                        })
                        .padding(8)
                        .width(Length::Fill)
                        .into()
                    },))
                    .spacing(4),
                ),
            ]
            .into(),
            State::Loading => center(text!("Loading...")).into(),
        }
    }
}
