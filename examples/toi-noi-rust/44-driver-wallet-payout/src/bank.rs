//! Ngân hàng nhận lệnh chi: một trait và một bản giả có thể cấu hình hành vi.

use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::sync::Mutex;

use ledger_core::Money;
use thiserror::Error;

/// Lệnh chi một khoản. `reference` do ta đặt (chính là `payout_key`) và ngân hàng dùng nó
/// để nhận ra lệnh gửi lại.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferRequest {
    /// Mã tham chiếu duy nhất của lệnh, ví dụ `payout:PB-2026-11-06.1:DR-001`.
    pub reference: String,
    /// Tài xế nhận tiền, ví dụ `DR-001`.
    pub beneficiary: String,
    /// Số tiền chi.
    pub amount: Money,
}

/// Ngân hàng đã nhận lệnh.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transfer {
    /// Mã giao dịch phía ngân hàng.
    pub bank_ref: String,
}

/// Kết quả không thành công của một lệnh chi.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum BankError {
    /// Ngân hàng từ chối dứt khoát: tiền chưa đi đâu cả.
    #[error("ngân hàng từ chối: {0}")]
    Rejected(String),
    /// Hết thời gian chờ: không biết ngân hàng đã nhận lệnh hay chưa.
    #[error("hết thời gian chờ ngân hàng")]
    Timeout,
}

/// Trạng thái một dòng trong sao kê.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineStatus {
    /// Tiền đã tới tài khoản nhận.
    Settled,
    /// Ngân hàng nhận lệnh rồi trả lại tiền (sai tài khoản, tài khoản đóng).
    Returned,
}

/// Một dòng sao kê.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatementLine {
    /// Mã tham chiếu của lệnh.
    pub reference: String,
    /// Mã giao dịch phía ngân hàng.
    pub bank_ref: String,
    /// Số tiền ngân hàng ghi nhận.
    pub amount: Money,
    /// Kết cục.
    pub status: LineStatus,
}

/// Cổng sang ngân hàng. Trả `impl Future + Send` để dùng được trong `tokio::spawn`
/// mà không cần crate `async-trait`.
pub trait Bank {
    /// Gửi lệnh chi. Cùng `reference` gửi lại phải cho cùng kết quả, không chi thêm lần nữa.
    fn transfer(
        &self,
        request: &TransferRequest,
    ) -> impl Future<Output = Result<Transfer, BankError>> + Send;

    /// Tra cứu lệnh theo mã tham chiếu, không chi gì. `Ok(None)`: ngân hàng chưa có lệnh này.
    fn lookup(
        &self,
        reference: &str,
    ) -> impl Future<Output = Result<Option<Transfer>, BankError>> + Send;

    /// Sao kê: mọi lệnh ngân hàng đã nhận.
    fn statement(&self) -> impl Future<Output = Result<Vec<StatementLine>, BankError>> + Send;
}

/// Hành vi của ngân hàng giả với một tài xế.
#[derive(Debug, Clone)]
pub enum Behavior {
    /// Nhận lệnh, tiền tới nơi.
    Accept,
    /// Từ chối ngay, không ghi gì.
    Reject(String),
    /// Nhận lệnh nhưng lần gọi đầu bị timeout phía ta; gửi lại cùng mã thì trả kết quả cũ,
    /// lần sau không còn timeout.
    TimeoutOnce,
    /// Lệnh mất trên đường: lần gọi đầu timeout và ngân hàng không nhận gì.
    LostOnce,
    /// Nhận lệnh, sau đó trả tiền lại: sao kê ghi `Returned`.
    ReturnedLater,
}

#[derive(Debug, Clone)]
struct Received {
    reference: String,
    beneficiary: String,
    amount: Money,
    bank_ref: String,
    status: LineStatus,
}

#[derive(Default)]
struct State {
    behaviors: HashMap<String, Behavior>,
    received: Vec<Received>,
    timed_out: HashSet<String>,
    lost: HashSet<String>,
    transfer_calls: HashMap<String, usize>,
    lookup_down: bool,
    hidden: HashSet<String>,
    amount_override: HashMap<String, Money>,
    extra_lines: Vec<StatementLine>,
}

