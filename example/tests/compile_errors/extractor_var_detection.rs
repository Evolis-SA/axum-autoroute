use axum_autoroute::{autoroute};

#[derive(serde::Serialize, serde::Deserialize)]
struct MyQuery(i32);

struct MyExtractor {
    text: String,
    num: i32
}

#[autoroute(POST, path="/test", responses=[(200, body=String, serializer=NONE, description="desc")])]
fn unable_to_detect_extractor_variable(_: Query<MyQuery>) -> UnableToDetectExtractorVariableResponses {}

#[autoroute(POST, path="/test", responses=[(200, body=String, serializer=NONE, description="desc")])]
fn unable_to_detect_extractor_variable(Query(_): Query<MyQuery>) -> E1RespoUnableToDetectExtractorVariableResponsesnses {}

#[autoroute(POST, path="/test", responses=[(200, body=String, serializer=NONE, description="desc")])]
fn unable_to_detect_extractor_variable(Query(MyQuery(_var)): Query<MyQuery>) -> UnableToDetectExtractorVariableResponses {}

#[autoroute(POST, path="/test", responses=[(200, body=String, serializer=NONE, description="desc")])]
fn unable_to_detect_extractor_variable(MyExtractor{_text, _num}: MyExtractor) -> UnableToDetectExtractorVariableResponses {}

fn main() {}
