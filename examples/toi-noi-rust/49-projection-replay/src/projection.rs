//! Projection: hàm gộp từ log thành bảng đọc, có checkpoint, xử lý theo lô trong một
//! transaction, dựng lại từ đầu và chuyển phiên bản bằng đổi tên bảng.

use std::collections::BTreeMap;
use std::future::Future;
use std::time::Duration;

use ledger_core::Money;
use sqlx::{AssertSqlSafe, PgConnection, PgPool, Row};
use thiserror::Error;

use crate::log::{Cursor, LogEvent, ReadMode, head, horizon, read_batch};

/// Lỗi của projection.
#[derive(Debug, Error)]
pub enum ProjectionError {
    /// Lỗi SQL.
    #[error("lỗi database: {0}")]
    Db(#[from] sqlx::Error),
    /// Cộng số dư bị tràn `i64`.
    #[error("tràn số khi cộng dồn")]
    Overflow,
    /// Chạy lô cho phiên bản không đúng vai (đã bị thay thế, hoặc chưa được kích hoạt).
    #[error("projection {name} v{version}: active = {actual}, cần {expected}")]
    WrongRole {
        /// Tên projection.
        name: &'static str,
        /// Phiên bản.
        version: i32,
        /// Giá trị `active` trong database.
        actual: bool,
        /// Giá trị cần có để chạy ở vai này.
        expected: bool,
    },
    /// Bản đang dựng đã đi trước bản đang chạy: cho bản chạy bắt kịp rồi chuyển.
    #[error("bản dựng lại đã vượt checkpoint của bản đang chạy")]
    BuildAhead,
    /// Hai bảng khác nhau khi chuyển phiên bản cùng logic.
    #[error("bảng dựng lại khác bảng đang chạy {diff_rows} dòng, không chuyển")]
    Mismatch {
        /// Số dòng khác nhau.
        diff_rows: i64,
    },
    /// Khoảng an toàn không qua đầu log: có transaction treo.
    #[error("khoảng an toàn không tiến: có transaction mở quá lâu")]
    HorizonStuck,
}

/// Một projection: gộp `LogEvent` vào bảng của nó. Phiên bản mới = logic mới, dựng lại từ đầu.
pub trait Projection: Sync {
    /// Tên trong `projection_checkpoints`.
    fn name(&self) -> &'static str;
    /// Phiên bản logic.
    fn version(&self) -> i32;
    /// Tên bảng mà truy vấn đọc dùng (không kèm phiên bản).
    fn live_table(&self) -> &'static str;
    /// `CREATE TABLE` cho bảng tên `table`.
    fn create_table_sql(&self, table: &str) -> String;
    /// Gộp một lô sự kiện vào `table`, trong transaction của người gọi.
    fn apply(
        &self,
        conn: &mut PgConnection,
        table: &str,
        events: &[LogEvent],
    ) -> impl Future<Output = Result<(), ProjectionError>> + Send;

    /// Bảng tạm khi dựng lại phiên bản này, ví dụ `balances_v2`.
    fn build_table(&self) -> String {
        format!("{}_v{}", self.live_table(), self.version())
    }
}

/// Số dư từng tài khoản = tổng posting.
#[derive(Debug, Clone, Copy)]
pub struct Balances {
    /// Phiên bản logic.
    pub version: i32,
}

impl Projection for Balances {
    fn name(&self) -> &'static str {
        "balances"
    }
    fn version(&self) -> i32 {
        self.version
    }
    fn live_table(&self) -> &'static str {
        "balances"
    }
    fn create_table_sql(&self, table: &str) -> String {
        format!("CREATE TABLE {table} (account_id text PRIMARY KEY, balance_dong bigint NOT NULL)")
    }

    async fn apply(
        &self,
        conn: &mut PgConnection,
        table: &str,
        events: &[LogEvent],
    ) -> Result<(), ProjectionError> {
        // Gộp trong bộ nhớ trước, rồi một câu UPSERT cho cả lô.
        let mut delta: BTreeMap<&str, Money> = BTreeMap::new();
        for event in events {
            for (account, amount) in &event.postings {
                let slot = delta.entry(account.as_str()).or_insert(Money::ZERO);
                *slot = slot
                    .checked_add(Money::from_dong(*amount))
                    .ok_or(ProjectionError::Overflow)?;
            }
        }
        let accounts: Vec<&str> = delta.keys().copied().collect();
        let amounts: Vec<i64> = delta.values().map(|m| m.dong()).collect();
        sqlx::query(AssertSqlSafe(format!(
            "INSERT INTO {table} (account_id, balance_dong) \
             SELECT * FROM unnest($1::text[], $2::bigint[]) \
             ON CONFLICT (account_id) DO UPDATE \
             SET balance_dong = {table}.balance_dong + EXCLUDED.balance_dong"
        )))
        .bind(&accounts)
        .bind(&amounts)
        .execute(&mut *conn)
        .await?;
        Ok(())
    }
}

