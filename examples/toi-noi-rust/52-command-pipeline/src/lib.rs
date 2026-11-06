//! Lab bài 52: command handler generic, pipeline validate → trace → transaction → handler,
//! domain event thu về sau handler và ghi outbox cùng transaction.
//!
//! Thiếu handler là lỗi biên dịch, không phải lỗi lúc chạy. Doc test dưới đây cố tình
//! gửi `VoidHold` (lab không viết handler cho nó) qua pipeline thật và phải KHÔNG biên dịch:
//!
//! ```compile_fail,E0277
//! use ledger_52_command_pipeline::{Ctx, CommandHandler, LedgerPipeline, VoidHold};
//!
//! async fn gui(pipeline: &LedgerPipeline, ctx: &mut Ctx) {
//!     let _ = pipeline.handle(VoidHold { hold_id: 7 }, ctx).await;
//! }
//! ```
//!
//! Kiểu `Output` cũng được kiểm: handler trả sai kiểu kết quả là lỗi E0271.
//!
//! Trên Rust stable, `compile_fail` chỉ kiểm đoạn code không biên dịch; mã lỗi trong thẻ không
//! được đối chiếu (rustdoc chỉ làm việc đó trên nightly). Hai mã E ở đây lấy từ lần chạy rustc riêng.
//!
//! ```compile_fail,E0271
//! use ledger_52_command_pipeline::{AppError, CommandHandler, Ctx, Ping};
//!
//! struct Sai;
//!
//! impl CommandHandler<Ping> for Sai {
//!     async fn handle(&self, _cmd: Ping, _ctx: &mut Ctx) -> Result<String, AppError> {
//!         Ok("sai kiểu".to_string())
//!     }
//! }
//! ```
//!
//! Cùng lời gọi với `CapturePayment` (có handler) thì biên dịch được:
//!
//! ```
//! use ledger_52_command_pipeline::{CapturePayment, Ctx, CommandHandler, LedgerPipeline};
//!
//! async fn gui(pipeline: &LedgerPipeline, ctx: &mut Ctx) {
//!     let cmd = CapturePayment {
//!         trip_id: "TN-0001".into(),
//!         customer: "CU-001".into(),
//!         driver: "DR-001".into(),
//!         fare_dong: 25_500,
//!     };
//!     let _ = pipeline.handle(cmd, ctx).await;
//! }
//! ```

pub mod command;
pub mod direct;
pub mod domain;
pub mod dynbus;
pub mod events;
pub mod handlers;
pub mod lab;
pub mod layers;
pub mod scenarios;

pub use command::{AppError, Command, CommandHandler, Ctx, Validate};
pub use dynbus::DynBus;
pub use handlers::{
    CapturePayment, CaptureReceipt, Echo, LedgerHandlers, Ping, PostRawEntry, VoidHold,
};
pub use layers::{NaivePublish, Sink, Traced, Transactional, Validated};

use sqlx::PgPool;

/// Pipeline chuẩn: validate (ngoài cùng) → trace → transaction → handler.
pub type LedgerPipeline = Validated<Traced<Transactional<LedgerHandlers>>>;

/// Dựng pipeline chuẩn trên `pool`.
pub fn ledger_pipeline(pool: PgPool) -> LedgerPipeline {
    Validated(Traced(Transactional::new(LedgerHandlers, pool)))
}
