//! Lab bài 41: chart of accounts, dòng tiền chuyến và giao hàng, metric đúng/sai bằng SQL thật.
//!
//! `cargo run` in kết quả từng bước; `cargo run -- --verify` kiểm thêm các giá trị kỳ vọng
//! và thoát mã 1 nếu sai. Mọi DDL nằm trong schema `lab_<uuid>` và bị xoá cuối ca.
//! Thời gian nghiệp vụ là chuỗi ISO cố định ở đầu file, không đọc đồng hồ máy.

use std::process::ExitCode;

use anyhow::{Result, ensure};
use ledger_41_chart_of_accounts::chart::{self, AccountKind, READINGS};
use ledger_41_chart_of_accounts::db::with_lab;
use ledger_41_chart_of_accounts::flows::{self, CLEARING};
use ledger_41_chart_of_accounts::metrics::{self, Policy, Report};
use ledger_41_chart_of_accounts::store::{self, PostOutcome, display_id};
use ledger_core::{AccountId, JournalEntry, LedgerError, Money, fare_for_km};
use sqlx::PgPool;

// Đồng hồ nghiệp vụ cố định: kết quả không phụ thuộc giờ máy chạy.
const AS_OF: &str = "2026-11-06T00:00:00Z";
const TN1_CAPTURE: &str = "2026-11-05T01:30:00Z";
const TN1_SPLIT: &str = "2026-11-05T01:31:00Z";
const DL1_CAPTURE: &str = "2026-11-05T05:10:00Z";
const DL1_SPLIT: &str = "2026-11-05T05:11:00Z";
const TN2_CAPTURE: &str = "2026-11-05T09:00:00Z";
const TN3_CAPTURE: &str = "2026-10-20T03:00:00Z";
const TN4_CAPTURE: &str = "2026-11-05T10:00:00Z";
const ADJUST_AT: &str = "2026-11-05T11:00:00Z";
const TOPUP_AT: &str = "2026-10-20T03:00:00Z";

/// Clearing còn tiền quá 3 ngày thì báo.
const CLEARING_MAX_DAYS: i32 = 3;

/// Kiểm một điều kiện ở chế độ `--verify`; chế độ thường chỉ in.
fn expect(verify: bool, ok: bool, what: &str) -> Result<()> {
    if verify {
        ensure!(ok, "VERIFY FAIL: {what}");
    }
    Ok(())
}

fn dong(n: i64) -> String {
    Money::from_dong(n).to_string()
}

fn signed(n: i64) -> String {
    if n > 0 {
        format!("+{}", dong(n))
    } else {
        dong(n)
    }
}

/// Ghi một bút toán hợp lệ, in các dòng của nó và trả mã.
async fn post_and_show(pool: &PgPool, entry: &JournalEntry, at: &str) -> Result<i64> {
    let PostOutcome::Created { entry_id } = store::post(pool, entry, at).await? else {
        anyhow::bail!("bút toán {} đã có sẵn", entry.idempotency_key());
    };
    println!("{}  {}", display_id(entry_id), entry.idempotency_key());
    for p in entry.postings() {
        println!("  {:<22}{:>12}", p.account, signed(p.amount.dong()));
    }
    Ok(entry_id)
}

async fn show_balances(pool: &PgPool) -> Result<Vec<store::Balance>> {
    let balances = store::balances(pool).await?;
    for b in &balances {
        println!(
            "  {:<22}{:<10}{:>12}",
            b.account,
            b.kind.as_str(),
            dong(b.dong)
        );
    }
    let total: i64 = balances.iter().map(|b| b.dong).sum();
    let by_kind: Vec<(AccountKind, i64)> = balances.iter().map(|b| (b.kind, b.dong)).collect();
    let (debit, credit) = chart::trial_totals(&by_kind);
    println!(
        "  tổng toàn sổ {}; đọc theo loại: bên nợ {}, bên có {}",
        dong(total),
        dong(debit),
        dong(credit)
    );
    Ok(balances)
}

fn balance_of(balances: &[store::Balance], account: &str) -> i64 {
    balances
        .iter()
        .find(|b| b.account == account)
        .map_or(0, |b| b.dong)
}

