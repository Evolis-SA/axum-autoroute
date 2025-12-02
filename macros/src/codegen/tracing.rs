use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{Ident, Stmt, parse_quote_spanned};

use crate::AutorouteInput;
use crate::args::extractors::AutorouteAxumExtractorType;
use crate::args::responses::AutorouteResponse;

/// Log input parameters of an autoroute function.
/// Modifies the function block of the ItemFn contained in AutorouteInput.
pub fn add_inputs_tracing(input: &mut AutorouteInput) {
    let msg = format!("'{}' triggered", input.fn_ident());

    let mut extractor_traces = Vec::new();
    for extractor in &input.axum_extractors {
        if extractor.to_trace() {
            let ty_str = if let AutorouteAxumExtractorType::Unknown { ty } = &*extractor.extractor_ty {
                ty.to_string()
            } else {
                extractor.extractor_ty.to_string()
            };

            let extracted_var = &extractor.extracted_var;
            extractor_traces.push(quote_spanned! {extractor.full_ty.span()=>
                tracing::debug!("* {}: {:?}", #ty_str, #extracted_var);
            });
        }
    }

    let tracing_stmts: Vec<Stmt> = parse_quote_spanned! {input.fn_ident().span()=>
        tracing::debug!(#msg);
        #(#extractor_traces)*
    };

    // add the tracing instructions at the start of the function
    input.itemfn.block.stmts.splice(0..0, tracing_stmts);
}

/// Log output parameters of an autoroute handler for a given response.
/// Returns the tracing instructions token stream.
pub fn output_tracing(
    input: &AutorouteInput,
    out_body_var: &Ident,
    resp: &AutorouteResponse,
) -> proc_macro2::TokenStream {
    let msg = format!(
        "'{}' finished -> {}:{}",
        input.fn_ident(),
        resp.status_code.as_code(),
        resp.status_code
    );

    let resp_trace = if resp.do_trace {
        quote_spanned! {input.fn_ident().span()=> tracing::debug!("* Response: {:?}", #out_body_var); }
    } else {
        quote! {}
    };

    quote_spanned! {input.fn_ident().span()=>
        tracing::debug!(#msg);
        #resp_trace
    }
}