/// Thu nhập theo ngày của `platform:revenue` và các ví `driver:*`.
///
/// Phiên bản 1 chia ngày theo UTC (lỗi: chuyến lúc 00:30 giờ Việt Nam rơi vào ngày hôm trước).
/// Phiên bản 2 chia theo `Asia/Ho_Chi_Minh`.
#[derive(Debug, Clone, Copy)]
pub struct IncomeByDay {
    /// Phiên bản logic.
    pub version: i32,
}

/// Quy tắc thu nhập: posting dương vào tài khoản thu nhập của bút toán thường, và posting âm của
/// bút toán đảo (trừ lại đúng phần nó đảo). Chi ra khỏi ví tài xế (payout) không tính.
pub fn income_delta(event: &LogEvent, account: &str, amount: i64) -> i64 {
    let income_account = account == "platform:revenue" || account.starts_with("driver:");
    let counts = if event.reversal_of.is_none() {
        amount > 0
    } else {
        amount < 0
    };
    if income_account && counts { amount } else { 0 }
}

impl Projection for IncomeByDay {
    fn name(&self) -> &'static str {
        "income_by_day"
    }
    fn version(&self) -> i32 {
        self.version
    }
    fn live_table(&self) -> &'static str {
        "income_by_day"
    }
    fn create_table_sql(&self, table: &str) -> String {
        format!(
            "CREATE TABLE {table} (day date NOT NULL, account_id text NOT NULL, \
             income_dong bigint NOT NULL, PRIMARY KEY (day, account_id))"
        )
    }

    async fn apply(
        &self,
        conn: &mut PgConnection,
        table: &str,
        events: &[LogEvent],
    ) -> Result<(), ProjectionError> {
        let mut delta: BTreeMap<(&str, &str), Money> = BTreeMap::new();
        for event in events {
            let day = if self.version >= 2 {
                event.day_vn.as_str()
            } else {
                event.day_utc.as_str()
            };
            for (account, amount) in &event.postings {
                let counted = income_delta(event, account, *amount);
                if counted == 0 {
                    continue;
                }
                let slot = delta.entry((day, account.as_str())).or_insert(Money::ZERO);
                *slot = slot
                    .checked_add(Money::from_dong(counted))
                    .ok_or(ProjectionError::Overflow)?;
            }
        }
        let days: Vec<&str> = delta.keys().map(|(d, _)| *d).collect();
        let accounts: Vec<&str> = delta.keys().map(|(_, a)| *a).collect();
        let amounts: Vec<i64> = delta.values().map(|m| m.dong()).collect();
        sqlx::query(AssertSqlSafe(format!(
            "INSERT INTO {table} (day, account_id, income_dong) \
             SELECT d::date, a, v FROM unnest($1::text[], $2::text[], $3::bigint[]) AS u(d, a, v) \
             ON CONFLICT (day, account_id) DO UPDATE \
             SET income_dong = {table}.income_dong + EXCLUDED.income_dong"
        )))
        .bind(&days)
        .bind(&accounts)
        .bind(&amounts)
        .execute(&mut *conn)
        .await?;
        Ok(())
    }
}

