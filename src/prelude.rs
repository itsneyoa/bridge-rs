//! Common types throughout the program
//!
//! ```no_run
//! use crate::prelude::*;
//! ```

pub use crate::errors::BridgeError;

/// Result type for all functions in this crate
pub type Result<T, E = BridgeError> = std::result::Result<T, E>;
