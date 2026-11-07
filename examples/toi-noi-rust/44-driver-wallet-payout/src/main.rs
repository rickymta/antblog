//! Chạy các thí nghiệm của bài 44 và in kết quả. `--verify` kiểm thêm các giá trị bài viết nêu.

use std::process::ExitCode;

use anyhow::Result;
use ledger_44_driver_wallet_payout::lab::with_pg;
use ledger_44_driver_wallet_payout::payout::Status;
use ledger_44_driver_wallet_payout::scenarios::{
    ParallelMode, earnings, idempotency, lost_before_receipt, parallel_batches, payout_flow,
    reconcile_anomalies, timeout_retry, withdrawal_limit,
};
use ledger_core::Money;

fn m(dong: i64) -> String {
    Money::from_dong(dong).to_string()
}

fn wallets_line(items: &[(String, i64)]) -> String {
    items
        .iter()
        .map(|(d, n)| format!("{d} {}", m(*n)))
        .collect::<Vec<_>>()
        .join(", ")
}

fn statuses_line(items: &[(String, Status)]) -> String {
    items
        .iter()
        .map(|(d, s)| format!("{d}={s}"))
        .collect::<Vec<_>>()
        .join(" ")
}

async fn run(verify: bool) -> Result<()> {
    with_pg(async move |lab| {
        let r = earnings(&lab).await?;
        println!("== 1. Thu nhập DR-001 theo ngày ==");
        println!("-- cắt ngày theo Asia/Ho_Chi_Minh");
        for d in &r.ict {
            println!("{}  {} chuyến  {}", d.day, d.trips, m(d.earned_dong));
        }
        println!("-- cắt ngày theo UTC");
        for d in &r.utc {
            println!("{}  {} chuyến  {}", d.day, d.trips, m(d.earned_dong));
        }
        println!("-- mốc sát nửa đêm (UTC, ngày UTC, ngày PostgreSQL, ngày jiff)");
        for b in &r.boundaries {
            println!("{}  {}  {}  {}", b.at, b.utc_day, b.sql_day, b.jiff_day);
        }
        println!(
            "cửa sổ ngày của 2026-11-06T10:00:00Z: [{}, {})",
            r.window.0, r.window.1
        );
        if verify {
            r.verify()?;
        }
        Ok(())
    })
    .await?;

    with_pg(async move |lab| {
        let r = payout_flow(&lab).await?;
        println!("== 2. Một lô payout từ tạo tới đối chiếu ==");
        println!(
            "ví trước khi tạo lô     : {}",
            wallets_line(&r.wallets_before)
        );
        let items: Vec<String> = r
            .created
            .items
            .iter()
            .map(|(d, n)| format!("{d} {}", m(*n)))
            .collect();
        println!("khoản trong lô          : {}", items.join(", "));
        println!(
            "ví sau khi tạo lô       : {}",
            wallets_line(&r.wallets_after_create)
        );
        println!("bank:payout:clearing    : {}", m(r.clearing_after_create));
        println!("trạng thái lô           : {}", r.batch_after_create);
        println!("-- gửi lần 1: {:?}", r.send1);
        println!(
            "các khoản               : {}",
            statuses_line(&r.statuses_after_send1)
        );
        println!("trạng thái lô           : {}", r.batch_after_send1);
        println!("ví DR-004 (bị từ chối)  : {}", m(r.dr004_after_reject));
        println!("-- gửi lần 2: {:?}", r.send2);
        println!(
            "DR-001: {} lần gửi, ngân hàng nhận {} lệnh; trạng thái lô {}",
            r.attempts_dr001, r.bank_orders_dr001, r.batch_after_send2
        );
        println!(
            "-- đối chiếu: settled {}, returned {}, lệch {}, thiếu {}, lạ {}",
            r.reconcile.settled.len(),
            r.reconcile.returned.len(),
            r.reconcile.amount_mismatch.len(),
            r.reconcile.missing_in_statement.len(),
            r.reconcile.unknown_in_statement.len()
        );
        let fin: Vec<(String, Status)> = r
            .final_rows
            .iter()
            .map(|p| (p.driver_id.clone(), p.status))
            .collect();
        println!("các khoản               : {}", statuses_line(&fin));
        println!("trạng thái lô           : {}", r.batch_final);
        println!(
            "ví cuối                 : {}",
            wallets_line(&r.wallets_final)
        );
        println!("bank:payout:clearing    : {}", m(r.clearing_final));
        for (key, ok) in &r.reversal_links {
            println!(
                "bút toán đảo            : {key}:reversal, reversal_of đúng bút toán gốc = {ok}"
            );
        }
        println!("tổng mọi posting        : {}", r.grand_total);
        println!("account_balances = posting : {}", r.projection_ok);
        if verify {
            r.verify()?;
        }
        Ok(())
    })
    .await?;

    with_pg(async move |lab| {
        let r = withdrawal_limit(&lab).await?;
        println!("== 3. Giới hạn rút 150.000 mỗi ngày giờ Việt Nam, đồng hồ truyền vào ==");
        println!(
            "10:00:00Z  lô .1: DR-003 rút {}",
            m(r.first_dr003.unwrap_or(0))
        );
        println!(
            "16:59:59Z  hạn mức còn lại của DR-003: {}",
            m(r.remaining_before_midnight)
        );
        println!(
            "16:59:59Z  lô .2: {} khoản, bỏ qua {:?}",
            r.second_items, r.second_skipped
        );
        println!(
            "17:00:00Z  hạn mức còn lại của DR-003: {}",
            m(r.remaining_after_midnight)
        );
        println!("17:00:00Z  lô 07/11: {:?}", r.third_items);
        println!(
            "nếu cắt ngày theo UTC, DR-003 đã dùng lúc 17:00:00Z: {}",
            m(r.used_if_utc_day)
        );
        if verify {
            r.verify()?;
        }
        Ok(())
    })
    .await?;

    for mode in [
        ParallelMode::NaiveNoUnique,
        ParallelMode::NaiveUnique,
        ParallelMode::SkipLocked,
    ] {
        with_pg(async move |lab| {
            let r = parallel_batches(&lab, mode).await?;
            println!("== 4. Hai lô song song, tối đa 3 ví mỗi lô: {mode:?} ==");
            // Lô nào chạy trước là may rủi nên xếp theo kết quả: lô thành công lớn trước, lô lỗi sau.
            let mut outcomes: Vec<_> = r.outcomes.iter().collect();
            outcomes.sort_by_key(|o| match o {
                Ok(drivers) => (
                    0,
                    std::cmp::Reverse(drivers.len()),
                    drivers.first().cloned(),
                ),
                Err(_) => (1, std::cmp::Reverse(0), None),
            });
            for (name, outcome) in ["lô thứ nhất", "lô thứ hai"].iter().zip(outcomes) {
                match outcome {
                    Ok(drivers) => println!("{name}: chọn {}", drivers.join(", ")),
                    Err(code) => println!("{name}: lỗi SQLSTATE {code}, rollback cả lô"),
                }
            }
            let per: Vec<String> = r
                .payouts_per_driver
                .iter()
                .map(|(d, n)| format!("{d}={n}"))
                .collect();
            println!("số khoản mỗi tài xế : {}", per.join(" "));
            println!("ví                  : {}", wallets_line(&r.wallets));
            println!("số lô còn lại       : {}", r.batches);
            if verify {
                r.verify()?;
            }
            Ok(())
        })
        .await?;
    }

    with_pg(async move |lab| {
        let r = timeout_retry(&lab).await?;
        println!("== 5. Timeout sau khi ngân hàng đã nhận lệnh ==");
        println!(
            "lượt 1: timeout {}, khoản {}, {} lần gửi, last_error {:?}",
            r.first_timed_out, r.after_first.0, r.after_first.1, r.after_first.2
        );
        println!("ví DR-001 sau lượt 1   : {}", m(r.wallet_after_first));
        println!(
            "ngân hàng đã nhận      : {} lệnh",
            r.bank_orders_after_first
        );
        println!(
            "lượt 2 (tra cứu lỗi)   : {:?}, khoản {}, transfer gọi {} lần",
            r.lookup_down, r.after_lookup_down.0, r.after_lookup_down.1
        );
        println!(
            "lượt 3 (tra cứu thấy)  : {:?}, khoản {}, {} lần gửi, bank_ref {:?}",
            r.second, r.after_second.0, r.after_second.1, r.after_second.2
        );
        println!(
            "ngân hàng đã nhận      : {} lệnh, transfer gọi {} lần",
            r.bank_orders_after_second, r.transfer_calls
        );
        println!(
            "retry bằng mã mới      : ngân hàng nhận {} lệnh cho cùng một tài xế",
            r.bank_orders_with_new_reference
        );
        if verify {
            r.verify()?;
        }
        Ok(())
    })
    .await?;

    with_pg(async move |lab| {
        let r = lost_before_receipt(&lab).await?;
        println!("== 5b. Lệnh mất trước khi tới ngân hàng ==");
        println!(
            "lượt 1                 : {:?}, ngân hàng đã nhận {} lệnh",
            r.first, r.orders_after_first
        );
        println!(
            "lượt 2 (tra cứu: chưa có): {:?}, khoản {}, {} lần gửi, cùng mã {}",
            r.second, r.after_second.0, r.after_second.1, r.after_second.2
        );
        println!("ngân hàng đã nhận      : {} lệnh", r.orders_after_second);
        if verify {
            r.verify()?;
        }
        Ok(())
    })
    .await?;

    with_pg(async move |lab| {
        let r = reconcile_anomalies(&lab).await?;
        println!("== 6. Đối chiếu với sao kê có lệch ==");
        let f = &r.first;
        println!("settled                : {}", f.settled.len());
        println!("  trong đó gỡ nút thắt timeout: {:?}", f.recovered_after_timeout);
        println!("returned (đã đảo)      : {:?}", f.returned);
        println!("lệch số tiền           : {:?}", f.amount_mismatch);
        println!("thiếu trong sao kê     : {:?}", f.missing_in_statement);
        println!("dòng lạ trong sao kê   : {:?}", f.unknown_in_statement);
        println!("các khoản              : {}", statuses_line(&r.statuses));
        println!("trạng thái lô          : {}", r.batch);
        println!(
            "đối chiếu lần 2        : settled {}, returned {}, ví không đổi = {}, bút toán đảo = {}",
            r.second.settled.len(),
            r.second.returned.len(),
            r.wallets.0 == r.wallets.1,
            r.reversal_entries
        );
        if verify {
            r.verify()?;
        }
        Ok(())
    })
    .await?;

    with_pg(async move |lab| {
        let r = idempotency(&lab).await?;
        println!("== 7. Idempotency theo mã lô và tài xế ==");
        println!(
            "hai lệnh tạo lô cùng mã chạy đồng thời: fresh = {}, cùng lô = {}, cùng khoản = {}",
            r.fresh_count, r.same_batch, r.same_items
        );
        println!(
            "khoản chi / bút toán payout:* : {} / {}",
            r.payouts_and_entries.0, r.payouts_and_entries.1
        );
        println!(
            "chèn lại bút toán cùng key    : tạo bản mới = {}",
            r.duplicate_entry_inserted
        );
        println!(
            "hai lượt gửi đồng thời        : tổng SENT = {}",
            r.concurrent_sent
        );
        println!("lượt gửi thứ ba               : {:?}", r.third_send);
        let orders: Vec<String> = r
            .bank_orders
            .iter()
            .map(|(d, n)| format!("{d}={n}"))
            .collect();
        println!("lệnh ngân hàng đã nhận        : {}", orders.join(" "));
        if verify {
            r.verify()?;
        }
        Ok(())
    })
    .await?;

    if verify {
        println!("VERIFY OK");
    }
    Ok(())
}

#[tokio::main]
async fn main() -> ExitCode {
    let verify = std::env::args().any(|a| a == "--verify");
    match run(verify).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("LỖI: {err:#}");
            ExitCode::FAILURE
        }
    }
}
