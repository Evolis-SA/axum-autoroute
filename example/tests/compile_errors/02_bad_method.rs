use axum_autoroute::autoroute;

#[autoroute(UNKNOWN, path="/home")]
fn unknown_method() {}

#[autoroute(get, path="/home")]
fn lowercase() {}

#[autoroute("GET", path="/home")]
fn lowercase() {}

fn main() {}