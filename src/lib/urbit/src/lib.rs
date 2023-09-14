//! # urbit
//!
//! the `urbit` crate is a collection of modules and other utilities to expose Urbit data
//! and other Urbit features and functionality within a Rust operating environment.
//!
//! More specifically, this crate exposes a general Urbit operating environment to a host
//! container; in this case the host container being a `holon`.

pub mod context;
pub mod error;
pub mod helper;

pub mod lens;
pub mod process;

pub mod api;
pub mod chat;
pub mod db;
pub mod sub;
pub mod ws;
