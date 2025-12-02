//! Custom wrapper of `utoipa_axum::router::OpenApiRouter`.

use std::convert::Infallible;

use axum::Router;
use axum::extract::Request;
use axum::handler::Handler;
use axum::response::IntoResponse;
use axum::routing::Route;
use tower::{Layer, Service};
use utoipa::openapi::OpenApi;
use utoipa_axum::router::{OpenApiRouter, UtoipaMethodRouter};

/// A wrapper of `utoipa_axum::router::OpenApiRouter`
/// allowing to separate public and private (not appearing in the openapi specification) routes.
/// If unspecified, the state of the router will be the unit type.
pub struct AutorouteApiRouter<S = ()>
where
    S: Send + Sync + Clone + 'static, // axum State
{
    pub_router: OpenApiRouter<S>,
    priv_router: OpenApiRouter<S>,
}

impl<S> Default for AutorouteApiRouter<S>
where
    S: Send + Sync + Clone + 'static, // axum State
{
    fn default() -> Self {
        Self::new()
    }
}

impl<S> AutorouteApiRouter<S>
where
    S: Send + Sync + Clone + 'static, // axum State
{
    /// Pass through for `utoipa_axum::router::OpenApiRouter::new`
    #[must_use]
    pub fn new() -> Self {
        Self {
            pub_router: OpenApiRouter::with_openapi(OpenApi::default()),
            priv_router: OpenApiRouter::new(),
        }
    }

    /// Pass through for `utoipa_axum::router::OpenApiRouter::with_openapi`
    #[must_use]
    pub fn new_with_openapi(openapi: OpenApi) -> Self {
        Self {
            pub_router: OpenApiRouter::with_openapi(openapi),
            priv_router: OpenApiRouter::new(),
        }
    }

    /// Add a new public route.
    #[must_use]
    pub fn with_pub_route(mut self, method_router: UtoipaMethodRouter<S>) -> Self {
        self.pub_router = self.pub_router.routes(method_router);
        self
    }

    /// Add several new public routes.
    #[must_use]
    pub fn with_pub_routes<I>(mut self, method_routers: I) -> Self
    where
        I: IntoIterator<Item = UtoipaMethodRouter<S>>,
    {
        for method_router in method_routers {
            self.pub_router = self.pub_router.routes(method_router);
        }
        self
    }

    /// Add a new private route.
    #[must_use]
    pub fn with_priv_route(mut self, method_router: UtoipaMethodRouter<S>) -> Self {
        self.priv_router = self.priv_router.routes(method_router);
        self
    }

    /// Add several new private routes.
    #[must_use]
    pub fn with_priv_routes<I>(mut self, method_routers: I) -> Self
    where
        I: IntoIterator<Item = UtoipaMethodRouter<S>>,
    {
        for method_router in method_routers {
            self.pub_router = self.pub_router.routes(method_router);
        }
        self
    }

    /// Return an `axum::Router` containing all the routes (public and private).
    /// Also returns an instance of utoipa `OpenApi` that will include the documentation only for public routes.
    pub fn split_for_parts(self) -> (Router<S>, OpenApi) {
        let (router, doc) = self.pub_router.split_for_parts();
        let router = router.merge(self.priv_router);
        (router, doc)
    }

    /// Same as `split_for_parts` but also including private routes in the documentation
    pub fn split_for_parts_with_private_doc(self) -> (Router<S>, OpenApi) {
        let merged_router = self.pub_router.merge(self.priv_router);
        merged_router.split_for_parts()
    }

    /// Pass through for `utoipa_axum::router::OpenApiRouter::nest`
    #[must_use]
    pub fn nest(mut self, path: &str, router: Self) -> Self {
        self.pub_router = self.pub_router.nest(path, router.pub_router);
        self.priv_router = self.priv_router.nest(path, router.priv_router);
        self
    }

    /// Pass through for `utoipa_axum::router::OpenApiRouter::merge`
    #[must_use]
    pub fn merge(mut self, router: Self) -> Self {
        self.pub_router = self.pub_router.merge(router.pub_router);
        self.priv_router = self.priv_router.merge(router.priv_router);
        self
    }

    /// Pass through for `utoipa_axum::router::OpenApiRouter::fallback`
    #[must_use]
    pub fn fallback<H, T>(mut self, handler: H) -> Self
    where
        H: Handler<T, S>,
        T: 'static,
    {
        // apply fallback only on public router
        // else it is impossible to merge the public and private routers together because both will have a fallback
        self.pub_router = self.pub_router.fallback(handler);
        self
    }

    /// Pass through for `utoipa_axum::router::OpenApiRouter::layer`
    #[must_use]
    pub fn layer<L>(mut self, layer: L) -> Self
    where
        L: Layer<Route> + Clone + Send + Sync + 'static,
        L::Service: Service<Request> + Clone + Send + Sync + 'static,
        <L::Service as Service<Request>>::Response: IntoResponse + 'static,
        <L::Service as Service<Request>>::Error: Into<Infallible> + 'static,
        <L::Service as Service<Request>>::Future: Send + 'static,
    {
        self.pub_router = self.pub_router.layer(layer.clone());
        self.priv_router = self.priv_router.layer(layer);
        self
    }

    /// Pass through for `utoipa_axum::router::OpenApiRouter::with_state`
    #[must_use]
    pub fn with_state<S2>(self, state: S) -> AutorouteApiRouter<S2>
    where
        S2: Send + Sync + Clone + 'static, // resulting axum State
    {
        AutorouteApiRouter {
            pub_router: self.pub_router.with_state(state.clone()),
            priv_router: self.priv_router.with_state(state),
        }
    }

    /// Apply the provided modifier to the openapi documentation
    #[must_use]
    pub fn modify_openapi<M>(mut self, modifier: &M) -> Self
    where
        M: utoipa::Modify,
    {
        modifier.modify(self.pub_router.get_openapi_mut());
        modifier.modify(self.priv_router.get_openapi_mut());
        self
    }
}
