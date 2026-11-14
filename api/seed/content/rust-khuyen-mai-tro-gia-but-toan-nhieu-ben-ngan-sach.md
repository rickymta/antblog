---
slug: rust-khuyen-mai-tro-gia-but-toan-nhieu-ben-ngan-sach
title: "Khuyến mãi và trợ giá: bút toán nhiều bên và ngân sách campaign"
summary: "Bài 45: mã giảm 5.000 của TN-0002 do nền tảng chịu thành bút toán cân; ai chịu giảm giá thì posting đổi thế nào; ngân sách campaign là tài khoản có giới hạn, chứng minh bằng Barrier; hết ngân sách, hoàn chuyến, trợ giá."
category: rust-advanced
level: advanced
tags: [rust, toi-noi, ledger, postgresql, concurrency, promotion]
language: vi
---

Khách `CU-002` đặt chuyến `TN-0002`, cước gốc 25.500 đồng, và nhập mã `WELCOME5K` giảm 5.000. Khách trả 20.500, nhưng tài xế `DR-002` vẫn nhận 20.400 và phí nền tảng 20% vẫn tính trên 25.500. Vậy 5.000 chênh lệch nằm ở đâu trong sổ? Và nếu nhiều khách nhập mã cùng lúc khi ngân sách chỉ còn 10.000, sổ có cho giảm quá số đó không?

Đây là bài 45 của lộ trình **Rust nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Bài dùng bất biến «bút toán cân» của [bài 13](/articles/rust-double-entry-journal-entry-posting) và [bài 20](/articles/rust-tong-hop-ledger-core-cargo-doc-workspace), cách ghi idempotent của [bài 24](/articles/rust-transaction-isolation-append-only-idempotency-key) và fixture schema riêng của [bài 27](/articles/rust-integration-test-postgresql-schema-rieng-ci). Lab chạy trên PostgreSQL 18 với `sqlx` 0.9 và `ledger-core` của bài 20.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Ngân sách còn 6.000, hai yêu cầu cùng xin 5.000. Mỗi yêu cầu đọc số dư, thấy đủ rồi mới trừ, trong một transaction READ COMMITTED. Cả hai có thể cùng thành công không?</p>
<details><summary>Xem đáp án</summary><p>Có. Hai <code>SELECT</code> thường cùng thấy 6.000 vì chưa bên nào commit, nên ngân sách về -4.000. Muốn một bên chờ bên kia, câu đọc phải khoá dòng số dư (<code>FOR UPDATE</code>).</p></details>
</div>

## Mã giảm 5.000 đi qua những tài khoản nào?

Sơ đồ chuẩn của [bài 41](/articles/rust-bai-toan-so-cai-chart-of-accounts-dong-tien-chuyen) có `promo:budget:<campaign>` (asset, số dư là ngân sách còn lại) và `platform:promo-expense` (expense). Điều cần đạt: khách trả 20.500, tài xế nhận 20.400, doanh thu 5.100, chi phí 5.000, ngân sách giảm 5.000, tổng bằng 0. Cộng thử:

```text
-20.500 + 20.400 + 5.100 + 5.000 - 5.000 = +5.000
```

Chỉ với sơ đồ chuẩn thì không cân được: trong sổ dòng tiền của bài 13, doanh thu và chi phí đều là ngăn chứa dương, nên còn 5.000 chưa có nguồn, đó là phần nền tảng trả thay khách. Lab thêm hai mẫu ngoài tám mẫu của bài 41:

- `platform:promo-wallet` (liability, cùng vai với ví khách: âm là đã trả ra): nguồn của phần trả thay.
- `platform:treasury` (asset, quỹ nền tảng): nguồn cấp ngân sách. JE-000001 cấp 10.000 từ quỹ sang `promo:budget:WELCOME5K`, asset sang asset, doanh thu không đổi.

