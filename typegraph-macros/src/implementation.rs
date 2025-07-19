use itertools::Itertools;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, ImplItem, Type, TypePath};

use crate::id::uid;
use crate::{function, Outcome};
use crate::{generic, NODE_DATA_LABEL};

#[derive(deluxe::ParseMetaItem, deluxe::ExtractAttributes)]
#[deluxe(attributes(typegraph))]
struct Attributes {
    id: Ident,
    meta: Option<Type>,
    #[deluxe(default)]
    cluster: Option<syn::Path>,
    #[deluxe(default)]
    generics: Vec<Type>,
}
#[derive(deluxe::ParseMetaItem, deluxe::ExtractAttributes)]
#[deluxe(attributes(typegraph))]
struct Attributes2(Ident, Option<Type>, Option<syn::Path>, Option<Vec<Type>>);
impl From<Attributes2> for Attributes {
    fn from(value: Attributes2) -> Self {
        Self {
            id: value.0,
            meta: value.1,
            cluster: value.2,
            generics: value.3.unwrap_or_default(),
        }
    }
}

#[derive(Default)]
struct State {
    arg_ids: Vec<Vec<proc_macro2::TokenStream>>,
    arg_names: Vec<Vec<proc_macro2::TokenStream>>,
    arg_types: Vec<Vec<syn::Type>>,
    fn_names: Vec<proc_macro2::TokenStream>,
    fn_trait_names: Vec<Ident>,
    fns: Vec<Ident>,
    fn_kinds: Vec<proc_macro2::TokenStream>,
    fn_mod_ids: Vec<Ident>,
    fn_output_kinds: Vec<proc_macro2::TokenStream>,
    fn_return_types: Vec<Option<syn::Type>>,
    fn_ids: Vec<proc_macro2::TokenStream>,
}

impl State {
    fn add_fn(&mut self, f: function::State) {
        self.arg_ids.push(f.arg_ids);
        self.arg_names.push(f.arg_names);
        self.arg_types.push(f.arg_types);
        self.fn_names.push(f.name);
        self.fns.push(f.fns_name);
        self.fn_trait_names.push(f.trait_name);
        self.fn_kinds.push(f.kind);
        self.fn_output_kinds.push(f.output_kind);
        self.fn_return_types.push(f.return_type);
        self.fn_ids.push(f.id);
        self.fn_mod_ids.push(f.mod_id);
    }
}

