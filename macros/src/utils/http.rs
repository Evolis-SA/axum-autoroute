use std::str::FromStr;

use axum::http::header::*;
use axum::http::{HeaderName, StatusCode};
use convert_case::{Case, Casing};
use strum::IntoEnumIterator;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, LitInt};

use super::spanned::SpannedValue;

#[derive(Debug, Clone, Copy, strum::Display, strum::EnumString, strum::EnumIter)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum HttpMethod {
    Get,
    Post,
    Delete,
    Put,
    Patch,
    Connect,
    Options,
    Head,
    Trace,
}

impl Parse for SpannedValue<HttpMethod> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let method_error = |span| {
            syn::Error::new(
                span,
                format!(
                    "unexpected method, should be one of: {}",
                    HttpMethod::iter()
                        .map(|met| met.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            )
        };

        let ident: Ident = input.parse().map_err(|e| method_error(e.span()))?;
        match HttpMethod::from_str(&ident.to_string()) {
            Ok(met) => Ok(SpannedValue::new(met, ident.span())),
            Err(_) => Err(method_error(ident.span())),
        }
    }
}

#[derive(Debug, Clone, Copy, strum::Display, strum::EnumString, strum::FromRepr, PartialEq, Eq, PartialOrd, Ord)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
#[repr(u16)]
pub(crate) enum HttpStatusCode {
    Continue = 100,
    SwitchingProtocols = 101,
    Processing = 102,
    Ok = 200,
    Created = 201,
    Accepted = 202,
    NonAuthoritativeInformation = 203,
    NoContent = 204,
    ResetContent = 205,
    PartialContent = 206,
    MultiStatus = 207,
    AlreadyReported = 208,
    ImUsed = 226,
    MultipleChoices = 300,
    MovedPermanently = 301,
    Found = 302,
    SeeOther = 303,
    NotModified = 304,
    UseProxy = 305,
    TemporaryRedirect = 307,
    PermanentRedirect = 308,
    BadRequest = 400,
    Unauthorized = 401,
    PaymentRequired = 402,
    Forbidden = 403,
    NotFound = 404,
    MethodNotAllowed = 405,
    NotAcceptable = 406,
    ProxyAuthenticationRequired = 407,
    RequestTimeout = 408,
    Conflict = 409,
    Gone = 410,
    LengthRequired = 411,
    PreconditionFailed = 412,
    PayloadTooLarge = 413,
    UriTooLong = 414,
    UnsupportedMediaType = 415,
    RangeNotSatisfiable = 416,
    ExpectationFailed = 417,
    ImATeapot = 418,
    MisdirectedRequest = 421,
    UnprocessableEntity = 422,
    Locked = 423,
    FailedDependency = 424,
    TooEarly = 425,
    UpgradeRequired = 426,
    PreconditionRequired = 428,
    TooManyRequests = 429,
    RequestHeaderFieldsTooLarge = 431,
    UnavailableForLegalReasons = 451,
    InternalServerError = 500,
    NotImplemented = 501,
    BadGateway = 502,
    ServiceUnavailable = 503,
    GatewayTimeout = 504,
    HttpVersionNotSupported = 505,
    VariantAlsoNegotiates = 506,
    InsufficientStorage = 507,
    LoopDetected = 508,
    NotExtended = 510,
    NetworkAuthenticationRequired = 511,
}

impl HttpStatusCode {
    pub(crate) fn as_code(self) -> u16 {
        self as u16
    }

    #[expect(unused)]
    pub(crate) fn as_axum(self) -> StatusCode {
        StatusCode::from_u16(self.as_code()).expect("unable to convert status to an axum code")
    }
}

impl Parse for SpannedValue<HttpStatusCode> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let status_error = |span| {
            syn::Error::new(
                span,
                "unexpected status code, should be a numeric status code or a constant as described in https://docs.rs/http/latest/http/status/struct.StatusCode.html",
            )
        };

        if let Ok(ident) = input.parse::<Ident>() {
            HttpStatusCode::from_str(&ident.to_string())
                .map(|sc| SpannedValue::new(sc, ident.span()))
                .map_err(|_| status_error(ident.span()))
        } else if let Ok(code_lit) = input.parse::<LitInt>() {
            let code_num: u16 = code_lit.base10_parse()?;
            HttpStatusCode::from_repr(code_num)
                .map(|sc| SpannedValue::new(sc, code_lit.span()))
                .ok_or(status_error(code_lit.span()))
        } else {
            Err(status_error(input.span()))
        }
    }
}

