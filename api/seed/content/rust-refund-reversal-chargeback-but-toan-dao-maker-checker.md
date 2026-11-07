---
slug: rust-refund-reversal-chargeback-but-toan-dao-maker-checker
title: "Refund, reversal và chargeback: bút toán đảo và maker-checker"
summary: "Bài 43: đảo toàn phần có reversal_of và không đảo được hai lần, hoàn một phần tách theo tỷ lệ, tổng hoàn không vượt số đã thu khi duyệt đồng thời, chargeback từ VíXanh và maker-checker, chạy trên PostgreSQL."
category: rust-advanced
level: advanced
tags: [rust, toi-noi, ledger, refund, chargeback, postgresql]
language: vi
---

Khách `CU-001` khiếu nại chuyến `TN-0001` sau khi 25.500 đồng đã được chia: 5.100 vào `platform:revenue`, 20.400 vào ví tài xế `DR-001`. Nhân viên hỗ trợ bấm «hoàn tiền». Sổ cái chỉ thêm, không có dòng nào để xoá, nên phải trả lời được: hoàn bao nhiêu, lấy từ ai, và ai cho phép. Một tuần sau, VíXanh còn có thể tự rút lại đúng khoản ấy qua chargeback, khi tiền đã nằm ở ví tài xế.

Đây là bài 43 của lộ trình **Rust nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Lab dùng dòng tiền hai bước của loạt (capture rồi split, [bài 13](/articles/rust-double-entry-journal-entry-posting)), schema và idempotency của [bài 24](/articles/rust-transaction-isolation-append-only-idempotency-key) và [bài 25](/articles/rust-schema-so-cai-accounts-journal-postings-partition), chạy trên PostgreSQL thật. [Bài 42](/articles/rust-hold-capture-void-so-du-kha-dung) xử lý tiền còn đang giữ; bài này xử lý tiền đã thu.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Sổ cái cấm UPDATE và DELETE. Muốn «xoá» bút toán thu cước sai, ta làm gì? Và nếu nhân viên bấm «đảo» hai lần thì sổ có hai bút toán đảo không?</p>
<details><summary>Xem đáp án</summary><p>Ghi một bút toán mới đổi dấu mọi dòng và trỏ về bút toán gốc bằng <code>reversal_of</code>. Bấm hai lần không được tạo bản thứ hai: bài này chặn ở ba tầng, tầng cuối là chỉ mục duy nhất trong database.</p></details>
</div>

## Sổ không xoá: ba cách trả tiền lại cho khách

Cả ba cách đều là bút toán mới; khác nhau ở người khởi xướng và liên kết.

| Cách | Ai khởi xướng | Bút toán | `reversal_of` |
| --- | --- | --- | --- |
| Đảo toàn phần (reversal) | Nền tảng, khi coi giao dịch chưa từng hợp lệ | Đảo từng bút toán capture, split | Có, trỏ về bút toán gốc |
| Hoàn một phần (refund) | Nhân viên tạo, người khác duyệt | Một bút toán mới theo tỷ lệ | Không |
| Chargeback | VíXanh, không hỏi nền tảng | Tiền rời clearing | Chỉ khi nền tảng thắng và đảo lại |

Hoàn một phần không đặt `reversal_of`, vì hai lý do. Nó không phải ảnh gương của bút toán nào: ba dòng khách, doanh thu, tài xế không đổi dấu đúng capture hay split. Và chỉ mục duy nhất ở mục sau chỉ cho mỗi bút toán một bản đảo, trong khi một chuyến có thể hoàn nhiều lần. Truy vết đi qua `refund_requests.entry_id` và key `trip:TN-0001:refund:<n>`, với `<n>` là mã yêu cầu.

## Đảo toàn phần: reversal_of và không đảo được hai lần

Đảo cả chuyến là đảo `split` rồi `capture`, ngược thứ tự lúc ghi. `reverse_entry` đổi dấu từng dòng bằng `JournalEntry::reversal_of` của `ledger-core` và ghi với key `<key gốc>:reversal`; kiểm tra bất biến (tổng bằng 0) vẫn do `JournalEntry` làm.

