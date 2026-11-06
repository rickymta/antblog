//! Strategy: cách proptest sinh dữ liệu ngẫu nhiên cho sổ cái.

use proptest::prelude::*;

use crate::account::AccountId;
use crate::entry::Posting;
use crate::money::Money;

/// Biên độ một posting sinh ra: ±1.000 tỷ đồng. Đủ lớn để thử số lớn,
/// đủ nhỏ để cộng tối đa 8 posting không tràn `i64`.
pub const MAX_AMOUNT: i64 = 1_000_000_000_000;

/// Bốn tài khoản của Tới Nơi; ít tài khoản để các posting dồn vào cùng một số dư.
pub const ACCOUNTS: [&str; 4] = [
    "customer:CU-001",
    "driver:DR-001",
    "platform:revenue",
    "psp:vixanh:clearing",
];

/// Một trong bốn tài khoản, chọn đều.
pub fn account() -> impl Strategy<Value = AccountId> {
    prop::sample::select(ACCOUNTS.to_vec())
        .prop_map(|raw| AccountId::new(raw).expect("mã tài khoản cố định hợp lệ"))
}

/// Số tiền có dấu trong [-MAX_AMOUNT, MAX_AMOUNT].
pub fn amount() -> impl Strategy<Value = i64> {
    -MAX_AMOUNT..=MAX_AMOUNT
}

/// Danh sách posting LUÔN cân: sinh n posting ngẫu nhiên (2..=8), rồi ép posting cuối
/// bằng -(tổng n-1 posting đầu). Cân do cách xây, không do thử rồi loại.
pub fn balanced_postings() -> impl Strategy<Value = Vec<Posting>> {
    prop::collection::vec((account(), amount()), 2..=8)
        .prop_map(|mut lines| {
            let last = lines.len() - 1;
            let sum: i64 = lines[..last].iter().map(|(_, dong)| *dong).sum();
            lines[last].1 = -sum;
            lines
                .into_iter()
                .map(|(account, dong)| Posting::new(account, Money::from_dong(dong)))
                .collect::<Vec<Posting>>()
        })
        // Loại bút toán toàn số 0 (hợp lệ nhưng vô nghĩa để thử).
        .prop_filter("có ít nhất một posting khác 0", |postings| {
            postings.iter().any(|p| p.amount != Money::ZERO)
        })
}

/// Dãy 0..=12 bút toán cân; test tự gán key `k-0`, `k-1`, ...
pub fn entry_sequence() -> impl Strategy<Value = Vec<Vec<Posting>>> {
    prop::collection::vec(balanced_postings(), 0..=12)
}

/// Tổng tiền có thiên hướng về biên. `any::<i64>()` chia đều trên 2^64 giá trị nên gần như
/// không bao giờ ra số nhỏ như 25.500 hay đúng `i64::MIN`; `prop_oneof!` trộn thêm các nhóm đó.
pub fn total_dong() -> impl Strategy<Value = i64> {
    prop_oneof![
        3 => -100_000i64..=100_000, // cỡ tiền thật của một chuyến
        2 => any::<i64>(),          // mọi độ lớn
        1 => Just(i64::MIN),
        1 => Just(i64::MAX),
    ]
}

/// Đầu vào của `split`: tổng bất kỳ (kể cả âm, `i64::MIN`) và 1..=8 trọng số có ít nhất một số khác 0.
pub fn split_input() -> impl Strategy<Value = (i64, Vec<u32>)> {
    (
        total_dong(),
        prop::collection::vec(0u32..=1_000, 1..=8)
            .prop_filter("tổng trọng số > 0", |w| w.iter().any(|&x| x > 0)),
    )
}
