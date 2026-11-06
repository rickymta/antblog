//! Command và handler của lab. Handler chỉ biết `Ctx`; transaction, trace, outbox là việc của lớp bọc.

use ledger_core::{AccountId, Money, TripCharge, default_policy};
use sqlx::PgConnection;

use crate::command::{AppError, Command, CommandHandler, Ctx, Validate};
use crate::domain::TripEntry;
use crate::events::{DomainEvent, LedgerEntryPosted, PostingDto, SCHEMA_VERSION};

/// Thu cước một chuyến đã hoàn thành.
#[derive(Debug, Clone)]
pub struct CapturePayment {
    /// Mã chuyến, ví dụ `TN-0001`.
    pub trip_id: String,
    /// Mã khách, ví dụ `CU-001` (tài khoản là `customer:<mã>`).
    pub customer: String,
    /// Mã tài xế, ví dụ `DR-001` (tài khoản là `driver:<mã>`).
    pub driver: String,
    /// Cước, đơn vị đồng.
    pub fare_dong: i64,
}

/// Kết quả của `CapturePayment`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureReceipt {
    /// Mã bút toán, `JE-000001`.
    pub entry_id: String,
    /// `true` nếu khóa idempotency đã có từ trước (không ghi gì mới).
    pub replayed: bool,
    /// Phí nền tảng, đồng.
    pub platform_fee_dong: i64,
    /// Tài xế nhận, đồng.
    pub driver_net_dong: i64,
}

impl Command for CapturePayment {
    type Output = CaptureReceipt;
    const NAME: &'static str = "CapturePayment";
}

impl Validate for CapturePayment {
    fn validate(&self) -> Result<(), String> {
        if self.trip_id.trim().is_empty() {
            return Err("trip_id rỗng".to_string());
        }
        if self.fare_dong <= 0 {
            return Err(format!("fare_dong phải dương, nhận {}", self.fare_dong));
        }
        Ok(())
    }
}

/// Nhả một khoản giữ tiền. Lab CỐ Ý không viết handler cho command này,
/// để chứng minh dispatch generic bắt thiếu handler lúc biên dịch.
#[derive(Debug, Clone)]
pub struct VoidHold {
    /// Mã khoản giữ.
    pub hold_id: i64,
}

impl Command for VoidHold {
    type Output = ();
    const NAME: &'static str = "VoidHold";
}

impl Validate for VoidHold {
    fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

/// Lệnh back-office ghi bút toán thô, BỎ QUA aggregate: dùng để mô phỏng một bug
/// ở tầng ứng dụng, khi chỉ còn trigger của database chặn bút toán lệch.
#[derive(Debug, Clone)]
pub struct PostRawEntry {
    /// Khóa idempotency.
    pub key: String,
    /// Các dòng `(tài khoản, số tiền có dấu)`.
    pub lines: Vec<(String, i64)>,
}

impl Command for PostRawEntry {
    type Output = String;
    const NAME: &'static str = "PostRawEntry";
}

impl Validate for PostRawEntry {
    fn validate(&self) -> Result<(), String> {
        if self.key.trim().is_empty() {
            return Err("key rỗng".to_string());
        }
        Ok(())
    }
}

/// Handler của lab: một struct hiện thực `CommandHandler` cho từng command nó biết.
#[derive(Debug, Default, Clone, Copy)]
pub struct LedgerHandlers;

/// Kết quả ghi header bút toán.
pub(crate) enum Header {
    New(i64),
    Existing(i64),
}

pub(crate) fn display_id(id: i64) -> String {
    format!("JE-{id:06}")
}

// Ghi header; trùng khóa thì không làm gì rồi đọc id cũ.
pub(crate) async fn insert_header(conn: &mut PgConnection, key: &str) -> Result<Header, AppError> {
    let inserted: Option<i64> = sqlx::query_scalar(
        "INSERT INTO journal_entries (idempotency_key) VALUES ($1) \
         ON CONFLICT (idempotency_key) DO NOTHING RETURNING id",
    )
    .bind(key)
    .fetch_optional(&mut *conn)
    .await?;
    if let Some(id) = inserted {
        return Ok(Header::New(id));
    }
    let id: i64 = sqlx::query_scalar("SELECT id FROM journal_entries WHERE idempotency_key = $1")
        .bind(key)
        .fetch_one(&mut *conn)
        .await?;
    Ok(Header::Existing(id))
}

pub(crate) async fn insert_posting(
    conn: &mut PgConnection,
    entry_id: i64,
    line_no: i32,
    account: &str,
    amount_dong: i64,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO postings (entry_id, line_no, account_id, amount_dong) VALUES ($1, $2, $3, $4)",
    )
    .bind(entry_id)
    .bind(line_no)
    .bind(account)
    .bind(amount_dong)
    .execute(&mut *conn)
    .await?;
    Ok(())
}