fn print_report(label: &str, r: &Report) {
    let row = |name: &str, value: String| println!("  {name:<30}{value}");
    println!("{label}");
    row("bút toán không cân:", r.unbalanced.len().to_string());
    for u in &r.unbalanced {
        println!(
            "    {}  tổng {}, {} dòng",
            u.key,
            dong(u.total_dong),
            u.lines
        );
    }
    row("tổng toàn sổ (postings):", dong(r.postings_total));
    row("tổng toàn sổ (projection):", dong(r.projection_total));
    row(
        "số dư âm bất thường:",
        r.abnormal_negative.len().to_string(),
    );
    for a in &r.abnormal_negative {
        println!("    {} ({})  {}", a.account, a.kind, dong(a.balance_dong));
    }
    row(
        &format!("clearing tồn đọng > {CLEARING_MAX_DAYS} ngày:"),
        r.stuck_clearing.len().to_string(),
    );
    for s in &r.stuck_clearing {
        println!(
            "    {} {}  {}, {} ngày",
            s.account,
            s.reference,
            dong(s.stuck_dong),
            s.age_days
        );
    }
}

async fn run(verify: bool) -> Result<()> {
    with_lab(move |pool| async move { scenario(verify, pool).await }).await?;
    with_lab(move |pool| async move { topup_model(verify, pool).await }).await
}

/// Sổ riêng, mô hình nạp ví: tài khoản biên chuyển từ ví khách sang clearing.
async fn topup_model(verify: bool, pool: PgPool) -> Result<()> {
    println!("\n== 10. Sổ riêng, mô hình nạp ví: clearing là tài khoản biên");
    let wallet = "customer:CU-001";
    let fare = fare_for_km(5);
    let steps = [
        (
            "nạp ví 40.000",
            flows::wallet_topup("CU-001", Money::from_dong(40_000))?,
            TOPUP_AT,
            (40_000, -40_000),
        ),
        (
            "capture 25.500",
            flows::trip_capture("TN-0001", "CU-001", fare)?,
            TN1_CAPTURE,
            (14_500, -14_500),
        ),
        (
            "chia 25.500",
            flows::trip_split("TN-0001", "DR-001", fare)?,
            TN1_SPLIT,
            (14_500, -40_000),
        ),
    ];
    for (label, entry, at, expected) in &steps {
        store::post(&pool, entry, at).await?;
        let now = store::balances(&pool).await?;
        let seen = (balance_of(&now, wallet), balance_of(&now, CLEARING));
        println!(
            "  sau {label:<15} ví {:>11}, clearing {:>12}",
            dong(seen.0),
            dong(seen.1)
        );
        expect(verify, seen == *expected, label)?;
    }

    let last = store::balances(&pool).await?;
    let by_table: Vec<(AccountKind, i64)> = last.iter().map(|b| (b.kind, b.dong)).collect();
    let (debit, credit) = chart::trial_totals(&by_table);
    // Đọc độ lớn số âm của tài khoản biên là tài sản: hai phía về bằng nhau.
    let flipped: Vec<(AccountKind, i64)> = last
        .iter()
        .map(|b| {
            (
                b.kind,
                if b.account == CLEARING {
                    -b.dong
                } else {
                    b.dong
                },
            )
        })
        .collect();
    let (flip_debit, flip_credit) = chart::trial_totals(&flipped);
    println!(
        "  đọc theo bảng: bên nợ {}, bên có {}; đổi dấu clearing: {} và {}",
        dong(debit),
        dong(credit),
        dong(flip_debit),
        dong(flip_credit)
    );
    expect(verify, (debit, credit) == (0, 80_000), "đọc theo bảng lệch")?;
    expect(
        verify,
        (flip_debit, flip_credit) == (40_000, 40_000),
        "đổi dấu clearing thì hai phía bằng nhau",
    )?;

    let report = metrics::measure(
        &pool,
        &Policy {
            as_of: AS_OF,
            clearing_max_days: CLEARING_MAX_DAYS,
            customer_may_be_negative: false,
        },
    )
    .await?;
    print_report("Báo cáo (metric của bài, chưa đổi):", &report);
    expect(
        verify,
        report.unbalanced.is_empty() && report.postings_total == 0 && report.projection_total == 0,
        "sổ vẫn cân",
    )?;
    expect(
        verify,
        report.abnormal_negative.len() == 1 && report.abnormal_negative[0].account == CLEARING,
        "metric số âm báo nhầm clearing",
    )?;
    expect(
        verify,
        report.stuck_clearing.len() == 1 && report.stuck_clearing[0].reference == "fixture:topup",
        "metric tồn đọng báo nhầm khoản nạp",
    )?;
    Ok(())
}

