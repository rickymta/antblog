//! Biến một lô bút toán thành các cột (mảng) để gửi một câu lệnh `unnest`.
//! Hai cách dựng: mượn `&str` từ bút toán, hoặc `clone` từng chuỗi như bài 30.

use std::collections::HashSet;

use ledger_core::JournalEntry;

/// Bỏ bút toán trùng `idempotency_key` trong cùng lô, giữ bản đầu tiên.
pub fn dedupe(entries: &[JournalEntry]) -> Vec<&JournalEntry> {
    let mut seen = HashSet::with_capacity(entries.len());
    entries
        .iter()
        .filter(|e| seen.insert(e.idempotency_key()))
        .collect()
}

/// Các cột dạng mượn: chuỗi vẫn nằm trong `JournalEntry`, vector chỉ giữ `&str`.
#[derive(Debug)]
pub struct Columns<'a> {
    /// Một khóa mỗi bút toán.
    pub keys: Vec<&'a str>,
    /// Khóa của bút toán chứa dòng đó (mỗi dòng một phần tử).
    pub key_of_line: Vec<&'a str>,
    /// Số thứ tự dòng trong bút toán, từ 1.
    pub line_nos: Vec<i32>,
    /// Mã tài khoản mỗi dòng.
    pub accounts: Vec<&'a str>,
    /// Số tiền có dấu mỗi dòng, đơn vị đồng.
    pub amounts: Vec<i64>,
}

impl<'a> Columns<'a> {
    /// Mượn chuỗi, cấp phát đúng 5 vector (mỗi cột một lần, biết trước dung lượng).
    pub fn borrow(entries: &[&'a JournalEntry]) -> Columns<'a> {
        let lines: usize = entries.iter().map(|e| e.postings().len()).sum();
        let mut cols = Columns {
            keys: Vec::with_capacity(entries.len()),
            key_of_line: Vec::with_capacity(lines),
            line_nos: Vec::with_capacity(lines),
            accounts: Vec::with_capacity(lines),
            amounts: Vec::with_capacity(lines),
        };
        for &entry in entries {
            cols.keys.push(entry.idempotency_key());
            for (i, p) in entry.postings().iter().enumerate() {
                cols.key_of_line.push(entry.idempotency_key());
                cols.line_nos
                    .push(i32::try_from(i + 1).expect("bút toán ít dòng"));
                cols.accounts.push(p.account.as_str());
                cols.amounts.push(p.amount.dong());
            }
        }
        cols
    }
}

/// Các cột dạng sở hữu: mỗi chuỗi `.to_string()` một lần, như `pg.rs` của bài 30.
#[derive(Debug)]
pub struct ColumnsOwned {
    /// Một khóa mỗi bút toán.
    pub keys: Vec<String>,
    /// Khóa của bút toán chứa dòng đó.
    pub key_of_line: Vec<String>,
    /// Số thứ tự dòng.
    pub line_nos: Vec<i32>,
    /// Mã tài khoản mỗi dòng.
    pub accounts: Vec<String>,
    /// Số tiền có dấu mỗi dòng.
    pub amounts: Vec<i64>,
}

impl ColumnsOwned {
    /// Sao chép chuỗi: cấp phát thêm một lần cho mỗi khóa và mỗi mã tài khoản.
    pub fn clone_from(entries: &[&JournalEntry]) -> ColumnsOwned {
        let (mut keys, mut key_of_line) = (Vec::new(), Vec::new());
        let (mut line_nos, mut accounts, mut amounts) = (Vec::new(), Vec::new(), Vec::new());
        for entry in entries {
            keys.push(entry.idempotency_key().to_string());
            for (i, p) in entry.postings().iter().enumerate() {
                key_of_line.push(entry.idempotency_key().to_string());
                line_nos.push(i32::try_from(i + 1).expect("bút toán ít dòng"));
                accounts.push(p.account.as_str().to_string());
                amounts.push(p.amount.dong());
            }
        }
        ColumnsOwned {
            keys,
            key_of_line,
            line_nos,
            accounts,
            amounts,
        }
    }
}
