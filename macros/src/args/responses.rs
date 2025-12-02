use std::str::FromStr;

use axum::http::HeaderName;
use mime::Mime;
use proc_macro2::Span;
use quote::quote;
use strum::IntoEnumIterator;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{Ident, LitBool, LitStr, Token, Type, TypePath, bracketed, parenthesized};

use crate::syn_bail;
use crate::utils::http::{HttpStatusCode, parse_header_name};
use crate::utils::mime::parse_mime;
use crate::utils::parse_named_ident;
use crate::utils::spanned::SpannedValue;

/// If the type is a tuple, split it so that only the last one will be the response body content.
/// The rest should be response parts.
/// The first element returned is the body type, the second is the list of parts type.
fn split_into_body_and_parts(ty: Type) -> syn::Result<(Type, Vec<Type>)> {
    let span = ty.span();
    if let Type::Tuple(tuple) = ty {
        // last type is the body type
        // the preceding ones are the parts
        let types: Vec<Type> = tuple.elems.iter().cloned().collect();
        if let Some((body, parts)) = types.split_last() {
            Ok((body.clone(), parts.to_vec()))
        } else {
            syn_bail!(span, "unexpected empty tuple type");
        }
    } else if let Type::Paren(paren) = ty {
        // unbox the inner type
        Ok((*paren.elem, Vec::new()))
    } else {
        // no parts to extract
        Ok((ty, Vec::new()))
    }
}

/// Struct holding the data for a response declaration in the `autoroute` macro parameters.
pub(crate) struct AutorouteResponse {
    /// The status code returned.
    pub(crate) status_code: SpannedValue<HttpStatusCode>,
    /// The type of the body returned.
    pub(crate) body: Type,
    /// The list of response parts (headers, cookies etc.) returned.
    /// Can be empty if not parts are returned.
    pub(crate) parts: Vec<Type>,
    /// An optional content mime type override.
    pub(crate) content_type: Option<SpannedValue<Mime>>,
    /// The serializer called on the body type (json by default, which means that the response body returned will be `Json<MyBodyType>`).
    pub(crate) serializer: AutorouteResponseSerializer,
    /// A list of headers returned to document in the openapi documentation.
    pub(crate) headers: Vec<AutorouteResponseHeader>,
    /// The optional description of the response.
    pub(crate) description: Option<LitStr>,
    /// Indicates whether this response should be traced or not.
    pub(crate) do_trace: bool,
    pub(crate) span: Span,
}

