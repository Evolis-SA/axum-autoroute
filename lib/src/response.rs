//! Utilities to generate `Response` with an associated `OpenApi` documentation.

use axum::body::{Body, HttpBody};
use axum::response::IntoResponse;
use utoipa::ToSchema;

#[derive(ToSchema)]
#[schema(value_type = String, format = Binary, content_media_type = "application/octet-stream")]
/// Utility struct wrapping an `axum::body::Body`.
/// Implements `utoipa::ToSchema` for the `OpenApi` documentation.
///
/// See the `response_file.rs` example for a usage demo.
pub struct RawResponseBody(Body);

impl IntoResponse for RawResponseBody {
    fn into_response(self) -> axum::response::Response {
        self.0.into_response()
    }
}

// Note: sadly implementing this prevents the implementation of Into<Body> and From<Body>
impl<T> From<T> for RawResponseBody
where
    T: Into<Body>,
{
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

impl std::fmt::Debug for RawResponseBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let size_hint = self.0.size_hint();
        let size_str;
        if let Some(exact) = size_hint.exact() {
            size_str = format!("exact_size: {exact}B");
        } else {
            size_str = format!(
                "size_hint: {}..{}B",
                size_hint.lower(),
                size_hint.upper().map_or("?".to_string(), |u| u.to_string())
            );
        }
        f.write_str(&format!("RawResponseBody({size_str})"))
    }
}
