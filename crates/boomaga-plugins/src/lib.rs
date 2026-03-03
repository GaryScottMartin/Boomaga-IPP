//! Plugin system for boomaga virtual printer
//!
//! This crate provides a runtime plugin system for extending boomaga functionality
//! with custom plugins written in Rust or loaded as dynamic libraries.

pub mod core;
pub mod loader;
pub mod api;

pub use core::*;
pub use loader::*;
pub use api::*;
