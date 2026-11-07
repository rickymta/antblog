//! Dữ liệu mẫu nhỏ cho báo cáo đối soát (mã VíXanh hư cấu của Tới Nơi).

use std::collections::HashMap;

/// Sao kê mẫu: 1-n (`vx_ch_0002`), lệch tiền, thiếu bên ta, và ba dòng hỏng.
pub const STATEMENT: &str = "\
external_ref,amount_vnd,settled_at
vx_ch_0001,25500,2026-11-05T03:00:12Z
vx_ch_0002,25.000,2026-11-05T04:10:20Z
vx_ch_0002,15.000,2026-11-05T04:10:21Z
vx_ch_0003,31000,2026-11-05T05:20:10Z
vx_ch_0005,52.000 VND,2026-11-05T05:55:00Z
vx_ch_0008,12a00,2026-11-05T08:50:05Z
,9000,2026-11-05T10:00:00Z
vx_ch_0011,15000
";

/// Phía sổ cái mẫu: ref -> tổng tiền.
#[must_use]
pub fn ledger() -> HashMap<String, i64> {
    HashMap::from([
        ("vx_ch_0001".to_owned(), 25_500),
        ("vx_ch_0002".to_owned(), 40_000),
        ("vx_ch_0003".to_owned(), 30_000),
        ("vx_ch_0004".to_owned(), 18_000),
    ])
}
