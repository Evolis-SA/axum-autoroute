use std::ops::Deref;
use std::str::FromStr;

use quote::quote;
use syn::spanned::Spanned;
use syn::{FnArg, GenericArgument, Ident, ItemFn, Pat, PatType, PathArguments, Type, TypePath, parse_quote_spanned};

use crate::args::extractor_attr::ExtractorAttr;
use crate::utils::spanned::SpannedValue;
use crate::{syn_bail, syn_error};

/// Enum describing the different axum extractors types that can be extracted from the function signature.
#[derive(Debug, Clone, strum::Display, strum::EnumString, strum::EnumIter)]
pub(crate) enum AutorouteAxumExtractorType {
    #[strum(disabled)]
    /// An extractor that is not known
    Unknown {
        #[cfg_attr(not(feature = "tracing"), expect(unused))]
        ty: Ident,
    },
    /// The axum extractor to extract a json struct from the request body
    #[strum(serialize = "Json")]
    JsonBody,
    /// An `axum::body::Body` without any extraction perfomed on it
    #[strum(serialize = "Body")]
    RawBody,
    /// Extractor from axum_typed_multipart to extract multipart data from the request body into a struct
    #[strum(serialize = "TypedMultipart")]
    TypedMultipartBody,
    /// Axum extractor to retrieve data from path parameters
    #[strum(serialize = "Path")]
    PathParam,
    /// Axum extractor to retrieve data from query parameters
    #[strum(serialize = "Query")]
    QueryParam,
}

/// Struct describing data detected in the function signature for an axum extractor.
pub(crate) struct AutorouteAxumExtractor {
    /// The extracted variable ($var in Json($var): Json<String>)
    pub(crate) extracted_var: Ident,
    /// The full extractor type (axum::extract::Json<String>)
    pub(crate) full_ty: TypePath,
    /// The extractor type (axum::extract::Json) as an enum value
    pub(crate) extractor_ty: SpannedValue<AutorouteAxumExtractorType>,
    /// The extracted type (String)
    pub(crate) extracted_ty: Type,
    /// The parsed content of the optional attribute attached to the extractor parameter
    pub(crate) attr: ExtractorAttr,
}

impl std::fmt::Debug for AutorouteAxumExtractor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let extracted_ty = &self.extracted_ty;
        f.debug_struct("AutorouteAxumExtractor")
            .field("extractor_ty", &self.extractor_ty)
            .field("extracted_ty", &quote! {#extracted_ty}.to_string())
            .field("extracted_var", &self.extracted_var.to_string())
            .field("attr", &self.attr)
            .finish_non_exhaustive()
    }
}

impl AutorouteAxumExtractor {
    pub(crate) fn is_parts_extractor(&self) -> bool {
        match *self.extractor_ty {
            AutorouteAxumExtractorType::Unknown { .. } => self.attr.is_parts_extractor(),
            AutorouteAxumExtractorType::JsonBody
            | AutorouteAxumExtractorType::RawBody
            | AutorouteAxumExtractorType::TypedMultipartBody => false,
            AutorouteAxumExtractorType::PathParam | AutorouteAxumExtractorType::QueryParam => true,
        }
    }

    pub(crate) fn content_types(&self) -> syn::Result<Vec<String>> {
        Ok(match *self.extractor_ty {
            AutorouteAxumExtractorType::JsonBody => vec!["application/json".to_string()],
            AutorouteAxumExtractorType::RawBody => vec!["application/octet-stream".to_string()],
            AutorouteAxumExtractorType::TypedMultipartBody => vec!["multipart/form-data".to_string()],
            AutorouteAxumExtractorType::Unknown { ty: _ } if !self.attr.content_types().is_empty() => {
                self.attr.content_types()
            }
            _ => syn_bail!(
                self.extractor_ty.span(),
                "INTERNAL_MACRO_ERROR: no default content_type implemented for {:?}",
                *self.extractor_ty
            ),
        })
    }

    #[cfg(feature = "tracing")]
    pub(crate) fn to_trace(&self) -> bool {
        match *self.extractor_ty {
            AutorouteAxumExtractorType::Unknown { ty: _ } => self.attr.to_trace(),
            _ => true,
        }
    }

    pub(crate) fn to_add_in_params(&self) -> bool {
        match *self.extractor_ty {
            AutorouteAxumExtractorType::Unknown { ty: _ } => self.attr.to_add_in_params(),
            _ => self.is_parts_extractor(),
        }
    }

    pub(crate) fn openapi_content(&self) -> syn::Result<Type> {
        Ok(match *self.extractor_ty {
            AutorouteAxumExtractorType::RawBody => parse_quote_spanned! {self.extracted_ty.span()=> [u8]},
            AutorouteAxumExtractorType::JsonBody
            | AutorouteAxumExtractorType::TypedMultipartBody
            | AutorouteAxumExtractorType::Unknown { ty: _ } => self.extracted_ty.clone(),
            _ => syn_bail!(
                self.extractor_ty.span(),
                "INTERNAL_MACRO_ERROR: no default content implemented for {:?}",
                *self.extractor_ty
            ),
        })
    }

    pub(crate) fn parse_many(itemfn: &ItemFn) -> syn::Result<Vec<Self>> {
        let mut extractors = Vec::new();
        for fnarg in &itemfn.sig.inputs {
            let FnArg::Typed(fnarg) = fnarg else {
                syn_bail!(fnarg.span(), "expected a typed function argument");
            };
            extractors.push(Self::parse_fn_arg(fnarg)?);
        }
        Ok(extractors)
    }

