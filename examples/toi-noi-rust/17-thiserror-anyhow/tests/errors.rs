//! Integration test: hợp đồng lỗi của thư viện, nhìn từ bên ngoài crate.

use std::error::Error;
use std::io;

use ledger_lab::{Config, LedgerError, MemoryStore, Money, StorageError};

/// `anyhow::Error` chỉ nhận lỗi `Send + Sync + 'static`; kiểm lúc biên dịch.
fn assert_anyhow_compatible<E: Error + Send + Sync + 'static>() {}

#[test]
fn error_types_can_cross_into_anyhow() {
    assert_anyhow_compatible::<LedgerError>();
    assert_anyhow_compatible::<StorageError>();
}

#[test]
fn question_mark_converts_io_error_through_from() {
    fn open() -> Result<(), StorageError> {
        // `?` gọi `From<io::Error> for StorageError` do `#[from]` sinh.
        Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "không có quyền",
        ))?
    }
    let err = open().unwrap_err();
    assert!(matches!(err, StorageError::Io(_)));
    assert_eq!(
        err.source().map(ToString::to_string).as_deref(),
        Some("không có quyền")
    );
}

#[test]
fn storage_error_chain_has_two_links() {
    let err = Config::parse("max_postings = 1x").unwrap_err();
    let mut links = vec![err.to_string()];
    let mut cur = err.source();
    while let Some(c) = cur {
        links.push(c.to_string());
        cur = c.source();
    }
    assert_eq!(
        links,
        [
            "khoá max_postings trong cấu hình không phải số nguyên",
            "invalid digit found in string"
        ]
    );
}

#[test]
fn domain_and_storage_errors_are_distinct_types() {
    let domain = LedgerError::Unbalanced {
        sum: Money::from_dong(-500),
    };
    assert_eq!(domain.http_status(), 422);
    let mut store = MemoryStore::new();
    store.set_fail_writes(true);
    assert!(store.append("JE-000001").is_err());
    // Bật lại thì ghi được: lỗi hạ tầng có thể thử lại, lỗi domain thì không.
    store.set_fail_writes(false);
    assert!(store.append("JE-000001").is_ok());
}
