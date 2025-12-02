use axum_autoroute::autoroute;
use axum;
use utoipa;

#[autoroute(GET, path="/home", responses=[(200, body=Test, description="desc")])]
/// doc
fn unknown_body_type() -> UnknownBodyTypeResponses { todo!() }

#[autoroute(GET, path="/home", responses=[(200, body=String, unknown, description="desc")])]
fn unknown_field() {}

#[autoroute(GET, path="/home", responses=[(200, body=String, serializer=Test, description="desc")])]
/// doc
fn bad_serializer_1() -> BadSerializer1Responses { todo!() }

#[autoroute(GET, path="/home", responses=[(200, body=String, serializer=None, description="desc")])]
/// doc
fn bad_serializer_2() -> BadSerializer2Responses { todo!() }

#[autoroute(GET, path="/home", responses=[(200, body=String, serializer=Option::Some, description="desc")])]
/// doc
fn bad_serializer_3() -> BadSerializer3Responses { todo!() }

fn main() {}