```rust
// src/ledger.rs (rút gọn: bỏ truy vấn lấy bút toán, giữ phần quyết định)
pub async fn reverse_entry(tx: &mut PgConnection, original_id: i64) -> Result<Posted, RefundError> {
    // ... đọc `key` và `reversal_of` của bút toán gốc ...
    if reversal_of.is_some() {
        return Err(RefundError::CannotReverseReversal(original_id));
    }
    let original = JournalEntry::try_new(&key, postings)?;
    let eid = EntryId::parse(&format!("JE-{original_id:06}")).ok_or(LedgerError::Overflow)?;
    let reversal = JournalEntry::reversal_of(eid, &original)?;
    post_entry(tx, &reversal).await
}
```

Không đảo được hai lần nhờ ba lớp. Một: key suy ra từ bút toán gốc, nên gọi lần hai rơi vào `ON CONFLICT DO NOTHING` của `post_entry` và trả `replayed = true`. Hai: không đảo bút toán đã là bút toán đảo. Ba, ở dữ liệu, cho ai đó đi đường vòng với key khác:

```sql
-- sql/01_schema.sql
CREATE UNIQUE INDEX journal_entries_one_reversal
    ON journal_entries (reversal_of) WHERE reversal_of IS NOT NULL;
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 235" role="img" aria-label="Hình 1. Bốn bút toán của chuyến TN-0001: JE-000001 capture và JE-000002 split là bút toán gốc; JE-000003 đảo JE-000002 và JE-000004 đảo JE-000001. Hai mũi tên cong là reversal_of."><defs><marker id="rust-refund-reversal-chargeback-but-toan-dao-maker-checker-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="55" width="180" height="135" rx="10" class="f-phys-soft s-phys"/><text x="34" y="83" class="t-b">JE-000001</text><text x="34" y="108" class="t-small">capture</text><text x="34" y="130" class="t-small">customer -25.500</text><text x="34" y="152" class="t-small">clearing +25.500</text><rect x="245" y="55" width="180" height="135" rx="10" class="f-phys-soft s-phys"/><text x="259" y="83" class="t-b">JE-000002</text><text x="259" y="108" class="t-small">split</text><text x="259" y="130" class="t-small">clearing -25.500</text><text x="259" y="152" class="t-small">revenue +5.100</text><text x="259" y="174" class="t-small">driver +20.400</text><rect x="470" y="55" width="180" height="135" rx="10" class="f-pk-soft s-pk"/><text x="484" y="83" class="t-b">JE-000003</text><text x="484" y="108" class="t-small">đảo của JE-000002</text><text x="484" y="130" class="t-small">reversal_of = 2</text><text x="484" y="152" class="t-small">mọi dấu đổi ngược</text><rect x="695" y="55" width="180" height="135" rx="10" class="f-pk-soft s-pk"/><text x="709" y="83" class="t-b">JE-000004</text><text x="709" y="108" class="t-small">đảo của JE-000001</text><text x="709" y="130" class="t-small">reversal_of = 1</text><text x="709" y="152" class="t-small">mọi dấu đổi ngược</text><path d="M560 55 C560 15 335 15 335 55" class="f-none s-pk dash" stroke-width="2" marker-end="url(#rust-refund-reversal-chargeback-but-toan-dao-maker-checker-0-a)"/><path d="M785 55 C785 5 110 5 110 55" class="f-none s-pk dash" stroke-width="2" marker-end="url(#rust-refund-reversal-chargeback-but-toan-dao-maker-checker-0-a)"/><text x="20" y="218" class="t-small">Đảo theo thứ tự ngược: split trước, capture sau. Sau JE-000004, mọi tài khoản của chuyến về 0.</text></svg>
</div>
<figcaption>Hình 1. Hai bút toán gốc và hai bút toán đảo. Mũi tên nét đứt là <code>reversal_of</code>; sổ chỉ thêm dòng, không sửa dòng nào.</figcaption>
</figure>

Lab thử bốn cách đảo lần hai và in kết quả thật:

```text
capture JE-000001, split JE-000002: customer -25.500 VND | clearing 0 VND | revenue 5.100 VND | driver 20.400 VND
đảo split -> JE-000003, đảo capture -> JE-000004: customer 0 VND | clearing 0 VND | revenue 0 VND | driver 0 VND
payment REFUNDED, đã hoàn 25500 đồng
reversal_of: [(1, None), (2, None), (3, Some(2)), (4, Some(1))]
đảo chuyến lần hai: bút toán 1 đã được đảo
reverse_entry(JE-000002) lại: JE-000003, replayed=true
đảo bút toán đảo: bút toán 3 là bút toán đảo, không đảo tiếp
chèn tay một bút toán đảo thứ hai: Some((Some("23505"), Some("journal_entries_one_reversal")))
```

`reverse_trip` chỉ chạy khi chuyến chưa hoàn đồng nào và không có yêu cầu đang chờ; nếu không, khách nhận tiền hai lần.

## Hoàn một phần: ai chịu phần phí?

Chính sách của lab là hoàn theo tỷ lệ: khách nhận đủ số hoàn, nền tảng trả lại phần phí tương ứng, tài xế trả phần còn lại; khách không chịu phần nào. Nền tảng chịu hết, hay tài xế chịu hết khi lỗi do tài xế, là quyết định kinh doanh; chỉ `refund_split` cần đổi.

Làm tròn là chỗ dễ lệch: làm tròn riêng từng lần thì nhiều lần hoàn nhỏ có thể cộng ra phí khác 5.100. Lab tính theo tổng luỹ kế: phần phí của một lần là hiệu giữa phí «đáng ra đã hoàn» sau lần này và trước lần này, còn tài xế nhận phần dư, nên tổng đúng từng đồng. Phép tính chỉ dùng số nguyên (`i128` để tích không tràn), làm tròn nửa lên, không `f64`:

```rust
// src/model.rs (gộp dòng cho gọn)
fn fee_target(cumulative: i64, fee: i64, captured: i64) -> i64 {
    let num = i128::from(cumulative) * i128::from(fee) * 2 + i128::from(captured);
    let den = i128::from(captured) * 2;
    i64::try_from(num / den).expect("kết quả không lớn hơn phí, vừa i64")
}

