//! Một "database lab": schema riêng `lab_<uuid>` với schema chuẩn của loạt (bài 25).

use std::error::Error;
use std::str::FromStr;

use sqlx::postgres::{PgConnectOptions, PgSslMode};
use sqlx::{Connection, PgConnection};
use uuid::Uuid;

use crate::proxy::Proxy;
use crate::workload::standard_accounts;

/// Lỗi chung của lab.
pub type BoxError = Box<dyn Error + Send + Sync>;

// Dùng đúng file SQL của bài 25, không chép lại.
const SCHEMA_SQL: &str = include_str!("../../25-schema-so-cai/sql/01_schema.sql");
const BALANCE_TRIGGER_SQL: &str = include_str!("../../25-schema-so-cai/sql/02_balance_check.sql");
const INDEX_SQL: &str = include_str!("../../25-schema-so-cai/sql/03_index.sql");

/// Bảng thêm cho thí nghiệm độ trễ (không thuộc schema chuẩn).
const PROBE_SQL: &str = "create table probe (
  id bigint generated always as identity primary key,
  created_at timestamptz not null default now())";

/// Schema lab. Gọi [`LabDb::drop_schema`] khi xong, kể cả khi lỗi.
pub struct LabDb {
    /// Tên schema `lab_<uuid>`.
    pub schema: String,
    /// Tùy chọn kết nối thẳng, đã đặt `search_path` về schema lab.
    pub options: PgConnectOptions,
    admin: PgConnection,
}

impl LabDb {
    /// Tạo schema và bảng. Có thể cài thêm constraint trigger cân bút toán của bài 25.
    ///
    /// # Errors
    /// Thiếu `TOI_NOI_PG_URL`, không kết nối được, hoặc DDL lỗi.
    pub async fn create(with_balance_trigger: bool) -> Result<LabDb, BoxError> {
        let url = std::env::var("TOI_NOI_PG_URL").map_err(|_| {
            "thiếu TOI_NOI_PG_URL. Bật hạ tầng: docker compose -f examples/toi-noi-rust/infra/docker-compose.yml up -d --wait, \
             rồi export TOI_NOI_PG_URL như trong infra/README.md"
        })?;
        let base = PgConnectOptions::from_str(&url)?;
        let mut admin = PgConnection::connect_with(&base)
            .await
            .map_err(|e| format!("không kết nối được PostgreSQL ({e}); compose đã chạy chưa?"))?;
        let schema = format!("lab_{}", Uuid::new_v4().simple());
        // Tên schema do ta sinh từ UUID (chỉ chữ và số), không có đầu vào ngoài.
        sqlx::raw_sql(sqlx::AssertSqlSafe(format!("create schema {schema}")))
            .execute(&mut admin)
            .await?;
        let options = base.options([("search_path", schema.as_str())]);
        let db = LabDb {
            schema,
            options,
            admin,
        };
        if let Err(e) = db.setup(with_balance_trigger).await {
            db.drop_schema().await;
            return Err(e);
        }
        Ok(db)
    }

    async fn setup(&self, with_balance_trigger: bool) -> Result<(), BoxError> {
        let mut conn = self.connect().await?;
        sqlx::raw_sql(SCHEMA_SQL).execute(&mut conn).await?;
        sqlx::raw_sql(INDEX_SQL).execute(&mut conn).await?;
        sqlx::raw_sql(PROBE_SQL).execute(&mut conn).await?;
        if with_balance_trigger {
            sqlx::raw_sql(BALANCE_TRIGGER_SQL)
                .execute(&mut conn)
                .await?;
        }
        let (ids, kinds): (Vec<String>, Vec<&str>) = standard_accounts().into_iter().unzip();
        sqlx::query("insert into accounts (id, kind) select * from unnest($1::text[], $2::text[])")
            .bind(ids)
            .bind(kinds)
            .execute(&mut conn)
            .await?;
        conn.close().await?;
        Ok(())
    }

    /// Kết nối thẳng tới PostgreSQL, `search_path` là schema lab.
    ///
    /// # Errors
    /// Không kết nối được.
    pub async fn connect(&self) -> Result<PgConnection, sqlx::Error> {
        PgConnection::connect_with(&self.options).await
    }

    /// Kết nối qua proxy đếm message (tắt TLS để proxy đọc được khung giao thức).
    ///
    /// # Errors
    /// Không kết nối được.
    pub async fn connect_via(&self, proxy: &Proxy) -> Result<PgConnection, sqlx::Error> {
        self.connect_via_with_cache(proxy, 100).await
    }

