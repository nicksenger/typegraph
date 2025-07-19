use std::collections::HashMap;

use proc_macro::TokenStream;
use quote::format_ident;
use syn::{Fields, Ident, Type};

use crate::{field, uid, Outcome, IGNORE_ATTRIBUTE};

pub struct State {
    pub id: proc_macro2::TokenStream,
    pub name: Ident,
    pub trait_name: Ident,
    pub label: Ident,
    pub fields: HashMap<Type, String>,
    pub id_label: Ident,
}

impl State {
    fn new(name: &Ident, node_label: &Ident) -> Self {
        let id = uid();
        let label = format_ident!("{}Variant{}", node_label, name);

        Self {
            id_label: format_ident!("{}Id", label),
            name: name.clone(),
            trait_name: format_ident!("{}Node", label),
            id,
            label,
            fields: Default::default(),
        }
    }

    pub fn fields(&self) -> (Vec<String>, Vec<Type>) {
        self.fields
            .iter()
            .map(|(ty, s)| (s.to_string(), ty.clone()))
            .unzip()
    }

    fn add_field(&mut self, field: field::State) {
        if let Some(s) = self.fields.get_mut(&field.ty) {
            *s = format!("{s}, {}", field.name);
        } else {
            self.fields.insert(field.ty, field.name.to_string());
        }
    }

    pub fn try_from_variant(
        variant: &mut syn::Variant,
        node_label: &Ident,
    ) -> Result<Outcome<Self>, TokenStream> {
        for attr in &variant.attrs {
            if attr
                .parse_args::<syn::Path>()
                .map(|i| i.is_ident(IGNORE_ATTRIBUTE))
                .unwrap_or_default()
            {
                return Ok(Outcome::Skip);
            }
        }

        let mut tv = Self::new(&variant.ident, node_label);

        match &mut variant.fields {
            Fields::Named(fields) => {
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
                            tv.add_field(tf);
                        }
                    }
                }
            }
            Fields::Unnamed(fields) => {
                for (i, field) in fields.unnamed.iter_mut().enumerate() {
                    let name = format_ident!("_{i}");

                    match field::State::try_from_field(field, name)? {
                        Outcome::Skip => {
                            continue;
                        }
                        Outcome::Connect(tf) => {
                            tv.add_field(tf);
                        }
                    }
                }
            }
            Fields::Unit => {}
        }

        Ok(Outcome::Connect(tv))
    }
}
