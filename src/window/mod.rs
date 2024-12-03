pub mod config;
pub mod main;

#[derive(Debug)]
pub enum Window {
    Main(main::State),
    Config(config::State),
}

