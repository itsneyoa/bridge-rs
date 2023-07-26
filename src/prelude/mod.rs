//! Common types throughout the program
//!
//! ```no_run
//! use crate::prelude::*;
//! ```

mod errors;
pub mod sanitiser;

pub use anyhow::anyhow;
pub use errors::BridgeError;
use futures::future::poll_fn;
pub use log::{debug, trace};
use once_cell::sync::{Lazy, OnceCell};
use std::{
    ops::Deref,
    sync::{Arc, Mutex},
    task::{Poll, Waker},
};

/// Result type for all functions in this crate
pub type Result<T, E = BridgeError> = std::result::Result<T, E>;

/// Trait for options and results which can fail, but we don't care about the error
pub trait Failable {
    /// Consume the result, logging the error if present
    fn failable(self);
}

impl<T> Failable for Result<T> {
    fn failable(self) {
        if let Err(e) = self {
            crate::output::send(e, crate::output::Warn);
        }
    }
}

/// A chat which messages can be sent from and to
#[derive(Debug, PartialEq, Clone)]
pub enum Chat {
    /// Guild chat varient
    ///
    /// Uses the `GUILD_CHANNEL_ID` ENV and `/gc` as the command prefix
    Guild,
    /// Officer chat
    ///
    /// Uses the `OFFICER_CHANNEL_ID` ENV and `/oc` as the command prefix
    Officer,
}

impl Chat {
    /// Get the prefix to send a chat message to Minecraft
    pub fn prefix(&self) -> &str {
        match self {
            Chat::Guild => "gc",
            Chat::Officer => "oc",
        }
    }
}

/// A [`OnceCell`] that can be awaited
#[derive(Debug)]
pub struct AsyncOnceCell<T>(OnceCell<T>, Lazy<Arc<Mutex<Option<Waker>>>>);

impl<T> AsyncOnceCell<T> {
    /// Create a new [`AsyncOnceCell`]
    pub const fn new() -> Self {
        Self(OnceCell::new(), Lazy::new(|| Arc::new(Mutex::new(None))))
    }

    /// Set the value of the cell
    pub fn set(&self, val: T) -> Result<(), T> {
        let res = self.0.set(val);

        if let Some(ref waker) = *self.1.lock().expect("Failed to acquire waker lock") {
            waker.wake_by_ref()
        }

        res
    }

    /// Asynchronously wait for the cell to be populated and return the inner values
    pub async fn wait(&self) -> &T {
        poll_fn(|cx| match self.0.get() {
            Some(val) => Poll::Ready(val),
            None => {
                let mut current = self.1.lock().expect("Failed to acquire waker lock");

                if !current
                    .as_ref()
                    .is_some_and(|waker| waker.will_wake(cx.waker()))
                {
                    *current = Some(cx.waker().clone());
                }

                Poll::Pending
            }
        })
        .await
    }
}

impl<T> Deref for AsyncOnceCell<T> {
    type Target = OnceCell<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
