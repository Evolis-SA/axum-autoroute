use axum::extract::{Json, Path, Query};
use axum_autoroute::prelude::*;
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
