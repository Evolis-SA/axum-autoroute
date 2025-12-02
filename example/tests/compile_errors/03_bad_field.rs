use axum_autoroute::autoroute;

#[autoroute]
fn no_param() {}

#[autoroute()]
fn no_param2() {}

#[autoroute(GET)]
fn missing_path() {}

#[autoroute(path="/home")]
fn missing_method() {}

#[autoroute(GET path)]
fn missing_comma() {}

#[autoroute(GET, path)]
fn missing_path_eq() {}

#[autoroute(GET, path=)]
fn missing_path_value() {}

#[autoroute(GET, path="/home")]
fn missing_responses_1() {}

#[autoroute(GET, path="/home", responses)]
fn missing_responses_2() {}

#[autoroute(GET, path="/home", responses[])]
fn missing_responses_3() {}

#[autoroute(GET, path="/home", responses=)]
fn missing_responses_4() {}

#[autoroute(GET, path="/home", responses=[])]
fn missing_responses_5() {}

#[autoroute(GET, path="/home", unknown="test")]
fn unknown_field() {}

fn main() {}