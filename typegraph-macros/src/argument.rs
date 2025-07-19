use syn::{FnArg, Ident, ImplItemFn, Pat, Type};

#[derive(deluxe::ParseMetaItem, deluxe::ExtractAttributes, Default)]
#[deluxe(attributes(typegraph))]
struct Attributes {
    #[deluxe(default)]
    force: Option<Type>,
    #[deluxe(default)]
    skip: bool,
}

pub fn extract_impl_fn(item: &mut ImplItemFn) -> Vec<(Ident, Type)> {
    let mut calls = vec![];

    for input in item.sig.inputs.iter_mut() {
        match input {
            FnArg::Receiver(_receiver) => {}
            FnArg::Typed(pat_type) => {
                let Attributes { force, skip } =
                    deluxe::extract_attributes(pat_type).unwrap_or_default();
                if skip {
                    continue;
                }

                let Pat::Ident(p) = &*pat_type.pat else {
                    continue;
                };
                let Type::Path(ty) = &*pat_type.ty else {
                    continue;
                };
                calls.push((
                    p.ident.clone(),
                    force.unwrap_or_else(|| Type::Path(ty.clone())),
                ));
            }
        }
    }

    calls
}
