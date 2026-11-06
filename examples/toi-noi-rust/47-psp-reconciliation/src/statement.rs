//! Đọc sao kê VíXanh dạng CSV theo luồng: mỗi lần một dòng, dòng hỏng ghi lại mà không dừng file.

use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

use csv::{ErrorKind, StringRecord};
use jiff::Timestamp;
use ledger_core::Money;
use serde::Deserialize;
use sha2::{Digest, Sha256};

/// Tiêu đề cột bắt buộc (giả định của lab về định dạng sao kê VíXanh).
pub const HEADER: [&str; 3] = ["external_ref", "amount_vnd", "settled_at"];

/// Hình dạng thô của một dòng; serde điền từ tiêu đề cột.
#[derive(Debug, Deserialize)]
struct RawRow {
    external_ref: String,
    amount_vnd: i64,
    settled_at: String,
}

/// Một dòng sao kê đã kiểm tra.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatementRow {
    /// Số dòng trong file, tiêu đề là dòng 1.
    pub line_no: u64,
    /// Mã giao dịch VíXanh.
    pub external_ref: String,
    /// Số tiền có dấu (thu dương, hoàn âm), đơn vị đồng.
    pub amount: Money,
    /// Thời điểm VíXanh ghi nhận; sao kê có thể ghi `Z` hoặc `+07:00`, đều ra cùng một thời điểm.
    pub settled_at: Timestamp,
}

/// Một dòng không đọc được, kèm số dòng để người vận hành tìm.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BadLine {
    /// Số dòng trong file.
    pub line_no: u64,
    /// Lý do.
    pub reason: String,
}

/// Lỗi làm hỏng cả file (khác với dòng hỏng): đọc ổ đĩa lỗi hoặc tiêu đề sai.
#[derive(Debug)]
pub enum StatementError {
    /// Lỗi I/O khi đọc file.
    Io(String),
    /// Tiêu đề cột không đúng [`HEADER`].
    BadHeader {
        /// Tiêu đề thực tế tìm thấy.
        found: String,
    },
}

impl std::fmt::Display for StatementError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StatementError::Io(e) => write!(f, "lỗi đọc file sao kê: {e}"),
            StatementError::BadHeader { found } => {
                write!(f, "tiêu đề sao kê sai: cần {HEADER:?}, thấy {found:?}")
            }
        }
    }
}

impl std::error::Error for StatementError {}

/// Đọc sao kê qua bất kỳ `Read` nào. Không cần `Seek`, không giữ quá một dòng trong bộ nhớ.
pub struct StatementReader<R: Read> {
    rdr: csv::Reader<R>,
    headers: StringRecord,
    record: StringRecord,
    good: u64,
    bad_total: u64,
    bad_kept: Vec<BadLine>,
    max_kept: usize,
}

impl<R: Read> StatementReader<R> {
    /// Mở `input` và kiểm tiêu đề. Chỉ giữ tối đa `max_kept` dòng hỏng đầu tiên (chi tiết);
    /// số dòng hỏng thật vẫn được đếm đủ, để file toàn dòng hỏng không làm tràn bộ nhớ.
    ///
    /// # Errors
    /// [`StatementError::Io`] hoặc [`StatementError::BadHeader`].
    pub fn new(input: R, max_kept: usize) -> Result<StatementReader<R>, StatementError> {
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .trim(csv::Trim::All)
            .from_reader(input);
        let headers = rdr
            .headers()
            .map_err(|e| StatementError::Io(e.to_string()))?
            .clone();
        if headers.iter().ne(HEADER) {
            return Err(StatementError::BadHeader {
                found: headers.iter().collect::<Vec<_>>().join(","),
            });
        }
        Ok(StatementReader {
            rdr,
            headers,
            record: StringRecord::new(),
            good: 0,
            bad_total: 0,
            bad_kept: Vec::new(),
            max_kept,
        })
    }

