use crate::JnixAttributes;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::collections::HashMap;
use syn::{Generics, Ident, Lifetime, Path, ReturnType, Token, Type, TypeParam, TypeParamBound};

pub struct ParsedGenerics {
    type_bounds: HashMap<String, String>,
    parameters: Vec<TokenStream>,
    lifetime_constraints: Vec<TokenStream>,
    type_constraints: Vec<TypeParam>,
}

impl ParsedGenerics {
    pub fn new(generics: &Generics, attributes: &JnixAttributes) -> Self {
        let (lifetimes, types) = Self::collect_generic_definitions(generics);
        let parameters = Self::collect_generic_params(&lifetimes, &types);
        let (lifetime_constraints, type_constraints) = Self::collect_constraints(generics);
        let bounds_attribute = attributes
            .get_value("bounds")
            .map(|literal| literal.value());
        let type_bounds = Self::collect_type_bounds(types, bounds_attribute);

        ParsedGenerics {
            type_bounds,
            parameters,
            lifetime_constraints,
            type_constraints,
        }
    }

    fn collect_generic_definitions(generics: &Generics) -> (Vec<Lifetime>, Vec<Ident>) {
        let lifetimes = generics
            .lifetimes()
            .map(|definition| definition.lifetime.clone())
            .collect();

        let types = generics
            .type_params()
            .map(|type_param| type_param.ident.clone())
            .collect();

        (lifetimes, types)
    }

