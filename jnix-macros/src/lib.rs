#![deny(missing_docs)]

//! # JNIX macros
//!
//! This is a companion crate to `jnix` that provides some procedural macros for interfacing JNI
//! with Rust.

extern crate proc_macro;

mod attributes;
mod fields;

use crate::{attributes::JnixAttributes, fields::ParsedFields};
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, LitStr};

/// Derives `IntoJava` for a type.
///
/// The generated `IntoJava` implementation for a unit struct will simply call the default
/// constructor of the target Java class.
///
/// The name of the target Java class must be specified using an attribute, like so:
/// `#[jnix(class_name = "my.package.MyClass"]`.
#[proc_macro_derive(IntoJava, attributes(jnix))]
pub fn derive_into_java(input: TokenStream) -> TokenStream {
    let parsed_input = parse_macro_input!(input as DeriveInput);
    let attributes = JnixAttributes::new(&parsed_input.attrs);
    let type_name = parsed_input.ident;
    let type_name_literal = LitStr::new(&type_name.to_string(), Span::call_site());
    let class_name = attributes
        .get_value("class_name")
        .expect("Missing Java class name")
        .value();
    let jni_class_name = class_name.replace(".", "/");
    let jni_class_name_literal = LitStr::new(&jni_class_name, Span::call_site());

    let fields = extract_struct_fields(parsed_input.data);
    let conversion = ParsedFields::new(fields).generate_struct_into_java(
        &jni_class_name_literal,
        &type_name_literal,
        &class_name,
    );

    let tokens = quote! {
        impl<'borrow, 'env: 'borrow> jnix::IntoJava<'borrow, 'env> for #type_name {
            const JNI_SIGNATURE: &'static str = concat!("L", #jni_class_name_literal, ";");

            type JavaType = jnix::jni::objects::AutoLocal<'env, 'borrow>;

            fn into_java(self, env: &'borrow jnix::JnixEnv<'env>) -> Self::JavaType {
                #conversion
            }
        }
    };

    TokenStream::from(tokens)
}

fn extract_struct_fields(data: Data) -> Fields {
    match data {
        Data::Struct(data) => data.fields,
        _ => panic!("Dervie(IntoJava) only supported on structs"),
    }
}