impl CommandHandler<CapturePayment> for LedgerHandlers {
    async fn handle(&self, cmd: CapturePayment, ctx: &mut Ctx) -> Result<CaptureReceipt, AppError> {
        ctx.log(format_args!("handler: CapturePayment trip={}", cmd.trip_id));

        // Dựng aggregate trong khối riêng: chính sách phí không sống qua điểm await.
        let mut aggregate = {
            let trip = TripCharge {
                trip_id: cmd.trip_id.clone(),
                customer: AccountId::new(&format!("customer:{}", cmd.customer))?,
                driver: AccountId::new(&format!("driver:{}", cmd.driver))?,
                fare: Money::from_dong(cmd.fare_dong),
            };
            TripEntry::capture(&trip, &default_policy())?
        };
        let amounts: Vec<i64> = aggregate
            .entry()
            .postings()
            .iter()
            .map(|p| p.amount.dong())
            .collect();

        let header = insert_header(ctx.tx()?, aggregate.entry().idempotency_key()).await?;
        let (id, replayed) = match header {
            Header::New(id) => (id, false),
            Header::Existing(id) => (id, true),
        };

        if replayed {
            // Không có thay đổi trạng thái thì không có sự kiện.
            ctx.log(format_args!(
                "handler: key đã có ({}), không ghi, không sinh sự kiện",
                display_id(id)
            ));
        } else {
            ctx.log(format_args!(
                "handler: đã INSERT journal_entries {}",
                display_id(id)
            ));
            let postings: Vec<(String, i64)> = aggregate
                .entry()
                .postings()
                .iter()
                .map(|p| (p.account.to_string(), p.amount.dong()))
                .collect();
            for (index, (account, amount)) in postings.iter().enumerate() {
                let line_no = i32::try_from(index + 1).expect("ít dòng");
                insert_posting(ctx.tx()?, id, line_no, account, *amount).await?;
            }
            aggregate.mark_posted(&display_id(id), ctx.now());
            let events = aggregate.take_events();
            ctx.log(format_args!(
                "handler: aggregate sinh {} sự kiện, chuyển cho ctx",
                events.len()
            ));
            ctx.raise(events);
        }

        Ok(CaptureReceipt {
            entry_id: display_id(id),
            replayed,
            platform_fee_dong: amounts[1],
            driver_net_dong: amounts[2],
        })
    }
}

impl CommandHandler<PostRawEntry> for LedgerHandlers {
    async fn handle(&self, cmd: PostRawEntry, ctx: &mut Ctx) -> Result<String, AppError> {
        ctx.log(format_args!("handler: PostRawEntry key={}", cmd.key));
        let id = match insert_header(ctx.tx()?, &cmd.key).await? {
            Header::New(id) | Header::Existing(id) => id,
        };
        for (index, (account, amount)) in cmd.lines.iter().enumerate() {
            let line_no = i32::try_from(index + 1).expect("ít dòng");
            insert_posting(ctx.tx()?, id, line_no, account, *amount).await?;
        }
        ctx.log(format_args!(
            "handler: đã ghi {} dòng cho {}, tổng {}",
            cmd.lines.len(),
            display_id(id),
            cmd.lines.iter().map(|(_, a)| a).sum::<i64>()
        ));
        // Bug có chủ ý: sinh sự kiện ngay sau khi INSERT, không đợi gì cả.
        ctx.raise([DomainEvent::EntryPosted(LedgerEntryPosted {
            schema_version: SCHEMA_VERSION,
            entry_id: display_id(id),
            idempotency_key: cmd.key.clone(),
            postings: cmd
                .lines
                .iter()
                .map(|(account, amount)| PostingDto {
                    account: account.clone(),
                    amount_dong: *amount,
                })
                .collect(),
            posted_at: ctx.now().to_string(),
        })]);
        Ok(display_id(id))
    }
}

/// Command không chạm database, dùng để đo chi phí lớp bọc và để test thứ tự lớp.
#[derive(Debug, Clone)]
pub struct Ping {
    /// Số đưa vào.
    pub n: u32,
}

impl Command for Ping {
    type Output = u32;
    const NAME: &'static str = "Ping";
}

impl Validate for Ping {
    fn validate(&self) -> Result<(), String> {
        if self.n == 0 {
            return Err("n phải khác 0".to_string());
        }
        Ok(())
    }
}

/// Handler thuần: trả `n + 1`, không cấp phát, không await thật.
#[derive(Debug, Default, Clone, Copy)]
pub struct Echo;

impl CommandHandler<Ping> for Echo {
    async fn handle(&self, cmd: Ping, ctx: &mut Ctx) -> Result<u32, AppError> {
        ctx.log(format_args!("handler: Ping n={}", cmd.n));
        Ok(cmd.n + 1)
    }
}
