---
slug: rust-vi-tai-xe-chia-tien-payout-theo-lo
title: "Ví tài xế: chia tiền, thu nhập và payout theo lô"
summary: "Bài 44: thu nhập theo ngày giờ Việt Nam khi chuyến qua nửa đêm UTC, payout theo lô với bốn trạng thái và bút toán đảo, hạn mức rút theo clock truyền vào, SKIP LOCKED, ngân hàng giả timeout, đối chiếu sao kê, trên PostgreSQL."
category: rust-advanced
level: advanced
tags: [rust, toi-noi, ledger, payout, postgresql, jiff]
language: vi
---

Cuối ngày 06/11, ứng dụng của tài xế `DR-001` hiện thu nhập hôm nay là 61.200 đồng, còn báo cáo gom theo ngày UTC ghi 20.400. Cả hai đều đúng số học: trong ba chuyến sau nửa đêm giờ Việt Nam, hai chuyến vẫn thuộc ngày 05/11 của UTC. Cùng đêm ấy nền tảng chi tiền ví về ngân hàng theo lô, và ngân hàng có thể nhận lệnh rồi không kịp trả lời.

Đây là bài 44 của lộ trình **Rust nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Bài dùng dòng tiền hai bước ([bài 13](/articles/rust-double-entry-journal-entry-posting)), schema và idempotency của [bài 24](/articles/rust-transaction-isolation-append-only-idempotency-key) và [bài 25](/articles/rust-schema-so-cai-accounts-journal-postings-partition), và `SKIP LOCKED` đã gặp ở [bài 37](/articles/rust-outbox-inbox-idempotent-consumer). [Bài 43](/articles/rust-refund-reversal-chargeback-but-toan-dao-maker-checker) đảo bút toán trong sổ; bài này chi tiền ra ngoài sổ, nơi lệnh đã tới ngân hàng thì không đảo được.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Ta nhờ ngân hàng chuyển 102.000 đồng cho <code>DR-001</code> và hết thời gian chờ mà không có phản hồi. Nên hoàn tiền về ví, hay gửi lại bằng một mã tham chiếu mới?</p>
<details><summary>Xem đáp án</summary><p>Cả hai đều có thể chi hai lần. Timeout là «không biết», không phải «thất bại»: hoàn tiền về ví trong khi ngân hàng đã nhận lệnh thì tài xế có tiền ở cả hai nơi, còn mã mới là lệnh thứ hai. Cách đúng là giữ khoản ở trạng thái chờ và tra cứu theo đúng mã cũ; chỉ khi ngân hàng trả lời chưa có lệnh mới gửi lại, vẫn mã cũ. Lab đếm số lệnh phía ngân hàng để chứng minh.</p></details>
</div>

## Thu nhập theo ngày: nửa đêm của tài xế không phải nửa đêm UTC

Ví `driver:DR-001` là tài khoản liability, số dư là tổng posting. Mỗi chuyến ghi hai bút toán; phần chia dùng `FeePolicy` của [bài 11](/articles/rust-trait-generics-chinh-sach-phi-lam-tron), nên cước 25.500 tách thành 5.100 cho nền tảng và 20.400 cho tài xế:

```rust
// src/fixture.rs (rút gọn: bỏ bút toán capture, chú thích tài liệu)
let fare = fare_for_km(5);
let split = default_policy().split(fare)?;
let neg_fare = Money::from_dong(-fare.dong());
let split_entry = JournalEntryBuilder::new(&format!("trip:{trip}:split"))
    .posting(acct("psp:vixanh:clearing")?, neg_fare)
    .posting(acct("platform:revenue")?, split.platform_fee)
    .posting(acct(&format!("driver:{driver}"))?, split.driver_net)
    .build()?;
```

Thu nhập là các posting dương của `trip:*:split`, nên khoản rút và khoản đảo của payout không lẫn vào. `posted_at` là `timestamptz`, một thời điểm tuyệt đối; múi giờ chỉ xuất hiện lúc cắt ngày, bằng `AT TIME ZONE` trong SQL:

```rust
// src/earnings.rs (rút gọn: gộp dòng SQL, bỏ phần ánh xạ hàng sang DayEarning)
let rows = sqlx::query(
    "SELECT to_char(e.posted_at AT TIME ZONE $2, 'YYYY-MM-DD') AS day, \
            count(*) AS trips, SUM(p.amount_dong)::bigint AS earned \
       FROM postings p JOIN journal_entries e ON e.id = p.entry_id \
      WHERE p.account_id = $1 AND p.amount_dong > 0 \
        AND e.idempotency_key LIKE 'trip:%:split' \
      GROUP BY day ORDER BY day",
)
.bind(format!("driver:{driver}"))
.bind(tz)
.fetch_all(pool)
.await?;
```

Năm chuyến của `DR-001`, hai chuyến cách nhau một giây quanh nửa đêm giờ Việt Nam (16:59:59Z và 17:00:00Z). Cùng truy vấn, chỉ đổi múi giờ:

