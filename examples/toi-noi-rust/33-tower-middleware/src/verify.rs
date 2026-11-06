//! Các kiểm chứng của bài 33. Mỗi hàm in điều đã quan sát và trả `Err` nếu sai kỳ vọng.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;

use axum::Router;
use axum::body::{Body, Bytes, to_bytes};
use axum::http::{HeaderMap, Method, Request, StatusCode};
use axum::routing::{get, post};
use jsonwebtoken::Algorithm;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::task::JoinSet;
use tower::ServiceExt;
use tower::limit::ConcurrencyLimitLayer;
use tower_http::timeout::TimeoutLayer;

use crate::api::AppState;
use crate::auth::{AUDIENCE, AuthConfig, Claims, now_secs};
use crate::capture::capture;
use crate::stack::{self, Settings};
use crate::{client, limits, order};

/// Kết quả một kiểm chứng.
pub type Check = Result<(), String>;

macro_rules! ensure {
    ($cond:expr, $($arg:tt)+) => {
        if !$cond {
            return Err(format!($($arg)+));
        }
    };
}

/// Bút toán mẫu của chuyến `TN-0001`: khách trả 25.500, tài xế nhận 20.400, nền tảng giữ 5.100.
pub const TN_0001: &str = r#"{"idempotencyKey":"trip:TN-0001:capture","postings":[{"account":"customer:CU-001","amountDong":-25500},{"account":"driver:DR-001","amountDong":20400},{"account":"platform:revenue","amountDong":5100}]}"#;

fn request(method: Method, uri: &str, token: Option<&str>, body: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(t) = token {
        builder = builder.header("authorization", format!("Bearer {t}"));
    }
    if body.is_some() {
        builder = builder.header("content-type", "application/json");
    }
    builder
        .body(Body::from(body.unwrap_or("").to_string()))
        .expect("request")
}

async fn call(app: &Router, req: Request<Body>) -> (StatusCode, HeaderMap, String) {
    let response = app
        .clone()
        .oneshot(req)
        .await
        .expect("Router không bao giờ lỗi");
    let (parts, body) = response.into_parts();
    let bytes = to_bytes(body, 1 << 20).await.expect("body");
    (
        parts.status,
        parts.headers,
        String::from_utf8_lossy(&bytes).into_owned(),
    )
}

fn claims(sub: &str, roles: &[&str], exp: u64) -> Claims {
    Claims {
        sub: sub.into(),
        roles: roles.iter().map(|r| (*r).into()).collect(),
        aud: AUDIENCE.into(),
        exp,
    }
}

fn token_for(state: &AppState, roles: &[&str]) -> String {
    state
        .auth
        .issue(&claims("svc:test", roles, now_secs() + 300))
}

/// 1. Thứ tự layer: `ServiceBuilder` đọc từ trên xuống; `Router::layer` nối tiếp thì ngược lại.
pub async fn check_order() -> Check {
    let run = |build: fn(&order::Log) -> Router| async move {
        let log = order::Log::default();
        let app = build(&log);
        let (status, _, _) = call(&app, request(Method::GET, "/", None, None)).await;
        (status, log.lock().expect("log").join(" "))
    };
    let (s1, by_builder) = run(order::by_service_builder).await;
    let (s2, by_router) = run(order::by_router_layer).await;
    println!("ServiceBuilder A, B, C : {by_builder}");
    println!("Router.layer A, B, C   : {by_router}");
    ensure!(
        s1 == StatusCode::OK && s2 == StatusCode::OK,
        "handler không trả 200"
    );
    ensure!(
        by_builder == "A:in B:in C:in handler C:out B:out A:out",
        "ServiceBuilder sai thứ tự: {by_builder}"
    );
    ensure!(
        by_router == "C:in B:in A:in handler A:out B:out C:out",
        "Router::layer sai thứ tự: {by_router}"
    );
    Ok(())
}

