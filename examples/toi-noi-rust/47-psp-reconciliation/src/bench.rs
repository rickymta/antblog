//! Sinh file sao kê lớn tại chỗ, kỳ vọng tính độc lập, đối chiếu SQL với Rust và đo.

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::time::Instant;

use anyhow::{Result, bail};
use jiff::Timestamp;
use jiff::civil::date;
use sqlx::PgPool;

use crate::kind::Kind;
use crate::lab::with_lab;
use crate::memory::MatchResult;
use crate::period::Period;
use crate::reconcile::{
    RunRequest, RunSummary, items_as_exceptions, load_items, match_in_memory, reconcile_file,
};
use crate::seed::{generated_amount, seed_generated_ledger};

fn at(period: &Period, offset_secs: i64) -> Timestamp {
    Timestamp::from_second(period.start.as_second() + offset_secs).expect("trong phạm vi jiff")
}

/// Giây lệch so với đầu kỳ của bút toán thứ `i`: công thức này cũng nằm trong `seed.rs` (SQL).
fn offset(i: i64) -> i64 {
    (i * 7) % 80_000
}

/// Số dòng và byte đã ghi.
#[derive(Debug, Clone, Copy)]
pub struct Generated {
    /// Số dòng dữ liệu (không tính tiêu đề), gồm cả dòng hỏng.
    pub lines: u64,
    /// Số byte của file.
    pub bytes: u64,
}

/// Ghi file sao kê cho `rows` ref, theo lớp `i % 1000` (xem [`seed_generated_ledger`]):
/// 0 chỉ có ở sao kê, 1 chỉ có ở sổ ta, 2 lệch 500 đồng, 3 dòng sao kê bị lặp, 4 tách hai dòng,
/// 5 ghi sau nửa đêm; mỗi 100.000 ref thêm một dòng hỏng. Ghi qua `BufWriter`, không dựng `Vec` nào.
///
/// # Errors
/// Lỗi ghi file.
pub fn write_generated_statement(path: &Path, rows: i64, period: &Period) -> Result<Generated> {
    let mut out = BufWriter::new(File::create(path)?);
    writeln!(out, "external_ref,amount_vnd,settled_at")?;
    let mut lines = 0_u64;
    for i in 1..=rows {
        let reference = format!("vx_ch_{i:08}");
        let amount = generated_amount(i);
        let psp_at = if i % 1000 == 5 {
            period.end.as_second() + 20 - period.start.as_second()
        } else {
            offset(i) + 30
        };
        let ts = at(period, psp_at);
        match i % 1000 {
            1 => {}
            3 => {
                writeln!(out, "{reference},{amount},{ts}")?;
                writeln!(out, "{reference},{amount},{ts}")?;
                lines += 2;
            }
            4 => {
                let first = amount / 2;
                writeln!(out, "{reference},{first},{ts}")?;
                writeln!(
                    out,
                    "{reference},{},{}",
                    amount - first,
                    at(period, psp_at + 1)
                )?;
                lines += 2;
            }
            _ => {
                writeln!(out, "{reference},{amount},{ts}")?;
                lines += 1;
            }
        }
        if i % 100_000 == 7 {
            writeln!(out, "{reference},12x00,{ts}")?;
            lines += 1;
        }
    }
    out.flush()?;
    let bytes = out.get_ref().metadata()?.len();
    Ok(Generated { lines, bytes })
}

/// Kết quả đúng phải ra, suy từ công thức sinh dữ liệu (không chạy đối soát).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Expected {
    /// Dòng sao kê hợp lệ.
    pub statement_rows: i64,
    /// Dòng hỏng.
    pub bad_lines: i64,
    /// Dòng sổ trong cửa sổ đọc.
    pub ledger_rows: i64,
    /// Ref khớp.
    pub matched: i64,
    /// Ref khớp có tách nhiều dòng.
    pub matched_split: i64,
    /// Thiếu bên ta.
    pub missing_in_ledger: i64,
    /// Thiếu bên PSP.
    pub missing_in_psp: i64,
    /// Lệch tiền.
    pub amount_mismatch: i64,
    /// Trùng.
    pub duplicate: i64,
    /// Khác ngày.
    pub date_shift: i64,
}

