#![deny(missing_docs)]

//! # JNIX macros
//!
//! This is a companion crate to `jnix` that provides some procedural macros for interfacing JNI
//! with Rust.

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{
    parse_macro_input, Attribute, Data, DeriveInput, Fields, Ident, Lit, LitStr, MetaNameValue,
};

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
    let type_name = parsed_input.ident;
    let type_name_literal = LitStr::new(&type_name.to_string(), Span::call_site());
    let class_name = parse_java_class_name(&parsed_input.attrs).expect("Missing Java class name");
    let jni_class_name = class_name.replace(".", "/");
    let jni_class_name_literal = LitStr::new(&jni_class_name, Span::call_site());

    let fields = extract_struct_fields(parsed_input.data);
    let (parameter_conversion, parameter_signatures, parameters) = generate_parameters(fields);

    let tokens = quote! {
        impl<'borrow, 'env: 'borrow> jnix::IntoJava<'borrow, 'env> for #type_name {
            const JNI_SIGNATURE: &'static str = concat!("L", #jni_class_name_literal, ";");

            type JavaType = jnix::jni::objects::AutoLocal<'env, 'borrow>;

            fn into_java(self, env: &'borrow jnix::JnixEnv<'env>) -> Self::JavaType {
                let mut constructor_signature = String::with_capacity(
                    1 + #( #parameter_signatures.as_bytes().len() + )* 2
                );

                constructor_signature.push_str("(");
                #( constructor_signature.push_str(#parameter_signatures); )*
                constructor_signature.push_str(")V");

                #( #parameter_conversion )*

                let parameters = [ #( jnix::AsJValue::as_jvalue(&#parameters) ),* ];

                let class = env.get_class(#jni_class_name_literal);
                let object = env.new_object(&class, constructor_signature, &parameters)
                    .expect(concat!(
                        "Failed to convert ",
                        #type_name_literal,
                        " Rust type into ",
                        #class_name,
                        " Java object",
                    ));

                env.auto_local(object)
            }
        }
    };

    TokenStream::from(tokens)
}

fn parse_java_class_name(attributes: &Vec<Attribute>) -> Option<String> {
    let jnix_ident = Ident::new("jnix", Span::call_site());
    let jnix_attribute = attributes
        .iter()
        .find(|attribute| attribute.path.is_ident(&jnix_ident))?;
    let meta: MetaNameValue = jnix_attribute.parse_args().expect("Invalid jnix attribute");

    if meta
        .path
        .is_ident(&Ident::new("class_name", Span::call_site()))
    {
        if let Lit::Str(class_name) = meta.lit {
            Some(class_name.value())
        } else {
            None
        }
    } else {
        None
    }
}

fn extract_struct_fields(data: Data) -> Fields {
    match data {
        Data::Struct(data) => data.fields,
        _ => panic!("Dervie(IntoJava) only supported on structs"),
    }
}

fn generate_parameters(
    fields: Fields,
) -> (Vec<TokenStream2>, Vec<TokenStream2>, Vec<TokenStream2>) {
    match fields {
        Fields::Unit => (vec![], vec![], vec![]),
        _ => panic!("Derive(IntoJava) only supported on unit structs"),
    }
}