`TN-0002` cần ba bút toán theo thứ tự của Hình 1.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 1. Ba bút toán của chuyến TN-0002 theo thứ tự: redeem lấy 5.000 từ ngân sách sang chi phí khuyến mãi, capture thu 20.500 từ khách vào clearing, split lấy 20.500 từ clearing và 5.000 từ ví khuyến mãi, chia thành tài xế 20.400 và doanh thu 5.100. Mỗi bút toán có tổng bằng 0."><defs><marker id="rs45-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="20" width="260" height="176" rx="10" class="f-pk-soft s-pk"/><text x="34" y="46" class="t-b">1. redeem (Promotion gọi)</text><text x="34" y="68" class="t-small">trip:TN-0002:promo:WELCOME5K</text><text x="34" y="98" class="t-small">promo:budget:WELCOME5K   -5.000</text><text x="34" y="118" class="t-small">platform:promo-expense   +5.000</text><text x="34" y="160" class="t-small">có thể bị từ chối: hết ngân sách</text><text x="34" y="178" class="t-small">chạy trước khi khách bị trừ tiền</text><rect x="310" y="20" width="260" height="176" rx="10" class="f-phys-soft s-phys"/><text x="324" y="46" class="t-b">2. capture (Payment)</text><text x="324" y="68" class="t-small">trip:TN-0002:capture</text><text x="324" y="98" class="t-small">customer:CU-002   -20.500</text><text x="324" y="118" class="t-small">psp:vixanh:clearing   +20.500</text><text x="324" y="160" class="t-small">ghi đúng số VíXanh đã thu</text><text x="324" y="178" class="t-small">không được phép bị từ chối</text><rect x="600" y="20" width="260" height="176" rx="10" class="f-phys-soft s-phys"/><text x="614" y="46" class="t-b">3. split (Trip hoàn tất)</text><text x="614" y="68" class="t-small">trip:TN-0002:split</text><text x="614" y="94" class="t-small">psp:vixanh:clearing   -20.500</text><text x="614" y="112" class="t-small">platform:promo-wallet   -5.000</text><text x="614" y="130" class="t-small">driver:DR-002   +20.400</text><text x="614" y="148" class="t-small">platform:revenue   +5.100</text><text x="614" y="178" class="t-small">phí 5.100 tính trên cước 25.500</text><path d="M280 106 L308 106" class="f-none s-phys" stroke-width="2" marker-end="url(#rs45-0-a)"/><path d="M570 106 L598 106" class="f-none s-phys" stroke-width="2" marker-end="url(#rs45-0-a)"/><rect x="20" y="214" width="840" height="72" rx="10" class="f-none s-line dash"/><text x="36" y="238" class="t-b">Mỗi bút toán có tổng bằng 0.</text><text x="36" y="258" class="t-small">Sau cả ba: khách -20.500, clearing 0, ví khuyến mãi -5.000, tài xế +20.400, doanh thu +5.100.</text><text x="36" y="276" class="t-small">Ngân sách -5.000, chi phí +5.000. Lãi của nền tảng: doanh thu 5.100 - chi phí 5.000 = 100.</text></svg>
</div>
<figcaption>Hình 1. Ba bút toán của <code>TN-0002</code>. Chỉ bút toán đầu có thể bị từ chối vì ngân sách; hai bút toán sau ghi tiền đã xảy ra.</figcaption>
</figure>

Output thật của lab (`cargo run`):

```text
  JE-000002 trip:TN-0002:promo:WELCOME5K
    promo:budget:WELCOME5K          -5000
    platform:promo-expense           5000
  JE-000003 trip:TN-0002:capture
    customer:CU-002                -20500
    psp:vixanh:clearing             20500
  JE-000004 trip:TN-0002:split
    psp:vixanh:clearing            -20500
    platform:promo-wallet           -5000
    driver:DR-002                   20400
    platform:revenue                 5100
  doanh thu 5100 - chi phí 5000 = 100; đọc theo loại: bên nợ 35500, bên có 35500
```

Doanh thu trừ chi phí còn 100, đúng lãi của nền tảng trên chuyến; đọc số dư theo loại như bài 41, hai phía bằng nhau. Sau `split`, ví -5.000 khớp chi phí +5.000.

Đánh đổi: thêm hai tài khoản và `split` có 4 posting; ví là tài khoản đối ứng của sổ dòng tiền, không phải chỗ có tiền thật. Cách ngắn hơn là bỏ ví, ghi doanh thu thuần +100: sổ vẫn cân, nhưng đọc theo loại thành doanh thu 100, chi phí 5.000, tức lỗ 4.900 trong khi nền tảng lãi 100.