    /// Parse a single extractor argument
    fn parse_fn_arg(fnarg: &PatType) -> syn::Result<Self> {
        let fntype = fnarg.ty.deref();

        // get the type path
        let Type::Path(full_ty) = fntype.clone() else {
            syn_bail!(fntype.span(), "should be a type path");
        };
        // get the last element of the path
        let Some(last_segment) = full_ty.path.segments.last() else {
            syn_bail!(full_ty.span(), "type path without a last segment");
        };

        // and check if it matches with one of the searched extractor
        let extractor_ty = AutorouteAxumExtractorType::from_str(&last_segment.ident.to_string()).unwrap_or(
            AutorouteAxumExtractorType::Unknown {
                ty: last_segment.ident.clone(),
            },
        );
        let extractor_ty = SpannedValue::new(extractor_ty, full_ty.span());

        let extracted_ty;
        // extract the generic type
        if let PathArguments::AngleBracketed(generic_args) = &last_segment.arguments {
            if generic_args.args.len() != 1 {
                syn_bail!(
                    full_ty.span(),
                    "only axum extractors with a single generic argument are currently supported"
                );
            }
            let generic_arg = generic_args.args.first().
                    // expect is ok, we checked that there was a single value just before
                    expect("expected at least one generic argument");
            let GenericArgument::Type(generic_type) = generic_arg else {
                syn_bail!(full_ty.span(), "axum extractor generic argument should be a type");
            };
            extracted_ty = generic_type.clone();
        } else {
            // or use the full type if there is no generic
            extracted_ty = fntype.clone();
        }

        let extracted_var = Self::detect_extractor_var(fnarg.pat.deref())?;
        let attr = ExtractorAttr::parse_fn_arg(fnarg)?;

        Self {
            extracted_var,
            full_ty,
            extractor_ty,
            extracted_ty,
            attr,
        }
        .validate()
    }

    /// Detect the variable to which the extractor will be affected
    fn detect_extractor_var(fnvarpat: &Pat) -> syn::Result<Ident> {
        match fnvarpat {
            // nested pattern
            Pat::Reference(patref) => Self::extractor_var_from_pat_ident(&patref.pat),
            Pat::Type(patty) => Self::extractor_var_from_pat_ident(&patty.pat),
            // destructuring pattern, allowing a single field
            Pat::Struct(patstruct) => Self::detect_extractor_var_from_pat_iter(
                fnvarpat,
                patstruct.fields.iter().map(|field| field.pat.deref()),
            ),
            Pat::Tuple(pattuple) => Self::detect_extractor_var_from_pat_iter(fnvarpat, pattuple.elems.iter()),
            Pat::TupleStruct(pattuple) => Self::detect_extractor_var_from_pat_iter(fnvarpat, pattuple.elems.iter()),
            // default
            _ => Self::extractor_var_from_pat_ident(fnvarpat),
        }
    }

    fn detect_extractor_var_from_pat_iter<'a, I>(fnvarpat: &Pat, mut it: I) -> syn::Result<Ident>
    where
        I: Iterator<Item = &'a Pat> + ExactSizeIterator,
    {
        if it.len() != 1 {
            syn_bail!(
                fnvarpat.span(),
                "unexpected destructuring pattern, should have a single element"
            );
        }

        // unwrap is ok, we just checked the iterator size
        Self::extractor_var_from_pat_ident(it.next().unwrap())
    }

    fn extractor_var_from_pat_ident(pat: &Pat) -> syn::Result<Ident> {
        match pat {
            // found ident
            Pat::Ident(patident) => Ok(patident.ident.clone()),
            // unexpected pattern type
            _ => Err(syn_error!(
                pat.span(),
                "unable to determine extractor variable: unexpected extractor pattern, there should be a named variable and if a destructuring pattern is used it should contain a single variable which cannot be a nested destructuring pattern"
            )),
        }
    }

    #[cfg_attr(not(feature = "unstable_extractor_attr"), expect(clippy::unnecessary_wraps))]
    fn validate(self) -> syn::Result<Self> {
        #[cfg(feature = "unstable_extractor_attr")]
        {
            static KNOWN_EXTRACTOR_TYPES: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
                <AutorouteAxumExtractorType as strum::IntoEnumIterator>::iter()
                    .map(|ty| ty.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            });

            // known extractor types cannot have extractor parts/body info in the attribute
            if !matches!(*self.extractor_ty, AutorouteAxumExtractorType::Unknown { ty: _ }) {
                use proc_macro2::Span;

                match &self.attr.variant {
                    crate::args::extractor_attr::ExtractorAttrVariant::Unspecified => (), // ok
                    crate::args::extractor_attr::ExtractorAttrVariant::PartsExtractor { into_params } => syn_bail!(
                        into_params.span(),
                        "into_params cannot be defined on a known extractor type ({})",
                        *KNOWN_EXTRACTOR_TYPES
                    ),
                    crate::args::extractor_attr::ExtractorAttrVariant::BodyExtractor { content_types } => syn_bail!(
                        content_types.get(0).map(|ct| ct.span()).unwrap_or(Span::call_site()),
                        "content_type cannot be defined on a known extractor type ({})",
                        *KNOWN_EXTRACTOR_TYPES
                    ),
                }
            }
        }

        Ok(self)
    }
}
