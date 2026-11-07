//! Hold: trạng thái, lỗi nghiệp vụ và các bước chuyển trạng thái cần bằng chứng `ActiveHold`.

use std::str::FromStr;

use jiff::Timestamp;
use ledger_core::{AccountId, JournalEntry, LedgerError, Money, Posting};
use sqlx::{PgConnection, Row};
use thiserror::Error;
use time::OffsetDateTime;

use crate::ledger::{PSP_CLEARING, insert_entry};

/// Bốn trạng thái của hold; khớp `CHECK (status IN (...))` trong DB.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HoldStatus {
    /// Đang giữ tiền, trừ vào số dư khả dụng.
    Active,
    /// Đã capture (toàn phần hoặc một phần); phần dư đã nhả.
    Captured,
    /// Đã hủy trước khi capture.
    Voided,
    /// Quá hạn, job đã nhả.
    Expired,
}

impl HoldStatus {
    /// Chuỗi lưu trong cột `status`.
    pub const fn as_str(self) -> &'static str {
        match self {
            HoldStatus::Active => "ACTIVE",
            HoldStatus::Captured => "CAPTURED",
            HoldStatus::Voided => "VOIDED",
            HoldStatus::Expired => "EXPIRED",
        }
    }
}

impl FromStr for HoldStatus {
    type Err = HoldError;

    /// Biên DB: chuỗi lạ bị từ chối, không đoán.
    fn from_str(raw: &str) -> Result<HoldStatus, HoldError> {
        match raw {
            "ACTIVE" => Ok(HoldStatus::Active),
            "CAPTURED" => Ok(HoldStatus::Captured),
            "VOIDED" => Ok(HoldStatus::Voided),
            "EXPIRED" => Ok(HoldStatus::Expired),
            other => Err(HoldError::UnknownStatus(other.to_string())),
        }
    }
}

/// Lỗi của lớp hold: lỗi nghiệp vụ tách khỏi lỗi hạ tầng.
#[derive(Debug, Error)]
pub enum HoldError {
    /// Số dư khả dụng không đủ cho hold mới.
    #[error("không đủ số dư khả dụng: còn {available}, cần {requested}")]
    InsufficientAvailable {
        /// Khả dụng tại thời điểm kiểm.
        available: Money,
        /// Số xin giữ.
        requested: Money,
    },
    /// Ví chưa có hàng trong `account_balances`: không có gì để khóa, không đoán số dư là 0.
    #[error("ví {0} chưa có hàng số dư")]
    NoBalanceRow(String),
    /// Không có hold với id này.
    #[error("không có hold {0}")]
    NotFound(i64),
    /// Hold không còn ACTIVE nên không chuyển trạng thái được.
    #[error("hold {id} đang ở trạng thái {}", status.as_str())]
    NotActive {
        /// Id hold.
        id: i64,
        /// Trạng thái hiện tại.
        status: HoldStatus,
    },
    /// Capture đến sau `expires_at` theo đồng hồ truyền vào.
    #[error("hold {id} đã quá hạn lúc {expires_at}")]
    Expired {
        /// Id hold.
        id: i64,
        /// Hạn của hold.
        expires_at: Timestamp,
    },
    /// Capture nhiều hơn số đã giữ.
    #[error("capture {requested} vượt số đã giữ {hold}")]
    CaptureExceedsHold {
        /// Số của hold.
        hold: Money,
        /// Số xin capture.
        requested: Money,
    },
    /// Số tiền không dương.
    #[error("số tiền phải dương, nhận {0}")]
    InvalidAmount(i64),
    /// Cùng key nhưng nội dung khác bản đã ghi.
    #[error("idempotency_key {key} đã dùng cho nội dung khác")]
    IdempotencyConflict {
        /// Key bị trùng.
        key: String,
    },
    /// Cột `status` chứa chuỗi ngoài bốn trạng thái.
    #[error("trạng thái hold lạ: {0}")]
    UnknownStatus(String),
    /// Không đổi được thời điểm giữa `jiff` và PostgreSQL.
    #[error("thời điểm ngoài miền: {0}")]
    Time(String),
    /// Dữ liệu không dựng lại được thành bút toán hợp lệ.
    #[error(transparent)]
    Ledger(#[from] LedgerError),
    /// Lỗi từ PostgreSQL hoặc kết nối.
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

/// `jiff::Timestamp` sang `OffsetDateTime` để bind vào TIMESTAMPTZ.
///
/// # Errors
/// `HoldError::Time` nếu ngoài miền của `time`.
pub fn to_pg(at: Timestamp) -> Result<OffsetDateTime, HoldError> {
    OffsetDateTime::from_unix_timestamp_nanos(at.as_nanosecond())
        .map_err(|e| HoldError::Time(e.to_string()))
}

/// TIMESTAMPTZ (đã đọc thành `OffsetDateTime`) sang `jiff::Timestamp`.
///
/// # Errors
/// `HoldError::Time` nếu ngoài miền của `jiff`.
pub fn from_pg(at: OffsetDateTime) -> Result<Timestamp, HoldError> {
    Timestamp::from_nanosecond(at.unix_timestamp_nanos())
        .map_err(|e| HoldError::Time(e.to_string()))
}

/// Một dòng của bảng `holds`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HoldRow {
    /// Id do DB cấp.
    pub id: i64,
    /// Tài khoản ví bị giữ tiền.
    pub account: String,
    /// Số đã giữ.
    pub amount: Money,
    /// Số đã capture (0 khi còn ACTIVE).
    pub captured: Money,
    /// Trạng thái.
    pub status: HoldStatus,
    /// Hạn của hold.
    pub expires_at: Timestamp,
}

impl HoldRow {
    /// Phần giữ đã nhả: `amount - captured` khi hold đã CAPTURED; 0 nếu chưa capture.
    pub fn released(&self) -> Money {
        if self.status == HoldStatus::Captured {
            Money::from_dong(self.amount.dong() - self.captured.dong())
        } else {
            Money::ZERO
        }
    }

