//! Backend-to-preview Unix-socket notification server.

use std::collections::VecDeque;
use std::fs;
use std::io;
use std::path::PathBuf;

use tokio::io::AsyncWriteExt;
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::mpsc;

use crate::Message;
use crate::transport::write_message;

/// Sender used by backend components to enqueue preview notifications.
pub type NotificationSender = mpsc::UnboundedSender<Message>;

/// Server which pairs queued notifications with connecting preview clients.
pub struct NotificationServer {
    socket_path: PathBuf,
    listener: UnixListener,
    receiver: mpsc::UnboundedReceiver<Message>,
}

impl NotificationServer {
    /// Bind a notification socket and return its message sender.
    pub fn bind(socket_path: PathBuf) -> io::Result<(Self, NotificationSender)> {
        if socket_path.exists() {
            fs::remove_file(&socket_path)?;
        }
        let listener = UnixListener::bind(&socket_path)?;
        let (sender, receiver) = mpsc::unbounded_channel();
        Ok((
            Self {
                socket_path,
                listener,
                receiver,
            },
            sender,
        ))
    }

    /// Accept preview clients and deliver each queued notification once.
    pub async fn run(mut self) -> io::Result<()> {
        let mut clients: VecDeque<UnixStream> = VecDeque::new();
        let mut messages: VecDeque<Message> = VecDeque::new();

        loop {
            tokio::select! {
                accepted = self.listener.accept() => {
                    let (mut stream, _) = accepted?;
                    if let Some(message) = messages.pop_front() {
                        write_message(&mut stream, &message).await?;
                        stream.shutdown().await?;
                    } else {
                        clients.push_back(stream);
                    }
                }
                message = self.receiver.recv() => {
                    let Some(message) = message else { return Ok(()); };
                    if let Some(mut stream) = clients.pop_front() {
                        write_message(&mut stream, &message).await?;
                        stream.shutdown().await?;
                    } else {
                        messages.push_back(message);
                    }
                }
            }
        }
    }
}

impl Drop for NotificationServer {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.socket_path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MessageDestination, MessagePayload, MessageSource, UnixSocketTransport};

    #[tokio::test]
    async fn delivers_notification_to_preview_client() {
        let socket_path =
            std::env::temp_dir().join(format!("boomaga-ipc-{}.sock", uuid::Uuid::new_v4()));
        let (server, sender) = NotificationServer::bind(socket_path.clone()).unwrap();
        let server_task = tokio::spawn(server.run());
        let transport = UnixSocketTransport::new(socket_path);
        let receive_task = tokio::spawn(async move { transport.receive_message().await });
        tokio::task::yield_now().await;

        let message = Message::new_notification(
            MessageSource::Backend,
            MessageDestination::Preview,
            MessagePayload::Custom {
                data_type: "notification".to_owned(),
                data: vec![],
            },
        );
        sender.send(message.clone()).unwrap();

        let received = receive_task.await.unwrap().unwrap();
        assert_eq!(received.message_id, message.message_id);
        server_task.abort();
    }
}
