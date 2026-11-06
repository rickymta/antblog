---
slug: rust-double-entry-journal-entry-posting
title: "Double-entry: journal entry, posting và bất biến tổng bằng 0"
summary: "Bài 13: ghi posting bằng số có dấu và ánh xạ sang nợ/có kế toán, dựng JournalEntry chỉ tồn tại khi cân (try_new, builder), chia tiền TN-0001 qua clearing, sửa sai bằng bút toán đảo, tính số dư từ posting."
category: rust
level: beginner
tags: [rust, toi-noi, ledger, double-entry, journal-entry, builder]
language: vi
---

Đây là bài 13 của lộ trình **Rust cơ bản qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Khách `CU-001` trả 25.500 VND cho chuyến `TN-0001` qua VíXanh. Tiền không đi thẳng tới tài xế: nó nằm ở tài khoản clearing của VíXanh, rồi mới chia 5.100 cho nền tảng và 20.400 cho tài xế. Hai bước, hai bút toán, và bút toán nào cũng phải cân. Bài này dựng mô hình đó từ lab 10, dùng các newtype của [bài 12](/articles/rust-newtype-value-object-money-account-id) ở dạng rút gọn như lab 10 (`Money`, `AccountId`, thêm một `EntryId` tối giản), chỉ với thư viện chuẩn.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Một bút toán có posting khách -25.500 và posting clearing +25.000 có thể tồn tại như một giá trị <code>JournalEntry</code> không? Người gọi có cách nào dựng nó mà không qua kiểm tra?</p>
<details><summary>Xem đáp án</summary><p>Không. Mọi trường của <code>JournalEntry</code> là private, nên struct literal ngoài module bị rustc từ chối (E0451). Cả ba cách tạo bút toán (<code>try_new</code>, builder, bút toán đảo) đều đi qua cùng một hàm kiểm tra. Chỉ code trong module <code>entry</code> mới phá được, nên module đó nhỏ và có test.</p></details>
</div>

## Mỗi dòng tiền có hai đầu: vì sao lab ghi số có dấu?

Mỗi đồng chuyển đi có điểm xuất phát và điểm đến. Sổ cái ghi cả hai đầu trong một **bút toán** (journal entry); mỗi đầu là một **posting**. Khách trả 25.500: `customer:CU-001` nhận posting -25.500, `psp:vixanh:clearing` nhận posting +25.500.

Loạt này giữ quy ước của bài 10: posting là số có dấu theo **chiều tiền** đối với tài khoản của nó, dương là vào, âm là ra. Vì đồng tiền không tự sinh ra, tổng mọi posting của một bút toán bằng 0. Đó là bất biến duy nhất cả bài bảo vệ.

Dấu này **không phải** nợ/có. Kế toán gán nợ/có theo loại tài khoản: tài sản tăng bên nợ, nợ phải trả và doanh thu tăng bên có. Cùng là dấu dương, +25.500 ở clearing ứng với ghi nợ, còn +5.100 ở `platform:revenue` ứng với ghi có. Vì vậy lab đặt tên theo đúng thứ nó đo, `Flow`, và không có kiểu nào tên `Debit` hay `Credit`:

```rust
// src/entry.rs (rút gọn: bỏ chú thích tài liệu)
pub enum Flow { In, Out }

impl Posting {
    pub fn flow(&self) -> Option<Flow> {
        match self.amount.dong() {
            n if n > 0 => Some(Flow::In),
            n if n < 0 => Some(Flow::Out),
            _ => None,
        }
    }
}
```

## Chart of accounts rút gọn: bốn tài khoản, ba loại

Chart of accounts là danh sách tài khoản và loại của chúng. Bản rút gọn của Tới Nơi có bốn tài khoản. Cột cuối là ánh xạ sang ngôn ngữ kế toán:

| Tài khoản | Loại | Số dương nghĩa là | Số dư tăng, kế toán ghi |
| --- | --- | --- | --- |
| `customer:CU-001` | liability | ví khách tăng, nền tảng nợ khách thêm; số âm là phần khách đã chi | bên có |
| `psp:vixanh:clearing` | asset | tiền VíXanh đang giữ, chờ về nền tảng | bên nợ |
| `platform:revenue` | revenue | phí nền tảng đã kiếm | bên có |
| `driver:DR-001` | liability | nền tảng nợ tài xế, chờ trả | bên có |

