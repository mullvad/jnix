use crate::{JnixAttributes, TypeParameters};
use heck::MixedCase;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    parse_str, spanned::Spanned, ExprClosure, Field, Fields, Ident, Index, LitStr, Member, Pat,
    PatType, Token, Type,
};

pub struct ParsedField {
    pub name: String,
    pub field: Field,
    pub attributes: JnixAttributes,
    pub member: Member,
    pub source_binding: Ident,
    pub span: Span,
    pub skip: bool,
}

impl ParsedField {
    pub fn new(name: String, field: Field, member: Member, span: Span) -> Self {
        let attributes = JnixAttributes::new(&field.attrs);
        let source_binding = Ident::new(&format!("_source_{}", name), span);
        let skip = attributes.has_flag("skip");

        ParsedField {
            name,
            field,
            attributes,
            member,
            source_binding,
            span,
            skip,
        }
    }

    pub fn from_named_field(field: Field) -> Self {
        let ident = field.ident.clone().expect("Named field with no name ident");
        let span = ident.span();
        let name = ident.to_string();
        let member = Member::Named(ident);

        ParsedField::new(name, field, member, span)
    }

    pub fn from_unnamed_field((field, index): (Field, u32)) -> Self {
        let span = field.ty.span();
        let name = format!("_{}", index);
        let member = Member::Unnamed(Index { index, span });

        ParsedField::new(name, field, member, span)
    }

    pub fn get_type(&self) -> &Type {
        &self.field.ty
    }

    pub fn binding(&self, prefix: &str) -> Ident {
        Ident::new(&format!("_{}_{}", prefix, self.name), self.span)
    }

