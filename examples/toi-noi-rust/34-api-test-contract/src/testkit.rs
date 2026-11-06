//! Đồ dùng cho test và lab: fixture PostgreSQL (schema `lab_<uuid>` như bài 27) và
//! hàm gọi router bằng `oneshot`. Nằm trong lib để cả `tests/` lẫn binary `--verify` dùng chung.

use anyhow::{Context, Result, bail};
use axum::Router;
use axum::body::{Body, Bytes, to_bytes};
use axum::http::{HeaderMap, Method, Request, StatusCode};
use sqlx::postgres::PgPoolOptions;
use sqlx::{AssertSqlSafe, Connection, Executor, PgConnection, PgPool};
use tower::ServiceExt;
use uuid::Uuid;

use crate::api::{AppState, router};
use crate::auth::{AuthConfig, SCOPE_READ, SCOPE_WRITE, issue_token};
use crate::store::{MIGRATOR, PgLedger};

/// Dấu tuổi của schema, cùng dạng với bài 27: lệnh `--cleanup-stale` của lab 27 dọn được schema này
/// khi tiến trình bị kill hoặc `Drop` không xoá được.
const MARKER: &str = "toi-noi-lab-27 created_at=";

/// Tên biến môi trường chứa URL PostgreSQL của lab.
pub const ENV_PG_URL: &str = "TOI_NOI_PG_URL";

/// Đọc URL PostgreSQL; thiếu là lỗi rõ, không phải lý do để bỏ qua test.
pub fn pg_url() -> Result<String> {
    match std::env::var(ENV_PG_URL) {
        Ok(v) if !v.trim().is_empty() => Ok(v),
        _ => bail!(
            "thiếu {ENV_PG_URL}. Bật PostgreSQL lab: \
             docker compose -f examples/toi-noi-rust/infra/docker-compose.yml up -d --wait, \
             rồi export {ENV_PG_URL}='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'"
        ),
    }
}

/// Một ca = một schema riêng + một pool chỉ nhìn thấy schema đó.
pub struct TestDb {
    /// Pool đã đặt `search_path` về schema của ca.
    pub pool: PgPool,
    /// Tên schema, dạng `lab_` + 32 ký tự hex.
    pub schema: String,
    url: String,
    closed: bool,
}

impl TestDb {
    /// Dựng schema, pool và chạy migration.
    pub async fn try_new() -> Result<TestDb> {
        let url = pg_url()?;
        let schema = format!("lab_{}", Uuid::new_v4().simple());
        let mut admin = PgConnection::connect(&url)
            .await
            .with_context(|| format!("không kết nối được PostgreSQL ở {ENV_PG_URL}"))?;
        admin
            .execute(AssertSqlSafe(format!("create schema {schema}")))
            .await?;
        let epoch: i64 = sqlx::query_scalar("select extract(epoch from now())::bigint")
            .fetch_one(&mut admin)
            .await?;
        admin
            .execute(AssertSqlSafe(format!(
                "comment on schema {schema} is '{MARKER}{epoch}'"
            )))
            .await?;
        admin.close().await?;

        // Mọi kết nối mới của pool tự đặt search_path trước khi dùng.
        let sp = schema.clone();
        let pool = PgPoolOptions::new()
            .max_connections(4)
            .after_connect(move |conn, _meta| {
                let sql = format!("set search_path to {sp}");
                Box::pin(async move {
                    conn.execute(AssertSqlSafe(sql)).await?;
                    Ok(())
                })
            })
            .connect(&url)
            .await?;
        let db = TestDb {
            pool,
            schema,
            url,
            closed: false,
        };
        if let Err(e) = MIGRATOR.run(&db.pool).await {
            db.close().await;
            return Err(e).context("chạy migration trong schema riêng thất bại");
        }
        Ok(db)
    }

    /// Như `try_new` nhưng panic với thông điệp rõ: dùng trong `#[tokio::test]`.
    pub async fn new() -> TestDb {
        match TestDb::try_new().await {
            Ok(db) => db,
            Err(e) => panic!("không dựng được TestDb: {e:#}"),
        }
    }

    /// Sổ cái trên pool của ca.
    pub fn ledger(&self) -> PgLedger {
        PgLedger::new(self.pool.clone())
    }

    /// Bốn tài khoản của chuyến TN-0001.
    pub async fn seed_accounts(&self) -> Result<()> {
        let ledger = self.ledger();
        for (id, kind) in [
            ("customer:CU-001", "liability"),
            ("driver:DR-001", "liability"),
            ("platform:revenue", "revenue"),
            ("psp:vixanh:clearing", "asset"),
        ] {
            ledger.ensure_account(id, kind).await?;
        }
        Ok(())
    }

    /// Dọn tường minh: đóng pool rồi xoá schema. Đường thường của test chạy xong.
    pub async fn close(mut self) {
        self.closed = true;
        self.pool.close().await;
        if let Err(e) = drop_schema(&self.url, &self.schema).await {
            eprintln!("không dọn được schema {}: {e:#}", self.schema);
        }
    }
}

