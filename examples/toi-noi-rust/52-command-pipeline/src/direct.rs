//! Đối chứng: cùng nghiệp vụ `CapturePayment` viết thành MỘT hàm thẳng, không trait, không lớp bọc.
//! Validate, transaction, thu sự kiện, ghi outbox và commit nằm hết trong thân hàm.

use ledger_core::{AccountId, Money, TripCharge, default_policy};
use sqlx::PgPool;

use crate::command::{AppError, Validate};
use crate::domain::TripEntry;
use crate::handlers::{
    CapturePayment, CaptureReceipt, Header, display_id, insert_header, insert_posting,
};

/// Thu cước một chuyến. Lỗi ở bất kỳ bước nào thoát bằng `?`: `tx` bị thả trước khi commit
/// nên giao dịch bị hủy (xem bài 26 về cách `Drop` của `Transaction` hoạt động).
///
/// # Errors
/// [`AppError::Validation`], [`AppError::Domain`] hoặc [`AppError::Storage`].
pub async fn capture_direct(
    pool: &PgPool,
    cmd: CapturePayment,
    now: &str,
) -> Result<CaptureReceipt, AppError> {
    cmd.validate().map_err(AppError::Validation)?;
    let mut aggregate = {
        let trip = TripCharge {
            trip_id: cmd.trip_id,
            customer: AccountId::new(&format!("customer:{}", cmd.customer))?,
            driver: AccountId::new(&format!("driver:{}", cmd.driver))?,
            fare: Money::from_dong(cmd.fare_dong),
        };
        TripEntry::capture(&trip, &default_policy())?
    };
    let postings: Vec<(String, i64)> = aggregate
        .entry()
        .postings()
        .iter()
        .map(|p| (p.account.to_string(), p.amount.dong()))
        .collect();

    let mut tx = pool.begin().await?;
    let (id, replayed) = match insert_header(&mut tx, aggregate.entry().idempotency_key()).await? {
        Header::New(id) => (id, false),
        Header::Existing(id) => (id, true),
    };
    if !replayed {
        for (index, (account, amount)) in postings.iter().enumerate() {
            let line_no = i32::try_from(index + 1).expect("ít dòng");
            insert_posting(&mut tx, id, line_no, account, *amount).await?;
        }
        aggregate.mark_posted(&display_id(id), now);
        for event in aggregate.take_events() {
            sqlx::query("INSERT INTO outbox (topic, key, payload) VALUES ($1, $2, $3::jsonb)")
                .bind(event.topic())
                .bind(event.key())
                .bind(event.payload())
                .execute(&mut *tx)
                .await?;
        }
    }
    tx.commit().await?;

    Ok(CaptureReceipt {
        entry_id: display_id(id),
        replayed,
        platform_fee_dong: postings[1].1,
        driver_net_dong: postings[2].1,
    })
}
