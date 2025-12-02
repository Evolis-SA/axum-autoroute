use convert_case::{Case, Casing};
use quote::{quote, quote_spanned};
use syn::Ident;

use crate::args::AutorouteInput;

/// Create declaration of dummy structs that will check that some types implement a specific trait.
/// As the code is generated near the route handler declaration, the creation of a new struct is needed as we have no guarantee that the target type is declared in the handler's crate.
pub(crate) fn declare_trait_checkers(input: &AutorouteInput) -> proc_macro2::TokenStream {
    let mut trait_checkers = Vec::new();

    // check that extractors implement either FromRequest or FromRequestParts
    for (i, extractor) in input.axum_extractors.iter().enumerate() {
        let full_type = extractor.full_ty.clone();
        let trait_checker = if extractor.is_parts_extractor() {
            let struct_name = Ident::new(
                &format!(
                    "_{}TraitChecker{i}FromRequestParts",
                    input.fn_ident().to_string().to_case(Case::Pascal)
                ),
                extractor.extractor_ty.span(),
            );
            // PhantomData is needed due to the generic S type of FromRequestParts
            quote_spanned! {extractor.extractor_ty.span()=> struct #struct_name<S>(std::marker::PhantomData<S>) where #full_type : axum::extract::FromRequestParts<S>;}
        } else {
            let struct_name = Ident::new(
                &format!(
                    "_{}TraitChecker{i}FromRequest",
                    input.fn_ident().to_string().to_case(Case::Pascal)
                ),
                extractor.extractor_ty.span(),
            );
            // PhantomData is needed due to the generic S type of FromRequest
            quote_spanned! {extractor.extractor_ty.span()=> struct #struct_name<S>(std::marker::PhantomData<S>) where #full_type : axum::extract::FromRequest<S>;}
        };
        trait_checkers.push(trait_checker);
    }

    quote! {#(#trait_checkers)*}
}