Vì sao không gộp ba việc thành một bút toán? `redeem` có thể bị từ chối và phải chạy trước khi khách bị trừ tiền; `capture` thì không được từ chối, vì VíXanh đã thu thì tiền phải vào sổ. `capture` giữ đúng số VíXanh thu (20.500) nên đối soát ở [bài 47](/articles/rust-doi-soat-psp-sao-ke-csv-khop-lech-bao-cao) không lệch, và mỗi bút toán có key riêng nên đảo được từng cái.

## Ai chịu giảm giá thì bút toán đổi thế nào, và trợ giá tài xế TN-0003 ra sao?

Promotion gửi sang hai con số: phần nền tảng chịu và phần tài xế chịu. Doanh thu luôn là phí đủ 5.100; tài xế nhận `20.400 - phần tài xế`; ngân sách, chi phí và ví chỉ đổi đúng phần nền tảng. Ba chuyến chạy thật cùng cước, cùng mức giảm (thay đổi số dư, đồng):

| Ai chịu 5.000 | Khách | Ngân sách | Chi phí | Ví | Tài xế | Doanh thu | Posting redeem / split |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| Nền tảng | -20.500 | -5.000 | +5.000 | -5.000 | +20.400 | +5.100 | 2 / 4 |
| Tài xế | -20.500 | 0 | 0 | 0 | +15.400 | +5.100 | không có / 3 |
| Nền tảng 2.500, tài xế 2.500 | -20.500 | -2.500 | +2.500 | -2.500 | +17.900 | +5.100 | 2 / 4 |

Clearing luôn về 0: VíXanh thu bao nhiêu thì chia hết bấy nhiêu. Tài xế chịu toàn bộ thì không có `redeem` và không có posting ví. Phép tính nằm trong `split_entry` (rút gọn: bỏ kiểm tràn số và đoạn bỏ posting bằng 0):

```rust
// src/promo.rs
let fee = default_policy().split(fare)?; // 20% của cước GỐC, không phải của số khách trả
let discount = shares.discount()?;
// ... collected = cước - mức giảm, không được âm
let driver_amount = fee.driver_net.checked_sub(shares.driver) /* ... */;
if driver_amount < Money::ZERO {
    return Err(PostError::InvalidShares(format!(
        "tài xế chịu {} nhưng chỉ có {}",
        shares.driver, fee.driver_net
    )));
}
// Doanh thu luôn là phí đủ; ví khuyến mãi trả thay phần nền tảng chịu.
for (account, amount) in [
    (CLEARING.to_string(), out),
    (PROMO_WALLET.to_string(), paid_for_customer),
    (format!("driver:{driver}"), driver_amount),
    (REVENUE.to_string(), fee.platform_fee),
] {
    // ...
}
```

Ledger chỉ chặn điều làm sổ vô nghĩa: phần chịu âm, mức giảm vượt cước, hoặc tài xế chịu nhiều hơn phần họ nhận (`InvalidShares`). Luật «giảm tối đa bao nhiêu» thuộc về Promotion: giảm 100% cước vẫn ghi được, ví trả cả 25.500, doanh thu 5.100, chi phí 25.500, tức nền tảng lỗ 20.400.

**Trợ giá tài xế** là mặt kia: khách trả đủ 25.500, `DR-003` của `TN-0003` được thưởng giờ cao điểm 3.000 từ campaign `PEAKDRV`, bằng một bút toán 4 posting:

```text
  JE-000035 trip:TN-0003:subsidy:PEAKDRV
    promo:budget:PEAKDRV            -3000
    platform:promo-expense           3000
    platform:promo-wallet           -3000
    driver:DR-003                    3000
```

Cả chuyến: khách -25.500, tài xế +23.400, doanh thu +5.100, chi phí +3.000, ngân sách -3.000, ví -3.000; nền tảng còn lãi 2.100. Trợ giá cũng bị kiểm ngân sách như `redeem`.

## Ngân sách campaign là tài khoản có giới hạn: kiểm ở đâu cho đúng?

Số dư ngân sách là phần còn lại nên không được dưới 0. Quota của Promotion C# bảo vệ sớm nhưng không biết tiền thật; chỗ biết số dư thật là Ledger, **trong cùng transaction với việc ghi bút toán**. Hai chế độ kiểm của lab chỉ khác nhau ở hai chỗ:

