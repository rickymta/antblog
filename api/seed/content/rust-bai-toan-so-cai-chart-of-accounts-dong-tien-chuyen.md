---
slug: rust-bai-toan-so-cai-chart-of-accounts-dong-tien-chuyen
title: "Bài toán sổ cái Tới Nơi: chart of accounts và dòng tiền một chuyến"
summary: "Bài 41: chart of accounts của Tới Nơi chạy trên PostgreSQL thật. Tám mẫu tài khoản, dòng tiền TN-0001 và DL-0001 qua clearing, ranh giới với Payment C#, bốn metric đúng/sai đo bằng SQL."
category: rust-advanced
level: advanced
tags: [rust, toi-noi, ledger, chart-of-accounts, double-entry, postgresql]
language: vi
---

Lúc 08:31, chuyến `TN-0001` vừa hoàn thành. Khách `CU-001` đã trả 25.500 đồng qua VíXanh, nhưng nền tảng chưa chia cho tài xế. Tiền đang ở đâu? Với bút toán ba dòng của bài 40, câu hỏi không có chỗ trả lời: tiền đi thẳng từ khách sang nền tảng và tài xế, không có lúc nào nó còn nằm ở VíXanh. Bài này dựng sơ đồ tài khoản đủ rộng để hỏi được câu đó, rồi viết bốn truy vấn SQL cho biết sổ đang đúng hay sai.

Đây là bài 41 của lộ trình **Rust nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)**, nối tiếp [bài 40](/articles/rust-tong-hop-ledger-api-postgresql-kafka-end-to-end), nơi `ledger-api` chạy end-to-end với PostgreSQL và Kafka. Từ đây loạt đi vào nghiệp vụ: lab chỉ dùng PostgreSQL thật và `ledger-core` của [bài 20](/articles/rust-tong-hop-ledger-core-cargo-doc-workspace).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Sau bước capture của <code>TN-0001</code> và trước bước chia, <code>customer:CU-001</code> là -25.500 và <code>psp:vixanh:clearing</code> là +25.500. Tổng toàn sổ bằng bao nhiêu? Nếu đọc số dư theo loại tài khoản để lập hai phía nợ và có thì hai phía có bằng nhau không?</p>
<details><summary>Xem đáp án</summary><p>Tổng toàn sổ bằng 0, vì bút toán cân. Hai phía thì không bằng: bên nợ 51.000, bên có 0. Chỉ sau bước chia, khi clearing về 0, hai phía mới cùng 25.500. Số dư theo loại giúp hiểu từng số dư, không phải bảng cân đối kế toán.</p></details>
</div>

## Sơ đồ tài khoản gồm những loại nào?

Chart of accounts là danh sách tài khoản được phép có trong sổ, kèm loại từng tài khoản. Của Tới Nơi có tám mẫu mã, bốn loại, trên bốn bảng chuẩn của [bài 25](/articles/rust-schema-so-cai-accounts-journal-postings-partition).

