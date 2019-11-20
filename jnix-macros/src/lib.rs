#![deny(missing_docs)]

//! # JNIX macros
//!
//! This is a companion crate to `jnix` that provides some procedural macros for interfacing JNI
//! with Rust.

extern crate proc_macro;

mod attributes;
mod fields;
mod parsed_type;
mod variants;

use crate::{
    attributes::JnixAttributes, fields::ParsedFields, parsed_type::ParsedType,
    variants::ParsedVariants,
};
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

/// Derives `IntoJava` for a type.
///
/// The name of the target Java class must be specified using an attribute, like so:
/// `#[jnix(class_name = "my.package.MyClass"]`.
///
/// # Structs
///
/// The generated `IntoJava` implementation for a struct will convert the field values into their
/// respective Java types. Then, the target Java class is constructed by calling a constructor with
/// the converted field values as parameters. Note that the field order is used as the constructor
/// parameter order.
///
/// Fields can be "preconverted" to a different Rust type, so that the resulting type is then used
/// to convert to the Java type. To do so, use the `#[jnix(map = "|value| ...")]` attribute with a
/// conversion closure.
///
/// Fields can be skipped using the `#[jnix(skip)]` attribute, so that they aren't used in the
/// conversion process, and therefore not used as a parameter for the constructor. The
/// `#[jnix(skip_all)]` attribute can be used on the struct to skip all fields.
///
/// # Enums
///
/// The generated `IntoJava` implementation for a enum that only has unit variants (i.e., no tuple
/// or struct variants) returns a static field value from the specified Java target class.  The
/// name used for the static field in the Java class is the same as the Rust variant name. This
/// allows the Rust enum to be mapped to a Java enum.
#[proc_macro_derive(IntoJava, attributes(jnix))]
pub fn derive_into_java(input: TokenStream) -> TokenStream {
    let parsed_type = ParsedType::new(parse_macro_input!(input as DeriveInput));

    TokenStream::from(parsed_type.generate_into_java())
}
