//! Đọc sao kê VíXanh giả dạng văn bản: dòng đầu là tiêu đề, mỗi dòng sau là
//! `external_ref,số_tiền`, dòng bắt đầu bằng `---` mở đầu phần chân trang.

use std::error::Error;
use std::fmt;

use crate::money::Money;
use crate::reconcile::PspRecord;

/// Lỗi đọc sao kê, luôn kèm số dòng (đếm từ 1) để người vận hành tìm được.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatementError {
    MissingComma { line: usize },
    EmptyRef { line: usize },
    BadAmount { line: usize, raw: String },
}

impl fmt::Display for StatementError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StatementError::MissingComma { line } => write!(f, "dòng {line}: thiếu dấu phẩy"),
            StatementError::EmptyRef { line } => write!(f, "dòng {line}: external_ref rỗng"),
            StatementError::BadAmount { line, raw } => {
                write!(f, "dòng {line}: số tiền không hợp lệ ({raw:?})")
            }
        }
    }
}

impl Error for StatementError {}

fn parse_line(line_no: usize, line: &str) -> Result<PspRecord, StatementError> {
    let (raw_ref, raw_amount) = line
        .split_once(',')
        .ok_or(StatementError::MissingComma { line: line_no })?;
    let external_ref = raw_ref.trim();
    if external_ref.is_empty() {
        return Err(StatementError::EmptyRef { line: line_no });
    }
    let raw_amount = raw_amount.trim();
    let dong = raw_amount
        .parse::<i64>()
        .map_err(|_| StatementError::BadAmount {
            line: line_no,
            raw: raw_amount.to_string(),
        })?;
    Ok(PspRecord {
        external_ref: external_ref.to_string(),
        amount: Money::from_dong(dong),
    })
}

/// Đọc cả sao kê. `collect::<Result<Vec<_>, _>>()` dừng ở lỗi đầu tiên và trả
/// lỗi đó; không có lỗi thì trả `Vec` đủ mọi dòng.
pub fn parse_statement(text: &str) -> Result<Vec<PspRecord>, StatementError> {
    text.lines()
        .enumerate() // đánh số trước khi bỏ tiêu đề, để số dòng báo lỗi đúng
        .skip(1) // bỏ dòng tiêu đề
        .take_while(|(_, line)| !line.starts_with("---")) // dừng ở chân trang
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(index, line)| parse_line(index + 1, line))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD: &str =
        "external_ref,amount_vnd\nVX-1,100\n\nVX-2, 250 \n---\ntổng: 2 dòng\nVX-3,999\n";

    #[test]
    fn parses_until_footer_and_skips_blank_lines() {
        let records = parse_statement(GOOD).unwrap();
        let refs: Vec<&str> = records.iter().map(|r| r.external_ref.as_str()).collect();
        assert_eq!(refs, ["VX-1", "VX-2"]);
        assert_eq!(records[1].amount, Money::from_dong(250));
    }

    #[test]
    fn first_error_wins_and_reports_line_number() {
        let text = "h\nVX-1,100\nVX-2,12a\nVX-3\n";
        assert_eq!(
            parse_statement(text),
            Err(StatementError::BadAmount {
                line: 3,
                raw: "12a".to_string()
            })
        );
    }

    #[test]
    fn missing_comma_and_empty_ref_are_rejected() {
        assert_eq!(
            parse_statement("h\nVX-1 100\n"),
            Err(StatementError::MissingComma { line: 2 })
        );
        assert_eq!(
            parse_statement("h\n ,100\n"),
            Err(StatementError::EmptyRef { line: 2 })
        );
    }

    #[test]
    fn header_only_is_empty() {
        assert_eq!(parse_statement("external_ref,amount_vnd\n"), Ok(vec![]));
    }
}
