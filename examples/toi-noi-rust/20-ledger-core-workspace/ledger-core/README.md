# ledger-core

Thư viện sổ cái double-entry trong bộ nhớ của **Tới Nơi** (dữ liệu hư cấu, dùng để học).
Mọi bút toán có tổng số tiền có dấu bằng 0, ghi idempotent theo `idempotency_key`,
sửa sai bằng bút toán đảo. Tiền là `Money(i64)` đơn vị đồng, không có float.

```rust
use ledger_core::{AccountId, JournalEntryBuilder, Ledger, Money, default_policy, FeePolicy};

// Cước 25.500 VND, phí nền tảng 20%.
let split = default_policy().split(Money::from_dong(25_500))?;
let entry = JournalEntryBuilder::new("trip:TN-0001:capture")
    .posting(AccountId::new("customer:CU-001")?, Money::from_dong(-25_500))
    .posting(AccountId::new("platform:revenue")?, split.platform_fee)
    .posting(AccountId::new("driver:DR-001")?, split.driver_net)
    .build()?;

let mut ledger = Ledger::new();
assert_eq!(ledger.post(entry)?.id.to_string(), "JE-000001");
assert_eq!(ledger.grand_total()?, Money::ZERO);
# Ok::<(), ledger_core::LedgerError>(())
```

Chưa có: database, mạng, async, ngày giờ trên bút toán, nhiều đơn vị tiền.
