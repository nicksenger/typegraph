use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, Type};

use crate::{
    argument, generic,
    id::{hashed, uid},
    Outcome, NODE_DATA_LABEL,
};

#[derive(deluxe::ParseMetaItem, deluxe::ExtractAttributes, Default)]
#[deluxe(attributes(typegraph))]
struct Attributes {
    #[deluxe(default)]
    generics: Vec<Type>,
    #[deluxe(default)]
    force_ret: Option<Type>,
    #[deluxe(default)]
    skip_ret: bool,
    #[deluxe(default)]
    skip: bool,
}

pub struct State {
    pub id: proc_macro2::TokenStream,
    pub mod_id: Ident,
    pub return_type: Option<Type>,
    pub name: proc_macro2::TokenStream,
    pub fns_name: Ident,
    pub trait_name: Ident,
    pub kind: proc_macro2::TokenStream,
    pub output_kind: proc_macro2::TokenStream,
    pub arg_names: Vec<proc_macro2::TokenStream>,
    pub arg_ids: Vec<proc_macro2::TokenStream>,
    pub arg_types: Vec<Type>,
}

impl State {
    pub fn try_from_fn(
        function: &mut syn::ImplItemFn,
        ident: &Ident,
        id: &Ident,
    ) -> Result<Outcome<Self>, TokenStream> {
        let Attributes {
            generics,
            force_ret,
            skip_ret,
            skip,
        } = deluxe::extract_attributes(function).unwrap_or_default();
        if !generics.is_empty() && generics.len() != function.sig.generics.type_params().count() {
            return Err(syn::Error::new_spanned(
                function,
                "The number of typegraph generic substitutions differs from the number of generic type parameters.",
            )
            .to_compile_error()
            .into());
        }
        if skip {
            return Ok(Outcome::Skip);
        }

        let mut generic_sub = generic::Substitution::new(generics, &function.sig.generics);

        let mut chars = function.sig.ident.to_string().chars().collect::<Vec<_>>();
        chars[0] = chars[0].to_uppercase().next().unwrap_or('a');
        let capitalized = chars.into_iter().filter(|c| *c != '_').collect::<String>();
        let name = function.sig.ident.clone();
        let generics = function.sig.generics.clone();
        let mod_id = format_ident!(
            "typegraph_{}_nodeimpl_{}_{}",
            ident.to_string().to_lowercase(),
            id.to_string().to_lowercase(),
            name.to_string().to_lowercase(),
        );
        let trait_name = format_ident!(
            "{}Impl{}Method{}",
            ident,
            id,
            function
                .sig
                .ident
                .to_string()
                .chars()
                .filter(|c| *c != '_')
                .collect::<String>()
        );
        let fns_name = format_ident!("{}Method{}{}{}", NODE_DATA_LABEL, ident, capitalized, id);

        let args = argument::extract_impl_fn(function);
        let arg_names = args.iter().map(|(n, _)| quote! { #n }).collect::<Vec<_>>();
        let (arg_ids, arg_types): (Vec<proc_macro2::TokenStream>, Vec<_>) = args
            .into_iter()
            .map(|(n, t)| (hashed(quote! { #n }.into()).into(), t))
            .unzip();

        let return_type = force_ret.or_else(|| {
            if skip_ret {
                None
            } else {
                match &function.sig.output {
                    syn::ReturnType::Default => None,
                    syn::ReturnType::Type(_, t) => Some(generic_sub.substitute(*t.clone())),
                }
            }
        });
        let (kind, output_kind) = if function.sig.asyncness.is_some() {
            (
                quote! { NodeKind::AsyncFunction },
                quote! { NodeOutputKind::AsyncFunction },
            )
        } else {
            (
                quote! { NodeKind::Function },
                quote! { NodeOutputKind::Function },
            )
        };

        Ok(Outcome::Connect(Self {
            id: uid(),
            mod_id,
            return_type,
            name: quote! { #name #generics },
            fns_name,
            trait_name,
            kind,
            output_kind,
            arg_names,
            arg_ids,
            arg_types: generic_sub.substitute_all(arg_types),
        }))
    }
}
