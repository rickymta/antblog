//! Cổng (port) của sổ cái: trait `LedgerRepository`, Unit of Work, service và adapter bộ nhớ.
//!
//! Crate này chỉ phụ thuộc `ledger-core`, `thiserror` và `tokio` (hàng đợi khóa, `spawn`).
//! Không có `sqlx`: adapter PostgreSQL nằm ở crate `ledger-pg`.

pub mod contract;
pub mod error;
pub mod memory;
pub mod repository;
pub mod service;

pub use error::RepoError;
pub use memory::MemoryRepository;
pub use repository::{LedgerRepository, LedgerUnitOfWork, PostOutcome, validate_entry};
pub use service::post_trip_capture;

use ledger_core::EntryId;

/// Sơ đồ tài khoản rút gọn của chuyến mẫu: (mã, loại). Cả hai adapter mở đúng danh sách này.
pub const CHART: [(&str, &str); 4] = [
    ("customer:CU-001", "liability"),
    ("driver:DR-001", "liability"),
    ("platform:revenue", "revenue"),
    ("psp:vixanh:clearing", "asset"),
];

/// Mã bút toán dạng số (cột `bigint` của PostgreSQL) sang `EntryId` (`JE-000001`).
pub fn entry_id_from_i64(id: i64) -> Option<EntryId> {
    EntryId::parse(&format!("JE-{id:06}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Bằng chứng tĩnh: manifest của cổng và của ledger-core không khai báo sqlx
    // (bỏ dòng `description`, vì mô tả của cổng có nhắc tên sqlx để nói nó không dùng).
    #[test]
    fn port_and_core_manifests_have_no_sqlx() {
        let port = include_str!("../Cargo.toml");
        let core = include_str!("../../../20-ledger-core-workspace/ledger-core/Cargo.toml");
        for (name, manifest) in [("ledger-port", port), ("ledger-core", core)] {
            let declares_sqlx = manifest
                .lines()
                .filter(|l| !l.trim_start().starts_with("description"))
                .any(|l| l.contains("sqlx"));
            assert!(!declares_sqlx, "{name} không được phụ thuộc sqlx");
        }
    }

    #[test]
    fn entry_id_round_trip() {
        assert_eq!(entry_id_from_i64(1).unwrap().to_string(), "JE-000001");
        assert!(entry_id_from_i64(0).is_none());
    }
}
