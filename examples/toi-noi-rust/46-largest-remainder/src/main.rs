//! Binary crate: chạy từng bước của bài 46 và, với `--verify`, kiểm các giá trị in trong bài.

use std::process::ExitCode;

use ledger_core::{
    AccountId, FeePolicy, HalfAwayFromZero, Ledger, LedgerError, Money, PercentageFee, TripCharge,
    capture_entry,
};
use ledger_lab::allocate::total_of;
use ledger_lab::book::{AllocatedFee, AllocationRecord, Rule, distribute_entry};
use ledger_lab::naive::{drift, round_each};
use ledger_lab::search::{
    alabama_counterexample, check_allocation_invariants, hashmap_tiebreak_winners,
    naive_minimal_counterexample, naive_raw_counterexample,
};
use ledger_lab::strategies::{ALL, round_dong};
use ledger_lab::{AllocError, Party, Share, allocate, allocate_with_plug};
use rust_decimal::{Decimal, RoundingStrategy};

/// Gom lỗi `--verify`; ở chế độ thường thì in kết quả.
struct Ctx {
    verify: bool,
    failures: Vec<String>,
}

impl Ctx {
    fn say(&self, text: &str) {
        if !self.verify {
            println!("{text}");
        }
    }

    fn check(&mut self, ok: bool, what: &str) {
        if !ok {
            self.failures.push(what.to_string());
        }
    }
}

fn dongs(shares: &[Share]) -> Vec<i64> {
    shares.iter().map(|s| s.amount.dong()).collect()
}

fn fmt_list(values: &[i64]) -> String {
    let texts: Vec<String> = values
        .iter()
        .map(|v| Money::from_dong(*v).to_string())
        .collect();
    texts.join(", ")
}

/// Số lệch có dấu; 0 viết là «0».
fn signed(value: i128) -> String {
    if value == 0 {
        "0".to_string()
    } else {
        format!("{value:+}")
    }
}

fn parties(spec: &[(&str, u64)]) -> Vec<Party> {
    spec.iter().map(|&(k, w)| Party::new(k, w)).collect()
}

/// Mục 1: làm tròn từng phần riêng lẻ.
fn naive_rounding(ctx: &mut Ctx) {
    ctx.say("== 1. Làm tròn từng phần riêng lẻ (nửa ra xa 0) ==");
    let cases: [(i64, &[u32], i128); 4] = [
        (100, &[1, 1, 1], -1),
        (25_501, &[80, 20], 0),
        (25_501, &[40, 40, 20], -1),
        (25_501, &[50, 50], 1),
    ];
    for (total, weights, expected_drift) in cases {
        let parts = round_each(total, weights, RoundingStrategy::MidpointAwayFromZero)
            .expect("đầu vào hợp lệ");
        let d = drift(total, &parts);
        ctx.say(&format!(
            "chia {} theo {weights:?}: {} | lệch {}",
            Money::from_dong(total),
            fmt_list(&parts),
            signed(d)
        ));
        ctx.check(d == expected_drift, "lệch của làm tròn từng phần");
    }
    // Ngay cả Decimal cũng không cứu được: 1/3 không biểu diễn hết được, ba phần ba chưa đủ một.
    let third = Decimal::ONE / Decimal::from(3);
    let sum = third + third + third;
    ctx.say(&format!("Decimal: 1/3 = {third}, ba lần 1/3 = {sum}"));
    ctx.check(sum != Decimal::ONE, "Decimal cũng lệch");
}

