//! Replay quyết định phí: dựng lại phí và phần tài xế từ sự kiện rồi so với bút toán đã ghi.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};

use crate::app::AppError;
use crate::policy::{PolicyVersion, split_for};
use ledger_core::Money;

/// Sự kiện quyết định phí của một chuyến, như đi qua dây: JSON camelCase có `schemaVersion`.
/// Nó mang phiên bản quy tắc, không mang số phí: phí tính lại được, quy tắc thì phải biết đúng bản.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeeEvent {
    /// Phiên bản hợp đồng sự kiện.
    pub schema_version: u8,
    /// Mã chuyến.
    pub trip_id: String,
    /// Cước của chuyến, đồng.
    pub fare_dong: i64,
    /// Phiên bản quy tắc phí đã dùng.
    pub policy_version: i32,
}

/// Đọc mọi sự kiện theo thứ tự ghi. `policy_version` `NULL` (chưa backfill) được hiểu là 1.
///
/// # Errors
/// Lỗi SQL.
pub async fn read_events(pool: &PgPool) -> Result<Vec<FeeEvent>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT trip_id, fare_dong, COALESCE(policy_version, 1) AS v FROM fee_events ORDER BY seq",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .iter()
        .map(|r| FeeEvent {
            schema_version: 1,
            trip_id: r.get("trip_id"),
            fare_dong: r.get("fare_dong"),
            policy_version: r.get("v"),
        })
        .collect())
}

/// Cách chọn quy tắc khi replay.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayMode {
    /// Đúng: dùng `policy_version` ghi trong sự kiện.
    Pinned,
    /// SAI để đối chứng: áp quy tắc hiện hành cho mọi sự kiện cũ.
    CurrentRule,
}

/// Một chuyến mà replay ra khác bút toán đã ghi.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TripDiff {
    /// Mã chuyến.
    pub trip_id: String,
    /// Phiên bản ghi trong sự kiện.
    pub event_version: i32,
    /// Phí do replay tính ra.
    pub expected_fee: i64,
    /// Phí trong bút toán đã ghi.
    pub actual_fee: i64,
    /// Phần tài xế do replay tính ra.
    pub expected_net: i64,
    /// Phần tài xế trong bút toán đã ghi.
    pub actual_net: i64,
}

/// Kết quả replay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayReport {
    /// Số sự kiện đã replay.
    pub trips: usize,
    /// Các chuyến lệch.
    pub diffs: Vec<TripDiff>,
    /// Tổng doanh thu nền tảng theo replay.
    pub expected_revenue: i64,
    /// Tổng posting của `platform:revenue` trong sổ.
    pub actual_revenue: i64,
}

impl ReplayReport {
    /// Replay khớp hoàn toàn với sổ.
    pub fn matches(&self) -> bool {
        self.diffs.is_empty() && self.expected_revenue == self.actual_revenue
    }
}

/// Replay mọi sự kiện và so với bút toán chia cước `trip:<id>:split` đã ghi. Chỉ đọc, không sửa gì.
///
/// # Errors
/// Lỗi SQL, `policy_version` lạ hoặc lỗi tính phí.
pub async fn replay(pool: &PgPool, mode: ReplayMode) -> Result<ReplayReport, AppError> {
    // Phí và phần tài xế thực tế của từng bút toán chia cước.
    let rows = sqlx::query(
        "SELECT e.idempotency_key AS key, p.account_id, p.amount_dong \
         FROM journal_entries e JOIN postings p ON p.entry_id = e.id \
         WHERE e.idempotency_key LIKE 'trip:%:split'",
    )
    .fetch_all(pool)
    .await?;
    let mut actual: HashMap<String, (i64, i64)> = HashMap::new();
    for row in &rows {
        let key: String = row.get("key");
        let trip = key
            .strip_prefix("trip:")
            .and_then(|k| k.strip_suffix(":split"))
            .unwrap_or(&key)
            .to_string();
        let account: String = row.get("account_id");
        let amount: i64 = row.get("amount_dong");
        let slot = actual.entry(trip).or_default();
        if account == "platform:revenue" {
            slot.0 = amount;
        } else if account.starts_with("driver:") {
            slot.1 = amount;
        }
    }

    let events = read_events(pool).await?;
    let mut diffs = Vec::new();
    let mut expected_revenue = 0;
    for event in &events {
        let recorded = PolicyVersion::from_number(event.policy_version)
            .ok_or(AppError::UnknownVersion(event.policy_version))?;
        let version = match mode {
            ReplayMode::Pinned => recorded,
            ReplayMode::CurrentRule => PolicyVersion::latest(),
        };
        let split = split_for(version, Money::from_dong(event.fare_dong))?;
        let (actual_fee, actual_net) = actual.get(&event.trip_id).copied().unwrap_or_default();
        expected_revenue += split.platform_fee.dong();
        if split.platform_fee.dong() != actual_fee || split.driver_net.dong() != actual_net {
            diffs.push(TripDiff {
                trip_id: event.trip_id.clone(),
                event_version: event.policy_version,
                expected_fee: split.platform_fee.dong(),
                actual_fee,
                expected_net: split.driver_net.dong(),
                actual_net,
            });
        }
    }
    let actual_revenue: i64 = sqlx::query_scalar(
        "SELECT COALESCE(sum(amount_dong), 0)::bigint FROM postings WHERE account_id = 'platform:revenue'",
    )
    .fetch_one(pool)
    .await?;
    Ok(ReplayReport {
        trips: events.len(),
        diffs,
        expected_revenue,
        actual_revenue,
    })
}
