use crate::{JnixAttributes, ParsedFields, ParsedGenerics, ParsedVariants, TypeParameters};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Data, DeriveInput, Ident, LitStr};

pub struct ParsedType {
    attributes: JnixAttributes,
    type_name: Ident,
    generics: ParsedGenerics,
    data: TypeData,
}

impl ParsedType {
    pub fn new(input: DeriveInput) -> Self {
        let attributes = JnixAttributes::new(&input.attrs);
        let generics = ParsedGenerics::new(&input.generics, &attributes);
        let data = TypeData::from(input.data, &attributes);

        ParsedType {
            attributes,
            type_name: input.ident,
            generics,
            data,
        }
    }

    pub fn generate_from_java(self) -> TokenStream {
        let class_name = self.class_name();

        let type_name = self.type_name;

        let trait_parameters = vec![quote! { 'env }, quote! { 'sub_env }];
        let trait_constraint = vec![quote! { 'env: 'sub_env }];
        let extra_type_bound =
            vec![quote! { jnix::FromJava<'env, jnix::jni::objects::JValue<'sub_env>> }];

        let impl_generics = self.generics.impl_generics(trait_parameters.clone());
        let trait_generics = quote! { <'env, jnix::jni::objects::JObject<'sub_env>> };
        let type_generics = self.generics.type_generics();
        let where_clause = self
            .generics
            .where_clause(trait_constraint, extra_type_bound);

        let jni_class_name = class_name.replace(".", "/");
        let jni_class_name_literal = LitStr::new(&jni_class_name, Span::call_site());

        let body = self.data.generate_from_java_body(
            &jni_class_name_literal,
            &class_name,
            &self.generics.type_parameters(),
        );

        quote! {
            impl #impl_generics jnix::FromJava #trait_generics for #type_name #type_generics
            #where_clause
            {
                const JNI_SIGNATURE: &'static str = concat!("L", #jni_class_name_literal, ";");

                fn from_java(
                    env: &jnix::JnixEnv<'env>,
                    jnix_source_object: jnix::jni::objects::JObject<'sub_env>,
                ) -> Self {
                    #body
                }
            }
        }
    }

    pub fn generate_into_java(self) -> TokenStream {
        let class_name = self.class_name();

        let type_name = self.type_name;
        let type_name_literal = LitStr::new(&type_name.to_string(), Span::call_site());

        let trait_parameters = vec![quote! { 'borrow }, quote! { 'env }];
        let trait_constraint = vec![quote! { 'env: 'borrow }];
        let extra_type_bound = vec![quote! { jnix::IntoJava<'borrow, 'env> }];

        let impl_generics = self.generics.impl_generics(trait_parameters.clone());
        let trait_generics = quote! { < #( #trait_parameters ),* > };
        let type_generics = self.generics.type_generics();
        let where_clause = self
            .generics
            .where_clause(trait_constraint, extra_type_bound);

        let jni_class_name = class_name.replace(".", "/");
        let jni_class_name_literal = LitStr::new(&jni_class_name, Span::call_site());

        let body = self.data.generate_into_java_body(
            &jni_class_name_literal,
            &type_name_literal,
            &class_name,
            &self.generics.type_parameters(),
        );

        quote! {
            impl #impl_generics jnix::IntoJava #trait_generics for #type_name #type_generics
            #where_clause
            {
                const JNI_SIGNATURE: &'static str = concat!("L", #jni_class_name_literal, ";");

                type JavaType = jnix::jni::objects::AutoLocal<'env, 'borrow>;

                #[allow(non_snake_case)]
                fn into_java(self, env: &'borrow jnix::JnixEnv<'env>) -> Self::JavaType {
                    #body
                }
            }
        }
    }

    fn class_name(&self) -> String {
        if let Some(literal) = self.attributes.get_value("class_name") {
            return literal.value();
        }

        if let Some(literal) = self.attributes.get_value("package") {
            let mut class_name = literal.value();

            class_name.push('.');
            class_name.push_str(&self.type_name.to_string());

            return class_name;
        }

        panic!("Missing Java class name");
    }
}

enum TypeData {
    Enum(ParsedVariants),
    Struct(ParsedFields),
}

impl TypeData {
    pub fn from(input_data: Data, attributes: &JnixAttributes) -> Self {
        match input_data {
            Data::Enum(data) => TypeData::Enum(ParsedVariants::new(data.variants)),
            Data::Struct(data) => TypeData::Struct(ParsedFields::new(data.fields, attributes)),
            Data::Union(_) => panic!("Dervie(IntoJava) not supported on unions"),
        }
    }

    pub fn generate_from_java_body(
        self,
        jni_class_name_literal: &LitStr,
        class_name: &str,
        type_parameters: &TypeParameters,
    ) -> TokenStream {
        match self {
            TypeData::Enum(variants) => variants.generate_enum_from_java(
                jni_class_name_literal,
                class_name,
                type_parameters,
            ),
            TypeData::Struct(fields) => fields.generate_struct_from_java(
                jni_class_name_literal,
                class_name,
                type_parameters,
            ),
        }
    }

    pub fn generate_into_java_body(
        self,
        jni_class_name_literal: &LitStr,
        type_name_literal: &LitStr,
        class_name: &str,
        type_parameters: &TypeParameters,
    ) -> TokenStream {
        match self {
            TypeData::Enum(variants) => variants.generate_enum_into_java(
                jni_class_name_literal,
                type_name_literal,
                class_name,
                type_parameters,
            ),
            TypeData::Struct(fields) => fields.generate_struct_into_java(
                jni_class_name_literal,
                type_name_literal,
                class_name,
                type_parameters,
            ),
        }
    }
}