```rust
// src/chart.rs (rút gọn: bỏ `label` và test)
pub enum AccountKind { Asset, Liability, Revenue }

pub fn kind_of(account: &AccountId) -> Option<AccountKind> {
    let id = account.as_str();
    if id.starts_with("customer:") || id.starts_with("driver:") {
        Some(AccountKind::Liability)
    } else if id == "platform:revenue" {
        Some(AccountKind::Revenue)
    } else if id.starts_with("psp:") {
        Some(AccountKind::Asset)
    } else {
        None
    }
}
```

Loại tài khoản giúp đọc số dư và lập báo cáo, nhưng **không tham gia kiểm cân**: bất biến chỉ nhìn số tiền có dấu. Nhờ vậy luật cân gói gọn trong một hàm và không phụ thuộc chart.

Bảng trên dùng để đọc số dư từng tài khoản, không phải để dịch từng bút toán. Thử dịch là thấy: bút toán chia ở mục sau có cả ba dòng rơi vào bên có. Sổ của loạt là sổ **dòng tiền**, coi mọi tài khoản như một ngăn chứa tiền; sổ kế toán chia hai vế tài sản và nguồn vốn, nên cùng chuyến này phải ghi bằng các bút toán khác. Chẳng hạn với mô hình khách trả sau, tài khoản khách là phải thu thay vì ví: chuyến xong thì ghi nợ phải thu khách 25.500, ghi có doanh thu 5.100, ghi có phải trả tài xế 20.400; VíXanh xác nhận thì ghi nợ clearing 25.500, ghi có phải thu khách 25.500. Dựng sổ đó nằm ngoài phạm vi cấp cơ bản.

## Làm sao để JournalEntry chỉ tạo được khi cân?

Câu trả lời gồm hai nửa: trường private và một cửa duy nhất. `JournalEntry` giữ `idempotency_key`, `postings` và `reverses` đều private. Ngoài module `entry`, không ai viết được `JournalEntry { .. }`. Thông điệp thật của rustc (rút gọn, từ một file thử đã xoá):

```text
error[E0451]: fields `idempotency_key`, `postings` and `reverses` of struct `JournalEntry` are private
 --> tests/tmp_err.rs:7:9
  |
6 |     let e = JournalEntry {
  |             ------------ in this type
7 |         idempotency_key: "k".to_string(),
  |         ^^^^^^^^^^^^^^^ private field
```

Truy cập chỉ đọc qua `postings()` trả `&[Posting]`, nên gọi `push` trên đó cũng không biên dịch (E0599, slice không có `push`). Trong module, mọi đường tạo bút toán gọi hàm `validated`:

```rust
// src/entry.rs
fn validated(
    idempotency_key: &str,
    postings: Vec<Posting>,
    reverses: Option<EntryId>,
) -> Result<JournalEntry, LedgerError> {
    if idempotency_key.trim().is_empty() {
        return Err(LedgerError::EmptyIdempotencyKey);
    }
    if postings.len() < 2 {
        return Err(LedgerError::TooFewPostings {
            count: postings.len(),
        });
    }
    let sum = total(&postings)?;
    if sum != Money::ZERO {
        return Err(LedgerError::Unbalanced { sum });
    }
    Ok(JournalEntry {
        idempotency_key: idempotency_key.to_string(),
        postings,
        reverses,
    })
}
```

