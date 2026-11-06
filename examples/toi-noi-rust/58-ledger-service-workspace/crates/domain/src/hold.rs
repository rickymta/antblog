//! Hold: giữ tiền trong ví mà chưa ghi bút toán (bài 42). Ở lab này chỉ có đường chính:
//! tạo hold, rồi hold được capture khi chuyến hoàn thành. Void và job hết hạn là phần lược.

use jiff::{SignedDuration, Timestamp};
use ledger_core::{AccountId, Money};
use thiserror::Error;

/// Bốn trạng thái của hold; khớp `CHECK (status IN (...))` trong DB.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HoldStatus {
    /// Đang giữ tiền, trừ vào số dư khả dụng.
    Active,
    /// Đã capture; phần dư đã nhả.
    Captured,
    /// Đã hủy trước khi capture.
    Voided,
    /// Quá hạn.
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

    /// Đọc lại từ cột `status`; chuỗi lạ bị từ chối, không đoán.
    pub fn parse(raw: &str) -> Option<HoldStatus> {
        match raw {
            "ACTIVE" => Some(HoldStatus::Active),
            "CAPTURED" => Some(HoldStatus::Captured),
            "VOIDED" => Some(HoldStatus::Voided),
            "EXPIRED" => Some(HoldStatus::Expired),
            _ => None,
        }
    }
}

/// Lỗi nghiệp vụ của hold.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum HoldError {
    /// Số tiền xin giữ không dương.
    #[error("số tiền giữ phải dương, nhận {0} đồng")]
    NonPositive(i64),
    /// Khóa idempotency rỗng.
    #[error("idempotency_key của hold rỗng")]
    EmptyKey,
    /// Thời hạn không dương.
    #[error("hold phải có hạn dương")]
    NonPositiveTtl,
    /// Số dư khả dụng không đủ.
    #[error("không đủ số dư khả dụng: còn {available}, cần {requested}")]
    InsufficientAvailable {
        /// Khả dụng tại thời điểm kiểm.
        available: Money,
        /// Số xin giữ.
        requested: Money,
    },
    /// Cùng key nhưng nội dung khác bản đã ghi.
    #[error("key {key} đã dùng cho hold khác nội dung")]
    KeyConflict {
        /// Key bị trùng.
        key: String,
    },
}

/// Yêu cầu giữ tiền đã kiểm hình dạng.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewHold {
    /// Ví bị giữ tiền.
    pub account: AccountId,
    /// Số giữ, dương.
    pub amount: Money,
    /// Khóa idempotency, thường là `trip:<id>:hold`.
    pub key: String,
    /// Hạn của hold, tính từ đồng hồ truyền vào.
    pub expires_at: Timestamp,
}

impl NewHold {
    /// Dựng yêu cầu: `expires_at = now + ttl`. Không đọc đồng hồ ở đây.
    ///
    /// # Errors
    /// [`HoldError`] khi số tiền không dương, key rỗng hoặc ttl không dương.
    pub fn new(
        account: AccountId,
        amount_dong: i64,
        key: &str,
        now: Timestamp,
        ttl: SignedDuration,
    ) -> Result<NewHold, HoldError> {
        if amount_dong <= 0 {
            return Err(HoldError::NonPositive(amount_dong));
        }
        if key.trim().is_empty() {
            return Err(HoldError::EmptyKey);
        }
        if ttl <= SignedDuration::ZERO {
            return Err(HoldError::NonPositiveTtl);
        }
        let expires_at = now
            .checked_add(ttl)
            .map_err(|_| HoldError::NonPositiveTtl)?;
        Ok(NewHold {
            account,
            amount: Money::from_dong(amount_dong),
            key: key.to_string(),
            expires_at,
        })
    }
}

/// `available = balance - held`. Tính theo i128 nên không tràn.
pub fn available(balance: Money, held: Money) -> i128 {
    i128::from(balance.dong()) - i128::from(held.dong())
}

