use axum::extract::Query;
use axum_autoroute::prelude::*;
use axum_autoroute::{AutorouteApiRouter, autoroute, method_routers};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

pub fn router() -> AutorouteApiRouter {
    AutorouteApiRouter::new().with_pub_routes(method_routers!(query_params1, query_params2, query_params3))
}

#[derive(Debug, Serialize, Deserialize, IntoParams, ToSchema)]
/// `QueryParam1` documentation
struct QueryParam1 {
    /// A positive number
    id: u32,
    /// A string
    str: String,
}

#[derive(Debug, Serialize, Deserialize, IntoParams, ToSchema)]
struct QueryParam2 {
    number: i32,
}

#[derive(Debug, Deserialize, IntoParams)]
struct QueryParam3 {
    #[param(inline)]
    list: QueryEnum,
}

#[derive(Debug, Deserialize, ToSchema)]
enum QueryEnum {
    Val1,
    Val2,
}

/// This route parses two query params from a single structure
#[autoroute(GET, path="/query1", tags=["query"],
    responses=[
        (OK, body=QueryParam1, description="Returns the provided query params"),
    ]
)]
async fn query_params1(Query(query1): Query<QueryParam1>) -> QueryParams1Responses {
    query1.into_ok()
}

/// This route parse two params (number then string)
#[autoroute(GET, path="/query2", tags=["query"],
    responses=[
        (OK, body=((QueryParam1, QueryParam2)), description="Returns the provided parameters"),
    ]
)]
async fn query_params2(
    #[extractor(trace = false)] Query(query1): Query<QueryParam1>,
    Query(query2): Query<QueryParam2>,
) -> QueryParams2Responses {
    (query1, query2).into_ok()
}

/// This route parses an enum
#[autoroute(GET, path="/query3", tags=["query"],
    responses=[
        (OK, body=String, description="Returns the provided parameter"),
    ]
)]
async fn query_params3(Query(query3): Query<QueryParam3>) -> QueryParams3Responses {
    (format!("{:?}", query3.list)).into_ok()
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
    async fn query_params1() {
        let (mut router, _) = router().split_for_parts();
        let service = build_service(&mut router).await;

        let response = service
            .call(request_empty(Method::GET, "/query1?id=1&str=test"))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response_to_json(response).await, json!({"id": 1, "str": "test"}));

        let response = service
            .call(request_empty(Method::GET, "/query1?str=test2&id=7"))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response_to_json(response).await, json!({"id": 7, "str": "test2"}));

        assert_traces!("query_params1.traces");
    }

    #[tokio::test]
    #[cfg_attr(feature = "tracing", tracing_test::traced_test)]
    async fn query_params2() {
        let (mut router, _) = router().split_for_parts();
        let service = build_service(&mut router).await;

        let response = service
            .call(request_empty(Method::GET, "/query2?id=8&str=test&number=-3"))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response_to_json(response).await,
            json!([{"id": 8, "str": "test"}, {"number": -3}])
        );

        assert_traces!("query_params2.traces");
    }

    #[tokio::test]
    #[cfg_attr(feature = "tracing", tracing_test::traced_test)]
    async fn query_params3() {
        let (mut router, _) = router().split_for_parts();
        let service = build_service(&mut router).await;

        let response = service
            .call(request_empty(Method::GET, "/query3?list=Val1"))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response_to_str(response).await, "\"Val1\"");

        let response = service
            .call(request_empty(Method::GET, "/query3?list=Val2"))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response_to_str(response).await, "\"Val2\"");

        let response = service
            .call(request_empty(Method::GET, "/query3?list=Val3"))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        assert_traces!("query_params3.traces");
    }

    #[test]
    fn params_query_openapi() {
        let (_, doc) = router().split_for_parts();
        check_openapi("params_query.openapi.json", &doc);
    }
}
