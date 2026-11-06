//! Kiểu kết quả của use case. Thuần dữ liệu, không biết dây truyền (JSON hay protobuf).

use ledger_domain::Money;

/// Một dòng của bút toán đã ghi.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostedLine {
    /// Mã tài khoản.
    pub account: String,
    /// Số tiền có dấu, đồng.
    pub amount_dong: i64,
}

/// Bút toán đã ghi, đủ để dựng sự kiện `LedgerEntryPosted`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostedEntry {
    /// Mã dạng `JE-000001`.
    pub entry_id: String,
    /// Khóa idempotency.
    pub idempotency_key: String,
    /// Các dòng.
    pub postings: Vec<PostedLine>,
    /// Thời điểm ghi, ISO 8601 UTC.
    pub posted_at: String,
}

/// Kết quả xử lý một `TripCompleted`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Applied {
    /// Hai bút toán mới (capture, split), đã xếp vào outbox.
    Posted(Vec<PostedEntry>),
    /// Inbox đã có sự kiện này: không ghi gì.
    Duplicate,
}

/// Kết quả ghi một bút toán.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PostOutcome {
    /// Bút toán mới.
    Created(PostedEntry),
    /// Key đã có, nội dung trùng: mã bút toán cũ.
    Replayed {
        /// Mã dạng `JE-000001`.
        entry_id: String,
    },
}

impl PostOutcome {
    /// Mã bút toán, mới hay cũ.
    pub fn entry_id(&self) -> &str {
        match self {
            PostOutcome::Created(posted) => &posted.entry_id,
            PostOutcome::Replayed { entry_id } => entry_id,
        }
    }

    /// `true` nếu đây là bản cũ.
    pub fn replayed(&self) -> bool {
        matches!(self, PostOutcome::Replayed { .. })
    }
}

/// Số dư và phần đang bị giữ của một tài khoản.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Funds {
    /// Số dư sổ cái.
    pub balance: Money,
    /// Tổng hold ACTIVE chưa hết hạn.
    pub held: Money,
}

impl Funds {
    /// `balance - held`.
    pub fn available(&self) -> i128 {
        ledger_domain::hold::available(self.balance, self.held)
    }
}

/// Kết quả tạo hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HoldOutcome {
    /// Hold mới.
    Created {
        /// Id do DB cấp.
        hold_id: i64,
    },
    /// Key đã có, nội dung trùng.
    Replayed {
        /// Id của hold cũ.
        hold_id: i64,
    },
}

impl HoldOutcome {
    /// Id hold, mới hay cũ.
    pub fn hold_id(self) -> i64 {
        match self {
            HoldOutcome::Created { hold_id } | HoldOutcome::Replayed { hold_id } => hold_id,
        }
    }

    /// `true` nếu đây là hold cũ.
    pub fn replayed(self) -> bool {
        matches!(self, HoldOutcome::Replayed { .. })
    }
}
