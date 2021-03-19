#![allow(clippy::type_complexity)]

pub mod actor;
pub mod lab;

pub use anyhow as error;
/// Alias of error::Result
pub type Result<T> = error::Result<T>;

/// Alias of error::Error
pub type Error = error::Error;

pub type ActorId = u64;
