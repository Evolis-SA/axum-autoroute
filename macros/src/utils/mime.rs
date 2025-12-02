use std::str::FromStr;

use mime::*;
use syn::parse::ParseStream;
use syn::{Ident, LitStr};

use crate::syn_error;
use crate::utils::spanned::SpannedValue;

#[derive(Debug, Clone, Copy, strum::Display, strum::EnumString, strum::EnumIter)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum KnownMimes {
    StarStar,
    TextStar,
    TextPlain,
    #[strum(serialize = "TEXT_PLAIN_UTF_8")]
    TextPlainUtf8,
    TextHtml,
    #[strum(serialize = "TEXT_HTML_UTF_8")]
    TextHtmlUtf8,
    TextCss,
    #[strum(serialize = "TEXT_CSS_UTF_8")]
    TextCssUtf8,
    TextJavascript,
    TextXml,
    TextEventStream,
    TextCsv,
    #[strum(serialize = "TEXT_CSV_UTF_8")]
    TextCsvUtf8,
    TextTabSeparatedValues,
    #[strum(serialize = "TEXT_TAB_SEPARATED_VALUES_UTF_8")]
    TextTabSeparatedValuesUtf8,
    TextVcard,
    ImageStar,
    ImageJpeg,
    ImageGif,
    ImagePng,
    ImageBmp,
    ImageSvg,
    FontWoff,
    FontWoff2,
    ApplicationJson,
    ApplicationJavascript,
    #[strum(serialize = "APPLICATION_JAVASCRIPT_UTF_8")]
    ApplicationJavascriptUtf8,
    ApplicationWwwFormUrlencoded,
    ApplicationOctetStream,
    ApplicationMsgpack,
    ApplicationPdf,
    MultipartFormData,
}

impl From<KnownMimes> for Mime {
    fn from(value: KnownMimes) -> Self {
        match value {
            KnownMimes::StarStar => STAR_STAR,
            KnownMimes::TextStar => TEXT_STAR,
            KnownMimes::TextPlain => TEXT_PLAIN,
            KnownMimes::TextPlainUtf8 => TEXT_PLAIN_UTF_8,
            KnownMimes::TextHtml => TEXT_HTML,
            KnownMimes::TextHtmlUtf8 => TEXT_HTML_UTF_8,
            KnownMimes::TextCss => TEXT_CSS,
            KnownMimes::TextCssUtf8 => TEXT_CSS_UTF_8,
            KnownMimes::TextJavascript => TEXT_JAVASCRIPT,
            KnownMimes::TextXml => TEXT_XML,
            KnownMimes::TextEventStream => TEXT_EVENT_STREAM,
            KnownMimes::TextCsv => TEXT_CSV,
            KnownMimes::TextCsvUtf8 => TEXT_CSV_UTF_8,
            KnownMimes::TextTabSeparatedValues => TEXT_TAB_SEPARATED_VALUES,
            KnownMimes::TextTabSeparatedValuesUtf8 => TEXT_TAB_SEPARATED_VALUES_UTF_8,
            KnownMimes::TextVcard => TEXT_VCARD,
            KnownMimes::ImageStar => IMAGE_STAR,
            KnownMimes::ImageJpeg => IMAGE_JPEG,
            KnownMimes::ImageGif => IMAGE_GIF,
            KnownMimes::ImagePng => IMAGE_PNG,
            KnownMimes::ImageBmp => IMAGE_BMP,
            KnownMimes::ImageSvg => IMAGE_SVG,
            KnownMimes::FontWoff => FONT_WOFF,
            KnownMimes::FontWoff2 => FONT_WOFF2,
            KnownMimes::ApplicationJson => APPLICATION_JSON,
            KnownMimes::ApplicationJavascript => APPLICATION_JAVASCRIPT,
            KnownMimes::ApplicationJavascriptUtf8 => APPLICATION_JAVASCRIPT_UTF_8,
            KnownMimes::ApplicationWwwFormUrlencoded => APPLICATION_WWW_FORM_URLENCODED,
            KnownMimes::ApplicationOctetStream => APPLICATION_OCTET_STREAM,
            KnownMimes::ApplicationMsgpack => APPLICATION_MSGPACK,
            KnownMimes::ApplicationPdf => APPLICATION_PDF,
            KnownMimes::MultipartFormData => MULTIPART_FORM_DATA,
        }
    }
}

pub(crate) fn parse_mime(input: ParseStream) -> syn::Result<SpannedValue<Mime>> {
    let mime_error = |span| {
        syn::Error::new(
            span,
            "unexpected mime type, should be a string or one of the constant defined in https://docs.rs/mime/latest/mime/index.html",
        )
    };

    if let Ok(ident) = input.parse::<Ident>() {
        let known_mime = KnownMimes::from_str(&ident.to_string()).map_err(|_| mime_error(ident.span()))?;
        Ok(SpannedValue::new(known_mime.into(), ident.span()))
    } else if let Ok(code_lit) = input.parse::<LitStr>() {
        Ok(SpannedValue::new(
            Mime::from_str(&code_lit.value()).map_err(|e| syn_error!(code_lit.span(), "invalid mime string: {e}"))?,
            code_lit.span(),
        ))
    } else {
        Err(mime_error(input.span()))
    }
}
