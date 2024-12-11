pub mod history;
pub mod settings;

#[derive(Debug)]
pub enum Window {
    History(history::State),
    Settings(settings::State),
}
