use axum::extract::Query;
use axum_autoroute::status_trait::IntoOk;
use axum_autoroute::{AutorouteApiRouter, autoroute, method_routers};
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::Cookie;
use serde::Deserialize;
use utoipa::IntoParams;

pub fn router() -> AutorouteApiRouter {
    AutorouteApiRouter::new().with_pub_routes(method_routers!(response_cookie))
}

#[derive(Debug, Deserialize, IntoParams)]
struct QueryParam {
    cookie_name: String,
    cookie_value: u32,
}

/// Retrieves cookie parameters from query and ask the browser to set the cookie
#[autoroute(GET, path="/response/cookie",
    responses=[
        (OK, body=(CookieJar, String), serializer=NONE, headers=[(SET_COOKIE, description="set the provided cookie")], description="Set the cookie into the browser"),
    ],
    tags=["response"],
)]
async fn response_cookie(Query(query): Query<QueryParam>, cookie_jar: CookieJar) -> ResponseCookieResponses {
    let previous_cookie = cookie_jar.get(&query.cookie_name);
    let new_cookie = Cookie::new(query.cookie_name, query.cookie_value.to_string());
    (
        CookieJar::new().add(new_cookie.clone()),
        format!(
            "previous_cookie={:?}, new_cookie={:?}",
            previous_cookie.map(Cookie::name_value),
            new_cookie.name_value()
        ),
    )
        .into_200()
}

#[cfg(test)]
mod test {
    use axum::body::Body;
    use axum::extract::Request;
    use axum::http::header::{COOKIE, SET_COOKIE};
    use axum::http::{HeaderValue, Method, StatusCode};
    use tower::Service;

    use super::router;
    use crate::test_utils::*;

    #[tokio::test]
    #[cfg_attr(feature = "tracing", tracing_test::traced_test)]
    async fn response_cookie() {
        let (mut router, _) = router().split_for_parts();
        let service = build_service(&mut router).await;

        let response = service
            .call(
                Request::builder()
                    .method(Method::GET)
                    .uri("/response/cookie?cookie_name=test&cookie_value=2")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let cookie = response.headers().get(SET_COOKIE).cloned();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(cookie, Some(HeaderValue::from_str("test=2").unwrap()));
        assert_eq!(
            response_to_str(response).await,
            r#"previous_cookie=None, new_cookie=("test", "2")"#
        );

        let response = service
            .call(
                Request::builder()
                    .method(Method::GET)
                    .uri("/response/cookie?cookie_name=test&cookie_value=7")
                    .header(COOKIE, cookie.unwrap())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let cookie = response.headers().get(SET_COOKIE).cloned();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(cookie, Some(HeaderValue::from_str("test=7").unwrap()));
        assert_eq!(
            response_to_str(response).await,
            r#"previous_cookie=Some(("test", "2")), new_cookie=("test", "7")"#
        );

        assert_traces!("response_cookie.traces");
    }

    #[test]
    fn response_cookie_openapi() {
        let (_, doc) = router().split_for_parts();
        check_openapi("response_cookie.openapi.json", &doc);
    }
}
