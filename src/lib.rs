#![allow(clippy::type_complexity)]

pub mod actor;
pub mod actor_spawner;
pub mod addr;
pub mod broker;
pub mod caller;
pub mod context;
pub mod leacteur;
pub mod message_producer;
pub mod messages;
pub mod runtime;
pub mod service;
pub mod supervisor;
pub mod utils;

pub use anyhow as error;
/// Alias of error::Result
pub type Result<T> = error::Result<T>;

/// Alias of error::Error
pub type Error = error::Error;

pub type ActorId = u64;

pub use actor::{Actor, Handler, Message, StreamHandler};
pub use addr::{Addr, WeakAddr};
pub use broker::Broker;
pub use caller::{Caller, Sender};
pub use context::Context;
pub use runtime::{block_on, sleep, spawn, timeout};
pub use service::{LocalService, Service};
pub use supervisor::Supervisor;
