//! Hợp đồng của command: kiểu kết quả gắn vào command, handler generic, ngữ cảnh một lần gọi.

use std::fmt;

use sqlx::{PgConnection, Postgres, Transaction};

use crate::events::DomainEvent;

/// Lỗi của tầng application. Pipeline và handler dùng chung một kiểu lỗi.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// Command bị từ chối ở lớp validate, chưa chạm database.
    #[error("validation: {0}")]
    Validation(String),
    /// Lỗi nghiệp vụ từ `ledger-core`.
    #[error("domain: {0}")]
    Domain(#[from] ledger_core::LedgerError),
    /// Lỗi hạ tầng từ sqlx (kể cả lỗi lúc COMMIT).
    #[error("storage: {0}")]
    Storage(#[from] sqlx::Error),
    /// Chỉ `DynBus` trả lỗi này: command chưa được đăng ký handler. Phát hiện lúc chạy.
    #[error("không có handler cho {0}")]
    NoHandler(&'static str),
    /// Handler cần transaction nhưng không có lớp `Transactional` bọc ngoài.
    #[error("handler cần transaction nhưng pipeline không có lớp Transactional")]
    NoTransaction,
}

impl AppError {
    /// SQLSTATE của lỗi PostgreSQL, nếu là lỗi database.
    pub fn sqlstate(&self) -> Option<String> {
        match self {
            AppError::Storage(sqlx::Error::Database(e)) => e.code().map(|c| c.into_owned()),
            _ => None,
        }
    }
}

/// Một ý định thay đổi trạng thái. Kiểu kết quả đi cùng command qua `Output`.
pub trait Command: Send + 'static {
    /// Kiểu handler phải trả về cho command này.
    type Output: Send + 'static;
    /// Tên dùng trong log và lỗi.
    const NAME: &'static str;
}

/// Kiểm tra đầu vào trước khi chạm trạng thái.
pub trait Validate {
    /// `Err(lý do)` nếu command không hợp lệ.
    ///
    /// # Errors
    /// Lý do từ chối, dạng văn bản.
    fn validate(&self) -> Result<(), String>;
}

/// Xử lý một loại command `C`. Handler thật và mọi lớp bọc cùng hiện thực trait này,
/// nên lớp bọc xếp chồng được: `Validated<Traced<Transactional<H>>>` cũng là một handler.
///
/// `-> impl Future + Send` (thay vì `async fn` trần) để future của kiểu generic dùng được
/// với `tokio::spawn`, như đã bàn ở bài 26.
pub trait CommandHandler<C: Command>: Send + Sync {
    /// Chạy command trong ngữ cảnh `ctx`.
    fn handle(
        &self,
        cmd: C,
        ctx: &mut Ctx,
    ) -> impl Future<Output = Result<C::Output, AppError>> + Send;
}

/// Ngữ cảnh của MỘT lần gọi: clock, transaction đang mở, sự kiện chờ thu về, log thứ tự.
pub struct Ctx {
    now: String,
    tracing: bool,
    tx: Option<Transaction<'static, Postgres>>,
    events: Vec<DomainEvent>,
    log: Vec<String>,
}

impl Ctx {
    /// Ngữ cảnh mới. `now` là thời điểm ISO 8601 UTC do người gọi đưa vào.
    pub fn new(now: &str) -> Ctx {
        Ctx {
            now: now.to_string(),
            tracing: true,
            tx: None,
            events: Vec::new(),
            log: Vec::new(),
        }
    }

    /// Ngữ cảnh tắt log: `log` không định dạng, không cấp phát (dùng khi đo).
    pub fn quiet(now: &str) -> Ctx {
        Ctx {
            tracing: false,
            ..Ctx::new(now)
        }
    }

    /// Thời điểm của lần gọi.
    pub fn now(&self) -> &str {
        &self.now
    }

    /// Ghi một dòng log thứ tự. `format_args!` chỉ định dạng khi log đang bật.
    pub fn log(&mut self, args: fmt::Arguments<'_>) {
        if self.tracing {
            self.log.push(args.to_string());
        }
    }

    /// Các dòng log theo thứ tự xảy ra.
    pub fn trace(&self) -> &[String] {
        &self.log
    }

    /// Handler ghi nhận sự kiện miền; pipeline thu về sau khi handler xong.
    pub fn raise(&mut self, events: impl IntoIterator<Item = DomainEvent>) {
        self.events.extend(events);
    }

    /// Lấy và xóa các sự kiện đã ghi nhận.
    pub fn take_events(&mut self) -> Vec<DomainEvent> {
        std::mem::take(&mut self.events)
    }

    /// Kết nối của transaction đang mở.
    ///
    /// # Errors
    /// [`AppError::NoTransaction`] nếu không có lớp `Transactional` bọc ngoài.
    pub fn tx(&mut self) -> Result<&mut PgConnection, AppError> {
        match self.tx.as_mut() {
            Some(tx) => Ok(&mut **tx),
            None => Err(AppError::NoTransaction),
        }
    }

    pub(crate) fn install_tx(&mut self, tx: Transaction<'static, Postgres>) {
        self.tx = Some(tx);
    }

    pub(crate) fn remove_tx(&mut self) -> Option<Transaction<'static, Postgres>> {
        self.tx.take()
    }
}
