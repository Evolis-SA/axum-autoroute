//! Axum autoroute proc macros.
#![warn(missing_docs)]
#![doc(html_logo_url = "https://raw.githubusercontent.com/Evolis-SA/axum-autoroute/refs/heads/main/axum_autoroute.png")]

use args::AutorouteInput;
use macros_internal::autoroute_path_internal;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{Ident, Path, PathArguments, PathSegment, parse_macro_input};
use utils::error::{syn_bail, syn_error};

use crate::codegen::route_info::route_info_name;
use crate::utils::PathList;

#[allow(clippy::doc_markdown)]
mod args;
#[allow(clippy::doc_markdown)]
mod codegen;
#[allow(clippy::doc_markdown)]
mod macros_internal;
#[allow(clippy::doc_markdown)]
mod utils;

/// Macro to put on top of an axum handler function.
/// It will be used to define several info about the handler (method, path, allowed responses)
/// and will also extract others from the function signature.
///
/// See the [axum-autoroute-example](https://github.com/Evolis-SA/axum-autoroute/tree/main/example) crate to get some samples.
///
/// # Autoroute fields
///
/// Required fields:
/// * The method as an http [`Method`](axum::http::method::Method) constant (e.g. `GET`, `POST`, etc.).
///     * **Must be the first attribute**.
/// * `path = "..."` The path of the route with its parameters in curly braces (e.g. `"{/my/route/{id}}"`).
///     * **Must be the second attribute**.
/// * `responses=[(...), ...]` A list of responses that can be returned by this route. See below for more information and example.
///
/// Optional fields:
/// * `tags=["mytag", ...]` A list of tags for this route. They can be used to group the routes (this is done by swagger-ui for instance).
///
///
///
/// # Responses fields
///
/// Each response of the `responses` list must be enclosed by braces and can have the following fields.
///
/// Required fields:
/// * The status code as either a numeric value (e.g. `200`, `404` etc.)
///   or an http [`StatusCode`](axum::http::status::StatusCode) constant (e.g. `OK`, `NOT_FOUND`, etc.).
///     * **Must be the first attribute**.
/// * `body=...` The type returned by this response.
///     * **Must be the second attribute**.
///     * If it's a type path (e.g. `String`, `MyStructOrEnum`, etc.) it will be the type of the response body.
///     * If a tuple is provided, the last element will be considered as the type of the response body,
///       while previous types will be considered as response parts
///       (and therefore must implement the [`IntoResponseParts`](axum::response::IntoResponseParts) trait.)
///         * For example with `body=(CookieJar, String)`, the response body will be of type `String`.
///         * If you want to return a tuple in the response body, it should be enclosed in two set of braces (e.g. `((String, u32))`).
///
/// Optional fields:
/// * `serializer=...` The serializer to use on the response body type.
///   Can be `NONE` to not use any serializer, or anything that can be called
///   with a single element having the type of the response body (a closure, function etc.).
///   Axum [`Json`](axum::extract::Json) by default if `default_serializer_json` is enabled.
/// * `description="..."` A description of this reponse to add to the openapi specification.
/// * `content_type=...` The `content_type` of the response as a string (e.g. `"text/plain"`) or as a [`Mime`](mime::Mime) constant (e.g. `TEXT_PLAIN`)
/// * `headers=[...]` A set of headers returned by this response that should be documented in the openapi specification.
///   Each header is enclosed by braces and can have the following fields:
///     * The header name as an http [`HeaderName`](axum::http::header) constant (e.g. `SET_COOKIE`, `CONTENT_ENCODING`, etc.)
///         * **Required, must be the first attribute**.
///     * `description="..."` An optional description for the openapi specification.
/// * `trace=true|false` Indicates whether the response content should be traced or not if the `tracing` feature is enabled (`true` by default).
///
///
///
/// # Extractors
///
/// The autoroute macro will try to automatically detect extractors from the function signature and add them into the openapi if needed.
///
/// Here is a list of the currently detected extractors:
/// * Parts extractors:
///     * `axum::extract::Path`. Must extract a struct or enum implementing `serde::Deserialize` and `utoipa::IntoParams`.
///     * `axum::extract::Query`. Must extract a struct or enum implementing `serde::Deserialize` and `utoipa::IntoParams`.
/// * Body extractors
///   (as specified in the [axum extractors documentation](https://docs.rs/axum/latest/axum/extract/index.html#the-order-of-extractors),
///   a single body extractor can be present and must be the last one in the function parameters):
///     * `axum::extract::Json`. Must extract a struct or enum implementing `serde::Serialize` and `utoipa::ToSchema`.
///     * `axum_typed_multipart::TypedMultipart`. Must extract a struct implementing `axum_typed_multipart::TryFromMultipart` and `utoipa::ToSchema`.
///     * `axum::body::Body`. To extract the raw body.
///
/// If an unknown extractor type is used, it will by default be considered as a parts extractor (see [`FromRequestParts`](axum::extract::FromRequestParts)) and will never be traced.
/// See the [Extractor attribute](#extractor-attribute) section below for more information on how to provide information about unknown extractors.
///
/// ## Extractor variable detection
///
/// Additionaly, the macro will try to find the variable associated to each extractor (typically for the traces added if the `tracing` feature is enabled).
/// Some restrictions are thus added on how extractor parameters can be named/destructured:
/// * They should always be named.
/// * Destructuring is allowed as long as a single element is present and there is no nested destructuring pattern.
///
/// Examples of supported extractor definitions:
/// * `var: Json<MyStruct>`.
/// * `Json(var): Json<MyStruct>`.
///
/// Examples of unsupported extractor definitions:
/// * `_: Json<MyStruct>`, because the variable is not named.
/// * `Json(MyStruct{var}) : Json<MyStruct>`, because there is a nested destructuring pattern.
/// * `MyExtractor(var1, var2) : MyExtractor`, because there is a destructuring pattern with multiple variables.
///
/// ## Extractor attribute
///
/// The `#[extractor(...)]` attribute can be added on every extractor inputs of the function to provide additional information on how to handle the extractor.
/// This can be used in order to customize an extractor behavior or to help handle unknown extractor types.
///
/// Note: this attribute can be used as either `#[extractor(...)]` or `#[autoroute_extractor(...)]`.
///
/// Available fields:
/// * `trace=true|false` Indicates whether the extractor content should be traced or not if the `tracing` feature is enabled. By default, tracing is:
///     * Enabled for known extractors.
///     * Disabled for unknown extractors unless they must be added in openapi specification (see below).
///
/// Unstable fields (gated by feature `unstable_extractor_attr`):
/// * `into_params=true|false` If true indicates that the extractor should be added in the openapi specification as a parameter (path, query etc.).
///     * Incompatible with `content_type`.
/// * `content_type=...` If set indicates that the associated function input is a body extractor and that it should be included in the openapi specification. It can be a string (e.g. `"text/plain"`) or a [`Mime`](mime::Mime) constant (e.g. `TEXT_PLAIN`).
///     * If several `content_type=...` assignments are performed in a single extractor attribute, they will all be added into the openapi specification.
///     * Incompatible with `into_params`.
///
///
///
/// # Tracing
///
/// If the `tracing` feature is enabled, each time an `autoroute` function is called:
/// * There will be a trace when the function starts.
/// * There will be a trace for each extractor variable detected (using its `Debug` implementation). See above for more info.
/// * There will be a trace when the function ends.
/// * The content of the response returned will be traced (using its `Debug` implementation).
#[proc_macro_attribute]
pub fn autoroute(meta: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    autoroute_path_internal(false, meta, item)
}

