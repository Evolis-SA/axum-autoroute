use axum::extract::Path;
use axum_autoroute::prelude::*;
use axum_autoroute::{AutorouteApiRouter, autoroute, method_routers};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

pub fn router() -> AutorouteApiRouter {
    AutorouteApiRouter::new().with_pub_routes(method_routers!(
        path_params_even,
        two_params,
        two_params_reverse,
        bad_path_param,
        multi_path_extractors,
    ))
}

#[derive(Debug, Serialize, Deserialize, IntoParams, ToSchema)]
/// Path param with a single number
struct ParamsEven {
    /// A number
    number: u32,
}

/// This route parses a single path params
#[autoroute(GET, path="/path/{number}", tags=["path"],
    responses=[
        (OK, body=u32, description="Returns the provided number if it is even"),
        (406, body=String, serializer=NONE, description="Returns an error if the provided number is odd"),
    ]
)]
async fn path_params_even(Path(path): Path<ParamsEven>) -> PathParamsEvenResponses {
    let ParamsEven { number } = path;
    if number % 2 == 0 {
        number.into_200()
    } else {
        format!("{number} is not even, it is odd !").into_not_acceptable()
    }
}

#[derive(Debug, Serialize, Deserialize, IntoParams, ToSchema)]
struct TwoParams {
    id: u32,
    name: String,
}

/// This route parse two params (number then string)
#[autoroute(GET, path="/path/{id}/{name}", tags=["path"],
    responses=[
        (OK, body=TwoParams, description="Returns the provided parameters"),
    ]
)]
async fn two_params(Path(params): Path<TwoParams>) -> TwoParamsResponses {
    params.into_ok()
}

/// This route parse two params (string then number)
#[autoroute(GET, path="/path/r/{name}/{id}", tags=["path"],
    responses=[
        (OK, body=TwoParams, description="Returns the provided parameters"),
    ]
)]
async fn two_params_reverse(Path(params): Path<TwoParams>) -> TwoParamsReverseResponses {
    params.into_ok()
}

/// This route tries to extract a params that does not exists, it will fail at runtime
#[autoroute(GET, path="/bad/path/{wrong}", tags=["path"],
    responses=[
        (OK, body=String, description="Always return OK"),
    ]
)]
async fn bad_path_param(
    #[cfg_attr(not(feature = "tracing"), expect(unused))] p: Path<ParamsEven>,
) -> BadPathParamResponses {
    "OK".to_string().into_ok()
}

/// This route uses multiple path extractors
#[autoroute(GET, path="/multi/path/{id}/{name}/{number}", tags=["path"],
    responses=[
        (OK, body=((ParamsEven, TwoParams)), description="Return the values extracted from path"),
    ]
)]
async fn multi_path_extractors(
    Path(params1): Path<ParamsEven>,
    Path(params2): Path<TwoParams>,
) -> MultiPathExtractorsResponses {
    (params1, params2).into_ok()
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
    async fn path_params_even() {
        let (mut router, _) = router().split_for_parts();
        let service = build_service(&mut router).await;

        // even
        let response = service.call(request_empty(Method::GET, "/path/8")).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response_to_str(response).await, "8");

        // odd
        let response = service.call(request_empty(Method::GET, "/path/17")).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_ACCEPTABLE);
        assert_eq!(response_to_str(response).await, "17 is not even, it is odd !");

        // not a number
        let response = service.call(request_empty(Method::GET, "/path/test")).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            response_to_str(response).await,
            "Invalid URL: Cannot parse `number` with value `test` to a `u32`"
        );

        assert_traces!("path_params_even.traces");
    }

    #[tokio::test]
    #[cfg_attr(feature = "tracing", tracing_test::traced_test)]
    async fn two_params() {
        let (mut router, _) = router().split_for_parts();
        let service = build_service(&mut router).await;

        let response = service.call(request_empty(Method::GET, "/path/8/test")).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response_to_json(response).await, json!({"id": 8, "name": "test"}));

        let response = service.call(request_empty(Method::GET, "/path/8/9")).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response_to_json(response).await, json!({"id": 8, "name": "9"}));

        let response = service.call(request_empty(Method::GET, "/path/test/9")).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            response_to_str(response).await,
            "Invalid URL: Cannot parse `id` with value `test` to a `u32`"
        );

        assert_traces!("two_path_params.traces");
    }

    #[tokio::test]
    #[cfg_attr(feature = "tracing", tracing_test::traced_test)]
    async fn two_params_reverse() {
        let (mut router, _) = router().split_for_parts();
        let service = build_service(&mut router).await;

        let response = service.call(request_empty(Method::GET, "/path/r/tt/16")).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response_to_json(response).await, json!({"id": 16, "name": "tt"}));

        let response = service.call(request_empty(Method::GET, "/path/r/32/16")).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response_to_json(response).await, json!({"id": 16, "name": "32"}));

        let response = service.call(request_empty(Method::GET, "/path/r/32/tt")).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            response_to_str(response).await,
            "Invalid URL: Cannot parse `id` with value `tt` to a `u32`"
        );

        assert_traces!("two_path_params_reverse.traces");
    }

    #[tokio::test]
    #[cfg_attr(feature = "tracing", tracing_test::traced_test)]
    async fn multiple_path_extractors() {
        let (mut router, _) = router().split_for_parts();
        let service = build_service(&mut router).await;

        let response = service
            .call(request_empty(Method::GET, "/multi/path/9/test/7"))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response_to_json(response).await,
            json!([{"number": 7}, {"id": 9, "name": "test"}])
        );

        assert_traces!("multi_path_extractors.traces");
    }

    #[tokio::test]
    #[cfg_attr(feature = "tracing", tracing_test::traced_test)]
    async fn runtime_path_extract_failure() {
        let (mut router, _) = router().split_for_parts();
        let service = build_service(&mut router).await;

        let response = service.call(request_empty(Method::GET, "/bad/path/128")).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(response_to_str(response).await, "Invalid URL: missing field `number`");

        assert_traces!("runtime_path_extract_failure.traces");
    }

    #[test]
    fn params_path_openapi() {
        let (_, doc) = router().split_for_parts();
        check_openapi("params_path.openapi.json", &doc);
    }
}