pub fn refund_split(captured: Money, fee: Money, already_refunded: Money, amount: Money) -> RefundSplit {
    let (c, f) = (captured.dong(), fee.dong());
    let before = already_refunded.dong();
    let after = before + amount.dong();
    let fee_part = fee_target(after, f, c) - fee_target(before, f, c);
    RefundSplit { fee_part, driver_part: amount.dong() - fee_part }
}
```

| Hoàn | Đã hoàn trước | Phí nền tảng trả | Tài xế trả |
| --- | --- | --- | --- |
| 10.000 | 0 | 2.000 | 8.000 |
| 15.500 | 10.000 | 3.100 | 12.400 |
| 12.347 | 0 | 2.469 (từ 2.469,4) | 9.878 |

Hai dòng đầu cộng ra đúng 5.100 và 20.400; unit test `pieces_add_up_to_original_fee` thử thêm bảy lần hoàn lẻ liên tiếp. Bút toán hoàn có ba posting (khách nhận, doanh thu và tài xế mỗi bên trả phần mình); dòng 0 đồng bị bỏ vì `CHECK (amount_dong <> 0)`.

## Hai yêu cầu cùng lúc, tổng hoàn vượt số đã thu

Chuyến thu 25.500. Hai yêu cầu 15.000 đều hợp lệ khi tạo; chỉ cộng lại mới vượt. Nếu hai checker duyệt cùng lúc và cùng đọc «đã hoàn 0», cả hai qua kiểm tra.

Có hai lớp chống. Lớp chính là `SELECT ... FOR UPDATE` trên dòng `payments`: ở mức `READ COMMITTED` mặc định, bên đến sau chờ, đọc số mới và nhận lỗi nghiệp vụ rõ ràng. Lớp cuối là `CHECK`, cho trường hợp ai đó quên khoá:

```sql
-- sql/01_schema.sql (rút gọn)
refunded_dong bigint NOT NULL DEFAULT 0,
CONSTRAINT refund_within_captured CHECK (refunded_dong >= 0 AND refunded_dong <= captured_dong)
```

`UPDATE` cộng tương đối (`refunded_dong = refunded_dong + $2`) chứ không ghi đè bằng số vừa đọc, nếu không lost update lọt qua `CHECK`. Cuộc đua dựng bằng `Barrier`, không `sleep`. Với `Guard::Locked`, điểm dừng nằm trước khi mở transaction; với `Guard::Unlocked` (tham số chỉ có ở lab, để thấy `CHECK` làm gì), điểm dừng nằm ngay sau khi đọc `payments`, nên cả hai chắc chắn đọc cùng số cũ.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Hình 2. Hai transaction duyệt hai yêu cầu 15.000 trên cùng chuyến. T1 khoá dòng payments, ghi bút toán và commit. T2 đứng chờ khoá, sau đó đọc refundable còn 10.500 và bị từ chối."><defs><marker id="rust-refund-reversal-chargeback-but-toan-dao-maker-checker-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="25" width="158" height="70" rx="10" class="f-phys-soft s-phys"/><text x="34" y="55" class="t-b">T1 · Barrier</text><text x="34" y="78" class="t-small">fin.binh, yêu cầu 1</text><rect x="195" y="25" width="200" height="70" rx="10" class="f-phys-soft s-phys"/><text x="209" y="55" class="t-b">FOR UPDATE</text><text x="209" y="78" class="t-small">refundable = 25.500</text><rect x="420" y="25" width="210" height="70" rx="10" class="f-phys-soft s-phys"/><text x="434" y="55" class="t-b">Ghi bút toán hoàn</text><text x="434" y="78" class="t-small">refunded 0 → 15.000</text><rect x="655" y="25" width="205" height="70" rx="10" class="f-pk-soft s-pk"/><text x="669" y="55" class="t-b">COMMIT</text><text x="669" y="78" class="t-small">khoá được nhả</text><path d="M178 60 L195 60" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-refund-reversal-chargeback-but-toan-dao-maker-checker-1-a)"/><path d="M395 60 L420 60" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-refund-reversal-chargeback-but-toan-dao-maker-checker-1-a)"/><path d="M630 60 L655 60" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-refund-reversal-chargeback-but-toan-dao-maker-checker-1-a)"/><rect x="20" y="140" width="158" height="70" rx="10" class="f-phys-soft s-phys"/><text x="34" y="170" class="t-b">T2 · Barrier</text><text x="34" y="193" class="t-small">fin.cuong, yêu cầu 2</text><rect x="195" y="140" width="435" height="70" rx="10" class="f-warn-soft s-warn dash"/><text x="209" y="170" class="t-b">FOR UPDATE: đứng chờ</text><text x="209" y="193" class="t-small">dòng payments đang do T1 giữ, T2 chưa đọc được gì</text><rect x="655" y="140" width="205" height="70" rx="10" class="f-warn-soft s-warn"/><text x="669" y="170" class="t-b">refundable = 10.500</text><text x="669" y="193" class="t-small">15.000 &gt; 10.500: từ chối</text><path d="M178 175 L195 175" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-refund-reversal-chargeback-but-toan-dao-maker-checker-1-a)"/><path d="M630 175 L655 175" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-refund-reversal-chargeback-but-toan-dao-maker-checker-1-a)"/><path d="M757 95 L757 140" class="f-none s-pk" stroke-width="2" marker-end="url(#rust-refund-reversal-chargeback-but-toan-dao-maker-checker-1-a)"/><text x="20" y="245" class="t-small">Thiếu khoá: cả hai cùng đọc 25.500 và cùng qua kiểm; CHECK refund_within_captured chặn bên ghi sau (SQLSTATE 23514).</text></svg>
</div>
<figcaption>Hình 2. Với khoá, bên thua nhận lỗi nghiệp vụ. Ai thắng thì tuỳ lần chạy, nhưng tổng hoàn luôn là 15.000. Hình vẽ T1 thắng.</figcaption>
</figure>

Kết quả thật của lab, hai biến thể liền nhau:

```text
-- 3. Hai yêu cầu 15.000 duyệt đồng thời (Locked)
thành công 1/2; bên thua: yêu cầu hoàn 15.000 VND vượt phần còn hoàn được 10.500 VND
đã hoàn 15000, bút toán hoàn 1, payment REFUND_PENDING, customer -10.500 VND
-- 3. Hai yêu cầu 15.000 duyệt đồng thời (Unlocked)
thành công 1/2; bên thua: lỗi database, SQLSTATE Some("23514"), constraint Some("refund_within_captured")
đã hoàn 15000, bút toán hoàn 1, payment REFUND_PENDING, customer -10.500 VND
```

Cả hai đều an toàn; khác ở chất lượng lỗi. Bản không khoá trả lỗi database khó dịch thành thông điệp cho nhân viên, và rollback sau khi đã làm gần hết việc. Test không kiểm bên nào thắng, chỉ kiểm bất biến: một bút toán hoàn, tổng 15.000, yêu cầu bên thua vẫn `PENDING`. Thứ tự khoá cố định (dòng yêu cầu rồi dòng `payments`) để không khoá chéo.

## Maker-checker: người tạo khác người duyệt

Maker-checker ở đây là bảng `refund_requests` với người tạo, người duyệt, trạng thái, và ràng buộc để chính database giữ luật, không chỉ code Rust.

```sql
-- sql/01_schema.sql (rút gọn)
status      text NOT NULL DEFAULT 'PENDING' CHECK (status IN ('PENDING', 'APPROVED', 'REJECTED')),
created_by  text NOT NULL CHECK (created_by <> ''),
decided_by  text,
CONSTRAINT checker_differs_from_maker CHECK (decided_by IS NULL OR decided_by <> created_by),
CONSTRAINT decision_matches_status CHECK (
    (status = 'PENDING'  AND decided_by IS NULL AND decided_at IS NULL AND entry_id IS NULL)
    OR (status = 'APPROVED' AND decided_by IS NOT NULL AND decided_at IS NOT NULL AND entry_id IS NOT NULL)
    OR (status = 'REJECTED' AND decided_by IS NOT NULL AND decided_at IS NOT NULL AND entry_id IS NULL))
