use std::fmt::Debug;
use std::fs;
use std::path::PathBuf;
use std::str::from_utf8;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::extract::Request;
use axum::http::{Method, header};
use axum::response::Response;
use serde_json::Value;
use tower::{Service, ServiceExt};
use utoipa::openapi::OpenApi;

pub async fn build_service<S>(router: &mut Router<S>) -> &mut Router<S>
where
    S: Clone + Send + Sync,
    Router<S>: Service<Request>,
    <Router<S> as Service<Request>>::Error: Debug,
{
    ServiceExt::<Request<Body>>::ready(router).await.unwrap()
}

pub fn request_json(method: Method, uri: &str, json: &Value) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json.to_string()))
        .unwrap()
}

pub fn request_empty(method: Method, uri: &str) -> Request {
    Request::builder().method(method).uri(uri).body(Body::empty()).unwrap()
}

pub async fn response_to_str(response: Response) -> String {
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    from_utf8(&body).unwrap().to_string()
}

pub async fn response_to_json(response: Response) -> Value {
    let axum::Json(value) = axum::Json::from_bytes(to_bytes(response.into_body(), usize::MAX).await.unwrap().as_ref())
        .expect("failed to deserialize as json");
    value
}

pub fn check_openapi(ref_filename: &str, openapi: &OpenApi) {
    let content = openapi
        .to_pretty_json()
        .expect("failed to generated openapi json specification");

    let ref_path = openapi_ref_dir().join(ref_filename);
    println!("loading reference openapi file: {ref_path:?}");

    let expected_content = fs::read_to_string(&ref_path);
    if expected_content.is_err() {
        save_wip_content(ref_filename, &content).unwrap();
    }
    let expected_content =
        expected_content.unwrap_or_else(|_| panic!("failed to read reference openapi file {ref_path:?}"));

    if expected_content.lines().ne(content.lines()) {
        println!("found delta in openapi spec");
        save_wip_content(ref_filename, &content).unwrap();
        panic!("unexpected openapi content");
    }
}

macro_rules! assert_traces {
    ($ref_filename:literal) => {
        #[cfg(feature = "tracing")]
        {
            logs_assert(move |lines| {
                let lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
                check_traces($ref_filename, &lines)
            });
        }
    };
}

pub(crate) use assert_traces;

#[cfg(feature = "tracing")]
pub fn check_traces(ref_filename: &str, lines: &Vec<String>) -> Result<(), String> {
    let content = lines
        .iter()
        .map(|s| strip_trace_datetime(s).to_string())
        .collect::<Vec<_>>()
        .join("\n");

    let ref_path = traces_ref_dir().join(ref_filename);
    println!("loading reference tracing file: {ref_path:?}");

    let expected_content = fs::read_to_string(&ref_path);
    if expected_content.is_err() {
        save_wip_content(ref_filename, &content)?;
    }

    let expected_content =
        expected_content.map_err(|e| format!("failed to read reference tracing file {ref_path:?}: {e}"))?;

    if expected_content.lines().ne(content.lines()) {
        println!("found delta in traces");
        save_wip_content(ref_filename, &content)?;
        return Err("unexpected traces content".to_string());
    }

    Ok(())
}

#[cfg(feature = "tracing")]
fn strip_trace_datetime(line: &str) -> &str {
    use std::sync::LazyLock;

    use regex::Regex;

    static TRACE_DATETIME_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new("^[0-9]+-[0-9]+-[0-9]+T[0-9]+:[0-9]+:[0-9]+.[0-9]+Z +").unwrap());
    if let Some(datetime_match) = TRACE_DATETIME_REGEX.find(line) {
        &line[datetime_match.end()..]
    } else {
        line
    }
}

fn current_dir() -> PathBuf {
    std::env::current_dir().expect("failed to retrieve current working directory")
}

fn openapi_ref_dir() -> PathBuf {
    current_dir().join("refs").join("openapi")
}

#[cfg(feature = "tracing")]
fn traces_ref_dir() -> PathBuf {
    current_dir().join("refs").join("tracing")
}

fn wip_dir() -> PathBuf {
    current_dir().join("wip")
}

fn save_wip_content(ref_filename: &str, content: &str) -> Result<(), String> {
    let wip_dir = wip_dir();
    let gitignore = wip_dir.join(".gitignore");
    let wip_content = wip_dir.join(ref_filename);

    if !wip_dir.exists() {
        if let Err(e) = fs::create_dir(wip_dir.clone()) {
            return Err(format!("failed to create directory {wip_dir:?}: {e}"));
        }
    }

    if let Err(e) = fs::write(gitignore.clone(), "*") {
        return Err(format!("failed to create and write {wip_dir:?}: {e}"));
    }
    if let Err(e) = fs::write(wip_content.clone(), content) {
        return Err(format!("failed to create and write {wip_dir:?}: {e}"));
    }
    println!("reference file written at {wip_content:?}");
    Ok(())
}