```rust
// src/store.rs (rút gọn)
let sql = match &spend.guard {
    BudgetGuard::Locked => {
        "select balance_dong from account_balances where account_id = $1 for update"
    }
    BudgetGuard::Unlocked { .. } => {
        "select balance_dong from account_balances where account_id = $1"
    }
};
let remaining: i64 = sqlx::query_scalar(sql)
    .bind(&account)
    .fetch_optional(&mut **tx)
    .await?
    .unwrap_or(0);
if let BudgetGuard::Unlocked { after_read: Some(gate) } = &spend.guard {
    gate.wait().await; // mọi bên đã đọc xong số cũ rồi mới cùng đi tiếp
}
if remaining < spend.amount.dong() {
    return Err(PostError::BudgetExhausted { /* campaign, requested, remaining */ });
}
```

Cổng `Barrier` chỉ có trong lab. Ở READ COMMITTED, `SELECT` thường thấy dữ liệu đã commit lúc câu lệnh bắt đầu nên hai transaction cùng đọc 10.000. Với `FOR UPDATE`, transaction đến sau chờ bên kia commit rồi đọc lại phiên bản mới nhất của dòng, nên thấy số dư đã trừ. Đây là hướng «khoá một dòng chung» mà bài 24 hẹn quay lại.

Thứ tự trong `post_with` cũng quan trọng: chèn header với `idempotency_key` **trước**, kiểm ngân sách sau. Yêu cầu gửi lại cùng key rẽ vào nhánh replay và nhận kết quả cũ kể cả khi ngân sách đã cạn. Đảo thứ tự thì lần thử lại sau timeout có thể nhận lỗi hết ngân sách cho yêu cầu đã thành công.

## Barrier chứng minh gì: bản không khoá bị vượt, bản có khoá không?

Lab dựng lịch đua bằng `tokio::sync::Barrier`: một cổng cho mọi tác vụ cùng bắt đầu, và với bản không khoá, một cổng thứ hai giữa lúc đọc và lúc ghi. Đó là lịch tệ nhất, không phải lịch hay gặp (rút gọn):