/// 2. Request id: sinh khi thiếu, giữ khi client gửi, có trong response (kể cả 401) và trong span.
pub async fn check_request_id() -> Check {
    let (logs, _guard) = capture();
    let state = AppState::for_lab();
    let app = crate::build_app(state.clone(), &Settings::default());
    let token = token_for(&state, &["finance"]);

    let (status, headers, _) = call(
        &app,
        request(Method::POST, "/v1/entries", Some(&token), Some(TN_0001)),
    )
    .await;
    ensure!(status == StatusCode::CREATED, "kỳ vọng 201, nhận {status}");
    let id = headers
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .ok_or("thiếu x-request-id")?
        .to_string();
    ensure!(id.len() == 36, "id không phải UUID: {id}");

    let mut fixed = request(Method::GET, "/healthz", None, None);
    fixed
        .headers_mut()
        .insert("x-request-id", "bff-req-42".parse().expect("header"));
    let (_, headers, _) = call(&app, fixed).await;
    let kept = headers
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    ensure!(kept == "bff-req-42", "id của client bị đổi: {kept}");

    let (status, headers, _) = call(
        &app,
        request(
            Method::GET,
            "/v1/accounts/driver:DR-001/balance",
            None,
            None,
        ),
    )
    .await;
    ensure!(
        status == StatusCode::UNAUTHORIZED && headers.contains_key("x-request-id"),
        "401 thiếu x-request-id"
    );

    let text = logs.text();
    let line = text
        .lines()
        .find(|l| l.contains("ghi bút toán"))
        .ok_or("không thấy log của handler")?;
    println!("response x-request-id: {id}");
    println!("log của handler      : {line}");
    ensure!(
        line.contains(&format!("request_id={id}")),
        "log của handler không mang request id"
    );
    ensure!(
        text.lines().any(|l| l.contains("request_id=bff-req-42")),
        "id của client không vào span"
    );
    Ok(())
}

// Cờ `finished` chỉ bật nếu handler chạy tới cuối: bằng chứng handler bị huỷ ở điểm `.await`.
async fn slow(finished: Arc<AtomicBool>) -> &'static str {
    tokio::time::sleep(Duration::from_millis(600)).await;
    finished.store(true, Ordering::SeqCst);
    "quá muộn"
}

/// 3. Timeout: mã trả về và việc request id vẫn có trong response quá hạn.
pub async fn check_timeout() -> Check {
    let (logs, _guard) = capture();
    let settings = Settings {
        timeout: Duration::from_millis(150),
        ..Settings::default()
    };
    let finished = Arc::new(AtomicBool::new(false));
    let handler = {
        let finished = finished.clone();
        move || slow(finished.clone())
    };
    let app = stack::apply(Router::new().route("/slow", get(handler)), &settings);
    let started = std::time::Instant::now();
    let (status, headers, body) = call(&app, request(Method::GET, "/slow", None, None)).await;
    let took = started.elapsed();
    println!(
        "TimeoutLayer::with_status_code(GATEWAY_TIMEOUT, 150ms): {status}, body {body:?}, mất {} ms",
        took.as_millis()
    );
    ensure!(
        status == StatusCode::GATEWAY_TIMEOUT && body.is_empty(),
        "kỳ vọng 504 rỗng"
    );
    ensure!(
        took < Duration::from_millis(500),
        "handler vẫn chạy hết 600 ms thay vì bị huỷ"
    );
    ensure!(
        headers.contains_key("x-request-id"),
        "response quá hạn thiếu x-request-id"
    );
    let line = logs
        .text()
        .lines()
        .find(|l| l.contains("finished processing request"))
        .map(str::to_string)
        .ok_or("TraceLayer không ghi log cho response 504")?;
    println!("log của TraceLayer: {line}");
    ensure!(line.contains("status=504"), "log không ghi 504: {line}");
    tokio::time::sleep(Duration::from_millis(700)).await; // quá hạn 600 ms của handler
    println!(
        "700 ms sau, handler có chạy tới cuối không: {}",
        finished.load(Ordering::SeqCst)
    );
    ensure!(
        !finished.load(Ordering::SeqCst),
        "future của handler vẫn chạy sau timeout"
    );

    // Thân request gửi dở: extractor đọc body nằm trong future của handler nên vẫn trong hạn.
    let app = stack::apply(
        Router::new().route(
            "/upload",
            post(|body: Bytes| async move { body.len().to_string() }),
        ),
        &settings,
    );
    let (addr, server) = client::spawn(app).await;
    let mut stream = TcpStream::connect(addr).await.expect("connect");
    stream
        .write_all(b"POST /upload HTTP/1.1\r\nHost: lab\r\nContent-Length: 10\r\n\r\nabc")
        .await
        .expect("write");
    let mut head = [0u8; 64];
    let read = stream.read(&mut head).await.expect("read");
    server.abort();
    let first = String::from_utf8_lossy(&head[..read]);
    let first = first.lines().next().unwrap_or("").to_string();
    println!("thân request gửi dở, hạn 150 ms: {first}");
    ensure!(
        first.contains("504"),
        "đọc body phải nằm trong hạn: {first}"
    );

    // Hàm `new` đã bị tower-http đánh dấu deprecated; dùng ở đây chỉ để thấy mặc định cũ.
    #[allow(deprecated)]
    let legacy = Router::new()
        .route("/slow", get(|| slow(Arc::default())))
        .layer(TimeoutLayer::new(Duration::from_millis(150)));
    let (status, _, _) = call(&legacy, request(Method::GET, "/slow", None, None)).await;
    println!("TimeoutLayer::new (deprecated), mặc định        : {status}");
    ensure!(
        status == StatusCode::REQUEST_TIMEOUT,
        "mặc định cũ không phải 408"
    );
    Ok(())
}