```text
-- cắt ngày theo Asia/Ho_Chi_Minh
2026-11-05  2 chuyến  40.800 VND
2026-11-06  3 chuyến  61.200 VND
-- cắt ngày theo UTC
2026-11-05  4 chuyến  81.600 VND
2026-11-06  1 chuyến  20.400 VND
```

Hạn mức rút cũng tính theo ngày của tài xế, nên Rust phải biết ngày nào. `jiff` ([bài 8](/articles/rust-string-str-ngay-gio-dinh-dang-tien)) làm việc đó và lab kiểm nó khớp PostgreSQL ở bốn mốc sát nửa đêm. Cửa sổ ngày là nửa mở `[đầu ngày, đầu ngày sau)` và dùng `start_of_day` thay vì cộng 24 giờ:

```rust
// src/clock.rs
pub fn local_day_bounds(at: Timestamp) -> Result<(Timestamp, Timestamp)> {
    let start = at.in_tz(LOCAL_TZ)?.start_of_day()?;
    let end = start.tomorrow()?.start_of_day()?;
    Ok((start.timestamp(), end.timestamp()))
}
```

```text
-- mốc sát nửa đêm (UTC, ngày UTC, ngày PostgreSQL, ngày jiff)
2026-11-05T16:59:59Z  2026-11-05  2026-11-05  2026-11-05
2026-11-05T17:00:00Z  2026-11-05  2026-11-06  2026-11-06
2026-11-06T16:59:59Z  2026-11-06  2026-11-06  2026-11-06
2026-11-06T17:00:00Z  2026-11-06  2026-11-07  2026-11-07
cửa sổ ngày của 2026-11-06T10:00:00Z: [2026-11-05T17:00:00Z, 2026-11-06T17:00:00Z)
```

Đúng 17:00:00Z thuộc ngày mới, 16:59:59Z thuộc ngày cũ: không thời điểm nào rơi vào hai ngày hay lọt khe.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 276" role="img" aria-label="Hình 1. Năm chuyến của DR-001 trên một trục UTC. Cắt ngày theo UTC cho 05/11 bốn chuyến và 06/11 một chuyến; cắt theo giờ Việt Nam, nửa đêm rơi vào 17:00Z, cho 05/11 hai chuyến và 06/11 ba chuyến."><defs><marker id="rust-vi-tai-xe-chia-tien-payout-theo-lo-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><text x="40" y="24" class="t-b">Ngày theo UTC</text><rect x="40" y="34" width="640" height="44" rx="8" class="f-phys-soft s-phys"/><text x="56" y="61" class="t-b">05/11 · 4 chuyến · 81.600</text><rect x="680" y="34" width="160" height="44" rx="8" class="f-warn-soft s-warn"/><text x="694" y="53" class="t-b">06/11 · 1 chuyến</text><text x="694" y="70" class="t-small">20.400</text><path d="M40 126 L840 126" class="f-none s-line" stroke-width="2"/><path d="M400 80 L400 216" class="f-none s-muted dash" stroke-width="2"/><path d="M680 80 L680 216" class="f-none s-muted dash" stroke-width="2"/><circle cx="80" cy="126" r="6" class="f-pk s-pk"/><text x="80" y="100" text-anchor="middle" class="t-b">TN-0001</text><text x="80" y="114" text-anchor="middle" class="t-small">09:00:00Z</text><circle cx="394" cy="126" r="6" class="f-pk s-pk"/><text x="384" y="100" text-anchor="end" class="t-b">TN-0002</text><text x="384" y="114" text-anchor="end" class="t-small">16:59:59Z</text><circle cx="406" cy="126" r="6" class="f-pk s-pk"/><text x="416" y="100" text-anchor="start" class="t-b">TN-0003</text><text x="416" y="114" text-anchor="start" class="t-small">17:00:00Z</text><circle cx="460.0" cy="126" r="6" class="f-pk s-pk"/><text x="460.0" y="146" text-anchor="middle" class="t-b">TN-0004</text><text x="460.0" y="160" text-anchor="middle" class="t-small">18:30:00Z</text><circle cx="760" cy="126" r="6" class="f-pk s-pk"/><text x="760" y="100" text-anchor="middle" class="t-b">TN-0005</text><text x="760" y="114" text-anchor="middle" class="t-small">02:00:00Z</text><text x="392" y="178" text-anchor="end" class="t-small">00:00 giờ Việt Nam = 17:00Z</text><text x="672" y="178" text-anchor="end" class="t-small">00:00Z</text><text x="40" y="206" class="t-b">Ngày theo giờ Việt Nam</text><rect x="40" y="216" width="360" height="44" rx="8" class="f-phys-soft s-phys"/><text x="56" y="243" class="t-b">05/11 · 2 chuyến · 40.800</text><rect x="400" y="216" width="440" height="44" rx="8" class="f-pk-soft s-pk"/><text x="416" y="243" class="t-b">06/11 · 3 chuyến · 61.200</text></svg>
</div>
<figcaption>Hình 1. Cùng năm chuyến, hai cách cắt ngày. Vạch nét đứt là hai nửa đêm: 17:00Z là 00:00 giờ Việt Nam, 00:00Z là nửa đêm UTC.</figcaption>
</figure>

