//! Port: hợp đồng mà adapter phải cài. Use case chỉ biết trait này.
//!
//! Dùng `impl Future + Send` thay cho `async fn` để use case chạy được trong `tokio::spawn`
//! mà không cần crate nào ngoài thư viện chuẩn.

use std::future::Future;

use jiff::Timestamp;
use ledger_domain::hold::NewHold;
use ledger_domain::trip::{CapturePlan, TripFare};
use ledger_domain::{AccountId, JournalEntry, Money};

use crate::error::Error;
use crate::model::{Applied, Funds, HoldOutcome, PostOutcome};

/// Cổng lưu trữ của sổ cái. Mỗi phương thức là một đơn vị công việc nguyên khối: hoặc xong
/// hết (bút toán, số dư, outbox cùng commit) hoặc không có gì.
pub trait LedgerStore: Send + Sync + 'static {
    /// Xử lý một `TripCompleted`: inbox, hai bút toán, `payments`, đóng hold của chuyến (nếu có),
    /// số dư và outbox trong MỘT transaction. `now` là đồng hồ của use case.
    fn apply_trip(
        &self,
        event_id: &str,
        trip: &TripFare,
        plan: &CapturePlan,
        now: Timestamp,
    ) -> impl Future<Output = Result<Applied, Error>> + Send;

    /// Ghi một bút toán thủ công, idempotent theo key.
    fn post_entry(
        &self,
        entry: &JournalEntry,
    ) -> impl Future<Output = Result<PostOutcome, Error>> + Send;

    /// Số dư hiện tại; `None` nếu tài khoản chưa từng có bút toán.
    fn balance(
        &self,
        account: &AccountId,
    ) -> impl Future<Output = Result<Option<Money>, Error>> + Send;

    /// Số dư kèm hold còn hiệu lực lúc `now`; `None` nếu chưa có tài khoản.
    fn funds(
        &self,
        account: &AccountId,
        now: Timestamp,
    ) -> impl Future<Output = Result<Option<Funds>, Error>> + Send;

    /// Tạo hold nếu đủ khả dụng, idempotent theo key.
    fn place_hold(
        &self,
        hold: &NewHold,
        now: Timestamp,
    ) -> impl Future<Output = Result<HoldOutcome, Error>> + Send;

    /// Hoàn lần `n` cho chuyến `trip_id`. Khóa dòng thanh toán, kiểm phần còn hoàn được, ghi bút toán.
    fn refund(
        &self,
        trip_id: &str,
        n: u32,
        amount: Money,
    ) -> impl Future<Output = Result<PostOutcome, Error>> + Send;

    /// Kho còn trả lời không (cho `/readyz`).
    fn ping(&self) -> impl Future<Output = Result<(), Error>> + Send;
}