    pub fn preconversion(&self) -> TokenStream {
        let source = &self.source_binding;

        match self.attributes.get_value("map") {
            Some(closure_string_literal) => {
                let mut closure = parse_str(&closure_string_literal.value())
                    .expect("Invalid closure syntax in jnix(map = ...) attribute");

                self.prepare_map_closure(&mut closure);

                quote! { (#closure)(#source) }
            }
            None => quote! { #source },
        }
    }

    fn prepare_map_closure(&self, closure: &mut ExprClosure) {
        assert!(
            closure.inputs.len() <= 1,
            "Too many parameters in jnix(map = ...) closure"
        );

        let input = closure
            .inputs
            .pop()
            .expect("Missing parameter in jnix(map = ...) closure")
            .into_value();

        closure.inputs.push_value(self.add_type_to_parameter(input));
    }

    fn add_type_to_parameter(&self, parameter: Pat) -> Pat {
        if let &Pat::Type(_) = &parameter {
            parameter
        } else {
            Pat::Type(PatType {
                attrs: vec![],
                pat: Box::new(parameter),
                colon_token: Token![:](Span::call_site()),
                ty: Box::new(self.field.ty.clone()),
            })
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FieldType {
    Unit,
    Named,
    Unnamed,
}

pub struct ParsedFields {
    fields: Vec<ParsedField>,
    field_type: FieldType,
}

impl ParsedFields {
    pub fn new(fields: Fields, attributes: &JnixAttributes) -> Self {
        let field_type = Self::get_field_type(&fields);
        let mut fields = Self::collect_parsed_fields(fields);

        if attributes.has_flag("skip_all") {
            for field in &mut fields {
                field.skip = true;
            }
        }

        ParsedFields { fields, field_type }
    }

    fn get_field_type(fields: &Fields) -> FieldType {
        match fields {
            Fields::Unit => FieldType::Unit,
            Fields::Named(_) => FieldType::Named,
            Fields::Unnamed(_) => FieldType::Unnamed,
        }
    }

    fn collect_parsed_fields(fields: Fields) -> Vec<ParsedField> {
        match fields {
            Fields::Unit => vec![],
            Fields::Named(fields) => fields
                .named
                .into_iter()
                .map(ParsedField::from_named_field)
                .collect(),
            Fields::Unnamed(fields) => fields
                .unnamed
                .into_iter()
                .zip(0..)
                .map(ParsedField::from_unnamed_field)
                .collect(),
        }
    }

    pub fn is_unit(&self) -> bool {
        self.field_type == FieldType::Unit
    }

    pub fn generate_enum_variant_parameters(&self) -> TokenStream {
        let names = self.original_bindings();

        match self.field_type {
            FieldType::Unit => quote! {},
            FieldType::Named => quote! { { #( #names ),* } },
            FieldType::Unnamed => quote! { ( #( #names ),* ) },
        }
    }

    pub fn generate_struct_from_java(
        &self,
        jni_class_name_literal: &LitStr,
        class_name: &str,
        type_parameters: &TypeParameters,
    ) -> TokenStream {
        self.generate_from_java(
            jni_class_name_literal,
            class_name,
            type_parameters,
            quote! { Self },
        )
    }

    pub fn generate_enum_variant_from_java(
        &self,
        jni_class_name_literal: &LitStr,
        class_name: &str,
        variant: &Ident,
        type_parameters: &TypeParameters,
    ) -> TokenStream {
        self.generate_from_java(
            jni_class_name_literal,
            class_name,
            type_parameters,
            quote! { Self::#variant },
        )
    }

    pub fn generate_struct_into_java(
        &self,
        jni_class_name_literal: &LitStr,
        type_name_literal: &LitStr,
        class_name: &str,
        type_parameters: &TypeParameters,
    ) -> TokenStream {
        let source_bindings = self.source_bindings();
        let members = self.members();
        let conversion = self.generate_into_java_conversion(
            jni_class_name_literal,
            type_name_literal,
            class_name,
            type_parameters,
        );

        quote! {
            #( let #source_bindings = self.#members; )*
            #conversion
        }
    }

    pub fn generate_enum_variant_into_java(
        &self,
        jni_class_name_literal: &LitStr,
        type_name_literal: &LitStr,
        class_name: &str,
        type_parameters: &TypeParameters,
    ) -> TokenStream {
        let source_bindings = self.source_bindings();
        let original_bindings = self.original_bindings();
        let conversion = self.generate_into_java_conversion(
            jni_class_name_literal,
            type_name_literal,
            class_name,
            type_parameters,
        );

        quote! {
            #( let #source_bindings = #original_bindings; )*
            #conversion
        }
    }

    fn generate_from_java(
        &self,
        jni_class_name_literal: &LitStr,
        class_name: &str,
        type_parameters: &TypeParameters,
        constructor_name: TokenStream,
    ) -> TokenStream {
        let names = self.original_bindings();
        let constructor = self.generate_enum_variant_parameters();
        let conversions = self.generate_from_java_conversions(class_name, type_parameters);
        let class_binding = if self.fields.is_empty() {
            quote! {}
        } else {
            quote! { let class = env.get_class(#jni_class_name_literal); }
        };

        quote! {
            #class_binding
            #( let #names = { #conversions }; )*

            #constructor_name #constructor
        }
    }

    fn generate_from_java_conversions<'a, 'b: 'a, 'c: 'a>(
        &'a self,
        class_name: &'b str,
        type_parameters: &'c TypeParameters,
    ) -> impl Iterator<Item = TokenStream> + 'a {
        self.fields.iter().map(move |field| {
            let getter_name = format!("get_{}", field.name).to_mixed_case();
            let getter_literal = LitStr::new(&getter_name, Span::call_site());
            let field_type = field.get_type();

            let jni_signature = if type_parameters.is_used_in_type(&field_type) {
                quote! { "Ljava/lang/Object;" }
            } else {
                quote! {
                    <#field_type as jnix::FromJava<jnix::jni::objects::JValue>>::JNI_SIGNATURE
                }
            };

            quote! {
                let jni_signature = #jni_signature;
                let method_signature = format!("(){}", jni_signature);
                let method_id = env.get_method_id(&class, #getter_literal, &method_signature)
                    .expect(
                        concat!("Failed to get method ID for ", #class_name, "::", #getter_literal),
                    );
                let return_type = jni_signature.parse().unwrap_or_else(|_| {
                    panic!("Invalid JNI signature: {}", jni_signature);
                });

                let java_value = env.call_method_unchecked(source, method_id, return_type, &[])
                    .expect(concat!("Failed to call ", #class_name, "::", #getter_literal));

                <#field_type as jnix::FromJava<_>>::from_java(env, java_value)
            }
        })
    }

    fn generate_into_java_conversion(
        &self,
        jni_class_name_literal: &LitStr,
        type_name_literal: &LitStr,
        class_name: &str,
        type_parameters: &TypeParameters,
    ) -> TokenStream {
        let signature_bindings = self.bindings("signature").collect();
        let final_bindings = self.bindings("final").collect();
        let declarations = self.declarations(&signature_bindings, &final_bindings, type_parameters);

        quote! {
            #( #declarations )*

            let mut constructor_signature = String::with_capacity(
                1 + #( #signature_bindings.as_bytes().len() + )* 2
            );

            constructor_signature.push_str("(");
            #( constructor_signature.push_str(#signature_bindings); )*
            constructor_signature.push_str(")V");

            let parameters = [ #( jnix::AsJValue::as_jvalue(&#final_bindings) ),* ];

            let class = env.get_class(#jni_class_name_literal);
            let object = env.new_object(&class, constructor_signature, &parameters)
                .expect(concat!("Failed to convert ",
                    #type_name_literal,
                    " Rust type into ",
                    #class_name,
                    " Java object",
                ));

            env.auto_local(object)
        }
    }

    fn declarations<'a, 'b, 'c, 'd, 'z>(
        &'a self,
        signature_bindings: &'b Vec<Ident>,
        final_bindings: &'c Vec<Ident>,
        type_parameters: &'d TypeParameters,
    ) -> impl Iterator<Item = TokenStream> + 'z
    where
        'a: 'z,
        'b: 'z,
        'c: 'z,
        'd: 'z,
    {
        self.fields
            .iter()
            .filter(|field| !field.skip)
            .zip(signature_bindings.iter().zip(final_bindings.iter()))
            .map(move |(field, (signature_binding, final_binding))| {
                let converted_binding = field.binding("converted");
                let conversion = field.preconversion();

                let signature = if let Some(target) = field.attributes.get_value("target_class") {
                    let signature = format!("L{};", target.value().replace(".", "/"));

                    quote! { #signature }
                } else if type_parameters.is_used_in_type(&field.get_type()) {
                    quote! { "Ljava/lang/Object;" }
                } else {
                    quote! { #converted_binding.jni_signature() }
                };

                quote! {
                    let #converted_binding = #conversion;
                    let #signature_binding = #signature;
                    let #final_binding = #converted_binding.into_java(env);
                }
            })
    }

    fn original_bindings(&self) -> impl Iterator<Item = Ident> + '_ {
        let is_named = self.field_type == FieldType::Named;

        self.fields.iter().map(move |field| {
            if is_named && field.skip {
                Ident::new(&format!("_{}", field.name), field.span)
            } else {
                Ident::new(&field.name, field.span)
            }
        })
    }

    fn source_bindings(&self) -> impl Iterator<Item = &Ident> + '_ {
        self.fields
            .iter()
            .filter(|field| !field.skip)
            .map(|field| &field.source_binding)
    }

    fn bindings(&self, prefix: &'static str) -> impl Iterator<Item = Ident> + '_ {
        self.fields
            .iter()
            .filter(|field| !field.skip)
            .map(move |field| field.binding(prefix))
    }

    fn members(&self) -> impl Iterator<Item = &Member> + '_ {
        self.fields
            .iter()
            .filter(|field| !field.skip)
            .map(|field| &field.member)
    }
}
