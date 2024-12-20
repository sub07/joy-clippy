use iced::{
    border,
    widget::{button, column, container, horizontal_rule, row, text},
    Alignment, Element, Task,
};

use crate::app::{self, Shortcut};

#[derive(Debug)]
pub enum ShortcutSelectionState {
    Listening(Shortcut),
    NotListening,
}

#[derive(Debug)]
pub struct State {
    pub toggle_shortcut: Shortcut,
    pub shortcut_selection_state: ShortcutSelectionState,
}

#[derive(Debug, Clone)]
pub enum Message {
    NewShortcutInput(Shortcut),
    ToggleShortcutSelection,
}

impl State {
    pub fn new(toggle_shortcut: Shortcut) -> State {
        State {
            toggle_shortcut,
            shortcut_selection_state: ShortcutSelectionState::NotListening,
        }
    }

    pub fn update(&mut self, message: Message) -> Task<app::Message> {
        match message {
            Message::NewShortcutInput(new_shortcut) => match self.shortcut_selection_state {
                ShortcutSelectionState::Listening(ref mut current_shortcut) => {
                    *current_shortcut = new_shortcut;
                    Task::none()
                }
                ShortcutSelectionState::NotListening => Task::none(),
            },
            Message::ToggleShortcutSelection => {
                let (new_state, task) = match self.shortcut_selection_state {
                    ShortcutSelectionState::Listening(ref shortcut) => {
                        self.toggle_shortcut = shortcut.clone();
                        (
                            ShortcutSelectionState::NotListening,
                            Task::done(app::Message::UpdateToggleShortcut(shortcut.clone())),
                        )
                    }
                    ShortcutSelectionState::NotListening => (
                        ShortcutSelectionState::Listening(self.toggle_shortcut.clone()),
                        Task::none(),
                    ),
                };
                self.shortcut_selection_state = new_state;
                task
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let (key_string, is_toggle_shortcut_edition_enabled) = match self.shortcut_selection_state {
            ShortcutSelectionState::Listening(ref shortcut) => (shortcut_string(shortcut), true),
            ShortcutSelectionState::NotListening => (shortcut_string(&self.toggle_shortcut), false),
        };

        let toggle_shortcut_button_label = text!("Toggle shortcut: ");

        let toggle_shortcut_button = button(text(key_string))
            .style(move |theme, status| {
                let mut style = button::primary(theme, status);
                if is_toggle_shortcut_edition_enabled {
                    style.border = border::rounded(2).color(theme.palette().danger).width(3);
                }
                style
            })
            .on_press(Message::ToggleShortcutSelection);

        column![
            text!("Settings").size(30),
            container(horizontal_rule(2)).padding([10, 0]),
            row![toggle_shortcut_button_label, toggle_shortcut_button].align_y(Alignment::Center)
        ]
        .padding(16)
        .into()
    }
}

fn shortcut_string(
    Shortcut {
        modifiers,
        logical_key,
        ..
    }: &Shortcut,
) -> String {
    let key = match logical_key {
        iced::keyboard::Key::Named(named) => format!("{named:#?}"),
        iced::keyboard::Key::Character(c) => c.to_string(),
        iced::keyboard::Key::Unidentified => "�".into(),
    };

    if modifiers.is_empty() {
        key
    } else {
        // TODO: find a better way to destructurate a bitflag structure
        let modifiers = format!("{modifiers:?}");
        let modifiers_pretty = &modifiers[10..modifiers.len() - 1];
        let modifiers_pretty = modifiers_pretty.replace("|", "+");
        format!("{modifiers_pretty} + {key}")
    }
}