Hàm này private; `try_new(key, postings)` chỉ gọi nó với `reverses = None`. Tên `try_new` báo rằng hàm tạo có thể thất bại (xem [bài 07](/articles/rust-result-option-toan-tu-hoi-loi-la-gia-tri)); `total` dùng `checked_add`, nên tổng tràn `i64` thành `Err(Overflow)` thay vì quay vòng.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 1. Ba hàm tạo bút toán là try_new, build của builder và reversal_of cùng đi qua hàm private validated; đúng thì ra JournalEntry, sai thì ra Err. Struct literal từ ngoài module bị rustc chặn bằng E0451."><defs><marker id="rust-double-entry-journal-entry-posting-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="30" width="280" height="56" rx="10" class="f-phys-soft s-phys"/><text x="36" y="54" class="t-b">JournalEntry::try_new</text><text x="36" y="73" class="t-small">nhận Vec&lt;Posting&gt;</text><rect x="20" y="105" width="280" height="56" rx="10" class="f-phys-soft s-phys"/><text x="36" y="129" class="t-b">JournalEntryBuilder::build</text><text x="36" y="148" class="t-small">tích lũy posting, kiểm ở cuối</text><rect x="20" y="180" width="280" height="56" rx="10" class="f-phys-soft s-phys"/><text x="36" y="204" class="t-b">JournalEntry::reversal_of</text><text x="36" y="223" class="t-small">đảo dấu posting, trỏ về bản gốc</text><rect x="360" y="40" width="260" height="150" rx="10" class="f-pk-soft s-pk"/><text x="376" y="68" class="t-b">fn validated(...)</text><text x="376" y="94" class="t-small">1. idempotency_key không rỗng</text><text x="376" y="114" class="t-small">2. ít nhất 2 posting</text><text x="376" y="134" class="t-small">3. tổng có dấu bằng 0 (checked)</text><text x="376" y="164" class="t-small">private với bên ngoài</text><rect x="680" y="70" width="180" height="90" rx="10" class="f-phys-soft s-phys"/><text x="696" y="102" class="t-b">JournalEntry</text><text x="696" y="124" class="t-small">trường private</text><text x="696" y="142" class="t-small">luôn cân</text><rect x="360" y="215" width="260" height="60" rx="10" class="f-warn-soft s-warn"/><text x="376" y="238" class="t-b">Err(LedgerError)</text><text x="376" y="256" class="t-small">Unbalanced, TooFewPostings…</text><rect x="20" y="250" width="280" height="40" rx="10" class="f-none s-warn dash"/><text x="36" y="268" class="t-small">JournalEntry { .. } ngoài module:</text><text x="36" y="283" class="t-small">không biên dịch (E0451)</text><path d="M300 58 L360 85" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-double-entry-journal-entry-posting-0-a)"/><path d="M300 133 L360 115" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-double-entry-journal-entry-posting-0-a)"/><path d="M300 208 L360 150" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-double-entry-journal-entry-posting-0-a)"/><path d="M620 115 L680 115" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-double-entry-journal-entry-posting-0-a)"/><text x="634" y="107" class="t-small">đúng</text><path d="M490 190 L490 215" class="f-none s-warn dash" stroke-width="2" marker-end="url(#rust-double-entry-journal-entry-posting-0-a)"/><text x="500" y="206" class="t-small">sai</text></svg>
</div>
<figcaption>Hình 1. Ba cửa vào, một hàm kiểm. Bút toán lệch dừng ở <code>Err</code>; không có đường thứ tư đi vòng.</figcaption>
</figure>

Với người từ C#: cùng ý với constructor kiểm tra tham số ở [bài class và encapsulation](/articles/csharp-class-property-encapsulation) hay `Money` của [bài record struct](/articles/csharp-record-struct-money-quote). Khác biệt chính là cách báo lỗi: constructor C# ném exception, còn `try_new` trả `Result` nằm ngay trong chữ ký.

## Builder: thêm từng posting, kiểm cân một lần ở cuối

Bút toán chia có ba posting; viết `vec![Posting::new(..), ..]` dài dòng và dễ sót. Builder cho phép thêm từng posting rồi kiểm một lần:

```rust
// src/entry.rs (rút gọn: bỏ `new` và chú thích)
pub struct JournalEntryBuilder {
    idempotency_key: String,
    postings: Vec<Posting>,
}

impl JournalEntryBuilder {
    pub fn posting(mut self, account: AccountId, amount: Money) -> JournalEntryBuilder {
        self.postings.push(Posting::new(account, amount));
        self
    }

    pub fn build(self) -> Result<JournalEntry, LedgerError> {
        JournalEntry::try_new(&self.idempotency_key, self.postings)
    }
}
```