```rust
// src/race.rs
let start = Arc::new(Barrier::new(n)); // mọi tác vụ bắt đầu cùng lúc
let mut set = JoinSet::new();
for i in 0..n {
    // ... mỗi tác vụ nhận bản sao ledger, cổng, guard và PromoGrant của chuyến riêng
    set.spawn(async move {
        start.wait().await;
        (trip, ledger.redeem(&grant, &guard).await)
    });
}
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 2. Hai bên cùng năm yêu cầu 5.000 vào ngân sách 10.000. Bên không khoá: cả năm cùng đọc 10.000, Barrier giữ cho cả năm đọc xong, cả năm cùng ghi, ngân sách về -15.000. Bên có khoá: yêu cầu đầu giữ khoá dòng, các yêu cầu sau chờ rồi đọc số mới, hai thành công, ba nhận lỗi hết ngân sách, ngân sách về 0."><defs><marker id="rs45-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><text x="30" y="22" class="t-b">Không khoá (SELECT thường)</text><text x="460" y="22" class="t-b">Có khoá (SELECT ... FOR UPDATE)</text><rect x="30" y="36" width="390" height="48" rx="8" class="f-phys-soft s-phys"/><text x="42" y="58" class="t-b">5 tác vụ cùng đọc số dư: 10.000</text><text x="42" y="75" class="t-small">mỗi tác vụ xin 5.000, đều thấy đủ</text><rect x="30" y="100" width="390" height="48" rx="8" class="f-none s-line dash"/><text x="42" y="122" class="t-b">Barrier: cả 5 đã đọc xong</text><text x="42" y="139" class="t-small">lịch đua tệ nhất, dựng có chủ ý</text><rect x="30" y="164" width="390" height="48" rx="8" class="f-phys-soft s-phys"/><text x="42" y="186" class="t-b">cả 5 cùng ghi: -5.000 mỗi bên</text><text x="42" y="203" class="t-small">cập nhật số dư xếp hàng nhưng đều đã được cho qua</text><rect x="30" y="228" width="390" height="58" rx="8" class="f-warn-soft s-warn"/><text x="42" y="252" class="t-b">Ngân sách -15.000, chi phí +25.000</text><text x="42" y="271" class="t-small">5 thành công: tiêu quá 15.000 đồng</text><rect x="460" y="36" width="390" height="48" rx="8" class="f-phys-soft s-phys"/><text x="472" y="58" class="t-b">T1 giữ khoá dòng ngân sách: đọc 10.000</text><text x="472" y="75" class="t-small">T2 đến T5 chờ ngay ở câu SELECT ... FOR UPDATE</text><rect x="460" y="100" width="390" height="48" rx="8" class="f-phys-soft s-phys"/><text x="472" y="122" class="t-b">T1 ghi, COMMIT: còn 5.000</text><text x="472" y="139" class="t-small">T2 nhận khoá, đọc lại 5.000, ghi, COMMIT: còn 0</text><rect x="460" y="164" width="390" height="48" rx="8" class="f-phys-soft s-phys"/><text x="472" y="186" class="t-b">T3, T4, T5 lần lượt nhận khoá</text><text x="472" y="203" class="t-small">đọc 0, BudgetExhausted, rollback</text><rect x="460" y="228" width="390" height="58" rx="8" class="f-pk-soft s-pk"/><text x="472" y="252" class="t-b">Ngân sách 0, chi phí +10.000</text><text x="472" y="271" class="t-small">2 thành công, 3 hết ngân sách</text><path d="M225 84 L225 100" class="f-none s-phys" stroke-width="2" marker-end="url(#rs45-1-a)"/><path d="M225 148 L225 164" class="f-none s-phys" stroke-width="2" marker-end="url(#rs45-1-a)"/><path d="M225 212 L225 228" class="f-none s-phys" stroke-width="2" marker-end="url(#rs45-1-a)"/><path d="M655 84 L655 100" class="f-none s-phys" stroke-width="2" marker-end="url(#rs45-1-a)"/><path d="M655 148 L655 164" class="f-none s-phys" stroke-width="2" marker-end="url(#rs45-1-a)"/><path d="M655 212 L655 228" class="f-none s-phys" stroke-width="2" marker-end="url(#rs45-1-a)"/></svg>
</div>
<figcaption>Hình 2. Năm yêu cầu 5.000 vào ngân sách 10.000. T1 đến T5 chỉ là tên minh hoạ; tác vụ nào nhận khoá trước thì không cố định.</figcaption>
</figure>

Kết quả thật của ba bản (5 yêu cầu × 5.000 vào ngân sách 10.000):

```text
  có khóa (for update)       thành công 2, hết ngân sách 3, lỗi DB 0 [], ngân sách còn 0, chi phí +10000
  không khóa                 thành công 5, hết ngân sách 0, lỗi DB 0 [], ngân sách còn -15000, chi phí +25000
  không khóa + trigger sàn   thành công 2, hết ngân sách 0, lỗi DB 3 ["23514", "23514", "23514"], ngân sách còn 0, chi phí +10000
