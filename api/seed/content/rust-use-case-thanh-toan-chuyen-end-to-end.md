---
slug: rust-use-case-thanh-toan-chuyen-end-to-end
title: "Use case thanh toán chuyến end-to-end"
summary: "Bài 54: ghép P05 thành một luồng chạy thật trên PostgreSQL và Kafka. TripCompleted và PaymentCaptured đến lệch thứ tự vẫn cho ra cùng sổ hai bút toán; có khuyến mãi, hoàn một phần, abort giữa chừng, đối soát clearing."
category: rust-advanced
level: advanced
tags: [rust, toi-noi, ledger, use-case, kafka, postgresql]
language: vi
---

Chuyến `TN-0001` hoàn thành lúc 08:31. Booking phát `TripCompleted`, Payment phát `PaymentCaptured`: hai service, hai topic, không ai hứa cái nào đến trước. Có lần tiền đến trước chuyến ba giây, có lần một sự kiện đến hai lần, có lần ledger chết đúng lúc đang ghi. Bài này lắp các mảnh của P05 thành một luồng chạy thật trên PostgreSQL và Kafka, rồi dựng từng tình huống bằng test.

Đây là bài 54 của lộ trình **Rust nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)**, nối tiếp [bài 53](/articles/rust-aggregate-invariant-optimistic-locking-for-update). Lab là workspace bốn crate như [bài 51](/articles/rust-hexagonal-clean-workspace-crates-port-adapter), dùng lại `ledger-core` của [bài 20](/articles/rust-tong-hop-ledger-core-cargo-doc-workspace) qua path dependency, không cần đến lab nào khác.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p><code>PaymentCaptured</code> của <code>TN-0001</code> đến trước <code>TripCompleted</code> ba giây. Ledger có nên ghi bút toán capture ngay, vì tiền đã thu thật? Nếu không, ledger còn thiếu gì?</p>
<details><summary>Xem đáp án</summary><p>Không nên. <code>PaymentCaptured</code> chỉ có <code>paymentId</code>, <code>tripId</code>, <code>amountDong</code>, <code>externalRef</code> và <code>occurredAt</code>: không có mã khách, mã tài xế hay cước. Ledger không biết trừ ví ai và không kiểm được số thu có khớp cước. Ledger phải nhớ dữ kiện, chưa ghi gì, và chỉ ghi khi đủ.</p></details>
</div>

## Luồng này ghép những bài nào, và bỏ những bài nào?