/// Same as [`macro@autoroute`] but with stdout messages helping for debug
#[cfg(feature = "debugging")]
#[proc_macro_attribute]
pub fn autoroute_debug(meta: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    autoroute_path_internal(true, meta, item)
}

#[proc_macro]
/// Returns a `RouteInfo` from the name of an handler.
pub fn route_info(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut path: Path = parse_macro_input!(item);
    let path_span = path.span();
    let Some(last_segment) = path.segments.last_mut() else {
        return syn_error!(path_span, "path without a last segment")
            .into_compile_error()
            .into();
    };
    // replace the last segment by the name of the RouteInfo constant
    *last_segment = PathSegment {
        ident: Ident::new(&route_info_name(&last_segment.ident.to_string()), path_span),
        arguments: PathArguments::None,
    };
    quote! {#path}.into()
}

#[proc_macro]
/// Returns an array of `RouteInfo` from a list of handlers name.
pub fn routes_info(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let paths: PathList = parse_macro_input!(item);
    let calls = paths
        .list
        .into_iter()
        .map(|p| quote_spanned! {p.span()=> axum_autoroute::route_info!(#p)})
        .collect::<Vec<_>>();
    quote! { [ #(#calls),* ] }.into()
}

/// Returns an [`UtoipaMethodRouter`](utoipa_axum::router::UtoipaMethodRouter) from the name of an handler.
#[proc_macro]
pub fn method_router(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let path: Path = parse_macro_input!(item);
    quote_spanned! {path.span()=>
        utoipa_axum::routes!(#path)
    }
    .into()
}

/// Returns an array of [`UtoipaMethodRouter`](utoipa_axum::router::UtoipaMethodRouter) from a list of handlers name.
#[proc_macro]
pub fn method_routers(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let paths: PathList = parse_macro_input!(item);
    let calls = paths
        .list
        .into_iter()
        .map(|p| quote_spanned! {p.span()=> axum_autoroute::method_router!(#p)})
        .collect::<Vec<_>>();
    quote! { [ #(#calls),* ] }.into()
}
