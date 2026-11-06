//! Mã tài khoản sổ cái, ví dụ `customer:CU-001`: parse có kiểm tra, không tự sửa dữ liệu.

use std::fmt;
use std::str::FromStr;

use crate::error::IdError;

/// Độ dài tối đa của mã tài khoản (đếm byte; mã chỉ chứa ASCII nên bằng số ký tự).
pub const MAX_ACCOUNT_ID_LEN: usize = 64;

/// Không `Copy` được vì chứa `String` (heap); `Clone` thì có.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AccountId(String);

impl AccountId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for AccountId {
    type Err = IdError;

    /// Dạng hợp lệ: `kind:tên` hoặc nhiều đoạn (`psp:vixanh:clearing`), mỗi đoạn không rỗng,
    /// chỉ gồm `A-Z a-z 0-9 -`. Khoảng trắng đầu/cuối bị từ chối, không bị cắt.
    fn from_str(s: &str) -> Result<AccountId, IdError> {
        if s.is_empty() {
            return Err(IdError::Empty);
        }
        if s.len() > MAX_ACCOUNT_ID_LEN {
            return Err(IdError::TooLong {
                max: MAX_ACCOUNT_ID_LEN,
            });
        }
        if let Some(ch) = s
            .chars()
            .find(|c| !(c.is_ascii_alphanumeric() || *c == '-' || *c == ':'))
        {
            return Err(IdError::InvalidChar { ch });
        }
        let segments = s.split(':').count();
        if segments < 2 || s.split(':').any(str::is_empty) {
            return Err(IdError::BadShape);
        }
        Ok(AccountId(s.to_string()))
    }
}

impl TryFrom<&str> for AccountId {
    type Error = IdError;

    fn try_from(s: &str) -> Result<AccountId, IdError> {
        s.parse()
    }
}

// Đi ra khỏi newtype luôn thành công: dùng `From`.
impl From<AccountId> for String {
    fn from(id: AccountId) -> String {
        id.0
    }
}

impl fmt::Display for AccountId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_ledger_accounts() {
        for raw in [
            "customer:CU-001",
            "driver:DR-001",
            "platform:revenue",
            "psp:vixanh:clearing",
        ] {
            assert_eq!(
                raw.parse::<AccountId>().map(|id| id.to_string()).ok(),
                Some(raw.to_string())
            );
        }
    }

    #[test]
    fn rejects_bad_input() {
        assert_eq!("".parse::<AccountId>(), Err(IdError::Empty));
        assert_eq!("customer".parse::<AccountId>(), Err(IdError::BadShape));
        assert_eq!("customer:".parse::<AccountId>(), Err(IdError::BadShape));
        assert_eq!(
            "customer::CU-001".parse::<AccountId>(),
            Err(IdError::BadShape)
        );
        assert_eq!(
            " driver:DR-001".parse::<AccountId>(),
            Err(IdError::InvalidChar { ch: ' ' })
        );
        assert_eq!(
            "driver:DR_001".parse::<AccountId>(),
            Err(IdError::InvalidChar { ch: '_' })
        );
        let long = format!("a:{}", "x".repeat(MAX_ACCOUNT_ID_LEN));
        assert_eq!(
            long.parse::<AccountId>(),
            Err(IdError::TooLong {
                max: MAX_ACCOUNT_ID_LEN
            })
        );
    }

    #[test]
    fn try_from_str_matches_parse() {
        assert_eq!(
            AccountId::try_from("driver:DR-001"),
            "driver:DR-001".parse::<AccountId>()
        );
        assert_eq!(AccountId::try_from("driver"), Err(IdError::BadShape));
    }

    #[test]
    fn converts_back_to_string() {
        let id: AccountId = "driver:DR-001".parse().unwrap();
        assert_eq!(String::from(id), "driver:DR-001");
    }

    #[test]
    fn ordering_is_lexicographic() {
        let a: AccountId = "customer:CU-001".parse().unwrap();
        let b: AccountId = "driver:DR-001".parse().unwrap();
        assert!(a < b);
    }
}
