use std::sync::Arc;
use std::sync::atomic::AtomicU8;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum_autoroute::AutorouteApiRouter;
use routes::{
    body_json, body_multipart, body_raw, hello, params_path, params_query, response_cookie, response_json, state,
};
use utoipa::OpenApi;

pub use self::doc::OpenApiDoc;
use crate::routes::{main_example, response_file, route_info};

pub mod routes;
#[cfg(test)]
mod test_utils;

#[derive(Debug)]
pub struct ApiState {
    pub counter: AtomicU8,
}

impl ApiState {
    #[must_use]
    pub fn new() -> Arc<Self> {
        Arc::new(Self { counter: 0.into() })
    }
}

mod doc {
    // clippy warning emitted by utoipa::OpenApi macro
    #![allow(clippy::needless_for_each)]

    #[derive(utoipa::OpenApi)]
    #[openapi(info(title = "My test OpenAPI spec", version = "A.B.C"), tags(
        (name="hello", description="This is the description of the 'hello' tag"),
        (name="world", description="This is the description of the 'world' tag"),
    ))]
    pub struct OpenApiDoc;
}

pub fn app() -> AutorouteApiRouter {
    let state = ApiState::new();
    AutorouteApiRouter::new_with_openapi(OpenApiDoc::openapi())
        .fallback(fallback_handler)
        .merge(hello::router())
        .merge(main_example::router())
        .merge(response_json::router())
        .merge(params_path::router())
        .merge(params_query::router())
        .merge(state::router().with_state(state))
        .merge(body_json::router())
        .merge(body_raw::router())
        .merge(body_multipart::router())
        .merge(response_cookie::router())
        .merge(response_file::router())
        .merge(route_info::router())
}

async fn fallback_handler() -> Response {
    (StatusCode::NOT_FOUND, "unknown route").into_response()
}
