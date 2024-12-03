use iced::{
    widget::{self, button, column, container, scrollable, text},
    Element, Length, Task,
};

use crate::{app, utils::ColorUtils};

#[derive(Debug)]
pub struct State {
    selected_item_cursor: i32,
    clipboard_history: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    MoveHistoryCursor(i32),
    Paste,
}

impl State {
    pub fn new(clipboard_history: Vec<String>) -> Self {
        Self {
            selected_item_cursor: 0,
            clipboard_history,
        }
    }

    pub fn update(&mut self, event: Message) -> Task<app::Message> {
        match event {
            Message::MoveHistoryCursor(direction) => {
                self.selected_item_cursor += direction;
                if self.selected_item_cursor < 0 {
                    self.selected_item_cursor = 0
                }
                if self.selected_item_cursor >= self.clipboard_history.len() as i32 {
                    self.selected_item_cursor = self.clipboard_history.len() as i32 - 1;
                }
                Task::none()
            }
            Message::Paste => Task::done(app::Message::Paste(
                self.clipboard_history.len() - self.selected_item_cursor as usize - 1,
            )),
        }
    }

    pub fn view(&self) -> Element<Message> {
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

            container::background(bg_color)
        }

        column![
            button(text!("Paste")).on_press(Message::Paste),
            scrollable(
                widget::Column::from_iter(self.clipboard_history.iter().rev().enumerate().map(
                    |(index, entry)| {
                        container(text!("{entry}").size(13))
                            .style(move |theme: &iced::Theme| {
                                row_bg_color(
                                    theme,
                                    index,
                                    index == self.selected_item_cursor as usize,
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