/// 4. CORS: service nội bộ không gắn; khi gắn thì chỉ cho đúng một origin.
pub async fn check_cors() -> Check {
    let origin = "https://admin.toi-noi.example";
    let preflight = |from: &str| {
        Request::builder()
            .method(Method::OPTIONS)
            .uri("/v1/entries")
            .header("origin", from.to_string())
            .header("access-control-request-method", "POST")
            .header("access-control-request-headers", "authorization")
            .body(Body::empty())
            .expect("request")
    };
    let state = AppState::for_lab();

    let plain = crate::build_app(state.clone(), &Settings::default());
    let (status, headers, _) = call(&plain, preflight(origin)).await;
    println!(
        "không gắn CORS, preflight   : {status}, allow-origin = {:?}",
        headers.get("access-control-allow-origin")
    );
    ensure!(
        !headers.contains_key("access-control-allow-origin"),
        "không gắn CORS mà vẫn có allow-origin"
    );

    let strict = crate::build_app(
        state,
        &Settings {
            cors_origin: Some(origin.into()),
            ..Settings::default()
        },
    );
    let (status, headers, _) = call(&strict, preflight(origin)).await;
    let allowed = headers
        .get("access-control-allow-origin")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    println!("gắn CORS, origin được phép  : {status}, allow-origin = {allowed:?}");
    ensure!(
        status == StatusCode::OK && allowed.as_deref() == Some(origin),
        "preflight của origin hợp lệ bị từ chối"
    );
    let (_, headers, _) = call(&strict, preflight("https://evil.example")).await;
    println!(
        "gắn CORS, origin lạ         : allow-origin = {:?}",
        headers.get("access-control-allow-origin")
    );
    ensure!(
        !headers.contains_key("access-control-allow-origin"),
        "origin lạ vẫn nhận allow-origin"
    );
    Ok(())
}