/// Ngân hàng giả trong bộ nhớ. Mặc định mọi tài xế `Accept`.
#[derive(Default)]
pub struct FakeBank {
    state: Mutex<State>,
}

impl FakeBank {
    /// Ngân hàng mới, mọi tài xế `Accept`.
    pub fn new() -> FakeBank {
        FakeBank::default()
    }

    /// Đặt hành vi cho một tài xế.
    pub fn set_behavior(&self, driver: &str, behavior: Behavior) {
        self.state
            .lock()
            .expect("mutex")
            .behaviors
            .insert(driver.to_string(), behavior);
    }

    /// Số lệnh ngân hàng đã nhận cho một tài xế (mỗi `reference` khác nhau tính một lệnh).
    pub fn received_count(&self, driver: &str) -> usize {
        let state = self.state.lock().expect("mutex");
        state
            .received
            .iter()
            .filter(|r| r.beneficiary == driver)
            .count()
    }

    /// Số lần `transfer` đã được gọi cho một tài xế, kể cả lần trùng mã và lần timeout.
    pub fn transfer_calls(&self, driver: &str) -> usize {
        let state = self.state.lock().expect("mutex");
        state.transfer_calls.get(driver).copied().unwrap_or(0)
    }

    /// Bật hoặc tắt lỗi tra cứu: khi bật, `lookup` trả `Timeout`.
    pub fn set_lookup_down(&self, down: bool) {
        self.state.lock().expect("mutex").lookup_down = down;
    }

    /// Làm sao kê ghi sai số tiền của một lệnh.
    pub fn tamper_amount(&self, reference: &str, amount: Money) {
        self.state
            .lock()
            .expect("mutex")
            .amount_override
            .insert(reference.to_string(), amount);
    }

    /// Làm sao kê thiếu dòng của một lệnh.
    pub fn hide_line(&self, reference: &str) {
        self.state
            .lock()
            .expect("mutex")
            .hidden
            .insert(reference.to_string());
    }

    /// Thêm vào sao kê một dòng mà ta không hề gửi lệnh.
    pub fn inject_line(&self, line: StatementLine) {
        self.state.lock().expect("mutex").extra_lines.push(line);
    }

    fn handle(&self, request: &TransferRequest) -> Result<Transfer, BankError> {
        let mut state = self.state.lock().expect("mutex");
        *state
            .transfer_calls
            .entry(request.beneficiary.clone())
            .or_insert(0) += 1;
        // Idempotent theo reference: đã nhận thì trả đúng kết quả cũ.
        if let Some(found) = state
            .received
            .iter()
            .find(|r| r.reference == request.reference)
        {
            return if found.amount == request.amount {
                Ok(Transfer {
                    bank_ref: found.bank_ref.clone(),
                })
            } else {
                Err(BankError::Rejected(
                    "mã tham chiếu đã dùng cho số tiền khác".into(),
                ))
            };
        }
        let mut behavior = state
            .behaviors
            .get(&request.beneficiary)
            .cloned()
            .unwrap_or(Behavior::Accept);
        // «Once»: tài xế này đã timeout một lần thì các lệnh sau (kể cả mã mới) chạy bình thường.
        if matches!(behavior, Behavior::TimeoutOnce)
            && !state.timed_out.insert(request.beneficiary.clone())
        {
            behavior = Behavior::Accept;
        }
        if matches!(behavior, Behavior::LostOnce) {
            if state.lost.insert(request.beneficiary.clone()) {
                return Err(BankError::Timeout);
            }
            behavior = Behavior::Accept;
        }
        let status = match &behavior {
            Behavior::Reject(reason) => return Err(BankError::Rejected(reason.clone())),
            Behavior::ReturnedLater => LineStatus::Returned,
            Behavior::Accept | Behavior::TimeoutOnce | Behavior::LostOnce => LineStatus::Settled,
        };
        let bank_ref = format!("BK-{:05}", state.received.len() + 1);
        state.received.push(Received {
            reference: request.reference.clone(),
            beneficiary: request.beneficiary.clone(),
            amount: request.amount,
            bank_ref: bank_ref.clone(),
            status,
        });
        match behavior {
            // Lệnh đã được ghi nhận phía ngân hàng, nhưng người gọi không nhận được phản hồi.
            Behavior::TimeoutOnce => Err(BankError::Timeout),
            _ => Ok(Transfer { bank_ref }),
        }
    }
}

