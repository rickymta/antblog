//! Nối phép chia với `ledger-core`: bút toán, `FeePolicy` và bản ghi quy tắc đã dùng.

use std::fmt;

use ledger_core::{AccountId, FeePolicy, JournalEntry, JournalEntryBuilder, LedgerError, Money};

use crate::allocate::{AllocError, Party, Share, allocate, allocate_with_plug};

/// Quy tắc chia số đồng lẻ: một quyết định nghiệp vụ, phải ghi lại cùng bút toán.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rule {
    /// Largest remainder, hòa phần dư thì `key` nhỏ hơn nhận trước.
    LargestRemainder,
    /// Bên có `key` này nhận phần còn lại, các bên khác làm tròn nửa ra xa 0.
    Plug(String),
}

impl Rule {
    /// Chia `total` theo quy tắc này.
    ///
    /// # Errors
    /// Như [`allocate`] và [`allocate_with_plug`].
    pub fn apply(&self, total: Money, parties: &[Party]) -> Result<Vec<Share>, AllocError> {
        match self {
            Rule::LargestRemainder => allocate(total, parties),
            Rule::Plug(key) => allocate_with_plug(total, parties, key),
        }
    }
}

/// Bản ghi cách chia: đủ để dựng lại đúng từng đồng. Dạng văn bản chuẩn hóa (trọng số xếp
/// theo key), nên cùng quyết định thì cùng chuỗi dù đầu vào xếp thứ tự nào.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AllocationRecord {
    rule: Rule,
    total: Money,
    weights: Vec<(String, u64)>,
}

impl AllocationRecord {
    /// Ghi lại quyết định chia `total` cho `parties` theo `rule`.
    pub fn new(rule: &Rule, total: Money, parties: &[Party]) -> AllocationRecord {
        let mut weights: Vec<(String, u64)> =
            parties.iter().map(|p| (p.key.clone(), p.weight)).collect();
        weights.sort();
        AllocationRecord {
            rule: rule.clone(),
            total,
            weights,
        }
    }
}

impl fmt::Display for AllocationRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.rule {
            Rule::LargestRemainder => write!(f, "rule=largest-remainder/v1 tie=key-asc")?,
            Rule::Plug(key) => write!(f, "rule=plug/v1 plug={key}")?,
        }
        write!(f, " total={} weights=", self.total.dong())?;
        for (i, (key, weight)) in self.weights.iter().enumerate() {
            if i > 0 {
                write!(f, ",")?;
            }
            write!(f, "{key}:{weight}")?;
        }
        Ok(())
    }
}

/// Bút toán chia: `pool` ra `-tổng`, mỗi bên nhận phần của mình. Phần âm (hoàn tiền) cho
/// bút toán ngược chiều. Luôn cân vì posting của `pool` là âm tổng các phần.
///
/// # Errors
/// `LedgerError::Overflow` nếu tổng tràn `i64`, `EmptyAccountId` nếu mã tài khoản rỗng.
pub fn distribute_entry(
    idempotency_key: &str,
    pool: &str,
    shares: &[Share],
) -> Result<JournalEntry, LedgerError> {
    let sum = shares
        .iter()
        .try_fold(Money::ZERO, |acc, s| acc.checked_add(s.amount))
        .ok_or(LedgerError::Overflow)?;
    let mut builder = JournalEntryBuilder::new(idempotency_key).posting(
        AccountId::new(pool)?,
        sum.checked_neg().ok_or(LedgerError::Overflow)?,
    );
    for share in shares {
        builder = builder.posting(AccountId::new(&share.key)?, share.amount);
    }
    builder.build()
}

/// Chính sách phí chia theo largest remainder, cắm vào `FeePolicy` của `ledger-core`.
///
/// Hai bên có `key` cố định `"driver"` và `"platform"`; hòa phần dư thì `"driver"` (nhỏ hơn
/// theo byte) nhận đồng lẻ. Đó là quyết định nghiệp vụ, nên nằm trong `name()`.
#[derive(Debug, Clone, Copy)]
pub struct AllocatedFee {
    platform_bps: u64,
}

impl AllocatedFee {
    /// Phí nền tảng `platform_bps` phần vạn (2.000 là 20%); `None` nếu quá 10.000.
    pub fn new(platform_bps: u64) -> Option<AllocatedFee> {
        (platform_bps <= 10_000).then_some(AllocatedFee { platform_bps })
    }
}

impl FeePolicy for AllocatedFee {
    fn name(&self) -> String {
        format!(
            "{}% · largest remainder, hòa: key tăng dần",
            self.platform_bps / 100
        )
    }

    fn fee(&self, fare: Money) -> Result<Money, LedgerError> {
        let parties = [
            Party::new("platform", self.platform_bps),
            Party::new("driver", 10_000 - self.platform_bps),
        ];
        // Với hai bên và trọng số hợp lệ chỉ có thể lỗi tràn số, không xảy ra với i64.
        let shares = allocate(fare, &parties).map_err(|_| LedgerError::Overflow)?;
        Ok(shares[0].amount)
    }
}