impl std::fmt::Debug for AutorouteResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            status_code,
            body,
            parts,
            content_type,
            serializer,
            headers,
            description,
            do_trace,
            span: _,
        } = self;
        f.debug_struct("AutorouteResponse")
            .field("status_code", status_code)
            .field("body", &quote! {#body}.to_string())
            .field("parts", &quote! {#(#parts),*}.to_string())
            .field("content_type", content_type)
            .field("serializer", serializer)
            .field("headers", headers)
            .field("description", &description.as_ref().map(LitStr::value))
            .field("do_trace", do_trace)
            .finish_non_exhaustive()
    }
}

/// Enum listing the different non-positional parameters of the responses.
#[derive(Debug, Clone, Copy, strum::Display, strum::EnumString, strum::EnumIter)]
#[strum(serialize_all = "snake_case")]
pub(crate) enum AutorouteResponseKey {
    ContentType,
    Serializer,
    Headers,
    Description,
    Trace,
}

impl Parse for AutorouteResponse {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key_error = |span| {
            syn::Error::new(
                span,
                format!(
                    "expected one of: {}",
                    AutorouteResponseKey::iter()
                        .map(|key| key.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            )
        };

        let content;
        let parentheses = parenthesized!(content in input);

        let status_code: SpannedValue<HttpStatusCode> = content.parse()?;
        content.parse::<Token![,]>()?;

        parse_named_ident(&content, "body")?;
        content.parse::<Token![=]>()?;
        let return_type = content.parse()?;
        let (body, parts) = split_into_body_and_parts(return_type)?;

        // optional fields
        let mut headers = Vec::new();
        let mut content_type = None;
        let mut serializer = AutorouteResponseSerializer::Default;
        let mut description = None;
        let mut do_trace = true;
        while !content.is_empty() {
            // allow trailing comma
            content.parse::<Token![,]>()?;
            if content.is_empty() {
                break;
            }

            let ident: Ident = content.parse().map_err(|e| key_error(e.span()))?;
            let key = AutorouteResponseKey::from_str(&ident.to_string()).map_err(|_| key_error(ident.span()))?;
            content.parse::<Token![=]>()?;
            match key {
                AutorouteResponseKey::ContentType => {
                    content_type = Some(parse_mime(&content)?);
                }
                AutorouteResponseKey::Serializer => {
                    serializer = content.parse()?;
                }
                AutorouteResponseKey::Headers => {
                    let headers_content;
                    bracketed!(headers_content in content);
                    let punctuated = headers_content.parse_terminated(AutorouteResponseHeader::parse, Token![,])?;
                    headers = punctuated.into_iter().collect();
                }
                AutorouteResponseKey::Description => {
                    description = Some(content.parse()?);
                }
                AutorouteResponseKey::Trace => {
                    do_trace = content.parse::<LitBool>()?.value;
                }
            }
        }

        Ok(Self {
            status_code,
            body,
            parts,
            content_type,
            serializer,
            headers,
            description,
            do_trace,
            span: parentheses.span.join(),
        })
    }
}

/// The serializer called on the response body type.
#[derive(Clone)]
pub(crate) enum AutorouteResponseSerializer {
    /// Json will be used by default, which means that the response body returned will be `Json<MyBodyType>`.
    Default,
    /// No serializer, the reponse body returned will be `MyBodyType`.
    None,
    /// Calls a custom serializer provided in the response declaration parameters.
    Path { path: TypePath },
}

impl std::fmt::Debug for AutorouteResponseSerializer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Default => write!(f, "Default"),
            Self::None => write!(f, "None"),
            Self::Path { path } => f
                .debug_struct("Path")
                .field("path", &quote! {#path}.to_string())
                .finish(),
        }
    }
}

impl Parse for AutorouteResponseSerializer {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if parse_named_ident(input, "NONE").is_ok() {
            Ok(Self::None)
        } else if let Ok(path) = input.parse() {
            Ok(Self::Path { path })
        } else {
            syn_bail!(
                input.span(),
                "serializer should be either `None` or a path to a serializing type (like `axum::Json`), function or closure"
            )
        }
    }
}

/// A struct describing the parameters of a header that can be provided in a response description.
/// This is only used to provide additional information in the openapi specification.
#[derive(Clone)]
pub(crate) struct AutorouteResponseHeader {
    /// The header name.
    pub(crate) header_name: SpannedValue<HeaderName>,
    /// The associated description.
    pub(crate) description: Option<LitStr>,
    pub(crate) span: Span,
}

impl std::fmt::Debug for AutorouteResponseHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AutorouteResponseHeader")
            .field("header_type", &self.header_name)
            .field("description", &self.description.as_ref().map(LitStr::value))
            .finish_non_exhaustive()
    }
}

impl Parse for AutorouteResponseHeader {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let parentheses = parenthesized!(content in input);

        let header_name = parse_header_name(&content)?;
        if !content.is_empty() {
            content.parse::<Token![,]>()?;
        }

        let mut description = None;
        if !content.is_empty() {
            parse_named_ident(&content, "description")?;
            content.parse::<Token![=]>()?;
            description = Some(content.parse()?);
        }
        if !content.is_empty() {
            content.parse::<Token![,]>()?;
        }

        Ok(Self {
            header_name,
            description,
            span: parentheses.span.join(),
        })
    }
}
