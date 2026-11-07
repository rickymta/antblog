//! Dữ liệu mẫu: các chuyến đã hoàn tất, ghi theo dòng tiền chuẩn hai bước của bài 13.

use anyhow::Result;
use jiff::Timestamp;
use ledger_core::{
    AccountId, FeePolicy, JournalEntry, JournalEntryBuilder, Money, default_policy, fare_for_km,
};
use sqlx::PgPool;

use crate::store::{ensure_account, post};

fn acct(raw: &str) -> Result<AccountId> {
    Ok(AccountId::new(raw)?)
}

/// Hai bút toán của một chuyến 5 km: capture (khách → clearing VíXanh) và split
/// (clearing → doanh thu nền tảng và ví tài xế).
///
/// # Errors
/// Lỗi tính phí hoặc dựng bút toán.
pub fn trip_entries(trip_no: u32, driver: &str) -> Result<(JournalEntry, JournalEntry)> {
    let fare = fare_for_km(5);
    let split = default_policy().split(fare)?;
    let neg_fare = Money::from_dong(-fare.dong());
    let trip = format!("TN-{trip_no:04}");
    let capture = JournalEntryBuilder::new(&format!("trip:{trip}:capture"))
        .posting(acct("customer:CU-001")?, neg_fare)
        .posting(acct("psp:vixanh:clearing")?, fare)
        .build()?;
    let split_entry = JournalEntryBuilder::new(&format!("trip:{trip}:split"))
        .posting(acct("psp:vixanh:clearing")?, neg_fare)
        .posting(acct("platform:revenue")?, split.platform_fee)
        .posting(acct(&format!("driver:{driver}"))?, split.driver_net)
        .build()?;
    Ok((capture, split_entry))
}

/// Ghi một chuyến hoàn tất lúc `at` (cả hai bút toán, một transaction).
///
/// # Errors
/// Lỗi SQL hoặc dựng bút toán.
pub async fn post_trip(pool: &PgPool, trip_no: u32, driver: &str, at: Timestamp) -> Result<()> {
    let (capture, split) = trip_entries(trip_no, driver)?;
    let mut tx = pool.begin().await?;
    ensure_account(&mut tx, &format!("driver:{driver}"), "liability").await?;
    post(&mut tx, &capture, at, None).await?;
    post(&mut tx, &split, at, None).await?;
    tx.commit().await?;
    Ok(())
}

/// `TN-0001` đến `TN-0005` của `DR-001` rải qua nửa đêm UTC và nửa đêm giờ Việt Nam;
/// bốn tài xế còn lại mỗi người vài chuyến trong ngày. Ví sau khi ghi:
/// DR-001 102.000, DR-002 40.800, DR-003 244.800, DR-004 61.200, DR-005 81.600.
///
/// # Errors
/// Lỗi SQL hoặc dựng bút toán.
pub async fn seed_standard(pool: &PgPool) -> Result<()> {
    let dr001 = [
        "2026-11-05T09:00:00Z", // 16:00 ICT, 05/11
        "2026-11-05T16:59:59Z", // 23:59:59 ICT, 05/11
        "2026-11-05T17:00:00Z", // 00:00:00 ICT, 06/11 (vẫn là 05/11 theo UTC)
        "2026-11-05T18:30:00Z", // 01:30 ICT, 06/11
        "2026-11-06T02:00:00Z", // 09:00 ICT, 06/11
    ];
    let mut trip_no = 0;
    for at in dr001 {
        trip_no += 1;
        post_trip(pool, trip_no, "DR-001", at.parse()?).await?;
    }
    let at: Timestamp = "2026-11-06T03:00:00Z".parse()?;
    for (driver, trips) in [("DR-002", 2), ("DR-003", 12), ("DR-004", 3), ("DR-005", 4)] {
        for _ in 0..trips {
            trip_no += 1;
            post_trip(pool, trip_no, driver, at).await?;
        }
    }
    Ok(())
}

/// `drivers` tài xế `DR-001`…, mỗi người `trips_each` chuyến lúc 10:00 ICT ngày 06/11.
///
/// # Errors
/// Lỗi SQL hoặc dựng bút toán.
pub async fn seed_uniform(pool: &PgPool, drivers: u32, trips_each: u32) -> Result<()> {
    let at: Timestamp = "2026-11-06T03:00:00Z".parse()?;
    let mut trip_no = 0;
    for d in 1..=drivers {
        for _ in 0..trips_each {
            trip_no += 1;
            post_trip(pool, trip_no, &format!("DR-{d:03}"), at).await?;
        }
    }
    Ok(())
}