Mỗi phương thức nhận `self` theo giá trị rồi trả lại, theo luật ownership của [bài 04](/articles/rust-ownership-move-borrow-but-toan): builder bị tiêu thụ ở `build`, không dùng lại được sau đó. Builder **không kiểm giữa chừng** vì sau posting đầu tiên bút toán hiển nhiên chưa cân; nó cũng không có luật riêng, `build` gọi `try_new` nên hai đường không thể lệch nhau. Test `builder_matches_try_new` kiểm đúng điều này.

## Dòng tiền hai bước: capture rồi chia

VíXanh thu tiền của khách trước, nền tảng chia sau. Hai bước đó là hai bút toán độc lập, mỗi bút toán tự cân:

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 290" role="img" aria-label="Hình 2. Dòng tiền chuyến TN-0001: bước 1 JE-000001 chuyển 25.500 từ customer:CU-001 sang psp:vixanh:clearing; bước 2 JE-000002 chia từ clearing 5.100 cho platform:revenue và 20.400 cho driver:DR-001."><defs><marker id="rust-double-entry-journal-entry-posting-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="55" width="220" height="90" rx="10" class="f-phys-soft s-phys"/><text x="36" y="83" class="t-b">customer:CU-001</text><text x="36" y="103" class="t-small">liability · ví khách</text><text x="36" y="123" class="t-small">số dư cuối: -25.500</text><rect x="330" y="55" width="240" height="90" rx="10" class="f-pk-soft s-pk"/><text x="346" y="83" class="t-b">psp:vixanh:clearing</text><text x="346" y="103" class="t-small">asset · tiền chờ về</text><text x="346" y="123" class="t-small">+25.500 rồi -25.500 = 0</text><rect x="650" y="10" width="210" height="80" rx="10" class="f-phys-soft s-phys"/><text x="666" y="36" class="t-b">platform:revenue</text><text x="666" y="56" class="t-small">revenue · phí 20%</text><text x="666" y="76" class="t-small">số dư cuối: +5.100</text><rect x="650" y="110" width="210" height="80" rx="10" class="f-phys-soft s-phys"/><text x="666" y="136" class="t-b">driver:DR-001</text><text x="666" y="156" class="t-small">liability · chờ trả tài xế</text><text x="666" y="176" class="t-small">số dư cuối: +20.400</text><path d="M240 100 L330 100" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-double-entry-journal-entry-posting-1-a)"/><text x="252" y="92" class="t-small">JE-000001</text><text x="268" y="118" class="t-small">25.500</text><path d="M570 85 L650 50" class="f-none s-pk" stroke-width="2" marker-end="url(#rust-double-entry-journal-entry-posting-1-a)"/><text x="572" y="62" class="t-small">5.100</text><path d="M570 115 L650 150" class="f-none s-pk" stroke-width="2" marker-end="url(#rust-double-entry-journal-entry-posting-1-a)"/><text x="572" y="148" class="t-small">20.400</text><rect x="20" y="205" width="410" height="70" rx="10" class="f-none s-line dash"/><text x="36" y="230" class="t-b">Bước 1 · JE-000001 · trip:TN-0001:capture</text><text x="36" y="252" class="t-small">customer -25.500 · psp:vixanh:clearing +25.500</text><rect x="450" y="205" width="410" height="70" rx="10" class="f-none s-line dash"/><text x="466" y="230" class="t-b">Bước 2 · JE-000002 · trip:TN-0001:split</text><text x="466" y="252" class="t-small">clearing -25.500 · revenue +5.100 · driver +20.400</text></svg>
</div>
<figcaption>Hình 2. Hai bút toán nối nhau qua <code>psp:vixanh:clearing</code>: sau bước 1 tiền nằm ở clearing, sau bước 2 clearing về 0.</figcaption>
</figure>

```rust
// src/main.rs
fn capture_entry() -> Result<JournalEntry, LedgerError> {
    JournalEntryBuilder::new("trip:TN-0001:capture")
        .posting(acct(CUSTOMER)?, Money::from_dong(-25_500))
        .posting(acct(CLEARING)?, Money::from_dong(25_500))
        .build()
}

fn split_entry() -> Result<JournalEntry, LedgerError> {
    JournalEntryBuilder::new("trip:TN-0001:split")
        .posting(acct(CLEARING)?, Money::from_dong(-25_500))
        .posting(acct(REVENUE)?, Money::from_dong(5_100))
        .posting(acct(DRIVER)?, Money::from_dong(20_400))
        .build()
}
```

Ghi hai bút toán vào `Ledger` rồi in dòng tiền vào trước, dòng tiền ra sau (số tiền không dấu):

```text
JE-000001  trip:TN-0001:capture
  Vào  psp:vixanh:clearing   25.500 VND
  Ra   customer:CU-001       25.500 VND
JE-000002  trip:TN-0001:split
  Vào  platform:revenue       5.100 VND
  Vào  driver:DR-001         20.400 VND
  Ra   psp:vixanh:clearing   25.500 VND
```

Mỗi bút toán có tổng vào bằng tổng ra (25.500 = 25.500; 5.100 + 20.400 = 25.500). Bước 1 có thể chạy xong mà bước 2 chưa chạy; lúc đó số dư clearing vẫn đúng là 25.500, và tổng toàn sổ vẫn bằng 0.

## Bút toán đảo: sửa sai mà không xoá gì

Sổ cái append-only: không `UPDATE`, không `DELETE`. Phát hiện `JE-000002` sai thì ghi thêm một bút toán **đảo** (reversal): cùng tài khoản, mọi số tiền đổi dấu, kèm mã bút toán gốc.

```rust
// src/entry.rs
pub fn reversal_of(
    original_id: EntryId,
    original: &JournalEntry,
) -> Result<JournalEntry, LedgerError> {
    let mut postings = Vec::with_capacity(original.postings.len());
    for p in &original.postings {
        let flipped = p.amount.checked_neg().ok_or(LedgerError::Overflow)?;
        postings.push(Posting::new(p.account.clone(), flipped));
    }
    let key = format!("{}:reversal", original.idempotency_key);
    JournalEntry::validated(&key, postings, Some(original_id))
}

// src/ledger.rs
pub fn reverse(&mut self, id: EntryId) -> Result<Posted<'_>, LedgerError> {
    let original = self.get(id).ok_or(LedgerError::UnknownEntry { id })?;
    let reversal = JournalEntry::reversal_of(id, original)?;
    self.post(reversal)
}
```

Có ba điểm cần nhìn. Một, bút toán đảo đi qua `validated` như mọi bút toán khác, và đảo dấu dùng `checked_neg` vì `i64::MIN` không có đối dương. Hai, key là key gốc cộng `:reversal`, nên gọi `reverse` hai lần với cùng mã chỉ ghi một bút toán (lần hai `replayed = true`). Ba, `get` trả `Option`, mã không có trong sổ thành `UnknownEntry` thay vì panic. Kết quả:

```text
JE-000003 replayed = false
JE-000003 replayed = true (đảo lần hai)
Số bút toán trong sổ: 3
JE-000003  trip:TN-0001:split:reversal  (đảo JE-000002)
  Vào  psp:vixanh:clearing   25.500 VND
  Ra   platform:revenue       5.100 VND
  Ra   driver:DR-001         20.400 VND
```

`JE-000002` vẫn nằm nguyên trong sổ. Người kiểm toán thấy cả bút toán sai, bút toán đảo và lý do chúng triệt tiêu nhau. Đảo xong, tài xế về 0 và clearing về 25.500 như sau bước 1.

## Vì sao số dư là projection, không phải một cột balance?

Số dư không phải dữ liệu gốc. Dữ liệu gốc là các posting; số dư là kết quả cộng chúng, gọi là **projection**:

```rust
// src/ledger.rs
pub fn balances(&self) -> Result<BTreeMap<AccountId, Money>, LedgerError> {
    let mut map: BTreeMap<AccountId, Money> = BTreeMap::new();
    for e in &self.entries {
        for p in e.postings() {
            let slot = map.entry(p.account.clone()).or_insert(Money::ZERO);
            *slot = slot.checked_add(p.amount).ok_or(LedgerError::Overflow)?;
        }
    }
    Ok(map)
}
```

Lưu thêm một cột `balance` mỗi khi ghi tưởng nhanh hơn, nhưng đưa vào ba rủi ro:

| Vấn đề | Cột `balance` lưu sẵn | Projection |
| --- | --- | --- |
| Race | Hai bên cùng đọc, cùng ghi đè: mất một lần cập nhật | Chỉ có thao tác thêm posting, không có bước đọc-rồi-ghi |
| Lệch | Hai nguồn sự thật; sai một chỗ không biết bên nào đúng | Một nguồn: posting |
| Audit | Chỉ thấy con số cuối, không biết nó từ đâu ra | Mỗi đồng truy về một posting và một bút toán |

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 280" role="img" aria-label="Hình 3. Hai lane so sánh. Lane trên, cột balance: A đọc 0, B đọc 0, A ghi 20.400, B ghi đè 20.400, mất một lần cập nhật. Lane dưới, projection: hai posting 20.400 cộng lại thành 40.800."><defs><marker id="rust-double-entry-journal-entry-posting-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><text x="20" y="28" class="t-b">Cột balance: đọc, tính, ghi đè</text><rect x="20" y="42" width="170" height="68" rx="10" class="f-phys-soft s-phys"/><text x="32" y="64" class="t-b">1 · A đọc</text><text x="32" y="84" class="t-small">thấy 0 VND</text><rect x="240" y="42" width="170" height="68" rx="10" class="f-phys-soft s-phys"/><text x="252" y="64" class="t-b">2 · B đọc</text><text x="252" y="84" class="t-small">thấy 0, A chưa ghi</text><rect x="460" y="42" width="170" height="68" rx="10" class="f-phys-soft s-phys"/><text x="472" y="64" class="t-b">3 · A ghi</text><text x="472" y="84" class="t-small">ô = 0 + 20.400</text><rect x="680" y="42" width="170" height="68" rx="10" class="f-warn-soft s-warn"/><text x="692" y="64" class="t-b">4 · B ghi đè</text><text x="692" y="84" class="t-small">ô = 0 + 20.400</text><text x="692" y="100" class="t-small">mất 20.400 của A</text><path d="M190 76 L240 76" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-double-entry-journal-entry-posting-2-a)"/><path d="M410 76 L460 76" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-double-entry-journal-entry-posting-2-a)"/><path d="M630 76 L680 76" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-double-entry-journal-entry-posting-2-a)"/><text x="20" y="170" class="t-b">Projection: cộng mọi posting (append-only)</text><rect x="20" y="186" width="170" height="60" rx="10" class="f-phys-soft s-phys"/><text x="32" y="210" class="t-b">posting 1</text><text x="32" y="230" class="t-small">DR-001 +20.400</text><rect x="240" y="186" width="170" height="60" rx="10" class="f-phys-soft s-phys"/><text x="252" y="210" class="t-b">posting 2</text><text x="252" y="230" class="t-small">DR-001 +20.400</text><rect x="460" y="186" width="230" height="60" rx="10" class="f-pk-soft s-pk"/><text x="472" y="210" class="t-b">Tổng: 40.800 VND</text><text x="472" y="230" class="t-small">luôn tính lại được từ posting</text><path d="M190 216 L240 216" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-double-entry-journal-entry-posting-2-a)"/><path d="M410 216 L460 216" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-double-entry-journal-entry-posting-2-a)"/></svg>
</div>
<figcaption>Hình 3. Cùng hai lần cộng 20.400: ô balance giữ 20.400 vì ghi đè xen kẽ, projection cho 40.800.</figcaption>
</figure>