```

Luồng duyệt trong `approve_with` (rút gọn, bỏ phần dựng bút toán đã in ở trên):

```rust
// src/refund.rs (rút gọn)
let mut tx = self.pool.begin().await?;
let req = lock_request(&mut tx, request_id).await?;          // FOR UPDATE trên dòng yêu cầu
if req.created_by == checker {
    tx.rollback().await?;
    // Lần bị từ chối vẫn phải để lại dấu vết: ghi ngoài transaction đã rollback.
    // Hai bước không nguyên tử: chết giữa chừng thì tiền không đổi nhưng mất dòng audit này.
    audit(&mut self.pool.acquire().await?, checker, "APPROVAL_DENIED", &subject, "...").await?;
    return Err(RefundError::SelfApproval { actor: checker.to_string() });
}
let pay = load_payment(&mut tx, &req.trip_id, guard == Guard::Locked).await?;
if req.status == "APPROVED" {
    return Ok(Approval { request_id, entry_id: req.entry_id.unwrap_or_default(), replayed: true, status: pay.status });
}
```

Rollback rồi `INSERT` là hai bước không nguyên tử. Tiến trình chết giữa hai bước, hoặc `INSERT` lỗi, thì tiền không đổi và không có bút toán nào, nhưng lần bị từ chối không để lại dòng audit; `INSERT` lỗi còn khiến người gọi nhận lỗi database thay vì `SelfApproval`. Dấu vết này chỉ là cố gắng tối đa, khác với audit của thay đổi, vốn cùng commit với thay đổi. Hệ thống thật nên ghi thêm log có cấu trúc trước khi ghi database. `reject` xử lý người tạo tự từ chối cũng theo cách này.

Duyệt lặp lại idempotent theo hai cách: khoá dòng yêu cầu xếp hàng hai checker cùng bấm, và key `trip:TN-0001:refund:<mã yêu cầu>` là lưới thứ hai. Lab thử duyệt lại bởi cùng người, bởi người khác, và đồng thời: luôn đúng một bút toán. Rust báo lỗi dễ hiểu, còn `UPDATE` thẳng để người duyệt trùng người tạo vẫn bị `checker_differs_from_maker` chặn, vì có ngày ai đó sẽ viết script sửa dữ liệu.

Trạng thái thanh toán đi theo máy trạng thái của [bài 14](/articles/rust-typestate-may-trang-thai-thanh-toan): tạo yêu cầu đưa `CAPTURED` sang `REFUND_PENDING`; hoàn đủ số đã thu thì sang `REFUNDED`. Hoàn một phần xong mà không còn yêu cầu chờ, hoặc mọi yêu cầu đều bị từ chối, thì phải về `CAPTURED`; bảng `transition` của `ledger-core` không có cạnh `REFUND_PENDING` về `CAPTURED`. Cạnh này do lab tự thêm: `approve_with` và `reject` gán thẳng, không qua `transition`, và lab 20 giữ nguyên. Thư viện vì thế không bảo vệ cạnh này, kiểm tra nằm ở code gọi. Nghiệp vụ thật đòi cạnh mà mô hình ban đầu chưa nghĩ tới; đưa nó vào thư viện chung là một quyết định riêng.

Audit ghi cùng transaction với thay đổi, trừ các lần bị từ chối ở trên. `audit_log` cũng bị trigger cấm sửa và xoá như `journal_entries`:

```text
  system:payment TRIP_CAPTURED     trip:TN-0001
  cs.lan       REFUND_REQUESTED  refund_request:1
  cs.lan       APPROVAL_DENIED   refund_request:1
  fin.binh     REFUND_APPROVED   refund_request:1
  cs.lan       REFUND_REQUESTED  refund_request:2
  fin.binh     REFUND_APPROVED   refund_request:2
