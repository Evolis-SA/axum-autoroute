# Axum Autoroute

The goal of this crate is to integrate [utoipa](https://docs.rs/utoipa/latest/utoipa/) tightly with [axum](https://docs.rs/axum/latest/axum/) to enforce that for each REST route, the code and the openapi documentation are matching.

<center><img src="../axum_autoroute.png" alt="" width="200"></center>

## Features

* Automatic detection of many axum extractors (`Path`, `Query`, `Json`, `TypedMultipart` etc.) from the function signature.
    * Detected extractors will be added to the openapi specification.
* Strict route responses.
    * An enum will be automatically generated from the route declared responses and will be enforced as the return type of the function. This ensures that the responses returned by the handler function are matching with the ones declared in the openapi specification.
* `AutorouteApiRouter`, an axum router keeping track of the openapi documentation and allowing the distinction between public and private (not documented in the openapi specification) routes.
    * Based on `utoipa_axum` [OpenApiRouter](https://docs.rs/utoipa-axum/latest/utoipa_axum/router/struct.OpenApiRouter.html).

## Example

<!--
DO NOT UPDATE THIS EXAMPLE DIRECTLY.
Instead, you should update the main_example.rs file, run the format, test and clippy scripts
and then copy the fixed content in here.
-->

```rust
use axum::extract::{Json, Path, Query};
use axum_autoroute::status_trait::IntoBadRequest;
use axum_autoroute::{AutorouteApiRouter, autoroute, method_routers};

pub fn router() -> AutorouteApiRouter {
    AutorouteApiRouter::new().with_pub_routes(method_routers!(my_route))
}

#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
/// Data to extract from the url path
struct PathParam {
    id: u8,
}

#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
/// Data to extract from the url query
struct QueryParam {
    text1: String,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
/// Data to extract from the json request body
struct JsonRequest {
    text2: String,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
/// Data to send in the response json body
struct JsonResponse {
    id: u8,
    text1: String,
    text2: String,
}

/// This route uses several different axum extractors.
/// It replies either:
/// * An HTTP code 400 (bad request) with an error message if the id is odd.
/// * An HTTP code 200 (ok) with a json structure summarizing all the extracted data if the id is even.
#[autoroute(GET, path="/my/route/{id}",
    responses=[
        (OK, body=JsonResponse, description="The id is even, return the extracted data."),
        (BAD_REQUEST, body=String, serializer=NONE, description="The id is odd, return an error message."),
    ]
)]
async fn my_route(
    Path(path): Path<PathParam>,     // path extraction
    Query(query): Query<QueryParam>, // query extraction
    Json(json): Json<JsonRequest>,   // json body extraction
) -> MyRouteResponses {
    if path.id % 2 == 0 {
        let resp = JsonResponse {
            id: path.id,
            text1: query.text1,
            text2: json.text2,
        };
        // a response can either be returned by using directly the variant of the generated enum
        MyRouteResponses::Ok(resp)
    } else {
        // or by using traits exposed by axum_autoroute for each HTTP return code
        // (here `IntoBadRequest` which exposes the functions `into_bad_request` and `into_400`)
        format!("The provided id ({}) is odd.", path.id).into_bad_request()
    }
}
```

See the [axum-autoroute-example](https://github.com/Evolis-SA/axum-autoroute/tree/main/example) crate for more samples.
