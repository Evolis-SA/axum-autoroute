#![allow(unused_imports)]

use axum_autoroute::autoroute;
use axum;
use utoipa;

#[autoroute(GET, path="/home", responses=[(200, body=String, unknown, description="desc")])]
fn unknown_field() {}

#[autoroute(GET, path="/home", responses=[(200, body=String, serializer=None, description="desc")])]
/// doc
fn bad_serializer_1() -> BadSerializer1Responses { todo!() }

#[autoroute(GET, path="/home", responses=[(200, body=String, serializer=Option::Some, description="desc")])]
/// doc
fn bad_serializer_2() -> BadSerializer2Responses { todo!() }

fn main() {}