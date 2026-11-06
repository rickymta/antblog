//! Mẫu actor: đúng một task sở hữu `Ledger`; mọi nơi khác gửi lệnh qua `mpsc` bounded và
//! nhận kết quả qua `oneshot`. Không có khóa nào trên `Ledger`.

use tokio::sync::{broadcast, mpsc, oneshot};
use tokio::task::JoinHandle;

use ledger_core::{AccountId, EntryId, JournalEntry, Ledger, LedgerError, Money};

use crate::{PostOutcome, post_outcome};

/// Sự kiện phát ra sau mỗi bút toán MỚI được ghi (ghi lại key cũ thì không phát).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedgerEntryPosted {
    /// Mã bút toán.
    pub entry_id: EntryId,
    /// Khóa idempotency của bút toán.
    pub idempotency_key: String,
    /// Tổng số tiền chuyển (tổng các posting dương).
    pub moved: Money,
}

/// Lệnh gửi tới actor. Mỗi lệnh mang theo `oneshot::Sender` để trả lời.
#[derive(Debug)]
pub enum Command {
    /// Ghi một bút toán.
    Post {
        /// Bút toán cần ghi.
        entry: JournalEntry,
        /// Nơi trả kết quả.
        reply: oneshot::Sender<Result<PostOutcome, LedgerError>>,
    },
    /// Hỏi số dư một tài khoản.
    Balance {
        /// Tài khoản cần hỏi.
        account: AccountId,
        /// Nơi trả kết quả.
        reply: oneshot::Sender<Result<Money, LedgerError>>,
    },
}

/// Lỗi phía người gọi actor.
#[derive(Debug, PartialEq, Eq)]
pub enum ActorError {
    /// Actor đã dừng (hàng đợi đóng hoặc không trả lời).
    Closed,
    /// Lỗi nghiệp vụ của sổ cái.
    Ledger(LedgerError),
}

/// Vì sao `try_post` không đưa được lệnh vào hàng đợi.
#[derive(Debug, PartialEq, Eq)]
pub enum SubmitError {
    /// Hàng đợi đầy: đó là backpressure, người gọi quyết định chờ, bỏ hay báo bận.
    Full,
    /// Actor đã dừng.
    Closed,
}

/// Tay cầm để gọi actor. Rẻ khi `clone`; actor dừng khi mọi tay cầm bị drop.
#[derive(Debug, Clone)]
pub struct LedgerHandle {
    tx: mpsc::Sender<Command>,
    events: broadcast::Sender<LedgerEntryPosted>,
}

/// Tạo tay cầm và đầu nhận lệnh. `capacity` là sức chứa hàng đợi lệnh, `event_capacity` là
/// sức chứa kênh `broadcast`. Chưa có actor nào chạy: gọi [`spawn`] với đầu nhận.
pub fn channel(capacity: usize, event_capacity: usize) -> (LedgerHandle, mpsc::Receiver<Command>) {
    let (tx, rx) = mpsc::channel(capacity);
    let (events, _) = broadcast::channel(event_capacity);
    (LedgerHandle { tx, events }, rx)
}

/// Chạy actor. Task sở hữu `ledger`; khi hàng đợi đóng (mọi tay cầm bị drop) nó trả `Ledger` về.
pub fn spawn(
    mut ledger: Ledger,
    mut rx: mpsc::Receiver<Command>,
    handle: &LedgerHandle,
) -> JoinHandle<Ledger> {
    let events = handle.events.clone();
    tokio::spawn(async move {
        while let Some(command) = rx.recv().await {
            match command {
                Command::Post { entry, reply } => {
                    let moved = moved_amount(&entry);
                    let key = entry.idempotency_key().to_string();
                    let result = post_outcome(&mut ledger, entry);
                    if let Ok(outcome) = &result
                        && !outcome.replayed
                    {
                        // `send` chỉ lỗi khi không có subscriber nào; sổ vẫn đã ghi, nên bỏ qua.
                        let _ = events.send(LedgerEntryPosted {
                            entry_id: outcome.id,
                            idempotency_key: key,
                            moved,
                        });
                    }
                    // Người gọi có thể đã bỏ chờ (timeout, hủy): không phải lỗi của actor.
                    let _ = reply.send(result);
                }
                Command::Balance { account, reply } => {
                    let _ = reply.send(ledger.balance(&account));
                }
            }
        }
        ledger
    })
}

fn moved_amount(entry: &JournalEntry) -> Money {
    let sum: i64 = entry
        .postings()
        .iter()
        .map(|p| p.amount.dong())
        .filter(|d| *d > 0)
        .sum();
    Money::from_dong(sum)
}

impl LedgerHandle {
    /// Gửi lệnh ghi và chờ kết quả. Nếu hàng đợi đầy, `send().await` chờ chỗ trống.
    ///
    /// # Errors
    /// [`ActorError::Closed`] nếu actor đã dừng, [`ActorError::Ledger`] nếu ghi lỗi.
    pub async fn post(&self, entry: JournalEntry) -> Result<PostOutcome, ActorError> {
        let (reply, answer) = oneshot::channel();
        self.tx
            .send(Command::Post { entry, reply })
            .await
            .map_err(|_| ActorError::Closed)?;
        answer
            .await
            .map_err(|_| ActorError::Closed)?
            .map_err(ActorError::Ledger)
    }

    /// Thử đưa lệnh ghi vào hàng đợi mà KHÔNG chờ. Trả đầu nhận kết quả nếu vào được.
    ///
    /// # Errors
    /// [`SubmitError::Full`] khi hàng đợi đầy, [`SubmitError::Closed`] khi actor đã dừng.
    pub fn try_post(
        &self,
        entry: JournalEntry,
    ) -> Result<oneshot::Receiver<Result<PostOutcome, LedgerError>>, SubmitError> {
        let (reply, answer) = oneshot::channel();
        self.tx
            .try_send(Command::Post { entry, reply })
            .map_err(|e| match e {
                mpsc::error::TrySendError::Full(_) => SubmitError::Full,
                mpsc::error::TrySendError::Closed(_) => SubmitError::Closed,
            })?;
        Ok(answer)
    }

    /// Hỏi số dư qua actor.
    ///
    /// # Errors
    /// Như [`LedgerHandle::post`].
    pub async fn balance(&self, account: AccountId) -> Result<Money, ActorError> {
        let (reply, answer) = oneshot::channel();
        self.tx
            .send(Command::Balance { account, reply })
            .await
            .map_err(|_| ActorError::Closed)?;
        answer
            .await
            .map_err(|_| ActorError::Closed)?
            .map_err(ActorError::Ledger)
    }

    /// Đăng ký nhận `LedgerEntryPosted`. Chỉ thấy sự kiện phát SAU lúc gọi.
    pub fn subscribe(&self) -> broadcast::Receiver<LedgerEntryPosted> {
        self.events.subscribe()
    }
}