const KNOWN_HTTP_HEADERS: &[HeaderName] = &[
    ACCEPT,
    ACCEPT_CHARSET,
    ACCEPT_ENCODING,
    ACCEPT_LANGUAGE,
    ACCEPT_RANGES,
    ACCESS_CONTROL_ALLOW_CREDENTIALS,
    ACCESS_CONTROL_ALLOW_HEADERS,
    ACCESS_CONTROL_ALLOW_METHODS,
    ACCESS_CONTROL_ALLOW_ORIGIN,
    ACCESS_CONTROL_EXPOSE_HEADERS,
    ACCESS_CONTROL_MAX_AGE,
    ACCESS_CONTROL_REQUEST_HEADERS,
    ACCESS_CONTROL_REQUEST_METHOD,
    AGE,
    ALLOW,
    ALT_SVC,
    AUTHORIZATION,
    CACHE_CONTROL,
    CACHE_STATUS,
    CDN_CACHE_CONTROL,
    CONNECTION,
    CONTENT_DISPOSITION,
    CONTENT_ENCODING,
    CONTENT_LANGUAGE,
    CONTENT_LENGTH,
    CONTENT_LOCATION,
    CONTENT_RANGE,
    CONTENT_SECURITY_POLICY,
    CONTENT_SECURITY_POLICY_REPORT_ONLY,
    CONTENT_TYPE,
    COOKIE,
    DNT,
    DATE,
    ETAG,
    EXPECT,
    EXPIRES,
    FORWARDED,
    FROM,
    HOST,
    IF_MATCH,
    IF_MODIFIED_SINCE,
    IF_NONE_MATCH,
    IF_RANGE,
    IF_UNMODIFIED_SINCE,
    LAST_MODIFIED,
    LINK,
    LOCATION,
    MAX_FORWARDS,
    ORIGIN,
    PRAGMA,
    PROXY_AUTHENTICATE,
    PROXY_AUTHORIZATION,
    PUBLIC_KEY_PINS,
    PUBLIC_KEY_PINS_REPORT_ONLY,
    RANGE,
    REFERER,
    REFERRER_POLICY,
    REFRESH,
    RETRY_AFTER,
    SEC_WEBSOCKET_ACCEPT,
    SEC_WEBSOCKET_EXTENSIONS,
    SEC_WEBSOCKET_KEY,
    SEC_WEBSOCKET_PROTOCOL,
    SEC_WEBSOCKET_VERSION,
    SERVER,
    SET_COOKIE,
    STRICT_TRANSPORT_SECURITY,
    TE,
    TRAILER,
    TRANSFER_ENCODING,
    UPGRADE,
    UPGRADE_INSECURE_REQUESTS,
    USER_AGENT,
    VARY,
    VIA,
    WARNING,
    WWW_AUTHENTICATE,
    X_CONTENT_TYPE_OPTIONS,
    X_DNS_PREFETCH_CONTROL,
    X_FRAME_OPTIONS,
    X_XSS_PROTECTION,
];

pub(crate) fn parse_header_name(input: ParseStream) -> syn::Result<SpannedValue<HeaderName>> {
    let header_error = |span| {
        syn::Error::new(
            span,
            "unexpected header name, should be one of the constant defined in https://docs.rs/http/latest/http/header/index.html",
        )
    };

    let header_ident: Ident = input.parse().map_err(|e| header_error(e.span()))?;
    if header_ident.to_string().is_case(Case::UpperSnake) {
        if let Ok(header_name) = HeaderName::from_str(&header_ident.to_string().to_case(Case::Kebab)) {
            if KNOWN_HTTP_HEADERS.contains(&header_name) {
                return Ok(SpannedValue::new(header_name, header_ident.span()));
            }
        }
    }

    Err(header_error(header_ident.span()))
}