    /// Như [`LabDb::connect_via`] nhưng đặt sức chứa cache prepared statement của kết nối
    /// (mặc định của sqlx là 100).
    ///
    /// # Errors
    /// Không kết nối được.
    pub async fn connect_via_with_cache(
        &self,
        proxy: &Proxy,
        capacity: usize,
    ) -> Result<PgConnection, sqlx::Error> {
        let options = self
            .options
            .clone()
            .host("127.0.0.1")
            .port(proxy.port)
            .ssl_mode(PgSslMode::Disable)
            .statement_cache_capacity(capacity);
        PgConnection::connect_with(&options).await
    }

    /// Mở proxy tới đúng máy chủ PostgreSQL của lab.
    ///
    /// # Errors
    /// Không bind được cổng.
    pub async fn proxy(&self) -> std::io::Result<Proxy> {
        Proxy::start(self.options.get_host(), self.options.get_port()).await
    }

    /// Xóa bút toán, posting, số dư; đặt lại identity để mọi lượt đo bắt đầu như nhau.
    ///
    /// # Errors
    /// Lỗi SQL.
    pub async fn reset(&self) -> Result<(), sqlx::Error> {
        let mut conn = self.connect().await?;
        sqlx::raw_sql(
            "truncate postings, journal_entries, account_balances, probe restart identity cascade",
        )
        .execute(&mut conn)
        .await?;
        conn.close().await
    }

    /// Xóa đúng schema của ca này. Gọi cả khi ca lỗi.
    pub async fn drop_schema(mut self) {
        let drop = format!("drop schema if exists {} cascade", self.schema);
        if let Err(e) = sqlx::raw_sql(sqlx::AssertSqlSafe(drop))
            .execute(&mut self.admin)
            .await
        {
            eprintln!("không xóa được schema {}: {e}", self.schema);
        }
        let _ = self.admin.close().await;
    }
}

/// Trạng thái cuối để so giữa các cách ghi: số bút toán, số posting, tổng posting, và số dư.
#[derive(Debug, PartialEq, Eq)]
pub struct State {
    /// Số dòng `journal_entries`.
    pub entries: i64,
    /// Số dòng `postings`.
    pub postings: i64,
    /// Tổng `amount_dong` của mọi posting (phải bằng 0).
    pub posting_sum: i64,
    /// `(account_id, balance_dong)` theo thứ tự tài khoản, chỉ tài khoản có số dư.
    pub balances: Vec<(String, i64)>,
    /// Tổng `version` của `account_balances`: số lần hàng số dư bị ghi.
    pub balance_versions: i64,
}

/// Đọc trạng thái hiện tại của schema lab.
///
/// # Errors
/// Lỗi SQL.
pub async fn read_state(conn: &mut PgConnection) -> Result<State, sqlx::Error> {
    let entries = sqlx::query_scalar("select count(*) from journal_entries")
        .fetch_one(&mut *conn)
        .await?;
    let postings = sqlx::query_scalar("select count(*) from postings")
        .fetch_one(&mut *conn)
        .await?;
    let posting_sum =
        sqlx::query_scalar("select coalesce(sum(amount_dong), 0)::bigint from postings")
            .fetch_one(&mut *conn)
            .await?;
    let balances =
        sqlx::query_as("select account_id, balance_dong from account_balances order by account_id")
            .fetch_all(&mut *conn)
            .await?;
    let balance_versions =
        sqlx::query_scalar("select coalesce(sum(version), 0)::bigint from account_balances")
            .fetch_one(&mut *conn)
            .await?;
    Ok(State {
        entries,
        postings,
        posting_sum,
        balances,
        balance_versions,
    })
}

/// Số dư trong `account_balances` có khớp tổng posting tính lại không (số dư là projection).
///
/// # Errors
/// Lỗi SQL.
pub async fn balances_match_postings(conn: &mut PgConnection) -> Result<bool, sqlx::Error> {
    let mismatched: i64 = sqlx::query_scalar(
        "select count(*) from (
           select coalesce(p.account_id, b.account_id) as id, p.s, b.balance_dong
           from (select account_id, sum(amount_dong)::bigint as s from postings group by account_id) p
           full join account_balances b on b.account_id = p.account_id
           where p.s is distinct from b.balance_dong) x",
    )
    .fetch_one(conn)
    .await?;
    Ok(mismatched == 0)
}
