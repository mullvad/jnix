//! This is a companion crate to [`jnix`] that provides some procedural macros for interfacing JNI
//! with Rust. See the [`jnix` crate documentation][doc] for more information
//!
//! [`jnix`]: https://crates.io/crates/jnix
//! [doc]: https://docs.rs/jnix/

#![deny(missing_docs)]

extern crate proc_macro;

mod attributes;
mod fields;
mod generics;
mod parsed_type;
mod variants;

use crate::{
    attributes::JnixAttributes,
    fields::ParsedFields,
    generics::{ParsedGenerics, TypeParameters},
    parsed_type::ParsedType,
    variants::ParsedVariants,
};
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

/// Derives `FromJava` for a type.
///
/// More specifically, `FromJava<'env, JObject<'sub_env>>` is derived for the type. This also makes
/// available a `FromJava<'env, AutoLocal<'sub_env, 'borrow>>` implementation through a blanket
/// implementation.
///
/// The name of the target Java class must be known for code generation. Either it can be specified
/// explicitly using an attribute, like so: `#[jnix(class_name = "my.package.MyClass"]`, or it can
/// be derived from the Rust type name as long as the containing Java package is specified using an
/// attribute, like so: `#[jnix(package = "my.package")]`.
///
/// # Structs
///
/// The generated `FromJava` implementation for a struct will construct the Rust type using values
/// for the fields obtained using getter methods. Each field name is prefixed with `get_` before
/// converted to mixed case (also known sometimes as camel case). Therefore, the source object must
/// have the necessary getter methods for the Rust type to be constructed correctly.
///
/// For tuple structs, since the fields don't have names, the field index starting from one is
/// used as the name.  Therefore, the source object must have getter methods named `component1`,
/// `component2`, ..., `componentN` for the "N" number of fields present in the Rust type. This
/// follows the convention used by Kotlin for a `data class`, so they are automatically generated
/// by the Kotlin complier for `data class`es.
///
/// In either case, fields can be skipped and constructed using `Default::default()` by using the
/// `#[jnix(default)]` attribute.
///
/// # Enums
///
/// The generate `FromJava` implementation for an enum that only has unit variants (i.e, no tuple
/// or struct variants) assumes that the source object is an instance of an `enum class`. The
/// source reference is compared to the static fields representing the entries of the `enum class`,
/// and once an entry is found matching the source reference, the respective variant is
/// constructed.
///
/// When an enum has at least one tuple or struct variant, the generated `FromJava` implementation
/// will assume that that there is a class hierarchy to represent the type. The source Java class
/// is assumed to be the super class, and a nested static class for each variant is assumed to be
/// declared in that super class. The source object is checked to see which of the sub-classes it
/// is an instance of. Once a sub-class is found, the respective variant is created similarly to
/// how a struct is constructed, so the same rules regarding the presence of getter methods apply.
///
/// # Examples
///
/// ## Structs with named fields
///
/// ```rust
/// #[derive(FromJava)]
/// #[jnix(package = "my.package")]
/// pub struct MyClass {
///     first_field: String,
///     second_field: String,
/// }
/// ```
///
/// ```java
/// package my.package;
///
/// public class MyClass {
///     private String firstField;
///     private String secondField;
///
///     public MyClass(String first, String second) {
///         firstField = first;
///         secondField = second;
///     }
///
///     // The following getter methods are used to obtain the values to build the Rust struct.
///     public String getFirstField() {
///         return firstField;
///     }
///
///     public String setSecondField() {
///         return secondField;
///     }
/// }
/// ```
///
/// ## Tuple structs
///
/// ```rust
/// #[derive(FromJava)]
/// #[jnix(class_name = "my.package.CustomClass")]
/// pub struct MyTupleStruct(String, String);
/// ```
///
/// ```java
/// package my.package;
///
/// public class CustomClass {
///     private String firstField;
///     private String secondField;
///
///     public MyClass(String first, String second) {
///         firstField = first;
///         secondField = second;
///     }
///
///     // The following getter methods are used to obtain the values to build the Rust tuple
///     // struct.
///     public String component1() {
///         return firstField;
///     }
///
///     public String component2() {
///         return secondField;
///     }
/// }
/// ```
///
/// ## Simple enums
///
/// ```rust
/// #[derive(FromJava)]
/// #[jnix(package "my.package")]
/// pub enum SimpleEnum {
///     First,
///     Second,
/// }
/// ```
///
/// ```java
/// package my.package;
///
/// public enum SimpleEnum {
///     First, Second
/// }
/// ```
///
/// ## Enums with fields
///
/// ```rust
/// #[derive(FromJava)]
/// #[jnix(package "my.package")]
/// pub enum ComplexEnum {
///     First {
///         name: String,
///     },
///     Second(String, String),
/// }
/// ```
///
/// ```java
/// package my.package;
///
/// public class ComplexEnum {
///     public static class First extends ComplexEnum {
///         private String name;
///
///         public First(String name) {
///             this.name = name;
///         }
///
///         public String getName() {
///             return name;
///         }
///     }
///
///     public static class Second extends ComplexEnum {
///         private String a;
///         private String b;
///
///         public Second(String a, String b) {
///             this.a = a;
///             this.b = b;
///         }
///
///         public String component1() {
///             return a;
///         }
///
///         public String component2() {
///             return b;
///         }
///     }
/// }
/// ```
#[proc_macro_derive(FromJava, attributes(jnix))]
pub fn derive_from_java(input: TokenStream) -> TokenStream {
    let parsed_type = ParsedType::new(parse_macro_input!(input as DeriveInput));

    TokenStream::from(parsed_type.generate_from_java())
}

/// Derives `IntoJava` for a type.
///
/// The name of the target Java class must be known for code generation. Either it can be specified
/// explicitly using an attribute, like so: `#[jnix(class_name = "my.package.MyClass"]`, or it can
/// be derived from the Rust type name as long as the containing Java package is specified using an
/// attribute, like so: `#[jnix(package = "my.package")]`.
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
/// The target class of a specific field can be set manually with the
/// `#[jnix(target_class = "...")]` attribute. However, be aware that the target class must have
/// the expected constructor with the parameter list based on the field order of the Rust type.
///
/// # Enums
///
/// The generated `IntoJava` implementation for a enum that only has unit variants (i.e., no tuple
/// or struct variants) returns a static field value from the specified Java target class.  The
/// name used for the static field in the Java class is the same as the Rust variant name. This
/// allows the Rust enum to be mapped to a Java enum.
///
/// When an enum has at least one tuple or struct variant, the generated `IntoJava` implementation
/// will assume that that there is a class hierarchy to represent the type. The target Java class
/// is used as the super class, and is the Java type returned from the conversion. The class is
/// assumed to have one inner class for each variant, and the conversion actually instantiates an
/// object for the respective variant type, using the same rules for the fields as the rules for
/// struct fields.
///
/// For both cases, variants can be prevented from being constructed from their respective Java
/// entries or sub-classes by using the `#[jnix(deny)]` attribute. If one of the entries is used in
/// an attempt to convert to the equivalent Rust variant, the code panics.
#[proc_macro_derive(IntoJava, attributes(jnix))]
pub fn derive_into_java(input: TokenStream) -> TokenStream {
    let parsed_type = ParsedType::new(parse_macro_input!(input as DeriveInput));

    TokenStream::from(parsed_type.generate_into_java())
}
