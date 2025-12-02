use axum_autoroute::status_trait::IntoOk;
use axum_autoroute::{AutorouteApiRouter, autoroute, method_router};
use serde::Serialize;
use utoipa::ToSchema;

pub fn router() -> AutorouteApiRouter {
    AutorouteApiRouter::new().with_pub_route(method_router!(response_json))
}

#[derive(Debug, Serialize, ToSchema)]
/// documentation of the main struct
struct MyResponse {
    /// this is an u32 field
    id: u32,
    /// this is a string field
    str: String,
    /// this field contains a generic structure
    structure: NestedStruct<u32>,
    /// this field is a list of enum values
    enum_iterator: Vec<NestedEnum>,
}

#[derive(Debug, Serialize, ToSchema)]
/// documentation of the enum
enum NestedEnum {
    /// documentation of the first variant
    Variant1,
    /// documentation of the second variant
    Variant2(
        /// documentation of the second variant field
        NestedStruct<String>,
    ),
    /// documentation of the third variant
    Variant3 {
        /// documentation of the third variant field
        value: u32,
    },
}

#[derive(Debug, Serialize, ToSchema)]
/// this is a struct with a generic type
struct NestedStruct<T> {
    /// name of the value
    name: String,
    /// the value itself
    value: T,
}

/// This route always return the same json struct
#[autoroute(GET, path="/response/json", tags=["response"],
    responses=[
        (200, body=MyResponse, description="Always return the same json"),
    ]
)]
async fn response_json() -> ResponseJsonResponses {
    MyResponse {
        id: 16,
        str: "MyResponse".to_string(),
        structure: NestedStruct {
            name: "nested struct".to_string(),
            value: 32,
        },
        enum_iterator: vec![
            NestedEnum::Variant1,
            NestedEnum::Variant2(NestedStruct {
                name: "variant2 struct".to_string(),
                value: "the value of this struct".to_string(),
            }),
            NestedEnum::Variant3 { value: 64 },
        ],
    }
    .into_ok()
}

#[cfg(test)]
mod test {
    use axum::http::{Method, StatusCode};
    use serde_json::json;
    use tower::ServiceExt;

    use super::router;
    use crate::test_utils::*;

    #[tokio::test]
    #[cfg_attr(feature = "tracing", tracing_test::traced_test)]
    async fn response_json() {
        let (router, _) = router().split_for_parts();
        let response = router
            .oneshot(request_empty(Method::GET, "/response/json"))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response_to_json(response).await,
            json!({
                "id": 16,
                "str": "MyResponse",
                "structure": {"name": "nested struct", "value": 32},
                "enum_iterator": [
                    "Variant1",
                    {"Variant2": {"name": "variant2 struct", "value":"the value of this struct"}},
                    {"Variant3":{"value": 64}}
                ]
            })
        );

        assert_traces!("response_json.traces");
    }

    #[test]
    fn response_json_openapi() {
        let (_, doc) = router().split_for_parts();
        check_openapi("response_json.openapi.json", &doc);
    }
}