Hai mẫu clearing là nơi tiền dừng giữa hai bước.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 290" role="img" aria-label="Hình 1. Tám mẫu tài khoản xếp theo bốn loại. Asset gồm clearing của VíXanh, clearing payout ngân hàng và ngân sách campaign. Liability gồm ví khách, ví tài xế và thuế giả định. Revenue có doanh thu nền tảng. Expense có chi phí khuyến mãi. Cả bốn loại đều bình thường ở chiều dương."><text x="10" y="26" class="t-b">Tám mẫu mã, bốn loại. Chiều bình thường của cả bốn loại là số dư dương.</text><rect x="10" y="40" width="200" height="200" rx="10" class="f-pk-soft s-pk"/><text x="24" y="66" class="t-b">asset</text><text x="24" y="84" class="t-small">tiền nền tảng đang giữ</text><text x="24" y="116" class="t-small">psp:vixanh:clearing</text><text x="24" y="138" class="t-small">bank:payout:clearing</text><text x="24" y="160" class="t-small">promo:budget:&lt;campaign&gt;</text><text x="24" y="206" class="t-small">dương: đang giữ</text><text x="24" y="224" class="t-small">âm: bất thường</text><rect x="230" y="40" width="200" height="200" rx="10" class="f-phys-soft s-phys"/><text x="244" y="66" class="t-b">liability</text><text x="244" y="84" class="t-small">nền tảng nợ người khác</text><text x="244" y="116" class="t-small">customer:&lt;id&gt;</text><text x="244" y="138" class="t-small">driver:&lt;id&gt;</text><text x="244" y="160" class="t-small">tax:vat:payable</text><text x="244" y="206" class="t-small">dương: nền tảng nợ</text><text x="244" y="224" class="t-small">âm: người giữ ví nợ</text><rect x="450" y="40" width="200" height="200" rx="10" class="f-phys-soft s-phys"/><text x="464" y="66" class="t-b">revenue</text><text x="464" y="84" class="t-small">phần nền tảng đã kiếm</text><text x="464" y="116" class="t-small">platform:revenue</text><text x="464" y="206" class="t-small">dương: đã kiếm</text><text x="464" y="224" class="t-small">âm: bất thường</text><rect x="670" y="40" width="200" height="200" rx="10" class="f-warn-soft s-warn"/><text x="684" y="66" class="t-b">expense</text><text x="684" y="84" class="t-small">chi phí đã phát sinh</text><text x="684" y="116" class="t-small">platform:promo-expense</text><text x="684" y="206" class="t-small">dương: đã chi</text><text x="684" y="224" class="t-small">âm: bất thường</text><text x="10" y="270" class="t-small">Loại không tham gia kiểm cân: bất biến của sổ chỉ nhìn số tiền có dấu của từng bút toán.</text></svg>
</div>
<figcaption>Hình 1. Tám mẫu mã chia vào bốn loại. Loại dùng để đọc số dư và đặt ngưỡng cảnh báo, không dùng để kiểm cân. Clearing âm có ngoại lệ, xem mục dòng tiền.</figcaption>
</figure>

Trong lab, chart là một hàm thuần: nhận mã tài khoản, trả loại hoặc `None`.

```rust
// src/chart.rs
pub fn kind_of(account: &AccountId) -> Option<AccountKind> {
    let parts: Vec<&str> = account.as_str().split(':').collect();
    match parts.as_slice() {
        ["customer" | "driver", who] if !who.is_empty() => Some(AccountKind::Liability),
        ["platform", "revenue"] => Some(AccountKind::Revenue),
        ["platform", "promo-expense"] => Some(AccountKind::Expense),
        ["promo", "budget", campaign] if !campaign.is_empty() => Some(AccountKind::Asset),
        ["psp", "vixanh", "clearing"] | ["bank", "payout", "clearing"] => Some(AccountKind::Asset),
        ["tax", "vat", "payable"] => Some(AccountKind::Liability),
        _ => None,
    }
}
```

Slice pattern trên `&[&str]` vừa tách mã vừa bắt phần `<id>`. Chạy lab, tám mẫu có loại còn `wallet:CU-001` thì không:

```text
  customer:CU-001         liability
  platform:promo-expense  expense
  psp:vixanh:clearing     asset
  wallet:CU-001           (ngoài sơ đồ, không mở được)
```

Vì sao đáng chặn? `store::post` tự mở tài khoản khi gặp mã mới nên khoá ngoại không cản được: gõ nhầm `custmer:CU-001` sẽ mở một tài khoản mới và tiền nằm đó, không ai nhìn. Chart từ chối mã ngoài sơ đồ; test `account_outside_the_chart_is_rejected_without_trace` kiểm không còn dòng nào trong `accounts` hay `journal_entries`.

## Số dư của mỗi loại đọc sang ngôn ngữ kế toán thế nào?

Từ [bài 13](/articles/rust-double-entry-journal-entry-posting), sổ của loạt này là sổ **dòng tiền**: dương là tiền vào tài khoản, âm là tiền ra, tổng bút toán bằng 0. Dấu đó không phải nợ hay có và không dịch được từng dòng. Một **số dư** thì đọc được theo loại tài khoản, và bảng dưới (hằng `READINGS` trong `chart.rs`) là phép ánh xạ duy nhất lab dùng.

