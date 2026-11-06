//! Đổi kiểu ở biên. sqlx 0.9 hỗ trợ `time` và `chrono` cho TIMESTAMPTZ, không hỗ trợ `jiff`,
//! nên ta đọc ra `time::OffsetDateTime` rồi đổi sang `jiff::Timestamp` qua nanô giây epoch.

use anyhow::{Context, Result};
use ledger_core::EntryId;
use time::OffsetDateTime;

/// TIMESTAMPTZ (đã đọc thành `OffsetDateTime`) sang `jiff::Timestamp`.
///
/// # Errors
/// Lỗi nếu thời điểm nằm ngoài miền của `jiff`.
pub fn to_jiff(at: OffsetDateTime) -> Result<jiff::Timestamp> {
    jiff::Timestamp::from_nanosecond(at.unix_timestamp_nanos())
        .context("thời điểm ngoài miền jiff::Timestamp")
}

/// Mã `JE-000001` từ khoá chính `bigint` của `journal_entries`.
///
/// # Errors
/// Lỗi nếu `raw` không dương.
pub fn entry_id_from_pk(raw: i64) -> Result<EntryId> {
    EntryId::parse(&format!("JE-{raw:06}"))
        .with_context(|| format!("id {raw} không thành JE-xxxxxx"))
}

/// Khoá chính `bigint` từ `JE-000001` (ledger-core không lộ số bên trong, nên đọc từ chuỗi hiển thị).
///
/// # Errors
/// Lỗi nếu chuỗi hiển thị không đúng dạng `JE-<số>`.
pub fn entry_pk(id: EntryId) -> Result<i64> {
    let shown = id.to_string();
    shown
        .strip_prefix("JE-")
        .and_then(|digits| digits.parse().ok())
        .with_context(|| format!("không đọc được số từ {shown}"))
}

/// Loại tài khoản cho cột `accounts.kind` (rút gọn cho lab; bài 25 giải thích thiết kế).
pub fn account_kind(account_id: &str) -> &'static str {
    match account_id.split(':').next() {
        Some("platform") => "revenue",
        Some("psp") => "asset",
        _ => "liability",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_round_trip_through_jiff() {
        let at = OffsetDateTime::from_unix_timestamp(1_793_000_000).unwrap();
        let ts = to_jiff(at).unwrap();
        assert_eq!(ts.as_second(), 1_793_000_000);
    }

    #[test]
    fn entry_pk_round_trip() {
        let id = entry_id_from_pk(1).unwrap();
        assert_eq!(id.to_string(), "JE-000001");
        assert_eq!(entry_pk(id).unwrap(), 1);
        assert!(entry_id_from_pk(0).is_err());
    }

    #[test]
    fn kinds_follow_account_prefix() {
        assert_eq!(account_kind("customer:CU-001"), "liability");
        assert_eq!(account_kind("platform:revenue"), "revenue");
        assert_eq!(account_kind("psp:vixanh:clearing"), "asset");
    }
}
