use std::ops::{Deref, DerefMut};

use error::syn_error;
use quote::quote;
use syn::parse::discouraged::Speculative;
use syn::parse::{self, Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Ident, Path, Token};

pub(crate) mod error;
pub(crate) mod http;
pub(crate) mod mime;
pub(crate) mod spanned;

pub(crate) fn parse_named_ident(input: ParseStream, name: &str) -> syn::Result<Ident> {
    // fork to avoid advancing the input if not found
    let fork = input.fork();
    let ident: Ident = fork
        .parse()
        .map_err(|e| syn_error!(e.span(), "expected ident `{name}`: {e}"))?;
    if ident == name {
        // found, advance input to the same cursor as the fork
        input.advance_to(&fork);
        Ok(ident)
    } else {
        Err(syn_error!(ident.span(), "expected ident `{name}`"))
    }
}

#[expect(unused)]
pub(crate) fn path_last_ident(path: &Path) -> syn::Result<Ident> {
    Ok(path
        .segments
        .last()
        .ok_or(syn::Error::new(path.span(), "unable to extract last segment from span"))?
        .ident
        .clone())
}

pub(crate) fn path_as_str(path: &Path) -> String {
    quote! {#path}.to_string()
}

macro_rules! printdbg {
    ($debug:ident, $($tt:tt),+) => {
        #[cfg(feature = "debugging")]
        if $debug { println!($($tt),+); }
    };
}
pub(crate) use printdbg;

pub(crate) struct PathList {
    pub(crate) list: Punctuated<Path, Token![,]>,
}

impl Deref for PathList {
    type Target = Punctuated<Path, Token![,]>;

    fn deref(&self) -> &Self::Target {
        &self.list
    }
}

impl DerefMut for PathList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.list
    }
}

impl Parse for PathList {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            list: Punctuated::parse_terminated(input)?,
        })
    }
}