impl Drop for TestDb {
    // Test panic giữa chừng thì `close()` không được gọi. `Drop` không await được, và `block_on` ngay
    // trong runtime của test thì panic (bài 27), nên dọn trên một thread riêng có runtime riêng rồi
    // `join`. Trong lúc `join`, runtime của test đứng yên: transaction còn mở không rollback được và
    // giữ khoá, nên `drop_schema` đặt `lock_timeout` để không treo; khi đó schema sót, dọn theo tuổi.
    fn drop(&mut self) {
        if self.closed {
            return;
        }
        let (url, schema) = (self.url.clone(), self.schema.clone());
        let cleanup = std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?;
            runtime.block_on(drop_schema(&url, &schema))
        });
        if !matches!(cleanup.join(), Ok(Ok(()))) {
            eprintln!("cảnh báo: không dọn được schema {}", self.schema);
        }
    }
}

// Kết nối quản trị riêng, không đi qua pool của ca.
async fn drop_schema(url: &str, schema: &str) -> Result<()> {
    let mut admin = PgConnection::connect(url).await?;
    admin.execute("set lock_timeout = '3s'").await?;
    admin
        .execute(AssertSqlSafe(format!(
            "drop schema if exists {schema} cascade"
        )))
        .await?;
    admin.close().await?;
    Ok(())
}

/// App thử nghiệm: DB riêng + khoá JWT ngẫu nhiên của ca (không có khoá cố định nào trong repo).
pub struct TestApp {
    /// Fixture database.
    pub db: TestDb,
    secret: String,
}

impl TestApp {
    /// Dựng DB, seed 4 tài khoản, sinh khoá JWT. Lỗi thì trả `Err` (dùng cho binary).
    pub async fn try_start() -> Result<TestApp> {
        let db = TestDb::try_new().await?;
        db.seed_accounts().await?;
        Ok(TestApp {
            db,
            secret: Uuid::new_v4().to_string(),
        })
    }

    /// Như `try_start` nhưng panic với thông điệp rõ: dùng trong `#[tokio::test]`.
    pub async fn start() -> TestApp {
        match TestApp::try_start().await {
            Ok(app) => app,
            Err(e) => panic!("không dựng được TestApp: {e:#}"),
        }
    }

    /// Router dùng DB của ca.
    pub fn router(&self) -> Router {
        router(AppState {
            ledger: self.db.ledger(),
            auth: std::sync::Arc::new(AuthConfig::new(self.secret.as_bytes())),
        })
    }

    /// Token hợp lệ 5 phút với các quyền cho trước.
    pub fn token(&self, scopes: &[&str]) -> String {
        issue_token(self.secret.as_bytes(), "bff-payment", scopes, 300)
    }

    /// Token đọc và ghi.
    pub fn full_token(&self) -> String {
        self.token(&[SCOPE_READ, SCOPE_WRITE])
    }

    /// Token ký bằng khoá khác (giả mạo).
    pub fn forged_token(&self) -> String {
        issue_token(b"khoa-khac", "attacker", &[SCOPE_READ, SCOPE_WRITE], 300)
    }

    /// Token đã hết hạn một giờ trước.
    pub fn expired_token(&self) -> String {
        issue_token(
            self.secret.as_bytes(),
            "bff-payment",
            &[SCOPE_READ, SCOPE_WRITE],
            -3600,
        )
    }

    /// Dọn schema của ca.
    pub async fn close(self) {
        self.db.close().await;
    }
}

/// Router không chạm DB: pool lazy không kết nối cho tới khi có truy vấn.
/// Dùng cho test chỉ cần tài liệu OpenAPI hoặc lỗi auth.
pub fn offline_router() -> Router {
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://offline:offline@127.0.0.1:1/offline")
        .expect("URL hợp lệ");
    router(AppState {
        ledger: PgLedger::new(pool),
        auth: std::sync::Arc::new(AuthConfig::new(b"offline-secret-khong-dung-that")),
    })
}

/// Phản hồi của một lần `oneshot`.
#[derive(Debug)]
pub struct Reply {
    /// Mã HTTP.
    pub status: StatusCode,
    /// Header của phản hồi.
    pub headers: HeaderMap,
    /// Thân.
    pub body: Bytes,
}

impl Reply {
    /// Thân đọc như JSON.
    pub fn json(&self) -> serde_json::Value {
        serde_json::from_slice(&self.body).expect("thân là JSON")
    }
}

/// Gọi router bằng `tower::ServiceExt::oneshot`: không socket, không HTTP parser,
/// chỉ `Request` đi vào và `Response` đi ra.
pub async fn send(
    app: &Router,
    method: Method,
    uri: &str,
    token: Option<&str>,
    json_body: Option<&str>,
) -> Reply {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(t) = token {
        builder = builder.header("authorization", format!("Bearer {t}"));
    }
    let body = match json_body {
        Some(b) => {
            builder = builder.header("content-type", "application/json");
            Body::from(b.to_string())
        }
        None => Body::empty(),
    };
    let request = builder.body(body).expect("request hợp lệ");
    // Router::clone rẻ (bên trong là Arc); oneshot tiêu thụ service nên cần bản sao.
    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("Router không lỗi");
    let (parts, body) = response.into_parts();
    let body = to_bytes(body, 1 << 20).await.expect("đọc được thân");
    Reply {
        status: parts.status,
        headers: parts.headers,
        body,
    }
}