#[allow(clippy::too_many_lines)]
async fn scenario(verify: bool, pool: PgPool) -> Result<()> {
    // 1. Sơ đồ tài khoản.
    println!("== 1. Sơ đồ tài khoản: tám mẫu, bốn loại");
    let samples = [
        "customer:CU-001",
        "driver:DR-001",
        "platform:revenue",
        "platform:promo-expense",
        "promo:budget:SUMMER",
        "psp:vixanh:clearing",
        "bank:payout:clearing",
        "tax:vat:payable",
        "wallet:CU-001",
    ];
    let mut known = 0;
    for raw in samples {
        match chart::kind_of(&AccountId::new(raw)?) {
            Some(kind) => {
                known += 1;
                println!("  {raw:<24}{}", kind.as_str());
            }
            None => println!("  {raw:<24}(ngoài sơ đồ, không mở được)"),
        }
    }
    expect(
        verify,
        known == 8,
        "tám mẫu của sơ đồ đều có loại, mẫu thứ chín thì không",
    )?;

    // 2. Cách đọc số dư theo loại.
    println!("\n== 2. Đọc số dư theo loại (dương = tiền vào ròng)");
    for r in &READINGS {
        println!(
            "  {:<10}dương: {}; âm: {}; thường ở {}",
            r.kind.as_str(),
            r.positive,
            r.negative,
            r.normal_side
        );
    }

    // 3. TN-0001 qua clearing.
    println!("\n== 3. TN-0001 qua clearing: 25.500 = 5.100 + 20.400");
    let fare = fare_for_km(5);
    let capture = flows::trip_capture("TN-0001", "CU-001", fare)?;
    let split = flows::trip_split("TN-0001", "DR-001", fare)?;
    let capture_id = post_and_show(&pool, &capture, TN1_CAPTURE).await?;
    println!("Số dư sau bước 1:");
    let after_one = show_balances(&pool).await?;
    post_and_show(&pool, &split, TN1_SPLIT).await?;
    println!("Số dư sau bước 2:");
    let after_two = show_balances(&pool).await?;
    expect(verify, capture_id == 1, "capture của TN-0001 là JE-000001")?;
    expect(
        verify,
        balance_of(&after_one, CLEARING) == 25_500,
        "sau bước 1 clearing giữ 25.500",
    )?;
    expect(
        verify,
        balance_of(&after_one, "customer:CU-001") == -25_500,
        "sau bước 1 khách -25.500",
    )?;
    expect(
        verify,
        balance_of(&after_two, CLEARING) == 0,
        "sau bước 2 clearing về 0",
    )?;
    expect(
        verify,
        balance_of(&after_two, "platform:revenue") == 5_100,
        "doanh thu 5.100",
    )?;
    expect(
        verify,
        balance_of(&after_two, "driver:DR-001") == 20_400,
        "tài xế 20.400",
    )?;
    let pair = |b: &[store::Balance]| {
        chart::trial_totals(&b.iter().map(|x| (x.kind, x.dong)).collect::<Vec<_>>())
    };
    expect(
        verify,
        pair(&after_one) == (51_000, 0),
        "đọc theo loại sau bước 1: nợ 51.000, có 0",
    )?;
    expect(
        verify,
        pair(&after_two) == (25_500, 25_500),
        "đọc theo loại sau bước 2: 25.500 và 25.500",
    )?;

    // 4. DL-0001: giao hàng, có thuế giả định.
    println!("\n== 4. DL-0001: phí giao 33.000, phí nền tảng 20%, thuế giả định nằm trong phí");
    let delivery_fee = Money::from_dong(33_000);
    post_and_show(
        &pool,
        &flows::delivery_capture("DL-0001", "CU-001", delivery_fee)?,
        DL1_CAPTURE,
    )
    .await?;
    post_and_show(
        &pool,
        &flows::delivery_split("DL-0001", "DR-002", delivery_fee)?,
        DL1_SPLIT,
    )
    .await?;
    let after_delivery = store::balances(&pool).await?;
    println!(
        "  shipper driver:DR-002 {}, tax:vat:payable {}, platform:revenue {}",
        dong(balance_of(&after_delivery, "driver:DR-002")),
        dong(balance_of(&after_delivery, flows::VAT)),
        dong(balance_of(&after_delivery, flows::REVENUE)),
    );
    expect(
        verify,
        balance_of(&after_delivery, "driver:DR-002") == 26_400,
        "shipper 26.400",
    )?;
    expect(
        verify,
        balance_of(&after_delivery, flows::VAT) == 600,
        "thuế giả định 600",
    )?;
    expect(
        verify,
        balance_of(&after_delivery, flows::REVENUE) == 5_100 + 6_000,
        "doanh thu 5.100 + 6.000",
    )?;
    expect(
        verify,
        balance_of(&after_delivery, CLEARING) == 0,
        "clearing về 0 sau cả hai luồng",
    )?;

    // 5. Hai dạng không sống chung một sổ.
    println!("\n== 5. Dạng ba dòng cùng key không sống chung với dạng hai bước");
    let again = store::post(&pool, &capture, TN1_CAPTURE).await?;
    println!("  ghi lại capture hai bước: {again:?}");
    expect(
        verify,
        again
            == PostOutcome::Replayed {
                entry_id: capture_id,
            },
        "ghi lại cùng nội dung là Replayed",
    )?;
    let legacy = flows::legacy_three_line("TN-0001", "CU-001", "DR-001", fare)?;
    let conflict = store::post(&pool, &legacy, TN1_CAPTURE).await.unwrap_err();
    println!("  ghi dạng ba dòng cùng key: {conflict}");
    let is_conflict = matches!(
        conflict.downcast_ref::<LedgerError>(),
        Some(LedgerError::IdempotencyConflict { key }) if key == "trip:TN-0001:capture"
    );
    expect(
        verify,
        is_conflict,
        "dạng ba dòng cùng key là IdempotencyConflict",
    )?;

    // 6. DB từ chối bút toán lệch khi trigger bật.
    println!("\n== 6. Trigger kiểm cân của DB từ chối bút toán lệch (đi vòng qua kiểu)");
    let lopsided = [("customer:CU-001", -25_500), (CLEARING, 25_000)];
    let rejected = store::post_raw(&pool, "demo:unbalanced", TN4_CAPTURE, &lopsided)
        .await
        .unwrap_err();
    let db_err = rejected
        .downcast_ref::<sqlx::Error>()
        .and_then(|e| e.as_database_error());
    let code = db_err
        .and_then(|e| e.code())
        .map(|c| c.to_string())
        .unwrap_or_default();
    let message = db_err.map(|e| e.message().to_string()).unwrap_or_default();
    println!("  SQLSTATE {code}: {message}");
    expect(
        verify,
        code == "23514" && message.contains("lệch"),
        "COMMIT bị từ chối với SQLSTATE 23514",
    )?;

    // 7. Metric trên sổ khỏe. TN-0002 mới capture, chưa chia: clearing giữ tiền nhưng chưa quá hạn.
    println!("\n== 7. Metric trên sổ khỏe (đo lúc {AS_OF})");
    let fare2 = fare_for_km(3);
    store::post(
        &pool,
        &flows::trip_capture("TN-0002", "CU-002", fare2)?,
        TN2_CAPTURE,
    )
    .await?;
    let lenient = Policy {
        as_of: AS_OF,
        clearing_max_days: CLEARING_MAX_DAYS,
        customer_may_be_negative: true,
    };
    let healthy = metrics::measure(&pool, &lenient).await?;
    print_report("Báo cáo:", &healthy);
    let clearing_now = balance_of(&store::balances(&pool).await?, CLEARING);
    println!(
        "  clearing đang giữ {} của TN-0002 (mới 15 giờ, chưa quá hạn)",
        dong(clearing_now)
    );
    expect(verify, healthy.is_healthy(), "sổ khỏe: mọi metric sạch")?;
    expect(
        verify,
        clearing_now == 16_500,
        "clearing giữ 16.500 của TN-0002",
    )?;

    // 8. Cài ba sự cố, tắt trigger để dữ liệu lệch lọt vào, rồi đo lại.
    println!("\n== 8. Cài sự cố rồi đo lại");
    store::post(
        &pool,
        &flows::trip_capture("TN-0003", "CU-003", fare_for_km(4))?,
        TN3_CAPTURE,
    )
    .await?;
    store::set_balance_triggers(&pool, false).await?;
    store::post_raw(&pool, "trip:TN-0004:capture", TN4_CAPTURE, &lopsided).await?;
    store::set_balance_triggers(&pool, true).await?;
    let adjust = ledger_core::JournalEntryBuilder::new("adjust:DR-009:manual-1")
        .posting(AccountId::new("driver:DR-009")?, Money::from_dong(-5_000))
        .posting(AccountId::new(flows::REVENUE)?, Money::from_dong(5_000))
        .build()?;
    store::post(&pool, &adjust, ADJUST_AT).await?;
    println!(
        "  đã cài: TN-0003 capture từ 20/10 chưa chia; TN-0004 lệch -500; DR-009 bị trừ khi chưa có tiền"
    );
    println!("  trigger đã bật lại, nhưng không kiểm lại dữ liệu cũ");
    let broken = metrics::measure(&pool, &lenient).await?;
    print_report("Báo cáo (ví khách được phép âm):", &broken);
    expect(
        verify,
        broken.unbalanced.len() == 1 && broken.unbalanced[0].key == "trip:TN-0004:capture",
        "một bút toán không cân",
    )?;
    expect(
        verify,
        broken.unbalanced[0].total_dong == -500 && broken.unbalanced[0].lines == 2,
        "lệch -500 trên 2 dòng",
    )?;
    expect(
        verify,
        broken.postings_total == -500 && broken.projection_total == -500,
        "tổng toàn sổ -500 ở cả hai cách đo",
    )?;
    expect(
        verify,
        broken.abnormal_negative.len() == 1
            && broken.abnormal_negative[0].account == "driver:DR-009",
        "một số dư âm bất thường: driver:DR-009",
    )?;
    expect(
        verify,
        broken.stuck_clearing.len() == 1
            && broken.stuck_clearing[0].reference == "trip:TN-0003"
            && broken.stuck_clearing[0].stuck_dong == 21_000
            && broken.stuck_clearing[0].age_days == 16,
        "clearing tồn đọng: trip:TN-0003, 21.000, 16 ngày",
    )?;

    // 9. Cùng dữ liệu, bỏ ngoại lệ của ví khách.
    let strict = Policy {
        customer_may_be_negative: false,
        ..lenient
    };
    let strict_report = metrics::measure(&pool, &strict).await?;
    println!("\n== 9. Cùng sổ, nhưng ví khách KHÔNG được âm (mô hình có nạp ví)");
    for a in &strict_report.abnormal_negative {
        println!("  {} ({})  {}", a.account, a.kind, dong(a.balance_dong));
    }
    expect(
        verify,
        strict_report.abnormal_negative.len() == 4,
        "bỏ ngoại lệ thì thêm ba ví khách",
    )?;
    Ok(())
}

#[tokio::main]
async fn main() -> ExitCode {
    let verify = std::env::args().any(|a| a == "--verify");
    match run(verify).await {
        Ok(()) => {
            if verify {
                println!("\nVERIFY OK");
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{e:#}");
            ExitCode::FAILURE
        }
    }
}