```

## Chargeback: tiền rời clearing, ai chịu tạm?

Chargeback không qua maker-checker vì không ai hỏi nền tảng: VíXanh rút 25.500 khi nền tảng đã chia xong. Lab ghi bút toán hai dòng: `psp:vixanh:clearing` -25.500, `customer:CU-001` +25.500. Clearing về 0 sau split nên đi âm, và số âm đó là chỗ nền tảng chịu tạm: tiền nền tảng nợ VíXanh trong khi ví tài xế vẫn giữ 20.400. Không ai tự trừ ví tài xế; đòi lại là quyết định khi có kết luận.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 230" role="img" aria-label="Hình 3. Số dư bốn tài khoản qua ba thời điểm: sau split, khi VíXanh rút 25.500 (clearing âm 25.500, tài xế vẫn giữ 20.400), và sau khi nền tảng thua và thu hồi từ tài xế (mọi tài khoản về 0)."><defs><marker id="rust-refund-reversal-chargeback-but-toan-dao-maker-checker-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="25" width="250" height="150" rx="10" class="f-phys-soft s-phys"/><text x="36" y="55" class="t-b">Sau split</text><text x="36" y="85" class="t-small">customer -25.500</text><text x="36" y="108" class="t-small">clearing 0</text><text x="36" y="131" class="t-small">revenue +5.100</text><text x="36" y="154" class="t-small">driver +20.400</text><rect x="315" y="25" width="250" height="150" rx="10" class="f-warn-soft s-warn"/><text x="331" y="55" class="t-b">VíXanh rút 25.500</text><text x="331" y="85" class="t-small">customer 0</text><text x="331" y="108" class="t-b">clearing -25.500</text><text x="331" y="131" class="t-small">revenue +5.100</text><text x="331" y="154" class="t-small">driver +20.400</text><rect x="610" y="25" width="250" height="150" rx="10" class="f-pk-soft s-pk"/><text x="626" y="55" class="t-b">Thua, thu hồi từ tài xế</text><text x="626" y="85" class="t-small">customer 0</text><text x="626" y="108" class="t-small">clearing 0</text><text x="626" y="131" class="t-small">revenue 0</text><text x="626" y="154" class="t-small">driver 0</text><path d="M270 100 L315 100" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-refund-reversal-chargeback-but-toan-dao-maker-checker-2-a)"/><path d="M565 100 L610 100" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-refund-reversal-chargeback-but-toan-dao-maker-checker-2-a)"/><text x="20" y="205" class="t-small">Giữa hai thời điểm sau, nền tảng chịu tạm khoản âm ở clearing; ví tài xế chưa bị đụng tới.</text></svg>
</div>
<figcaption>Hình 3. Chargeback mở rồi kết luận. Vụ việc được ghi theo <code>psp_case_ref</code> của VíXanh, nên webhook gửi lại không nhân đôi bút toán (xem <a href="/articles/rust-webhook-psp-hmac-chong-replay-xu-ly-tre-trung">bài 38</a>).</figcaption>
</figure>

Kết luận có ba hướng, đều in ra từ lab:

| Kết luận | Bút toán | customer / clearing / revenue / driver |
| --- | --- | --- |
| Thua, thu hồi từ tài xế | `driver` và `revenue` trả theo tỷ lệ, clearing +25.500 | 0 / 0 / 0 / 0 |
| Thua, nền tảng chịu | `revenue` -25.500, clearing +25.500 | 0 / 0 / -20.400 / +20.400 |
| Thắng | Đảo bút toán chargeback (`reversal_of`) | -25.500 / 0 / +5.100 / +20.400 |

Doanh thu -20.400 ở hàng hai là khoản lỗ nền tảng chịu: tài xế giữ tiền, nền tảng mất cả phần của tài xế. Sơ đồ tài khoản chuẩn chưa có tài khoản chi phí riêng cho chargeback nên lab trừ thẳng `platform:revenue`. Hàng một tái dùng `refund_split`.

Các ca lab chưa xử lý, cần chính sách riêng:

- **Ví tài xế âm.** Thu hồi hoặc hoàn một phần khi tài xế đã rút hết tiền để lại số dư âm. Ví âm không vào lô payout ở [bài 44](/articles/rust-vi-tai-xe-chia-tien-payout-theo-lo), nhưng chưa có khấu trừ hay đòi nợ.
- **Hoàn tiền chồng chargeback.** `open` chỉ so với số đã thu, không trừ số đã hoàn; chargeback đang mở cũng không chặn duyệt hoàn. Khách có thể được trả hai lần.
- **Nhiều vụ trên một chuyến.** Mỗi `psp_case_ref` chỉ so với số đã thu, nên tổng các vụ có thể vượt số đó.
- **Một người quyết.** `reverse_trip` và `resolve_lost` chỉ ghi `actor`, không qua maker-checker.

## Reason code là enum có bảng

Mỗi yêu cầu hoàn và chargeback mang một reason code, giữ ở hai nơi: enum `ReasonCode` (đóng, thêm mã thì compiler chỉ chỗ phải sửa) và bảng `reason_codes` (khoá ngoại từ `refund_requests` và `chargebacks`, nên mã lạ không vào được database). Hai nơi có thể lệch, nên test so theo tập hợp:

```rust
// src/main.rs (thí nghiệm 5, rút gọn)
let mut table: Vec<(String, String)> = rows.iter().map(|r| (r.get("code"), r.get("kind"))).collect();
let mut en: Vec<(String, String)> = ReasonCode::ALL.iter()
    .map(|r| (r.as_str().to_string(), r.kind().as_str().to_string())).collect();