/// Mục 2: bảng các `RoundingStrategy`.
fn strategy_table(ctx: &mut Ctx) {
    ctx.say("\n== 2. RoundingStrategy của rust_decimal ==");
    let samples =
        ["2.4", "2.5", "3.5", "-2.5", "-2.6"].map(|s| s.parse::<Decimal>().expect("số hợp lệ"));
    ctx.say(&format!(
        "{:<22}{:>5}{:>5}{:>5}{:>6}{:>6} | lệch tổng: 100/[1,1,1], 25.501/[80,20], 25.501/[50,50]",
        "strategy", "2.4", "2.5", "3.5", "-2.5", "-2.6"
    ));
    for (name, strategy) in ALL {
        let row: Vec<i64> = samples
            .iter()
            .map(|&d| round_dong(d, strategy).expect("vừa i64"))
            .collect();
        let d1 = drift(100, &round_each(100, &[1, 1, 1], strategy).expect("hợp lệ"));
        let d2 = drift(
            25_501,
            &round_each(25_501, &[80, 20], strategy).expect("hợp lệ"),
        );
        let d3 = drift(
            25_501,
            &round_each(25_501, &[50, 50], strategy).expect("hợp lệ"),
        );
        ctx.say(&format!(
            "{name:<22}{:>5}{:>5}{:>5}{:>6}{:>6} | {}, {}, {}",
            row[0],
            row[1],
            row[2],
            row[3],
            row[4],
            signed(d1),
            signed(d2),
            signed(d3)
        ));
        if name == "MidpointNearestEven" {
            ctx.check(row == vec![2, 2, 4, -2, -3], "NearestEven");
            ctx.check((d1, d2, d3) == (-1, 0, -1), "drift NearestEven");
        }
        if name == "MidpointAwayFromZero" {
            ctx.check(row == vec![2, 3, 4, -3, -3], "AwayFromZero giữa");
        }
        if name == "ToZero" {
            ctx.check((d1, d2, d3) == (-1, -1, -1), "drift ToZero");
        }
        if name == "AwayFromZero" {
            ctx.check((d1, d2, d3) == (2, 1, 1), "drift AwayFromZero");
        }
    }
}