pub fn impl_macro(attr: TokenStream, mut implementation: syn::ItemImpl) -> TokenStream {
    let Attributes {
        id,
        meta,
        cluster,
        generics,
    } = match deluxe::parse(attr.clone())
        .or_else(|_| deluxe::parse::<Attributes2>(attr.clone()).map(Into::into))
    {
        Ok(desc) => desc,
        Err(e) => return e.into_compile_error().into(),
    };
    if !generics.is_empty() && generics.len() != implementation.generics.type_params().count() {
        return syn::Error::new_spanned(
            implementation,
            "The number of typegraph generic substitutions differs from the number of generic type parameters.",
        )
        .to_compile_error()
        .into();
    }
    let (impl_generics, _ty_generics, _where_clause) = implementation.generics.split_for_impl();
    //let impl_generics = if generics.is_empty() {
    //    quote! { #impl_generics }
    //} else {
    //    quote! { #ty_generics }
    //};
    let mut generic_sub = generic::Substitution::new(generics, &implementation.generics);
    let self_ty = implementation.self_ty.clone();
    let ident = match &*self_ty {
        Type::Path(TypePath { path, .. }) => path.segments.last().map(|seg| seg.ident.clone()),
        Type::Tuple(_) => None,
        _ => None,
    }
    .unwrap_or(format_ident!("unknown"));
    let impl_node_label = format_ident!("{}Impl{}{}", NODE_DATA_LABEL, ident, id);
    let impl_trait_name = format_ident!("{}Impl{}", ident, id);
    let impl_id = uid();

    let mut state = State::default();
    for item in &mut implementation.items {
        if let ImplItem::Fn(f) = item {
            match function::State::try_from_fn(f, &ident, &id) {
                Ok(Outcome::Skip) => {
                    continue;
                }
                Ok(Outcome::Connect(f)) => {
                    state.add_fn(f);
                }
                Err(tt) => {
                    return tt;
                }
            }
        }
    }

    #[cfg(feature = "inert")]
    {
        return quote! { #implementation }.into();
    }

    let mod_id = format_ident!("typegraph_{}", impl_node_label.to_string().to_lowercase());

    let impl_metadata: proc_macro2::TokenStream = match &meta {
        Some(ty) => quote! { #ty },
        None => quote! { #mod_id::#impl_node_label },
    };
    let subgraph = cluster
        .into_iter()
        .flat_map(|s| s.segments.into_iter().map(|p| p.ident))
        .collect::<Vec<_>>();
    let fn_subgraphs = state
        .fns
        .iter()
        .map(|_| subgraph.clone())
        .collect::<Vec<_>>();

    let return_types = state
        .fn_return_types
        .into_iter()
        .map(|o| {
            o.map(|x| vec![generic_sub.substitute(x)])
                .unwrap_or_default()
        })
        .collect::<Vec<_>>();

    let arg_edges = state
        .arg_types
        .iter()
        .map(|v_ty| {
            v_ty.iter()
                .map(|ty| {
                    quote! {
                        ::typegraph::Edge<
                            <#ty as ::typegraph::Typegraph>::Id,
                            Self::Id,
                        >
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let ret_edges = return_types
        .iter()
        .map(|v_ty| {
            v_ty.iter()
                .map(|ty| {
                    quote! {
                        ::typegraph::Edge<
                            Self::Id,
                            <#ty as ::typegraph::Typegraph>::Id,
                        >
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let flat_arg_types = state
        .arg_types
        .iter()
        .flatten()
        .cloned()
        .unique()
        .map(|ty| generic_sub.substitute(ty))
        .collect::<Vec<_>>();
    let flat_ret_types = return_types.iter().flatten().cloned().unique();
    let arg_ret_types = flat_arg_types.into_iter().chain(flat_ret_types);

    let State {
        fn_trait_names,
        fns,
        fn_kinds,
        fn_names,
        fn_ids,
        fn_mod_ids,
        fn_output_kinds,
        arg_names,
        arg_types,
        ..
    } = state;

    let fn_impl_generics = fn_trait_names
        .iter()
        .map(|_| impl_generics.clone())
        .collect::<Vec<_>>();
    let fn_metadata = fn_mod_ids
        .iter()
        .zip(&fns)
        .map(|(mod_id, method)| match &meta {
            Some(ty) => quote! { #ty },
            None => quote! { #mod_id::#method },
        })
        .collect::<Vec<_>>();

    #[cfg(feature = "value")]
    let impl_name = implementation
        .trait_
        .as_ref()
        .map(|(_, n, _)| {
            quote! {
                stringify!(impl #impl_generics #n for #self_ty)
            }
        })
        .unwrap_or_else(|| {
            quote! {
                stringify!(impl #impl_generics #self_ty)
            }
        });
    #[cfg(feature = "value")]
    let value_impl = quote! {
        impl Value<NodeKind> for #impl_node_label {
            fn value() -> NodeKind {
                NodeKind::Implementation(
                    #impl_name,
                    &[#(stringify!(#subgraph)),*],
                )
            }
        }
    };
    #[cfg(not(feature = "value"))]
    let value_impl = quote! {};

    #[cfg(feature = "value")]
    let fn_value_impls = fn_mod_ids.iter().zip(&fns).zip(&fn_kinds).zip(&fn_names).zip(&fn_subgraphs).zip(&arg_names).zip(&arg_types).map(|((((((m, f), k), n), s), an), at)| {
        quote! {
            impl ::typegraph::Value<::typegraph::NodeKind> for #m::#f {
                fn value() -> ::typegraph::NodeKind {
                    ::typegraph::#k(
                        stringify!(#n),
                        &[#(stringify!(#s)),*],
                        &[#((stringify!(#an), <<#at as ::typegraph::Typegraph>::Id as ::typegraph::Unsigned>::U32)),*]
                    )
                }
            }
        }
    });
    #[cfg(not(feature = "value"))]
    let fn_value_impls = fn_mod_ids.iter().map(|_| quote! {});

    quote! {
        #implementation

        pub trait #impl_trait_name #impl_generics {
            type Nodes;
            type Edges;
            type Types;
        }
        impl #impl_generics #impl_trait_name #impl_generics for #self_ty {
            type Nodes = ::typegraph::set![
                ::typegraph::NodeOutput<#mod_id::ids::ImplNodeId, #mod_id::#impl_node_label, #impl_metadata>,
                #(<#self_ty as #fn_trait_names #fn_impl_generics>::Node),*
            ];
            type Edges = ::typegraph::merge_sets![
                ::typegraph::set![
                    ::typegraph::Edge<<#self_ty as ::typegraph::Typegraph>::Id, #mod_id::ids::ImplNodeId>,
                    #(::typegraph::Edge<#mod_id::ids::ImplNodeId, #fn_mod_ids::ids::FnNodeId>),*
                ],
                #(<#self_ty as #fn_trait_names #fn_impl_generics>::Edges),*
            ];
            type Types = ::typegraph::list![#(#arg_ret_types),*];
        }

        mod #mod_id {
            use typegraph::*;

            pub struct #impl_node_label;
            impl NodeOutputData for #impl_node_label {
                const ID: u32 = <ids::ImplNodeId as Unsigned>::U32;
                const KIND: NodeOutputKind = NodeOutputKind::Implementation;
                const NAME: &'static str = stringify!(#impl_node_label);
            }
            #value_impl

            pub mod ids {
                use typegraph::num::*;
                pub type ImplNodeId = #impl_id;
            }
        }

        #(
            pub trait #fn_trait_names #fn_impl_generics {
                type Id;
                type Node;
                type Edges;
            }
            impl #fn_impl_generics #fn_trait_names #fn_impl_generics for #self_ty {
                type Id = #fn_mod_ids::ids::FnNodeId;
                type Node = ::typegraph::NodeOutput<Self::Id, #fn_mod_ids::#fns, #fn_metadata>;
                type Edges = ::typegraph::merge_sets![
                    ::typegraph::set![#(#ret_edges),*],
                    ::typegraph::set![#(#arg_edges),*]
                ];
            }

            #fn_value_impls

            mod #fn_mod_ids {
                use ::typegraph::*;

                pub struct #fns;
                impl NodeOutputData for #fns {
                    const ID: u32 = <ids::FnNodeId as Unsigned>::U32;
                    const KIND: NodeOutputKind = #fn_output_kinds;
                    const NAME: &'static str = stringify!(#fns);
                }

                pub mod ids {
                    use ::typegraph::num::*;
                    pub type FnNodeId = #fn_ids;
                }
            }
        )*
    }
    .into()
}
