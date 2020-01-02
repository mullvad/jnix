use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::collections::HashSet;
use syn::{Generics, Ident, Lifetime, Path, ReturnType, Token, Type, TypeParam, TypeParamBound};

pub struct ParsedGenerics {
    type_parameters: Vec<Ident>,
    parameters: Vec<TokenStream>,
    lifetime_constraints: Vec<TokenStream>,
    type_constraints: Vec<TypeParam>,
}

impl ParsedGenerics {
    pub fn new(generics: &Generics) -> Self {
        let (lifetimes, types) = Self::collect_generic_definitions(generics);
        let parameters = Self::collect_generic_params(&lifetimes, &types);
        let (lifetime_constraints, type_constraints) = Self::collect_constraints(generics);

        ParsedGenerics {
            type_parameters: types,
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

    pub fn type_parameters(&self) -> TypeParameters {
        TypeParameters {
            params: self
                .type_parameters
                .iter()
                .map(|param| param.to_string())
                .collect(),
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

    pub fn trait_generics(
        &self,
        trait_parameters: impl IntoIterator<Item = TokenStream>,
    ) -> TokenStream {
        let trait_parameters = trait_parameters.into_iter();

        quote! { < #( #trait_parameters ),* > }
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
    params: HashSet<String>,
}

impl TypeParameters {
    pub fn is_used_in_type(&self, type_to_check: &Type) -> bool {
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
            .map(|ident| self.params.contains(&ident.to_string()))
            .unwrap_or(false)
    }

    fn is_used_in_bounds<'a>(&self, bounds: impl IntoIterator<Item = &'a TypeParamBound>) -> bool {
        bounds.into_iter().any(|bound| match bound {
            TypeParamBound::Lifetime(_) => false,
            TypeParamBound::Trait(bound) => self.contains_path(&bound.path),
        })
    }
}
