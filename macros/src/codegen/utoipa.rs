use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::Ident;
use syn::spanned::Spanned;

use crate::{AutorouteInput, syn_bail};

pub(crate) fn declare_utoipa_path_meta(input: &AutorouteInput) -> syn::Result<proc_macro2::TokenStream> {
    let method = input.method();
    let method_lower = Ident::new(&method.to_string().to_lowercase(), method.span());
    let path = input.path().value();

    let tags = if input.meta.tags.is_empty() {
        None
    } else {
        let tags = input.meta.tags.clone();
        Some(quote! {tags = [ #(#tags),* ], })
    };

    let mut request_body = None;
    let mut params = Vec::new();
    for extractor in &*input.axum_extractors {
        let extracted_ty = extractor.extracted_ty.clone();
        if extractor.is_parts_extractor() {
            if extractor.to_add_in_params() {
                params.push(extracted_ty);
            }
        } else {
            let content_types = extractor.content_types()?;
            let openapi_content = extractor.openapi_content()?;
            set_request_body(
                &mut request_body,
                quote_spanned! {extractor.extractor_ty.span()=> request_body(content(
                    #( (#openapi_content = #content_types), )*
                )), },
            )?;
        }
    }

    let mut responses = Vec::new();
    for resp in &*input.meta.responses {
        let status_code = resp.status_code;
        let status_code_ident = Ident::new(&status_code.to_string(), resp.status_code.span());
        let body_type = resp.body.clone();
        let description = resp
            .description
            .clone()
            .map(|desc| quote_spanned! {desc.span()=> description=#desc, });

        let content_type = resp.content_type.clone().map(|ct| {
            let ct_str = ct.to_string();
            quote_spanned! {ct.span()=> content_type=#ct_str, }
        });

        let headers = if resp.headers.is_empty() {
            None
        } else {
            let headers = resp.headers.iter().map(|header| {
                let header_name = header.header_name.as_str();
                let description = header
                    .description
                    .as_ref()
                    .map(|desc| quote_spanned! {desc.span()=> description=#desc});
                quote_spanned! {header.span=> (#header_name, #description)}
            });
            Some(quote! {headers(#(#headers),*)})
        };

        responses.push(quote_spanned! {resp.span=> (
            status=#status_code_ident,
            body=#body_type,
            #content_type
            #description
            #headers
        )});
    }

    Ok(quote! {
        #[utoipa::path(
            #method_lower,
            path = #path,
            #tags
            #request_body
            responses(#(#responses),*),
            params(#(#params),*),
        )]
    })
}

fn set_request_body(target: &mut Option<TokenStream>, value: TokenStream) -> syn::Result<()> {
    if target.is_some() {
        syn_bail!(value.span(), "multiple extractors consuming the body are defined");
    }
    *target = Some(value);
    Ok(())
}
