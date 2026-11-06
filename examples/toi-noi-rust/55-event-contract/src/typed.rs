//! Kiểu có kiểu sinh từ proto v1 và bút toán mẫu của chuyến `TN-0001`.
use prost::Message;

/// Code sinh bởi prost-build từ `schemas/ledger_entry_posted.v1.proto`.
pub mod v1 {
    include!(concat!(env!("OUT_DIR"), "/toinoi.events.v1.rs"));
}

/// 2026-11-06T02:30:00Z tính bằng giây từ epoch (giờ nghiệp vụ truyền vào, không đọc đồng hồ).
pub const POSTED_AT_SECONDS: i64 = 1_793_932_200;

/// Dựng sự kiện từ các dòng (tài khoản, số tiền có dấu). Dùng chung cho mọi mẫu.
pub fn entry_posted(
    entry_id: &str,
    idempotency_key: &str,
    postings: &[(&str, i64)],
    posted_at_seconds: i64,
) -> v1::LedgerEntryPosted {
    v1::LedgerEntryPosted {
        schema_version: 1,
        entry_id: entry_id.to_owned(),
        idempotency_key: idempotency_key.to_owned(),
        postings: postings
            .iter()
            .map(|(account, amount)| v1::Posting {
                account: (*account).to_owned(),
                amount_dong: *amount,
            })
            .collect(),
        posted_at: Some(prost_types::Timestamp {
            seconds: posted_at_seconds,
            nanos: 0,
        }),
    }
}

/// Bút toán capture của `TN-0001`: khách -25.500, clearing +25.500.
pub fn sample_capture() -> v1::LedgerEntryPosted {
    entry_posted(
        "JE-000001",
        "trip:TN-0001:capture",
        &[
            ("customer:CU-001", -25_500),
            ("psp:vixanh:clearing", 25_500),
        ],
        POSTED_AT_SECONDS,
    )
}

/// Cùng bút toán nhưng số tiền 2^53 + 1: bảo đảm protobuf nhị phân giữ đúng, JSON Schema thì từ chối.
pub fn sample_big() -> v1::LedgerEntryPosted {
    entry_posted(
        "JE-000001",
        "trip:TN-0001:capture",
        &[
            ("customer:CU-001", -BIG_AMOUNT),
            ("psp:vixanh:clearing", BIG_AMOUNT),
        ],
        POSTED_AT_SECONDS,
    )
}

/// 2^53 + 1 = 9.007.199.254.740.993: số nguyên đầu tiên mà `f64` không biểu diễn được.
pub const BIG_AMOUNT: i64 = 9_007_199_254_740_993;

/// Mã hóa nhị phân.
pub fn encode(event: &v1::LedgerEntryPosted) -> Vec<u8> {
    event.encode_to_vec()
}

/// Giải mã nhị phân (lỗi là `Err`, không panic).
pub fn decode(bytes: &[u8]) -> Result<v1::LedgerEntryPosted, prost::DecodeError> {
    v1::LedgerEntryPosted::decode(bytes)
}
