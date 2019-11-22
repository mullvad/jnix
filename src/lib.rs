//! # JNIX
//!
//! This crate provides high-level extensions to help with the usage of [JNI] in Rust code. Internally,
//! it uses the [`jni-rs`] crate for the low-level JNI operations.
//!
//! [JNI]: https://en.wikipedia.org./wiki/Java_Native_Interface
//! [`jni-rs`]: https://crates.io/crates/jni

#![deny(missing_docs)]

pub extern crate jni;

mod as_jvalue;
mod into_java;
mod jnix_env;

pub use self::{as_jvalue::AsJValue, into_java::IntoJava, jnix_env::JnixEnv};
#[cfg(feature = "derive")]
pub use jnix_macros::IntoJava;
