use std::str::FromStr;

use extractors::AutorouteAxumExtractor;
use responses::AutorouteResponse;
use strum::IntoEnumIterator;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, ItemFn, LitStr, Token, bracketed, parse2};

use crate::syn_bail;
use crate::utils::error::syn_error;
use crate::utils::http::HttpMethod;
use crate::utils::parse_named_ident;
use crate::utils::spanned::SpannedValue;

pub(crate) mod extractor_attr;
pub(crate) mod extractors;
pub(crate) mod responses;

/// Enum listing the different non-positional parameters of the `autoroute` macro.
#[derive(Debug, Clone, Copy, strum::Display, strum::EnumString, strum::EnumIter)]
#[strum(serialize_all = "snake_case")]
pub(crate) enum AutorouteMetaKey {
    Responses,
    Tags,
}

/// Struct holding data extracted from the `autoroute` macro arguments.
pub(crate) struct AutorouteMeta {
    /// The HTTP method to use.
    pub(crate) method: SpannedValue<HttpMethod>,
    /// The path of the route.
    pub(crate) path: LitStr,
    /// The list of possible responses returned by the route.
    pub(crate) responses: SpannedValue<Vec<AutorouteResponse>>,
    /// The tags of the route.
    /// Used in openapi documentation and by swagger-ui to group routes.
    pub(crate) tags: Vec<LitStr>,
}

impl std::fmt::Debug for AutorouteMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            method,
            path,
            responses,
            tags,
        } = self;
        f.debug_struct("AutorouteResponse")
            .field("method", method)
            .field("path", &path.value())
            .field("tags", &tags.iter().map(LitStr::value).collect::<Vec<_>>())
            .field("responses", responses)
            .finish_non_exhaustive()
    }
}

impl Parse for AutorouteMeta {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key_error = |span| {
            syn::Error::new(
                span,
                format!(
                    "expected one of: {}",
                    AutorouteMetaKey::iter()
                        .map(|key| key.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            )
        };

        let method = input.parse()?;
        input.parse::<Token![,]>()?;

        parse_named_ident(input, "path")?;
        input.parse::<Token![=]>()?;
        let path = input.parse()?;

        // parse unordered args
        let mut responses = None;
        let mut tags = None;
        while !input.is_empty() {
            // allow trailing comma
            input.parse::<Token![,]>()?;
            if input.is_empty() {
                break;
            }

            let ident: Ident = input.parse().map_err(|e| key_error(e.span()))?;
            let key = AutorouteMetaKey::from_str(&ident.to_string()).map_err(|_| key_error(ident.span()))?;
            input.parse::<Token![=]>()?;
            match key {
                AutorouteMetaKey::Responses => {
                    if responses.is_some() {
                        syn_bail!(ident.span(), "{} already defined", key.to_string());
                    }
                    let content;
                    let brackets = bracketed!(content in input);
                    let punctuated = content.parse_terminated(AutorouteResponse::parse, Token![,])?;
                    if punctuated.is_empty() {
                        syn_bail!(ident.span(), "at least one response is required");
                    }
                    responses = Some(SpannedValue::new(
                        punctuated.into_iter().collect(),
                        brackets.span.join(),
                    ));
                }
                AutorouteMetaKey::Tags => {
                    if tags.is_some() {
                        syn_bail!(ident.span(), "{} already defined", key.to_string());
                    }
                    let content;
                    bracketed!(content in input);
                    let punctuated = content.parse_terminated(<LitStr as Parse>::parse, Token![,])?;
                    tags = Some(punctuated.into_iter().collect());
                }
            }
        }

        Ok(AutorouteMeta {
            method,
            path,
            responses: responses.ok_or(syn_error!(input.span(), "no {} defined", AutorouteMetaKey::Responses))?,
            tags: tags.unwrap_or_default(),
        })
    }
}

/// Data extracted from the `autoroute` macro call.
pub(crate) struct AutorouteInput {
    /// Data extracted from the macro parameters.
    pub(crate) meta: AutorouteMeta,
    /// The list of axum extractors detected in the target fn input parameters.
    pub(crate) axum_extractors: Vec<AutorouteAxumExtractor>,
    /// The target function item.
    pub(crate) itemfn: ItemFn,
}

impl std::fmt::Debug for AutorouteInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AutorouteInput")
            .field("meta", &self.meta)
            .field("axum_extractors", &self.axum_extractors)
            .finish_non_exhaustive()
    }
}

impl AutorouteInput {
    pub(crate) fn build(meta_args: proc_macro2::TokenStream, item: proc_macro2::TokenStream) -> syn::Result<Self> {
        let itemfn: ItemFn = parse2(item)?;
        let meta: AutorouteMeta = parse2(meta_args)?;
        let axum_extractors = AutorouteAxumExtractor::parse_many(&itemfn)?;

        Ok(Self {
            meta,
            axum_extractors,
            itemfn,
        })
    }

    pub(crate) fn fn_ident(&self) -> Ident {
        self.itemfn.sig.ident.clone()
    }

    pub(crate) fn method(&self) -> SpannedValue<HttpMethod> {
        self.meta.method
    }

    pub(crate) fn path(&self) -> LitStr {
        self.meta.path.clone()
    }
}
