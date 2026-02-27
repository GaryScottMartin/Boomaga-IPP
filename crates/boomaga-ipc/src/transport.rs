//! Unix socket transport implementation

use std::fs;
use std::io;
use std::net::{Shutdown, UnixListener, UnixStream};
use std::os::unix::net::UnixStreamExt;
use std::path::PathBuf;
use std::time::Duration;
use tokio::net::UnixStream as TokioUnixStream;
use tokio::sync::mpsc;
use tracing::{info, error, debug};
use crate::protocol::{Message, MessageType};

/// Unix socket transport
pub struct UnixSocket {
    /// Socket path
    socket_path: PathBuf,
    /// Listener
    listener: Option<UnixListener>,
    /// Client connections
    clients: Vec<TokioUnixStream>,
    /// Receiver channel for incoming messages
    receiver: mpsc::Receiver<io::Result<Message>>,
}

impl UnixSocket {
    /// Create a new Unix socket
    pub fn new(socket_path: PathBuf) -> Result<Self, io::Error> {
        // Remove existing socket file if present
        if socket_path.exists() {
            fs::remove_file(&socket_path)?;
        }

        let listener = UnixListener::bind(&socket_path)?;
        info!("Unix socket created at: {:?}", socket_path);

        Ok(Self {
            socket_path,
            listener: Some(listener),
            clients: Vec::new(),
            receiver: mpsc::channel(100),
        })
    }

    /// Start listening for connections
    pub async fn listen(&mut self) -> Result<(), io::Error> {
        let listener = self.listener.take().ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotConnected, "Socket listener not initialized")
        })?;

        // Accept connections in a task
        tokio::spawn(async move {
            loop {
                match listener.accept() {
                    Ok((stream, addr)) => {
                        info!("New client connected: {}", addr);
                        // Store client
                        // In production, would maintain client connections
                    }
                    Err(e) => {
                        error!("Error accepting client: {}", e);
                    }
                }
            }
        });

        Ok(())
    }

    /// Send a message to the socket
    pub async fn send(&self, message: Message) -> Result<(), io::Error> {
        // In production, this would serialize and send the message
        // For now, just log it
        debug!("Sending message: {:?} to {:?}", message.message_type, message.destination);

        Ok(())
    }

    /// Receive a message from the socket
    pub async fn recv(&mut self) -> Option<Message> {
        // In production, this would receive messages from the socket
        // For now, return None
        None
    }

    /// Send raw bytes
    pub async fn send_bytes(&self, bytes: Vec<u8>) -> Result<(), io::Error> {
        // In production, serialize and send bytes
        debug!("Sending {} bytes", bytes.len());
        Ok(())
    }

    /// Receive raw bytes
    pub async fn recv_bytes(&mut self) -> Option<Vec<u8>> {
        // In production, receive and deserialize bytes
        None
    }

    /// Close the socket
    pub fn close(&mut self) -> Result<(), io::Error> {
        // Close all client connections
        for client in self.clients.drain(..) {
            let _ = client.shutdown(Shutdown::Both);
        }

        // Remove socket file
        if self.socket_path.exists() {
            fs::remove_file(&self.socket_path)?;
        }

        info!("Unix socket closed: {:?}", self.socket_path);

        Ok(())
    }

    /// Check if socket is connected
    pub fn is_connected(&self) -> bool {
        !self.clients.is_empty()
    }
}

/// Unix socket transport for async usage
pub struct UnixSocketTransport {
    /// Socket path
    socket_path: PathBuf,
}

impl UnixSocketTransport {
    /// Create a new transport
    pub fn new(socket_path: PathBuf) -> Self {
        Self { socket_path }
    }

    /// Connect to the socket
    pub async fn connect(&self) -> Result<TokioUnixStream, io::Error> {
        let timeout = Duration::from_secs(5);

        // In production, would implement actual connection logic
        // For now, just return a mock stream
        info!("Connecting to socket at: {:?}", self.socket_path);

        Ok(TokioUnixStream::connect(&self.socket_path).await?)
    }

    /// Send message
    pub async fn send_message(&self, message: Message) -> Result<(), io::Error> {
        let stream = self.connect().await?;

        // In production, serialize and send message
        debug!("Sending message: {:?}", message.message_type);

        Ok(())
    }

    /// Receive message
    pub async fn receive_message(&self) -> Result<Message, io::Error> {
        let stream = self.connect().await?;

        // In production, receive and deserialize message
        Ok(Message::new_request(
            crate::protocol::MessageSource::Ipc,
            crate::protocol::MessageDestination::Backend,
            crate::protocol::MessagePayload::Custom {
                data_type: "ping".to_string(),
                data: vec![],
            },
        ))
    }
}