impl Bank for FakeBank {
    async fn transfer(&self, request: &TransferRequest) -> Result<Transfer, BankError> {
        self.handle(request)
    }

    async fn lookup(&self, reference: &str) -> Result<Option<Transfer>, BankError> {
        let state = self.state.lock().expect("mutex");
        if state.lookup_down {
            return Err(BankError::Timeout);
        }
        Ok(state
            .received
            .iter()
            .find(|r| r.reference == reference)
            .map(|r| Transfer {
                bank_ref: r.bank_ref.clone(),
            }))
    }

    async fn statement(&self) -> Result<Vec<StatementLine>, BankError> {
        let state = self.state.lock().expect("mutex");
        let mut lines: Vec<StatementLine> = state
            .received
            .iter()
            .filter(|r| !state.hidden.contains(&r.reference))
            .map(|r| StatementLine {
                reference: r.reference.clone(),
                bank_ref: r.bank_ref.clone(),
                amount: state
                    .amount_override
                    .get(&r.reference)
                    .copied()
                    .unwrap_or(r.amount),
                status: r.status,
            })
            .collect();
        lines.extend(state.extra_lines.iter().cloned());
        Ok(lines)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(reference: &str, driver: &str, dong: i64) -> TransferRequest {
        TransferRequest {
            reference: reference.into(),
            beneficiary: driver.into(),
            amount: Money::from_dong(dong),
        }
    }

    #[tokio::test]
    async fn same_reference_is_one_transfer() {
        let bank = FakeBank::new();
        bank.set_behavior("DR-001", Behavior::TimeoutOnce);
        let req = request("r1", "DR-001", 100_000);
        assert_eq!(bank.transfer(&req).await, Err(BankError::Timeout));
        let retry = bank.transfer(&req).await.unwrap();
        assert_eq!(retry.bank_ref, "BK-00001");
        assert_eq!(bank.received_count("DR-001"), 1);
    }

    #[tokio::test]
    async fn lookup_tells_received_from_lost_without_paying() {
        let bank = FakeBank::new();
        bank.set_behavior("DR-001", Behavior::TimeoutOnce);
        bank.set_behavior("DR-002", Behavior::LostOnce);
        let received = request("r1", "DR-001", 100_000);
        let lost = request("r2", "DR-002", 100_000);
        assert_eq!(bank.transfer(&received).await, Err(BankError::Timeout));
        assert_eq!(bank.transfer(&lost).await, Err(BankError::Timeout));
        let found = bank.lookup("r1").await.unwrap().unwrap();
        assert_eq!(found.bank_ref, "BK-00001");
        assert_eq!(bank.lookup("r2").await, Ok(None));
        assert_eq!(bank.transfer_calls("DR-001"), 1);
        assert_eq!(bank.received_count("DR-002"), 0);
    }

    #[tokio::test]
    async fn new_reference_is_a_second_transfer() {
        let bank = FakeBank::new();
        bank.set_behavior("DR-001", Behavior::TimeoutOnce);
        assert_eq!(
            bank.transfer(&request("r1", "DR-001", 100_000)).await,
            Err(BankError::Timeout)
        );
        bank.transfer(&request("r2", "DR-001", 100_000))
            .await
            .unwrap();
        assert_eq!(bank.received_count("DR-001"), 2);
    }

    #[tokio::test]
    async fn same_reference_other_amount_is_rejected() {
        let bank = FakeBank::new();
        bank.transfer(&request("r1", "DR-001", 100_000))
            .await
            .unwrap();
        let err = bank
            .transfer(&request("r1", "DR-001", 90_000))
            .await
            .unwrap_err();
        assert!(matches!(err, BankError::Rejected(_)));
    }
}