Lab mô phỏng xen kẽ này một cách tất định, không cần luồng thật (`naive::lost_update`):

```text
Ô số dư sau hai lần cộng 20.400 xen kẽ: 20.400 VND
Projection từ hai posting:              40.800 VND
```

Bản thật xảy ra giữa hai request đồng thời, nên khó tái hiện và khó debug; mô phỏng chỉ cho thấy cơ chế. Production có thể giữ bảng số dư làm cache, miễn cập nhật cùng transaction với bút toán và luôn tính lại được từ posting.

## Chạy ví dụ và kiểm chứng

Từ gốc repository:

```bash
cargo run --manifest-path examples/toi-noi-rust/13-double-entry-journal/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/13-double-entry-journal/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/13-double-entry-journal/Cargo.toml
```

Lệnh thứ hai in `VERIFY OK` và thoát với mã 0; kiểm sai thì in `VERIFY FAIL: …` và thoát mã 1. Lệnh thứ ba chạy 31 unit test, 4 integration test và 3 doc test (trong đó hai doc test `compile_fail` chứng minh `Money(5)` và `JournalEntry { .. }` không biên dịch được từ ngoài crate); tất cả pass. Số dư sau hai bút toán in ra từ projection:

```text
  customer:CU-001     liability   -25.500 VND
  driver:DR-001       liability    20.400 VND
  platform:revenue    revenue       5.100 VND
  psp:vixanh:clearing asset             0 VND
Tổng toàn sổ: 0 VND
```

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Dấu của lab là chiều tiền, không phải nợ/có: báo cáo kế toán cần bút toán riêng, và chart of accounts thật có nhiều tài khoản hơn bốn cái này, do kế toán duyệt. Lab bỏ qua bước khách nạp ví nên <code>customer:CU-001</code> mang số âm. Sổ cái chỉ nằm trong bộ nhớ, <code>balances</code> quét toàn bộ bút toán, chưa có ngày giờ, đồng thời, phân quyền, và phí 5.100 viết cứng thay vì tính từ tỷ lệ. Đây là mô hình để học bất biến, không phải service chạy được trên production.</p>
</div>

