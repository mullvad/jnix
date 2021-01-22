use crate::{JnixAttributes, ParsedFields, TypeParameters};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{punctuated::Punctuated, Ident, LitStr, Token, Variant};

pub struct ParsedVariant {
    pub name: Ident,
    pub fields: ParsedFields,
    pub attributes: JnixAttributes,
}

impl From<Variant> for ParsedVariant {
    fn from(variant: Variant) -> Self {
        ParsedVariant {
            name: variant.ident,
            fields: ParsedFields::new(variant.fields, &JnixAttributes::empty()),
            attributes: JnixAttributes::new(&variant.attrs),
        }
    }
}

pub struct ParsedVariants {
    variants: Vec<ParsedVariant>,
    enum_class: bool,
}

impl ParsedVariants {
    pub fn new(variants: Punctuated<Variant, Token![,]>) -> Self {
        let variants: Vec<_> = variants.into_iter().map(ParsedVariant::from).collect();
        let only_has_unit_fields = variants.iter().all(|variant| variant.fields.is_unit());

        ParsedVariants {
            variants,
            enum_class: only_has_unit_fields,
        }
    }

    pub fn generate_enum_from_java(
        self,
        jni_class_name_literal: &LitStr,
        class_name: &str,
        type_parameters: &TypeParameters,
    ) -> TokenStream {
        let (class_binding, error_description, conversions) = if self.enum_class {
            let class_binding = quote! { let class = env.get_class(#jni_class_name_literal); };
            let error_description = "Java enum class entry";
            let conversions =
                self.generate_enum_class_from_java_conversions(jni_class_name_literal, class_name);

            (class_binding, error_description, conversions)
        } else {
            let class_binding = quote! {};
            let error_description = "sub-class";
            let conversions = self.generate_sealed_class_from_java_conversions(
                jni_class_name_literal,
                class_name,
                type_parameters,
            );

            (class_binding, error_description, conversions)
        };

        quote! {
            #class_binding

            None
                #( .or_else(|| { #conversions }) )*
                .unwrap_or_else(|| panic!(
                    concat!("Invalid ", #error_description, " of ", #jni_class_name_literal),
                ))
        }
    }

    pub fn generate_enum_into_java(
        self,
        jni_class_name_literal: &LitStr,
        type_name_literal: &LitStr,
        class_name: &str,
        type_parameters: &TypeParameters,
    ) -> TokenStream {
        let conversions = if self.enum_class {
            self.generate_enum_class_into_java_conversions(
                jni_class_name_literal,
                type_name_literal,
                class_name,
            )
        } else {
            self.generate_sealed_class_into_java_conversions(
                jni_class_name_literal,
                type_name_literal,
                class_name,
                type_parameters,
            )
        };

        let parameters = self
            .variants
            .iter()
            .map(|variant| variant.fields.generate_enum_variant_parameters());

        let variants = self.variants.iter().map(|variant| &variant.name);

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

    fn generate_enum_class_from_java_conversions<'borrow, 'jni_class_name_literal, 'class_name>(
        &'borrow self,
        jni_class_name_literal: &'jni_class_name_literal LitStr,
        class_name: &'class_name str,
    ) -> Box<dyn Iterator<Item = TokenStream> + 'borrow>
    where
        'jni_class_name_literal: 'borrow,
        'class_name: 'borrow,
    {
        Box::new(self.variants.iter().map(move |variant| {
            let variant_name = &variant.name;
            let span = variant_name.span();
            let variant_name_literal = LitStr::new(&variant_name.to_string(), span);
            let variant_class_name = LitStr::new(&format!("{}.{}", class_name, variant_name), span);

            let constructor = if variant.attributes.has_flag("deny") {
                quote! {
                    panic!(
                        concat!("Can't create variant ", #variant_name_literal, " from Java type"),
                    );
                }
            } else {
                quote! { Some(Self::#variant_name) }
            };

            quote! {
                let candidate = env
                    .get_static_field(
                        &class,
                        #variant_name_literal,
                        concat!("L", #jni_class_name_literal, ";"),
                    )
                    .expect(concat!("Failed to get Java enum class variant ", #variant_class_name));

                match candidate {
                    jnix::jni::objects::JValue::Object(candidate) => {
                        let found = env
                            .is_same_object(source, candidate)
                            .expect(concat!(
                                "Failed to compare object to enum class entry of ",
                                #variant_class_name,
                            ));

                        if found {
                            #constructor
                        } else {
                            None
                        }
                    }
                    _ => panic!(concat!(
                        "Invalid Java enum class variant retrieved for ",
                        #variant_class_name,
                    )),
                }
            }
        }))
    }

    fn generate_sealed_class_from_java_conversions<
        'borrow,
        'jni_class_name_literal,
        'class_name,
        'type_parameters,
    >(
        &'borrow self,
        jni_class_name_literal: &'jni_class_name_literal LitStr,
        class_name: &'class_name str,
        type_parameters: &'type_parameters TypeParameters,
    ) -> Box<dyn Iterator<Item = TokenStream> + 'borrow>
    where
        'jni_class_name_literal: 'borrow,
        'class_name: 'borrow,
        'type_parameters: 'borrow,
    {
        let jni_class_name = jni_class_name_literal.value();

        Box::new(self.variants.iter().map(move |variant| {
            let variant_name_literal = LitStr::new(&variant.name.to_string(), variant.name.span());
            let variant_class_name = format!("{}.{}", class_name, variant.name);
            let variant_class_name_literal = LitStr::new(&variant_class_name, variant.name.span());

            let variant_jni_class_name = format!("{}${}", jni_class_name, variant.name);
            let variant_jni_class_name_literal =
                LitStr::new(&variant_jni_class_name, Span::call_site());

            let constructor = if variant.attributes.has_flag("deny") {
                quote! {
                    panic!(
                        concat!("Can't create variant ", #variant_name_literal, " from Java type"),
                    );
                }
            } else {
                let fields_constructor = variant.fields.generate_enum_variant_from_java(
                    &variant_jni_class_name_literal,
                    &variant_class_name,
                    &variant.name,
                    type_parameters,
                );

                quote! { Some({ #fields_constructor }) }
            };

            quote! {
                let candidate = env.get_class(#variant_jni_class_name_literal);
                let found = env.is_instance_of(source, &candidate)
                    .expect(concat!(
                        "Failed to check if object is an instance of class ",
                        #variant_class_name_literal,
                    ));

                if found {
                    #constructor
                } else {
                    None
                }
            }
        }))
    }

    fn generate_enum_class_into_java_conversions(
        &self,
        jni_class_name_literal: &LitStr,
        type_name_literal: &LitStr,
        class_name: &str,
    ) -> Vec<TokenStream> {
        self.variants
            .iter()
            .map(|variant| {
                let variant_name_literal =
                    LitStr::new(&variant.name.to_string(), Span::call_site());

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

    fn generate_sealed_class_into_java_conversions(
        &self,
        jni_class_name_literal: &LitStr,
        type_name_literal: &LitStr,
        class_name: &str,
        type_parameters: &TypeParameters,
    ) -> Vec<TokenStream> {
        let jni_class_name = jni_class_name_literal.value();
        let type_name = type_name_literal.value();

        self.variants
            .iter()
            .map(|variant| {
                let variant_class_name = format!("{}.{}", class_name, variant.name);

                let variant_jni_class_name = format!("{}${}", jni_class_name, variant.name);
                let variant_jni_class_name_literal =
                    LitStr::new(&variant_jni_class_name, Span::call_site());

                if variant.fields.is_unit() {
                    self.generate_unit_variant_into_java_conversion(
                        &variant_jni_class_name,
                        variant_jni_class_name_literal,
                        &variant_class_name,
                    )
                } else {
                    self.generate_variant_with_fields_into_java_conversion(
                        variant_jni_class_name_literal,
                        &type_name,
                        &variant_class_name,
                        type_parameters,
                        variant,
                    )
                }
            })
            .collect()
    }

    fn generate_unit_variant_into_java_conversion(
        &self,
        variant_jni_class_name: &str,
        variant_jni_class_name_literal: LitStr,
        variant_class_name: &str,
    ) -> TokenStream {
        let variant_class_name_literal = LitStr::new(&variant_class_name, Span::call_site());

        let field_signature = format!("L{};", variant_jni_class_name);
        let field_signature_literal = LitStr::new(&field_signature, Span::call_site());

        quote! {
            let class = env.get_class(#variant_jni_class_name_literal);

            let field_id = env
                .get_static_field_id(&class, "INSTANCE", #field_signature_literal)
                .expect(concat!(
                    "Failed to get field ID for ",
                    #variant_class_name_literal,
                    ".INSTANCE"
                ));

            let field_type =
                jnix::jni::signature::JavaType::Object(#variant_jni_class_name_literal.to_owned());

            let instance = env
                .get_static_field_unchecked(&class, field_id, field_type)
                .expect(concat!(
                    "Failed to retrieve ",
                    #variant_class_name_literal,
                    ".INSTANCE static field"
                ))
                .l()
                .expect(concat!(
                    "The ",
                    #variant_class_name_literal,
                    ".INSTANCE field is not an object"
                ));

            env.auto_local(instance)
        }
    }

    fn generate_variant_with_fields_into_java_conversion(
        &self,
        variant_jni_class_name_literal: LitStr,
        type_name: &str,
        variant_class_name: &str,
        type_parameters: &TypeParameters,
        variant: &ParsedVariant,
    ) -> TokenStream {
        let variant_type_name = format!("{}::{}", type_name, variant.name);
        let variant_type_name_literal = LitStr::new(&variant_type_name, Span::call_site());

        variant.fields.generate_enum_variant_into_java(
            &variant_jni_class_name_literal,
            &variant_type_name_literal,
            &variant_class_name,
            type_parameters,
        )
    }
}
