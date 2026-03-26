#![warn(missing_docs)]
#![doc(html_logo_url = "https://raw.githubusercontent.com/Evolis-SA/axum-autoroute/refs/heads/main/axum_autoroute.png")]
#![doc = include_str!("../README.md")]
//!
//! # Crate features
//! * `debugging`: Enables the [`autoroute_debug`] macro.
//! * `tracing`: Enables automatic tracing of input/output parameters of the handlers.
//!    * The parameters will be displayed using their `Debug` implementation.
//! * `default_serializer_json` (default): If enabled, the default serializer for autoroute responses will be [`Json`](axum::extract::Json).
//! * `unstable_extractor_attr`: Enables some unstable extractor attribute fields.

#[cfg(feature = "debugging")]
pub use axum_autoroute_macros::autoroute_debug;
pub use axum_autoroute_macros::{autoroute, method_router, method_routers, route_info, routes_info};
pub use route_info::RouteInfo;
pub use router::AutorouteApiRouter;

pub mod response;
mod route_info;
mod router;
pub mod status_trait;
