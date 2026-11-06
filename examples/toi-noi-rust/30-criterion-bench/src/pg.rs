//! Đo ghi PostgreSQL thật qua sqlx: schema riêng `lab_<uuid>`, nhiều cách ghi cùng một dữ liệu.

use std::error::Error;
use std::time::{Duration, Instant};

use ledger_core::{JournalEntry, Money};
use sqlx::postgres::PgPoolOptions;
use sqlx::{AssertSqlSafe, PgPool, Postgres, Transaction};
use tokio::task::JoinSet;
use uuid::Uuid;

pub type BoxError = Box<dyn Error + Send + Sync>;

/// Schema chuẩn của loạt (rút gọn: chưa có `account_balances`, xem bài 25).
const DDL: &str = "
create table accounts (
  id text primary key,
  kind text not null check (kind in ('asset','liability','revenue','expense')),
  created_at timestamptz not null default now());
create table journal_entries (
  id bigint generated always as identity primary key,
  idempotency_key text not null unique,
  reversal_of bigint references journal_entries(id),
  posted_at timestamptz not null default now());
create table postings (
  entry_id bigint not null references journal_entries(id),
  line_no int not null,
  account_id text not null references accounts(id),
  amount_dong bigint not null check (amount_dong <> 0),
  primary key (entry_id, line_no));
";

/// Mở pool; mỗi kết nối mới đặt `search_path` về schema lab (và `synchronous_commit` nếu cần).
async fn open_pool(
    url: &str,
    schema: &str,
    max_connections: u32,
    sync_commit: Option<&'static str>,
) -> Result<PgPool, sqlx::Error> {
    let mut setup = format!("set search_path to {schema}");
    if let Some(value) = sync_commit {
        setup.push_str(&format!("; set synchronous_commit to {value}"));
    }
    PgPoolOptions::new()
        .max_connections(max_connections)
        .after_connect(move |conn, _meta| {
            let stmt = setup.clone();
            Box::pin(async move {
                // `setup` ghép từ UUID do ta sinh và hằng 'on'/'off', không có đầu vào ngoài.
                sqlx::raw_sql(AssertSqlSafe(stmt)).execute(conn).await?;
                Ok(())
            })
        })
        .connect(url)
        .await
}

/// Một database lab: schema riêng, pool đặt `search_path` ngay khi mở kết nối.
pub struct LabDb {
    pub pool: PgPool,
    pub schema: String,
    url: String,
    admin: PgPool,
}

impl LabDb {
    /// Tạo schema `lab_<uuid>` và bảng. Thiếu `TOI_NOI_PG_URL` hoặc không kết nối được thì lỗi rõ.
    ///
    /// # Errors
    /// Biến môi trường thiếu, PostgreSQL không chạy, hoặc DDL lỗi.
    pub async fn create(max_connections: u32) -> Result<LabDb, BoxError> {
        let url = std::env::var("TOI_NOI_PG_URL").map_err(|_| {
            "thiếu TOI_NOI_PG_URL. Bật hạ tầng: docker compose -f examples/toi-noi-rust/infra/docker-compose.yml up -d --wait, \
             rồi export TOI_NOI_PG_URL như trong infra/README.md"
        })?;
        let admin = PgPoolOptions::new()
            .max_connections(1)
            .connect(&url)
            .await
            .map_err(|e| format!("không kết nối được PostgreSQL ({e}); compose đã chạy chưa?"))?;
        let schema = format!("lab_{}", Uuid::new_v4().simple());
        // Tên schema do ta sinh từ UUID (chỉ chữ và số), không có đầu vào ngoài: sqlx 0.9 buộc khẳng định điều đó.
        sqlx::raw_sql(AssertSqlSafe(format!("create schema {schema}")))
            .execute(&admin)
            .await?;
        let pool = open_pool(&url, &schema, max_connections, None).await?;
        let db = LabDb {
            pool,
            schema,
            url,
            admin,
        };
        if let Err(e) = sqlx::raw_sql(DDL).execute(&db.pool).await {
            db.drop_schema().await;
            return Err(e.into());
        }
        Ok(db)
    }

