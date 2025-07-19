use proc_macro::TokenStream;
use quote::format_ident;
use syn::{Ident, PathArguments, Type};

use crate::{Outcome, NODE_DATA_LABEL};

#[derive(deluxe::ParseMetaItem, deluxe::ExtractAttributes, Default)]
#[deluxe(attributes(typegraph))]
struct Attributes {
    #[deluxe(default)]
    force: Option<Type>,
    #[deluxe(default)]
    skip: bool,
}

pub struct State {
    pub name: Ident,
    pub ty: Type,
}

impl State {
    pub fn try_from_field(
        field: &mut syn::Field,
        name: syn::Ident,
    ) -> Result<Outcome<Self>, TokenStream> {
        let Attributes { force, skip } = deluxe::extract_attributes(field).unwrap_or_default();
        if skip {
            return Ok(Outcome::Skip);
        }

        let mut tp =
            match field.ty.clone() {
                Type::Path(tp) => tp,
                Type::Reference(tr) => {
                    let Type::Path(tp) = *tr.elem else {
                        return Err(syn::Error::new_spanned(field, "Unsupported type reference")
                            .to_compile_error()
                            .into());
                    };
                    tp
                }
                _ => return Err(syn::Error::new_spanned(
                    field,
                    "Typegraph fields must either be paths, or have a path explicitly provided.",
                )
                .to_compile_error()
                .into()),
            };
        let edge_label = format_ident!(
            "{}{}",
            NODE_DATA_LABEL,
            tp.path.segments.last().map(|seg| &seg.ident).unwrap() // TODO
        );
        if let Some(l) = tp.path.segments.last_mut() {
            l.ident = edge_label;
            l.arguments = PathArguments::None;
        }

        Ok(Outcome::Connect(Self {
            ty: force.unwrap_or(field.ty.clone()),
            name: name.clone(),
        }))
    }
}