/// 5. JWT HS256: token hợp lệ qua, mọi biến thể hỏng đều 401 và log chỉ ghi loại lỗi.
pub async fn check_jwt() -> Check {
    let (logs, _guard) = capture();
    let secret = format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    );
    let auth = Arc::new(AuthConfig::with_secret(secret.as_bytes()));
    let state = AppState {
        ledger: Default::default(),
        auth: auth.clone(),
    };
    let app = crate::build_app(state, &Settings::default());
    let uri = "/v1/accounts/driver:DR-001/balance";
    let now = now_secs();

    let other = AuthConfig::for_lab(); // khoá khác: chữ ký sai
    let mut no_aud = serde_json::json!({ "sub": "svc:test", "roles": ["cs"], "exp": now + 300 });
    let wrong_aud = Claims {
        aud: "payment-api".into(),
        ..claims("svc:test", &["cs"], now + 300)
    };
    let cases: Vec<(&str, String, StatusCode)> = vec![
        (
            "hợp lệ (cs)",
            auth.issue(&claims("svc:test", &["cs"], now + 300)),
            StatusCode::OK,
        ),
        (
            "hết hạn 120 giây trước",
            auth.issue(&claims("svc:test", &["cs"], now - 120)),
            StatusCode::UNAUTHORIZED,
        ),
        (
            "sai chữ ký",
            other.issue(&claims("svc:test", &["cs"], now + 300)),
            StatusCode::UNAUTHORIZED,
        ),
        ("sai aud", auth.issue(&wrong_aud), StatusCode::UNAUTHORIZED),
        (
            "thiếu aud",
            auth.sign(Algorithm::HS256, &no_aud),
            StatusCode::UNAUTHORIZED,
        ),
        (
            "ký HS512 cùng khoá",
            auth.issue_alg(Algorithm::HS512, &claims("svc:test", &["cs"], now + 300)),
            StatusCode::UNAUTHORIZED,
        ),
        (
            "thiếu roles",
            {
                no_aud["aud"] = AUDIENCE.into();
                no_aud.as_object_mut().expect("object").remove("roles");
                auth.sign(Algorithm::HS256, &no_aud)
            },
            StatusCode::UNAUTHORIZED,
        ),
        (
            "chuỗi rác",
            "khong.phai.jwt".into(),
            StatusCode::UNAUTHORIZED,
        ),
    ];
    for (label, token, expected) in &cases {
        let (status, headers, body) =
            call(&app, request(Method::GET, uri, Some(token), None)).await;
        println!("{label:<24} -> {status}");
        ensure!(
            status == *expected,
            "{label}: kỳ vọng {expected}, nhận {status}"
        );
        if *expected == StatusCode::UNAUTHORIZED {
            ensure!(
                headers.contains_key("www-authenticate"),
                "{label}: thiếu WWW-Authenticate"
            );
            ensure!(
                !body.contains("Expired") && !body.contains("Signature"),
                "{label}: body lộ lý do"
            );
        }
    }
    // Không header, và scheme khác Bearer.
    let (status, _, _) = call(&app, request(Method::GET, uri, None, None)).await;
    ensure!(
        status == StatusCode::UNAUTHORIZED,
        "thiếu Authorization phải 401"
    );
    let basic = Request::builder()
        .uri(uri)
        .header("authorization", "Basic dXNlcjpwYXNz")
        .body(Body::empty())
        .expect("request");
    let (status, _, _) = call(&app, basic).await;
    ensure!(status == StatusCode::UNAUTHORIZED, "scheme Basic phải 401");
    println!("thiếu header / scheme Basic -> 401");

    // Lý do chỉ có trong log, dạng tên lỗi.
    let text = logs.text();
    let reasons: Vec<&str> = text
        .lines()
        .filter_map(|l| l.split("reason=").nth(1))
        .collect();
    println!("lý do trong log: {}", reasons.join(" | "));
    for reason in [
        "ExpiredSignature",
        "InvalidSignature",
        "InvalidAudience",
        "InvalidAlgorithm",
    ] {
        ensure!(text.contains(reason), "log thiếu lý do {reason}");
    }
    for (_, token, _) in &cases {
        ensure!(!text.contains(token.as_str()), "log chứa token");
    }

    // Leeway mặc định của thư viện là 60 giây: token hết hạn 30 giây trước vẫn lọt qua `Validation` mặc định.
    let stale = auth.issue(&claims("svc:test", &["cs"], now - 30));
    let mut default_rules = jsonwebtoken::Validation::new(Algorithm::HS256);
    default_rules.set_audience(&[AUDIENCE]);
    let accepted = jsonwebtoken::decode::<Claims>(
        &stale,
        &jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()),
        &default_rules,
    )
    .is_ok();
    let (status, _, _) = call(&app, request(Method::GET, uri, Some(&stale), None)).await;
    println!(
        "hết hạn 30 giây trước: leeway mặc định (60 s) {} ; lab (5 s) -> {status}",
        if accepted {
            "chấp nhận"
        } else {
            "từ chối"
        }
    );
    ensure!(
        accepted && status == StatusCode::UNAUTHORIZED,
        "leeway: mặc định phải chấp nhận, lab phải từ chối"
    );

    // `aud` chỉ bị kiểm khi token CÓ `aud`: nếu không liệt kê vào required_spec_claims, token thiếu `aud` lọt qua.
    let key = jsonwebtoken::DecodingKey::from_secret(secret.as_bytes());
    let bare = auth.sign(
        Algorithm::HS256,
        &serde_json::json!({ "sub": "x", "exp": now + 300 }),
    );
    let loose = jsonwebtoken::decode::<serde_json::Value>(&bare, &key, &default_rules).is_ok();
    let mut strict = default_rules.clone();
    strict.set_required_spec_claims(&["exp", "aud", "sub"]);
    let strict_err = jsonwebtoken::decode::<serde_json::Value>(&bare, &key, &strict)
        .err()
        .map(|e| format!("{:?}", e.kind()));
    println!(
        "token không có aud: rule mặc định chấp nhận = {loose}; có required_spec_claims -> {strict_err:?}"
    );
    ensure!(
        loose && strict_err.as_deref() == Some("MissingRequiredClaim(\"aud\")"),
        "required_spec_claims không như kỳ vọng: {strict_err:?}"
    );
    Ok(())
}

