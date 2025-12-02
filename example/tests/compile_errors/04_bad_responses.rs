use axum_autoroute::autoroute;

#[autoroute(GET, path="/home", responses=[()])]
fn missing_status() {}

#[autoroute(GET, path="/home", responses=[(10)])]
fn bad_status_1() {}

#[autoroute(GET, path="/home", responses=[(NotFound)])]
fn bad_status_2() {}

#[autoroute(GET, path="/home", responses=[(StatusCode::NOT_FOUND)])]
fn bad_status_3() {}

#[autoroute(GET, path="/home", responses=[("200")])]
fn bad_status_4() {}

#[autoroute(GET, path="/home", responses=[(200)])]
fn missing_ty_or_filename_1() {}

#[autoroute(GET, path="/home", responses=[(200,)])]
fn missing_ty_or_filename_2() {}

#[autoroute(GET, path="/home", responses=[(200, body)])]
fn bad_ty_1() {}

#[autoroute(GET, path="/home", responses=[(200, body=)])]
fn bad_ty_2() {}

#[autoroute(GET, path="/home", responses=[(200, body=String, description)])]
fn bad_description_1() {}

#[autoroute(GET, path="/home", responses=[(200, body=String, description=)])]
fn bad_description_2() {}

#[autoroute(GET, path="/home", responses=[(200, body=String, description=""), ()])]
fn invalid_second_response() {}

#[autoroute(GET, path="/home", responses=[
    (200, body=String, description="desc")
    (NOT_FOUND, body=usize, description="desc")
])]
fn missing_comma() {}

#[autoroute(GET, path="/home", responses=[
    (200, body=String, description="desc"),
    (OK, body=usize, description="desc"),
])]
/// doc
fn duplicated_status() -> DuplicatedStatusResponses { todo!() }

fn main() {}