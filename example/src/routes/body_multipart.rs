use axum::body::Bytes;
use axum_autoroute::status_trait::IntoOk;
use axum_autoroute::{AutorouteApiRouter, autoroute, method_router};
use axum_typed_multipart::{FieldData, TryFromField, TryFromMultipart, TypedMultipart};
use utoipa::ToSchema;

pub fn router() -> AutorouteApiRouter {
    AutorouteApiRouter::new().with_pub_route(method_router!(body_multipart))
}

#[derive(Debug, TryFromMultipart, ToSchema)]
/// Multipart structure
struct MultipartStructure {
    /// A number
    num: u32,
    /// Multiple strings
    names: Vec<String>,
    /// Multiple enum variants
    variants: Vec<MultipartEnum>,
    /// Single file
    #[schema(value_type = String, format = Binary, content_media_type = "application/octet-stream")]
    file: FieldData<Bytes>,
    /// Multiple files
    #[schema(value_type = Vec<String>, format = Binary, content_media_type = "application/octet-stream")]
    files: Vec<FieldData<Bytes>>,
}

#[derive(Debug, TryFromField, ToSchema)]
/// A multipart enum
/// Must be an enum without any variant field
enum MultipartEnum {
    /// Multipart variant 1
    V1,
    /// Multipart variant 2
    V2,
}

/// Parse a multipart body
#[autoroute(POST, path="/body/multipart", tags=["body"],
    responses=[
        (200, body=String, serializer=NONE, description="Returns a string describing the received body"),
    ]
)]
async fn body_multipart(TypedMultipart(mpart): TypedMultipart<MultipartStructure>) -> BodyMultipartResponses {
    format!(
        "num={}, names={:?}, variants={:?}, file={:?}, files={:?}",
        mpart.num,
        mpart.names,
        mpart.variants,
        mpart.file.contents,
        mpart.files.into_iter().map(|f| f.contents).collect::<Vec<_>>(),
    )
    .into_ok()
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use axum::body::Body;
    use axum::http::header::CONTENT_TYPE;
    use axum::http::{Method, Request, StatusCode};
    use common_multipart_rfc7578::client::multipart::{Body as MPartBody, Form};
    use tower::ServiceExt;

    use super::router;
    use crate::test_utils::*;

    #[tokio::test]
    #[cfg_attr(feature = "tracing", tracing_test::traced_test)]
    async fn body_multipart() {
        let (router, _) = router().split_for_parts();

        let file_text = "this is the content of the text file";
        let files_text = ["first file content", "second file content"];

        let mut form = Form::default();
        form.add_text("num", "32");
        form.add_reader_file_with_mime("file", Cursor::new(file_text), "text_file.txt", mime::TEXT_PLAIN);
        // each variant field will be an entry in the vector
        for variant in ["V2", "V1", "V2"] {
            form.add_text("variants", variant);
        }
        // each names field will be an entry in the vector
        for name in ["name a", "name b", "name c"] {
            form.add_text("names", name);
        }
        // each files field will be a new file in the vector
        for file in files_text {
            form.add_reader_file_with_mime("files", Cursor::new(file), "text_file.txt", mime::TEXT_PLAIN);
        }

        let content_type = form.content_type();
        let mpart_body = MPartBody::from(form);
        let axum_body = Body::from_stream(mpart_body);

        let response = router
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/body/multipart")
                    .header(CONTENT_TYPE, content_type)
                    .body(axum_body)
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response_to_str(response).await,
            r#"num=32, names=["name a", "name b", "name c"], variants=[V2, V1, V2], file=b"this is the content of the text file", files=[b"first file content", b"second file content"]"#,
        );

        assert_traces!("body_multipart.traces");
    }

    #[test]
    fn body_multipart_openapi() {
        let (_, doc) = router().split_for_parts();
        // TODO: currently, swagger ui does not handle correctly the array of string and array of enum
        // https://github.com/juhaku/utoipa/issues/1397
        check_openapi("body_multipart.openapi.json", &doc);
    }
}
