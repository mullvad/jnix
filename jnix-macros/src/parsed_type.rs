use crate::{JnixAttributes, ParsedFields};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Data, DeriveInput, Ident, LitStr};

pub struct ParsedType {
    attributes: JnixAttributes,
    type_name: Ident,
    data: TypeData,
}

impl ParsedType {
    pub fn new(input: DeriveInput) -> Self {
        let attributes = JnixAttributes::new(&input.attrs);
        let data = TypeData::from(input.data, &attributes);

        ParsedType {
            attributes,
            type_name: input.ident,
            data,
        }
    }

    pub fn generate_into_java(self) -> TokenStream {
        let type_name = self.type_name;
        let type_name_literal = LitStr::new(&type_name.to_string(), Span::call_site());

        let class_name = self
            .attributes
            .get_value("class_name")
            .expect("Missing Java class name")
            .value();

        let jni_class_name = class_name.replace(".", "/");
        let jni_class_name_literal = LitStr::new(&jni_class_name, Span::call_site());

        let body = self.data.generate_into_java_body(
            &jni_class_name_literal,
            &type_name_literal,
            &class_name,
        );

        quote! {
            impl<'borrow, 'env: 'borrow> jnix::IntoJava<'borrow, 'env> for #type_name {
                const JNI_SIGNATURE: &'static str = concat!("L", #jni_class_name_literal, ";");

                type JavaType = jnix::jni::objects::AutoLocal<'env, 'borrow>;

                #[allow(non_snake_case)]
                fn into_java(self, env: &'borrow jnix::JnixEnv<'env>) -> Self::JavaType {
                    #body
                }
            }
        }
    }
}

enum TypeData {
    Struct(ParsedFields),
}

impl TypeData {
    pub fn from(input_data: Data, attributes: &JnixAttributes) -> Self {
        match input_data {
            Data::Struct(data) => TypeData::Struct(ParsedFields::new(data.fields, attributes)),
            _ => panic!("Dervie(IntoJava) only supported on structs"),
        }
    }

    pub fn generate_into_java_body(
        self,
        jni_class_name_literal: &LitStr,
        type_name_literal: &LitStr,
        class_name: &str,
    ) -> TokenStream {
        match self {
            TypeData::Struct(fields) => fields.generate_struct_into_java(
                jni_class_name_literal,
                type_name_literal,
                class_name,
            ),
        }
    }
}
