#![allow(unused_attributes)]
extern crate quote;
extern crate syn;

use std::collections::HashMap;

use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse_macro_input, parse_quote, Data, DataEnum, DataStruct, DeriveInput, Fields, Ident, Path,
    PathArguments, Type, Visibility,
};

mod argument;
mod field;
mod function;
mod generic;
mod id;
mod implementation;
mod variant;

use id::uid;

const MAX_NODE_IDS: u32 = 32768;
const NODE_DATA_LABEL: &str = "NodeData";
const HELPER_ATTRIBUTE: &str = "typegraph";
const IGNORE_ATTRIBUTE: &str = "skip";

#[derive(deluxe::ParseMetaItem, deluxe::ExtractAttributes)]
#[deluxe(attributes(typegraph))]
struct Attributes {
    #[deluxe(default, alias = impls)]
    implementations: Vec<Path>,
    meta: Option<syn::Type>,
    cluster: Option<syn::Path>,
    #[deluxe(default)]
    generic: bool,
    #[deluxe(default)]
    generics: Vec<Type>,
}

struct Kind {
    node_output_kind: syn::Path,
    node_kind: syn::Path,
    prefix: proc_macro2::TokenStream,
}

impl Kind {
    fn from_data(data: &Data, is_generic: bool) -> Self {
        let (node_output_kind, node_kind, prefix): (syn::Path, syn::Path, _) = match &data {
            Data::Struct(_) => (
                if is_generic {
                    parse_quote! { NodeOutputKind::Generic }
                } else {
                    parse_quote! { NodeOutputKind::Struct }
                },
                if is_generic {
                    parse_quote! { NodeKind::Generic }
                } else {
                    parse_quote! { NodeKind::Struct }
                },
                quote! { struct },
            ),
            Data::Enum(_) => (
                parse_quote! { NodeOutputKind::Enum },
                parse_quote! { NodeKind::Enum },
                quote! { enum },
            ),
            Data::Union(_) => (
                parse_quote! { NodeOutputKind::Union },
                parse_quote! { NodeKind::Struct },
                quote! {},
            ),
        };

        Self {
            node_output_kind,
            node_kind,
            prefix,
        }
    }
}

struct State {
    node_id: proc_macro2::TokenStream,
    node_label: Ident,
    mod_label: Ident,
    enum_variants: Vec<Ident>,
    enum_variant_names: Vec<Ident>,
    enum_variant_ids: Vec<proc_macro2::TokenStream>,
    enum_variant_tys: Vec<Vec<Type>>,
    enum_variant_trait_names: Vec<Ident>,
    enum_variant_field_edge_labels: Vec<Vec<String>>,
    enum_variant_id_labels: Vec<Ident>,
    fields: HashMap<Type, String>,
}

impl State {
    fn new(ident: &Ident) -> Self {
        let node_label = format_ident!("{}{}", NODE_DATA_LABEL, ident);

        Self {
            mod_label: format_ident!("typegraph_{}", node_label.to_string().to_lowercase()),
            node_label,
            node_id: uid(),
            enum_variants: vec![],
            enum_variant_names: vec![],
            enum_variant_ids: vec![],
            enum_variant_tys: vec![],
            enum_variant_trait_names: vec![],
            enum_variant_field_edge_labels: vec![],
            enum_variant_id_labels: vec![],
            fields: Default::default(),
        }
    }

    fn fields(&self) -> (Vec<String>, Vec<Type>) {
        let (field_names, field_tys): (Vec<_>, Vec<_>) = self
            .fields
            .iter()
            .map(|(ty, s)| (s.to_string(), ty.clone()))
            .unzip();

        (field_names, field_tys)
    }

    fn add_field(&mut self, field: field::State) {
        if let Some(s) = self.fields.get_mut(&field.ty) {
            *s = format!("{s}, {}", field.name);
        } else {
            self.fields.insert(field.ty, field.name.to_string());
        }
    }

    fn add_variant(&mut self, tv: variant::State) {
        let (field_names, field_tys) = tv.fields();
        self.enum_variants.push(tv.label);
        self.enum_variant_names.push(tv.name);
        self.enum_variant_ids.push(tv.id);
        self.enum_variant_trait_names.push(tv.trait_name);
        self.enum_variant_tys.push(field_tys);
        self.enum_variant_field_edge_labels.push(field_names);
        self.enum_variant_id_labels.push(tv.id_label);
    }