    /// Dòng hợp lệ kế tiếp; `Ok(None)` khi hết file. Dòng hỏng được ghi lại rồi bỏ qua.
    ///
    /// # Errors
    /// [`StatementError::Io`] khi đọc ổ đĩa lỗi (không thể đọc tiếp).
    pub fn next_row(&mut self) -> Result<Option<StatementRow>, StatementError> {
        loop {
            match self.rdr.read_record(&mut self.record) {
                Ok(false) => return Ok(None),
                Ok(true) => {
                    let line_no = self.record.position().map_or(0, csv::Position::line);
                    match parse_record(&self.record, &self.headers, line_no) {
                        Ok(row) => {
                            self.good += 1;
                            return Ok(Some(row));
                        }
                        Err(reason) => self.note_bad(line_no, reason),
                    }
                }
                Err(e) => {
                    if matches!(e.kind(), ErrorKind::Io(_)) {
                        return Err(StatementError::Io(e.to_string()));
                    }
                    let line_no = e.position().map_or(0, csv::Position::line);
                    self.note_bad(line_no, describe(&e));
                }
            }
        }
    }

    fn note_bad(&mut self, line_no: u64, reason: String) {
        self.bad_total += 1;
        if self.bad_kept.len() < self.max_kept {
            self.bad_kept.push(BadLine { line_no, reason });
        }
    }

    /// Số dòng hợp lệ đã trả ra.
    pub fn good(&self) -> u64 {
        self.good
    }

    /// Số dòng hỏng đã gặp (đếm đủ).
    pub fn bad_total(&self) -> u64 {
        self.bad_total
    }

    /// Chi tiết các dòng hỏng đầu tiên (tối đa `max_kept`).
    pub fn bad_lines(&self) -> &[BadLine] {
        &self.bad_kept
    }

    /// Trả lại nguồn đã bọc (để lấy hash sau khi đọc hết).
    pub fn into_inner(self) -> R {
        self.rdr.into_inner()
    }
}

/// Mô tả lỗi cấp bản ghi bằng tiếng Việt, ngắn và không lặp lại số dòng.
fn describe(e: &csv::Error) -> String {
    match e.kind() {
        ErrorKind::UnequalLengths {
            expected_len, len, ..
        } => format!("số cột sai: có {len}, cần {expected_len}"),
        ErrorKind::Utf8 { .. } => "không phải UTF-8 hợp lệ".to_string(),
        _ => e.to_string(),
    }
}

fn parse_record(
    record: &StringRecord,
    headers: &StringRecord,
    line_no: u64,
) -> Result<StatementRow, String> {
    // `deserialize` của bản ghi: serde ghép cột theo tên tiêu đề, sai kiểu thì báo cột nào.
    let raw: RawRow = record
        .deserialize(Some(headers))
        .map_err(|e| match e.kind() {
            ErrorKind::Deserialize { err, .. } => {
                let column = err
                    .field()
                    .and_then(|i| usize::try_from(i).ok())
                    .and_then(|i| headers.get(i))
                    .unwrap_or("?");
                format!("cột {column} không hợp lệ: {}", err.kind())
            }
            _ => e.to_string(),
        })?;
    if raw.external_ref.is_empty() {
        return Err("external_ref rỗng".to_string());
    }
    if raw.amount_vnd == 0 {
        return Err("số tiền bằng 0".to_string());
    }
    let settled_at: Timestamp = raw
        .settled_at
        .parse()
        .map_err(|_| format!("settled_at không phải RFC 3339: {:?}", raw.settled_at))?;
    Ok(StatementRow {
        line_no,
        external_ref: raw.external_ref,
        amount: Money::from_dong(raw.amount_vnd),
        settled_at,
    })
}

/// Bọc một `Read` và băm SHA-256 mọi byte đi qua: hash tính trong cùng lượt đọc với parse.
pub struct HashingReader<R> {
    inner: R,
    hasher: Sha256,
}

impl<R: Read> HashingReader<R> {
    /// Bọc `inner`.
    pub fn new(inner: R) -> HashingReader<R> {
        HashingReader {
            inner,
            hasher: Sha256::new(),
        }
    }

    /// SHA-256 (hex thường, 64 ký tự) của mọi byte đã đọc qua.
    pub fn finish_hex(self) -> String {
        hex::encode(self.hasher.finalize())
    }
}

