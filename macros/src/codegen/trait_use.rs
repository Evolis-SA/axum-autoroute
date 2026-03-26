use std::ops::Deref;

use quote::quote_spanned;
use syn::{Stmt, parse_quote_spanned};

use crate::{AutorouteInput, codegen::responses::response_into_status_trait_name};


/// Add use of Into... traits at the beginning of each autoroute handler
pub fn add_use_traits(input: &mut AutorouteInput) {
    let mut use_traits = Vec::new();
    for response in input.meta.responses.deref() {

        let trait_name = response_into_status_trait_name(response);
        use_traits.push(quote_spanned! {response.status_code.span()=>
            use axum_autoroute::status_trait::#trait_name;
        });
    }

    let use_traits_stmts: Vec<Stmt> = parse_quote_spanned! {input.meta.responses.span()=>
        #(#use_traits)*
    };

    // add the use instructions at the start of the function
    input.itemfn.block.stmts.splice(0..0, use_traits_stmts);
}