## Payout là một bút toán, không phải một lần gọi API

Khi gom ví vào lô, tiền rời ví ngay: `driver:<id>` âm, `bank:payout:clearing` dương (asset, tiền đang chi ra ngân hàng, xem [bài 41](/articles/rust-bai-toan-so-cai-chart-of-accounts-dong-tien-chuyen)). Không chờ ngân hàng trả lời mới trừ ví, vì trong lúc chờ ví đó có thể bị chi lần nữa.

Mỗi lô có một dòng `payout_batches`; mỗi tài xế trong lô có một dòng `payouts` mang `payout_key = payout:<mã lô>:<tài xế>`. Chuỗi này vừa là `idempotency_key` của bút toán, vừa là mã tham chiếu gửi ngân hàng, vừa là cột unique. Thêm một chỉ mục unique một phần theo kỳ:

```sql
-- sql/schema.sql (trích)
CREATE UNIQUE INDEX payouts_period_driver_uq ON payouts (period, driver_id) WHERE status <> 'FAILED';
```

Mỗi tài xế nhiều nhất một khoản còn hiệu lực trong một kỳ; khoản `FAILED` đã đảo nên không chiếm chỗ. Trạng thái lô suy ra từ các khoản chứ không lưu độc lập.

```rust
// src/payout.rs
pub fn can_go_to(self, next: Status) -> bool {
    matches!(
        (self, next),
        (Status::Created, Status::Sent | Status::Failed)
            | (Status::Sent, Status::Settled | Status::Failed)
    )
}

pub fn derive_batch_status(c: Counts) -> Status {
    if c.created > 0 {
        Status::Created
    } else if c.sent > 0 {
        Status::Sent
    } else if c.settled == 0 && c.failed > 0 {
        Status::Failed
    } else {
        Status::Settled
    }
}
```

Mỗi lần đổi trạng thái là `UPDATE ... WHERE id = $1 AND status = $2`: hai tiến trình cùng đổi một khoản thì một bên thấy 0 hàng được cập nhật và bỏ qua.

`SENT` chỉ có nghĩa ngân hàng đã nhận lệnh. Trạng thái này không biết kết quả xử lý: tiền có thể đã tới, còn đang chờ, hoặc sẽ bị trả lại. Kết cục đến từ sao kê, nên `SENT` chưa phải «đã chi xong».

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 304" role="img" aria-label="Hình 2. Vòng đời một khoản payout: CREATED sang SENT rồi SETTLED, hoặc sang FAILED từ CREATED khi ngân hàng từ chối và từ SENT khi sao kê báo bị trả lại; timeout để khoản ở lại CREATED và tra cứu trước khi gửi lại. SENT chưa biết kết quả xử lý. Trạng thái lô lấy theo khoản đi chậm nhất."><defs><marker id="rust-vi-tai-xe-chia-tien-payout-theo-lo-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><text x="20" y="16" class="t-small">timeout: vẫn CREATED, tra cứu trước khi gửi lại</text><path d="M60 50 C60 24 130 24 130 48" class="f-none s-warn dash" stroke-width="2" marker-end="url(#rust-vi-tai-xe-chia-tien-payout-theo-lo-1-a)"/><rect x="20" y="50" width="150" height="60" rx="10" class="f-phys-soft s-phys"/><text x="36" y="76" class="t-b">CREATED</text><text x="36" y="96" class="t-small">đã trừ ví, chờ bank</text><rect x="290" y="50" width="150" height="60" rx="10" class="f-phys-soft s-phys"/><text x="306" y="76" class="t-b">SENT</text><text x="306" y="96" class="t-small">chưa biết kết cục</text><rect x="560" y="50" width="150" height="60" rx="10" class="f-pk-soft s-pk"/><text x="576" y="76" class="t-b">SETTLED</text><text x="576" y="96" class="t-small">sao kê: tiền đã tới</text><path d="M170 80 L288 80" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-vi-tai-xe-chia-tien-payout-theo-lo-1-a)"/><text x="229" y="72" text-anchor="middle" class="t-small">bank nhận lệnh</text><path d="M440 80 L558 80" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-vi-tai-xe-chia-tien-payout-theo-lo-1-a)"/><text x="499" y="72" text-anchor="middle" class="t-small">sao kê: đã tới</text><rect x="290" y="170" width="150" height="60" rx="10" class="f-warn-soft s-warn"/><text x="306" y="196" class="t-b">FAILED</text><text x="306" y="216" class="t-small">đã đảo, tiền về ví</text><path d="M95 112 L95 200 L288 200" class="f-none s-warn" stroke-width="2" marker-end="url(#rust-vi-tai-xe-chia-tien-payout-theo-lo-1-a)"/><text x="104" y="162" class="t-small">bank từ chối</text><path d="M365 112 L365 168" class="f-none s-warn" stroke-width="2" marker-end="url(#rust-vi-tai-xe-chia-tien-payout-theo-lo-1-a)"/><text x="374" y="146" class="t-small">sao kê: bị trả lại</text><text x="460" y="196" class="t-small">bút toán đảo: driver:&lt;id&gt; dương, clearing âm,</text><text x="460" y="214" class="t-small">reversal_of trỏ về bút toán trừ ví</text><rect x="20" y="244" width="840" height="52" rx="10" class="f-none s-line"/><text x="36" y="265" class="t-b">Trạng thái lô = khoản đi chậm nhất</text><text x="36" y="285" class="t-small">còn CREATED → CREATED · còn SENT → SENT · mọi khoản có kết cục → SETTLED · mọi khoản FAILED → FAILED</text></svg>
</div>
<figcaption>Hình 2. Vòng đời một khoản payout. Mũi tên viền cam là đường thất bại; trạng thái lô lấy theo khoản đi chậm nhất.</figcaption>
</figure>

