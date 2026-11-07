//! Bốn cách ghi cùng một dữ liệu. Cả bốn giữ idempotency (`ON CONFLICT DO NOTHING`)
//! và cập nhật `account_balances` trong cùng transaction với bút toán.

use std::collections::HashMap;
use std::io::Write as _;

use ledger_core::{JournalEntry, Posting};
use sqlx::{Connection, PgConnection, Postgres, Transaction};

use crate::account::escape_copy_text;
use crate::columns::{Columns, dedupe};

const INSERT_ENTRY_ROW: &str = "insert into journal_entries (idempotency_key) values ($1)
     on conflict (idempotency_key) do nothing returning id";
const INSERT_POSTING_ROW: &str =
    "insert into postings (entry_id, line_no, account_id, amount_dong) values ($1, $2, $3, $4)";
const UPSERT_BALANCE_ROW: &str = "insert into account_balances (account_id, balance_dong)
     values ($1, $2)
     on conflict (account_id) do update
       set balance_dong = account_balances.balance_dong + excluded.balance_dong,
           version = account_balances.version + 1, updated_at = now()";

const INSERT_ENTRIES_BATCH: &str = "insert into journal_entries (idempotency_key)
     select * from unnest($1::text[])
     on conflict (idempotency_key) do nothing returning id, idempotency_key";
const INSERT_POSTINGS_UNNEST: &str =
    "insert into postings (entry_id, line_no, account_id, amount_dong)
     select * from unnest($1::bigint[], $2::int[], $3::text[], $4::bigint[])";
const UPSERT_BALANCES_BATCH: &str = "insert into account_balances (account_id, balance_dong)
     select * from unnest($1::text[], $2::bigint[])
     on conflict (account_id) do update
       set balance_dong = account_balances.balance_dong + excluded.balance_dong,
           version = account_balances.version + 1, updated_at = now()";
const COPY_POSTINGS: &str = "copy postings (entry_id, line_no, account_id, amount_dong) from stdin";

// Một câu lệnh duy nhất: ba CTE ghi dữ liệu chạy trong một round trip.
const WRITE_ALL_CTE: &str = "with new_entries as (
       insert into journal_entries (idempotency_key)
       select * from unnest($1::text[])
       on conflict (idempotency_key) do nothing
       returning id, idempotency_key),
     new_postings as (
       insert into postings (entry_id, line_no, account_id, amount_dong)
       select ne.id, t.line_no, t.account_id, t.amount
       from unnest($2::text[], $3::int[], $4::text[], $5::bigint[])
            as t(key, line_no, account_id, amount)
       join new_entries ne on ne.idempotency_key = t.key
       returning account_id, amount_dong)
     insert into account_balances (account_id, balance_dong)
     select account_id, sum(amount_dong)::bigint from new_postings
     group by account_id order by account_id
     on conflict (account_id) do update
       set balance_dong = account_balances.balance_dong + excluded.balance_dong,
           version = account_balances.version + 1, updated_at = now()";

const BARE_POSTINGS: &str = "insert into postings (entry_id, line_no, account_id, amount_dong)
     select je.id, t.line_no, t.account_id, t.amount
     from unnest($1::text[], $2::int[], $3::text[], $4::bigint[])
          as t(key, line_no, account_id, amount)
     join journal_entries je on je.idempotency_key = t.key";

/// Cách ghi.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Strategy {
    /// Mỗi bút toán một transaction, mỗi dòng một `INSERT` posting và một `UPSERT` số dư.
    PerRow,
    /// Mỗi lô một transaction; bút toán, posting, số dư mỗi thứ một câu lệnh mảng (`unnest`).
    Unnest(usize),
    /// Như `Unnest` nhưng posting đi bằng `COPY ... FROM STDIN` (`copy_in_raw`).
    Copy(usize),
    /// Mỗi lô đúng một câu lệnh: ba CTE ghi, không `BEGIN`/`COMMIT` riêng.
    Cte(usize),
    /// Như cách 3 của bài 30, để so cùng tải: hai câu lệnh mỗi lô, không `ON CONFLICT`,
    /// không `RETURNING`, không cập nhật số dư. Chỉ dùng khi đo, không giữ bảo đảm của ba cách trên.
    Bare(usize),
}