/// Kỳ vọng cho `rows` ref sinh bởi [`write_generated_statement`] và [`seed_generated_ledger`].
pub fn expected_counts(rows: i64) -> Expected {
    let mut e = Expected::default();
    for i in 1..=rows {
        let (stmt, ledger) = match i % 1000 {
            0 => {
                e.missing_in_ledger += 1;
                (1, 0)
            }
            1 => {
                e.missing_in_psp += 1;
                (0, 1)
            }
            2 => {
                e.amount_mismatch += 1;
                (1, 1)
            }
            3 => {
                e.duplicate += 1;
                (2, 1)
            }
            4 => {
                e.matched += 1;
                e.matched_split += 1;
                (2, 1)
            }
            5 => {
                e.date_shift += 1;
                (1, 1)
            }
            _ => {
                e.matched += 1;
                (1, 1)
            }
        };
        e.statement_rows += stmt;
        e.ledger_rows += ledger;
        if i % 100_000 == 7 {
            e.bad_lines += 1;
        }
    }
    e
}

/// So kết quả đã lưu với kỳ vọng.
///
/// # Errors
/// Có số nào khác kỳ vọng (thông báo nêu số nào).
pub fn check_summary(s: &RunSummary, e: &Expected) -> Result<()> {
    let n = |kind| s.exceptions.get(&kind).copied().unwrap_or(0);
    let pairs = [
        ("statement_rows", s.statement_rows, e.statement_rows),
        ("bad_lines", s.bad_lines, e.bad_lines),
        ("ledger_rows", s.ledger_rows, e.ledger_rows),
        ("matched", s.refs_matched, e.matched),
        ("matched_split", s.refs_matched_split, e.matched_split),
        (
            "missing_in_ledger",
            n(Kind::MissingInLedger),
            e.missing_in_ledger,
        ),
        ("missing_in_psp", n(Kind::MissingInPsp), e.missing_in_psp),
        (
            "amount_mismatch",
            n(Kind::AmountMismatch),
            e.amount_mismatch,
        ),
        ("duplicate", n(Kind::Duplicate), e.duplicate),
        ("date_shift", n(Kind::DateShift), e.date_shift),
    ];
    for (name, got, want) in pairs {
        if got != want {
            bail!("{name}: có {got}, kỳ vọng {want}");
        }
    }
    Ok(())
}

/// Chạy cùng dữ liệu bằng hai cách rồi so từng ngoại lệ.
#[derive(Debug)]
pub struct CrossCheck {
    /// Kết quả khớp bằng SQL (đã lưu).
    pub summary: RunSummary,
    /// Kết quả khớp trong Rust.
    pub memory: MatchResult,
    /// Hai danh sách ngoại lệ giống hệt nhau (loại, ref, hai tổng, hai số dòng).
    pub identical: bool,
}

/// Sinh `rows` ref vào `file`, nạp sổ cái vào `pool`, đối soát bằng SQL và bằng Rust.
///
/// # Errors
/// Lỗi ghi file, cơ sở dữ liệu hoặc đối soát.
pub async fn cross_check(pool: &PgPool, file: &Path, rows: i64) -> Result<CrossCheck> {
    let period = Period::business_day(date(2026, 11, 5))?;
    write_generated_statement(file, rows, &period)?;
    seed_generated_ledger(pool, rows, &period).await?;
    let req = RunRequest {
        provider: "vixanh",
        path: file,
        period,
        run_at: period.end,
    };
    let outcome = reconcile_file(pool, &req).await?;
    let summary = outcome.summary().clone();
    let items = load_items(pool, summary.run_id).await?;
    let (memory, _) = match_in_memory(pool, file, &period).await?;
    let identical = items_as_exceptions(&items) == memory.exceptions
        && summary.refs_matched == i64::try_from(memory.matched)?
        && summary.refs_matched_split == i64::try_from(memory.matched_split)?
        && summary.refs_out_of_period == i64::try_from(memory.out_of_period)?;
    Ok(CrossCheck {
        summary,
        memory,
        identical,
    })
}

