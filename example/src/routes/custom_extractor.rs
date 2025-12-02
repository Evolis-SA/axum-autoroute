use axum::Json;
use axum::extract::{FromRequest, FromRequestParts, Query};
use axum_autoroute::status_trait::IntoOk;
use axum_autoroute::{AutorouteApiRouter, autoroute, method_routers};
use serde::Deserialize;
use utoipa::{IntoParams, OpenApi, ToSchema};

use crate::OpenApiDoc;

pub fn router() -> AutorouteApiRouter {
    AutorouteApiRouter::new_with_openapi(OpenApiDoc::openapi()).with_pub_routes(method_routers!(
        custom_body_extractor_1,
        custom_body_extractor_2,
        custom_body_extractor_3,
        custom_body_extractor_4,
        custom_query_extractor_1,
        custom_query_extractor_2,
        custom_query_extractor_3,
        custom_query_extractor_4,
        custom_query_extractor_5,
    ))
}

#[derive(Debug, FromRequest)]
#[from_request(via(Json))]
struct CustomJsonExtractor<T>(T);

#[derive(Debug, Deserialize, ToSchema)]
struct MyJsonStruct {
    txt: String,
}

/// no extractor attr specified, so not documented in openapi and not traced
#[autoroute(POST, path="/extractor/custom_body1", tags=["custom extractor"],
    responses=[
        (OK, body=String, serializer=NONE),
    ]
)]
async fn custom_body_extractor_1(j: CustomJsonExtractor<MyJsonStruct>) -> CustomBodyExtractor1Responses {
    j.0.txt.into_ok()
}

/// content_type defined, documented in openapi and traced
#[autoroute(POST, path="/extractor/custom_body2", tags=["custom extractor"],
    responses=[
        (OK, body=String, serializer=NONE),
    ]
)]
async fn custom_body_extractor_2(
    #[extractor(content_type=APPLICATION_JSON)] j: CustomJsonExtractor<MyJsonStruct>,
) -> CustomBodyExtractor2Responses {
    j.0.txt.into_ok()
}

/// no extractor attr specified, so not documented in openapi but force trace
#[autoroute(POST, path="/extractor/custom_body3", tags=["custom extractor"],
    responses=[
        (OK, body=String, serializer=NONE),
    ]
)]
async fn custom_body_extractor_3(
    #[autoroute_extractor(trace = true)] j: CustomJsonExtractor<MyJsonStruct>,
) -> CustomBodyExtractor3Responses {
    j.0.txt.into_ok()
}

/// content_type defined, documented in openapi but force not traced
#[autoroute(POST, path="/extractor/custom_body4", tags=["custom extractor"],
    responses=[
        (OK, body=String, serializer=NONE),
    ]
)]
async fn custom_body_extractor_4(
    #[extractor(content_type=APPLICATION_JSON, content_type="application/yaml", trace = false)] j: CustomJsonExtractor<
        MyJsonStruct,
    >,
) -> CustomBodyExtractor4Responses {
    j.0.txt.into_ok()
}

#[derive(Debug, FromRequestParts)]
#[from_request(via(Query))]
struct CustomQueryExtractor<T>(T);

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in=Query)]
struct MyQueryStruct {
    num: i32,
}

/// no extractor attr specified, so not documented in openapi and not traced
#[autoroute(POST, path="/extractor/custom_parts1", tags=["custom extractor"],
    responses=[
        (OK, body=String, serializer=NONE),
    ]
)]
async fn custom_query_extractor_1(q: CustomQueryExtractor<MyQueryStruct>) -> CustomQueryExtractor1Responses {
    q.0.num.to_string().into_ok()
}

/// into_params specified and true, documented in openapi and traced
#[autoroute(POST, path="/extractor/custom_parts2", tags=["custom extractor"],
    responses=[
        (OK, body=String, serializer=NONE),
    ]
)]
async fn custom_query_extractor_2(
    #[extractor(into_params = true)] q: CustomQueryExtractor<MyQueryStruct>,
) -> CustomQueryExtractor2Responses {
    q.0.num.to_string().into_ok()
}

/// into_params specified and false, not documented in openapi nor traced
#[autoroute(POST, path="/extractor/custom_parts3", tags=["custom extractor"],
    responses=[
        (OK, body=String, serializer=NONE),
    ]
)]
async fn custom_query_extractor_3(
    #[extractor(into_params = false)] q: CustomQueryExtractor<MyQueryStruct>,
) -> CustomQueryExtractor3Responses {
    q.0.num.to_string().into_ok()
}

/// into_params specified and true, documented in openapi but trace disabled
#[autoroute(POST, path="/extractor/custom_parts4", tags=["custom extractor"],
    responses=[
        (OK, body=String, serializer=NONE),
    ]
)]
async fn custom_query_extractor_4(
    #[extractor(into_params = true, trace = false)] q: CustomQueryExtractor<MyQueryStruct>,
) -> CustomQueryExtractor4Responses {
    q.0.num.to_string().into_ok()
}

/// into_params specified and false, not documented in openapi but trace enabled
#[autoroute(POST, path="/extractor/custom_parts5", tags=["custom extractor"],
    responses=[
        (OK, body=String, serializer=NONE),
    ]
)]
async fn custom_query_extractor_5(
    #[extractor(into_params = false, trace = true)] q: CustomQueryExtractor<MyQueryStruct>,
) -> CustomQueryExtractor5Responses {
    q.0.num.to_string().into_ok()
}

#[cfg(test)]
mod test {
    use axum::http::{Method, StatusCode};
    use serde_json::json;
    use tower::Service;

    use super::router;
    use crate::test_utils::*;

    #[tokio::test]
    #[cfg_attr(feature = "tracing", tracing_test::traced_test)]
    async fn custom_body_extractor() {
        let (mut router, _) = router().split_for_parts();
        let service = build_service(&mut router).await;

        for i in 1..=4 {
            let response = service
                .call(request_json(
                    Method::POST,
                    &format!("/extractor/custom_body{i}"),
                    &json!({"txt": format!("text of body extractor No{i}")}),
                ))
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            assert_eq!(response_to_str(response).await, format!("text of body extractor No{i}"));
        }

        assert_traces!("custom_body_extractor.traces");
    }

    #[tokio::test]
    #[cfg_attr(feature = "tracing", tracing_test::traced_test)]
    async fn custom_query_extractor() {
        let (mut router, _) = router().split_for_parts();
        let service = build_service(&mut router).await;

        for i in 1..=5 {
            let response = service
                .call(request_empty(
                    Method::POST,
                    &format!("/extractor/custom_parts{i}?num={}", i * 2),
                ))
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            assert_eq!(response_to_str(response).await, format!("{}", i * 2));
        }

        assert_traces!("custom_query_extractor.traces");
    }

    #[test]
    fn custom_extractor_openapi() {
        let (_, doc) = router().split_for_parts();
        check_openapi("custom_extractor.openapi.json", &doc);
    }
}