/// Vai của một lần chạy: phục vụ đọc (`Live`) hay đang dựng lại (`Build`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// Phiên bản đang phục vụ truy vấn đọc (`active = true`).
    Live,
    /// Phiên bản đang dựng lại vào bảng riêng (`active = false`).
    Build,
}

/// Điểm giả lập sự cố trong `run_batch_with`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Crash {
    /// Không sự cố.
    No,
    /// Tiến trình chết trước COMMIT: bảng và checkpoint cùng bị rollback.
    BeforeCommit,
}

fn table_of<P: Projection>(p: &P, role: Role) -> String {
    match role {
        Role::Live => p.live_table().to_string(),
        Role::Build => p.build_table(),
    }
}

/// Tạo bảng đang chạy và dòng checkpoint `active` cho phiên bản này.
///
/// # Errors
/// Lỗi SQL.
pub async fn install<P: Projection>(pool: &PgPool, p: &P) -> Result<(), ProjectionError> {
    let mut tx = pool.begin().await?;
    sqlx::query(AssertSqlSafe(p.create_table_sql(p.live_table())))
        .execute(&mut *tx)
        .await?;
    sqlx::query("INSERT INTO projection_checkpoints (name, version, active) VALUES ($1, $2, true)")
        .bind(p.name())
        .bind(p.version())
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

/// Đọc checkpoint (và `active`) của một phiên bản, khoá dòng đó đến hết transaction:
/// hai worker cùng projection phải xếp hàng ở đây.
async fn lock_checkpoint(
    conn: &mut PgConnection,
    name: &str,
    version: i32,
) -> Result<(Cursor, bool), ProjectionError> {
    let row = sqlx::query(
        "SELECT last_tx_id, last_entry_id, active FROM projection_checkpoints \
         WHERE name = $1 AND version = $2 FOR UPDATE",
    )
    .bind(name)
    .bind(version)
    .fetch_one(&mut *conn)
    .await?;
    Ok((
        Cursor {
            tx_id: row.get("last_tx_id"),
            entry_id: row.get("last_entry_id"),
        },
        row.get("active"),
    ))
}

async fn save_checkpoint(
    conn: &mut PgConnection,
    name: &str,
    version: i32,
    cursor: Cursor,
) -> Result<(), ProjectionError> {
    sqlx::query(
        "UPDATE projection_checkpoints SET last_tx_id = $3, last_entry_id = $4 \
         WHERE name = $1 AND version = $2",
    )
    .bind(name)
    .bind(version)
    .bind(cursor.tx_id)
    .bind(cursor.entry_id)
    .execute(&mut *conn)
    .await?;
    Ok(())
}

/// Vị trí đã xử lý của một phiên bản (không khoá).
///
/// # Errors
/// Lỗi SQL, hoặc phiên bản chưa có checkpoint.
pub async fn checkpoint(
    pool: &PgPool,
    name: &str,
    version: i32,
) -> Result<Cursor, ProjectionError> {
    let row = sqlx::query(
        "SELECT last_tx_id, last_entry_id FROM projection_checkpoints WHERE name = $1 AND version = $2",
    )
    .bind(name)
    .bind(version)
    .fetch_one(pool)
    .await?;
    Ok(Cursor {
        tx_id: row.get("last_tx_id"),
        entry_id: row.get("last_entry_id"),
    })
}

/// Xử lý một lô: đọc, gộp vào bảng và ghi checkpoint trong CÙNG một transaction.
/// Trả số sự kiện đã áp dụng (0 nghĩa là chưa có gì mới đi qua khoảng an toàn).
///
/// # Errors
/// Lỗi SQL, hoặc phiên bản không đúng vai.
pub async fn run_batch<P: Projection>(
    pool: &PgPool,
    p: &P,
    role: Role,
    mode: ReadMode,
    limit: i64,
) -> Result<usize, ProjectionError> {
    run_batch_with(pool, p, role, mode, limit, Crash::No).await
}

/// Như [`run_batch`], có thể giả lập tiến trình chết trước COMMIT.
///
/// # Errors
/// Lỗi SQL, hoặc phiên bản không đúng vai.
pub async fn run_batch_with<P: Projection>(
    pool: &PgPool,
    p: &P,
    role: Role,
    mode: ReadMode,
    limit: i64,
    crash: Crash,
) -> Result<usize, ProjectionError> {
    let table = table_of(p, role);
    let mut tx = pool.begin().await?;
    let (cursor, active) = lock_checkpoint(&mut tx, p.name(), p.version()).await?;
    if active != (role == Role::Live) {
        return Err(ProjectionError::WrongRole {
            name: p.name(),
            version: p.version(),
            actual: active,
            expected: role == Role::Live,
        });
    }
    let events = read_batch(&mut tx, mode, cursor, None, limit).await?;
    let Some(last) = events.last().map(LogEvent::cursor) else {
        return Ok(0); // drop tx: không có gì để lưu
    };
    p.apply(&mut tx, &table, &events).await?;
    save_checkpoint(&mut tx, p.name(), p.version(), last).await?;
    if crash == Crash::BeforeCommit {
        return Ok(events.len()); // drop tx = ROLLBACK: bảng và checkpoint cùng quay lại
    }
    tx.commit().await?;
    Ok(events.len())
}

/// SAI, chỉ để minh họa: cộng vào bảng và COMMIT, rồi "chết" trước khi ghi checkpoint
/// (checkpoint lẽ ra phải nằm trong transaction này). Chạy lại sẽ cộng trùng lô.
///
/// # Errors
/// Lỗi SQL.
pub async fn apply_commit_without_checkpoint<P: Projection>(
    pool: &PgPool,
    p: &P,
    limit: i64,
) -> Result<usize, ProjectionError> {
    let mut tx = pool.begin().await?;
    let (cursor, _) = lock_checkpoint(&mut tx, p.name(), p.version()).await?;
    let events = read_batch(&mut tx, ReadMode::Safe, cursor, None, limit).await?;
    p.apply(&mut tx, p.live_table(), &events).await?;
    tx.commit().await?; // bảng đã đổi; tiến trình chết ở đây, checkpoint không bao giờ được ghi
    Ok(events.len())
}

/// Chờ khoảng an toàn vượt đầu log (mọi giao dịch của ta đã xong). Giao dịch của phiên khác
/// cũng giữ khoảng này, nên đây là chờ theo trạng thái, không theo thời gian cố định.
///
/// # Errors
/// [`ProjectionError::HorizonStuck`] nếu sau rất nhiều lần thử vẫn không tiến.
pub async fn wait_horizon_past_head(pool: &PgPool) -> Result<(), ProjectionError> {
    for _ in 0..6000 {
        let mut conn = pool.acquire().await?;
        let Some(head) = head(&mut conn).await? else {
            return Ok(());
        };
        if horizon(&mut conn).await? > head.tx_id {
            return Ok(());
        }
        drop(conn);
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    Err(ProjectionError::HorizonStuck)
}

/// Chạy lô cho tới khi hết sự kiện, sau khi khoảng an toàn đã vượt đầu log.
///
/// # Errors
/// Lỗi SQL, vai sai, hoặc khoảng an toàn không tiến.
pub async fn drain<P: Projection>(
    pool: &PgPool,
    p: &P,
    role: Role,
    mode: ReadMode,
    limit: i64,
) -> Result<usize, ProjectionError> {
    wait_horizon_past_head(pool).await?;
    let mut total = 0;
    loop {
        let applied = run_batch(pool, p, role, mode, limit).await?;
        if applied == 0 {
            return Ok(total);
        }
        total += applied;
    }
}

/// Bắt đầu dựng lại phiên bản `p`: tạo bảng riêng và dòng checkpoint `active = false`, con trỏ 0.
///
/// # Errors
/// Lỗi SQL (ví dụ phiên bản đã tồn tại).
pub async fn start_rebuild<P: Projection>(pool: &PgPool, p: &P) -> Result<(), ProjectionError> {
    let mut tx = pool.begin().await?;
    sqlx::query(AssertSqlSafe(p.create_table_sql(&p.build_table())))
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        "INSERT INTO projection_checkpoints (name, version, active) VALUES ($1, $2, false)",
    )
    .bind(p.name())
    .bind(p.version())
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

/// Số dòng khác nhau giữa hai bảng cùng cấu trúc (khác một chiều hoặc chiều kia).
///
/// # Errors
/// Lỗi SQL.
pub async fn diff_rows(
    conn: &mut PgConnection,
    left: &str,
    right: &str,
) -> Result<i64, ProjectionError> {
    let sql = format!(
        "SELECT count(*) FROM ((SELECT * FROM {left} EXCEPT SELECT * FROM {right}) \
         UNION ALL (SELECT * FROM {right} EXCEPT SELECT * FROM {left})) AS d"
    );
    Ok(sqlx::query_scalar(AssertSqlSafe(sql))
        .fetch_one(&mut *conn)
        .await?)
}

/// Kết quả chuyển phiên bản.
#[derive(Debug, Clone, Copy)]
pub struct CutoverReport {
    /// Số sự kiện bản mới phải bắt kịp bên trong transaction chuyển.
    pub caught_up: usize,
    /// Số dòng khác nhau giữa bảng cũ và bảng mới ngay trước khi đổi tên.
    pub diff_rows: i64,
}

/// Chuyển từ `old` (đang `active`) sang `new` (đã dựng lại) trong một transaction:
/// khoá cả hai checkpoint, cho bản mới bắt kịp đúng tới checkpoint của bản cũ, rồi đổi tên bảng
/// và lật `active`. Nếu `require_same`, hai bảng phải giống nhau hoàn toàn, nếu không thì hủy.
///
/// # Errors
/// Lỗi SQL, vai sai, `BuildAhead` hoặc `Mismatch`.
pub async fn cutover<O: Projection, N: Projection>(
    pool: &PgPool,
    old: &O,
    new: &N,
    require_same: bool,
) -> Result<CutoverReport, ProjectionError> {
    let mut tx = pool.begin().await?;
    let (old_cursor, old_active) = lock_checkpoint(&mut tx, old.name(), old.version()).await?;
    let (mut new_cursor, new_active) = lock_checkpoint(&mut tx, new.name(), new.version()).await?;
    if !old_active || new_active {
        return Err(ProjectionError::WrongRole {
            name: new.name(),
            version: new.version(),
            actual: new_active,
            expected: false,
        });
    }
    if new_cursor > old_cursor {
        return Err(ProjectionError::BuildAhead);
    }
    let new_table = new.build_table();
    let mut caught_up = 0;
    loop {
        let events = read_batch(&mut tx, ReadMode::Safe, new_cursor, Some(old_cursor), 500).await?;
        let Some(last) = events.last().map(LogEvent::cursor) else {
            break;
        };
        new.apply(&mut tx, &new_table, &events).await?;
        caught_up += events.len();
        new_cursor = last;
    }
    save_checkpoint(&mut tx, new.name(), new.version(), new_cursor).await?;
    let diff = diff_rows(&mut tx, old.live_table(), &new_table).await?;
    if require_same && diff != 0 {
        return Err(ProjectionError::Mismatch { diff_rows: diff });
    }
    // Đổi tên và lật cờ cùng transaction: người đọc thấy bảng cũ hoặc bảng mới, không có lúc nào ở giữa.
    let retired = format!("{}_v{}", old.live_table(), old.version());
    sqlx::query(AssertSqlSafe(format!(
        "ALTER TABLE {} RENAME TO {retired}",
        old.live_table()
    )))
    .execute(&mut *tx)
    .await?;
    sqlx::query(AssertSqlSafe(format!(
        "ALTER TABLE {new_table} RENAME TO {}",
        new.live_table()
    )))
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "UPDATE projection_checkpoints SET active = false WHERE name = $1 AND version = $2",
    )
    .bind(old.name())
    .bind(old.version())
    .execute(&mut *tx)
    .await?;
    sqlx::query("UPDATE projection_checkpoints SET active = true WHERE name = $1 AND version = $2")
        .bind(new.name())
        .bind(new.version())
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(CutoverReport {
        caught_up,
        diff_rows: diff,
    })
}
