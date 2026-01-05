use axum::body::{Body, to_bytes};
use axum_autoroute::prelude::*;
use axum_autoroute::{AutorouteApiRouter, autoroute, method_routers};

pub fn router() -> AutorouteApiRouter {
    AutorouteApiRouter::new().with_pub_routes(method_routers!(body_raw))
}

/// Receives a raw body and return its bytes size or fail if it is greater than 100 bytes
#[autoroute(POST, path="/body/raw", tags=["body"],
    responses=[
        (200, body=usize, description="Returns the size of the received body"),
        (500, body=String, serializer=NONE, description="Failed to receive the body"),
    ]
)]
async fn body_raw(body: Body) -> BodyRawResponses {
    if let Ok(bytes) = to_bytes(body, 100).await {
        bytes.len().into_ok()
    } else {
        "Failed to receive body, size greater than 100 bytes"
            .to_string()
            .into_internal_server_error()
    }
}

#[cfg(test)]
mod test {
    use axum::body::Body;
    use axum::extract::Request;
    use axum::http::{Method, StatusCode};
    use tower::Service;

    use super::router;
    use crate::test_utils::*;

    #[tokio::test]
    #[cfg_attr(feature = "tracing", tracing_test::traced_test)]
    async fn body_raw() {
        let (mut router, _) = router().split_for_parts();
        let service = build_service(&mut router).await;

        let response = service
            .call(
                Request::builder()
                    .method(Method::POST)
                    .uri("/body/raw")
                    .body(Body::from(vec![0u8; 90]))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response_to_json(response).await, 90);

        let response = service
            .call(
                Request::builder()
                    .method(Method::POST)
                    .uri("/body/raw")
                    .body(Body::from(vec![0u8; 101]))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(
            response_to_str(response).await,
            "Failed to receive body, size greater than 100 bytes"
        );

        assert_traces!("body_raw.traces");
    }

    #[test]
    fn body_json_openapi() {
        let (_, doc) = router().split_for_parts();
        check_openapi("body_raw.openapi.json", &doc);
    }
}
