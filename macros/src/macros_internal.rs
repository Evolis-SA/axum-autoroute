use quote::quote;
use syn::spanned::Spanned;
use syn::{ReturnType, Type, parse_quote_spanned};

use crate::args::AutorouteInput;
use crate::args::extractor_attr::ExtractorAttr;
use crate::codegen::responses::{declare_responses_enum, responses_enum_ident, responses_enum_name};
use crate::codegen::route_info::declare_route_info;
use crate::codegen::trait_checkers::declare_trait_checkers;
use crate::codegen::utoipa::declare_utoipa_path_meta;
use crate::syn_error;
use crate::utils::error::syn_bail;
use crate::utils::printdbg;

pub(crate) fn autoroute_path_internal(
    debug: bool,
    meta: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    match autoroute_path_internal2(debug, meta.into(), item.into()) {
        Ok(token_stream) => token_stream,
        Err(compile_err) => {
            syn_error!(compile_err.span(), "autoroute macro failed: {compile_err}").into_compile_error()
        }
    }
    .into()
}

fn autoroute_path_internal2(
    #[cfg_attr(not(feature = "debugging"), expect(unused))] debug: bool,
    meta: proc_macro2::TokenStream,
    item: proc_macro2::TokenStream,
) -> syn::Result<proc_macro2::TokenStream> {
    printdbg!(debug, "### #[autoroute_path] start");
    let mut input = AutorouteInput::build(meta, item)?;
    printdbg!(debug, "--- autoroute args ---\n{input:#?}");

    let responses_enum = declare_responses_enum(&input)?;
    printdbg!(debug, "--- responses_enum ---\n{responses_enum}");

    let utoipa_path_meta = declare_utoipa_path_meta(&input)?;
    printdbg!(debug, "--- utoipa_path_meta ---\n{utoipa_path_meta}");

    let trait_checkers = declare_trait_checkers(&input);
    printdbg!(debug, "--- trait_checkers ---\n{trait_checkers}");

    let route_info = declare_route_info(&input);
    printdbg!(debug, "--- route_info ---\n{route_info}");

    set_func_return_type(&mut input)?;
    ExtractorAttr::remove_extractor_attrs(&mut input);

    #[cfg(feature = "tracing")]
    crate::codegen::tracing::add_inputs_tracing(&mut input);

    let func = input.itemfn;
    let quoted = quote! {
        #utoipa_path_meta
        #func

        #responses_enum

        #trait_checkers

        #route_info
    };
    printdbg!(debug, "### #[autoroute_path] end");
    Ok(quoted)
}

fn set_func_return_type(input: &mut AutorouteInput) -> syn::Result<()> {
    check_func_return_type(input)?;
    let span = input.itemfn.sig.output.span();
    let ident = responses_enum_ident(input);
    input.itemfn.sig.output = parse_quote_spanned! {span=> -> #ident};
    Ok(())
}

fn check_func_return_type(input: &AutorouteInput) -> syn::Result<()> {
    let expected_name = responses_enum_name(input);
    let func_return = &input.itemfn.sig.output;
    let mut err_span = func_return.span();
    if let ReturnType::Type(_, box_type) = func_return {
        err_span = box_type.span();
        if let Type::Path(path) = &**box_type
            && let Some(ident) = path.path.get_ident()
            && *ident == expected_name
        {
            return Ok(());
        }
    }
    syn_bail!(err_span, "expecting return type `{expected_name}`")
}