/// Mục 3: largest remainder chuẩn.
fn largest_remainder(ctx: &mut Ctx) -> Result<(), AllocError> {
    ctx.say("\n== 3. Largest remainder ==");
    let fare = Money::from_dong(25_500);
    let canonical = allocate(
        fare,
        &parties(&[("platform:revenue", 20), ("driver:DR-001", 80)]),
    )?;
    ctx.say(&format!(
        "25.500 theo 20/80: {}",
        fmt_list(&dongs(&canonical))
    ));
    ctx.check(
        dongs(&canonical) == vec![5_100, 20_400],
        "25.500 theo 20/80",
    );

    for (total, spec) in [
        (100, vec![("a", 1), ("b", 1), ("c", 1)]),
        (25_501, vec![("a", 40), ("b", 40), ("c", 20)]),
        (25_501, vec![("a", 50), ("b", 50)]),
    ] {
        let shares = allocate(Money::from_dong(total), &parties(&spec))?;
        ctx.say(&format!(
            "{} theo {:?}: {}",
            Money::from_dong(total),
            spec.iter().map(|&(_, w)| w).collect::<Vec<_>>(),
            fmt_list(&dongs(&shares))
        ));
        ctx.check(total_of(&shares) == i128::from(total), "tổng khớp");
    }

    // Tất định: đảo thứ tự đầu vào, phần của từng key không đổi.
    let forward = allocate(
        Money::from_dong(100),
        &parties(&[("a", 1), ("b", 1), ("c", 1)]),
    )?;
    let backward = allocate(
        Money::from_dong(100),
        &parties(&[("c", 1), ("b", 1), ("a", 1)]),
    )?;
    ctx.say(&format!(
        "100 chia ba, thứ tự a,b,c: {} | thứ tự c,b,a: {}",
        fmt_list(&dongs(&forward)),
        fmt_list(&dongs(&backward))
    ));
    ctx.check(dongs(&forward) == vec![34, 33, 33], "a,b,c");
    ctx.check(
        dongs(&backward) == vec![33, 33, 34],
        "c,b,a: key a vẫn nhận 34",
    );

    // Phá hòa bằng HashMap thì mỗi lần một khác.
    let winners = hashmap_tiebreak_winners(200);
    ctx.say(&format!(
        "phá hòa theo HashMap, 200 lần thử: có hơn một bên nhận đồng lẻ = {}",
        winners.len() > 1
    ));
    ctx.check(winners.len() > 1, "HashMap không tất định");

    // Trọng số bằng 0, tổng trọng số bằng 0, rỗng, trùng key.
    let zero = allocate(
        Money::from_dong(10),
        &parties(&[("a", 1), ("b", 0), ("c", 1), ("d", 1)]),
    )?;
    ctx.say(&format!(
        "10 theo [1, 0, 1, 1]: {}",
        fmt_list(&dongs(&zero))
    ));
    ctx.check(dongs(&zero) == vec![4, 0, 3, 3], "trọng số 0");
    let errors = [
        allocate(fare, &parties(&[("a", 0), ("b", 0)])),
        allocate(fare, &[]),
        allocate(fare, &parties(&[("a", 1), ("a", 1)])),
    ];
    for e in &errors {
        ctx.say(&format!("lỗi: {}", e.as_ref().expect_err("phải lỗi")));
    }
    ctx.check(
        errors[0] == Err(AllocError::ZeroWeightSum),
        "tổng trọng số 0",
    );
    ctx.check(errors[1] == Err(AllocError::NoParties), "rỗng");

    // Số âm (hoàn tiền): đối xứng từng đồng.
    let three = parties(&[("a", 1), ("b", 1), ("c", 1)]);
    let paid = allocate(Money::from_dong(25_501), &three)?;
    let refund = allocate(Money::from_dong(-25_501), &three)?;
    ctx.say(&format!(
        "25.501 chia ba: {} | hoàn: {}",
        fmt_list(&dongs(&paid)),
        fmt_list(&dongs(&refund))
    ));
    ctx.check(dongs(&paid) == vec![8_501, 8_500, 8_500], "chia ba 25.501");
    ctx.check(dongs(&refund) == vec![-8_501, -8_500, -8_500], "hoàn");

    // Tràn i64: nhân bằng i64 thất bại, bằng i128 thì không.
    let weight: u64 = 1_000_000_000_000_000;
    let in_i64 = 25_500i64.checked_mul(i64::try_from(weight).expect("vừa i64"));
    let in_i128 = i128::from(25_500) * i128::from(weight);
    ctx.say(&format!(
        "25.500 × {weight}: i64 = {in_i64:?}, i128 = {in_i128}"
    ));
    ctx.check(
        in_i64.is_none() && in_i128 == 25_500_000_000_000_000_000,
        "i64 tràn, i128 không",
    );
    let extreme = allocate(
        Money::from_dong(i64::MIN),
        &parties(&[("a", u64::MAX), ("b", u64::MAX)]),
    )?;
    ctx.say(&format!(
        "i64::MIN theo hai trọng số u64::MAX: tổng = {} (đúng bằng i64::MIN: {})",
        total_of(&extreme),
        total_of(&extreme) == i128::from(i64::MIN)
    ));
    ctx.check(total_of(&extreme) == i128::from(i64::MIN), "i64::MIN");
    Ok(())
}

/// Mục 4: so với «bên cuối nhận phần còn lại».
fn plug_rule(ctx: &mut Ctx) -> Result<(), AllocError> {
    ctx.say("\n== 4. Ai nhận đồng lẻ? ==");
    let four = parties(&[("a", 1), ("b", 1), ("c", 1), ("d", 1)]);
    let plugged = allocate_with_plug(Money::from_dong(6), &four, "d")?;
    let fair = allocate(Money::from_dong(6), &four)?;
    ctx.say(&format!(
        "6 đồng cho 4 bên bằng nhau: phần còn lại cho d = {} | largest remainder = {}",
        fmt_list(&dongs(&plugged)),
        fmt_list(&dongs(&fair))
    ));
    ctx.check(dongs(&plugged) == vec![2, 2, 2, 0], "plug");
    ctx.check(dongs(&fair) == vec![2, 2, 1, 1], "largest remainder 6/4");
    Ok(())
}

