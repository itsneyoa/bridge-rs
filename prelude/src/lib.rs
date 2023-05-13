//! Common types throughout the program
//!
//! ```no_run
//! use crate::prelude::*;
//! ```

mod config;
mod errors;
mod types;

pub use anyhow::anyhow;
pub use config::Config;
pub use errors::BridgeError;
pub use log::{debug, error, info, trace, warn};
pub use types::*;

/// Result type for all functions in this crate
pub type Result<T, E = BridgeError> = std::result::Result<T, E>;

// TODO: Move this to a more appropriate place
/// Trait for options and results which can fail, but we don't care about the error
pub trait Failable {
    /// Consume the result, logging the error if present
    fn failable(self);
}

impl<T> Failable for Result<T> {
    fn failable(self) {
        if let Err(e) = self {
            warn!("{}", e);
        }
    }
}

/// Failable for tuples, typically returned by [`tokio::join`]
impl<T> Failable for (Result<T>, Result<T>) {
    fn failable(self) {
        self.0.failable();
        self.1.failable();
    }
}

impl Failable for Result<Option<FromMinecraft>, async_broadcast::SendError<FromMinecraft>> {
    fn failable(self) {
        if let Err(e) = self {
            warn!("{:?}", e);
        }
    }
}