    pub(crate) fn from_row(row: &sqlx::postgres::PgRow) -> Result<HoldRow, HoldError> {
        let status: String = row.get("status");
        Ok(HoldRow {
            id: row.get("id"),
            account: row.get("account_id"),
            amount: Money::from_dong(row.get("amount_dong")),
            captured: Money::from_dong(row.get("captured_dong")),
            status: status.parse()?,
            expires_at: from_pg(row.get("expires_at"))?,
        })
    }
}

/// Kết quả khóa một hold.
#[derive(Debug)]
pub enum Locked {
    /// Hold còn ACTIVE và đang bị khóa bởi transaction này: bằng chứng để chuyển trạng thái.
    Active(ActiveHold),
    /// Hold đã đóng (CAPTURED, VOIDED, EXPIRED); người gọi tự quyết replay hay lỗi.
    Closed(HoldRow),
}

/// Bằng chứng «hold này đang ACTIVE và transaction hiện tại giữ khóa dòng của nó».
/// Không `Clone`, trường private, chỉ [`lock`] tạo ra; `capture` và `void` nhận `self`
/// nên mỗi bằng chứng dùng đúng một lần.
///
/// Lỗi E0382 nếu dùng lại sau `capture` (đã thử, rồi gỡ để lab build xanh):
/// `active.capture(...).await?; active.void(&mut tx).await?;`
#[derive(Debug)]
pub struct ActiveHold {
    id: i64,
    account: String,
    amount: Money,
    expires_at: Timestamp,
}

/// `SELECT ... FOR UPDATE` dòng hold. Người khác chuyển trạng thái cùng hold phải chờ transaction này.
///
/// # Errors
/// `NotFound` nếu không có dòng; lỗi SQL hoặc trạng thái lạ.
pub async fn lock(conn: &mut PgConnection, id: i64) -> Result<Locked, HoldError> {
    let row = sqlx::query(
        "SELECT id, account_id, amount_dong, captured_dong, status, expires_at \
         FROM holds WHERE id = $1 FOR UPDATE",
    )
    .bind(id)
    .fetch_optional(&mut *conn)
    .await?
    .ok_or(HoldError::NotFound(id))?;
    let hold = HoldRow::from_row(&row)?;
    Ok(match hold.status {
        HoldStatus::Active => Locked::Active(ActiveHold {
            id: hold.id,
            account: hold.account,
            amount: hold.amount,
            expires_at: hold.expires_at,
        }),
        _ => Locked::Closed(hold),
    })
}

impl ActiveHold {
    /// Id hold.
    pub fn id(&self) -> i64 {
        self.id
    }

