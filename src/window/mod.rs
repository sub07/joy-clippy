pub mod settings;
pub mod history;

#[derive(Debug)]
pub enum Window {
    History(history::State),
    Settings(settings::State),
}

