//! Dữ liệu mẫu của Tới Nơi: các bút toán theo thứ tự thời gian, thời điểm cố định (không đọc đồng hồ).

use std::collections::HashMap;

use anyhow::{Result, anyhow};
use ledger_core::{AccountId, JournalEntry, JournalEntryBuilder, Money};
use sqlx::PgPool;

use crate::log::append_entry;

/// Một bước của kịch bản: bút toán, thời điểm ghi sổ và (nếu là bút toán đảo) key của bút toán bị đảo.
pub struct Step {
    /// `idempotency_key`.
    pub key: &'static str,
    /// Thời điểm ghi sổ, ISO 8601 UTC.
    pub posted_at: &'static str,
    /// `(account_id, amount_dong)`.
    pub postings: &'static [(&'static str, i64)],
    /// Key của bút toán mà bước này đảo.
    pub reverses: Option<&'static str>,
}

/// Tám bút toán của ba chuyến. `TN-0002` được đảo toàn phần như bài 43 (đảo `split` rồi `capture`,
/// mỗi bút toán đảo đổi dấu đúng các dòng của bút toán gốc); thanh toán `TN-0002` lúc 17:30 UTC là
/// 00:30 ngày hôm sau theo giờ Việt Nam (chỗ phiên bản 1 và 2 của thu nhập theo ngày khác nhau).
pub const STANDARD: [Step; 8] = [
    Step {
        key: "trip:TN-0001:capture",
        posted_at: "2026-11-03T09:00:00Z",
        postings: &[
            ("customer:CU-001", -25_500),
            ("psp:vixanh:clearing", 25_500),
        ],
        reverses: None,
    },
    Step {
        key: "trip:TN-0001:split",
        posted_at: "2026-11-03T09:00:30Z",
        postings: &[
            ("psp:vixanh:clearing", -25_500),
            ("platform:revenue", 5_100),
            ("driver:DR-001", 20_400),
        ],
        reverses: None,
    },
    Step {
        key: "trip:TN-0002:capture",
        posted_at: "2026-11-03T17:29:00Z",
        postings: &[
            ("customer:CU-002", -16_500),
            ("psp:vixanh:clearing", 16_500),
        ],
        reverses: None,
    },
    Step {
        key: "trip:TN-0002:split",
        posted_at: "2026-11-03T17:30:00Z",
        postings: &[
            ("psp:vixanh:clearing", -16_500),
            ("platform:revenue", 3_300),
            ("driver:DR-001", 13_200),
        ],
        reverses: None,
    },
    Step {
        key: "trip:TN-0002:split:reversal",
        posted_at: "2026-11-04T02:00:00Z",
        postings: &[
            ("psp:vixanh:clearing", 16_500),
            ("platform:revenue", -3_300),
            ("driver:DR-001", -13_200),
        ],
        reverses: Some("trip:TN-0002:split"),
    },
    Step {
        key: "trip:TN-0002:capture:reversal",
        posted_at: "2026-11-04T02:00:30Z",
        postings: &[
            ("customer:CU-002", 16_500),
            ("psp:vixanh:clearing", -16_500),
        ],
        reverses: Some("trip:TN-0002:capture"),
    },
    Step {
        key: "trip:TN-0003:capture",
        posted_at: "2026-11-04T03:00:00Z",
        postings: &[
            ("customer:CU-003", -12_000),
            ("psp:vixanh:clearing", 12_000),
        ],
        reverses: None,
    },
    Step {
        key: "trip:TN-0003:split",
        posted_at: "2026-11-04T03:00:30Z",
        postings: &[
            ("psp:vixanh:clearing", -12_000),
            ("platform:revenue", 2_400),
            ("driver:DR-002", 9_600),
        ],
        reverses: None,
    },
];

/// Chuyến thứ tư, ghi thêm khi projection đang dựng lại.
pub const TN_0004: [Step; 2] = [
    Step {
        key: "trip:TN-0004:capture",
        posted_at: "2026-11-04T05:00:00Z",
        postings: &[
            ("customer:CU-001", -25_500),
            ("psp:vixanh:clearing", 25_500),
        ],
        reverses: None,
    },
    Step {
        key: "trip:TN-0004:split",
        posted_at: "2026-11-04T05:00:30Z",
        postings: &[
            ("psp:vixanh:clearing", -25_500),
            ("platform:revenue", 5_100),
            ("driver:DR-002", 20_400),
        ],
        reverses: None,
    },
];

/// Dựng `JournalEntry` của `ledger-core` từ một bước (kiểm bất biến tổng bằng 0).
///
/// # Errors
/// Mã tài khoản rỗng hoặc bút toán không cân.
pub fn entry_of(step: &Step) -> Result<JournalEntry> {
    let mut builder = JournalEntryBuilder::new(step.key);
    for (account, amount) in step.postings {
        builder = builder.posting(AccountId::new(account)?, Money::from_dong(*amount));
    }
    Ok(builder.build()?)
}

/// Ghi lần lượt từng bước, mỗi bước một transaction riêng. Trả `key -> id` đã cấp.
///
/// # Errors
/// Lỗi SQL hoặc bút toán đảo trỏ tới key chưa ghi.
pub async fn append_steps(pool: &PgPool, steps: &[Step]) -> Result<HashMap<&'static str, i64>> {
    let mut ids: HashMap<&'static str, i64> = HashMap::new();
    for step in steps {
        let reversal_of = match step.reverses {
            Some(key) => Some(
                *ids.get(key)
                    .ok_or_else(|| anyhow!("bút toán {key} chưa được ghi"))?,
            ),
            None => None,
        };
        let entry = entry_of(step)?;
        let mut tx = pool.begin().await?;
        let id = append_entry(&mut tx, &entry, step.posted_at, reversal_of).await?;
        tx.commit().await?;
        ids.insert(step.key, id);
    }
    Ok(ids)
}
