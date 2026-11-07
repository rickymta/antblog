//! Báo cáo đối soát sao kê với sổ cái (bản nhỏ trong bộ nhớ của bài 47).

use std::collections::HashMap;
use std::fmt::Write as _;

use serde::Serialize;

use crate::clock::Clock;
use crate::statement::{format_amount, parse_line};

/// Một dòng sao kê không đọc được.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BadLine {
    /// Số dòng trong file, tiêu đề là dòng 1.
    pub line_no: usize,
    /// Lý do.
    pub reason: String,
}

/// Loại lệch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MismatchKind {
    /// Sao kê có, sổ ta không có.
    MissingInLedger,
    /// Sổ ta có, sao kê không có.
    MissingInPsp,
    /// Cả hai có nhưng tổng tiền khác nhau.
    AmountMismatch,
}

/// Một ref lệch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Mismatch {
    /// Mã giao dịch VíXanh.
    pub external_ref: String,
    /// Loại lệch.
    pub kind: MismatchKind,
    /// Tổng tiền phía sổ cái.
    pub ledger_dong: Option<i64>,
    /// Tổng tiền phía sao kê.
    pub psp_dong: Option<i64>,
}

/// Báo cáo một lần đối soát. Ba trường đầu đổi theo mỗi lần chạy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Report {
    /// Mã lần chạy.
    pub run_id: String,
    /// Thời điểm tạo báo cáo (UTC, RFC 3339).
    pub generated_at: String,
    /// Thời gian chạy theo đồng hồ truyền vào.
    pub duration_ms: i64,
    /// Số ref khớp.
    pub matched: usize,
    /// Dòng sao kê hỏng.
    pub bad_lines: Vec<BadLine>,
    /// Các ref lệch, sắp theo `external_ref`.
    pub mismatches: Vec<Mismatch>,
}

/// Mã lần chạy mới (UUID v4): thay đổi mỗi lần gọi.
#[must_use]
pub fn new_run_id() -> String {
    format!("run-{}", uuid::Uuid::new_v4())
}

/// Đọc sao kê: cộng tiền theo `external_ref` (1-n gộp thành một tổng) và gom dòng hỏng.
#[must_use]
pub fn aggregate(statement: &str) -> (HashMap<String, i64>, Vec<BadLine>) {
    let mut totals: HashMap<String, i64> = HashMap::new();
    let mut bad = Vec::new();
    for (i, line) in statement.lines().enumerate().skip(1) {
        let line_no = i + 1;
        if line.trim().is_empty() {
            continue;
        }
        match parse_line(line) {
            Ok(row) => {
                let slot = totals.entry(row.external_ref).or_insert(0);
                match slot.checked_add(row.amount_dong) {
                    Some(sum) => *slot = sum,
                    None => bad.push(BadLine {
                        line_no,
                        reason: "tổng tiền của ref tràn i64".to_owned(),
                    }),
                }
            }
            Err(e) => bad.push(BadLine {
                line_no,
                reason: e.to_string(),
            }),
        }
    }
    (totals, bad)
}

/// Bản đầu tiên: duyệt thẳng `HashMap`, thứ tự kết quả đổi theo từng tiến trình (và từng map).
/// Giữ lại để test chứng minh nó không tất định.
#[doc(hidden)]
#[must_use]
pub fn compare_unordered(
    ledger: &HashMap<String, i64>,
    psp: &HashMap<String, i64>,
) -> Vec<Mismatch> {
    let mut out = Vec::new();
    for (r, &l) in ledger {
        match psp.get(r) {
            None => out.push(mismatch(r, MismatchKind::MissingInPsp, Some(l), None)),
            Some(&p) if p != l => {
                out.push(mismatch(r, MismatchKind::AmountMismatch, Some(l), Some(p)));
            }
            Some(_) => {}
        }
    }
    for (r, &p) in psp {
        if !ledger.contains_key(r) {
            out.push(mismatch(r, MismatchKind::MissingInLedger, None, Some(p)));
        }
    }
    out
}

/// Bản đã sửa: cùng logic, nhưng sắp theo `external_ref` nên thứ tự cố định.
#[must_use]
pub fn compare(ledger: &HashMap<String, i64>, psp: &HashMap<String, i64>) -> Vec<Mismatch> {
    let mut out = compare_unordered(ledger, psp);
    out.sort_by(|a, b| a.external_ref.cmp(&b.external_ref));
    out
}

fn mismatch(r: &str, kind: MismatchKind, l: Option<i64>, p: Option<i64>) -> Mismatch {
    Mismatch {
        external_ref: r.to_owned(),
        kind,
        ledger_dong: l,
        psp_dong: p,
    }
}

/// Đối soát `statement` với `ledger`; giờ lấy từ `clock`, mã lần chạy do người gọi đưa vào.
#[must_use]
pub fn reconcile(
    ledger: &HashMap<String, i64>,
    statement: &str,
    clock: &dyn Clock,
    run_id: &str,
) -> Report {
    let started = clock.now();
    let (psp, bad_lines) = aggregate(statement);
    let mismatches = compare(ledger, &psp);
    let matched = ledger
        .iter()
        .filter(|&(r, l)| psp.get(r) == Some(l))
        .count();
    let finished = clock.now();
    Report {
        run_id: run_id.to_owned(),
        generated_at: finished.to_string(),
        duration_ms: finished.as_millisecond() - started.as_millisecond(),
        matched,
        bad_lines,
        mismatches,
    }
}

impl Report {
    /// Bản văn bản cho người đọc (Finance), dùng làm snapshot dạng text.
    #[must_use]
    pub fn render_text(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(
            out,
            "Đối soát {} lúc {} ({} ms)",
            self.run_id, self.generated_at, self.duration_ms
        );
        let _ = writeln!(
            out,
            "khớp {} | lệch {} | dòng hỏng {}",
            self.matched,
            self.mismatches.len(),
            self.bad_lines.len()
        );
        for m in &self.mismatches {
            let side = |v: Option<i64>| v.map_or_else(|| "-".to_owned(), format_amount);
            let _ = writeln!(
                out,
                "  {:<11} {:<16} sổ={} sao kê={}",
                m.external_ref,
                format!("{:?}", m.kind),
                side(m.ledger_dong),
                side(m.psp_dong)
            );
        }
        for b in &self.bad_lines {
            let _ = writeln!(out, "  dòng {}: {}", b.line_no, b.reason);
        }
        out
    }
}