```

Bản không khoá không báo lỗi nào. Sổ vẫn cân, chỉ quy tắc «ngân sách không âm» bị phá, cùng gốc với write skew ở bài 24: quyết định dựa trên số đã đọc mà không khoá. Dòng thứ ba là lưới an toàn: trigger `budget_floor` (`sql/budget_floor.sql`, chỉ `BEFORE UPDATE` vì số dư được cập nhật bằng `INSERT ... ON CONFLICT DO UPDATE`) chặn số dư âm bằng SQLSTATE `23514`. Nhưng Promotion nhận lỗi hạ tầng mờ, nên trigger không thay được lệnh kiểm trong code.

Test không kiểm ai thắng hay thời gian, chỉ kiểm bất biến: số thành công, số bị từ chối, số dư cuối, «đã tiêu + còn lại = đã cấp». Pool phải có ít nhất bằng số tác vụ cùng chờ ở `Barrier`, nếu không chúng chờ nhau mãi: lab đặt pool 8 cho 5 tác vụ.

## Hợp đồng với Promotion service: ai quyết định gì, theo thứ tự nào, lỗi gì?

Promotion (C#) quyết định mã nào hợp lệ, mức giảm, ai chịu, quota theo lượt dùng. Ledger (Rust) chỉ ghi tiền: nhận hai con số đã quyết, kiểm chúng khớp tiền thật, ghi hoặc từ chối. Hình 3 là thứ tự.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 3. Thứ tự giữa các service: Promotion C# quyết định mức giảm rồi gọi Ledger Rust để giữ ngân sách và ghi bút toán promo; sau khi Ledger trả OK, Payment C# mới thu 20.500 và phát PaymentCaptured, Ledger ghi capture; Trip C# phát TripCompleted, Ledger kiểm tiền thu bằng cước trừ giảm rồi ghi split. Huỷ trước capture thì Promotion gửi release và Ledger đảo bút toán promo."><defs><marker id="rs45-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="30" width="185" height="84" rx="10" class="f-phys-soft s-phys"/><text x="32" y="56" class="t-b">1. Promotion (C#)</text><text x="32" y="76" class="t-small">quyết định mức giảm</text><text x="32" y="94" class="t-small">và ai chịu (hai con số)</text><rect x="238" y="30" width="185" height="84" rx="10" class="f-pk-soft s-pk"/><text x="250" y="56" class="t-b">2. Ledger (Rust)</text><text x="250" y="76" class="t-small">RedeemPromotion: giữ</text><text x="250" y="94" class="t-small">ngân sách, ghi bút toán</text><rect x="456" y="30" width="185" height="84" rx="10" class="f-phys-soft s-phys"/><text x="468" y="56" class="t-b">3. Payment (C#)</text><text x="468" y="76" class="t-small">VíXanh thu 20.500</text><text x="468" y="94" class="t-small">phát PaymentCaptured</text><rect x="674" y="30" width="185" height="84" rx="10" class="f-phys-soft s-phys"/><text x="686" y="56" class="t-b">4. Trip (C#)</text><text x="686" y="76" class="t-small">TripCompleted: cước</text><text x="686" y="94" class="t-small">gốc 25.500, tài xế</text><path d="M205 72 L236 72" class="f-none s-phys" stroke-width="2" marker-end="url(#rs45-2-a)"/><path d="M423 72 L454 72" class="f-none s-phys" stroke-width="2" marker-end="url(#rs45-2-a)"/><path d="M641 72 L672 72" class="f-none s-phys" stroke-width="2" marker-end="url(#rs45-2-a)"/><rect x="20" y="150" width="185" height="70" rx="10" class="f-none s-line dash"/><text x="32" y="176" class="t-small">quota lượt dùng, mã,</text><text x="32" y="194" class="t-small">đối tượng: của Promotion</text><rect x="238" y="150" width="185" height="70" rx="10" class="f-pk-soft s-pk"/><text x="250" y="176" class="t-b">bút toán promo</text><text x="250" y="194" class="t-small">hoặc lỗi hết ngân sách</text><rect x="456" y="150" width="185" height="70" rx="10" class="f-pk-soft s-pk"/><text x="468" y="176" class="t-b">bút toán capture</text><text x="468" y="194" class="t-small">đúng số VíXanh thu</text><rect x="674" y="150" width="185" height="70" rx="10" class="f-pk-soft s-pk"/><text x="686" y="176" class="t-b">bút toán split</text><text x="686" y="194" class="t-small">kiểm thu = cước - giảm</text><path d="M112 114 L112 150" class="f-none s-line dash" stroke-width="2" marker-end="url(#rs45-2-a)"/><path d="M330 114 L330 150" class="f-none s-phys dash" stroke-width="2" marker-end="url(#rs45-2-a)"/><path d="M548 114 L548 150" class="f-none s-phys dash" stroke-width="2" marker-end="url(#rs45-2-a)"/><path d="M766 114 L766 150" class="f-none s-phys dash" stroke-width="2" marker-end="url(#rs45-2-a)"/><rect x="20" y="240" width="840" height="48" rx="10" class="f-warn-soft s-warn"/><text x="36" y="262" class="t-b">Huỷ trước capture</text><text x="36" y="280" class="t-small">Promotion gửi release, Ledger đảo bút toán promo và ngân sách quay lại.</text></svg>
</div>
<figcaption>Hình 3. Redeem là lời gọi đồng bộ vì Promotion cần câu trả lời trước khi khách thấy giá; capture và split đến qua sự kiện Kafka như ở <a href="/articles/rust-kafka-rdkafka-consumer-producer-schema">bài 36</a> và <a href="/articles/rust-outbox-inbox-idempotent-consumer">bài 37</a>.</figcaption>
</figure>

Redeem phải xảy ra trước khi Payment thu tiền: hết ngân sách thì Promotion báo giá 25.500 ngay, thay vì để khách thấy 20.500 rồi bị đổi giá. `PaymentCaptured` và `TripCompleted` đi hai luồng nên có thể lệch thứ tự: split đến trước capture nhận `TRIP_NOT_CAPTURED`, consumer giữ lại rồi thử lại sau. Ledger kiểm bốn điều: phần chịu không âm, phần tài xế chịu không vượt phần tài xế nhận, tiền VíXanh đã thu bằng cước trừ mức giảm, và phần nền tảng chịu bằng số đã redeem. Hai điều cuối bắt được hai service nói hai số khác nhau:

```text
  tiền thu lệch cước trừ giảm: cước 25.500 VND trừ giảm 5.000 VND phải là 20.500 VND, nhưng VíXanh đã thu 25.500 VND
