//! Fixture dùng chung: mỗi ca có một schema `lab_<uuid>` riêng trong database lab.
//! Mỗi file trong `tests/` chỉ dùng một phần các hàm ở đây nên cho phép `dead_code`.
#![allow(dead_code)]

use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use ledger_lab::{MIGRATOR, PgLedger};
use sqlx::postgres::PgPoolOptions;
use sqlx::{AssertSqlSafe, Connection, Executor, PgConnection, PgPool, Row};
use uuid::Uuid;

/// Tên biến môi trường chứa URL PostgreSQL của lab.
pub const ENV_PG_URL: &str = "TOI_NOI_PG_URL";
/// Dấu ghi trong comment của schema: PostgreSQL không lưu thời điểm tạo schema, nên ta tự ghi.
pub const MARKER: &str = "toi-noi-lab-27 created_at=";

/// Đọc URL từ giá trị biến môi trường. Thiếu hoặc rỗng là lỗi, không phải lý do để bỏ qua ca test.
pub fn pg_url_from(value: Option<String>) -> Result<String> {
    match value {
        Some(v) if !v.trim().is_empty() => Ok(v),
        _ => bail!(
            "thiếu {ENV_PG_URL}. Bật PostgreSQL lab: \
             docker compose -f examples/toi-noi-rust/infra/docker-compose.yml up -d --wait, \
             rồi export {ENV_PG_URL}='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'"
        ),
    }
}

/// Một ca test = một schema riêng + một pool chỉ nhìn thấy schema đó.
pub struct TestDb {
    /// Pool đã đặt `search_path` về schema của ca.
    pub pool: PgPool,
    /// Tên schema, dạng `lab_` + 32 ký tự hex.
    pub schema: String,
    /// URL của database lab (để mở kết nối quản trị khi dọn).
    pub url: String,
    closed: bool,
}

impl TestDb {
    /// Dựng schema, pool và chạy migration. Lỗi thì trả `Err` (dùng cho binary).
    pub async fn try_new() -> Result<TestDb> {
        let url = pg_url_from(std::env::var(ENV_PG_URL).ok())?;
        let schema = format!("lab_{}", Uuid::new_v4().simple());

        // Kết nối quản trị riêng, không đi qua pool của ca.
        let mut admin = PgConnection::connect(&url).await.with_context(|| {
            format!("không kết nối được PostgreSQL ở {ENV_PG_URL}; compose đã bật chưa?")
        })?;
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
            db.close().await; // setup hỏng thì dọn ngay, không để schema mồ côi
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

    /// Adapter sổ cái trên pool của ca.
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

    /// Dọn tường minh. `Drop` không async được nên không thể tự làm việc này.
    pub async fn close(mut self) {
        self.closed = true;
        self.pool.close().await; // đóng pool trước: hết kết nối còn giữ schema
        match PgConnection::connect(&self.url).await {
            Ok(mut admin) => {
                let sql = format!("drop schema if exists {} cascade", self.schema);
                if let Err(e) = admin.execute(AssertSqlSafe(sql)).await {
                    eprintln!("không DROP được schema {}: {e}", self.schema);
                }
                let _ = admin.close().await;
            }
            Err(e) => eprintln!("không dọn được schema {}: {e}", self.schema),
        }
    }
}

impl Drop for TestDb {
    // Drop chạy cả khi test panic nhưng không await được: chỉ cảnh báo, schema sẽ sót.
    fn drop(&mut self) {
        if !self.closed {
            eprintln!(
                "cảnh báo: TestDb {} bị drop mà chưa gọi close(); schema còn sót, dọn bằng --cleanup-stale",
                self.schema
            );
        }
    }
}

/// Schema có tồn tại không (kết nối quản trị riêng, không đi qua pool của ca).
pub async fn schema_exists(url: &str, schema: &str) -> Result<bool> {
    let mut admin = PgConnection::connect(url).await?;
    let found: bool =
        sqlx::query_scalar("select exists (select 1 from pg_namespace where nspname = $1)")
            .bind(schema)
            .fetch_one(&mut admin)
            .await?;
    admin.close().await?;
    Ok(found)
}

/// Đếm schema `lab_%` hiện có (của mọi lab dùng chung database).
pub async fn count_lab_schemas(url: &str) -> Result<i64> {
    let mut admin = PgConnection::connect(url).await?;
    let n: i64 =
        sqlx::query_scalar("select count(*) from pg_namespace where nspname like 'lab\\_%'")
            .fetch_one(&mut admin)
            .await?;
    admin.close().await?;
    Ok(n)
}

/// Xoá schema do fixture này tạo (có MARKER) và đã quá `max_age_secs`. Trả danh sách đã xoá.
/// Schema không có MARKER (của lab khác, hoặc ai đó tự tạo) không bị đụng tới.
pub async fn cleanup_stale_schemas(url: &str, max_age_secs: i64) -> Result<Vec<String>> {
    let mut admin = PgConnection::connect(url).await?;
    let rows = sqlx::query(
        "select n.nspname \
         from pg_namespace n join pg_description d \
           on d.objoid = n.oid and d.classoid = 'pg_namespace'::regclass \
         where n.nspname like 'lab\\_%' \
           and d.description like $1 \
           and extract(epoch from now())::bigint - substring(d.description from $2)::bigint > $3 \
         order by n.nspname",
    )
    .bind(format!("{MARKER}%"))
    .bind(i32::try_from(MARKER.len() + 1)?)
    .bind(max_age_secs)
    .fetch_all(&mut admin)
    .await?;
    let mut dropped = Vec::new();
    for row in rows {
        let name: String = row.get("nspname");
        admin
            .execute(AssertSqlSafe(format!(
                "drop schema if exists {name} cascade"
            )))
            .await?;
        dropped.push(name);
    }
    admin.close().await?;
    Ok(dropped)
}

/// Thời điểm hiện tại theo giây Unix (phía client), dùng để giả lập schema cũ.
pub fn now_epoch() -> i64 {
    i64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_secs()),
    )
    .unwrap_or(0)
}