table.sort();
en.sort();
c.expect(table == en, "enum ReasonCode khớp bảng reason_codes");
```

`kind()` tách mã hoàn tiền (`FARE_DISPUTE`, `DUPLICATE_CHARGE`...) khỏi mã chargeback (`FRAUD`, `NOT_RECEIVED`); dùng nhầm loại là `WrongReasonKind`. Có mã thay vì ghi chú tự do thì báo cáo gom được theo nguyên nhân.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Đây là mô hình học tập. Người dùng chỉ là chuỗi (<code>cs.lan</code>, <code>fin.binh</code>), chưa có xác thực, phân quyền hay MFA theo §11.2 của kiến trúc; thực tế, ai được duyệt là dữ liệu phân quyền, không phải so sánh hai chuỗi. Lab hoàn về ví khách; hoàn về thẻ phải qua VíXanh, có trạng thái chờ và đối soát. Chưa có hạn mức duyệt theo số tiền hay hết hạn yêu cầu. Mọi tên và số liệu là hư cấu của Tới Nơi; quy tắc thật phụ thuộc hợp đồng với cổng thanh toán và quy định, không suy ra từ bài này.</p>
</div>

## Chạy ví dụ và kiểm chứng

Lab dùng PostgreSQL của `examples/toi-noi-rust/infra`; mỗi thí nghiệm tạo schema `lab_<uuid>` riêng và xoá khi xong. Thiếu `TOI_NOI_PG_URL` thì lab fail rõ. Từ gốc repository:

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
cargo run --manifest-path examples/toi-noi-rust/43-refund-reversal-chargeback/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/43-refund-reversal-chargeback/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/43-refund-reversal-chargeback/Cargo.toml
```