```

Lỗi trả về theo cách ở [bài 35](/articles/rust-tonic-grpc-get-balance-post-entry-interop-dotnet): mã gRPC cho cách xử lý chung, mã nghiệp vụ để bên gọi rẽ nhánh mà không đọc câu chữ.

| Tình huống | gRPC | Mã nghiệp vụ | Gửi lại y nguyên? |
| --- | --- | --- | --- |
| Hết ngân sách | `FAILED_PRECONDITION` | `PROMO_BUDGET_EXHAUSTED` | Không: số dư không tự đầy lên |
| Tiền thu lệch cước trừ giảm, thiếu redeem | `FAILED_PRECONDITION` | `PROMO_AMOUNT_MISMATCH`, `PROMO_NOT_REDEEMED` | Không, sửa nguồn trước |
| Split đến trước capture | `FAILED_PRECONDITION` | `TRIP_NOT_CAPTURED` | Có, sau khi capture đến |
| Phần chịu sai | `INVALID_ARGUMENT` | `PROMO_SHARES_INVALID` | Không |
| Cùng key, khác nội dung | `ALREADY_EXISTS` | `IDEMPOTENCY_CONFLICT` | Không |
| Lỗi database | `INTERNAL` | `LEDGER_INTERNAL` | Có |

Lỗi hết ngân sách mang đủ dữ kiện để Promotion quyết định:

```text
  campaign RACE-LOCK hết ngân sách: cần 5.000 VND, còn 0 VND
  gRPC FAILED_PRECONDITION, mã PROMO_BUDGET_EXHAUSTED, gửi lại được: false; replay yêu cầu đã thắng vẫn Ok: true
```

Lần bị từ chối rollback nên không để lại dấu vết. Bản phác lời gọi `RedeemPromotion` (không phải code lab; lab gọi hàm Rust tương đương):

```json
{ "tripId": "TN-0002", "campaign": "WELCOME5K",
  "platformShareDong": 5000, "driverShareDong": 0 }
```

## Hoàn chuyến: ngân sách trở lại và chi phí được đảo thế nào?

`reverse_trip` đảo từng bút toán gốc của chuyến, mới nhất trước, bằng `JournalEntry::reversal_of` của `ledger-core`; bút toán `promo` bị đảo nên ngân sách quay lại và chi phí được đảo (rút gọn: bỏ đoạn đọc posting từ database):

```rust
// src/promo.rs
let original = JournalEntry::try_new(&key, postings)?;
let entry_id = EntryId::parse(&format!("JE-{id:06}")).ok_or_else(|| {
    PostError::InvalidShares(format!("mã bút toán {id} không hợp lệ"))
})?;
let reversal = JournalEntry::reversal_of(entry_id, &original)?;
out.push(self.post(&reversal).await?); // bút toán đảo không đi qua kiểm ngân sách
```

Key của bút toán đảo là `<key gốc>:reversal`, nên gọi hoàn chuyến lần hai chỉ ra replay. Output thật:

```text
  JE-000026 trip:TN-0002:split:reversal
  JE-000027 trip:TN-0002:capture:reversal
  JE-000028 trip:TN-0002:promo:WELCOME5K:reversal
  gọi lần hai: toàn replay = true; ngân sách còn 10000
  chuyến + hoàn chuyến                  0        0        0        0        0        0        0
