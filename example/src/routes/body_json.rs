use axum::Json;
use axum_autoroute::prelude::*;
use axum_autoroute::{AutorouteApiRouter, autoroute, method_routers};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub fn router() -> AutorouteApiRouter {
    AutorouteApiRouter::new().with_pub_routes(method_routers!(body_json1, body_json2))
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
/// The json body type
struct MyBodyJson {
    /// A set of different values
    collection: Vec<MyEnum>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
/// documentation of the enum
enum MyEnum {
    /// documentation of the first variant
    Variant1,
    /// documentation of the second variant
    Variant2(
        /// documentation of the second variant field
        MyStruct<String>,
    ),
    /// documentation of the third variant
    Variant3 {
        /// documentation of the third variant field
        value: u32,
    },
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
/// this is a struct with a generic type
struct MyStruct<T> {
    /// name of the value
    name: String,
    /// the value itself
    value: T,
}

/// Parse and return the provided body
#[autoroute(POST, path="/body/json/1", tags=["body"],
    responses=[
        (200, body=MyBodyJson, description="Returns the received body"),
    ]
)]
async fn body_json1(Json(json): Json<MyBodyJson>) -> BodyJson1Responses {
    json.into_ok()
}

/// Same as above, but the tracing will be different if the feature is activated because the json extractor is not destructured
#[autoroute(POST, path="/body/json/2", tags=["body"],
    responses=[
        (200, body=MyBodyJson, description="Returns the received body"),
    ]
)]
async fn body_json2(json: Json<MyBodyJson>) -> BodyJson2Responses {
    json.0.into_ok()
}

#[cfg(test)]
mod test {
    use axum::http::{Method, StatusCode};
    use serde_json::{Value, json};
    use tower::Service;

    use super::router;
    use crate::routes::body_json::{MyBodyJson, MyEnum, MyStruct};
    use crate::test_utils::*;

    #[tokio::test]
    #[cfg_attr(feature = "tracing", tracing_test::traced_test)]
    async fn body_json() {
        let (mut router, _) = router().split_for_parts();
        let service = build_service(&mut router).await;

        let json: Value = json!(MyBodyJson {
            collection: vec![
                MyEnum::Variant1,
                MyEnum::Variant2(MyStruct {
                    name: "test".to_string(),
                    value: "value".to_string()
                }),
                MyEnum::Variant3 { value: 32 }
            ]
        });
        let response = service
            .call(request_json(Method::POST, "/body/json/1", &json))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response_to_json(response).await, json);

        assert_traces!("body_json.traces");
    }

    #[test]
    fn body_json_openapi() {
        let (_, doc) = router().split_for_parts();
        check_openapi("body_json.openapi.json", &doc);
    }
}