| Loại | Số dư dương nghĩa là | Số dư âm nghĩa là | Kế toán gọi số dư bình thường là |
| --- | --- | --- | --- |
| asset | tiền hoặc quyền đòi tiền nền tảng đang giữ | bất thường: đã trả ra nhiều hơn đã nhận (clearing có ngoại lệ, xem mục sau) | số dư bên nợ |
| liability | nền tảng nợ người giữ ví | người giữ ví nợ nền tảng | số dư bên có |
| revenue | phần nền tảng đã kiếm | bất thường: hoàn nhiều hơn đã kiếm | số dư bên có |
| expense | chi phí đã phát sinh | bất thường: hoàn nhiều hơn đã chi | số dư bên nợ |

Phép ánh xạ chỉ có nghĩa trên số dư: cùng dấu dương, +25.500 ở clearing là số dư bên nợ, còn +5.100 ở `platform:revenue` là số dư bên có.

## Dòng tiền một chuyến chạy qua clearing ra sao?

Bước 1, **capture**: VíXanh thu 25.500 của khách, tiền nằm ở `psp:vixanh:clearing`. Bước 2, **chia**: clearing trả ra 25.500, nền tảng nhận 5.100 (20%) và tài xế `DR-001` nhận 20.400. Mỗi bước là một bút toán tự cân, có `idempotency_key` riêng.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 296" role="img" aria-label="Hình 2. Chuyến TN-0001 qua hai bút toán. Bước 1, JE-000001: customer:CU-001 trừ 25.500, psp:vixanh:clearing nhận 25.500. Bước 2, JE-000002: clearing trả ra 25.500, platform:revenue nhận 5.100 và driver:DR-001 nhận 20.400. Dạng ba dòng của bài 10, 14, 20 gộp hai bước và bỏ clearing."><defs><marker id="rust-bai-toan-so-cai-chart-of-accounts-dong-tien-chuyen-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="95" width="190" height="80" rx="10" class="f-phys-soft s-phys"/><text x="34" y="119" class="t-b">customer:CU-001</text><text x="34" y="139" class="t-small">liability · ví khách</text><text x="34" y="157" class="t-small">số dư cuối: -25.500</text><rect x="360" y="95" width="190" height="80" rx="10" class="f-pk-soft s-pk"/><text x="374" y="119" class="t-b">psp:vixanh:clearing</text><text x="374" y="139" class="t-small">asset · VíXanh đang giữ</text><text x="374" y="157" class="t-small">vào 25.500, ra 25.500</text><rect x="690" y="36" width="180" height="64" rx="10" class="f-phys-soft s-phys"/><text x="704" y="60" class="t-b">platform:revenue</text><text x="704" y="80" class="t-small">revenue · +5.100</text><rect x="690" y="170" width="180" height="64" rx="10" class="f-phys-soft s-phys"/><text x="704" y="194" class="t-b">driver:DR-001</text><text x="704" y="214" class="t-small">liability · +20.400</text><path d="M214 135 L356 135" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-bai-toan-so-cai-chart-of-accounts-dong-tien-chuyen-0-a)"/><text x="285" y="108" text-anchor="middle" class="t-b">1. capture</text><text x="285" y="124" text-anchor="middle" class="t-small">JE-000001 · 25.500</text><path d="M550 125 L686 72" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-bai-toan-so-cai-chart-of-accounts-dong-tien-chuyen-0-a)"/><path d="M550 145 L686 200" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-bai-toan-so-cai-chart-of-accounts-dong-tien-chuyen-0-a)"/><text x="610" y="18" text-anchor="middle" class="t-b">2. chia · JE-000002</text><text x="610" y="34" text-anchor="middle" class="t-small">25.500 = 5.100 + 20.400</text><rect x="10" y="250" width="860" height="36" rx="10" class="f-none s-warn dash"/><text x="26" y="273" class="t-small">Dạng ba dòng của bài 10, 14, 20 gộp hai bút toán, bỏ clearing: gọn hơn, nhưng mất khoảnh khắc "đã thu, chưa chia".</text></svg>
</div>
<figcaption>Hình 2. Hai bút toán nối nhau qua <code>psp:vixanh:clearing</code>. Sau bước 1 tiền nằm ở clearing, sau bước 2 clearing về 0.</figcaption>
</figure>

