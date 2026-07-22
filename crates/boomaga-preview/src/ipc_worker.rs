//! Xilem worker bridge for backend Unix-socket notifications.

use std::fmt;
use std::path::PathBuf;
use std::time::Duration;

use boomaga_ipc::{Message, UnixSocketTransport};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use xilem::core::{MessageProxy, NoElement, View};
use xilem::view::worker;
use xilem::ViewCtx;

use crate::app::AppData;

#[derive(Debug)]
pub enum IpcCommand {
    Connect(PathBuf),
}

pub enum IpcEvent {
    Message(Message),
    Disconnected(String),
}

impl fmt::Debug for IpcEvent {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Message(message) => formatter
                .debug_tuple("Message")
                .field(&message.payload)
                .finish(),
            Self::Disconnected(message) => formatter
                .debug_tuple("Disconnected")
                .field(message)
                .finish(),
        }
    }
}

pub fn ipc_worker() -> impl View<AppData, (), ViewCtx, Element = NoElement> {
    worker(
        run_ipc_worker,
        |data: &mut AppData, sender| data.install_ipc(sender),
        |data: &mut AppData, event| data.handle_ipc_event(event),
    )
}

async fn run_ipc_worker(
    proxy: MessageProxy<IpcEvent>,
    mut receiver: UnboundedReceiver<IpcCommand>,
) {
    while let Some(IpcCommand::Connect(path)) = receiver.recv().await {
        let transport = UnixSocketTransport::new(path);
        loop {
            match transport.receive_message().await {
                Ok(message) => {
                    if proxy.message(IpcEvent::Message(message)).is_err() {
                        return;
                    }
                }
                Err(error) => {
                    if proxy
                        .message(IpcEvent::Disconnected(error.to_string()))
                        .is_err()
                    {
                        return;
                    }
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }
}

pub type IpcSender = UnboundedSender<IpcCommand>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ipc_commands_and_events_are_send() {
        fn assert_send<T: Send>() {}
        assert_send::<IpcCommand>();
        assert_send::<IpcEvent>();
    }
}
