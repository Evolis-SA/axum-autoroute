use axum::http::Method;

/// A structure holding information about a route handler (namely its method and path)
/// A new instance of this struct will be implemented by each [`autoroute`](crate::autoroute) handler.
pub struct RouteInfo {
    method: Method,
    path: &'static str,
}

impl RouteInfo {
    /// Create a new `RouteInfo`.
    #[must_use]
    pub const fn new(method: Method, path: &'static str) -> Self {
        Self { method, path }
    }

    /// Get the HTTP method handled.
    #[must_use]
    pub fn method(&self) -> &Method {
        &self.method
    }

    /// Get the uri path handled.
    #[must_use]
    pub fn path(&self) -> &'static str {
        self.path
    }
}