```

Dòng cuối cộng thay đổi của chuyến với hoàn chuyến cho bảy tài khoản: toàn 0. Refund một phần, chargeback và maker-checker là việc của [bài 43](/articles/rust-refund-reversal-chargeback-but-toan-dao-maker-checker).

## Chạy ví dụ và kiểm chứng

Từ gốc repo:

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
cargo run  --manifest-path examples/toi-noi-rust/45-promo-budget/Cargo.toml
cargo run  --manifest-path examples/toi-noi-rust/45-promo-budget/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/45-promo-budget/Cargo.toml
```

`cargo test` chạy 7 test đơn vị (không cần database) và 14 ca trên PostgreSQL thật. `--verify` in `VERIFY OK`. Mỗi ca luôn dọn schema, kể cả khi trả `Err`, nhờ `with_db` nhận `impl AsyncFnOnce(&TestDb) -> Result<T>`. Thiếu biến môi trường thì lab **fail**, không bỏ qua:

```text
$ env -u TOI_NOI_PG_URL cargo run -- --verify
LỖI: thiếu TOI_NOI_PG_URL. Bật PostgreSQL lab: docker compose -f examples/toi-noi-rust/infra/docker-compose.yml up -d --wait, rồi export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
```

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Cách ghi nhận khuyến mãi ở đây là mô hình học tập, không phải khuyến nghị kế toán hay thuế. Lab không dựng số dư mở kỳ nên <code>platform:treasury</code> âm, thứ bảng metric của bài 41 coi là bất thường. Chi phí ghi từ lúc redeem, nên trước split nó chưa có ví đối ứng. <code>settle</code> nhận <code>Shares</code> làm tham số vì chưa có bảng lưu phần chịu, và chưa có inbox chờ đủ dữ kiện khi chúng lệch thứ tự. Chưa có gRPC hay Kafka thật, chưa có hết hạn campaign, chưa xử lý hoàn chuyến sau khi tài xế đã rút tiền. Khoá một dòng ngân sách nóng làm các yêu cầu cùng campaign xếp hàng; đánh đổi này chưa được đo.</p></div>

**Bài tập:** chạy 8 yêu cầu × 3.000 đồng vào ngân sách 10.000, một lần bản có khoá, một lần bản không khoá dựng bằng `Barrier`. Dự đoán số yêu cầu thành công, số dư ngân sách cuối và chi phí tăng thêm của từng bản. Pool cần bao nhiêu kết nối?

<details><summary>Xem lời giải</summary><p>Bản có khoá: 3 thành công (9.000 vừa trong 10.000), 5 nhận <code>BudgetExhausted</code>, ngân sách còn 1.000, chi phí +9.000. Bản không khoá: cả 8 cùng đọc 10.000 nên cả 8 cùng qua, ngân sách còn <code>10.000 - 24.000 = -14.000</code>, chi phí +24.000. Bản này cần pool từ 8 kết nối vì cả 8 transaction mở cùng lúc; lab dùng 12.</p></details>

Bài trước: [bài 44](/articles/rust-vi-tai-xe-chia-tien-payout-theo-lo). Bài tiếp: [bài 46](/articles/rust-lam-tron-chia-tien-largest-remainder-khong-mat-dong).

## Tài liệu gốc

- [docs.rs: `tokio::sync::Barrier`](https://docs.rs/tokio/1.54.1/tokio/sync/struct.Barrier.html) và [`tokio::task::JoinSet`](https://docs.rs/tokio/1.54.1/tokio/task/struct.JoinSet.html).
- [docs.rs: sqlx 0.9.0](https://docs.rs/sqlx/0.9.0/sqlx/), `query_scalar` và `Transaction`.
- [PostgreSQL: khoá mức dòng (`FOR UPDATE`)](https://www.postgresql.org/docs/18/explicit-locking.html#LOCKING-ROWS).
- [PostgreSQL: mức cô lập READ COMMITTED](https://www.postgresql.org/docs/18/transaction-iso.html#XACT-READ-COMMITTED).
- [PostgreSQL: mã lỗi SQLSTATE](https://www.postgresql.org/docs/18/errcodes-appendix.html).
- [std: `AsyncFnOnce`](https://doc.rust-lang.org/std/ops/trait.AsyncFnOnce.html).
