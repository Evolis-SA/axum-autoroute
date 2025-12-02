use convert_case::{Case, Casing};
use quote::quote_spanned;
use syn::Ident;

use crate::args::AutorouteInput;

pub(crate) fn declare_route_info(input: &AutorouteInput) -> proc_macro2::TokenStream {
    let method = input.method();
    let method_ident = Ident::new(&method.to_string(), method.span());
    let method = quote_spanned! {method.span()=> axum::http::Method::#method_ident};
    let path = input.path();
    let route_info = route_info_ident(input);
    let vis = input.itemfn.vis.clone();

    quote_spanned! {path.span()=>
        #[allow(unused)]
        #vis const #route_info: axum_autoroute::RouteInfo = axum_autoroute::RouteInfo::new(#method, #path);
    }
}

/// Name of the RouteInfo constant as String
pub(crate) fn route_info_name(handler_name: &str) -> String {
    format!("{}_ROUTE_INFO", handler_name.to_case(Case::Constant))
}

/// Name of the RouteInfo constant as Ident
pub(crate) fn route_info_ident(input: &AutorouteInput) -> Ident {
    let fn_ident = input.fn_ident();
    Ident::new(&route_info_name(&fn_ident.to_string()), fn_ident.span())
}