/// 6. Phân quyền theo vai trò và thứ tự: xác thực, rồi quyền, rồi mới tới body.
pub async fn check_authz() -> Check {
    let state = AppState::for_lab();
    let app = crate::build_app(state.clone(), &Settings::default());
    let finance = token_for(&state, &["finance"]);
    let cs = token_for(&state, &["cs"]);
    let stranger = token_for(&state, &["admin"]); // vai trò không có trong hệ thống
    let balance = "/v1/accounts/driver:DR-001/balance";

    let (status, _, body) = call(
        &app,
        request(Method::POST, "/v1/entries", Some(&finance), Some(TN_0001)),
    )
    .await;
    println!("finance ghi TN-0001        -> {status} {body}");
    ensure!(
        status == StatusCode::CREATED && body.contains("JE-000001"),
        "finance phải ghi được (201)"
    );
    let (status, _, body) = call(
        &app,
        request(Method::POST, "/v1/entries", Some(&finance), Some(TN_0001)),
    )
    .await;
    println!("finance ghi lại cùng key   -> {status} {body}");
    ensure!(
        status == StatusCode::OK && body.contains("\"replayed\":true"),
        "replay phải 200"
    );

    let (status, headers, body) = call(
        &app,
        request(Method::POST, "/v1/entries", Some(&cs), Some(TN_0001)),
    )
    .await;
    println!("cs ghi bút toán            -> {status} {body}");
    ensure!(status == StatusCode::FORBIDDEN, "cs ghi phải 403");
    ensure!(
        headers.get("content-type").and_then(|v| v.to_str().ok())
            == Some("application/problem+json"),
        "403 phải là problem+json"
    );
    let (status, _, _) = call(
        &app,
        request(Method::POST, "/v1/entries", Some(&stranger), Some(TN_0001)),
    )
    .await;
    ensure!(status == StatusCode::FORBIDDEN, "vai trò lạ phải 403");

    let (status, _, body) = call(&app, request(Method::GET, balance, Some(&cs), None)).await;
    println!("cs tra số dư DR-001        -> {status} {body}");
    ensure!(
        status == StatusCode::OK && body.contains("\"balanceDong\":20400"),
        "cs phải đọc được 20400"
    );
    let (status, _, _) = call(&app, request(Method::GET, balance, Some(&finance), None)).await;
    ensure!(status == StatusCode::OK, "finance phải đọc được số dư");
    let (status, _, _) = call(&app, request(Method::GET, balance, Some(&stranger), None)).await;
    ensure!(
        status == StatusCode::FORBIDDEN,
        "vai trò lạ đọc số dư phải 403"
    );

    // Nghiệp vụ vẫn do handler quyết định, sau khi qua cổng quyền.
    let (status, _, _) = call(&app, request(Method::POST, "/v1/entries", Some(&finance), Some(r#"{"idempotencyKey":"k","postings":[{"account":"a","amountDong":5},{"account":"b","amountDong":-4}]}"#))).await;
    ensure!(
        status == StatusCode::UNPROCESSABLE_ENTITY,
        "bút toán lệch phải 422, nhận {status}"
    );

    // Thứ tự đánh giá: body rác không token -> 401; body rác với cs -> 403; chỉ finance mới tới bước parse (422).
    let garbage = Some("không phải json");
    let (a, _, _) = call(&app, request(Method::POST, "/v1/entries", None, garbage)).await;
    let (b, _, _) = call(
        &app,
        request(Method::POST, "/v1/entries", Some(&cs), garbage),
    )
    .await;
    let (c, _, _) = call(
        &app,
        request(Method::POST, "/v1/entries", Some(&finance), garbage),
    )
    .await;
    println!("body rác: không token {a}, cs {b}, finance {c}");
    ensure!(
        a == StatusCode::UNAUTHORIZED && b == StatusCode::FORBIDDEN,
        "401/403 phải đến trước lỗi body"
    );
    ensure!(
        c == StatusCode::BAD_REQUEST || c == StatusCode::UNPROCESSABLE_ENTITY,
        "finance + body rác phải bị Json từ chối"
    );

    let (status, _, _) = call(&app, request(Method::GET, "/healthz", None, None)).await;
    ensure!(status == StatusCode::OK, "healthz không cần token");
    Ok(())
}

/// 7. Không log token: `Authorization` bị che; bỏ lớp che thì token lộ ngay trong log.
pub async fn check_no_token_in_logs() -> Check {
    let state = AppState::for_lab();
    let token = token_for(&state, &["cs"]);
    let uri = "/v1/accounts/driver:DR-001/balance";

    let (logs, guard) = capture();
    let app = crate::build_app(
        state.clone(),
        &Settings {
            trace_headers: true,
            ..Settings::default()
        },
    );
    let (status, _, _) = call(&app, request(Method::GET, uri, Some(&token), None)).await;
    ensure!(status == StatusCode::OK, "kỳ vọng 200");
    let hidden = logs.text();
    drop(guard);
    let line = hidden
        .lines()
        .find(|l| l.contains("authorization"))
        .ok_or("span không có header")?;
    println!(
        "che Authorization : {}",
        line.split("headers=").nth(1).unwrap_or(line)
    );
    ensure!(
        !hidden.contains(&token) && hidden.contains("Sensitive"),
        "token lọt vào log dù đã đánh dấu nhạy cảm"
    );

    let (logs, _guard) = capture();
    let leaky = crate::build_app(
        state,
        &Settings {
            trace_headers: true,
            hide_authorization: false,
            ..Settings::default()
        },
    );
    call(&leaky, request(Method::GET, uri, Some(&token), None)).await;
    let exposed = logs.text().contains(&token);
    println!("bỏ lớp che        : token có trong log = {exposed}");
    ensure!(
        exposed,
        "đối chứng: bỏ lớp che mà token không lộ thì phép thử vô nghĩa"
    );
    Ok(())
}

async fn work(in_flight: Arc<AtomicUsize>, peak: Arc<AtomicUsize>) -> &'static str {
    let now = in_flight.fetch_add(1, Ordering::SeqCst) + 1;
    peak.fetch_max(now, Ordering::SeqCst);
    tokio::time::sleep(Duration::from_millis(100)).await;
    in_flight.fetch_sub(1, Ordering::SeqCst);
    "xong"
}

fn work_router() -> (Router, Arc<AtomicUsize>) {
    let (in_flight, peak) = (Arc::new(AtomicUsize::new(0)), Arc::new(AtomicUsize::new(0)));
    let (f1, p1, f2, p2) = (in_flight.clone(), peak.clone(), in_flight, peak.clone());
    // Hai route khác nhau cùng một bộ đếm: để thấy giới hạn dùng chung chứ không tính theo route.
    let router = Router::new()
        .route("/a", get(move || work(f1.clone(), p1.clone())))
        .route("/b", get(move || work(f2.clone(), p2.clone())));
    (router, peak)
}

async fn burst(app: &Router, paths: &[&str]) -> Vec<StatusCode> {
    let mut set = JoinSet::new();
    for path in paths {
        let (app, path) = (app.clone(), (*path).to_string());
        set.spawn(async move {
            app.oneshot(request(Method::GET, &path, None, None))
                .await
                .expect("infallible")
                .status()
        });
    }
    let mut out = Vec::new();
    while let Some(done) = set.join_next().await {
        out.push(done.expect("task"));
    }
    out
}

/// 8. `tower::limit` cho concurrency: xếp hàng so với từ chối ngay.
pub async fn check_concurrency() -> Check {
    let six = ["/a", "/b", "/a", "/b", "/a", "/b"];

    // Bẫy: `Router` chưa qua `with_state` dựng lại route (và `ConcurrencyLimit` mới, semaphore mới) cho MỖI request.
    let (router, peak) = work_router();
    burst(&router.layer(ConcurrencyLimitLayer::new(2)), &six).await;
    println!(
        "layer thẳng, chưa with_state : đỉnh đồng thời = {} (giới hạn 2 không có tác dụng)",
        peak.load(Ordering::SeqCst)
    );
    ensure!(
        peak.load(Ordering::SeqCst) > 2,
        "bẫy with_state không tái hiện: đỉnh = {}",
        peak.load(Ordering::SeqCst)
    );

    // `axum::serve(listener, router)` gọi `with_state(())` cho TỪNG kết nối: mỗi kết nối một semaphore, dù chỉ một route.
    let (router, peak) = work_router();
    let (addr, server) = client::spawn(router.layer(ConcurrencyLimitLayer::new(2))).await;
    let mut set = JoinSet::new();
    for _ in 0..6 {
        set.spawn(async move { client::send(addr, "GET", "/a", &[], "").await.status });
    }
    set.join_all().await;
    server.abort();
    println!(
        "layer thẳng, axum::serve, 6 kết nối, 1 route : đỉnh đồng thời = {}",
        peak.load(Ordering::SeqCst)
    );
    ensure!(
        peak.load(Ordering::SeqCst) > 2,
        "bẫy theo kết nối không tái hiện: đỉnh = {}",
        peak.load(Ordering::SeqCst)
    );

    // Đã with_state nhưng dùng layer thường: mỗi route một semaphore riêng, hai route cho phép tới 2 + 2 request.
    let (router, peak) = work_router();
    burst(
        &router.with_state(()).layer(ConcurrencyLimitLayer::new(2)),
        &six,
    )
    .await;
    println!(
        "ConcurrencyLimitLayer(2), 2 route : đỉnh đồng thời = {} (mỗi route một semaphore)",
        peak.load(Ordering::SeqCst)
    );
    ensure!(
        peak.load(Ordering::SeqCst) == 4,
        "kỳ vọng đỉnh 4 với hai route, nhận {}",
        peak.load(Ordering::SeqCst)
    );

    let (router, peak) = work_router();
    let statuses = burst(&limits::concurrency_queued(router, 2), &six).await;
    let ok = statuses.iter().filter(|s| **s == StatusCode::OK).count();
    println!(
        "GlobalConcurrencyLimit(2), xếp hàng : {ok}/6 trả 200, đỉnh đồng thời = {}",
        peak.load(Ordering::SeqCst)
    );
    ensure!(
        ok == 6 && peak.load(Ordering::SeqCst) == 2,
        "xếp hàng: cả 6 phải xong và đỉnh đồng thời phải bằng 2"
    );

    let (router, peak) = work_router();
    let app = stack::apply(
        router,
        &Settings {
            max_in_flight: 2,
            ..Settings::default()
        },
    );
    let statuses = burst(&app, &six).await;
    let ok = statuses.iter().filter(|s| **s == StatusCode::OK).count();
    let shed = statuses
        .iter()
        .filter(|s| **s == StatusCode::SERVICE_UNAVAILABLE)
        .count();
    println!(
        "GlobalConcurrencyLimit(2) + LoadShed : {ok} trả 200, {shed} trả 503, đỉnh đồng thời = {}",
        peak.load(Ordering::SeqCst)
    );
    ensure!(
        ok == 2 && shed == 4 && peak.load(Ordering::SeqCst) == 2,
        "shed: kỳ vọng 2 xong, 4 bị từ chối"
    );
    Ok(())
}

/// 9. Rate limit toàn cục: cần `Buffer`, giới hạn chung mọi client, không phải theo client.
pub async fn check_rate_limit() -> Check {
    let two_routes = || {
        Router::new()
            .route("/a", get(|| async { "a" }))
            .route("/b", get(|| async { "b" }))
    };
    let per = Duration::from_millis(600);
    let status = |app: &Router, path: &'static str, client: &'static str| {
        let app = app.clone();
        async move {
            let mut req = request(Method::GET, path, None, None);
            req.headers_mut()
                .insert("x-client", client.parse().expect("header"));
            app.oneshot(req).await.expect("infallible").status()
        }
    };

    // Bẫy: layer qua `Router::layer` được áp cho từng route, mỗi route một bucket.
    let app = limits::rate_limited_per_route(two_routes(), 3, per);
    for _ in 0..3 {
        status(&app, "/a", "bff-a").await;
    }
    let (a4, b1) = (
        status(&app, "/a", "bff-a").await,
        status(&app, "/b", "bff-a").await,
    );
    println!("Router::layer, 2 route : /a lần 4: {a4}; /b lần đầu: {b1}");
    ensure!(
        a4 == StatusCode::SERVICE_UNAVAILABLE && b1 == StatusCode::OK,
        "bẫy bucket theo route không tái hiện: /a {a4}, /b {b1}"
    );

    let app = limits::rate_limited(two_routes(), 3, per);
    let mut seen = Vec::new();
    for _ in 0..3 {
        seen.push(status(&app, "/a", "bff-a").await);
    }
    let other = status(&app, "/b", "bff-b").await; // client khác, route khác, chưa gọi lần nào
    tokio::time::sleep(Duration::from_millis(650)).await;
    let after = status(&app, "/b", "bff-b").await;
    println!(
        "một service, 2 route   : bff-a /a 3 lần: {seen:?}; bff-b /b: {other}; sau 650 ms: {after}"
    );
    ensure!(
        seen.iter().all(|s| *s == StatusCode::OK),
        "3 lần đầu phải qua"
    );
    ensure!(
        other == StatusCode::SERVICE_UNAVAILABLE,
        "hạn mức toàn cục: client khác, route khác cũng bị từ chối"
    );
    ensure!(after == StatusCode::OK, "sau một chu kỳ phải qua lại");
    Ok(())
}

/// 10. Qua socket thật: cổng 0, header thật, không dùng `oneshot`.
pub async fn check_socket() -> Check {
    let state = AppState::for_lab();
    let finance = token_for(&state, &["finance"]);
    let (addr, server) = client::spawn(crate::build_app(state, &Settings::default())).await;
    ensure!(addr.port() != 0, "chưa đọc được cổng thật");
    let auth = format!("Bearer {finance}");
    let headers = [
        ("Authorization", auth.as_str()),
        ("Content-Type", "application/json"),
    ];
    let created = client::send(addr, "POST", "/v1/entries", &headers, TN_0001).await;
    let denied = client::send(addr, "GET", "/v1/accounts/driver:DR-001/balance", &[], "").await;
    let balance = client::send(
        addr,
        "GET",
        "/v1/accounts/driver:DR-001/balance",
        &headers,
        "",
    )
    .await;
    server.abort();
    println!(
        "socket 127.0.0.1:<cổng thật>: POST {} , GET không token {} , GET có token {}",
        created.status, denied.status, balance.status
    );
    ensure!(
        created.status == 201 && denied.status == 401 && balance.status == 200,
        "kết quả qua socket sai"
    );
    ensure!(
        balance.body.contains("\"balanceDong\":20400"),
        "số dư qua socket sai: {}",
        balance.body
    );
    ensure!(
        created.header("x-request-id").is_some(),
        "socket: thiếu x-request-id"
    );
    Ok(())
}

type Pending = std::pin::Pin<Box<dyn std::future::Future<Output = Check>>>;

/// Chạy mọi kiểm chứng theo thứ tự; trả về số kiểm chứng lỗi.
pub async fn run_all() -> usize {
    crate::capture::init();
    let checks: Vec<(&str, Pending)> = vec![
        ("thứ tự layer", Box::pin(check_order())),
        ("request id", Box::pin(check_request_id())),
        ("timeout", Box::pin(check_timeout())),
        ("cors", Box::pin(check_cors())),
        ("jwt", Box::pin(check_jwt())),
        ("phân quyền", Box::pin(check_authz())),
        ("không log token", Box::pin(check_no_token_in_logs())),
        ("concurrency", Box::pin(check_concurrency())),
        ("rate limit", Box::pin(check_rate_limit())),
        ("socket thật", Box::pin(check_socket())),
    ];
    let mut failed = 0;
    for (name, check) in checks {
        println!("== {name}");
        if let Err(why) = check.await {
            println!("LỖI: {why}");
            failed += 1;
        }
    }
    failed
}
