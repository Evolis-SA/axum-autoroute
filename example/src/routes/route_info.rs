use axum::extract::Path;
use axum_autoroute::status_trait::IntoOk;
use axum_autoroute::{AutorouteApiRouter, autoroute, method_routers, route_info};
use serde::Deserialize;
use utoipa::IntoParams;

pub fn router() -> AutorouteApiRouter {
    AutorouteApiRouter::new().with_pub_routes(method_routers!(route_1, route_2, route_3))
}

#[autoroute(GET, path="/route/1", tags=["info"],
    responses=[
        (OK, body=String, serializer=NONE),
    ]
)]
pub async fn route_1() -> Route1Responses {
    let info = route_info!(route_1);
    format!("You called {}:{}", info.method(), info.path()).into_ok()
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct PathParam {
    p: String,
}

#[autoroute(GET, path="/route/{p}", tags=["info"],
    responses=[
        (OK, body=String, serializer=NONE),
    ]
)]
pub async fn route_2(Path(param): Path<PathParam>) -> Route2Responses {
    let info = route_info!(route_2);
    format!("You called {}:{} with {}", info.method(), info.path(), param.p).into_ok()
}

#[autoroute(POST, path="/route/{p}", tags=["info"],
    responses=[
        (OK, body=String, serializer=NONE),
    ]
)]
pub async fn route_3(Path(param): Path<PathParam>) -> Route3Responses {
    let info = route_info!(route_3);
    format!("You called {}:{} with {}", info.method(), info.path(), param.p).into_ok()
}

#[cfg(test)]
mod test {
    use axum::http::{Method, StatusCode};
    use axum_autoroute::route_info;
    use tower::Service;

    use super::router;
    use crate::test_utils::*;

    #[tokio::test]
    #[cfg_attr(feature = "tracing", tracing_test::traced_test)]
    async fn route_info1() {
        let (mut router, _) = router().split_for_parts();
        let service = build_service(&mut router).await;

        let response = service.call(request_empty(Method::GET, "/route/1")).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response_to_str(response).await, "You called GET:/route/1");

        let response = service.call(request_empty(Method::DELETE, "/route/1")).await.unwrap();
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);

        assert_traces!("route_info1.traces");
    }

    #[tokio::test]
    #[cfg_attr(feature = "tracing", tracing_test::traced_test)]
    async fn route_info2() {
        let (mut router, _) = router().split_for_parts();
        let service = build_service(&mut router).await;

        let response = service.call(request_empty(Method::GET, "/route/2")).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response_to_str(response).await, "You called GET:/route/{p} with 2");

        let response = service.call(request_empty(Method::GET, "/route/57")).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response_to_str(response).await, "You called GET:/route/{p} with 57");

        let response = service.call(request_empty(Method::GET, "/route/test")).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response_to_str(response).await, "You called GET:/route/{p} with test");

        assert_traces!("route_info2.traces");
    }

    #[tokio::test]
    #[cfg_attr(feature = "tracing", tracing_test::traced_test)]
    async fn route_info3() {
        let (mut router, _) = router().split_for_parts();
        let service = build_service(&mut router).await;

        let response = service.call(request_empty(Method::POST, "/route/1")).await.unwrap();
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);

        let response = service.call(request_empty(Method::POST, "/route/57")).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response_to_str(response).await, "You called POST:/route/{p} with 57");

        let response = service.call(request_empty(Method::POST, "/route/test")).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response_to_str(response).await, "You called POST:/route/{p} with test");

        assert_traces!("route_info3.traces");
    }

    #[test]
    fn get_info() {
        let info1 = route_info!(super::route_1);
        assert_eq!(info1.method(), Method::GET);
        assert_eq!(info1.path(), "/route/1");

        use super::ROUTE_2_ROUTE_INFO;
        let info2 = route_info!(route_2);
        assert_eq!(info2.method(), Method::GET);
        assert_eq!(info2.path(), "/route/{p}");

        let info3 = route_info!(crate::route_info::route_3);
        assert_eq!(info3.method(), Method::POST);
        assert_eq!(info3.path(), "/route/{p}");
    }
}
