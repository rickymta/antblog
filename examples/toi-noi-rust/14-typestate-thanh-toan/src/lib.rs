//! Thư viện sổ cái rút gọn của Tới Nơi (bài 14, phần P02).
//!
//! Từ lab bài 10 (`Money`, `AccountId`, `JournalEntry`, `Ledger`), bài này thêm máy
//! trạng thái thanh toán theo hai cách: `enum` kiểm tra lúc chạy ([`PaymentRecord`])
//! và typestate kiểm tra lúc biên dịch ([`Payment`]).

pub mod account;
pub mod entry;
pub mod error;
pub mod ledger;
pub mod money;
pub mod record;
pub mod settle;
pub mod state;
pub mod typestate;

pub use account::AccountId;
pub use entry::{JournalEntry, Posting};
pub use error::LedgerError;
pub use ledger::{Ledger, Posted};
pub use money::Money;
pub use record::PaymentRecord;
pub use settle::PaymentTerms;
pub use state::{PaymentError, PaymentEvent, PaymentState, transition};
pub use typestate::{
    AnyPayment, Authorized, Captured, Failed, Initiated, Payment, RefundPending, Refunded, State,
};
