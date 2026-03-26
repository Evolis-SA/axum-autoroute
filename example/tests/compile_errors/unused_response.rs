#![deny(unused_imports)] // make it crash on warnings

use axum_autoroute::autoroute;

#[autoroute(GET, path="/home", responses=[
    (NOT_FOUND, body=String, description="desc"),
    (OK, body=String, description="desc"),
])]
/// doc
fn unused_response_type() -> UnusedResponseTypeResponses { 
    "It's always found".to_string().into_ok()
}

fn main() {}