impl Strategy {
    /// Nhãn in ra.
    pub fn label(self) -> String {
        match self {
            Strategy::PerRow => "từng dòng, 1 transaction/bút toán".to_string(),
            Strategy::Unnest(b) => format!("lô {b}, unnest 3 câu lệnh"),
            Strategy::Copy(b) => format!("lô {b}, COPY posting"),
            Strategy::Cte(b) => format!("lô {b}, 1 câu lệnh CTE"),
            Strategy::Bare(b) => format!("lô {b}, như bài 30 (không số dư)"),
        }
    }

    /// Số round trip mong đợi cho `new_entries` bút toán mới (kết nối đã khởi động,
    /// câu lệnh đã prepare sẵn): đếm từ thiết kế, kiểm lại bằng proxy.
    pub fn expected_round_trips(self, new_entries: usize) -> u64 {
        let batches = |b: usize| new_entries.div_ceil(b) as u64;
        match self {
            // BEGIN, entry, 3 × (posting + số dư), COMMIT.
            Strategy::PerRow => 9 * new_entries as u64,
            // BEGIN, entry, posting, số dư, COMMIT.
            Strategy::Unnest(b) => 5 * batches(b),
            // Như trên, nhưng posting tốn hai lượt: chờ CopyInResponse, rồi chờ kết thúc COPY.
            Strategy::Copy(b) => 6 * batches(b),
            Strategy::Cte(b) => batches(b),
            // BEGIN, entry, posting, COMMIT.
            Strategy::Bare(b) => 4 * batches(b),
        }
    }
}

/// Ghi `entries` bằng `strategy` qua `conn`. Bút toán đã có (cùng key) bị bỏ qua.
///
/// # Errors
/// Lỗi SQL; transaction đang mở được rollback khi `Transaction` bị drop.
pub async fn write(
    conn: &mut PgConnection,
    strategy: Strategy,
    entries: &[JournalEntry],
) -> Result<(), sqlx::Error> {
    match strategy {
        Strategy::PerRow => per_row(conn, entries).await,
        Strategy::Unnest(b) => batched(conn, entries, b, false).await,
        Strategy::Copy(b) => batched(conn, entries, b, true).await,
        Strategy::Cte(b) => cte(conn, entries, b).await,
        Strategy::Bare(b) => bare(conn, entries, b).await,
    }
}

async fn per_row(conn: &mut PgConnection, entries: &[JournalEntry]) -> Result<(), sqlx::Error> {
    for entry in entries {
        let mut tx = conn.begin().await?;
        let id: Option<i64> = sqlx::query_scalar(INSERT_ENTRY_ROW)
            .bind(entry.idempotency_key())
            .fetch_optional(&mut *tx)
            .await?;
        if let Some(id) = id {
            // Khóa hàng số dư theo thứ tự mã tài khoản: hai bút toán chạm cùng tài khoản
            // không thể khóa chéo nhau (deadlock).
            let mut lines: Vec<(usize, &Posting)> = entry.postings().iter().enumerate().collect();
            lines.sort_by(|a, b| a.1.account.as_str().cmp(b.1.account.as_str()));
            for (index, posting) in lines {
                sqlx::query(INSERT_POSTING_ROW)
                    .bind(id)
                    .bind(i32::try_from(index + 1).expect("bút toán ít dòng"))
                    .bind(posting.account.as_str())
                    .bind(posting.amount.dong())
                    .execute(&mut *tx)
                    .await?;
                sqlx::query(UPSERT_BALANCE_ROW)
                    .bind(posting.account.as_str())
                    .bind(posting.amount.dong())
                    .execute(&mut *tx)
                    .await?;
            }
        }
        tx.commit().await?;
    }
    Ok(())
}

