use std::sync::Arc;
use std::sync::atomic::Ordering;

use axum::extract::State;
use axum_autoroute::status_trait::IntoOk;
use axum_autoroute::{AutorouteApiRouter, autoroute, method_router};

use crate::ApiState;

pub fn router() -> AutorouteApiRouter<Arc<ApiState>> {
    AutorouteApiRouter::new().with_pub_route(method_router!(state_incr))
}

/// Increment a state counter
#[autoroute(GET, path="/state/incr", tags=["extensions"],
    responses=[
        (OK, body=u8, description="Return the previous value of the state counter"),
    ]
)]
async fn state_incr(State(state): State<Arc<ApiState>>) -> StateIncrResponses {
    let prev = state.counter.fetch_add(1, Ordering::Relaxed);
    prev.into_ok()
}

#[cfg(test)]
mod test {
    use axum::http::{Method, StatusCode};
    use tower::Service;

    use super::router;
    use crate::ApiState;
    use crate::test_utils::*;

    #[tokio::test]
    #[cfg_attr(feature = "tracing", tracing_test::traced_test)]
    async fn state_incr() {
        let (router, _) = router().split_for_parts();
        let mut router = router.with_state(ApiState::new());
        let service = build_service(&mut router).await;

        let response = service.call(request_empty(Method::GET, "/state/incr")).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response_to_str(response).await, "0");

        let response = service.call(request_empty(Method::GET, "/state/incr")).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response_to_str(response).await, "1");

        assert_traces!("state_incr.traces");
    }

    #[test]
    fn state_openapi() {
        let (_, doc) = router().split_for_parts();
        check_openapi("state.openapi.json", &doc);
    }
}
