use proc_macro2::TokenStream;
use quote::quote;
use syn::{Fields, LitStr};

struct ParsedField;

pub struct ParsedFields {
    _fields: Vec<ParsedField>,
}

impl ParsedFields {
    pub fn new(fields: Fields) -> Self {
        ParsedFields {
            _fields: Self::collect_parsed_fields(fields),
        }
    }

    fn collect_parsed_fields(fields: Fields) -> Vec<ParsedField> {
        match fields {
            Fields::Unit => vec![],
            _ => panic!("Only unit structs are currently supported"),
        }
    }

    pub fn generate_struct_into_java(
        &self,
        jni_class_name_literal: &LitStr,
        type_name_literal: &LitStr,
        class_name: &str,
    ) -> TokenStream {
        self.generate_into_java_conversion(jni_class_name_literal, type_name_literal, class_name)
    }

    fn generate_into_java_conversion(
        &self,
        jni_class_name_literal: &LitStr,
        type_name_literal: &LitStr,
        class_name: &str,
    ) -> TokenStream {
        quote! {
            let mut constructor_signature = String::with_capacity(3);

            constructor_signature.push_str("()V");

            let class = env.get_class(#jni_class_name_literal);
            let object = env.new_object(&class, constructor_signature, &[])
                .expect(concat!("Failed to convert ",
                    #type_name_literal,
                    " Rust type into ",
                    #class_name,
                    " Java object",
                ));

            env.auto_local(object)
        }
    }
}