## Chọn ví vào lô: ngưỡng, hạn mức và đồng hồ truyền vào

Chính sách của lab: ví đủ 50.000 mới vào lô, mỗi tài xế rút tối đa 150.000 mỗi ngày giờ Việt Nam. Khoản chi là số nhỏ hơn giữa số dư và hạn mức còn lại; nhỏ hơn ngưỡng thì ví bị bỏ qua kèm lý do.

```rust
// src/payout.rs, trong create_batch (rút gọn)
let used = used_in_window(&mut tx, &driver_id, day_start, day_end).await?;
let remaining = (request.policy.daily_limit.dong() - used).max(0);
let amount = balance.min(remaining);
if amount < request.policy.min_payout.dong() {
    skipped.push(Skipped { driver_id, balance_dong: balance, remaining_limit_dong: remaining });
    continue;
}
```

`create_batch` nhận `&impl Clock` và không đọc giờ hệ thống. `created_at` của khoản chi, `posted_at` của bút toán và cửa sổ ngày của hạn mức đều từ `clock.now()`. Test đẩy `FixedClock` qua nửa đêm giờ Việt Nam bằng một lệnh gán, không `sleep`:

```text
10:00:00Z  lô .1: DR-003 rút 150.000 VND
16:59:59Z  hạn mức còn lại của DR-003: 0 VND
16:59:59Z  lô .2: 0 khoản, bỏ qua [Skipped { driver_id: "DR-003", balance_dong: 94800, remaining_limit_dong: 0 }]
17:00:00Z  hạn mức còn lại của DR-003: 150.000 VND
17:00:00Z  lô 07/11: [("DR-003", 94800)]
nếu cắt ngày theo UTC, DR-003 đã dùng lúc 17:00:00Z: 150.000 VND
```

Ví `DR-003` có 244.800 nên lô đầu chỉ chi 150.000. Một giây trước nửa đêm giờ Việt Nam, hạn mức bằng 0 nên lô hai bỏ qua. Đúng 17:00:00Z hạn mức hồi lại và lô sau chi nốt 94.800. Cửa sổ cắt theo UTC sẽ bắt tài xế chờ tới 07:00 sáng.

## Hai lô chạy song song: SKIP LOCKED và unique theo kỳ

Cron bắn hai lần hay một người bấm chạy tay đều làm hai lô cùng kỳ chồng nhau. Lab dựng năm ví 61.200, mỗi lô lấy tối đa 3 ví, và cho hai lô cùng chờ ở một `Barrier` sau bước chọn, trước bước ghi. Có ba cách chạy; bước chọn của cách thứ ba là:

```rust
// src/payout.rs
const SELECT_LOCKED: &str = "SELECT substr(account_id, 8) AS driver_id, balance_dong \
     FROM account_balances \
     WHERE account_id LIKE 'driver:%' AND balance_dong >= $1 \
     ORDER BY account_id LIMIT $2 FOR UPDATE SKIP LOCKED";
```

```text
== 4. Hai lô song song, tối đa 3 ví mỗi lô: NaiveNoUnique ==
lô thứ nhất: chọn DR-001, DR-002, DR-003
lô thứ hai: chọn DR-001, DR-002, DR-003
ví                  : DR-001 -61.200 VND, DR-002 -61.200 VND, DR-003 -61.200 VND, DR-004 61.200 VND, DR-005 61.200 VND
== 4. Hai lô song song, tối đa 3 ví mỗi lô: NaiveUnique ==
lô thứ nhất: chọn DR-001, DR-002, DR-003
lô thứ hai: lỗi SQLSTATE 23505, rollback cả lô
ví                  : DR-001 0 VND, DR-002 0 VND, DR-003 0 VND, DR-004 61.200 VND, DR-005 61.200 VND
== 4. Hai lô song song, tối đa 3 ví mỗi lô: SkipLocked ==
lô thứ nhất: chọn DR-001, DR-002, DR-003
lô thứ hai: chọn DR-004, DR-005
ví                  : DR-001 0 VND, DR-002 0 VND, DR-003 0 VND, DR-004 0 VND, DR-005 0 VND
```

