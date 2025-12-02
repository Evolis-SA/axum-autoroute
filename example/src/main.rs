use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::{env, fs};

use axum::Router;
use axum_autoroute_example::app;
use tokio::signal;
use utoipa::openapi::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() {
    #[cfg(feature = "tracing")]
    init_tracing();

    // create the listener
    let listener = tokio::net::TcpListener::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9100))
        .await
        .expect("failed to start listener");
    let listener_addr = listener.local_addr().unwrap();
    println!("server listening at {listener_addr}");

    // init router
    let (router, doc) = app().split_for_parts();

    // save doc
    save_openapi_doc(&doc);

    // add swagger ui
    // done after the middlewares so that they are not applied here
    let router = router.merge(serve_swagger_ui(doc, &listener_addr));

    // start server
    axum::serve(listener, router.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("failed to start server");

    println!("server stopped");
}

fn save_openapi_doc(doc: &OpenApi) {
    let openapi_filepath = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("openapi.json");
    let doc = doc.to_pretty_json().expect("failed to generate openapi documentation");

    fs::write(&openapi_filepath, doc).expect("failed to write openapi documentation to file");
    println!(
        "openapi documentation generated here: '{}'",
        openapi_filepath.to_string_lossy()
    );
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => { println!("SIGINT triggered !") },
        () = terminate => { println!("SIGTERM triggered !") },
    }

    println!("starting graceful shutdown");
}

fn serve_swagger_ui(doc: OpenApi, addr: &SocketAddr) -> Router {
    let swagger_route = "/swagger-ui".to_string();
    let openapi_route = "/openapi.json".to_string();
    println!("serve swagger-ui at route 'http://{addr}{swagger_route}'");
    Router::new().merge(SwaggerUi::new(swagger_route).url(openapi_route, doc))
}

#[cfg(feature = "tracing")]
fn init_tracing() {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    println!("init tracing");
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();
}
