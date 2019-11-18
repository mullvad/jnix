use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::iter;
use syn::{
    parse_str, Generics, Ident, Lifetime, Token, TraitBound, TraitBoundModifier, TypeParamBound,
};

pub struct ParsedGenerics {
    parameters: Vec<TokenStream>,
    constraints: Vec<TokenStream>,
}

impl ParsedGenerics {
    pub fn new(generics: &Generics) -> Self {
        let (lifetimes, types) = Self::collect_generic_definitions(generics);
        let parameters = Self::collect_generic_params(&lifetimes, &types);
        let constraints = Self::collect_constraints(generics);

        ParsedGenerics {
            parameters,
            constraints,
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

    fn collect_constraints(generics: &Generics) -> Vec<TokenStream> {
        let extra_type_constraint = Self::create_extra_type_constraint();
        let extra_lifetime_constraints = iter::once(quote! { 'env: 'borrow });

        let lifetime_constraints = generics
            .lifetimes()
            .filter(|definition| definition.colon_token.is_some())
            .map(|definition| quote! { #definition });

        let type_constraints = generics.type_params().cloned().map(|mut type_param| {
            if type_param.colon_token.is_none() {
                type_param.colon_token = Some(Token![:](Span::call_site()));
            }

            type_param.bounds.push(extra_type_constraint.clone());

            quote! { #type_param }
        });

        lifetime_constraints
            .chain(extra_lifetime_constraints)
            .chain(type_constraints)
            .collect()
    }

    fn create_extra_type_constraint() -> TypeParamBound {
        TypeParamBound::Trait(TraitBound {
            paren_token: None,
            modifier: TraitBoundModifier::None,
            lifetimes: None,
            path: parse_str("jnix::IntoJava<'borrow, 'env>")
                .expect("Invalid syntax in hardcoded string"),
        })
    }

    pub fn impl_generics(&self) -> TokenStream {
        let trait_parameters = [quote! { 'borrow }, quote! { 'env }];
        let impl_parameters = trait_parameters.iter().chain(self.parameters.iter());

        quote! { < #( #impl_parameters ),* > }
    }

    pub fn trait_generics(&self) -> TokenStream {
        quote! { <'borrow, 'env> }
    }

    pub fn type_generics(&self) -> Option<TokenStream> {
        let parameters = &self.parameters;

        if parameters.is_empty() {
            None
        } else {
            Some(quote! { < #( #parameters ),* > })
        }
    }

    pub fn where_clause(&self) -> TokenStream {
        let constraints = self.constraints.iter();

        quote! { where #( #constraints ),* }
    }
}