    fn try_from_data(
        data: &mut syn::Data,
        ident: &Ident,
        is_generic: bool,
    ) -> Result<Outcome<Self>, TokenStream> {
        let mut state = Self::new(ident);

        match data {
            Data::Struct(DataStruct { fields, .. }) => match fields {
                Fields::Named(fields) => {
                    if is_generic {
                        return Err(syn::Error::new_spanned(
                            fields,
                            "Only unit structs may be used for typegraph generics.",
                        )
                        .to_compile_error()
                        .into());
                    }

                    for field in &mut fields.named {
                        let name = field
                            .ident
                            .as_ref()
                            .cloned()
                            .unwrap_or_else(|| format_ident!("unknown"));

                        match field::State::try_from_field(field, name)? {
                            Outcome::Skip => {
                                continue;
                            }
                            Outcome::Connect(tf) => {
                                state.add_field(tf);
                            }
                        }
                    }
                }
                Fields::Unnamed(fields) => {
                    if is_generic {
                        return Err(syn::Error::new_spanned(
                            fields,
                            "Only unit structs may be used for typegraph generics.",
                        )
                        .to_compile_error()
                        .into());
                    }

                    for (i, field) in fields.unnamed.iter_mut().enumerate() {
                        let name = format_ident!("_{i}");

                        match field::State::try_from_field(field, name)? {
                            Outcome::Skip => {
                                continue;
                            }
                            Outcome::Connect(tf) => {
                                state.add_field(tf);
                            }
                        }
                    }
                }
                Fields::Unit => {}
            },
            Data::Enum(DataEnum { variants, .. }) => {
                if is_generic {
                    return Err(syn::Error::new_spanned(
                        variants,
                        "Only unit structs may be used for typegraph generics.",
                    )
                    .to_compile_error()
                    .into());
                }
                for variant in variants.iter_mut() {
                    match variant::State::try_from_variant(variant, &state.node_label)? {
                        Outcome::Skip => {
                            continue;
                        }
                        Outcome::Connect(tv) => {
                            state.add_variant(tv);
                        }
                    }
                }
            }
            _ => {}
        }

        Ok(Outcome::Connect(state))
    }
}

struct Implementations(Vec<Path>);
impl Implementations {
    pub fn new(attr_paths: Vec<Path>, ident: &Ident) -> Self {
        Self(
            attr_paths
                .into_iter()
                .map(|mut p| {
                    let n = p.segments.last().map(|seg| &seg.ident).cloned().unwrap();
                    let x = format_ident!("{}Impl{}", ident, n);
                    if let Some(l) = p.segments.last_mut() {
                        l.ident = x;
                        l.arguments = PathArguments::None;
                    }
                    p
                })
                .collect(),
        )
    }
}

enum Outcome<T> {
    Skip,
    Connect(T),
}