/// Cách khớp được đo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// COPY vào bảng tạm rồi khớp bằng SQL (đường chính của lab).
    Sql,
    /// Nạp cả hai phía vào `HashMap` trong Rust.
    Rust,
}

fn ms(d: std::time::Duration) -> u128 {
    d.as_millis()
}

/// Đo một cách khớp trên file `path` có `rows` ref. Số đo in ra là của máy chạy lệnh, không phải SLA.
/// Bộ nhớ đo bên ngoài: `/usr/bin/time -l <binary> bench ...` (dòng `maximum resident set size`).
///
/// # Errors
/// Lỗi hạ tầng hoặc kết quả khác kỳ vọng.
pub async fn run_bench(mode: Mode, path: std::path::PathBuf, rows: i64) -> Result<()> {
    with_lab(async move |lab| {
        let period = Period::business_day(date(2026, 11, 5))?;
        let t = Instant::now();
        seed_generated_ledger(&lab.pool, rows, &period).await?;
        println!(
            "nạp sổ cái sinh sẵn : {} ms (ngoài phép đo)",
            ms(t.elapsed())
        );
        let expected = expected_counts(rows);
        match mode {
            Mode::Sql => {
                let req = RunRequest {
                    provider: "vixanh",
                    path: &path,
                    period,
                    run_at: period.end,
                };
                let outcome = reconcile_file(&lab.pool, &req).await?;
                let crate::reconcile::RunOutcome::Created { summary, timings } = outcome else {
                    bail!("đáng lẽ là run mới");
                };
                println!("chế độ              : SQL (COPY vào bảng tạm, khớp bằng SQL)");
                println!("băm SHA-256         : {} ms", ms(timings.hash));
                println!("đọc CSV + COPY      : {} ms", ms(timings.load));
                println!("khớp + ghi ngoại lệ : {} ms", ms(timings.matching));
                println!("cả run              : {} ms", ms(timings.total));
                println!(
                    "dòng sao kê / hỏng / sổ : {} / {} / {}",
                    summary.statement_rows, summary.bad_lines, summary.ledger_rows
                );
                println!("ref khớp            : {}", summary.refs_matched);
                for (kind, n) in &summary.exceptions {
                    println!("{:<20}: {n}", kind.as_str());
                }
                check_summary(&summary, &expected)?;
            }
            Mode::Rust => {
                let (result, t) = match_in_memory(&lab.pool, &path, &period).await?;
                println!("chế độ              : Rust (HashMap giữ cả hai phía)");
                println!("đọc CSV vào HashMap : {} ms", ms(t.read_statement));
                println!("đọc sổ cái từ PG    : {} ms", ms(t.read_ledger));
                println!("phân loại           : {} ms", ms(t.classify));
                println!("ref khớp            : {}", result.matched);
                let count = |kind| result.exceptions.iter().filter(|e| e.kind == kind).count();
                for kind in [
                    Kind::MissingInLedger,
                    Kind::MissingInPsp,
                    Kind::AmountMismatch,
                    Kind::Duplicate,
                    Kind::DateShift,
                ] {
                    println!("{:<20}: {}", kind.as_str(), count(kind));
                }
                let want = [
                    (Kind::MissingInLedger, expected.missing_in_ledger),
                    (Kind::MissingInPsp, expected.missing_in_psp),
                    (Kind::AmountMismatch, expected.amount_mismatch),
                    (Kind::Duplicate, expected.duplicate),
                    (Kind::DateShift, expected.date_shift),
                ];
                for (kind, n) in want {
                    if i64::try_from(count(kind))? != n {
                        bail!("{}: khác kỳ vọng {n}", kind.as_str());
                    }
                }
                if i64::try_from(result.matched)? != expected.matched {
                    bail!("số ref khớp khác kỳ vọng {}", expected.matched);
                }
            }
        }
        println!("khớp kỳ vọng suy từ công thức sinh dữ liệu: có");
        Ok(())
    })
    .await
}
