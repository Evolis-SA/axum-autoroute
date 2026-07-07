use axum_autoroute::{autoroute, route_info, routes_info, RouteInfo};

/// This route always says hello.
#[autoroute(GET, path="/hello", tags=["hello", "world"],
    responses=[
        (IM_A_TEAPOT, body=String, serializer=NONE, description="Always says hello"),
    ]
)]
async fn hello_route() -> HelloRouteResponses {
    "Hello World!".to_string().into_im_a_teapot()
}

fn main() {
    // ok
    let _: &RouteInfo = route_info!(hello_route);
    let _: &'static RouteInfo = route_info!(hello_route);
    let _: &[&RouteInfo] = routes_info!(hello_route);
    let _: &[&'static RouteInfo] = routes_info!(hello_route);

    // failures
    let _: RouteInfo = route_info!(hello_route);
    let _: &[RouteInfo] = routes_info!(hello_route);
    let _: [&RouteInfo] = routes_info!(hello_route);
    let _: [RouteInfo] = routes_info!(hello_route);
}
