//! Công cụ cho test tất định: bộ sinh số giả ngẫu nhiên có seed và dữ liệu mẫu.

use std::collections::HashMap;

/// Seed mặc định của CI. Đặt biến môi trường `LEDGER_TEST_SEED` để chạy lại một seed khác.
pub const DEFAULT_SEED: u64 = 20_261_105;

/// Seed cho lần chạy này: lấy từ `LEDGER_TEST_SEED` nếu có, không thì [`DEFAULT_SEED`].
#[must_use]
pub fn seed_from_env() -> u64 {
    std::env::var("LEDGER_TEST_SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_SEED)
}

/// SplitMix64: bộ sinh số giả ngẫu nhiên 8 dòng, cùng seed luôn ra cùng dãy.
pub struct SplitMix64(u64);

impl SplitMix64 {
    /// Tạo bộ sinh từ `seed`.
    #[must_use]
    pub fn new(seed: u64) -> SplitMix64 {
        SplitMix64(seed)
    }

    /// Số 64 bit kế tiếp.
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Số trong `lo..hi` (hi > lo).
    pub fn range(&mut self, lo: u64, hi: u64) -> u64 {
        lo + self.next_u64() % (hi - lo)
    }
}

/// Sao kê và sổ cái sinh theo seed, kèm số lệch đã cài sẵn để test đối chiếu.
pub struct Synthetic {
    /// Nội dung sao kê CSV (có dòng tiêu đề).
    pub statement: String,
    /// Phía sổ cái: ref -> tổng tiền.
    pub ledger: HashMap<String, i64>,
    /// Số ref chỉ có ở sao kê.
    pub missing_in_ledger: usize,
    /// Số ref chỉ có ở sổ cái.
    pub missing_in_psp: usize,
    /// Số ref lệch tiền.
    pub amount_mismatch: usize,
    /// Số ref khớp.
    pub matched: usize,
}

/// Sinh `n` giao dịch theo `seed`: cứ khoảng 1/10 số ref được cài một kiểu lệch.
#[must_use]
pub fn synthetic(seed: u64, n: usize) -> Synthetic {
    let mut rng = SplitMix64::new(seed);
    let mut s = Synthetic {
        statement: String::from("external_ref,amount_vnd,settled_at\n"),
        ledger: HashMap::new(),
        missing_in_ledger: 0,
        missing_in_psp: 0,
        amount_mismatch: 0,
        matched: 0,
    };
    for i in 0..n {
        let external_ref = format!("vx_ch_{i:05}");
        let amount = i64::try_from(rng.range(1, 200)).expect("nhỏ") * 1_000;
        let line = format!("{external_ref},{amount},2026-11-05T03:00:00Z\n");
        match rng.range(0, 10) {
            0 => {
                // Chỉ có ở sao kê.
                s.statement.push_str(&line);
                s.missing_in_ledger += 1;
            }
            1 => {
                // Chỉ có ở sổ cái.
                s.ledger.insert(external_ref, amount);
                s.missing_in_psp += 1;
            }
            2 => {
                // Cả hai có nhưng lệch 500 đồng.
                s.statement.push_str(&line);
                s.ledger.insert(external_ref, amount + 500);
                s.amount_mismatch += 1;
            }
            _ => {
                s.statement.push_str(&line);
                s.ledger.insert(external_ref, amount);
                s.matched += 1;
            }
        }
    }
    s
}
