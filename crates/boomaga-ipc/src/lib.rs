//! IPC library for boomaga virtual printer
//!
//! This crate provides inter-process communication mechanisms between
//! the IPP backend service and the preview application using Unix
//! Domain Sockets and D-Bus.

pub mod protocol;
pub mod transport;

pub use protocol::{Message, MessageType, Request, Response};
pub use transport::{UnixSocket, UnixSocketTransport};
