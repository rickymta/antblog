//! Mã tài khoản sổ cái, ví dụ `customer:CU-001`.

use std::fmt;

use crate::error::LedgerError;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AccountId(String);

impl AccountId {
    /// Tạo mã tài khoản; chuỗi rỗng (hoặc toàn khoảng trắng) bị từ chối.
    pub fn new(raw: &str) -> Result<AccountId, LedgerError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(LedgerError::EmptyAccountId);
        }
        Ok(AccountId(trimmed.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AccountId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // `pad` để `{:<20}` căn lề được.
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
}
