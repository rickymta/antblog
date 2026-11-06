//! Bộ test hợp đồng viết MỘT lần, chạy cho mọi adapter của `LedgerRepository`.
//!
//! Điều kiện: repository vừa tạo, đã mở các tài khoản của [`crate::CHART`], chưa có bút toán nào.

use std::sync::Arc;

use ledger_core::{
    AccountId, JournalEntry, Money, Posting, TripCharge, default_policy, fare_for_km,
};

use crate::error::RepoError;
use crate::repository::{LedgerRepository, LedgerUnitOfWork};
use crate::service::post_trip_capture;

fn acct(raw: &str) -> AccountId {
    AccountId::new(raw).expect("mã tài khoản hợp lệ")
}

fn trip(id: &str, km: u32) -> TripCharge {
    TripCharge {
        trip_id: id.to_string(),
        customer: acct("customer:CU-001"),
        driver: acct("driver:DR-001"),
        fare: fare_for_km(km),
    }
}

fn ensure(ok: bool, msg: impl Into<String>) -> Result<(), String> {
    if ok { Ok(()) } else { Err(msg.into()) }
}

async fn expect_balance<R: LedgerRepository>(
    repo: &R,
    account: &str,
    dong: i64,
) -> Result<(), String> {
    let got = repo
        .balance(&acct(account))
        .await
        .map_err(|e| e.to_string())?;
    ensure(
        got == Money::from_dong(dong),
        format!("số dư {account}: cần {dong}, có {}", got.dong()),
    )
}

async fn expect_totals<R: LedgerRepository>(
    repo: &R,
    customer: i64,
    platform: i64,
    driver: i64,
) -> Result<(), String> {
    expect_balance(repo, "customer:CU-001", customer).await?;
    expect_balance(repo, "platform:revenue", platform).await?;
    expect_balance(repo, "driver:DR-001", driver).await?;
    // Sổ cân: mọi bút toán tổng 0 nên tổng số dư các tài khoản phải bằng 0.
    expect_balance(repo, "psp:vixanh:clearing", 0).await?;
    ensure(customer + platform + driver == 0, "tổng số dư khác 0")
}