Cả hai dựng bằng `JournalEntryBuilder` của `ledger-core`, nên không tồn tại khi lệch. Phí đến từ `default_policy()` của [bài 11](/articles/rust-trait-generics-chinh-sach-phi-lam-tron), không tính tay:

```rust
// src/flows.rs (rút gọn: bỏ chú thích)
fn capture(reference: &str, customer: &str, amount: Money) -> Result<JournalEntry, LedgerError> {
    JournalEntryBuilder::new(&format!("{reference}:capture"))
        .posting(acct(&format!("customer:{customer}"))?, neg(amount)?)
        .posting(acct(CLEARING)?, amount)
        .build()
}

pub fn trip_split(trip: &str, driver: &str, fare: Money) -> Result<JournalEntry, LedgerError> {
    let split = default_policy().split(fare)?;
    JournalEntryBuilder::new(&format!("trip:{trip}:split"))
        .posting(acct(CLEARING)?, neg(fare)?)
        .posting(acct(REVENUE)?, split.platform_fee)
        .posting(acct(&format!("driver:{driver}"))?, split.driver_net)
        .build()
}
```

Lab ghi vào PostgreSQL thật rồi in (rút gọn, bỏ các dòng số dư từng tài khoản); dòng `đọc theo loại` là phép thử ở câu hỏi đầu bài.

```text
JE-000001  trip:TN-0001:capture
  customer:CU-001        -25.500 VND
  psp:vixanh:clearing    +25.500 VND
  tổng toàn sổ 0 VND; đọc theo loại: bên nợ 51.000 VND, bên có 0 VND
JE-000002  trip:TN-0001:split
  psp:vixanh:clearing    -25.500 VND
  platform:revenue        +5.100 VND
  driver:DR-001          +20.400 VND
  tổng toàn sổ 0 VND; đọc theo loại: bên nợ 25.500 VND, bên có 25.500 VND
```

Sau bước 1 hai phía lệch (51.000 so với 0). Trong sổ dòng tiền, bước 1 chỉ chuyển tiền giữa hai ngăn chứa; sổ kế toán ghi cùng sự kiện bằng bút toán khác (xem đoạn khách trả sau ở bài 13). Dùng bảng ánh xạ để hiểu từng số dư và đặt ngưỡng cảnh báo, đừng dùng nó để dựng báo cáo kế toán.

**Số dư clearing đọc thế nào, kể cả khi âm?** Sổ dòng tiền đóng kín, tổng mọi số dư luôn bằng 0, nên tiền từ ngoài vào không thể tự xuất hiện: nó hiện thành số âm ở một **tài khoản biên**. Khách trả theo chuyến thì biên là ví khách (mục metric). Khách nạp ví trước, như [bài 42](/articles/rust-hold-capture-void-so-du-kha-dung), thì biên là clearing. Clearing là cửa giữa sổ và VíXanh: dương là cửa đã nhận từ các tài khoản trong sổ nhiều hơn đã đẩy vào, âm là ngược lại. Con số đó không phải tiền thật ở VíXanh: sau bước chia clearing về 0 dù VíXanh có thể chưa chuyển tiền về; số thật nằm ở sao kê ([bài 47](/articles/rust-doi-soat-psp-sao-ke-csv-khop-lech-bao-cao)).

| Sự kiện | Dòng của clearing | Số dư clearing | Đọc là |
| --- | --- | --- | --- |
| Trả theo chuyến: capture | +25.500 | +25.500 | đã thu, chưa chia |
| Trả theo chuyến: chia | -25.500 | 0 | đã chia hết |
| Nạp ví (ví khách +40.000) | -40.000 | -40.000 | 40.000 từ ngoài đã qua VíXanh vào ví |
| Nạp ví: capture | +25.500 | -14.500 | 14.500 còn trong ví, 25.500 đã thu chưa chia |
| Nạp ví: chia | -25.500 | -40.000 | cả 40.000 đang ở ví khách, doanh thu và ví tài xế |