/// Kiểm đủ khả dụng cho `requested`.
///
/// # Errors
/// [`HoldError::InsufficientAvailable`].
pub fn ensure_available(balance: Money, held: Money, requested: Money) -> Result<(), HoldError> {
    let left = available(balance, held);
    if left < i128::from(requested.dong()) {
        // `left` nhỏ hơn một số i64 dương nên luôn vừa i64 khi cắt về biên dưới.
        let shown = i64::try_from(left).unwrap_or(i64::MIN);
        return Err(HoldError::InsufficientAvailable {
            available: Money::from_dong(shown),
            requested,
        });
    }
    Ok(())
}

/// Hold đang ACTIVE đọc từ DB.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveHold {
    /// Số đã giữ.
    pub amount: Money,
    /// Hạn của hold.
    pub expires_at: Timestamp,
}

/// Kết quả đóng một hold khi chuyến hoàn thành.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Settlement {
    /// Hold capture `captured`; phần dư tự nhả vì hold không còn ACTIVE.
    Captured {
        /// Số được capture.
        captured: Money,
    },
    /// Hold đã quá hạn lúc chuyến hoàn thành: đóng EXPIRED, không capture gì.
    Expired,
}

impl ActiveHold {
    /// Đóng hold cho một chuyến cước `fare` theo đồng hồ `now`.
    /// Cước lớn hơn số giữ thì chỉ capture phần đã giữ (phần còn lại ghi nợ ví, ví có thể âm).
    pub fn settle(&self, now: Timestamp, fare: Money) -> Settlement {
        if now >= self.expires_at {
            Settlement::Expired
        } else {
            Settlement::Captured {
                captured: self.amount.min(fare),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(raw: &str) -> Timestamp {
        raw.parse().unwrap()
    }

    #[test]
    fn available_is_balance_minus_held() {
        let (balance, held) = (Money::from_dong(100_000), Money::from_dong(30_000));
        assert_eq!(available(balance, held), 70_000);
        assert!(ensure_available(balance, held, Money::from_dong(70_000)).is_ok());
        assert_eq!(
            ensure_available(balance, held, Money::from_dong(70_001)).unwrap_err(),
            HoldError::InsufficientAvailable {
                available: Money::from_dong(70_000),
                requested: Money::from_dong(70_001)
            }
        );
    }

    #[test]
    fn new_hold_expiry_comes_from_the_given_clock() {
        let now = t("2026-11-06T08:00:00Z");
        let hold = NewHold::new(
            AccountId::new("customer:CU-001").unwrap(),
            30_000,
            "trip:TN-0001:hold",
            now,
            SignedDuration::from_mins(15),
        )
        .unwrap();
        assert_eq!(hold.expires_at, t("2026-11-06T08:15:00Z"));
    }

    #[test]
    fn settle_depends_only_on_the_clock_argument() {
        let hold = ActiveHold {
            amount: Money::from_dong(30_000),
            expires_at: t("2026-11-06T08:15:00Z"),
        };
        let fare = Money::from_dong(25_500);
        assert_eq!(
            hold.settle(t("2026-11-06T08:14:59Z"), fare),
            Settlement::Captured { captured: fare }
        );
        assert_eq!(
            hold.settle(t("2026-11-06T08:15:00Z"), fare),
            Settlement::Expired
        );
        // Cước lớn hơn số giữ: chỉ capture phần đã giữ.
        assert_eq!(
            hold.settle(t("2026-11-06T08:00:00Z"), Money::from_dong(40_000)),
            Settlement::Captured {
                captured: Money::from_dong(30_000)
            }
        );
    }

    #[test]
    fn status_roundtrip_and_unknown() {
        for s in [
            HoldStatus::Active,
            HoldStatus::Captured,
            HoldStatus::Voided,
            HoldStatus::Expired,
        ] {
            assert_eq!(HoldStatus::parse(s.as_str()), Some(s));
        }
        assert_eq!(HoldStatus::parse("active"), None);
    }
}