/// Các dòng posting của những bút toán thật sự mới, cộng độ biến thiên số dư theo tài khoản.
struct NewRows<'a> {
    entry_ids: Vec<i64>,
    line_nos: Vec<i32>,
    accounts: Vec<&'a str>,
    amounts: Vec<i64>,
    // Đã gộp theo tài khoản và sắp xếp theo mã: mỗi tài khoản một lần ghi, khóa theo thứ tự cố định.
    delta_accounts: Vec<&'a str>,
    delta_amounts: Vec<i64>,
}

impl<'a> NewRows<'a> {
    /// `ids` là id vừa được cấp cho các key mới; `remove` để key trùng trong lô chỉ ghi một lần.
    fn build(chunk: &'a [JournalEntry], ids: &mut HashMap<String, i64>) -> NewRows<'a> {
        let mut rows = NewRows {
            entry_ids: Vec::new(),
            line_nos: Vec::new(),
            accounts: Vec::new(),
            amounts: Vec::new(),
            delta_accounts: Vec::new(),
            delta_amounts: Vec::new(),
        };
        let mut deltas: Vec<(&str, i64)> = Vec::new();
        for entry in chunk {
            let Some(id) = ids.remove(entry.idempotency_key()) else {
                continue; // key đã có từ trước, hoặc trùng trong lô
            };
            for (i, p) in entry.postings().iter().enumerate() {
                rows.entry_ids.push(id);
                rows.line_nos
                    .push(i32::try_from(i + 1).expect("bút toán ít dòng"));
                rows.accounts.push(p.account.as_str());
                rows.amounts.push(p.amount.dong());
                deltas.push((p.account.as_str(), p.amount.dong()));
            }
        }
        deltas.sort_unstable_by(|a, b| a.0.cmp(b.0));
        for (account, amount) in deltas {
            if rows.delta_accounts.last() == Some(&account) {
                let last = rows.delta_amounts.last_mut().expect("cùng độ dài");
                *last = last.checked_add(amount).expect("không tràn i64");
            } else {
                rows.delta_accounts.push(account);
                rows.delta_amounts.push(amount);
            }
        }
        rows
    }

    /// Posting dạng text của `COPY`: bốn cột cách nhau bằng tab, mỗi dòng kết thúc bằng `\n`.
    fn copy_text(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.entry_ids.len() * 40);
        for i in 0..self.entry_ids.len() {
            writeln!(
                buf,
                "{}\t{}\t{}\t{}",
                self.entry_ids[i],
                self.line_nos[i],
                escape_copy_text(self.accounts[i]),
                self.amounts[i]
            )
            .expect("ghi vào Vec<u8> không lỗi");
        }
        buf
    }
}

async fn copy_postings(
    tx: &mut Transaction<'_, Postgres>,
    rows: &NewRows<'_>,
) -> Result<u64, sqlx::Error> {
    let mut copy = tx.copy_in_raw(COPY_POSTINGS).await?;
    copy.send(rows.copy_text()).await?;
    copy.finish().await
}

