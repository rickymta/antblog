//! Hash chain của sổ (bài 50), bản rút gọn để so sánh bản gốc với bản restore.
//!
//! Không lưu bảng chuỗi: đọc bút toán theo `id`, dựng byte chuẩn, nối `SHA-256(prev || bytes)`.
//! Cùng dữ liệu thì cùng `head` trên mọi máy; đổi một posting thì `head` đổi.

use std::collections::BTreeMap;

use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row};

/// `prev_hash` của bút toán đầu tiên.
pub const GENESIS: [u8; 32] = [0; 32];
const DOMAIN: &[u8] = b"TKH-LEDGER-ENTRY-V1";

/// Một posting như được lưu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalPosting {
    /// Số dòng, từ 1.
    pub line_no: i32,
    /// Mã tài khoản.
    pub account_id: String,
    /// Số tiền có dấu, đồng.
    pub amount_dong: i64,
}

/// Mọi trường mà hash bao phủ.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalEntry {
    /// `journal_entries.id`.
    pub entry_id: i64,
    /// `idempotency_key`.
    pub idempotency_key: String,
    /// Bút toán bị đảo, nếu có.
    pub reversal_of: Option<i64>,
    /// `posted_at` tính bằng micro giây từ epoch UTC.
    pub posted_at_micros: i64,
    /// Các posting.
    pub postings: Vec<CanonicalPosting>,
}

fn put_str(out: &mut Vec<u8>, s: &str) {
    let len = u32::try_from(s.len()).expect("chuỗi ngắn hơn 4 GiB");
    out.extend_from_slice(&len.to_be_bytes());
    out.extend_from_slice(s.as_bytes());
}

impl CanonicalEntry {
    /// Byte chuẩn: tiền tố phiên bản, các trường theo thứ tự cố định, số nguyên big-endian.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(96 + self.postings.len() * 40);
        out.extend_from_slice(DOMAIN);
        out.extend_from_slice(&self.entry_id.to_be_bytes());
        put_str(&mut out, &self.idempotency_key);
        match self.reversal_of {
            None => out.push(0),
            Some(id) => {
                out.push(1);
                out.extend_from_slice(&id.to_be_bytes());
            }
        }
        out.extend_from_slice(&self.posted_at_micros.to_be_bytes());
        let count = u32::try_from(self.postings.len()).expect("ít hơn 2^32 posting");
        out.extend_from_slice(&count.to_be_bytes());
        let mut lines: Vec<&CanonicalPosting> = self.postings.iter().collect();
        lines.sort_by_key(|p| p.line_no);
        for p in lines {
            out.extend_from_slice(&p.line_no.to_be_bytes());
            put_str(&mut out, &p.account_id);
            out.extend_from_slice(&p.amount_dong.to_be_bytes());
        }
        out
    }
}

/// `entry_hash = SHA-256(prev_hash || canonical_bytes)`.
pub fn entry_hash(prev: &[u8; 32], canonical: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(prev);
    hasher.update(canonical);
    hasher.finalize().into()
}

/// Kết quả gấp cả chuỗi.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chain {
    /// Số bút toán đã nối.
    pub entries: usize,
    /// Hash của bút toán cuối (hoặc [`GENESIS`] khi sổ trống).
    pub head: [u8; 32],
}

impl Chain {
    /// `head` dạng hex.
    pub fn head_hex(&self) -> String {
        hex::encode(self.head)
    }
}

/// Nối các bút toán theo thứ tự đưa vào.
pub fn fold(entries: impl IntoIterator<Item = CanonicalEntry>) -> Chain {
    let mut head = GENESIS;
    let mut count = 0;
    for entry in entries {
        head = entry_hash(&head, &entry.to_bytes());
        count += 1;
    }
    Chain {
        entries: count,
        head,
    }
}

/// Đọc bút toán từ DB (tối đa `limit` bút toán đầu, theo `id`) rồi gấp thành chuỗi.
///
/// # Errors
/// Lỗi SQL.
pub async fn read_chain(pool: &PgPool, limit: Option<i64>) -> Result<Chain, sqlx::Error> {
    let limit = limit.unwrap_or(i64::MAX);
    let entries = sqlx::query(
        "SELECT id, idempotency_key, reversal_of, \
                (extract(epoch FROM posted_at) * 1000000)::bigint AS micros \
         FROM journal_entries ORDER BY id LIMIT $1",
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;
    let lines = sqlx::query(
        "SELECT entry_id, line_no, account_id, amount_dong FROM postings \
         WHERE entry_id IN (SELECT id FROM journal_entries ORDER BY id LIMIT $1) \
         ORDER BY entry_id, line_no",
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;
    let mut by_entry: BTreeMap<i64, Vec<CanonicalPosting>> = BTreeMap::new();
    for row in lines {
        by_entry
            .entry(row.get("entry_id"))
            .or_default()
            .push(CanonicalPosting {
                line_no: row.get("line_no"),
                account_id: row.get("account_id"),
                amount_dong: row.get("amount_dong"),
            });
    }
    Ok(fold(entries.into_iter().map(|row| {
        let entry_id: i64 = row.get("id");
        CanonicalEntry {
            entry_id,
            idempotency_key: row.get("idempotency_key"),
            reversal_of: row.get("reversal_of"),
            posted_at_micros: row.get("micros"),
            postings: by_entry.remove(&entry_id).unwrap_or_default(),
        }
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: i64, key: &str, amount: i64) -> CanonicalEntry {
        CanonicalEntry {
            entry_id: id,
            idempotency_key: key.to_string(),
            reversal_of: None,
            posted_at_micros: 1_794_000_000_000_000,
            postings: vec![
                CanonicalPosting {
                    line_no: 1,
                    account_id: "customer:CU-001".to_string(),
                    amount_dong: -amount,
                },
                CanonicalPosting {
                    line_no: 2,
                    account_id: "psp:vixanh:clearing".to_string(),
                    amount_dong: amount,
                },
            ],
        }
    }

    #[test]
    fn same_data_same_head_and_append_extends_the_chain() {
        let a = vec![entry(1, "trip:TN-0001:capture", 25_500)];
        let b = vec![entry(1, "trip:TN-0001:capture", 25_500)];
        assert_eq!(fold(a.clone()), fold(b));
        let mut longer = a.clone();
        longer.push(entry(2, "trip:TN-0002:capture", 25_500));
        assert_ne!(fold(a.clone()).head, fold(longer.clone()).head);
        // Phần đầu của chuỗi dài hơn vẫn cho đúng head của chuỗi ngắn: sổ chỉ nối thêm.
        assert_eq!(fold(longer[..1].to_vec()), fold(a));
    }

    #[test]
    fn changing_one_amount_changes_the_head() {
        let honest = fold(vec![entry(1, "k", 25_500), entry(2, "k2", 100)]);
        let tampered = fold(vec![entry(1, "k", 25_500), entry(2, "k2", 101)]);
        assert_ne!(honest.head, tampered.head);
        assert_eq!(fold(Vec::new()).head, GENESIS);
    }
}