`--verify` kiểm từng giá trị in ở trên; đạt thì in `VERIFY OK`, thoát mã 0. `cargo test` chạy 4 unit test (tách phí, reason code) và 11 test tích hợp, gồm trigger append-only, `CHECK` ở database và một cuộc đua ba checker bằng `Barrier` kiểm bất biến chứ không kiểm thứ tự. Lab dùng `sqlx` 0.9.0, `tokio` 1.54.1 và `ledger-core` của bài 20.

**Bài tập:** chuyến đã hoàn 10.000 qua yêu cầu 1. Sau đó có người gọi `reverse_trip`. Lab trả gì, và vì sao đó đúng hơn việc đảo hai bút toán rồi trừ phần đã hoàn?

<details><summary>Xem lời giải</summary><p>Lab trả <code>PartialRefundExists</code> với số đã hoàn 10.000, vì <code>reverse_trip</code> chỉ chạy khi <code>refunded_dong = 0</code> và trạng thái <code>CAPTURED</code>. «Trừ phần đã hoàn» nghĩa là sửa bút toán đã ghi hoặc ghi bù một khoản mất dấu nguyên nhân, đều khó đối soát. Đường đúng là hoàn nốt 15.500 qua yêu cầu mới, vẫn có người tạo, người duyệt và audit.</p></details>

Bài trước: [bài 42](/articles/rust-hold-capture-void-so-du-kha-dung). Bài tiếp: [bài 44](/articles/rust-vi-tai-xe-chia-tien-payout-theo-lo).

## Tài liệu gốc

- [PostgreSQL: Table Constraints (CHECK, UNIQUE, partial index)](https://www.postgresql.org/docs/current/ddl-constraints.html) và [Partial Indexes](https://www.postgresql.org/docs/current/indexes-partial.html).
- [PostgreSQL: SELECT, mục The Locking Clause](https://www.postgresql.org/docs/current/sql-select.html#SQL-FOR-UPDATE-SHARE) và [Explicit Locking](https://www.postgresql.org/docs/current/explicit-locking.html).
- [PostgreSQL: Appendix A, bảng mã lỗi SQLSTATE](https://www.postgresql.org/docs/current/errcodes-appendix.html).
- [docs.rs: sqlx 0.9, `DatabaseError`](https://docs.rs/sqlx/0.9.0/sqlx/error/trait.DatabaseError.html).
- [docs.rs: tokio, `sync::Barrier`](https://docs.rs/tokio/latest/tokio/sync/struct.Barrier.html).
- [The Rust Programming Language: Defining an Enum](https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html).
