//! This crate provides high-level extensions to help with the usage of [JNI] in Rust code. Internally,
//! it uses the [`jni-rs`] crate for the low-level JNI operations.
//!
//! Some helper traits are provided, such as:
//!
//! - [`AsJValue`]: for allowing a JNI type to be convected to a `JValue` wrapper type.
//! - [`IntoJava`]: for allowing a Rust type to be converted to a Java type.
//!
//! A [`JnixEnv`] helper type is also provided, which is a [`JNIEnv`] wrapper that contains an
//! internal class cache for preloaded classes.
//!
//! If compiled with the `derive` feature flag, the crate also exports a [derive procedural macro
//! for `IntoJava`][derive-into-java], which allows writing conversion code a lot easier.
//! An example would be:
//!
//! ```rust,ignore
//! use jnix::{jni::JNIEnv, JnixEnv, IntoJava};
//!
//! // Rust type definition
//! #[derive(Default, IntoJava)]
//! #[jnix(package = "my.package")]
//! pub struct MyData {
//!     number: i32,
//!     string: String,
//! }
//!
//! // Some JNI function
//! #[no_mangle]
//! #[allow(non_snake_case)]
//! pub extern "system" fn Java_my_package_JniClass_getData<'env>(
//!     env: JNIEnv<'env>,
//!     _this: JObject<'env>,
//! ) -> JObject<'env> {
//!     // Create the `JnixEnv` wrapper
//!     let env = JnixEnv::from(env);
//!
//!     // Prepare the result type
//!     let data = MyData::default();
//!
//!     // Since a smart pointer is returned from `into_java`, the inner object must be "leaked" so
//!     // that the garbage collector can own it afterwards
//!     data.into_java(&env).forget()
//! }
//! ```
//!
//! ```java
//! package my.package;
//!
//! public class MyData {
//!     public MyData(number: int, string: String) {
//!         // This is the constructor that is called by the generated `IntoJava` code
//!     }
//! }
//! ```
//!
//! [JNI]: https://en.wikipedia.org./wiki/Java_Native_Interface
//! [`jni-rs`]: https://crates.io/crates/jni
//! [`JNIEnv`]: https://docs.rs/jni/0.14.0/jni/struct.JNIEnv.html
//! [`AsJValue`]: https://docs.rs/jnix/0.1.0/jnix/as_jvalue/trait.AsJValue.html
//! [`IntoJava`]: https://docs.rs/jnix/0.1.0/jnix/into_java/trait.IntoJava.html
//! [`JnixEnv`]: https://docs.rs/jnix/0.1.0/jnix/jnix_env/struct.JnixEnv.html
//! [derive-into-java]: https://docs.rs/jnix-macros/0.1.0/jnix_macros/derive.IntoJava.html

#![deny(missing_docs)]

pub extern crate jni;

mod as_jvalue;
mod into_java;
mod jnix_env;

pub use self::{as_jvalue::AsJValue, into_java::IntoJava, jnix_env::JnixEnv};
#[cfg(feature = "derive")]
pub use jnix_macros::IntoJava;