Hình 1 là đường đi của một chuyến. Bảng dưới nói mỗi bước lấy gì từ bài nào; phần bỏ qua quan trọng không kém phần lấy, vì đó là điều lab chưa chứng minh.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 1. Booking phát TripCompleted lên topic trip.completed, Payment phát PaymentCaptured lên topic payment.captured. Một consumer đọc cả hai topic, giao cho handle. Handle đọc aggregate TripPayment, để domain quyết định, rồi commit trong một transaction PostgreSQL gồm aggregate có version, bút toán, số dư, psp_refs và outbox. Relay đọc outbox và phát hai LedgerEntryPosted cho Payment và Admin."><defs><marker id="rs54-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="10" y="30" width="170" height="76" rx="10" class="f-phys-soft s-phys"/><text x="24" y="56" class="t-b">Booking (C#)</text><text x="24" y="76" class="t-small">phát TripCompleted</text><text x="24" y="92" class="t-small">topic trip.completed</text><rect x="10" y="184" width="170" height="76" rx="10" class="f-phys-soft s-phys"/><text x="24" y="210" class="t-b">Payment (C#)</text><text x="24" y="230" class="t-small">phát PaymentCaptured</text><text x="24" y="246" class="t-small">topic payment.captured</text><rect x="210" y="14" width="490" height="272" rx="10" class="f-none s-pk dash"/><text x="226" y="40" class="t-b">ledger (Rust), một process</text><rect x="226" y="56" width="170" height="56" rx="10" class="f-phys-soft s-phys"/><text x="240" y="80" class="t-b">consumer</text><text x="240" y="98" class="t-small">commit offset sau DB</text><rect x="226" y="128" width="170" height="76" rx="10" class="f-phys-soft s-phys"/><text x="240" y="152" class="t-b">handle</text><text x="240" y="170" class="t-small">validate, quyết định</text><text x="240" y="186" class="t-small">thua version: đọc lại</text><rect x="226" y="216" width="170" height="56" rx="10" class="f-phys-soft s-phys"/><text x="240" y="240" class="t-b">TripPayment</text><text x="240" y="258" class="t-small">nhớ dữ kiện, version n</text><rect x="424" y="56" width="264" height="216" rx="10" class="f-pk-soft s-pk"/><text x="438" y="80" class="t-b">PostgreSQL, MỘT transaction</text><text x="438" y="106" class="t-small">trip_payments: aggregate + version</text><text x="438" y="126" class="t-small">journal_entries, postings</text><text x="438" y="146" class="t-small">account_balances (số dư)</text><text x="438" y="166" class="t-small">psp_refs (mã giao dịch VíXanh)</text><text x="438" y="186" class="t-small">outbox (LedgerEntryPosted)</text><text x="438" y="250" class="t-small">COMMIT: cùng có, hoặc cùng không</text><rect x="724" y="96" width="146" height="100" rx="10" class="f-phys-soft s-phys"/><text x="738" y="122" class="t-b">relay outbox</text><text x="738" y="142" class="t-small">LedgerEntryPosted</text><text x="738" y="160" class="t-small">2 sự kiện ra Kafka</text><text x="738" y="178" class="t-small">Payment, Admin đọc</text><path d="M180 68 L226 84" class="f-none s-phys" stroke-width="2" marker-end="url(#rs54-0-a)"/><path d="M180 222 L226 100" class="f-none s-phys" stroke-width="2" marker-end="url(#rs54-0-a)"/><path d="M311 112 L311 128" class="f-none s-phys" stroke-width="2" marker-end="url(#rs54-0-a)"/><path d="M311 204 L311 216" class="f-none s-phys" stroke-width="2" marker-end="url(#rs54-0-a)"/><path d="M396 166 L424 166" class="f-none s-phys" stroke-width="2" marker-end="url(#rs54-0-a)"/><path d="M688 146 L724 146" class="f-none s-phys" stroke-width="2" marker-end="url(#rs54-0-a)"/></svg>
</div>
<figcaption>Hình 1. Một chuyến đi qua hai topic vào và một topic ra. Khung PostgreSQL là phần phải nguyên khối: aggregate, hai bút toán, số dư và outbox cùng có hoặc cùng không.</figcaption>
</figure>

| Bước | Lab 54 dùng | Từ bài |
| --- | --- | --- |
| Chia crate | `domain` ← `application` ← `infrastructure` ← `api`; port là trait trả `impl Future + Send`; test đọc `Cargo.toml` để chặn `tokio`, `sqlx` vào `domain` (chỉ thấy phụ thuộc trực tiếp; bài 51 đọc `cargo metadata`). Một port `PaymentStore` thay ba port. Bỏ feature `kafka`, `Clock`, `EventPublisher` (outbox nằm trong `commit`); kho bộ nhớ chỉ còn trong unit test | [51](/articles/rust-hexagonal-clean-workspace-crates-port-adapter) |
| Lệnh và handler | Giữ dispatch generic và sự kiện domain do aggregate sinh, rút còn enum `Command` và một hàm `handle` generic trên `PaymentStore`. Bỏ trait `Command`/`CommandHandler` với `Output` liên kết và ba lớp `Validated`, `Traced`, `Transactional` | [52](/articles/rust-command-handler-pipeline-domain-event-khong-framework) |
| Khoá lạc quan | Chỉ lấy mẫu, không lấy `Account` hay `account_limits`: `trip_payments.version`, `UPDATE ... WHERE version = $n`, thêm nhánh tạo bằng `unique_violation` mà bài 53 không có. Đọc lại ngay, tối đa 8 lần, không có jitter như `with_retry`; `40P01` thì consumer xử lý lại message. Số dư vẫn cộng dồn kiểu bài 25 | [53](/articles/rust-aggregate-invariant-optimistic-locking-for-update) |
| Tính phí | `default_policy()` 20%, làm tròn nửa ra xa 0: 5.100 và 20.400 | [11](/articles/rust-trait-generics-chinh-sach-phi-lam-tron), [20](/articles/rust-tong-hop-ledger-core-cargo-doc-workspace) |
| Hai bút toán | `JournalEntryBuilder`, dấu dòng tiền, key `capture` rồi `split` qua clearing | [13](/articles/rust-double-entry-journal-entry-posting), [41](/articles/rust-bai-toan-so-cai-chart-of-accounts-dong-tien-chuyen) |
| Ghi idempotent | `ON CONFLICT DO NOTHING` rồi so nội dung; trigger cấm UPDATE và DELETE | [24](/articles/rust-transaction-isolation-append-only-idempotency-key) |
| Schema | bốn bảng chuẩn, `02_balance_check.sql` giống từng byte bản lab 25, `account_balances` là projection | [25](/articles/rust-schema-so-cai-accounts-journal-postings-partition) |
| Kafka | tạo topic rồi chờ có leader, commit offset sau khi DB commit | [36](/articles/rust-kafka-rdkafka-consumer-producer-schema) |
| Outbox | ghi cùng transaction, relay `SKIP LOCKED`, at-least-once. Bỏ inbox | [37](/articles/rust-outbox-inbox-idempotent-consumer) |
| Khung đủ vòng | Kafka vào, PostgreSQL, Kafka ra. Đổi ba dòng thành hai bước; bài 40 giả lập cái chết bằng cách huỷ task, ở đây là tiến trình con `abort` thật | [40](/articles/rust-tong-hop-ledger-api-postgresql-kafka-end-to-end) |
| Khuyến mãi | `platform:treasury`, `platform:promo-wallet`, khoá hàng ngân sách của campaign | [45](/articles/rust-khuyen-mai-tro-gia-but-toan-nhieu-ben-ngan-sach) |
| Hoàn một phần | chia theo tổng luỹ kế, key `trip:<id>:refund:<n>`, không đặt `reversal_of`. Bỏ maker-checker, đảo toàn phần, chargeback | [43](/articles/rust-refund-reversal-chargeback-but-toan-dao-maker-checker) |
| Đối soát | so theo dòng clearing và `external_ref` qua `psp_refs`, bốn loại kết luận. Bỏ `DATE_SHIFT` và đọc CSV | [47](/articles/rust-doi-soat-psp-sao-ke-csv-khop-lech-bao-cao) |
| Không dùng | hold ([42](/articles/rust-hold-capture-void-so-du-kha-dung)): chuyến trả qua VíXanh, không giữ chỗ trong ví; payout ([44](/articles/rust-vi-tai-xe-chia-tien-payout-theo-lo)); chia nhiều bên ([46](/articles/rust-lam-tron-chia-tien-largest-remainder-khong-mat-dong)); đóng kỳ ([48](/articles/rust-so-du-theo-thoi-diem-snapshot-dong-ky)); rebuild projection ([49](/articles/rust-event-sourcing-co-chung-muc-projection-replay)), lab chỉ đối chiếu số dư với posting; hash chain và role ([50](/articles/rust-audit-append-only-hash-chain-phan-quyen-pii)) | 42, 44, 46, 48, 49, 50 |

## Ai nhớ dữ kiện đến sớm, và vì sao thứ tự không còn quan trọng?

Aggregate `TripPayment` là một dòng cho mỗi chuyến trong `trip_payments` (cột `state` jsonb, cột `version`), có ba ngăn: dữ kiện chuyến, thu tiền, khuyến mãi. Dữ kiện trùng hệt thì bỏ qua, khác nội dung thì từ chối, mới thì lưu rồi gọi `advance`:

```rust
// crates/domain/src/payment.rs (rút gọn: bỏ đoạn dựng hai bút toán)
fn advance(&mut self, events: &mut Vec<DomainEvent>) -> Result<(), DomainError> {
    if self.settled {
        return Ok(());
    }
    let (Some(trip), Some(capture)) = (&self.trip, &self.capture) else {
        return Ok(()); // còn thiếu một trong hai: chờ
    };
    if capture.amount_dong != trip.fare_dong - self.discount() {
        return Ok(()); // số thu không khớp: không ghi gì, không đoán
    }
    // ... dựng capture_entry và split_entry bằng flows.rs
    self.settled = true;
    events.push(DomainEvent::Settled { entries: vec![capture_entry, split_entry] });
    Ok(())
}
```

`advance` chỉ nhìn trạng thái hiện tại nên không có nhánh «nếu capture đến trước». Test `every_arrival_order_settles_the_same` thử sáu hoán vị của ba dữ kiện: trạng thái cuối và tập bút toán phải y hệt. Nếu bỏ lời gọi `advance` khỏi `record_trip`, test đỏ ở hoán vị `[1, 2, 0]`.

Đường Kafka được kiểm lại bằng hai lần chạy thật: phát dữ kiện đầu, đợi bộ đếm `handled` lên 1, xem trạng thái aggregate, rồi mới phát dữ kiện thứ hai:

```text
-- chuyến xong trước
     TripCompleted đến trước (version 1): chờ PaymentCaptured; sổ còn 0 bút toán
-- tiền về trước
     PaymentCaptured đến trước (version 1): chờ TripCompleted; sổ còn 0 bút toán
```

Cả hai lần đều cho ra `JE-000001` (`capture`) rồi `JE-000002` (`split`); test so ảnh chụp sổ và danh sách `LedgerEntryPosted` của hai lần: phải bằng nhau.

Phép so số thu với cước trừ khuyến mãi có hệ quả cố ý: số thu lệch thì không ghi gì. `TN-0004` có cước 25.500, VíXanh báo thu 24.000: sổ không đổi, aggregate nói rõ nó chờ gì. `TN-0002` thu 20.500 vì có mã giảm 5.000, quyết định khuyến mãi đến sau hai sự kiện kia và chính quyết định đó làm hai vế bằng nhau. Đoán «chắc là có khuyến mãi» thì một khoản thu sai sẽ thành doanh thu sai.

Lab không có bảng `inbox` của [bài 37](/articles/rust-outbox-inbox-idempotent-consumer): aggregate nhớ dữ kiện nào đã nhận nên tự nhận ra bản trùng, kể cả bản khác nội dung mà inbox khoá theo `event_id` bỏ qua mà không báo (bài tập cuối bài 40).

Aggregate chờ này có giá: thêm một bảng trạng thái, thêm dữ liệu jsonb mà các lần đổi cấu trúc về sau vẫn phải đọc được, và thêm trạng thái kẹt cần theo dõi. Nếu Payment gộp sẵn mã khách, mã tài xế và cước vào `PaymentCaptured`, ledger chỉ cần một sự kiện và không nên dựng aggregate chờ này.

## Hai dữ kiện cùng đến thì ai thắng?

Hai task nhận hai dữ kiện của một chuyến, cùng đọc aggregate, mỗi bên chỉ thấy dữ kiện của mình. Không có khoá thì bên commit sau ghi đè bên trước và dữ kiện kia biến mất. Lab dùng khoá lạc quan của bài 53, hai nhánh vì có hai cách thua:

```rust
// crates/infrastructure/src/pg.rs (rút gọn: bỏ chú thích)
if work.expected_version == 0 {
    // chưa có dòng nào: hai bên cùng INSERT, bên sau nhận unique_violation
    let inserted = sqlx::query("INSERT INTO trip_payments (trip_id, version, state) VALUES ($1, 1, $2)")
        /* .bind(...).execute(...) */;
    // Err(err) nếu is_unique_violation(&err) => Err(StoreError::VersionConflict)
}
// đã có dòng: UPDATE có điều kiện, bên thua thấy 0 dòng khớp
sqlx::query(
    "UPDATE trip_payments SET version = version + 1, state = $2, updated_at = now() \
     WHERE trip_id = $1 AND version = $3",
)
/* ... rows_affected() == 0 => Err(StoreError::VersionConflict) */
```

Bên thua không trả lỗi cho ai vì không có ai để trả: `handle` đọc lại, quyết định lại rồi commit lại.

```rust
// crates/application/src/handler.rs (rút gọn)
for attempt in 1..=MAX_ATTEMPTS {
    let (mut payment, expected_version) = match store.load(trip_id).await? {
        Some(loaded) => (loaded.payment, loaded.version),
        None => (TripPayment::new(trip_id), 0),
    };
    let events = decide(&mut payment, command)?;
    // ... events rỗng: lệnh trùng, trả về luôn, không commit
    match store.commit(Work { payment, expected_version, entries /* từ events */ }).await {
        Ok(committed) => return Ok(/* Outcome */),
        Err(StoreError::VersionConflict) => conflicts += 1, // đọc lại, quyết định lại
        Err(other) => return Err(other.into()),
    }
}
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 280" role="img" aria-label="Hình 2. Hai lệnh cùng chuyến đều đọc version 0. Lệnh A, TripCompleted, ghi INSERT và thắng với version 1. Lệnh B, PaymentCaptured, INSERT bị unique_violation nên thua, đọc lại thấy TripCompleted rồi quyết định lại, chốt sổ và commit lên version 2 cùng hai bút toán."><defs><marker id="rs54-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><text x="18" y="28" class="t-b">Lệnh A: TripCompleted</text><rect x="18" y="40" width="156" height="64" rx="10" class="f-phys-soft s-phys"/><text x="32" y="66" class="t-b">load</text><text x="32" y="86" class="t-small">version 0 (chưa có)</text><rect x="190" y="40" width="156" height="64" rx="10" class="f-phys-soft s-phys"/><text x="204" y="66" class="t-b">decide</text><text x="204" y="86" class="t-small">TripRecorded</text><rect x="362" y="40" width="156" height="64" rx="10" class="f-pk-soft s-pk"/><text x="376" y="66" class="t-b">commit: INSERT</text><text x="376" y="86" class="t-small">thắng, version 1</text><path d="M174 72 L190 72" class="f-none s-phys" stroke-width="2" marker-end="url(#rs54-1-a)"/><path d="M346 72 L362 72" class="f-none s-phys" stroke-width="2" marker-end="url(#rs54-1-a)"/><text x="18" y="138" class="t-b">Lệnh B: PaymentCaptured</text><rect x="18" y="148" width="156" height="64" rx="10" class="f-phys-soft s-phys"/><text x="32" y="174" class="t-b">load</text><text x="32" y="194" class="t-small">version 0 (chưa có)</text><rect x="190" y="148" width="156" height="64" rx="10" class="f-phys-soft s-phys"/><text x="204" y="174" class="t-b">decide</text><text x="204" y="194" class="t-small">CaptureRecorded</text><rect x="362" y="148" width="156" height="64" rx="10" class="f-warn-soft s-warn"/><text x="376" y="174" class="t-b">commit: INSERT</text><text x="376" y="194" class="t-small">unique_violation</text><rect x="534" y="148" width="156" height="64" rx="10" class="f-phys-soft s-phys"/><text x="548" y="174" class="t-b">đọc lại: v1</text><text x="548" y="194" class="t-small">thấy TripCompleted</text><rect x="706" y="148" width="156" height="64" rx="10" class="f-pk-soft s-pk"/><text x="720" y="174" class="t-b">decide + commit</text><text x="720" y="194" class="t-small">Settled, version 2</text><path d="M174 180 L190 180" class="f-none s-phys" stroke-width="2" marker-end="url(#rs54-1-a)"/><path d="M346 180 L362 180" class="f-none s-phys" stroke-width="2" marker-end="url(#rs54-1-a)"/><path d="M518 180 L534 180" class="f-none s-phys" stroke-width="2" marker-end="url(#rs54-1-a)"/><path d="M690 180 L706 180" class="f-none s-phys" stroke-width="2" marker-end="url(#rs54-1-a)"/><path d="M440 104 L440 148" class="f-none s-warn dash" stroke-width="2" marker-end="url(#rs54-1-a)"/><text x="18" y="244" class="t-small">Barrier của Gated: cả hai đã load xong version 0 rồi mới cùng đi tiếp. Bên đến commit sau là bên thua.</text><text x="18" y="264" class="t-small">Hai bút toán và hai dòng outbox chỉ nằm trong commit cuối của bên thua.</text></svg>
</div>
<figcaption>Hình 2. Pha tạo: bên thua nhận <code>unique_violation</code> rồi đọc lại. Pha cập nhật thay <code>INSERT</code> bằng <code>UPDATE ... WHERE version = 1</code>: bên thua thấy 0 dòng khớp.</figcaption>
</figure>

Race condition được dựng bằng `Gated`, decorator của `PaymentStore`: `load` đầu tiên của mỗi bên chờ nhau ở `Barrier`, nên cả hai chắc chắn đọc xong version cũ trước khi ai commit.

```rust
// crates/api/src/gated.rs
async fn load(&self, trip_id: &str) -> Result<Option<Loaded>, StoreError> {
    let loaded = self.inner.load(trip_id).await;
    if self.loads.fetch_add(1, Ordering::SeqCst) < self.parties {
        self.barrier.wait().await; // mọi bên đã đọc version cũ rồi mới cùng đi tiếp
    }
    loaded
}
```

Hai pha, mỗi pha mười vòng: pha 1 đua tạo, pha 2 đua cập nhật (aggregate đã có version 1, `PaymentCaptured` 20.500 và khuyến mãi 5.000 cùng đọc nó). Bên thắng thay đổi giữa các vòng nên test chỉ kiểm invariant:

```text
pha 1, tạo: 10 vòng, hai lệnh cùng thấy «chưa có aggregate» rồi cùng commit
  mỗi vòng: đúng 1 lần thua version, số lần thử [1, 2], 2 bút toán, aggregate version 2
pha 2, cập nhật: 10 vòng, aggregate đã có version 1; PaymentCaptured 20.500 và khuyến mãi 5.000 cùng đọc nó
  mỗi vòng: đúng 1 lần thua version, 3 bút toán (promo, capture, split), aggregate version 3
toàn sổ: 51 bút toán, 0 bút toán lệch, tổng 0, projection khớp, ngân sách 50.000 dùng hết đúng 10 lần
```

(Rút gọn: bỏ dòng đếm bên thắng của pha 1.) Pha 2 được thêm vào vì bản đầu chỉ có pha 1 và test vẫn xanh khi bỏ điều kiện `version = $3`: race condition khi tạo không đi qua câu `UPDATE`. Giờ bỏ điều kiện đó thì test đỏ với `pha 2, vòng 0: 0 lần thua version, mong đúng 1`; không dịch `unique_violation` thì pha 1 đỏ với `duplicate key value violates unique constraint`.

[Use case đặt chuyến của loạt C#](/articles/dotnet10-use-case-dat-chuyen-end-to-end) có cùng hình dạng: validate, handler, một ranh giới commit, outbox, receipt chống replay, và kịch bản hai tài xế cùng nhận chuyến cho ra một 200 và một 409. Khác ở nguồn lệnh: ở đó là HTTP nên bên thua trong kịch bản nhận chuyến nhận 409 và tự quyết; ở đây là Kafka, không ai để trả 409, nên bên thua đọc lại và làm tiếp. Đường tạo chuyến bên đó giống hơn: bên thua constraint của receipt thì rollback rồi đọc kết quả đã commit, cùng dáng với bên thua `unique_violation` ở đây. Vai trò của receipt ở đây do aggregate đảm nhận: cùng key khác nội dung thì `DomainError::Conflict` thay cho 409.

## Hai bút toán thay một: bài 40 khác bài này ở đâu?

`ledger-api` của [bài 40](/articles/rust-tong-hop-ledger-api-postgresql-kafka-end-to-end) ghi một bút toán ba dòng (khách -25.500, doanh thu 5.100, tài xế 20.400) với key `trip:TN-0001:capture`. Bài này làm theo [bài 41](/articles/rust-bai-toan-so-cai-chart-of-accounts-dong-tien-chuyen): capture đưa tiền khách sang `psp:vixanh:clearing`, split chia từ clearing:

```text
JE-000001 trip:TN-0001:capture: customer:CU-001 -25.500 VND, psp:vixanh:clearing 25.500 VND
JE-000002 trip:TN-0001:split: psp:vixanh:clearing -25.500 VND, platform:revenue 5.100 VND, driver:DR-001 20.400 VND
số dư: customer -25.500, clearing 0, revenue 5.100, driver 20.400; aggregate version 2
```

Việc chuyển dạng có bốn hệ quả, và hệ quả đầu tiên đắt nhất.

- **Key cũ mang nội dung mới.** Cùng `trip:TN-0001:capture` nhưng hai dòng thay vì ba. Trên sổ đã có bút toán ba dòng, capture hai dòng bị từ chối (`idempotency_key trip:TN-0001:capture đã dùng cho bút toán khác nội dung`) và aggregate rollback theo, không nhớ `PaymentCaptured`. Consumer coi đó là lỗi vĩnh viễn và bỏ message. Sổ đang chạy dạng ba dòng cần key mới hoặc migration, chủ đề của [bài 56](/articles/rust-feature-flag-config-migration-expand-contract).
- **Hai sự kiện thay một.** Muốn biết «đã thu» thì đọc `capture`, muốn biết tài xế nhận bao nhiêu thì đọc `split`.
- **Có dòng để đối soát.** Dòng clearing +25.500 của `capture` so được với sao kê VíXanh; dạng ba dòng không có dòng nào như vậy.
- **Khoảng thời gian «đã thu, chưa chia» chưa quan sát được.** Hai bút toán nằm trong một transaction nên không ai thấy clearing ở +25.500; tách thành hai transaction chỉ có nghĩa khi split phải chờ điều gì đó, như một quyết định phí. Lợi ích còn lại là có key riêng để hoàn hoặc đảo từng bước.

Nhánh khuyến mãi dùng hai tài khoản của [bài 45](/articles/rust-khuyen-mai-tro-gia-but-toan-nhieu-ben-ngan-sach), `platform:treasury` và `platform:promo-wallet`. `TN-0002` có mã giảm 5.000 do nền tảng chịu; phí tính trên cước gốc nên doanh thu vẫn 5.100 và tài xế vẫn 20.400:

```text
trip:TN-0002:promo:WELCOME5K: promo:budget:WELCOME5K -5.000 VND, platform:promo-expense 5.000 VND
trip:TN-0002:capture: customer:CU-002 -20.500 VND, psp:vixanh:clearing 20.500 VND
trip:TN-0002:split: psp:vixanh:clearing -20.500 VND, platform:promo-wallet -5.000 VND, platform:revenue 5.100 VND, driver:DR-002 20.400 VND
doanh thu 5.100 - chi phí khuyến mãi 5.000 = 100; ngân sách còn 5.000; clearing 0
TN-0003 xin 6.000 khi ngân sách còn 5.000: BudgetExhausted, không có aggregate, không có bút toán
```

Bút toán dùng ngân sách khoá hàng số dư của campaign (`FOR UPDATE`) trong cùng transaction, nên hết ngân sách thì không để lại dấu vết nào. Hoàn một phần theo tổng luỹ kế của [bài 43](/articles/rust-refund-reversal-chargeback-but-toan-dao-maker-checker): hoàn 10.000 là 2.000 phí cộng 8.000 từ tài xế, key `trip:TN-0001:refund:1`, không đặt `reversal_of` vì nó không phải bút toán đảo của bút toán nào; gửi lại không ghi thêm, xin 16.000 khi chỉ còn 15.500 thì từ chối. Chuyến có khuyến mãi chưa hoàn được.

## Chết giữa chừng và sự kiện trùng: chuyện gì còn lại?

Sự kiện trùng đi qua chính aggregate: mỗi loại hai lần, thêm một `TripCompleted` có cước đổi thành 26.000. Test đợi `handled` lên 5 rồi kiểm: 2 message trùng không làm gì, 1 message đổi nội dung bị từ chối (chỉ ghi log, chưa có DLQ), sổ vẫn hai bút toán.

Lab không giả lập việc chết giữa chừng ngay trong process: tiến trình con `trip-payment serve` tự gọi `std::process::abort()` ở message chốt sổ, tại một trong hai điểm chọn trước quanh `COMMIT`; tiến trình chính kiểm hiện trường rồi chạy service mới cùng consumer group. Không có `Drop`, không có `ROLLBACK` từ client, nhưng cũng không phải `kill` từ bên ngoài ở thời điểm ngẫu nhiên.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Hình 3. Một message chốt sổ đi qua nhận message, ghi aggregate bút toán số dư outbox trong transaction, COMMIT, commit offset Kafka, rồi relay phát outbox. Điểm chết A nằm trước COMMIT: DB không có gì, Kafka giao lại, lần chạy lại xử lý thật. Điểm chết B nằm sau COMMIT: DB có hai bút toán và hai dòng outbox chưa phát, Kafka giao lại, aggregate nhận ra bản trùng, relay phát nốt."><defs><marker id="rs54-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="40" width="144" height="84" rx="10" class="f-phys-soft s-phys"/><text x="34" y="66" class="t-b">nhận message</text><text x="34" y="88" class="t-small">offset n</text><text x="34" y="104" class="t-small">chưa commit</text><rect x="194" y="40" width="144" height="84" rx="10" class="f-phys-soft s-phys"/><text x="208" y="66" class="t-b">BEGIN + ghi</text><text x="208" y="88" class="t-small">aggregate, outbox,</text><text x="208" y="104" class="t-small">2 bút toán, số dư</text><rect x="368" y="40" width="144" height="84" rx="10" class="f-pk-soft s-pk"/><text x="382" y="66" class="t-b">COMMIT</text><text x="382" y="88" class="t-small">tất cả hoặc</text><text x="382" y="104" class="t-small">không gì</text><rect x="542" y="40" width="144" height="84" rx="10" class="f-phys-soft s-phys"/><text x="556" y="66" class="t-b">commit offset</text><text x="556" y="88" class="t-small">Kafka biết</text><text x="556" y="104" class="t-small">đã xong</text><rect x="716" y="40" width="144" height="84" rx="10" class="f-phys-soft s-phys"/><text x="730" y="66" class="t-b">relay outbox</text><text x="730" y="88" class="t-small">LedgerEntryPosted</text><text x="730" y="104" class="t-small">đánh dấu đã phát</text><path d="M164 82 L194 82" class="f-none s-phys" stroke-width="2" marker-end="url(#rs54-2-a)"/><path d="M338 82 L368 82" class="f-none s-phys" stroke-width="2" marker-end="url(#rs54-2-a)"/><path d="M512 82 L542 82" class="f-none s-phys" stroke-width="2" marker-end="url(#rs54-2-a)"/><path d="M686 82 L716 82" class="f-none s-phys" stroke-width="2" marker-end="url(#rs54-2-a)"/><path d="M353 24 L353 138" class="f-none s-warn dash" stroke-width="2"/><text x="353" y="18" text-anchor="middle" class="t-b">A</text><path d="M527 24 L527 138" class="f-none s-warn dash" stroke-width="2"/><text x="527" y="18" text-anchor="middle" class="t-b">B</text><rect x="20" y="152" width="420" height="104" rx="10" class="f-warn-soft s-warn"/><text x="34" y="178" class="t-b">A: chết (SIGABRT) trước COMMIT</text><text x="34" y="200" class="t-small">DB: 0 bút toán, 0 dòng outbox, aggregate version 1</text><text x="34" y="218" class="t-small">offset chưa commit nên Kafka giao lại message</text><text x="34" y="236" class="t-small">chạy lại: xử lý thật, ghi mới 2 bút toán</text><rect x="460" y="152" width="400" height="104" rx="10" class="f-warn-soft s-warn"/><text x="474" y="178" class="t-b">B: chết (SIGABRT) sau COMMIT</text><text x="474" y="200" class="t-small">DB: 2 bút toán, 2 dòng outbox chưa phát</text><text x="474" y="218" class="t-small">offset chưa commit nên Kafka giao lại message</text><text x="474" y="236" class="t-small">chạy lại: aggregate nhận ra trùng, relay phát nốt</text></svg>
</div>
<figcaption>Hình 3. Hai điểm chết quanh <code>COMMIT</code>. Cả hai kết thúc ở cùng một sổ: đúng hai bút toán và đúng hai <code>LedgerEntryPosted</code>.</figcaption>
</figure>

Tiến trình con chạy không có relay để outbox còn nguyên lúc chết. `wait_with_output` đọc log tới EOF rồi mới trả về, không đoán bằng `sleep`:

```rust
// crates/api/src/scenarios.rs (rút gọn)
let child = Process::new(exe).arg("serve").envs(env.child_vars(fault, false))
    .stdout(Stdio::piped()).stderr(Stdio::piped()).kill_on_drop(true).spawn()?;
let output = tokio::time::timeout(WITHIN, child.wait_with_output()).await??;
ensure!(output.status.signal() == Some(6), "mong chết vì SIGABRT");
```

```text
tiến trình con chết (SIGABRT): FAULT: tự abort ở điểm chọn trước: sau COMMIT DB, trước khi commit offset
hiện trường: 2 bút toán đã commit, 2 dòng outbox chưa phát, aggregate version 2, offset chưa commit
chạy lại: message giao lại bị aggregate nhận ra là trùng (noops 1, ghi mới 0); relay phát nốt 2 dòng outbox
kết quả cuối: đúng 2 bút toán, đúng 2 LedgerEntryPosted, driver:DR-001 20.400, y hệt luồng không chết
```

Ca chết trước `COMMIT` để lại `0 bút toán, 0 dòng outbox, aggregate version 1`, rồi lần chạy lại ghi mới 2 bút toán. Nếu đưa `commit_message` lên trước bước xử lý thì cả hai ca đỏ sau hạn chót `quá 60s mà chưa thấy: message giao lại được xử lý`; ở ca trước `COMMIT` bút toán mất vĩnh viễn.

## Đối soát cuối luồng: so theo dòng clearing

Cuối luồng, sổ phải khớp sao kê VíXanh. Như [bài 47](/articles/rust-doi-soat-psp-sao-ke-csv-khop-lech-bao-cao), lab so từng dòng clearing theo `external_ref`, không so số dư: clearing về 0 dù VíXanh có thể chưa chuyển tiền. Bảng `psp_refs` chỉ có bút toán capture, vì sao kê không có bút toán chia:

```sql
-- crates/infrastructure/src/pg.rs (clearing_lines)
SELECT r.external_ref, SUM(p.amount_dong)::bigint AS amount_dong
FROM psp_refs r JOIN postings p ON p.entry_id = r.entry_id AND p.account_id = 'psp:vixanh:clearing'
GROUP BY r.external_ref ORDER BY r.external_ref
```

Hàm phân loại là hàm thuần trong `domain`: gom mỗi phía theo ref, cộng tiền, so hai tổng. Lab chốt sổ `TN-0001` và `TN-0002`, hoàn 10.000 cho `TN-0001` (không đụng clearing), đối soát với sao kê dựng từ hai ref đó, rồi bóp méo sao kê:

```text
dòng clearing của bút toán capture: vx_ch_0001 25.500 VND, vx_ch_0002 20.500 VND
sao kê đúng: 2 ref MATCHED; số dư clearing 0 (đối soát so theo dòng, không so số dư)
  vx_ch_0001 MISSING_IN_PSP: sổ ta 25.500 VND, VíXanh không có
  vx_ch_0002 AMOUNT_MISMATCH: sổ ta 20.500 VND, VíXanh 20.000 VND
  vx_ch_0077 MISSING_IN_LEDGER: sổ ta không có, VíXanh 9.000 VND
```

## Chạy ví dụ và kiểm chứng

Hạ tầng và biến môi trường lấy theo `examples/toi-noi-rust/infra/README.md`. Từ gốc repository:

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
export TOI_NOI_KAFKA='127.0.0.1:9092'
cargo run  --manifest-path examples/toi-noi-rust/54-trip-payment-use-case/Cargo.toml              # kể lại từng kịch bản
cargo run  --manifest-path examples/toi-noi-rust/54-trip-payment-use-case/Cargo.toml -- --verify  # chỉ in tên kịch bản và VERIFY OK
cargo test --manifest-path examples/toi-noi-rust/54-trip-payment-use-case/Cargo.toml
```

`--verify` chạy chín kịch bản, mỗi kịch bản một schema `lab_<uuid>` và bộ topic `lab.54.<uuid>.*` riêng, dọn khi kịch bản kết thúc, kể cả khi lỗi; sai hoặc thiếu biến môi trường thì thoát với mã 1. `cargo test` chạy 31 test: 15 của `domain`, 3 của `application` với kho trong bộ nhớ, 13 của `api` gồm 11 kịch bản thật và hai test kiến trúc (`domain`, `application` không khai báo `tokio`, `sqlx`, `rdkafka`; trigger cân bút toán giống từng byte bản lab 25). Không test nào assert thời gian.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Luồng này dùng để học, không phải service có thể deploy. Hai điểm chết là hai điểm chọn trước, tiến trình tự <code>abort</code>, không phải kill ngẫu nhiên; khe hở khác quanh hai điểm đó chưa được thử. Chưa có HTTP hay gRPC.</p></div>

## Lab đơn giản hoá điều gì so với production?

- **Không DLQ.** Message hỏng, đổi nội dung hay vi phạm key chỉ bị ghi log rồi bỏ; dữ kiện mất khỏi luồng.
- **Chờ không có hạn.** Aggregate chờ vô thời hạn và không có cảnh báo nào; production cần metric «chờ quá N phút» như bốn metric của bài 41. Domain cũng không có đồng hồ: `postedAt` do database gán.
- **Không hold, payout, đóng kỳ, hash chain, phân quyền.** Lab chạy bằng một role PostgreSQL.
- **Một consumer, một partition mỗi topic.** Hai instance cùng group và rebalance chưa dựng; tranh chấp version chỉ dựng bằng `Barrier`.
- **Đối soát một lần.** Sao kê là mảng trong bộ nhớ, không phải CSV streaming hay job định kỳ.
- **Không đo hiệu năng.** Hai bút toán thay một nghĩa là nhiều dòng và nhiều sự kiện hơn mỗi chuyến; lab không đo cái giá đó.

**Bài tập:** `PaymentCaptured` của `TN-0001` đến lần hai với `paymentId = PAY-0002`, `externalRef = vx_ch_0002`, cùng số tiền 25.500 (khách bị thu hai lần). Lab làm gì, số dư `customer:CU-001` là bao nhiêu, và đối soát báo gì?

<details><summary>Xem lời giải</summary><p>Aggregate đã nhớ <code>PAY-0001</code>, nên bản thứ hai bị <code>DomainError::Conflict</code>: consumer ghi log, đếm vào <code>poison</code> và bỏ qua. Số dư <code>customer:CU-001</code> vẫn -25.500. Khoản thu thứ hai vẫn nằm ở VíXanh, nên đối soát báo <code>vx_ch_0002</code> là <code>MISSING_IN_LEDGER</code> 25.500. Hoàn tiền cần DLQ giữ lại sự kiện và quy trình của Finance; ghi thẳng vào ví khách là biến lỗi của VíXanh thành số dư của khách.</p></details>

Bài trước: [bài 53](/articles/rust-aggregate-invariant-optimistic-locking-for-update). Bài tiếp: [bài 55](/articles/rust-hop-dong-su-kien-xuyen-ngon-ngu-protobuf-version).

## Tài liệu gốc

- [docs.rs: sqlx 0.9.0](https://docs.rs/sqlx/0.9.0/sqlx/), phần `Transaction` và `types::Json`.
- [docs.rs: rdkafka](https://docs.rs/rdkafka): `StreamConsumer`, `FutureProducer`.
- [docs.rs: `tokio::sync::Barrier`](https://docs.rs/tokio/1.54.1/tokio/sync/struct.Barrier.html) và [`Child::wait_with_output`](https://docs.rs/tokio/1.54.1/tokio/process/struct.Child.html#method.wait_with_output).
- [PostgreSQL: READ COMMITTED](https://www.postgresql.org/docs/18/transaction-iso.html#XACT-READ-COMMITTED), chỗ UPDATE kiểm lại điều kiện trên dòng mới.
- [PostgreSQL: mã lỗi SQLSTATE](https://www.postgresql.org/docs/18/errcodes-appendix.html), `23505`.
- [Apache Kafka: Message Delivery Semantics](https://kafka.apache.org/documentation/#semantics).
