//! Mã tài khoản sổ cái, ví dụ `customer:CU-001`.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::error::LedgerError;

/// Mã tài khoản đã kiểm tra: không rỗng, đã cắt khoảng trắng hai đầu.
///
/// Khi deserialize, JSON cũng đi qua [`AccountId::new`], nên không thể
/// nạp một mã rỗng từ bên ngoài.
///
/// ```
/// use ledger_core::AccountId;
///
/// assert!(AccountId::new("driver:DR-001").is_ok());
/// assert!(AccountId::new("   ").is_err());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct AccountId(String);

impl AccountId {
    /// Tạo mã tài khoản.
    ///
    /// # Errors
    /// [`LedgerError::EmptyAccountId`] nếu chuỗi rỗng hoặc toàn khoảng trắng.
    pub fn new(raw: &str) -> Result<AccountId, LedgerError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(LedgerError::EmptyAccountId);
        }
        Ok(AccountId(trimmed.to_string()))
    }

    /// Mã dưới dạng `&str`.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for AccountId {
    type Error = LedgerError;

    fn try_from(raw: String) -> Result<AccountId, LedgerError> {
        AccountId::new(&raw)
    }
}

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
    fn rejects_blank_id() {
        assert_eq!(AccountId::new("  "), Err(LedgerError::EmptyAccountId));
    }

    #[test]
    fn trims_whitespace() -> Result<(), LedgerError> {
        let id = AccountId::new("  driver:DR-001 ")?;
        assert_eq!(id.as_str(), "driver:DR-001");
        Ok(())
    }

    #[test]
    fn json_blank_id_is_rejected() {
        let result: Result<AccountId, _> = serde_json::from_str("\"  \"");
        assert!(result.is_err());
    }
}
