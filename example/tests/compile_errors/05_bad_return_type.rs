use axum_autoroute::autoroute;

#[autoroute(GET, path="/home", responses=[(200, body=String, description="response description")])]
fn missing_return() {}

#[autoroute(GET, path="/home", responses=[(200, body=String, description="response description")])]
fn bad_return() -> Test {}

fn main() {}