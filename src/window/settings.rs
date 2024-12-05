use iced::{widget::text, Element, Task};

use crate::app;

#[derive(Debug)]
pub struct State;

#[derive(Debug, Clone)]
pub enum Message {}

impl State {
    pub fn new() -> State {
        State
    }

    pub fn update(&mut self, _: Message) -> Task<app::Message> {
        Task::none()
    }

    pub fn view(&self) -> Element<Message> {
        text!("todo").into()
    }
}
