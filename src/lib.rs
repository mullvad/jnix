//! # JNIX
//!
//! This crate provides some helper high-level extensions for an idiomatic way of using JNI with
//! Rust.

#![deny(missing_docs)]

pub extern crate jni;

mod as_jvalue;
mod into_java;
mod jnix_env;

pub use self::{as_jvalue::AsJValue, into_java::IntoJava, jnix_env::JnixEnv};
#[cfg(feature = "derive")]
pub use jnix_macros::IntoJava;
