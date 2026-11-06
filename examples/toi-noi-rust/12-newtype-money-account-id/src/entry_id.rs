//! Mã bút toán `JE-000001`: lưu số thứ tự `u64`, hiển thị và parse theo một dạng chuẩn duy nhất.

use std::fmt;
use std::str::FromStr;

use crate::error::IdError;

/// Có `Copy`: chỉ là một `u64`. `Ord` so sánh theo số, nên `JE-1000000` đứng sau `JE-999999`
/// (so sánh chuỗi sẽ cho kết quả ngược).
///
/// Không dùng được `EntryId` ở chỗ cần `AccountId`:
///
/// ```compile_fail,E0308
/// use ledger_lab::{AccountId, EntryId};
///
/// let _: AccountId = EntryId::try_from(1).unwrap();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EntryId(u64);

impl EntryId {
    /// Từ vị trí trong `Vec` (bắt đầu từ 0) sang số thứ tự (bắt đầu từ 1); luôn hợp lệ.
    pub(crate) fn from_index(index: usize) -> EntryId {
        EntryId(index as u64 + 1)
    }

    pub const fn sequence(self) -> u64 {
        self.0
    }
}

impl TryFrom<u64> for EntryId {
    type Error = IdError;

    /// Số thứ tự bắt đầu từ 1; `0` bị từ chối.
    fn try_from(sequence: u64) -> Result<EntryId, IdError> {
        if sequence == 0 {
            return Err(IdError::BadEntryId);
        }
        Ok(EntryId(sequence))
    }
}

impl From<EntryId> for u64 {
    fn from(id: EntryId) -> u64 {
        id.0
    }
}

impl fmt::Display for EntryId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(&format!("JE-{:06}", self.0))
    }
}

impl FromStr for EntryId {
    type Err = IdError;

    fn from_str(s: &str) -> Result<EntryId, IdError> {
        let digits = s.strip_prefix("JE-").ok_or(IdError::BadEntryId)?;
        // `u64::from_str` nhận cả "+5"; ta chỉ chấp nhận chữ số.
        if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
            return Err(IdError::BadEntryId);
        }
        let sequence: u64 = digits.parse().map_err(|_| IdError::BadEntryId)?;
        let id = EntryId::try_from(sequence)?;
        // Chỉ nhận dạng chuẩn: "JE-1" hay "JE-0000001" parse được số nhưng không phải mã đã in ra.
        if id.to_string() != s {
            return Err(IdError::BadEntryId);
        }
        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn displays_six_digits() {
        assert_eq!(EntryId::try_from(1).unwrap().to_string(), "JE-000001");
        assert_eq!(EntryId::try_from(42).unwrap().to_string(), "JE-000042");
        assert_eq!(
            EntryId::try_from(1_000_000).unwrap().to_string(),
            "JE-1000000"
        );
    }

    #[test]
    fn parse_round_trips() {
        for n in [1_u64, 2, 999_999, 1_000_000, u64::MAX] {
            let id = EntryId::try_from(n).unwrap();
            assert_eq!(id.to_string().parse::<EntryId>(), Ok(id));
        }
    }

    #[test]
    fn rejects_non_canonical_or_bad_input() {
        for raw in [
            "",
            "JE-",
            "JE-000000",
            "JE-1",
            "JE-0000001",
            "JE-+00001",
            "je-000001",
            "JE-00000x",
            " JE-000001",
            "TN-000001",
            "JE-99999999999999999999999",
        ] {
            assert_eq!(raw.parse::<EntryId>(), Err(IdError::BadEntryId), "{raw:?}");
        }
        assert_eq!(EntryId::try_from(0), Err(IdError::BadEntryId));
    }

    #[test]
    fn orders_by_number_not_by_text() {
        let a = EntryId::try_from(999_999).unwrap();
        let b = EntryId::try_from(1_000_000).unwrap();
        assert!(a < b);
        assert!(a.to_string() > b.to_string()); // so sánh chuỗi cho kết quả ngược
    }
}
