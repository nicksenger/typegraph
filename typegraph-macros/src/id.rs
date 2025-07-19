use proc_macro::{Punct, Spacing, TokenStream, TokenTree};
use quote::quote;
use rand::RngCore;
use std::hash::{DefaultHasher, Hasher};

use crate::MAX_NODE_IDS;

pub fn uid() -> proc_macro2::TokenStream {
    new(quote! {}.into()).into()
}

#[allow(unused)]
fn prefixed_ident(prefix: &TokenStream, id: &str) -> impl Iterator<Item = TokenTree> {
    prefix.clone().into_iter().chain(vec![
        Punct::new(':', Spacing::Joint).into(),
        Punct::new(':', Spacing::Alone).into(),
        proc_macro::TokenTree::from(proc_macro::Ident::new(id, proc_macro::Span::call_site())),
    ])
}

fn unprefixed_ident(id: &str) -> impl Iterator<Item = TokenTree> {
    [proc_macro::TokenTree::from(proc_macro::Ident::new(
        id,
        proc_macro::Span::call_site(),
    ))]
    .into_iter()
}

enum TypenumUint {
    Lsb(Box<TypenumUint>, bool),
    Term,
}

impl From<u32> for TypenumUint {
    fn from(x: u32) -> Self {
        if x == 0 {
            Self::Term
        } else {
            Self::Lsb(Box::new(Self::from(x >> 1)), (x & 1) != 0)
        }
    }
}

impl TypenumUint {
    fn write_ts(&self, ts: &mut TokenStream) {
        match self {
            Self::Term => ts.extend(unprefixed_ident("UTerm")),
            Self::Lsb(high, bit) => {
                ts.extend(unprefixed_ident("UInt"));
                ts.extend(std::iter::once::<TokenTree>(
                    Punct::new('<', Spacing::Alone).into(),
                ));
                high.write_ts(ts);
                ts.extend(std::iter::once::<TokenTree>(
                    Punct::new(',', Spacing::Alone).into(),
                ));
                ts.extend(unprefixed_ident(if *bit { "B1" } else { "B0" }));
                ts.extend(std::iter::once::<TokenTree>(
                    Punct::new('>', Spacing::Alone).into(),
                ));
            }
        }
    }

    #[allow(unused)]
    fn write_prefixed_ts(&self, prefix: &TokenStream, ts: &mut TokenStream) {
        match self {
            Self::Term => ts.extend(prefixed_ident(prefix, "UTerm")),
            Self::Lsb(high, bit) => {
                ts.extend(prefixed_ident(prefix, "UInt"));
                ts.extend(std::iter::once::<TokenTree>(
                    Punct::new('<', Spacing::Alone).into(),
                ));
                high.write_prefixed_ts(prefix, ts);
                ts.extend(std::iter::once::<TokenTree>(
                    Punct::new(',', Spacing::Alone).into(),
                ));
                ts.extend(prefixed_ident(prefix, if *bit { "B1" } else { "B0" }));
                ts.extend(std::iter::once::<TokenTree>(
                    Punct::new('>', Spacing::Alone).into(),
                ));
            }
        }
    }
}

fn u32_to_tokenstream(n: u32, _prefix: TokenStream) -> TokenStream {
    let mut result = TokenStream::new();
    TypenumUint::from(n % MAX_NODE_IDS).write_ts(&mut result);
    result
}

fn split_off_prefix(args: TokenStream) -> (TokenStream, TokenStream) {
    let mut args = args.into_iter();
    let local = (&mut args)
        .take_while(|tt| !matches!(tt, TokenTree::Punct(ref p) if p.as_char() == '|'))
        .collect();
    let mut prefix: TokenStream = args.collect();
    if prefix.is_empty() {
        let x: Vec<TokenTree> = vec![
            Punct::new(':', Spacing::Joint).into(),
            Punct::new(':', Spacing::Alone).into(),
            proc_macro::TokenTree::from(proc_macro::Ident::new(
                "typegraph",
                proc_macro::Span::call_site(),
            )),
        ];
        prefix = x.into_iter().collect();
    }
    (local, prefix)
}

pub fn new(args: TokenStream) -> TokenStream {
    let (args, prefix) = split_off_prefix(args);
    assert!(args.is_empty(), "new IDs take no arguments");
    u32_to_tokenstream(rand::rng().next_u32(), prefix)
}

pub fn hashed(args: TokenStream) -> TokenStream {
    let (args, prefix) = split_off_prefix(args);
    use std::hash::Hash;
    let s = args.to_string();
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    let n = hasher.finish();
    u32_to_tokenstream((n % u32::MAX as u64) as u32, prefix)
}
