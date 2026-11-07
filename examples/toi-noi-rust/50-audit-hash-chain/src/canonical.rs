//! Dạng byte chuẩn (canonical bytes) của một bút toán và công thức hash chain.
//!
//! Hàm thuần: không đọc đồng hồ, không đụng database. Cùng đầu vào luôn ra cùng byte,
//! trên mọi máy và mọi phiên bản PostgreSQL, nên người kiểm tra tính lại được.

use sha2::{Digest, Sha256};

/// `prev_hash` của bút toán đầu tiên: 32 byte 0.
pub const GENESIS: [u8; 32] = [0; 32];

/// Tiền tố cố định, có số phiên bản: đổi dạng byte thì đổi `V1` thành `V2`, hash cũ không lẫn.
const DOMAIN: &[u8] = b"TKH-LEDGER-ENTRY-V1";

/// Một posting như được lưu trong bảng `postings`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalPosting {
    /// Số thứ tự dòng trong bút toán, bắt đầu từ 1.
    pub line_no: i32,
    /// Mã tài khoản, ví dụ `customer:CU-001`.
    pub account_id: String,
    /// Số tiền có dấu, đơn vị đồng.
    pub amount_dong: i64,
}

/// Mọi trường của bút toán mà hash phải bao phủ.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalEntry {
    /// Mã bút toán (`journal_entries.id`).
    pub entry_id: i64,
    /// `idempotency_key`.
    pub idempotency_key: String,
    /// Bút toán bị đảo, nếu có.
    pub reversal_of: Option<i64>,
    /// `posted_at` tính bằng micro giây từ epoch UTC (độ chính xác của `timestamptz`).
    pub posted_at_micros: i64,
    /// Các posting; hàm tự sắp theo `line_no` nên thứ tự đầu vào không ảnh hưởng.
    pub postings: Vec<CanonicalPosting>,
}

// Chuỗi luôn đi kèm độ dài (u32 big-endian): hai bút toán khác nhau không thể ra cùng byte.
fn put_str(out: &mut Vec<u8>, s: &str) {
    let len = u32::try_from(s.len()).expect("chuỗi ngắn hơn 4 GiB");
    out.extend_from_slice(&len.to_be_bytes());
    out.extend_from_slice(s.as_bytes());
}

impl CanonicalEntry {
    /// Byte chuẩn: tiền tố phiên bản, rồi các trường theo thứ tự cố định, số nguyên big-endian.
    ///
    /// ```text
    /// "TKH-LEDGER-ENTRY-V1" | entry_id:i64 | key:(u32 len, utf8) | reversal:(0 | 1, i64)
    ///   | posted_at_micros:i64 | số posting:u32 | { line_no:i32 | account:(u32 len, utf8) | amount:i64 }*
    /// ```
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
pub fn entry_hash(prev_hash: &[u8; 32], canonical: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(prev_hash);
    hasher.update(canonical);
    hasher.finalize().into()
}

/// JE-000001 của TN-0001 (dạng hai bước, bước capture) với `posted_at` cố định để so sánh byte.
pub fn demo_entry() -> CanonicalEntry {
    CanonicalEntry {
        entry_id: 1,
        idempotency_key: "trip:TN-0001:capture".to_string(),
        reversal_of: None,
        posted_at_micros: 1_794_000_000_000_000,
        postings: vec![
            CanonicalPosting {
                line_no: 1,
                account_id: "customer:CU-001".to_string(),
                amount_dong: -25_500,
            },
            CanonicalPosting {
                line_no: 2,
                account_id: "psp:vixanh:clearing".to_string(),
                amount_dong: 25_500,
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> CanonicalEntry {
        demo_entry()
    }

    #[test]
    fn bytes_are_deterministic_and_order_independent() {
        let a = sample();
        let mut b = sample();
        b.postings.reverse();
        assert_eq!(a.to_bytes(), b.to_bytes());
        // 19 (tiền tố) + 8 + (4 + 20) + 1 + 8 + 4 + (4 + 4 + 15 + 8) + (4 + 4 + 19 + 8) = 130 byte
        assert_eq!(a.to_bytes().len(), 130);
    }

    #[test]
    fn every_field_changes_the_bytes() {
        let base = sample().to_bytes();
        let mut variants = Vec::new();
        let mut v = sample();
        v.entry_id = 2;
        variants.push(v);
        let mut v = sample();
        v.idempotency_key.push('x');
        variants.push(v);
        let mut v = sample();
        v.reversal_of = Some(1);
        variants.push(v);
        let mut v = sample();
        v.posted_at_micros += 1;
        variants.push(v);
        let mut v = sample();
        v.postings[0].line_no = 3;
        variants.push(v);
        let mut v = sample();
        v.postings[0].amount_dong = -25_501;
        variants.push(v);
        let mut v = sample();
        v.postings[1].account_id = "psp:vixanh:clearinG".to_string();
        variants.push(v);
        for v in variants {
            assert_ne!(v.to_bytes(), base);
        }
    }

    #[test]
    fn length_prefix_prevents_boundary_ambiguity() {
        // Hai cách chia chuỗi khác nhau quanh ranh giới key và tài khoản phải ra byte khác nhau.
        let mut a = sample();
        a.idempotency_key = "ab".to_string();
        a.postings[0].account_id = "c".to_string();
        let mut b = sample();
        b.idempotency_key = "a".to_string();
        b.postings[0].account_id = "bc".to_string();
        assert_ne!(a.to_bytes(), b.to_bytes());
    }

    #[test]
    fn hash_matches_known_value() {
        // Giá trị đối chiếu độc lập bằng `shasum -a 256` (xem README của lab).
        let canonical = sample().to_bytes();
        let hash = entry_hash(&GENESIS, &canonical);
        assert_eq!(
            hex::encode(hash),
            "549ceaaa1d58872d830b67c2a31ff6c6f0f11d03221db1b0b342927a79b6cf11"
        );
    }

    #[test]
    fn chain_links_change_with_prev() {
        let c = sample().to_bytes();
        let h1 = entry_hash(&GENESIS, &c);
        let h2 = entry_hash(&h1, &c);
        assert_ne!(h1, h2);
    }
}