/// Mục 5: nối với `ledger-core`.
fn ledger_link(ctx: &mut Ctx) -> Result<(), Box<dyn std::error::Error>> {
    ctx.say("\n== 5. Nối với sổ cái ==");
    let mut ledger = Ledger::new();

    // Bước chia của TN-0001: clearing -25.500, doanh thu +5.100, tài xế +20.400.
    let split_parties = parties(&[("platform:revenue", 20), ("driver:DR-001", 80)]);
    let shares = allocate(Money::from_dong(25_500), &split_parties)?;
    let entry = distribute_entry("trip:TN-0001:split", "psp:vixanh:clearing", &shares)?;
    let amounts: Vec<i64> = entry.postings().iter().map(|p| p.amount.dong()).collect();
    ctx.say(&format!("trip:TN-0001:split → {amounts:?}"));
    ctx.check(
        amounts == vec![-25_500, 5_100, 20_400],
        "bút toán split TN-0001",
    );
    ledger.post(entry)?;

    // Chuyến ghép TN-0012: ba khách chia cước 25.501.
    let riders = parties(&[
        ("customer:CU-001", 1),
        ("customer:CU-002", 1),
        ("customer:CU-003", 1),
    ]);
    let fare = Money::from_dong(-25_501); // âm: tiền ra khỏi ví khách
    let rule = Rule::LargestRemainder;
    let shares = rule.apply(fare, &riders)?;
    let record = AllocationRecord::new(&rule, fare, &riders);
    ctx.say(&format!("bản ghi quy tắc: {record}"));
    ctx.check(
        record.to_string()
            == "rule=largest-remainder/v1 tie=key-asc total=-25501 weights=customer:CU-001:1,customer:CU-002:1,customer:CU-003:1",
        "bản ghi quy tắc",
    );
    let capture = distribute_entry("trip:TN-0012:capture", "psp:vixanh:clearing", &shares)?;
    let posted = ledger.post(capture.clone())?;
    let id = posted.id;
    ctx.say(&format!("{id}: {}", fmt_list(&dongs(&shares))));
    ctx.check(dongs(&shares) == vec![-8_501, -8_500, -8_500], "ba khách");

    // Ghi lại cùng quyết định: replay, không thêm bút toán.
    let again = ledger.post(capture)?;
    ctx.say(&format!(
        "ghi lại cùng quy tắc: replayed = {}",
        again.replayed
    ));
    ctx.check(again.replayed, "replay");

    // Quy tắc khác cho cùng key: nội dung đổi, sổ từ chối.
    let plug = Rule::Plug("customer:CU-003".to_string());
    let other = distribute_entry(
        "trip:TN-0012:capture",
        "psp:vixanh:clearing",
        &plug.apply(fare, &riders)?,
    )?;
    let conflict = ledger.post(other).expect_err("khác nội dung");
    ctx.say(&format!("đổi quy tắc, cùng key: {conflict}"));
    ctx.check(
        conflict
            == LedgerError::IdempotencyConflict {
                key: "trip:TN-0012:capture".to_string(),
            },
        "xung đột idempotency",
    );

    // Hoàn toàn bộ: allocate(-t) trùng bút toán đảo của sổ cái.
    let reversed = ledger.reverse(id)?.entry.clone();
    let refund = allocate(Money::from_dong(25_501), &riders)?;
    let refund_entry = distribute_entry("trip:TN-0012:refund:1", "psp:vixanh:clearing", &refund)?;
    let same = refund_entry
        .postings()
        .iter()
        .zip(reversed.postings())
        .all(|(a, b)| a.amount == b.amount);
    ctx.say(&format!(
        "hoàn toàn bộ trùng bút toán đảo từng posting = {same}"
    ));
    ctx.check(same, "hoàn trùng bút toán đảo");
    ctx.check(ledger.grand_total()? == Money::ZERO, "tổng toàn sổ bằng 0");

    // FeePolicy: cùng giao diện, quyết định đồng lẻ khác nhau.
    let trip = TripCharge {
        trip_id: "TN-0003".to_string(),
        customer: AccountId::new("customer:CU-001")?,
        driver: AccountId::new("driver:DR-001")?,
        fare: Money::from_dong(25_501),
    };
    let half_away = PercentageFee::new(Decimal::new(50, 2), HalfAwayFromZero);
    let by_remainder = AllocatedFee::new(5_000).expect("trong 0..=10000");
    for policy in [&half_away as &dyn FeePolicy, &by_remainder] {
        let entry = capture_entry(&trip, policy)?;
        let amounts: Vec<i64> = entry.postings().iter().map(|p| p.amount.dong()).collect();
        ctx.say(&format!("{:<44} → {amounts:?}", policy.name()));
    }
    let a = capture_entry(&trip, &half_away)?;
    let b = capture_entry(&trip, &by_remainder)?;
    let pa: Vec<i64> = a.postings().iter().map(|p| p.amount.dong()).collect();
    let pb: Vec<i64> = b.postings().iter().map(|p| p.amount.dong()).collect();
    ctx.check(pa == vec![-25_501, 12_751, 12_750], "PercentageFee 50% lẻ");
    ctx.check(pb == vec![-25_501, 12_750, 12_751], "AllocatedFee 50% lẻ");
    // Với 20% và cước 25.500 hai chính sách trùng nhau.
    let same_fee = AllocatedFee::new(2_000)
        .expect("hợp lệ")
        .fee(Money::from_dong(25_500))?;
    ctx.check(same_fee == Money::from_dong(5_100), "20% của 25.500");
    Ok(())
}