/// Chạy toàn bộ hợp đồng; trả tên các ca đã qua, hoặc lý do ca đầu tiên hỏng.
///
/// `R: 'static` và `Arc<R>` để cùng chữ ký với [`run_concurrent`], nơi `tokio::spawn` future của service.
///
/// # Errors
/// Chuỗi mô tả ca hỏng.
pub async fn run<R>(repo: Arc<R>) -> Result<Vec<&'static str>, String>
where
    R: LedgerRepository + 'static,
{
    let repo = &*repo;
    let policy = default_policy();
    let mut passed = Vec::new();
    let err = |e: RepoError| e.to_string();

    // 1. Ghi chuyến TN-0001: ba posting, số dư theo dòng tiền.
    let first = post_trip_capture(repo, &trip("TN-0001", 5), &policy)
        .await
        .map_err(err)?;
    ensure(
        first.id.to_string() == "JE-000001" && !first.replayed,
        "ghi lần đầu",
    )?;
    expect_totals(repo, -25_500, 5_100, 20_400).await?;
    passed.push("ghi TN-0001: JE-000001, số dư -25.500 / 5.100 / 20.400");

    // 2. Idempotent: cùng key, cùng nội dung -> bút toán cũ, số dư không đổi.
    let again = post_trip_capture(repo, &trip("TN-0001", 5), &policy)
        .await
        .map_err(err)?;
    ensure(
        again.id == first.id && again.replayed,
        "ghi lần hai phải là replay",
    )?;
    expect_totals(repo, -25_500, 5_100, 20_400).await?;
    passed.push("idempotent: ghi lại trả JE-000001, replayed = true, số dư giữ nguyên");

    // 3. Cùng key, khác nội dung (6 km thay vì 5 km) -> IdempotencyConflict.
    let conflict = post_trip_capture(repo, &trip("TN-0001", 6), &policy).await;
    ensure(
        matches!(
            conflict,
            Err(RepoError::Domain(
                ledger_core::LedgerError::IdempotencyConflict { .. }
            ))
        ),
        format!("cần IdempotencyConflict, có {conflict:?}"),
    )?;
    expect_totals(repo, -25_500, 5_100, 20_400).await?;
    passed.push("cùng key khác nội dung: IdempotencyConflict, số dư giữ nguyên");

    // 4. Rollback khi drop: ghi trong Uow rồi bỏ, không commit.
    {
        let entry =
            ledger_core::capture_entry(&trip("TN-0002", 2), &policy).map_err(|e| e.to_string())?;
        let mut uow = repo.begin().await.map_err(err)?;
        let out = uow.post(&entry).await.map_err(err)?;
        ensure(!out.replayed, "TN-0002 trong Uow là bản mới")?;
        let inside = LedgerUnitOfWork::balance(&mut uow, &acct("customer:CU-001"))
            .await
            .map_err(err)?;
        ensure(
            inside == Money::from_dong(-37_500),
            "trong Uow phải thấy chính dòng mình vừa ghi",
        )?;
    } // uow bị thả ở đây, chưa commit
    expect_totals(repo, -25_500, 5_100, 20_400).await?;
    let redo = post_trip_capture(repo, &trip("TN-0002", 2), &policy)
        .await
        .map_err(err)?;
    ensure(
        !redo.replayed,
        "bút toán đã rollback thì ghi lại phải là bản mới",
    )?;
    expect_totals(repo, -37_500, 7_500, 30_000).await?;
    passed.push("rollback khi drop: Uow bỏ dở không để lại bút toán lẫn số dư");

    // 5. Hai bút toán trong một Uow, commit một lần.
    {
        let mut uow = repo.begin().await.map_err(err)?;
        for (id, km) in [("TN-0003", 3), ("TN-0004", 1)] {
            let e =
                ledger_core::capture_entry(&trip(id, km), &policy).map_err(|e| e.to_string())?;
            uow.post(&e).await.map_err(err)?;
        }
        uow.commit().await.map_err(err)?;
    }
    expect_totals(repo, -66_000, 13_200, 52_800).await?;
    passed.push("một Uow, hai bút toán, một commit: số dư -66.000 / 13.200 / 52.800");

    // 6. Lỗi giữa chừng: bút toán thứ nhất đã post trong Uow, bút toán thứ hai lỗi -> không còn gì.
    {
        let mut uow = repo.begin().await.map_err(err)?;
        let ok =
            ledger_core::capture_entry(&trip("TN-0005", 4), &policy).map_err(|e| e.to_string())?;
        uow.post(&ok).await.map_err(err)?;
        let ghost = JournalEntry::try_new(
            "manual:unknown-account",
            vec![
                Posting::new(acct("customer:CU-001"), Money::from_dong(-100)),
                Posting::new(acct("driver:DR-404"), Money::from_dong(100)),
            ],
        )
        .map_err(|e| e.to_string())?;
        let bad = uow.post(&ghost).await;
        ensure(
            matches!(bad, Err(RepoError::UnknownAccount(ref a)) if a.as_str() == "driver:DR-404"),
            format!("cần UnknownAccount, có {bad:?}"),
        )?;
    } // bỏ Uow: TN-0005 cũng biến mất
    expect_totals(repo, -66_000, 13_200, 52_800).await?;
    passed.push("lỗi giữa chừng: tài khoản lạ bị từ chối, bút toán post trước đó cũng hủy");

    // 7. Posting bằng 0 bị từ chối giống nhau ở mọi adapter.
    let zero = JournalEntry::try_new(
        "manual:zero-posting",
        vec![
            Posting::new(acct("customer:CU-001"), Money::from_dong(-100)),
            Posting::new(acct("driver:DR-001"), Money::from_dong(100)),
            Posting::new(acct("platform:revenue"), Money::ZERO),
        ],
    )
    .map_err(|e| e.to_string())?;
    let mut uow = repo.begin().await.map_err(err)?;
    let res = uow.post(&zero).await;
    drop(uow);
    ensure(
        matches!(res, Err(RepoError::ZeroAmount { .. })),
        format!("cần ZeroAmount, có {res:?}"),
    )?;
    expect_totals(repo, -66_000, 13_200, 52_800).await?;
    passed.push("posting bằng 0: ZeroAmount");

    Ok(passed)
}

/// Ca đồng thời: tám task cùng thu cước một chuyến; đúng một task ghi mới, bảy task nhận replay.
///
/// Gọi sau [`run`] trên cùng repository (nó dùng chuyến `TN-0006`).
///
/// # Errors
/// Chuỗi mô tả ca hỏng.
pub async fn run_concurrent<R>(repo: Arc<R>) -> Result<&'static str, String>
where
    R: LedgerRepository + 'static,
{
    let mut handles = Vec::new();
    for _ in 0..8 {
        let repo = Arc::clone(&repo);
        // spawn đòi future là Send + 'static; future của service generic qua được nhờ `+ Send` trong trait.
        handles.push(tokio::spawn(async move {
            let policy = default_policy();
            post_trip_capture(&*repo, &trip("TN-0006", 2), &policy).await
        }));
    }
    let mut fresh = 0;
    let mut ids = Vec::new();
    for h in handles {
        let out = h
            .await
            .map_err(|e| e.to_string())?
            .map_err(|e| e.to_string())?;
        fresh += usize::from(!out.replayed);
        ids.push(out.id);
    }
    ensure(fresh == 1, format!("cần đúng 1 bản ghi mới, có {fresh}"))?;
    ensure(
        ids.iter().all(|id| *id == ids[0]),
        "mọi task phải thấy cùng một bút toán",
    )?;
    expect_totals(&*repo, -78_000, 15_600, 62_400).await?;
    Ok("8 task đồng thời cùng key: đúng 1 bản ghi mới, 7 replay, số dư -78.000 / 15.600 / 62.400")
}
