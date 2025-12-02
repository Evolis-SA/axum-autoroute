use axum_autoroute::autoroute;

#[derive(Debug, axum::extract::FromRequest)]
#[from_request(via(axum::extract::Json))]
struct CustomJsonExtractor<T>(T);

#[derive(Debug, axum::extract::FromRequestParts)]
#[from_request(via(axum::extract::Query))]
struct CustomQueryExtractor<T>(T);

#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
struct MyQueryStruct {
    num1: i32,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
struct MyJsonStruct {
    txt: String,
}

#[autoroute(POST, path="/test", responses=[(200, body=String, serializer=NONE, description="desc")])]
fn content_type_on_known_extractor(#[extractor(content_type="application/json")] query: Query<MyQuery>) -> ContentTypeOnKnownExtractorResponses {}

#[autoroute(POST, path="/test", responses=[(200, body=String, serializer=NONE, description="desc")])]
fn parts_and_body_extractor(#[extractor(content_type=APPLICATION_JSON, into_params=true)] json: CustomJsonExtractor<MyJsonStruct>) -> ContentTypeOnKnownExtractorResponses {}

#[autoroute(POST, path="/test", responses=[(200, body=String, serializer=NONE, description="desc")])]
fn invalid_content_type(#[extractor(content_type=[], into_params=true)] json: CustomJsonExtractor<MyJsonStruct>) -> ContentTypeOnKnownExtractorResponses {}

fn main() {}
