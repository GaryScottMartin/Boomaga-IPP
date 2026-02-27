//! Page layout engine for boomaga
//!
//! This crate provides algorithms for page layout transformations including
//! N-up, booklet creation, and various other print layout options.

pub mod n_up;
pub mod booklet;
pub mod imposition;
pub mod transforms;

pub use n_up::*;
pub use booklet::*;
pub use imposition::*;
pub use transforms::*;