    /// Pool thứ hai trên cùng schema, đặt `synchronous_commit` cho mọi kết nối của nó
    /// (cài đặt của phiên, không đổi cấu hình server).
    ///
    /// # Errors
    /// Không kết nối được.
    pub async fn pool_with_sync_commit(
        &self,
        max_connections: u32,
        value: &'static str,
    ) -> Result<PgPool, sqlx::Error> {
        open_pool(&self.url, &self.schema, max_connections, Some(value)).await
    }

    /// Xóa đúng schema của ca này. Gọi cả khi ca lỗi.
    pub async fn drop_schema(self) {
        self.pool.close().await;
        let drop = format!("drop schema if exists {} cascade", self.schema);
        if let Err(e) = sqlx::raw_sql(AssertSqlSafe(drop))
            .execute(&self.admin)
            .await
        {
            eprintln!("không xóa được schema {}: {e}", self.schema);
        }
        self.admin.close().await;
    }

    /// Thêm mọi tài khoản xuất hiện trong `entries` (FK của `postings` cần chúng).
    ///
    /// # Errors
    /// Lỗi SQL.
    pub async fn seed_accounts(&self, entries: &[JournalEntry]) -> Result<(), sqlx::Error> {
        let mut ids: Vec<String> = entries
            .iter()
            .flat_map(JournalEntry::postings)
            .map(|p| p.account.as_str().to_string())
            .collect();
        ids.sort();
        ids.dedup();
        sqlx::query(
            "insert into accounts (id, kind)
             select id, case when id like 'platform:%' then 'revenue' else 'liability' end
             from unnest($1::text[]) as t(id) on conflict do nothing",
        )
        .bind(ids)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Xóa bút toán và posting, đặt lại identity để mọi lượt đo bắt đầu giống nhau.
    ///
    /// # Errors
    /// Lỗi SQL.
    pub async fn reset(&self) -> Result<(), sqlx::Error> {
        sqlx::raw_sql("truncate postings, journal_entries restart identity cascade")
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

/// Ba cách ghi cần so (cách 4 chỉ để giải thích fsync).
#[derive(Debug, Clone, Copy)]
pub enum Strategy {
    /// Mỗi bút toán một transaction: `BEGIN`, 2 `INSERT`, `COMMIT`.
    PerEntryTx,
    /// Một transaction cho mỗi lô; trong lô vẫn 2 câu lệnh cho mỗi bút toán.
    Batched(usize),
    /// Một transaction cho mỗi lô và đúng 2 câu lệnh cho cả lô (mảng + `unnest`).
    BatchedUnnest(usize),
    /// Mỗi bút toán một transaction, nhưng chạy song song trên nhiều kết nối.
    ConcurrentPerEntryTx(usize),
}

impl Strategy {
    pub fn label(self) -> String {
        match self {
            Strategy::PerEntryTx => "1 transaction / bút toán".to_string(),
            Strategy::Batched(n) => format!("lô {n}, 2 câu lệnh / bút toán"),
            Strategy::BatchedUnnest(n) => format!("lô {n}, 2 câu lệnh / lô (unnest)"),
            Strategy::ConcurrentPerEntryTx(k) => format!("1 transaction / bút toán, {k} kết nối"),
        }
    }
}

/// Kết quả một lượt ghi.
#[derive(Debug, Clone)]
pub struct PgRun {
    pub strategy: Strategy,
    pub entries: usize,
    pub elapsed: Duration,
}

impl PgRun {
    #[allow(clippy::cast_precision_loss)]
    pub fn per_second(&self) -> f64 {
        self.entries as f64 / self.elapsed.as_secs_f64()
    }
}

async fn insert_postings(
    tx: &mut Transaction<'_, Postgres>,
    entry_id: i64,
    entry: &JournalEntry,
) -> Result<(), sqlx::Error> {
    let (mut lines, mut accounts, mut amounts) = (Vec::new(), Vec::new(), Vec::new());
    for (i, p) in entry.postings().iter().enumerate() {
        lines.push(i32::try_from(i + 1).expect("ít posting"));
        accounts.push(p.account.as_str().to_string());
        amounts.push(p.amount.dong());
    }
    sqlx::query(
        "insert into postings (entry_id, line_no, account_id, amount_dong)
         select $1, t.line_no, t.account_id, t.amount
         from unnest($2::int[], $3::text[], $4::bigint[]) as t(line_no, account_id, amount)",
    )
    .bind(entry_id)
    .bind(lines)
    .bind(accounts)
    .bind(amounts)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn insert_one(
    tx: &mut Transaction<'_, Postgres>,
    entry: &JournalEntry,
) -> Result<(), sqlx::Error> {
    let id: i64 = sqlx::query_scalar(
        "insert into journal_entries (idempotency_key) values ($1) returning id",
    )
    .bind(entry.idempotency_key())
    .fetch_one(&mut **tx)
    .await?;
    insert_postings(tx, id, entry).await
}

async fn per_entry_tx(pool: &PgPool, entries: &[JournalEntry]) -> Result<(), sqlx::Error> {
    for entry in entries {
        let mut tx = pool.begin().await?;
        insert_one(&mut tx, entry).await?;
        tx.commit().await?;
    }
    Ok(())
}

async fn batched(pool: &PgPool, entries: &[JournalEntry], size: usize) -> Result<(), sqlx::Error> {
    for chunk in entries.chunks(size) {
        let mut tx = pool.begin().await?;
        for entry in chunk {
            insert_one(&mut tx, entry).await?;
        }
        tx.commit().await?;
    }
    Ok(())
}

async fn batched_unnest(
    pool: &PgPool,
    entries: &[JournalEntry],
    size: usize,
) -> Result<(), sqlx::Error> {
    for chunk in entries.chunks(size) {
        let (mut keys, mut key_of_line) = (Vec::new(), Vec::new());
        let (mut lines, mut accounts, mut amounts) = (Vec::new(), Vec::new(), Vec::new());
        for entry in chunk {
            keys.push(entry.idempotency_key().to_string());
            for (i, p) in entry.postings().iter().enumerate() {
                key_of_line.push(entry.idempotency_key().to_string());
                lines.push(i32::try_from(i + 1).expect("ít posting"));
                accounts.push(p.account.as_str().to_string());
                amounts.push(p.amount.dong());
            }
        }
        let mut tx = pool.begin().await?;
        sqlx::query(
            "insert into journal_entries (idempotency_key) select * from unnest($1::text[])",
        )
        .bind(keys)
        .execute(&mut *tx)
        .await?;
        // Nối lại theo idempotency_key để không phụ thuộc thứ tự identity.
        sqlx::query(
            "insert into postings (entry_id, line_no, account_id, amount_dong)
             select je.id, t.line_no, t.account_id, t.amount
             from unnest($1::text[], $2::int[], $3::text[], $4::bigint[])
                  as t(key, line_no, account_id, amount)
             join journal_entries je on je.idempotency_key = t.key",
        )
        .bind(key_of_line)
        .bind(lines)
        .bind(accounts)
        .bind(amounts)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
    }
    Ok(())
}

async fn concurrent(pool: &PgPool, entries: &[JournalEntry], tasks: usize) -> Result<(), BoxError> {
    let per_task = entries.len().div_ceil(tasks);
    let mut set = JoinSet::new();
    for chunk in entries.chunks(per_task) {
        let (pool, chunk) = (pool.clone(), chunk.to_vec());
        set.spawn(async move { per_entry_tx(&pool, &chunk).await });
    }
    while let Some(done) = set.join_next().await {
        done??;
    }
    Ok(())
}

/// Ghi `entries` bằng `strategy` trên bảng rỗng và đo thời gian tường (wall clock).
///
/// # Errors
/// Lỗi SQL hoặc task bị hủy.
pub async fn run(
    db: &LabDb,
    strategy: Strategy,
    entries: &[JournalEntry],
) -> Result<PgRun, BoxError> {
    run_on(db, &db.pool, strategy, entries).await
}

/// Như [`run`] nhưng ghi qua `pool` do người gọi chọn (cùng schema với `db`).
///
/// # Errors
/// Lỗi SQL hoặc task bị hủy.
pub async fn run_on(
    db: &LabDb,
    pool: &PgPool,
    strategy: Strategy,
    entries: &[JournalEntry],
) -> Result<PgRun, BoxError> {
    db.reset().await?;
    write_all(pool, strategy, entries).await
}

/// Như [`run_on`] nhưng KHÔNG xóa bảng trước: dùng để thử ghi trùng key.
///
/// # Errors
/// Lỗi SQL (key trùng) hoặc task bị hủy.
pub async fn run_on_existing(
    db: &LabDb,
    strategy: Strategy,
    entries: &[JournalEntry],
) -> Result<PgRun, BoxError> {
    write_all(&db.pool, strategy, entries).await
}

async fn write_all(
    pool: &PgPool,
    strategy: Strategy,
    entries: &[JournalEntry],
) -> Result<PgRun, BoxError> {
    let start = Instant::now();
    match strategy {
        Strategy::PerEntryTx => per_entry_tx(pool, entries).await?,
        Strategy::Batched(n) => batched(pool, entries, n).await?,
        Strategy::BatchedUnnest(n) => batched_unnest(pool, entries, n).await?,
        Strategy::ConcurrentPerEntryTx(k) => concurrent(pool, entries, k).await?,
    }
    Ok(PgRun {
        strategy,
        entries: entries.len(),
        elapsed: start.elapsed(),
    })
}

/// Kiểm kết quả nghiệp vụ sau một lượt ghi: đủ dòng, tổng bằng 0, số dư tài xế khớp.
///
/// # Errors
/// Lỗi SQL, hoặc `Err` mô tả điều kiện sai.
pub async fn check(
    db: &LabDb,
    expected_entries: usize,
    driver: &str,
    expected_driver: Money,
) -> Result<(), BoxError> {
    let entries: i64 = sqlx::query_scalar("select count(*) from journal_entries")
        .fetch_one(&db.pool)
        .await?;
    let postings: i64 = sqlx::query_scalar("select count(*) from postings")
        .fetch_one(&db.pool)
        .await?;
    let total: i64 =
        sqlx::query_scalar("select coalesce(sum(amount_dong), 0)::bigint from postings")
            .fetch_one(&db.pool)
            .await?;
    let driver_balance: i64 = sqlx::query_scalar(
        "select coalesce(sum(amount_dong), 0)::bigint from postings where account_id = $1",
    )
    .bind(driver)
    .fetch_one(&db.pool)
    .await?;
    let n = i64::try_from(expected_entries)?;
    if entries != n {
        return Err(format!("journal_entries: có {entries}, cần {n}").into());
    }
    if postings != 3 * n {
        return Err(format!("postings: có {postings}, cần {}", 3 * n).into());
    }
    if total != 0 {
        return Err(format!("tổng posting {total}, cần 0").into());
    }
    if driver_balance != expected_driver.dong() {
        return Err(format!(
            "số dư {driver}: {driver_balance}, cần {}",
            expected_driver.dong()
        )
        .into());
    }
    Ok(())
}

/// Dòng cấu hình PostgreSQL để in cùng số đo.
///
/// # Errors
/// Lỗi SQL.
pub async fn server_info(db: &LabDb) -> Result<String, sqlx::Error> {
    let version: String = sqlx::query_scalar("select split_part(version(), ' on ', 1)")
        .fetch_one(&db.pool)
        .await?;
    let fsync: String = sqlx::query_scalar("show fsync").fetch_one(&db.pool).await?;
    let sync_commit: String = sqlx::query_scalar("show synchronous_commit")
        .fetch_one(&db.pool)
        .await?;
    Ok(format!(
        "{version}; fsync={fsync}; synchronous_commit={sync_commit}"
    ))
}

/// Thời gian trung bình của một round trip (`select 1`) trên một kết nối, lặp `times` lần.
///
/// # Errors
/// Lỗi SQL.
pub async fn round_trip(pool: &PgPool, times: u32) -> Result<Duration, sqlx::Error> {
    let mut conn = pool.acquire().await?;
    let start = Instant::now();
    for _ in 0..times {
        sqlx::query_scalar::<_, i32>("select 1")
            .fetch_one(&mut *conn)
            .await?;
    }
    Ok(start.elapsed() / times)
}
