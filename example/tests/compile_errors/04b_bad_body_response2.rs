#![allow(unused_imports)]

use axum_autoroute::autoroute;
use axum;
use utoipa;

#[autoroute(GET, path="/home", responses=[(200, body=UnknownType, description="desc")])]
/// doc
fn unknown_body_type() -> UnknownBodyTypeResponses { todo!() }

#[autoroute(GET, path="/home", responses=[(200, body=String, serializer=UnknownType, description="desc")])]
/// doc
fn unknown_serializer() -> UnknownSerializerResponses { todo!() }

fn main() {}