use std::str::FromStr;

use strum::IntoEnumIterator;
use syn::parse::Parse;
use syn::{Attribute, FnArg, Ident, LitBool, Meta, MetaList, PatType, Token};

use crate::AutorouteInput;
use crate::utils::path_as_str;

/// Enum listing the different parameters of the extractor attribute.
#[derive(Debug, Clone, Copy, strum::Display, strum::EnumString, strum::EnumIter)]
#[strum(serialize_all = "snake_case")]
pub(crate) enum ExtractorAttrKey {
    #[cfg(feature = "unstable_extractor_attr")]
    ContentType,
    #[cfg(feature = "unstable_extractor_attr")]
    IntoParams,
    Trace,
}

#[derive(Debug, Default)]
pub(crate) struct ExtractorAttr {
    /// Indicates whether the extractor should be logged or not
    pub(crate) do_trace: Option<LitBool>,
    #[cfg(feature = "unstable_extractor_attr")]
    pub(crate) variant: ExtractorAttrVariant,
}

#[cfg(feature = "unstable_extractor_attr")]
#[derive(Debug, Default)]
pub(crate) enum ExtractorAttrVariant {
    /// Nothing was specified about this extractor in the attributes.
    /// It will be considered as a parts extractor, but will be ignored for all code generation (no tracing or inclusion in openapi documentation).
    #[default]
    Unspecified,
    /// The attribute provides information about a parts extractor.
    PartsExtractor {
        /// Indicates whether the extractor should be integrated in utoipa params.
        into_params: LitBool,
    },
    /// The attribute provides information about a body extractor.
    BodyExtractor {
        /// Mime type of the body
        content_types: Vec<crate::utils::spanned::SpannedValue<mime::Mime>>,
    },
}

impl Parse for ExtractorAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let key_error = |span| {
            syn::Error::new(
                span,
                format!(
                    "expected one of: {}",
                    ExtractorAttrKey::iter()
                        .map(|key| key.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            )
        };

        let mut extractor_attr = Self::default();

        let mut is_first = true;
        while !input.is_empty() {
            if is_first {
                is_first = false;
            } else {
                // allow trailing comma
                input.parse::<Token![,]>()?;
                if input.is_empty() {
                    break;
                }
            }

            let ident: Ident = input.parse().map_err(|e| key_error(e.span()))?;
            let key = ExtractorAttrKey::from_str(&ident.to_string()).map_err(|_| key_error(ident.span()))?;
            input.parse::<Token![=]>()?;
            match key {
                ExtractorAttrKey::Trace => {
                    extractor_attr.do_trace = Some(input.parse()?);
                }
                #[cfg(feature = "unstable_extractor_attr")]
                ExtractorAttrKey::IntoParams => {
                    let value = input.parse()?;
                    match &mut extractor_attr.variant {
                        ExtractorAttrVariant::Unspecified => {
                            extractor_attr.variant = ExtractorAttrVariant::PartsExtractor { into_params: value }
                        }
                        ExtractorAttrVariant::PartsExtractor { into_params } => *into_params = value,
                        ExtractorAttrVariant::BodyExtractor { content_types: _ } => crate::syn_bail!(
                            ident.span(),
                            "into_params cannot be defined in an extractor attribute containing content_type"
                        ),
                    }
                }
                #[cfg(feature = "unstable_extractor_attr")]
                ExtractorAttrKey::ContentType => {
                    let mime = crate::utils::mime::parse_mime(input)?;
                    match &mut extractor_attr.variant {
                        ExtractorAttrVariant::Unspecified => {
                            extractor_attr.variant = ExtractorAttrVariant::BodyExtractor {
                                content_types: vec![mime],
                            }
                        }
                        ExtractorAttrVariant::PartsExtractor { into_params: _ } => crate::syn_bail!(
                            ident.span(),
                            "content_type cannot be defined in an extractor attribute containing into_params"
                        ),
                        ExtractorAttrVariant::BodyExtractor { content_types } => content_types.push(mime),
                    }
                }
            }
        }

        Ok(extractor_attr)
    }
}

impl ExtractorAttr {
    #[cfg_attr(not(feature = "unstable_extractor_attr"), expect(clippy::unused_self))]
    pub(crate) fn is_parts_extractor(&self) -> bool {
        #[cfg(feature = "unstable_extractor_attr")]
        if matches!(self.variant, ExtractorAttrVariant::BodyExtractor { .. }) {
            return false;
        }
        true
    }

    #[cfg(feature = "tracing")]
    pub(crate) fn to_trace(&self) -> bool {
        if let Some(do_trace) = &self.do_trace {
            return do_trace.value;
        }

        #[cfg(feature = "unstable_extractor_attr")]
        // if do_trace was not specified, default behavior is to trace if displayed in openapi spec
        match &self.variant {
            ExtractorAttrVariant::PartsExtractor { into_params } => into_params.value,
            ExtractorAttrVariant::BodyExtractor { content_types: _ } => true,
            ExtractorAttrVariant::Unspecified => false,
        }

        #[cfg(not(feature = "unstable_extractor_attr"))]
        false
    }

    #[cfg_attr(not(feature = "unstable_extractor_attr"), expect(clippy::unused_self))]
    pub(crate) fn to_add_in_params(&self) -> bool {
        #[cfg(feature = "unstable_extractor_attr")]
        if let ExtractorAttrVariant::PartsExtractor { into_params } = &self.variant {
            return into_params.value;
        }

        false
    }

    #[cfg_attr(not(feature = "unstable_extractor_attr"), expect(clippy::unused_self))]
    pub(crate) fn content_types(&self) -> Vec<String> {
        #[cfg(feature = "unstable_extractor_attr")]
        if let ExtractorAttrVariant::BodyExtractor { content_types } = &self.variant {
            return content_types.iter().map(ToString::to_string).collect();
        }

        Vec::new()
    }

    /// Parse the extractor from a function input parameter
    pub(crate) fn parse_fn_arg(fnarg: &PatType) -> syn::Result<Self> {
        for attr in &fnarg.attrs {
            if let Some(meta) = Self::as_extractor_attr(attr) {
                return meta.parse_args();
            }
        }
        Ok(Self::default())
    }

    /// Remove extractor attributes from the ItemFn signature
    pub(crate) fn remove_extractor_attrs(input: &mut AutorouteInput) {
        for mut fnarg in &mut input.itemfn.sig.inputs {
            if let FnArg::Typed(fnarg) = &mut fnarg {
                fnarg.attrs = fnarg
                    .attrs
                    .iter()
                    .filter(|attr| Self::as_extractor_attr(attr).is_none())
                    .cloned()
                    .collect();
            }
        }
    }

    fn as_extractor_attr(attr: &Attribute) -> Option<MetaList> {
        const EXTRACTOR_ATTR_PATHS: [&str; 2] = ["extractor", "autoroute_extractor"];

        if let Meta::List(meta) = &attr.meta
            && EXTRACTOR_ATTR_PATHS.contains(&path_as_str(&meta.path).as_str())
        {
            Some(meta.clone())
        } else {
            None
        }
    }
}
