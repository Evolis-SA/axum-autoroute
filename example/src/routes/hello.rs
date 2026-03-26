use axum_autoroute::{AutorouteApiRouter, autoroute, method_routers};
use utoipa::OpenApi;

use crate::OpenApiDoc;

pub fn router() -> AutorouteApiRouter {
    AutorouteApiRouter::new_with_openapi(OpenApiDoc::openapi())
        .with_pub_routes(method_routers!(hello_route, no_description))
        .with_priv_routes(method_routers!(bonjour_secret_route, bye_secret_route))
}

/// This route always says hello.
#[autoroute(GET, path="/hello", tags=["hello", "world"],
    responses=[
        (IM_A_TEAPOT, body=String, serializer=NONE, description="Always says hello"),
    ]
)]
async fn hello_route() -> HelloRouteResponses {
    "Hello World!".to_string().into_im_a_teapot()
}

#[autoroute(GET, path="/no/description", tags=["hello", "world"],
    responses=[
        (IM_A_TEAPOT, body=String, serializer=NONE),
    ]
)]
async fn no_description() -> NoDescriptionResponses {
    "Hello World!".to_string().into_im_a_teapot()
}

/// This route is hidden.
#[autoroute(GET, path="/secret/bonjour", tags=["bonjour", "secret"],
    responses=[
        (IM_A_TEAPOT, body=String, serializer=NONE, description="Always says bonjour"),
    ]
)]
async fn bonjour_secret_route() -> BonjourSecretRouteResponses {
    "Bonjour World !".to_string().into_im_a_teapot()
}

/// This route is hidden.
#[autoroute(GET, path="/secret/bye", tags=["bye", "secret"],
    responses=[
        (IM_A_TEAPOT, body=String, serializer=NONE, description="Always says bye"),
    ]
)]
async fn bye_secret_route() -> ByeSecretRouteResponses {
    "Bye World!".to_string().into_im_a_teapot()
}

#[cfg(test)]
mod test {
    use axum::http::{Method, StatusCode};
    use tower::ServiceExt;

    use super::router;
    use crate::test_utils::*;

    #[tokio::test]
    #[cfg_attr(feature = "tracing", tracing_test::traced_test)]
    async fn hello_route() {
        let (router, _) = router().split_for_parts();
        let response = router.oneshot(request_empty(Method::GET, "/hello")).await.unwrap();

        assert_eq!(response.status(), StatusCode::IM_A_TEAPOT);
        assert_eq!(response_to_str(response).await, "Hello World!");

        assert_traces!("hello.traces");
    }

    #[test]
    fn hello_openapi() {
        let (_, doc) = router().split_for_parts();
        check_openapi("hello.openapi.json", &doc);
        let (_, doc) = router().split_for_parts_with_private_doc();
        check_openapi("hello.openapi.private.json", &doc);
    }
}
