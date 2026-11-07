//! Parser một dòng sao kê VíXanh dạng `external_ref,amount_vnd,settled_at` (bản nhỏ của bài 47).
//! Có hai bản đọc số tiền: `parse_amount` (đã sửa) và `parse_amount_v1` (bản ngây thơ có lỗi thật,
//! giữ lại để cargo-fuzz tái hiện).

use std::fmt;

use jiff::Timestamp;

/// Một dòng sao kê đã kiểm tra.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatementRow {
    /// Mã giao dịch VíXanh.
    pub external_ref: String,
    /// Số tiền có dấu (thu dương, hoàn âm), đơn vị đồng, khác 0.
    pub amount_dong: i64,
    /// Thời điểm VíXanh ghi nhận.
    pub settled_at: Timestamp,
}

/// Lý do một dòng bị từ chối.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// Số cột khác 3.
    FieldCount(usize),
    /// `external_ref` rỗng.
    EmptyRef,
    /// `external_ref` dài quá 64 byte hoặc có ký tự ngoài `[A-Za-z0-9_]`.
    BadRef,
    /// Số tiền không đúng định dạng hoặc ngoài phạm vi `i64`.
    BadAmount,
    /// Số tiền bằng 0.
    ZeroAmount,
    /// `settled_at` không phải RFC 3339.
    BadTime,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::FieldCount(n) => write!(f, "số cột sai: có {n}, cần 3"),
            ParseError::EmptyRef => f.write_str("external_ref rỗng"),
            ParseError::BadRef => f.write_str("external_ref không hợp lệ"),
            ParseError::BadAmount => f.write_str("số tiền không hợp lệ"),
            ParseError::ZeroAmount => f.write_str("số tiền bằng 0"),
            ParseError::BadTime => f.write_str("settled_at không phải RFC 3339"),
        }
    }
}

impl std::error::Error for ParseError {}

impl StatementRow {
    /// Viết lại thành một dòng sao kê; `parse_line(&row.to_line())` phải trả đúng `row`.
    #[must_use]
    pub fn to_line(&self) -> String {
        format!(
            "{},{},{}",
            self.external_ref,
            format_amount(self.amount_dong),
            self.settled_at
        )
    }
}

/// Đọc một dòng sao kê (không gồm ký tự xuống dòng).
///
/// # Errors
/// [`ParseError`] cho từng kiểu dòng hỏng.
pub fn parse_line(line: &str) -> Result<StatementRow, ParseError> {
    parse_line_with(line, parse_amount)
}

/// Như [`parse_line`] nhưng dùng bản đọc số tiền ngây thơ có lỗi (chỉ để fuzz tái hiện).
///
/// # Errors
/// [`ParseError`] cho từng kiểu dòng hỏng.
#[doc(hidden)]
pub fn parse_line_v1(line: &str) -> Result<StatementRow, ParseError> {
    parse_line_with(line, parse_amount_v1)
}

fn parse_line_with(
    line: &str,
    amount_parser: fn(&str) -> Result<i64, ParseError>,
) -> Result<StatementRow, ParseError> {
    let fields: Vec<&str> = line.split(',').collect();
    let [external_ref, amount, settled_at] = fields[..] else {
        return Err(ParseError::FieldCount(fields.len()));
    };
    let external_ref = external_ref.trim();
    if external_ref.is_empty() {
        return Err(ParseError::EmptyRef);
    }
    if external_ref.len() > 64
        || !external_ref
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_')
    {
        return Err(ParseError::BadRef);
    }
    let amount_dong = amount_parser(amount)?;
    let settled_at = settled_at
        .trim()
        .parse::<Timestamp>()
        .map_err(|_| ParseError::BadTime)?;
    Ok(StatementRow {
        external_ref: external_ref.to_owned(),
        amount_dong,
        settled_at,
    })
}

/// Đọc số tiền: dấu `-` tùy chọn, chữ số (có thể nhóm bằng dấu chấm kiểu `25.500`),
/// hậu tố ` VND` tùy chọn. Số 0 và số ngoài `i64` bị từ chối. Làm việc trên byte nên không bao giờ
/// cắt giữa một ký tự UTF-8.
///
/// # Errors
/// [`ParseError::BadAmount`] hoặc [`ParseError::ZeroAmount`].
pub fn parse_amount(raw: &str) -> Result<i64, ParseError> {
    let raw = raw.trim().as_bytes();
    let (negative, rest) = match raw.strip_prefix(b"-") {
        Some(r) => (true, r),
        None => (false, raw),
    };
    // Ba byte cuối là ASCII nếu khớp «VND», nên cắt ở đó luôn đúng ranh giới ký tự.
    let rest = if rest.len() >= 3 && rest[rest.len() - 3..].eq_ignore_ascii_case(b"VND") {
        rest[..rest.len() - 3].trim_ascii_end()
    } else {
        rest
    };
    if !valid_grouping(rest) {
        return Err(ParseError::BadAmount);
    }
    let limit = i128::from(i64::MAX) + 1; // |i64::MIN|
    let mut value: i128 = 0;
    for &b in rest.iter().filter(|&&b| b != b'.') {
        value = value * 10 + i128::from(b - b'0');
        if value > limit {
            return Err(ParseError::BadAmount);
        }
    }
    let signed = if negative { -value } else { value };
    let amount = i64::try_from(signed).map_err(|_| ParseError::BadAmount)?;
    if amount == 0 {
        return Err(ParseError::ZeroAmount);
    }
    Ok(amount)
}

/// Chữ số liền nhau, hoặc nhóm kiểu `1.234.567`: nhóm đầu 1-3 chữ số, các nhóm sau đúng 3.
fn valid_grouping(digits: &[u8]) -> bool {
    if digits.is_empty() {
        return false;
    }
    if !digits.contains(&b'.') {
        return digits.iter().all(u8::is_ascii_digit);
    }
    let mut groups = digits.split(|&b| b == b'.');
    let first_ok = groups
        .next()
        .is_some_and(|g| (1..=3).contains(&g.len()) && g.iter().all(u8::is_ascii_digit));
    first_ok && groups.all(|g| g.len() == 3 && g.iter().all(u8::is_ascii_digit))
}

/// Viết số tiền theo kiểu có nhóm: `25.500`, `-1.200.000`.
#[must_use]
pub fn format_amount(amount: i64) -> String {
    let digits = amount.unsigned_abs().to_string();
    let mut out = String::new();
    if amount < 0 {
        out.push('-');
    }
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push('.');
        }
        out.push(c);
    }
    out
}

/// Bản đọc số tiền ngây thơ, viết lần đầu: cắt chuỗi theo chỉ số byte và đảo dấu bằng `-n`.
/// Có lỗi thật mà test thủ công không thấy; xem `tests/fuzz_regression.rs`.
///
/// # Errors
/// [`ParseError::BadAmount`] hoặc [`ParseError::ZeroAmount`].
#[doc(hidden)]
pub fn parse_amount_v1(raw: &str) -> Result<i64, ParseError> {
    let raw = raw.trim();
    // Bỏ hậu tố «VND» nếu có.
    let body = if raw.len() >= 3 && raw[raw.len() - 3..].eq_ignore_ascii_case("VND") {
        raw[..raw.len() - 3].trim_end()
    } else {
        raw
    };
    let (negative, digits) = match body.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, body),
    };
    let n: i64 = digits
        .replace('.', "")
        .parse()
        .map_err(|_| ParseError::BadAmount)?;
    let value = if negative { -n } else { n };
    if value == 0 {
        Err(ParseError::ZeroAmount)
    } else {
        Ok(value)
    }
}