**Bài tập:** sổ đang có `JE-000001` và `JE-000002`. Bạn phát hiện phí đúng là 5.000, tài xế nhận 20.500. Nên ghi những bút toán nào, và số dư `platform:revenue`, `driver:DR-001`, `psp:vixanh:clearing` sau đó là bao nhiêu? Vì sao không ghi lại bút toán chia mới với key `trip:TN-0001:split`?

<details><summary>Xem lời giải</summary><p>Hai bút toán. Trước hết <code>ledger.reverse(JE-000002)</code> tạo <code>JE-000003</code> (clearing +25.500, revenue -5.100, driver -20.400). Sau đó ghi <code>JE-000004</code> dưới key mới, ví dụ <code>trip:TN-0001:split-v2</code>: clearing -25.500, revenue +5.000, driver +20.500. Số dư: revenue 5.000, driver 20.500, clearing 0 (và customer vẫn -25.500, tổng toàn sổ 0). Key <code>trip:TN-0001:split</code> đã gắn với nội dung cũ, nên ghi nội dung khác dưới key đó trả <code>IdempotencyConflict</code>; sửa sai không được ghi đè mà phải bù bằng bút toán mới. Kịch bản này chạy trong test <code>correction_is_reversal_plus_new_entry_under_a_new_key</code> của lab.</p></details>

Bài trước: [bài 12](/articles/rust-newtype-value-object-money-account-id). Bài tiếp: [bài 14](/articles/rust-typestate-may-trang-thai-thanh-toan).

## Tài liệu gốc

- [The Rust Book: Defining and Instantiating Structs](https://doc.rust-lang.org/book/ch05-01-defining-structs.html).
- [The Rust Reference: Visibility and privacy](https://doc.rust-lang.org/reference/visibility-and-privacy.html).
- [The Rust Book: Method Syntax](https://doc.rust-lang.org/book/ch05-03-method-syntax.html).
- [std: `i64::checked_neg`](https://doc.rust-lang.org/std/primitive.i64.html#method.checked_neg).
- [std: `BTreeMap`](https://doc.rust-lang.org/std/collections/struct.BTreeMap.html).
- [Rust error code E0451](https://doc.rust-lang.org/error_codes/E0451.html).