    /// Capture `amount` (toàn phần hoặc một phần) theo đồng hồ `now`: ghi bút toán
    /// ví -amount / clearing +amount rồi đóng hold trong cùng transaction. Phần dư tự nhả
    /// vì hold không còn ACTIVE. Trả id bút toán.
    ///
    /// # Errors
    /// `Expired` nếu `now >= expires_at`; `CaptureExceedsHold`; `IdempotencyConflict`; lỗi SQL.
    pub async fn capture(
        self,
        conn: &mut PgConnection,
        now: Timestamp,
        amount: Money,
        key: &str,
    ) -> Result<i64, HoldError> {
        if now >= self.expires_at {
            return Err(HoldError::Expired {
                id: self.id,
                expires_at: self.expires_at,
            });
        }
        if amount.dong() <= 0 {
            return Err(HoldError::InvalidAmount(amount.dong()));
        }
        if amount > self.amount {
            return Err(HoldError::CaptureExceedsHold {
                hold: self.amount,
                requested: amount,
            });
        }
        let entry = capture_entry(&self.account, amount, key)?;
        let Some(entry_id) = insert_entry(conn, &entry).await? else {
            return Err(HoldError::IdempotencyConflict {
                key: key.to_string(),
            });
        };
        sqlx::query("UPDATE holds SET status = 'CAPTURED', captured_dong = $2 WHERE id = $1")
            .bind(self.id)
            .bind(amount.dong())
            .execute(&mut *conn)
            .await?;
        Ok(entry_id)
    }

    /// Hủy hold (chuyến bị hủy): không ghi bút toán, chỉ đóng hold.
    ///
    /// # Errors
    /// Lỗi SQL.
    pub async fn void(self, conn: &mut PgConnection) -> Result<(), HoldError> {
        sqlx::query("UPDATE holds SET status = 'VOIDED' WHERE id = $1")
            .bind(self.id)
            .execute(&mut *conn)
            .await?;
        Ok(())
    }
}

/// Bút toán capture: ví khách ra `amount`, clearing của VíXanh vào `amount`.
///
/// # Errors
/// Mã tài khoản sai hoặc bút toán không hợp lệ.
pub fn capture_entry(account: &str, amount: Money, key: &str) -> Result<JournalEntry, HoldError> {
    let out = amount
        .checked_neg()
        .ok_or(HoldError::InvalidAmount(amount.dong()))?;
    Ok(JournalEntry::try_new(
        key,
        vec![
            Posting::new(AccountId::new(account)?, out),
            Posting::new(AccountId::new(PSP_CLEARING)?, amount),
        ],
    )?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_roundtrip_and_unknown() {
        for s in [
            HoldStatus::Active,
            HoldStatus::Captured,
            HoldStatus::Voided,
            HoldStatus::Expired,
        ] {
            assert_eq!(s.as_str().parse::<HoldStatus>().unwrap(), s);
        }
        assert!(matches!(
            "active".parse::<HoldStatus>(),
            Err(HoldError::UnknownStatus(_))
        ));
    }

    #[test]
    fn timestamp_roundtrip_through_pg_type() {
        let ts: Timestamp = "2026-11-06T08:15:00Z".parse().unwrap();
        assert_eq!(from_pg(to_pg(ts).unwrap()).unwrap(), ts);
    }
}