#[proc_macro_derive(Typegraph, attributes(typegraph))]
pub fn typegraph_derive(input: TokenStream) -> TokenStream {
    #[cfg(feature = "inert")]
    {
        return quote! {}.into();
    }

    let mut input: DeriveInput = parse_macro_input!(input);
    let Attributes {
        implementations,
        meta,
        cluster,
        generic,
        generics,
    } = match deluxe::extract_attributes(&mut input) {
        Ok(desc) => desc,
        Err(e) => return e.into_compile_error().into(),
    };
    let (implementations, implementation_generics): (Vec<_>, Vec<_>) = implementations
        .into_iter()
        .map(|path| {
            if let Some(last_segment) = path.segments.last() {
                if let PathArguments::AngleBracketed(args) = &last_segment.arguments {
                    return (path.clone(), quote! { #args });
                }
            }
            (path, quote! {})
        })
        .unzip();
    if !generics.is_empty() && generics.len() != input.generics.type_params().count() {
        return syn::Error::new_spanned(
            input.generics,
            "The number of typegraph generic substitutions differs from the number of generic type parameters.",
        )
        .to_compile_error()
        .into();
    }
    let mut generic_sub = generic::Substitution::new(generics, &input.generics);

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let Implementations(impl_paths) = Implementations::new(implementations, &input.ident);
    let state = match State::try_from_data(&mut input.data, &input.ident, generic) {
        Ok(Outcome::Connect(s)) => s,
        Ok(Outcome::Skip) => unreachable!(),
        Err(tt) => {
            return tt;
        }
    };

    let (mod_label, node_label) = (&state.mod_label, &state.node_label);
    let metadata: proc_macro2::TokenStream = match meta.clone() {
        Some(ty) => quote! { #ty },
        None => quote! { #mod_label::#node_label },
    };
    let var_meta: Vec<proc_macro2::TokenStream> = match meta {
        Some(ty) => vec![quote! { #ty }],
        None => state.enum_variants.iter().map(|v| quote! { #v }).collect(),
    };
    let subgraph = cluster
        .into_iter()
        .flat_map(|s| s.segments.into_iter().map(|p| p.ident))
        .collect::<Vec<_>>();
    let variant_subgraphs = state
        .enum_variants
        .iter()
        .map(|_| subgraph.clone())
        .collect::<Vec<_>>();

    let (field_edge_labels, field_tys) = state.fields();
    let field_tys = field_tys
        .into_iter()
        .map(|ty| generic_sub.substitute(ty))
        .collect::<Vec<_>>();
    let field_edge_ids_or_stub = field_tys.iter().map(|t| {
        quote! { ::typegraph::Edge<Self::Id, <#t as ::typegraph::Typegraph>::Id> }
    });

    let (variant_tys, variant_field_edge_labels) =
        (state.enum_variant_tys, state.enum_variant_field_edge_labels);
    let variant_tys = variant_tys
        .into_iter()
        .map(|tys| generic_sub.substitute_all(tys))
        .collect::<Vec<_>>();
    let variant_field_edge_ids_or_stub = variant_tys.iter().map(|v| {
        v.iter()
            .map(|t| {
                quote! { ::typegraph::Edge<Self::Id, <#t as ::typegraph::Typegraph>::Id> }
            })
            .collect::<Vec<_>>()
    });

    let State {
        node_id,
        enum_variants,
        enum_variant_trait_names,
        enum_variant_ids,
        enum_variant_names,
        enum_variant_id_labels,
        ..
    } = state;

    let Kind {
        node_output_kind,
        node_kind,
        prefix,
    } = Kind::from_data(&input.data, generic);

    let allow_tokens = if generic {
        quote! { #[allow(non_camel_case_types)] }
    } else {
        quote! {}
    };

    let ident = input.ident;
    let node_kind_tokens = if generic {
        let stripped = format_ident!("{}", ident.to_string().replace('_', ""));
        quote! {
            ::typegraph::#node_kind(
                stringify!(#stripped),
                &[#(stringify!(#subgraph)),*],
                &[#((#field_edge_labels, <<#field_tys as ::typegraph::Typegraph>::Id as ::typegraph::Unsigned>::U32)),*]
            )
        }
    } else {
        quote! {
            ::typegraph::#node_kind(
                stringify!(#prefix #ident #ty_generics),
                &[#(stringify!(#subgraph)),*],
                &[#((#field_edge_labels, <<#field_tys as ::typegraph::Typegraph>::Id as ::typegraph::Unsigned>::U32)),*]
            )
        }
    };

    #[cfg(feature = "value")]
    let enum_variant_value_impls = enum_variants.iter()
        .zip(&enum_variant_names)
        .zip(&variant_subgraphs)
        .zip(&variant_field_edge_labels)
        .zip(&variant_tys)
        .map(|((((var, name), sub), label), ty)| {
            quote! {
                impl ::typegraph::Value<::typegraph::NodeKind> for #var {
                    fn value() -> ::typegraph::NodeKind {
                        ::typegraph::NodeKind::Variant(
                            stringify!(#name),
                            &[#(stringify!(#sub)),*],
                            &[#((#label, <<#ty as ::typegraph::Typegraph>::Id as ::typegraph::Unsigned>::U32))*]
                        )
                    }
                }
        }});
    #[cfg(not(feature = "value"))]
    let enum_variant_value_impls = quote! {};

    #[cfg(feature = "value")]
    let value_impl = quote! {
        impl ::typegraph::Value<::typegraph::NodeKind> for #mod_label::#node_label {
            fn value() -> ::typegraph::NodeKind {
                #node_kind_tokens
            }
        }
    };
    #[cfg(not(feature = "value"))]
    let value_impl = quote! {};

    quote! {
        impl #impl_generics ::typegraph::Typegraph for #ident #ty_generics #where_clause {
            type Id = #mod_label::ids::NodeId;
            type Node = #mod_label::#node_label;
            type Nodes = ::typegraph::merge_sets![
                ::typegraph::set![
                    ::typegraph::NodeOutput<Self::Id, #mod_label::#node_label, #metadata>,
                    #(<#enum_variants as #enum_variant_trait_names>::Node),*
                ],
                #(<Self as #impl_paths #implementation_generics>::Nodes),*
            ];
            type Edges = ::typegraph::merge_sets![
                ::typegraph::set![#(#field_edge_ids_or_stub),*],
                ::typegraph::set![#(::typegraph::Edge<Self::Id, #mod_label::ids::#enum_variant_id_labels>),*],
                ::typegraph::merge_sets![#(<#enum_variants as #enum_variant_trait_names>::Edges),*],
                #(<Self as #impl_paths #implementation_generics>::Edges),*
            ];
            type Types = ::typegraph::merge_lists![
                ::typegraph::list![#(#field_tys),*],
                ::typegraph::merge_lists![#(<#enum_variants as #enum_variant_trait_names>::Types),*],
                #(<Self as #impl_paths #implementation_generics>::Types),*
            ];
        }

        #(
            pub trait #enum_variant_trait_names {
                type Id;
                type Node;
                type Edges;
                type Types;
            }
            pub struct #enum_variants;
            impl ::typegraph::NodeOutputData for #enum_variants {
                const ID: u32 = <<Self as #enum_variant_trait_names>::Id as ::typegraph::Unsigned>::U32;
                const KIND: ::typegraph::NodeOutputKind = ::typegraph::NodeOutputKind::Variant;
                const NAME: &'static str = stringify!(#enum_variant_names);
            }
            #enum_variant_value_impls
            impl #enum_variant_trait_names for #enum_variants {
                type Id = #mod_label::ids::#enum_variant_id_labels;
                type Node = ::typegraph::NodeOutput<Self::Id, Self, #var_meta>;
                type Edges = ::typegraph::set![#(#variant_field_edge_ids_or_stub),*];
                type Types = ::typegraph::list![#(#variant_tys),*];
            }
        )*

        mod #mod_label {
            use typegraph::*;

            #allow_tokens
            pub struct #node_label;
            impl NodeOutputData for #node_label {
                const ID: u32 = <ids::NodeId as Unsigned>::U32;
                const KIND: NodeOutputKind = #node_output_kind;
                const NAME: &'static str = stringify!(#node_label);
            }

            pub mod ids {
                use typegraph::num::*;

                pub type NodeId = #node_id;
                #(
                    pub type #enum_variant_id_labels = #enum_variant_ids; 
                )*
            }
        }

        #value_impl
    }
    .into()
}

#[proc_macro_attribute]
pub fn typegraph(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as syn::Item);
    match input {
        syn::Item::Impl(x) => implementation::impl_macro(attr, x),
        syn::Item::Enum(mut x) => {
            #[cfg(not(feature = "inert"))]
            {
                x.vis = Visibility::Public(syn::token::Pub {
                    span: x.ident.span(),
                });
            }

            let attr: proc_macro2::TokenStream = attr.into();
            let attr: syn::Attribute = parse_quote! { #[typegraph(#attr)] };
            let y = strip_attributes(syn::Item::Enum(x.clone()));
            x.attrs.push(attr);
            let x = syn::Item::Enum(x);

            let derived: proc_macro2::TokenStream =
                typegraph_derive(x.into_token_stream().into()).into();
            let tokens: proc_macro2::TokenStream = y.into_token_stream();
            quote! {
                #tokens
                #derived
            }
            .into()
        }
        syn::Item::Struct(mut x) => {
            #[cfg(not(feature = "inert"))]
            {
                x.vis = Visibility::Public(syn::token::Pub {
                    span: x.ident.span(),
                });
            }

            let attr: proc_macro2::TokenStream = attr.into();
            let attr: syn::Attribute = parse_quote! { #[typegraph(#attr)] };
            let y = strip_attributes(syn::Item::Struct(x.clone()));
            x.attrs.push(attr);
            let x = syn::Item::Struct(x);

            let derived: proc_macro2::TokenStream =
                typegraph_derive(x.into_token_stream().into()).into();
            let tokens: proc_macro2::TokenStream = y.into_token_stream();
            quote! {
                #tokens
                #derived
            }
            .into()
        }
        syn::Item::Fn(x) => syn::Item::Fn(x).into_token_stream().into(),
        item => syn::Error::new_spanned(
            item,
            "This macro can only be applied to structs, enums, and implementations.",
        )
        .to_compile_error()
        .into(),
    }
}

fn strip_attributes(item: syn::Item) -> proc_macro2::TokenStream {
    match item {
        syn::Item::Struct(mut x) => {
            x.fields.iter_mut().for_each(|f| {
                f.attrs = f
                    .attrs
                    .iter()
                    .filter(|a| !a.path().is_ident(HELPER_ATTRIBUTE))
                    .cloned()
                    .collect()
            });
            syn::Item::Struct(x).into_token_stream()
        }

        syn::Item::Enum(mut x) => {
            x.variants.iter_mut().for_each(|variant| {
                variant.attrs = variant
                    .attrs
                    .iter()
                    .filter(|a| !a.path().is_ident(HELPER_ATTRIBUTE))
                    .cloned()
                    .collect();

                match &mut variant.fields {
                    Fields::Named(f) => {
                        f.named.iter_mut().for_each(|f| {
                            f.attrs = f
                                .attrs
                                .iter()
                                .filter(|a| !a.path().is_ident(HELPER_ATTRIBUTE))
                                .cloned()
                                .collect()
                        });
                    }
                    Fields::Unnamed(f) => {
                        f.unnamed.iter_mut().for_each(|f| {
                            f.attrs = f
                                .attrs
                                .iter()
                                .filter(|a| !a.path().is_ident(HELPER_ATTRIBUTE))
                                .cloned()
                                .collect()
                        });
                    }
                    _ => {}
                }
            });

            syn::Item::Enum(x).into_token_stream()
        }

        x => x.into_token_stream(),
    }
}