/// Mục 6: phản ví dụ và bất biến bằng proptest.
fn search_demo(ctx: &mut Ctx) {
    ctx.say("\n== 6. Tìm phản ví dụ ==");
    let raw = naive_raw_counterexample();
    let minimal = naive_minimal_counterexample();
    ctx.say(&format!("làm tròn từng phần, phản ví dụ thô: {raw:?}"));
    ctx.say(&format!("sau shrinking: {minimal:?}"));
    ctx.check(raw.is_some() && minimal.is_some(), "bản ngây thơ bị bắt");
    if let Some((total, weights)) = &minimal {
        let parts =
            round_each(*total, weights, RoundingStrategy::MidpointAwayFromZero).expect("hợp lệ");
        ctx.check(drift(*total, &parts) != 0, "phản ví dụ thật sự lệch");
    }
    // Phản ví dụ nhỏ hơn mà shrinking không tới được (tối ưu cục bộ).
    let tiny = round_each(1, &[1, 1], RoundingStrategy::MidpointAwayFromZero).expect("hợp lệ");
    ctx.say(&format!(
        "1 đồng cho [1, 1]: {tiny:?}, lệch {}",
        signed(drift(1, &tiny))
    ));
    ctx.check(tiny == vec![1, 1], "1 đồng cho hai bên");

    let alabama = alabama_counterexample(10, 3).expect("có ca nhỏ");
    ctx.say(&format!(
        "tăng tổng {} → {} theo trọng số {:?}: {:?} → {:?}",
        alabama.total,
        alabama.total + 1,
        alabama.weights,
        alabama.before,
        alabama.after
    ));
    ctx.check(
        alabama.total == 3
            && alabama.weights == [1, 3, 3]
            && alabama.before == vec![1, 1, 1]
            && alabama.after == vec![0, 2, 2],
        "Alabama",
    );

    let result = check_allocation_invariants(1_000);
    ctx.say(&format!(
        "1.000 ca (tổng khớp, mỗi phần lệch dưới 1 đồng): {}",
        if result.is_ok() { "đạt" } else { "FAIL" }
    ));
    ctx.check(result.is_ok(), "1.000 ca bất biến");
    if let Err(e) = result {
        eprintln!("  {e}");
    }
}

fn run(verify: bool) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut ctx = Ctx {
        verify,
        failures: Vec::new(),
    };
    naive_rounding(&mut ctx);
    strategy_table(&mut ctx);
    largest_remainder(&mut ctx)?;
    plug_rule(&mut ctx)?;
    ledger_link(&mut ctx)?;
    search_demo(&mut ctx);
    Ok(ctx.failures)
}

fn main() -> ExitCode {
    let verify = std::env::args().any(|arg| arg == "--verify");
    match run(verify) {
        Ok(failures) if failures.is_empty() => {
            if verify {
                println!("VERIFY OK");
            }
            ExitCode::SUCCESS
        }
        Ok(failures) => {
            for f in &failures {
                eprintln!("VERIFY FAIL: {f}");
            }
            ExitCode::FAILURE
        }
        Err(e) => {
            eprintln!("Lỗi: {e}");
            ExitCode::FAILURE
        }
    }
}
