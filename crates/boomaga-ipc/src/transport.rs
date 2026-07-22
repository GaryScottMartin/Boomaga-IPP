//! Unix socket transport implementation

use crate::protocol::{Message, MessageType, PROTOCOL_VERSION};
use std::fs;
use std::io;
use std::os::unix::net::UnixListener;
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::net::UnixStream as TokioUnixStream;
use tokio::sync::mpsc;
use tracing::{debug, error, info};

/// Write one newline-delimited JSON message.
pub async fn write_message<W>(writer: &mut W, message: &Message) -> io::Result<()>
where
    W: AsyncWrite + Unpin,
{
    let mut encoded = serde_json::to_vec(message)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    encoded.push(b'\n');
    writer.write_all(&encoded).await
}

/// Read one newline-delimited JSON message.
pub async fn read_message<R>(reader: R) -> io::Result<Message>
where
    R: AsyncRead + Unpin,
{
    let mut encoded = String::new();
    if BufReader::new(reader).read_line(&mut encoded).await? == 0 {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "socket closed",
        ));
    }
    let message: Message = serde_json::from_str(&encoded)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    if message.protocol_version != PROTOCOL_VERSION {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unsupported protocol version {}", message.protocol_version),
        ));
    }
    Ok(message)
}

/// Unix socket transport
pub struct UnixSocket {
    /// Socket path
    socket_path: PathBuf,
    /// Listener
    listener: Option<UnixListener>,
    /// Client connections
    clients: Vec<TokioUnixStream>,
    /// Receiver channel for incoming messages
    receiver: Option<mpsc::Receiver<io::Result<Message>>>,
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
            receiver: Some(mpsc::channel(100).1),
        })
    }

    /// Start listening for connections
    pub async fn listen(&mut self) -> Result<(), io::Error> {
        let listener = self.listener.take().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotConnected,
                "Socket listener not initialized",
            )
        })?;

        // Accept connections in a task
        tokio::spawn(async move {
            loop {
                match listener.accept() {
                    Ok((stream, addr)) => {
                        info!("New client connected: {:?}", addr);
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
        debug!(
            "Sending message: {:?} to {:?}",
            message.message_type, message.destination
        );

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
        self.clients.clear();

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
        info!("Connecting to socket at: {:?}", self.socket_path);

        Ok(TokioUnixStream::connect(&self.socket_path).await?)
    }

    /// Send message
    pub async fn send_message(&self, message: Message) -> Result<(), io::Error> {
        let mut stream = self.connect().await?;
        debug!("Sending message: {:?}", message.message_type);
        write_message(&mut stream, &message).await?;
        stream.shutdown().await
    }

    /// Receive message
    pub async fn receive_message(&self) -> Result<Message, io::Error> {
        let stream = self.connect().await?;
        read_message(stream).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{MessageDestination, MessagePayload, MessageSource, PROTOCOL_VERSION};

    #[tokio::test]
    async fn framed_message_round_trip() {
        let message = Message::new_notification(
            MessageSource::Backend,
            MessageDestination::Preview,
            MessagePayload::Custom {
                data_type: "test".to_owned(),
                data: vec![1, 2, 3],
            },
        );
        let (mut writer, reader) = tokio::io::duplex(4096);

        write_message(&mut writer, &message).await.unwrap();
        let decoded = read_message(reader).await.unwrap();

        assert_eq!(decoded.protocol_version, PROTOCOL_VERSION);
        assert_eq!(decoded.message_id, message.message_id);
        match decoded.payload {
            MessagePayload::Custom { data_type, data } => {
                assert_eq!(data_type, "test");
                assert_eq!(data, vec![1, 2, 3]);
            }
            payload => panic!("unexpected payload: {payload:?}"),
        }
    }

    #[tokio::test]
    async fn rejects_unsupported_protocol_version() {
        let mut message = Message::new_notification(
            MessageSource::Backend,
            MessageDestination::Preview,
            MessagePayload::Custom {
                data_type: "test".to_owned(),
                data: vec![],
            },
        );
        message.protocol_version = PROTOCOL_VERSION + 1;
        let (mut writer, reader) = tokio::io::duplex(4096);
        write_message(&mut writer, &message).await.unwrap();

        let error = read_message(reader).await.unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    }
}
