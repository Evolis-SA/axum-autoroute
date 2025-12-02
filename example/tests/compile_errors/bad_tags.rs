use axum_autoroute::autoroute;

#[autoroute(GET, path="/home", tags, responses=[(200, body=Test, description="desc")])]
fn bad_tags_1() {}

#[autoroute(GET, path="/home", tags=, responses=[(200, body=Test, description="desc")])]
fn bad_tags_2() {}

#[autoroute(GET, path="/home", tags=[27], responses=[(200, body=Test, description="desc")])]
fn bad_tags_3() {}

#[autoroute(GET, path="/home", tags=["a" "b"], responses=[(200, body=Test, description="desc")])]
fn bad_tags_4() {}

fn main() {}