`SELECT` thường, không chỉ mục unique: hai lô chi trùng, ba ví âm 61.200. Có chỉ mục unique theo kỳ: lô thua nhận 23505 và rollback toàn bộ, sổ đúng nhưng một lô bị bỏ phí. `FOR UPDATE SKIP LOCKED`: lô sau bỏ qua ví đang khoá và lấy phần còn lại, không trùng, ví nào cũng về 0.

Lô nào lấy ví nào là may rủi, hai tập còn có thể xen kẽ; khối trên chỉ là một lần chạy. Test chỉ kiểm bất biến: hai tập rời nhau, hợp lại đủ năm ví, kích thước 2 và 3, mọi ví về 0. Khoá trên hàng ví còn khiến việc kiểm hạn mức rồi ghi của một tài xế không thể xen với lô khác. Chỉ mục unique vẫn giữ làm lưới thứ hai cho đường ghi không đi qua khoá, như một script SQL chạy tay.

## Timeout sau khi ngân hàng đã nhận lệnh

Ngân hàng nằm sau một trait, trả `impl Future<...> + Send` để gọi được trong `tokio::spawn` mà không cần crate `async-trait`:

```rust
// src/bank.rs (rút gọn: bỏ chú thích tài liệu)
pub enum BankError {
    Rejected(String), // từ chối dứt khoát: tiền chưa đi đâu
    Timeout,          // không biết ngân hàng đã nhận hay chưa
}

pub trait Bank {
    fn transfer(&self, request: &TransferRequest)
        -> impl Future<Output = Result<Transfer, BankError>> + Send;
    // Tra cứu theo mã, không chi gì. Ok(None): ngân hàng chưa có lệnh này.
    fn lookup(&self, reference: &str)
        -> impl Future<Output = Result<Option<Transfer>, BankError>> + Send;
    fn statement(&self) -> impl Future<Output = Result<Vec<StatementLine>, BankError>> + Send;
}
```

