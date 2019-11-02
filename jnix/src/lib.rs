#![deny(missing_docs)]

//! # JNIX
//!
//! This crate provides some helper high-level extensions for an idiomatic way of using JNI with
//! Rust.

pub extern crate jni;

mod jnix_env;

pub use self::jnix_env::JnixEnv;
