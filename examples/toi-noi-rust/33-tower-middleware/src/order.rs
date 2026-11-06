//! Thứ tự layer: cùng ba lớp A, B, C nhưng hai cách ghép cho hai thứ tự khác nhau.

use std::sync::{Arc, Mutex};

use axum::Router;
use axum::extract::{Request, State};
use axum::middleware::{Next, from_fn_with_state};
use axum::response::Response;
use axum::routing::get;
use tower::ServiceBuilder;

/// Nhật ký dùng chung: mỗi lớp ghi `X:in` khi request đi vào và `X:out` khi response đi ra.
pub type Log = Arc<Mutex<Vec<String>>>;

#[derive(Clone)]
struct Probe {
    name: &'static str,
    log: Log,
}

impl Probe {
    fn new(name: &'static str, log: &Log) -> Probe {
        Probe {
            name,
            log: log.clone(),
        }
    }
    fn note(&self, what: &str) {
        self.log
            .lock()
            .expect("log")
            .push(format!("{}:{what}", self.name));
    }
}

async fn probe(State(p): State<Probe>, req: Request, next: Next) -> Response {
    p.note("in");
    let response = next.run(req).await;
    p.note("out");
    response
}

fn handler_router(log: &Log) -> Router {
    let log = log.clone();
    Router::new().route(
        "/",
        get(move || async move {
            log.lock().expect("log").push("handler".into());
            "ok"
        }),
    )
}

/// `ServiceBuilder`: lớp viết trước là lớp ngoài cùng. Kỳ vọng `A:in B:in C:in handler C:out B:out A:out`.
pub fn by_service_builder(log: &Log) -> Router {
    handler_router(log).layer(
        ServiceBuilder::new()
            .layer(from_fn_with_state(Probe::new("A", log), probe))
            .layer(from_fn_with_state(Probe::new("B", log), probe))
            .layer(from_fn_with_state(Probe::new("C", log), probe)),
    )
}

/// `Router::layer` nối tiếp: lớp gọi sau bọc ra ngoài. Kỳ vọng `C:in B:in A:in handler A:out B:out C:out`.
pub fn by_router_layer(log: &Log) -> Router {
    handler_router(log)
        .layer(from_fn_with_state(Probe::new("A", log), probe))
        .layer(from_fn_with_state(Probe::new("B", log), probe))
        .layer(from_fn_with_state(Probe::new("C", log), probe))
}