    fn collect_generic_params(lifetimes: &Vec<Lifetime>, types: &Vec<Ident>) -> Vec<TokenStream> {
        let lifetimes = lifetimes.iter().map(|lifetime| quote! { #lifetime });
        let types = types.iter().map(|type_param| quote! { #type_param });

        lifetimes.chain(types).collect()
    }

    fn collect_constraints(generics: &Generics) -> (Vec<TokenStream>, Vec<TypeParam>) {
        let lifetime_constraints = generics
            .lifetimes()
            .filter(|definition| definition.colon_token.is_some())
            .map(|definition| quote! { #definition })
            .collect();

        let type_constraints = generics.type_params().cloned().collect();

        (lifetime_constraints, type_constraints)
    }

    fn collect_type_bounds(types: Vec<Ident>, bounds: Option<String>) -> HashMap<String, String> {
        let mut type_bounds = if let Some(bounds_string) = bounds {
            bounds_string
                .split(",")
                .filter_map(Self::parse_bounds_for_one_type)
                .collect()
        } else {
            HashMap::with_capacity(types.len())
        };

        for maybe_unbounded_type in types.into_iter().map(|identifier| identifier.to_string()) {
            if !type_bounds.contains_key(&maybe_unbounded_type) {
                type_bounds.insert(maybe_unbounded_type, "Ljava/lang/Object;".to_owned());
            }
        }

        type_bounds
    }

    fn parse_bounds_for_one_type(bounds_string: &str) -> Option<(String, String)> {
        let mut parts = bounds_string.splitn(2, ":");
        let bounded_type = parts.next()?.trim().to_owned();
        let bound_class = parts.next()?.trim();
        let bound_signature = format!("L{};", bound_class.replace('.', "/"));

        Some((bounded_type, bound_signature))
    }

    pub fn type_parameters(&self) -> TypeParameters {
        TypeParameters {
            bounds: self.type_bounds.clone(),
        }
    }

    pub fn impl_generics(
        &self,
        trait_parameters: impl IntoIterator<Item = TokenStream>,
    ) -> TokenStream {
        let impl_parameters = trait_parameters
            .into_iter()
            .chain(self.parameters.iter().cloned());

        quote! { < #( #impl_parameters ),* > }
    }

    pub fn type_generics(&self) -> Option<TokenStream> {
        let parameters = &self.parameters;

        if parameters.is_empty() {
            None
        } else {
            Some(quote! { < #( #parameters ),* > })
        }
    }

    pub fn where_clause(
        &self,
        extra_constraints: impl IntoIterator<Item = TokenStream>,
        extra_type_bounds: impl IntoIterator<Item = TokenStream>,
    ) -> TokenStream {
        let constraints = extra_constraints
            .into_iter()
            .chain(self.lifetime_constraints.iter().cloned())
            .chain(self.build_type_constraints(extra_type_bounds));

        quote! { where #( #constraints ),* }
    }

    fn build_type_constraints(
        &self,
        extra_type_bounds_tokens: impl IntoIterator<Item = TokenStream>,
    ) -> impl Iterator<Item = TokenStream> {
        let extra_type_bounds: Vec<TypeParamBound> = extra_type_bounds_tokens
            .into_iter()
            .map(|tokens| syn::parse2(tokens).expect("Invalid type bound specification"))
            .collect();

        self.type_constraints
            .clone()
            .into_iter()
            .map(move |mut type_param| {
                if !extra_type_bounds.is_empty() {
                    if type_param.colon_token.is_none() {
                        type_param.colon_token = Some(Token![:](Span::call_site()));
                    }

                    type_param.bounds.extend(extra_type_bounds.clone());
                }

                quote! { #type_param }
            })
    }
}

pub struct TypeParameters {
    bounds: HashMap<String, String>,
}

impl TypeParameters {
    pub fn is_empty(&self) -> bool {
        self.bounds.is_empty()
    }

    pub fn erased_type_for(&self, type_to_erase: &Type) -> Option<String> {
        match type_to_erase {
            Type::Path(path) => {
                let type_name = path.path.get_ident()?.to_string();

                self.bounds.get(&type_name).cloned()
            }
            complex_path => {
                if self.is_used_in_type(complex_path) {
                    Some("Ljava/lang/Object;".to_owned())
                } else {
                    None
                }
            }
        }
    }

    fn is_used_in_type(&self, type_to_check: &Type) -> bool {
        match type_to_check {
            Type::Never(_) => false,

            Type::Path(path) => self.contains_path(&path.path),

            Type::Array(array) => self.is_used_in_type(&array.elem),
            Type::Group(group) => self.is_used_in_type(&group.elem),
            Type::Paren(paren) => self.is_used_in_type(&paren.elem),
            Type::Ptr(pointer) => self.is_used_in_type(&pointer.elem),
            Type::Reference(reference) => self.is_used_in_type(&reference.elem),
            Type::Slice(slice) => self.is_used_in_type(&slice.elem),

            Type::Tuple(tuple) => tuple.elems.iter().any(|elem| self.is_used_in_type(elem)),

            Type::ImplTrait(impl_trait) => self.is_used_in_bounds(&impl_trait.bounds),
            Type::TraitObject(trait_object) => self.is_used_in_bounds(&trait_object.bounds),

            Type::BareFn(function) => {
                let type_parameter_in_input = function
                    .inputs
                    .iter()
                    .any(|input| self.is_used_in_type(&input.ty));

                if type_parameter_in_input {
                    return true;
                }

                match &function.output {
                    ReturnType::Default => false,
                    ReturnType::Type(_, output) => self.is_used_in_type(&output),
                }
            }

            Type::Infer(_) => panic!("Can't check for type parameter before type is inferred"),
            Type::Macro(_) => panic!("Can't check for type parameter in macro call"),
            Type::Verbatim(_) => panic!("Can't check for type parameter in unstructured tokens"),

            _ => panic!("Can't check for type parameter in unknown type"),
        }
    }

    fn contains_path(&self, path: &Path) -> bool {
        path.get_ident()
            .map(|ident| self.bounds.contains_key(&ident.to_string()))
            .unwrap_or(false)
    }

    fn is_used_in_bounds<'a>(&self, bounds: impl IntoIterator<Item = &'a TypeParamBound>) -> bool {
        bounds.into_iter().any(|bound| match bound {
            TypeParamBound::Lifetime(_) => false,
            TypeParamBound::Trait(bound) => self.contains_path(&bound.path),
        })
    }
}
