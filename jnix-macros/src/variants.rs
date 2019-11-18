use crate::{JnixAttributes, ParsedFields};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{punctuated::Punctuated, Ident, LitStr, Token, Variant};

pub struct ParsedVariants {
    names: Vec<Ident>,
    fields: Vec<ParsedFields>,
    enum_class: bool,
}

impl ParsedVariants {
    pub fn new(variants: Punctuated<Variant, Token![,]>) -> Self {
        let size = variants.iter().count();
        let mut names = Vec::with_capacity(size);
        let mut fields = Vec::with_capacity(size);

        for variant in variants {
            names.push(variant.ident);
            fields.push(ParsedFields::new(variant.fields, &JnixAttributes::empty()));
        }

        let only_has_unit_fields = fields.iter().all(ParsedFields::is_unit);

        ParsedVariants {
            names,
            fields,
            enum_class: only_has_unit_fields,
        }
    }

    pub fn generate_enum_into_java(
        self,
        jni_class_name_literal: &LitStr,
        type_name_literal: &LitStr,
        class_name: &str,
    ) -> TokenStream {
        let conversions = if self.enum_class {
            self.generate_enum_class_conversions(
                jni_class_name_literal,
                type_name_literal,
                class_name,
            )
        } else {
            self.generate_sealed_class_conversions(
                jni_class_name_literal,
                type_name_literal,
                class_name,
            )
        };

        let parameters = self
            .fields
            .iter()
            .map(|field| field.generate_enum_variant_parameters());

        let variants = self.names;

        quote! {
            match self {
                #(
                    Self::#variants #parameters => {
                        #conversions
                    }
                )*
            }
        }
    }

    fn generate_enum_class_conversions(
        &self,
        jni_class_name_literal: &LitStr,
        type_name_literal: &LitStr,
        class_name: &str,
    ) -> Vec<TokenStream> {
        self.names
            .iter()
            .map(|variant_name| {
                let variant_name_literal =
                    LitStr::new(&variant_name.to_string(), Span::call_site());

                quote! {
                    let class = env.get_class(#jni_class_name_literal);
                    let variant = env.get_static_field(
                        &class,
                        #variant_name_literal,
                        concat!("L", #jni_class_name_literal, ";"),
                    ).expect(concat!("Failed to convert ",
                        #type_name_literal, "::", #variant_name_literal,
                        " Rust enum variant into ",
                        #class_name,
                        " Java enum class variant",
                    ));

                    match variant {
                        jnix::jni::objects::JValue::Object(object) => env.auto_local(object),
                        _ => panic!(concat!("Conversion from ",
                            #type_name_literal, "::", #variant_name_literal,
                            " Rust enum variant into ",
                            #class_name,
                            " Java object returned an invalid result.",
                        )),
                    }
                }
            })
            .collect()
    }

    fn generate_sealed_class_conversions(
        &self,
        jni_class_name_literal: &LitStr,
        type_name_literal: &LitStr,
        class_name: &str,
    ) -> Vec<TokenStream> {
        let jni_class_name = jni_class_name_literal.value();
        let type_name = type_name_literal.value();

        self.names
            .iter()
            .zip(self.fields.iter())
            .map(|(variant_name, fields)| {
                let variant_class_name = format!("{}.{}", class_name, variant_name);

                let variant_jni_class_name = format!("{}${}", jni_class_name, variant_name);
                let variant_jni_class_name_literal =
                    LitStr::new(&variant_jni_class_name, Span::call_site());

                let variant_type_name = format!("{}::{}", type_name, variant_name);
                let variant_type_name_literal = LitStr::new(&variant_type_name, Span::call_site());

                fields.generate_enum_variant_into_java(
                    &variant_jni_class_name_literal,
                    &variant_type_name_literal,
                    &variant_class_name,
                )
            })
            .collect()
    }
}