Capture ở hai mô hình là cùng một bút toán, clearing cùng nhận vào; nạp ví đi chiều ngược, từ cửa vào ví, nên clearing âm. Số âm tự nó chưa nói đúng sai: nạp ví là bình thường, chargeback ở [bài 43](/articles/rust-refund-reversal-chargeback-but-toan-dao-maker-checker) là phần nền tảng chịu tạm, chia khi chưa capture là lỗi (bài tập cuối bài). Phân biệt bằng tham chiếu của bút toán, không bằng con số. Bảng ánh xạ cũng không đọc đúng clearing âm do nạp ví: kế toán ghi khoản nạp là tài sản tăng bên nợ. Mục 10 của lab đo điều này: đọc theo bảng ra bên nợ 0, bên có 80.000; đổi dấu clearing thì hai phía cùng 40.000.

**Vì sao bài 10, 14 và 20 chỉ có ba dòng?** Dạng ba dòng đủ để dạy bất biến và làm tròn, nhưng thiếu ba thứ service thật cần. Một, không có trạng thái "VíXanh đã thu, chưa chia", nên chia lỗi hay chậm cũng không thấy. Hai, không có dòng clearing để đặt cạnh sao kê khi đối soát. Ba, hoàn tiền giữa capture và chia không có chỗ rơi riêng. Hai dạng cũng không sống chung một sổ: cùng key `trip:TN-0001:capture` với nội dung khác nhau là xung đột.

```text
  ghi lại capture hai bước: Replayed { entry_id: 1 }
  ghi dạng ba dòng cùng key: key trip:TN-0001:capture đã dùng cho bút toán khác nội dung
```

Cùng nội dung thì `Replayed`, đúng ý idempotency của [bài 24](/articles/rust-transaction-isolation-append-only-idempotency-key); khác nội dung thì `LedgerError::IdempotencyConflict`. `ledger-api` ở bài 40 vẫn ghi dạng ba dòng; chuyển sang hai bước là quyết định có migration, bài này không làm.

## Đơn giao hàng DL-0001 có gì khác chuyến đi?

Cùng khung, thêm một đích ở bước chia. Đơn giao `DL-0001` thu phí giao 33.000 (số liệu hư cấu). Phí nền tảng 20% là 6.600, trong đó thuế giả định 10% đã nằm sẵn: 6.600 / 11 = 600, nên doanh thu 6.000. Shipper `DR-002` nhận 26.400, và dùng mẫu `driver:<id>` vì ví đối tác cùng loại liability.

```rust
// src/flows.rs (rút gọn)
pub fn delivery_split(order: &str, shipper: &str, fee: Money) -> Result<JournalEntry, LedgerError> {
    let split = default_policy().split(fee)?;
    let vat = Money::from_dong(split.platform_fee.dong() / VAT_INCLUSIVE_DIVISOR); // 11
    let revenue = split.platform_fee.checked_sub(vat).ok_or(LedgerError::Overflow)?;
    JournalEntryBuilder::new(&format!("delivery:{order}:split"))
        .posting(acct(CLEARING)?, neg(fee)?)
        .posting(acct(REVENUE)?, revenue)
        .posting(acct(VAT)?, vat)
        .posting(acct(&format!("driver:{shipper}"))?, split.driver_net)
        .build()
}
```

```text
JE-000004  delivery:DL-0001:split
  psp:vixanh:clearing    -33.000 VND
  platform:revenue        +6.000 VND
  tax:vat:payable           +600 VND
  driver:DR-002          +26.400 VND
```

Phép chia cho 11 chia hết với số liệu này; số khác sẽ có phần dư, và dồn nó vào đâu là câu hỏi của [bài 46](/articles/rust-lam-tron-chia-tien-largest-remainder-khong-mat-dong). Cả ví dụ này, kể cả thuế, chỉ là mô hình học tập, không phải cách tính theo một quy định nào.

## Payment giữ trạng thái, Ledger giữ tiền đã xảy ra: ranh giới ở đâu?

