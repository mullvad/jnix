#![deny(missing_docs)]

//! # JNIX
//!
//! This crate provides some helper high-level extensions for an idiomatic way of using JNI with
//! Rust.

pub extern crate jni;

mod as_jvalue;
mod jnix_env;

pub use self::{as_jvalue::AsJValue, jnix_env::JnixEnv};