async fn batched(
    conn: &mut PgConnection,
    entries: &[JournalEntry],
    batch: usize,
    use_copy: bool,
) -> Result<(), sqlx::Error> {
    for chunk in entries.chunks(batch) {
        let keys: Vec<&str> = chunk.iter().map(JournalEntry::idempotency_key).collect();
        let mut tx = conn.begin().await?;
        let fresh: Vec<(i64, String)> = sqlx::query_as(INSERT_ENTRIES_BATCH)
            .bind(&keys[..])
            .fetch_all(&mut *tx)
            .await?;
        if !fresh.is_empty() {
            let mut ids: HashMap<String, i64> = fresh.into_iter().map(|(id, k)| (k, id)).collect();
            let rows = NewRows::build(chunk, &mut ids);
            if use_copy {
                copy_postings(&mut tx, &rows).await?;
            } else {
                sqlx::query(INSERT_POSTINGS_UNNEST)
                    .bind(&rows.entry_ids[..])
                    .bind(&rows.line_nos[..])
                    .bind(&rows.accounts[..])
                    .bind(&rows.amounts[..])
                    .execute(&mut *tx)
                    .await?;
            }
            sqlx::query(UPSERT_BALANCES_BATCH)
                .bind(&rows.delta_accounts[..])
                .bind(&rows.delta_amounts[..])
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;
    }
    Ok(())
}

async fn cte(
    conn: &mut PgConnection,
    entries: &[JournalEntry],
    batch: usize,
) -> Result<(), sqlx::Error> {
    for chunk in entries.chunks(batch) {
        // Câu lệnh nối posting với bút toán mới theo key, nên key trùng trong lô phải bỏ trước.
        let uniq = dedupe(chunk);
        let cols = Columns::borrow(&uniq);
        sqlx::query(WRITE_ALL_CTE)
            .bind(&cols.keys[..])
            .bind(&cols.key_of_line[..])
            .bind(&cols.line_nos[..])
            .bind(&cols.accounts[..])
            .bind(&cols.amounts[..])
            .execute(&mut *conn)
            .await?;
    }
    Ok(())
}

async fn bare(
    conn: &mut PgConnection,
    entries: &[JournalEntry],
    batch: usize,
) -> Result<(), sqlx::Error> {
    for chunk in entries.chunks(batch) {
        let uniq = dedupe(chunk);
        let cols = Columns::borrow(&uniq);
        let mut tx = conn.begin().await?;
        sqlx::query(
            "insert into journal_entries (idempotency_key) select * from unnest($1::text[])",
        )
        .bind(&cols.keys[..])
        .execute(&mut *tx)
        .await?;
        sqlx::query(BARE_POSTINGS)
            .bind(&cols.key_of_line[..])
            .bind(&cols.line_nos[..])
            .bind(&cols.accounts[..])
            .bind(&cols.amounts[..])
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
    }
    Ok(())
}

/// Thử đẩy một bút toán lệch bằng `COPY` thẳng vào `postings` rồi commit.
/// Trả SQLSTATE của lỗi: constraint trigger cân bút toán vẫn chạy theo từng dòng của `COPY`.
///
/// # Errors
/// Lỗi không phải do trigger (kết nối, SQL).
pub async fn copy_unbalanced_probe(conn: &mut PgConnection) -> Result<Option<String>, sqlx::Error> {
    let mut tx = conn.begin().await?;
    let id: i64 = sqlx::query_scalar(
        "insert into journal_entries (idempotency_key) values ('probe:unbalanced') returning id",
    )
    .fetch_one(&mut *tx)
    .await?;
    let mut copy = tx.copy_in_raw(COPY_POSTINGS).await?;
    // Hai dòng +100 và -40: tổng 60, lệch.
    let text = format!("{id}\t1\tplatform:revenue\t100\n{id}\t2\tdriver:DR-001\t-40\n");
    copy.send(text.into_bytes()).await?;
    copy.finish().await?;
    match tx.commit().await {
        Ok(()) => Ok(None),
        Err(sqlx::Error::Database(e)) => Ok(e.code().map(|c| c.into_owned())),
        Err(other) => Err(other),
    }
}

/// Thống kê hàm trigger `check_entry_balanced` trong schema hiện tại, từ server:
/// `(số lần chạy, tổng ms tự thân trong hàm)`. Cần `set track_functions = 'pl'` trên phiên đã ghi
/// và gọi hàm này trên cùng kết nối. Số lần chạy không phụ thuộc tải; số ms thì có.
///
/// # Errors
/// Lỗi SQL.
pub async fn trigger_stats(conn: &mut PgConnection) -> Result<(i64, f64), sqlx::Error> {
    // Đẩy thống kê của phiên này lên server ở cuối câu lệnh kế tiếp rồi mới đọc.
    sqlx::query("select pg_stat_force_next_flush()")
        .execute(&mut *conn)
        .await?;
    sqlx::query_as(
        "select coalesce(sum(calls), 0)::bigint, coalesce(sum(self_time), 0)::float8
         from pg_stat_user_functions where schemaname = current_schema()",
    )
    .fetch_one(conn)
    .await
}