impl<R: Read> Read for HashingReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.inner.read(buf)?;
        self.hasher.update(&buf[..n]);
        Ok(n)
    }
}

/// SHA-256 của file, đọc từng khối 64 KiB (không nạp cả file).
///
/// # Errors
/// Lỗi mở hoặc đọc file.
pub fn sha256_file(path: &Path) -> io::Result<String> {
    let mut reader = HashingReader::new(File::open(path)?);
    let mut chunk = vec![0u8; 64 * 1024];
    while reader.read(&mut chunk)? > 0 {}
    Ok(reader.finish_hex())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read_all(text: &str) -> (Vec<StatementRow>, Vec<BadLine>, u64) {
        let mut reader = StatementReader::new(text.as_bytes(), 100).unwrap();
        let mut rows = Vec::new();
        while let Some(row) = reader.next_row().unwrap() {
            rows.push(row);
        }
        let bad = reader.bad_lines().to_vec();
        (rows, bad, reader.bad_total())
    }

    const H: &str = "external_ref,amount_vnd,settled_at\n";

    #[test]
    fn bad_lines_are_listed_with_line_numbers_and_file_continues() {
        let text = format!(
            "{H}a,100,2026-11-05T01:00:00Z\nb,12x,2026-11-05T01:00:00Z\nc,300\n,400,2026-11-05T01:00:00Z\nd,500,hôm qua\ne,0,2026-11-05T01:00:00Z\nf,600,2026-11-05T01:00:00Z\n"
        );
        let (rows, bad, total) = read_all(&text);
        let refs: Vec<&str> = rows.iter().map(|r| r.external_ref.as_str()).collect();
        assert_eq!(refs, ["a", "f"]);
        assert_eq!(rows[1].line_no, 8);
        let lines: Vec<u64> = bad.iter().map(|b| b.line_no).collect();
        assert_eq!(lines, [3, 4, 5, 6, 7]);
        assert_eq!(total, 5);
    }

    #[test]
    fn invalid_utf8_is_one_bad_line_not_a_crash() {
        let mut bytes = H.as_bytes().to_vec();
        bytes.extend_from_slice(b"a,100,2026-11-05T01:00:00Z\n\xff\xfe,5,2026-11-05T01:00:00Z\nb,200,2026-11-05T01:00:00Z\n");
        let mut reader = StatementReader::new(bytes.as_slice(), 10).unwrap();
        let mut good = 0;
        while reader.next_row().unwrap().is_some() {
            good += 1;
        }
        assert_eq!(good, 2);
        assert_eq!(reader.bad_lines()[0].line_no, 3);
    }

    #[test]
    fn quoted_ref_with_comma_and_offset_timestamp() {
        let text = format!("{H}\"vx,1\",-5000,2026-11-05T07:00:00+07:00\n");
        let (rows, bad, _) = read_all(&text);
        assert!(bad.is_empty());
        assert_eq!(rows[0].external_ref, "vx,1");
        assert_eq!(rows[0].amount, Money::from_dong(-5_000));
        assert_eq!(rows[0].settled_at.to_string(), "2026-11-05T00:00:00Z");
    }

    #[test]
    fn wrong_header_fails_the_whole_file() {
        let err = StatementReader::new("ref,amount\nx,1\n".as_bytes(), 10).err();
        assert!(matches!(err, Some(StatementError::BadHeader { .. })));
    }

    #[test]
    fn kept_details_are_capped_but_count_is_exact() {
        let mut text = String::from(H);
        for _ in 0..50 {
            text.push_str("x,abc,2026-11-05T01:00:00Z\n");
        }
        let mut reader = StatementReader::new(text.as_bytes(), 3).unwrap();
        while reader.next_row().unwrap().is_some() {}
        assert_eq!(reader.bad_total(), 50);
        assert_eq!(reader.bad_lines().len(), 3);
    }

    #[test]
    fn sha256_matches_known_vector() {
        let mut reader = HashingReader::new("abc".as_bytes());
        let mut sink = Vec::new();
        reader.read_to_end(&mut sink).unwrap();
        assert_eq!(
            reader.finish_hex(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