Payment (C#) nói chuyện với VíXanh: tạo giao dịch, nhận webhook, thử lại khi timeout, và giữ sáu trạng thái `INITIATED`, `AUTHORIZED`, `CAPTURED`, `FAILED`, `REFUND_PENDING`, `REFUNDED` cùng `external_ref`, mã giao dịch của VíXanh. Ledger (Rust) không biết VíXanh đang nghĩ gì; nó ghi tiền đã chuyển giữa các tài khoản và trả số dư. "VíXanh đã báo `CAPTURED` chưa?" hỏi Payment; "tiền của `TN-0001` đã vào clearing chưa?" hỏi Ledger, qua bút toán `trip:TN-0001:capture`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 264" role="img" aria-label="Hình 3. Payment bằng C# giữ trạng thái với VíXanh và external_ref; Ledger bằng Rust giữ bút toán và số dư. Payment gửi PaymentCaptured hoặc gọi gRPC PostEntry sang Ledger; Ledger phát LedgerEntryPosted ngược lại với key trip:TN-0001:capture. Không bên nào ghi vào database của bên kia."><defs><marker id="rust-bai-toan-so-cai-chart-of-accounts-dong-tien-chuyen-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="10" y="36" width="340" height="170" rx="10" class="f-phys-soft s-phys"/><text x="26" y="62" class="t-b">Payment (C#)</text><text x="26" y="86" class="t-small">giữ trạng thái với VíXanh</text><text x="26" y="106" class="t-small">INITIATED · AUTHORIZED · CAPTURED</text><text x="26" y="126" class="t-small">FAILED · REFUND_PENDING · REFUNDED</text><text x="26" y="146" class="t-small">external_ref: mã giao dịch VíXanh</text><text x="26" y="166" class="t-small">retry, timeout, webhook</text><rect x="530" y="36" width="340" height="170" rx="10" class="f-pk-soft s-pk"/><text x="546" y="62" class="t-b">Ledger (Rust)</text><text x="546" y="86" class="t-small">giữ tiền đã xảy ra</text><text x="546" y="106" class="t-small">journal_entries, postings</text><text x="546" y="126" class="t-small">idempotency_key unique</text><text x="546" y="146" class="t-small">số dư từng tài khoản</text><text x="546" y="166" class="t-small">không có cột trạng thái thanh toán</text><path d="M354 100 L526 100" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-bai-toan-so-cai-chart-of-accounts-dong-tien-chuyen-1-a)"/><text x="440" y="86" text-anchor="middle" class="t-b">PaymentCaptured</text><text x="440" y="118" text-anchor="middle" class="t-small">hoặc gRPC PostEntry</text><path d="M526 160 L354 160" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-bai-toan-so-cai-chart-of-accounts-dong-tien-chuyen-1-a)"/><text x="440" y="146" text-anchor="middle" class="t-b">LedgerEntryPosted</text><text x="440" y="178" text-anchor="middle" class="t-small">key trip:TN-0001:capture</text><rect x="10" y="218" width="860" height="36" rx="10" class="f-none s-warn dash"/><text x="26" y="241" class="t-small">Gặp nhau ở sự kiện và idempotency_key. Không bên nào ghi vào database của bên kia.</text></svg>
</div>
<figcaption>Hình 3. Ranh giới Payment và Ledger. Sổ cái không có cột trạng thái thanh toán, và Payment không có bảng bút toán.</figcaption>
</figure>

Hai bên được phép lệch nhau, và đó là chỗ metric cần soi. Payment ghi `CAPTURED` mà Ledger chưa có bút toán capture là độ trễ sự kiện; Ledger có capture mà chưa chia là tiền nằm ở clearing.

## Làm sao biết sổ đang đúng hay sai? Bốn metric bằng SQL

Mỗi metric là một truy vấn chỉ đọc trong `src/metrics.rs`. Thời gian vào qua tham số `as_of` do người gọi truyền, nên kết quả không phụ thuộc đồng hồ máy.

| Metric | Sổ đúng khi | Cần vì |
| --- | --- | --- |
| Bút toán không cân (tổng khác 0 hoặc dưới 2 dòng) | rỗng | trigger chỉ chặn ghi mới, không kiểm lại dữ liệu cũ |
| Tổng toàn sổ, theo `postings` và theo `account_balances` | 0 và 0 | bản theo projection rẻ, bản theo `postings` quét toàn bảng |
| Số dư âm bất thường theo loại | rỗng | số âm ở asset, revenue, expense và liability (trừ ví khách ở mô hình này) là báo động |
| Clearing tồn đọng quá N ngày | rỗng | tiền đã thu mà chưa chia thì ai đó đang chờ |

```sql
-- UNBALANCED_SQL (rút gọn: bỏ ORDER BY)
SELECT e.idempotency_key,
       COALESCE(SUM(p.amount_dong), 0)::bigint AS total_dong,
       COUNT(p.entry_id)::bigint               AS lines
FROM journal_entries e
LEFT JOIN postings p ON p.entry_id = e.id
GROUP BY e.id, e.idempotency_key
HAVING COALESCE(SUM(p.amount_dong), 0) <> 0 OR COUNT(p.entry_id) < 2;

-- STUCK_CLEARING_SQL ($1 = as_of, $2 = N ngày; rút gọn)
SELECT p.account_id, split_part(e.idempotency_key, ':', 1) || ':' || split_part(e.idempotency_key, ':', 2) AS reference,
       SUM(p.amount_dong)::bigint AS stuck_dong
FROM postings p JOIN journal_entries e ON e.id = p.entry_id
WHERE p.account_id LIKE '%:clearing'
GROUP BY p.account_id, reference
HAVING SUM(p.amount_dong) <> 0
   AND MIN(e.posted_at) <= $1::text::timestamptz - make_interval(days => $2);
```

`LEFT JOIN` ở truy vấn đầu để bút toán không có posting nào cũng hiện ra. Truy vấn thứ hai gom theo tham chiếu chuyến hoặc đơn: cặp capture và chia có tổng clearing bằng 0 thì biến mất. Trên sổ gồm `TN-0001`, `DL-0001` và `TN-0002` (mới capture, chưa chia), cả bốn metric sạch:

```text
Báo cáo:
  bút toán không cân:           0
  tổng toàn sổ (postings):      0 VND
  tổng toàn sổ (projection):    0 VND
  số dư âm bất thường:          0
  clearing tồn đọng > 3 ngày:   0
  clearing đang giữ 16.500 VND của TN-0002 (mới 15 giờ, chưa quá hạn)
```

Clearing giữ tiền không phải lỗi; giữ quá lâu mới là lỗi, nên N là chính sách, chọn theo chu kỳ đối soát với VíXanh. Rồi lab cài ba sự cố: `TN-0003` capture từ 20/10 mà chưa chia; `TN-0004` lệch -500 nhờ tắt trigger rồi bật lại; `driver:DR-009` bị trừ 5.000 khi chưa có tiền (bút toán cân nên trigger không phản đối). Đây là cài lỗi có chủ đích, đại diện cho import hàng loạt hay SQL tay.

```text
Báo cáo (ví khách được phép âm):
  bút toán không cân:           1
    trip:TN-0004:capture  tổng -500 VND, 2 dòng
  tổng toàn sổ (postings):      -500 VND
  tổng toàn sổ (projection):    -500 VND
  số dư âm bất thường:          1
    driver:DR-009 (liability)  -5.000 VND
  clearing tồn đọng > 3 ngày:   1
    psp:vixanh:clearing trip:TN-0003  21.000 VND, 16 ngày
```

Mỗi sự cố bị đúng một metric nói tên. Tổng toàn sổ chỉ cho biết có lệch, truy vấn đầu chỉ ra bút toán nào.

**Khi nào `customer:<id>` âm là bình thường?** Trong lab, khách trả qua VíXanh theo từng chuyến, ví không được nạp trước, nên `customer:<id>` là tài khoản biên: tiền vào hệ thống hiện ra là số âm ở ví khách. `customer:CU-001` -84.000 là tổng khách đã trả (25.500 + 25.500 + 33.000). Truy vấn thứ ba vì thế có cờ `customer_may_be_negative`. Tắt cờ, cùng sổ đó báo thêm ba ví khách:

```text
  customer:CU-001 (liability)  -84.000 VND
  customer:CU-002 (liability)  -16.500 VND
  customer:CU-003 (liability)  -21.000 VND
  driver:DR-009 (liability)  -5.000 VND
```

Ví tài xế không có ngoại lệ ở cả hai chế độ: số âm nghĩa là nền tảng đã trả tiền chưa có.

Mô hình nạp ví đổi biên: ví khách phải không âm nên tắt cờ, còn clearing mang số âm. Mục 10 của lab chạy sổ đó (nạp từ 20/10) và cho thấy hai metric viết cho mô hình trả theo chuyến báo nhầm:

```text
  số dư âm bất thường:          1
    psp:vixanh:clearing (asset)  -40.000 VND
  clearing tồn đọng > 3 ngày:   1
    psp:vixanh:clearing fixture:topup  -40.000 VND, 16 ngày
```

Đổi mô hình thì phải viết lại ngoại lệ của hai truy vấn này.

## Chạy ví dụ và kiểm chứng

Hạ tầng theo `examples/toi-noi-rust/infra/README.md`. Từ gốc repository:

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
cargo run --manifest-path examples/toi-noi-rust/41-chart-of-accounts/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/41-chart-of-accounts/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/41-chart-of-accounts/Cargo.toml
```

Lệnh đầu in các khối ở trên theo mười mục; lệnh thứ hai kiểm từng giá trị đó, in `VERIFY OK` (sai thì thoát mã 1). `cargo test` chạy 8 unit test và 12 test với PostgreSQL thật, mỗi test một schema `lab_<uuid>`, dọn cả khi thân test lỗi. Không test nào `sleep` hay đọc đồng hồ.

Thử phá để thấy test đỏ đúng chỗ. Đổi `<=` thành `<` trong `STUCK_CLEARING_SQL` thì chỉ test đường biên hỏng.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Số liệu, tên và tỷ lệ thuế đều hư cấu; thuế là mô hình học tập, không phải tư vấn thuế, và sơ đồ tài khoản thật do kế toán duyệt. Luồng chính bỏ qua nạp ví nên ví khách mang số âm; mục 10 chỉ minh hoạ cách đọc clearing. Chưa có dữ liệu nhiều tháng, chưa đo bốn truy vấn trên bảng lớn, chưa có chỉ mục riêng cho chúng. Hold, hoàn tiền, payout và đối soát sao kê chưa có.</p></div>

## Bài tập

Sự kiện chia của `TN-0001` được xử lý trước sự kiện capture (đến sai thứ tự giữa hai service). Sổ có còn cân không? Metric nào báo, và nó báo gì?

<details><summary>Xem lời giải</summary><p>Mỗi bút toán vẫn cân, nên metric đầu và tổng toàn sổ sạch. Clearing có số dư -25.500: <code>asset</code> âm là bất thường nên metric số dư âm báo <code>psp:vixanh:clearing</code>, và metric tồn đọng báo <code>trip:TN-0001</code> với -25.500 khi đủ N ngày. Test <code>split_without_capture_is_caught_by_two_metrics</code> kiểm cả hai. Capture đến sau thì clearing về 0 và cả hai biến mất; ngưỡng N chính là khoảng chờ cho phép.</p></details>

Bài trước: [bài 40](/articles/rust-tong-hop-ledger-api-postgresql-kafka-end-to-end). Bài tiếp: [bài 42](/articles/rust-hold-capture-void-so-du-kha-dung).

## Tài liệu gốc

- [The Rust Reference: Slice patterns](https://doc.rust-lang.org/reference/patterns.html#slice-patterns).
- [std: `i64::signum`](https://doc.rust-lang.org/std/primitive.i64.html#method.signum).
- [docs.rs: sqlx](https://docs.rs/sqlx), gồm `query_as` và `PgConnectOptions::options`.
- [The Rust Book: The `match` Control Flow Construct](https://doc.rust-lang.org/book/ch06-02-match.html).
- [PostgreSQL: CREATE TRIGGER](https://www.postgresql.org/docs/current/sql-createtrigger.html), phần constraint trigger.