`FakeBank` giữ các lệnh đã nhận trong bộ nhớ, với một `Behavior` cho từng tài xế: `Accept`, `Reject`, `ReturnedLater` (nhận rồi trả tiền lại, hiện trên sao kê), `TimeoutOnce` (ghi nhận lệnh rồi mới trả `Timeout`) và `LostOnce` (trả `Timeout`, không ghi nhận gì). Từ phía ta, hai ca cuối giống hệt nhau; chỉ tra cứu mới phân biệt được.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 3. Timeout sau khi ngân hàng đã nhận lệnh: payout service ghi bút toán và gửi lệnh, ngân hàng ghi nhận BK-00001 nhưng phản hồi không về; khoản ở lại CREATED, lượt sau tra cứu theo cùng reference, thấy lệnh đã có nên sang SENT mà không gọi transfer lần hai."><defs><marker id="rust-vi-tai-xe-chia-tien-payout-theo-lo-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><text x="340" y="20" text-anchor="middle" class="t-b">Payout service</text><text x="720" y="20" text-anchor="middle" class="t-b">Ngân hàng</text><path d="M340 28 L340 292" class="f-none s-muted dash" stroke-width="2"/><path d="M720 28 L720 292" class="f-none s-muted dash" stroke-width="2"/><rect x="20" y="34" width="310" height="48" rx="8" class="f-pk-soft s-pk"/><text x="34" y="54" class="t-b">1. COMMIT một transaction</text><text x="34" y="72" class="t-small">ví −102.000 · clearing +102.000 · CREATED</text><path d="M340 102 L718 102" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-vi-tai-xe-chia-tien-payout-theo-lo-2-a)"/><text x="530" y="94" text-anchor="middle" class="t-small">transfer(reference = payout:PB-2026-11-06.1:DR-001)</text><rect x="734" y="108" width="136" height="38" rx="8" class="f-phys-soft s-phys"/><text x="746" y="124" class="t-small">đã nhận lệnh</text><text x="746" y="140" class="t-small">BK-00001</text><path d="M720 168 L342 168" class="f-none s-warn dash" stroke-width="2" marker-end="url(#rust-vi-tai-xe-chia-tien-payout-theo-lo-2-a)"/><text x="530" y="160" text-anchor="middle" class="t-small">phản hồi không tới nơi: Timeout</text><rect x="20" y="148" width="310" height="48" rx="8" class="f-warn-soft s-warn"/><text x="34" y="168" class="t-b">2. Không biết tiền đã đi chưa</text><text x="34" y="186" class="t-small">vẫn CREATED, attempts = 1, không đảo</text><path d="M340 226 L718 226" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-vi-tai-xe-chia-tien-payout-theo-lo-2-a)"/><text x="530" y="218" text-anchor="middle" class="t-small">lookup(cùng reference), không chi gì</text><path d="M720 254 L342 254" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-vi-tai-xe-chia-tien-payout-theo-lo-2-a)"/><text x="530" y="246" text-anchor="middle" class="t-small">Some(BK-00001): lệnh đã có, không gọi transfer lần nữa</text><rect x="20" y="236" width="310" height="48" rx="8" class="f-pk-soft s-pk"/><text x="34" y="256" class="t-b">3. SENT, bank_ref = BK-00001</text><text x="34" y="274" class="t-small">transfer chỉ gọi 1 lần; kết cục chờ sao kê</text></svg>
</div>
<figcaption>Hình 3. Timeout sau khi ngân hàng đã nhận lệnh: tra cứu theo cùng reference thấy lệnh cũ, nên không gọi <code>transfer</code> lần hai.</figcaption>
</figure>

```rust
// src/payout.rs, trong send_batch (rút gọn: bỏ dựng `request`, gộp dòng)
if payout.attempts > 0 {
    match bank.lookup(&payout.payout_key).await {
        Ok(Some(found)) => {
            let done = advance(pool, clock.now(), payout.id, Status::Created, Status::Sent,
                Some(&found.bank_ref)).await?;
            if done { report.sent += 1; }
            continue;
        }
        Ok(None) => {} // ngân hàng trả lời chưa có lệnh: gửi lại đúng mã cũ
        Err(_) => { report.timed_out += 1; continue; } // vẫn không biết, không gửi
    }
}
sqlx::query("UPDATE payouts SET attempts = attempts + 1 WHERE id = $1")
    .bind(payout.id).execute(pool).await?;
match bank.transfer(&request).await {
    Ok(transfer) => { /* advance như trên, với transfer.bank_ref */ }
    Err(BankError::Rejected(reason)) => {
        if fail_payout(pool, clock.now(), payout.id, &reason).await? { report.rejected += 1; }
    }
    Err(BankError::Timeout) => {
        sqlx::query("UPDATE payouts SET last_error = 'timeout' WHERE id = $1")
            .bind(payout.id).execute(pool).await?;
        report.timed_out += 1;
    }
}
```

Vì lệnh có thể đã tới nơi, gặp `Timeout` thì `send_batch` không đảo, không đổi mã và không tự chi lại. `attempts` được ghi trước khi gọi, nên `CREATED` với `attempts > 0` nghĩa là «đã từng gửi, chưa rõ kết cục», kể cả khi tiến trình chết giữa chừng. Lượt sau phải tra cứu: thấy lệnh thì sang `SENT` mà không gọi `transfer`; tra cứu lỗi thì giữ nguyên; ngân hàng trả lời chưa có lệnh mới được gửi lại, vẫn đúng `payout_key`. Mã mới là lệnh chi thứ hai.

```text
lượt 1: timeout 1, khoản CREATED, 1 lần gửi, last_error Some("timeout")
ví DR-001 sau lượt 1   : 0 VND
ngân hàng đã nhận      : 1 lệnh
lượt 2 (tra cứu lỗi)   : SendReport { sent: 0, rejected: 0, timed_out: 1 }, khoản CREATED, transfer gọi 1 lần
lượt 3 (tra cứu thấy)  : SendReport { sent: 1, rejected: 0, timed_out: 0 }, khoản SENT, 1 lần gửi, bank_ref Some("BK-00001")
ngân hàng đã nhận      : 1 lệnh, transfer gọi 1 lần
retry bằng mã mới      : ngân hàng nhận 2 lệnh cho cùng một tài xế
== 5b. Lệnh mất trước khi tới ngân hàng ==
lượt 1                 : SendReport { sent: 0, rejected: 0, timed_out: 1 }, ngân hàng đã nhận 0 lệnh
lượt 2 (tra cứu: chưa có): SendReport { sent: 1, rejected: 0, timed_out: 0 }, khoản SENT, 2 lần gửi, cùng mã payout:PB-2026-11-06.1:DR-001
ngân hàng đã nhận      : 1 lệnh
```

`Ok(None)` ở đây tin được vì `FakeBank` trả lời tức thời và idempotent theo `reference`. Ngân hàng thật có thể chưa kịp hiện lệnh đang xử lý; gửi lại chỉ an toàn khi họ cam kết idempotency theo mã, nếu không thì chuyển điều tra tay.

## Thất bại và sao kê: bút toán đảo, rồi đối chiếu

Ngân hàng từ chối dứt khoát (`Rejected`) thì tiền chưa đi đâu: khoản về `FAILED` kèm bút toán đảo trả tiền về ví, cùng một transaction. `JournalEntry::reversal_of` đổi dấu mọi posting, key thành `<payout_key>:reversal`, cột `reversal_of` trỏ về bút toán đã trừ ví.

```rust
// src/payout.rs, trong fail_payout (rút gọn: bỏ bind, đọc cột, ghi số dư và trạng thái lô)
let row = sqlx::query(
    "UPDATE payouts SET status = 'FAILED', last_error = $2, updated_at = $3::timestamptz \
      WHERE id = $1 AND status IN ('CREATED', 'SENT') \
      RETURNING batch_id, driver_id, amount_dong, payout_key, entry_id",
)
// ...
let Some(row) = row else {
    tx.rollback().await?;
    return Ok(false); // đã kết thúc từ trước: không đảo lần hai
};
// ...
let original = payout_entry(&key, &driver_id, amount)?;
let reversal = JournalEntry::reversal_of(original_id, &original)?;
let reversal_id = insert_entry(&mut tx, &reversal, now, Some(entry_id)).await?
    .ok_or_else(|| anyhow!("bút toán đảo của {key} đã có"))?;
```

Một lô bốn khoản, bốn hành vi ngân hàng: `DR-001` timeout, `DR-003` nhận, `DR-004` từ chối, `DR-005` nhận rồi trả lại.

```text
ví sau khi tạo lô       : DR-001 0 VND, DR-002 40.800 VND, DR-003 94.800 VND, DR-004 0 VND, DR-005 0 VND
bank:payout:clearing    : 394.800 VND
-- gửi lần 1: SendReport { sent: 2, rejected: 1, timed_out: 1 }
các khoản               : DR-001=CREATED DR-003=SENT DR-004=FAILED DR-005=SENT
trạng thái lô           : CREATED
ví DR-004 (bị từ chối)  : 61.200 VND
-- gửi lần 2: SendReport { sent: 1, rejected: 0, timed_out: 0 }
-- đối chiếu: settled 2, returned 1, lệch 0, thiếu 0, lạ 0
các khoản               : DR-001=SETTLED DR-003=SETTLED DR-004=FAILED DR-005=FAILED
trạng thái lô           : SETTLED
bank:payout:clearing    : 252.000 VND
tổng mọi posting        : 0
```

`DR-002` có 40.800, dưới ngưỡng, nên không vào lô. Lô `SETTLED` nghĩa là mọi khoản có kết cục, không phải mọi khoản thành công. Clearing cuối bằng 252.000, đúng tổng hai khoản `SETTLED`, và không về 0: sơ đồ tài khoản chuẩn chưa có `bank:cash` để tất toán, và lab không thêm tài khoản ngoài chuẩn. Lab còn kiểm `account_balances` bằng tổng posting của chính tài khoản đó.

`DR-005` là ca chỉ sao kê mới biết: ngân hàng báo nhận rồi trả tiền lại. `reconcile` xếp từng khoản vào một nhóm; chỉ nhóm có kết cục rõ ràng mới được tự sửa sổ.

| Dòng sao kê | Khoản của ta | Việc làm |
|---|---|---|
| khớp mã và tiền, đã tới | `SENT` | sang `SETTLED` |
| khớp mã và tiền, đã tới | `CREATED` (timeout) | sang `SENT` rồi `SETTLED`, ghi `recovered_after_timeout` |
| khớp mã, bị trả lại | `CREATED` hoặc `SENT` | sang `FAILED` kèm bút toán đảo |
| khớp mã, lệch tiền | bất kỳ | chỉ báo, chờ người xem |
| không có dòng | `SENT` | báo thiếu |
| không có dòng | `CREATED` | bỏ qua, lượt gửi sau tra cứu |
| có dòng mà ta không có mã | không có | báo lạ |

```text
== 6. Đối chiếu với sao kê có lệch ==
settled                : 2
  trong đó gỡ nút thắt timeout: ["payout:PB-2026-11-06.1:DR-002"]
returned (đã đảo)      : ["payout:PB-2026-11-06.1:DR-005"]
lệch số tiền           : [("payout:PB-2026-11-06.1:DR-003", 61200, 60000)]
thiếu trong sao kê     : ["payout:PB-2026-11-06.1:DR-004"]
dòng lạ trong sao kê   : ["payout:PB-2026-11-06.9:DR-999"]
các khoản              : DR-001=SETTLED DR-002=SETTLED DR-003=SENT DR-004=SENT DR-005=FAILED
trạng thái lô          : SENT
đối chiếu lần 2        : settled 0, returned 0, ví không đổi = true, bút toán đảo = 1
```

Số tiền lệch và dòng thiếu không bị tự sửa vì chưa biết bên nào sai. Đối chiếu lần hai không đổi gì. Sao kê CSV lớn và phân loại lệch đầy đủ là việc của [bài 47](/articles/rust-doi-soat-psp-sao-ke-csv-khop-lech-bao-cao).

## Idempotency theo mã lô và tài xế

Ba tầng chặn ba kiểu trùng khác nhau:

| Tầng | Chặn | Cơ chế |
|---|---|---|
| mã lô | chạy lại hoặc chạy đồng thời cùng một lô | `batch_code` unique, `ON CONFLICT DO NOTHING` |
| mã lô và tài xế | bút toán trừ ví và lệnh ngân hàng trùng | `payout_key` là `idempotency_key` và là `reference` |
| trạng thái | hai tiến trình cùng đổi một khoản | `UPDATE ... WHERE status = $from` |

```text
hai lệnh tạo lô cùng mã chạy đồng thời: fresh = 1, cùng lô = true, cùng khoản = true
khoản chi / bút toán payout:* : 4 / 4
chèn lại bút toán cùng key    : tạo bản mới = false
hai lượt gửi đồng thời        : tổng SENT = 4
lượt gửi thứ ba               : SendReport { sent: 0, rejected: 0, timed_out: 0 }
lệnh ngân hàng đã nhận        : DR-001=1 DR-003=1 DR-004=1 DR-005=1
```

Lệnh tạo lô sau chờ lệnh trước commit, thấy conflict và trả về đúng lô đã có. Hai lượt gửi đồng thời cùng gọi ngân hàng, nhưng ngân hàng trả lệnh cũ theo `reference` và chỉ một bên thắng lúc sang `SENT`, nên tổng là 4 chứ không phải 8. Cùng mô hình với [bài transaction và idempotency của loạt .NET](/articles/dotnet10-transaction-concurrency-idempotency): khoá do bên gọi đặt, bên nhận lưu và trả lại kết quả cũ.

## Chạy ví dụ và kiểm chứng

Cần PostgreSQL của `examples/toi-noi-rust/infra`. Từ gốc repository:

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
cargo run --manifest-path examples/toi-noi-rust/44-driver-wallet-payout/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/44-driver-wallet-payout/Cargo.toml
```

`--verify` in các khối kết quả ở trên (bài trích từng dòng nguyên văn) rồi kiểm từng giá trị; đạt thì in `VERIFY OK` và thoát mã 0. `cargo test` chạy 7 unit test và 10 test tích hợp, mỗi test một schema `lab_<uuid>` xoá khi xong. Lab dùng `sqlx` 0.9.0, `tokio` 1.54.1, `jiff` 0.2.39 và `ledger-core` của lab 20.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Ngân hàng là một struct trong bộ nhớ, không có phí chuyển khoản hay file lô. Chart chưa có <code>bank:cash</code> nên <code>bank:payout:clearing</code> không được tất toán khi khoản <code>SETTLED</code> và số dư chỉ tăng theo tổng đã chi; sổ thật còn một bút toán lúc ngân hàng trừ tiền. Thu nhập theo ngày chưa trừ khoản hoàn của bài 43. <code>send_batch</code> gọi ngân hàng tuần tự, chưa giành quyền gửi giữa hai lượt đồng thời (dựa vào <code>reference</code>), chưa có backoff hay hạn chót chuyển khoản timeout sang điều tra tay. Payout chưa có maker-checker. Số liệu và ngưỡng đều hư cấu.</p></div>

**Bài tập:** giả sử `send_batch` gọi `fail_payout` ngay khi `Timeout`. Với dữ liệu của lab, `DR-001` gặp chuyện gì ở các lô sau, và chỉ mục unique theo kỳ có cứu được không?

<details><summary>Xem lời giải</summary><p>Ngân hàng đã nhận lệnh 102.000 nên tiền tới tài xế, còn ví lại có thêm 102.000 do bút toán đảo. Ví vẫn đủ ngưỡng nên lô kỳ sau chi thêm 102.000: tài xế nhận 204.000 trên 102.000 thu nhập. Chỉ mục unique một phần chỉ chặn trong cùng kỳ, và khoản đã đảo bị loại khỏi chỉ mục. Sổ cũng lệch ngân hàng: ngân hàng có một lệnh mà sổ nói đã đảo. Vì vậy timeout giữ khoản ở <code>CREATED</code>, và chỉ kết quả tra cứu hoặc sao kê mới được quyết định bước tiếp theo.</p></details>

Bài trước: [bài 43](/articles/rust-refund-reversal-chargeback-but-toan-dao-maker-checker). Bài tiếp: [bài 45](/articles/rust-khuyen-mai-tro-gia-but-toan-nhieu-ben-ngan-sach).

## Tài liệu gốc

- [PostgreSQL: SELECT, mục The Locking Clause (`FOR UPDATE`, `SKIP LOCKED`)](https://www.postgresql.org/docs/current/sql-select.html#SQL-FOR-UPDATE-SHARE).
- [PostgreSQL: Date/Time Functions, mục `AT TIME ZONE`](https://www.postgresql.org/docs/current/functions-datetime.html#FUNCTIONS-DATETIME-ZONECONVERT).
- [PostgreSQL: Partial Indexes](https://www.postgresql.org/docs/current/indexes-partial.html).
- [docs.rs: jiff 0.2, `Zoned::start_of_day`](https://docs.rs/jiff/0.2.39/jiff/struct.Zoned.html#method.start_of_day).
- [docs.rs: tokio, `sync::Barrier`](https://docs.rs/tokio/1.54.1/tokio/sync/struct.Barrier.html).
- [Rust Reference: `impl Trait` types](https://doc.rust-lang.org/reference/types/impl-trait.html).
