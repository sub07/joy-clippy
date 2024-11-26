use std::thread;

use clipboard_rs::{ClipboardHandler, ClipboardWatcher, ClipboardWatcherContext};
use iced::{
    futures::{SinkExt, Stream},
    stream,
};
use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::window::AppMessage;

pub struct ClipboardListener(Sender<()>);

impl ClipboardListener {
    pub fn new() -> (ClipboardListener, Receiver<()>) {
        let (tx, rx) = mpsc::channel(100);
        (ClipboardListener(tx), rx)
    }

    pub fn subscribe() -> impl Stream<Item = AppMessage> {
        stream::channel(100, |mut output| async move {
            let (listener, mut rx) = ClipboardListener::new();
            thread::spawn(|| {
                let mut clipboard_watcher: ClipboardWatcherContext<ClipboardListener> =
                    ClipboardWatcherContext::new().unwrap();
                clipboard_watcher.add_handler(listener).start_watch();
            });

            loop {
                rx.recv().await.unwrap();
                output.send(AppMessage::ClipboardEvent).await.unwrap();
            }
        })
    }
}

impl ClipboardHandler for ClipboardListener {
    fn on_clipboard_change(&mut self) {
        self.0.blocking_send(()).unwrap();
    }
}
