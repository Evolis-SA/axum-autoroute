use axum::http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use axum::http::{HeaderMap, HeaderValue};
use axum_autoroute::response::RawResponseBody;
use axum_autoroute::prelude::*;
use axum_autoroute::{AutorouteApiRouter, autoroute, method_routers};

pub fn router() -> AutorouteApiRouter {
    AutorouteApiRouter::new().with_pub_routes(method_routers!(response_file_attachment, response_file_inline))
}

/// Returns a file attachment using a `RawResponseBody` and headers.
#[autoroute(GET, path="/response/file/attachment", tags=["response"],
    responses=[
        (200, body=(HeaderMap, RawResponseBody), serializer=NONE, content_type=APPLICATION_OCTET_STREAM, headers=[(CONTENT_TYPE), (CONTENT_ENCODING)], description="Return a file as attachment (download)"),
    ]
)]
async fn response_file_attachment() -> ResponseFileAttachmentResponses {
    let mut headers = HeaderMap::new();
    headers.append(CONTENT_TYPE, HeaderValue::from_static(mime::TEXT_PLAIN.as_ref()));
    headers.append(
        CONTENT_DISPOSITION,
        HeaderValue::from_static("attachment; filename=text_file.txt"),
    );
    (headers, "the file content".into()).into_200()
}

/// Returns an inline file using a `RawResponseBody` and headers.
#[autoroute(GET, path="/response/file/inline", tags=["response"],
    responses=[
        (200, body=(HeaderMap, RawResponseBody), serializer=NONE, content_type="application/octet-stream", headers=[(CONTENT_TYPE), (CONTENT_ENCODING)], description="Return an inline file"),
    ]
)]
async fn response_file_inline() -> ResponseFileInlineResponses {
    let mut headers = HeaderMap::new();
    headers.append(CONTENT_TYPE, HeaderValue::from_static(mime::TEXT_PLAIN.as_ref()));
    headers.append(
        CONTENT_DISPOSITION,
        HeaderValue::from_static("inline; filename=text_file.txt"),
    );
    (headers, "the file content".into()).into_200()
}

#[cfg(test)]
mod test {
    use axum::http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
    use axum::http::{HeaderValue, Method, StatusCode};
    use tower::ServiceExt;

    use super::router;
    use crate::test_utils::*;

    #[tokio::test]
    #[cfg_attr(feature = "tracing", tracing_test::traced_test)]
    async fn response_file_attachment() {
        let (router, _) = router().split_for_parts();
        let response = router
            .oneshot(request_empty(Method::GET, "/response/file/attachment"))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let headers = response.headers();
        assert_eq!(
            headers.get(CONTENT_TYPE).unwrap(),
            HeaderValue::from_static("text/plain")
        );
        assert_eq!(
            headers.get(CONTENT_DISPOSITION).unwrap(),
            HeaderValue::from_static("attachment; filename=text_file.txt")
        );
        assert_eq!(response_to_str(response).await, "the file content");

        assert_traces!("response_file_attachment.traces");
    }

    #[tokio::test]
    #[cfg_attr(feature = "tracing", tracing_test::traced_test)]
    async fn response_file_inline() {
        let (router, _) = router().split_for_parts();
        let response = router
            .oneshot(request_empty(Method::GET, "/response/file/inline"))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let headers = response.headers();
        assert_eq!(
            headers.get(CONTENT_TYPE).unwrap(),
            HeaderValue::from_static("text/plain")
        );
        assert_eq!(
            headers.get(CONTENT_DISPOSITION).unwrap(),
            HeaderValue::from_static("inline; filename=text_file.txt")
        );
        assert_eq!(response_to_str(response).await, "the file content");

        assert_traces!("response_file_inline.traces");
    }

    #[test]
    fn response_file_openapi() {
        let (_, doc) = router().split_for_parts();
        check_openapi("response_file.openapi.json", &doc);
    }
}
