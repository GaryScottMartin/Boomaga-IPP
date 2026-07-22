//! IPC library for boomaga virtual printer
//!
//! This crate provides inter-process communication mechanisms between
//! the IPP backend service and the preview application using Unix
//! Domain Sockets and D-Bus.

pub mod notifier;
pub mod protocol;
pub mod transport;

pub use notifier::{NotificationSender, NotificationServer};
pub use protocol::{
    Message, MessageDestination, MessagePayload, MessageSource, MessageType, Request, Response,
    PROTOCOL_VERSION,
};
pub use transport::{UnixSocket, UnixSocketTransport};
