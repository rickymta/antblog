---
slug: rust-serde-json-but-toan-decimal-thoi-gian
title: "serde: serialize bút toán sang JSON, Decimal và thời gian"
summary: "Bài 18: serde và serde_json cho bút toán TN-0001: camelCase khớp service C#, Money số nguyên, tỷ lệ phí Decimal dạng chuỗi, jiff Timestamp UTC, schemaVersion, deny_unknown_fields và lỗi đọc JSON có dòng, cột."
category: rust-beginner
level: beginner
tags: [rust, toi-noi, ledger, serde, json, jiff]
language: vi
---

Sau khi ghi bút toán `JE-000001`, service sổ cái phát sự kiện `LedgerEntryPosted`, và các service C# đọc nó. Hai phía chỉ chia sẻ một văn bản JSON, không chia sẻ kiểu nào. Nếu Rust ghi `entry_id` còn C# chờ `entryId`, `System.Text.Json` không ném lỗi: nó để thuộc tính ở giá trị mặc định (đã thử trên .NET 10). Hợp đồng JSON phải khớp từng chữ, và phía đọc phải từ chối thứ sai hợp đồng thay vì đoán.

Đây là bài 18 của lộ trình **Rust cơ bản qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Lab xuất phát từ lab bài 10, giữ `Money`, `AccountId`, `Posting`, `JournalEntry`, và thêm bốn crate: `serde`, `serde_json`, `rust_decimal` ([bài 03](/articles/rust-kieu-so-tien-vnd-i64-decimal)), `jiff` ([bài 08](/articles/rust-string-str-ngay-gio-dinh-dang-tien)).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Một thân JSON có hai posting, <code>-100</code> và <code>90</code>, được đọc vào <code>JournalEntry</code>. Kết quả là một bút toán lệch 10 đồng, hay một lỗi?</p>
<details><summary>Xem đáp án</summary><p>Một lỗi. <code>Deserialize</code> của <code>JournalEntry</code> không dựng struct trực tiếp mà đi qua <code>JournalEntry::try_new</code>, nên bất biến «tổng bằng 0» vẫn đứng. Lab kiểm bằng test <code>reading_json_cannot_bypass_the_balance_invariant</code>.</p></details>
</div>

## `derive` và `rename_all`: JSON của bút toán trông thế nào?

`#[derive(Serialize, Deserialize)]` sinh code đọc và ghi lúc biên dịch, theo đúng tên và kiểu trường. Field Rust viết `snake_case`, JSON của ta dùng camelCase, và một dòng `rename_all` làm cầu nối. Đây là payload của sự kiện (`src/event.rs`, bỏ chú thích; các thuộc tính khác giải thích ở mục sau):

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EntryPosted {
    pub schema_version: SchemaV1,
    pub entry_id: String,
    pub posted_at: Timestamp,
    #[serde(with = "rust_decimal::serde::str")]
    pub fee_rate: Decimal,
    pub entry: JournalEntry,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels: Vec<String>,
}
```

`serde_json::to_string_pretty(&event)` ghi ra đúng file `tests/golden/ledger_entry_posted.v1.json`:

```json
{
  "type": "LedgerEntryPosted",
  "schemaVersion": 1,
  "entryId": "JE-000001",
  "postedAt": "2026-11-06T03:15:00Z",
  "feeRate": "0.20",
  "entry": {
    "idempotencyKey": "trip:TN-0001:capture",
    "postings": [
      {
        "account": "customer:CU-001",
        "amount": -25500
      },
      {
        "account": "platform:revenue",
        "amount": 5100
      },
      {
        "account": "driver:DR-001",
        "amount": 20400
      }
    ]
  }
}
```

`from_str` đi chiều ngược lại. Muốn đổi tên một trường riêng thì dùng `#[serde(rename = "…")]`. Tên biến thể `LedgerEntryPosted` giữ PascalCase vì đó là tên sự kiện, và `rename_all` trên struct không đụng tới nó.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Hình 1. Ba cột: struct Rust với tên snake_case, văn bản JSON camelCase ở giữa, kiểu C# PascalCase. serde và System.Text.Json cùng đổi tên về dạng camelCase trên dây, nên hai phía gặp nhau ở JSON."><defs><marker id="rust-serde-json-but-toan-decimal-thoi-gian-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="20" width="260" height="165" rx="10" class="f-phys-soft s-phys"/><text x="36" y="46" class="t-b">Rust · EntryPosted</text><text x="36" y="76" class="t-mono">entry_id: String</text><text x="36" y="102" class="t-mono">posted_at: Timestamp</text><text x="36" y="128" class="t-mono">fee_rate: Decimal</text><text x="36" y="154" class="t-mono">amount: Money</text><rect x="310" y="20" width="260" height="165" rx="10" class="f-pk-soft s-pk"/><text x="326" y="46" class="t-b">JSON trên dây</text><text x="326" y="76" class="t-mono">"entryId": "JE-000001"</text><text x="326" y="102" class="t-mono">"postedAt": "2026-11-06T03:15:00Z"</text><text x="326" y="128" class="t-mono">"feeRate": "0.20"</text><text x="326" y="154" class="t-mono">"amount": 25500</text><rect x="600" y="20" width="260" height="165" rx="10" class="f-phys-soft s-phys"/><text x="616" y="46" class="t-b">C# · System.Text.Json</text><text x="616" y="76" class="t-mono">string EntryId</text><text x="616" y="102" class="t-mono">DateTimeOffset PostedAt</text><text x="616" y="128" class="t-mono">decimal FeeRate</text><text x="616" y="154" class="t-mono">long Amount</text><path d="M282 102 L308 102" class="f-none s-phys" stroke-width="2" marker-start="url(#rust-serde-json-but-toan-decimal-thoi-gian-0-a)" marker-end="url(#rust-serde-json-but-toan-decimal-thoi-gian-0-a)"/><path d="M572 102 L598 102" class="f-none s-phys" stroke-width="2" marker-start="url(#rust-serde-json-but-toan-decimal-thoi-gian-0-a)" marker-end="url(#rust-serde-json-but-toan-decimal-thoi-gian-0-a)"/><rect x="20" y="200" width="840" height="38" rx="8" class="f-warn-soft s-warn"/><text x="36" y="224">Hai phía cùng đọc và ghi một văn bản. Lệch một chữ (entry_id thay vì entryId) là hợp đồng gãy.</text></svg>
</div>
<figcaption>Hình 1. Rust ghi <code>rename_all = "camelCase"</code>, C# dùng camelCase của ASP.NET Core. Điểm chung duy nhất là văn bản JSON.</figcaption>
</figure>

Hợp đồng nên ghi rõ camelCase thay vì trông vào mặc định của một phía: ở C#, `JsonSerializer` trơn giữ PascalCase (bảng cuối bài).

## Tiền là số nguyên, tỷ lệ phí là chuỗi: vì sao hai lựa chọn khác nhau?

`Money` mang `#[serde(transparent)]`: trên dây nó chỉ là số nguyên đồng, `25500` hay `-25500`. `serde_json` vốn ghi newtype như vậy; thuộc tính này biến điều đó thành hợp đồng viết rõ. Số thực không lọt vào: `25500.5` đọc vào `Money` là lỗi, không bị cắt phần lẻ.

Số nguyên đủ với C# (`long`), nhưng JavaScript đọc mọi số JSON thành `double`, chính xác tới 2^53: trên Node, `JSON.parse("9223372036854775807")` ra `9223372036854776000`. Với client JavaScript hoặc số tổng hợp lớn, dùng chuỗi: `#[serde(with = "ledger_lab::money::as_string")]` ra `{"amount":"25500"}`. Mỗi trường chỉ một dạng; `as_string` từ chối số trần, `Money` mặc định từ chối chuỗi.

Tỷ lệ phí là chuỗi `"0.20"`: nhiều bộ đọc JSON đưa số thực qua `double`, nơi `0.2` không biểu diễn đúng, còn chuỗi giữ nguyên từng chữ số. Các feature serde của `rust_decimal` 1.43:

| Feature | Tác dụng |
| --- | --- |
| `serde` | `Decimal` ghi ra chuỗi, đọc nhận cả chuỗi lẫn số |
| `serde-with-str` | thêm module `rust_decimal::serde::str` cho `#[serde(with = …)]`: ghi chuỗi, đọc chỉ nhận chuỗi |
| `serde-with-float` | thêm module `float`: ghi số thực qua `f64`, có thể mất chính xác |
| `serde-float` | đổi `impl Serialize` của `Decimal` trong cả bản build: ghi số thực |
| `serde-str` | đổi `impl Deserialize` của `Decimal` trong cả bản build: đọc chỉ nhận chuỗi |

Hai dòng cuối là bẫy: Cargo hợp nhất feature của một crate trong cả bản build, nên một crate khác bật `serde-float` là mọi `Decimal` không có `with` đổi dạng JSON mà ta không sửa dòng code nào. Lab chỉ bật `serde-with-str` và đặt `with = "rust_decimal::serde::str"` ngay trên trường; bật thêm `serde-float`, trường đó vẫn là chuỗi (đã thử). Đo được: `"feeRate": 0.2` (số) bị từ chối với `invalid type: floating point`; `"0.2"` đọc được và ghi lại đúng `"0.2"`, vì `Decimal` giữ scale. `0.20` và `0.2` so bằng nhau nhưng khác chữ, nên đừng so tỷ lệ bằng chuỗi. Phí tính bằng `Decimal`: `25.500 × 0.20` làm tròn về đồng ra `5.100 VND`.

## Thời gian, trường tuỳ chọn và trường lạ

Với feature `serde`, `jiff::Timestamp` ghi `2026-11-06T03:15:00Z`. Đọc `2026-11-06T10:15:00+07:00` thì chuẩn hoá về `…03:15:00Z`; chuỗi thiếu `Z` hoặc offset bị từ chối, như ở bài 08. Phía C#, `DateTimeOffset` offset 0 ghi `…+00:00` chứ không ghi `Z` (đã thử). Cả hai là ISO 8601 và đọc chéo được, nhưng chuỗi khác nhau: so thời điểm, không so chuỗi.

`Option` cùng `skip_serializing_if` cho trường có thể vắng: `memo: None` không xuất hiện trong JSON. `#[serde(default)]` cho phép trường thiếu khi đọc. Derive vốn coi `Option` thiếu là `None`, nhưng `Vec` thiếu là lỗi `missing field`, nên `labels` cần `default` để producer cũ vẫn đọc được. Lab in khoá ngoài cùng của hai sự kiện:

```text
không có memo/labels: ["entry", "entryId", "feeRate", "postedAt", "schemaVersion", "type"]
có memo/labels:       ["entry", "entryId", "feeRate", "labels", "memo", "postedAt", "schemaVersion", "type"]
```

Mặc định serde bỏ qua trường lạ. `#[serde(deny_unknown_fields)]` biến nó thành lỗi: `idempotency_key` gõ sai bị bắt ngay, kèm vị trí, thay vì bị bỏ qua rồi báo «thiếu key». Cái giá: bên đọc strict vỡ khi producer thêm một trường tương thích. Lab đặt strict để lỗi hiện ra; consumer sự kiện ngoài sổ cái thường nên bỏ nó và dựa vào `schemaVersion`.

## `schemaVersion` và `tag = "type"`: sự kiện nào, phiên bản nào?

`schemaVersion: 1` là một kiểu riêng, `SchemaV1`, chỉ chấp nhận số 1 (`src/event.rs`):

```rust
impl<'de> Deserialize<'de> for SchemaV1 {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<SchemaV1, D::Error> {
        match u32::deserialize(deserializer)? {
            1 => Ok(SchemaV1),
            other => Err(de::Error::custom(format!(
                "schemaVersion {other} không được hỗ trợ, cần 1"
            ))),
        }
    }
}
```

Ai gửi `schemaVersion: 2` bị từ chối ngay lúc đọc, thay vì bị hiểu nhầm theo luật của phiên bản 1. Khi cần v2, thêm kiểu mới và chọn kiểu theo số phiên bản đọc được; đừng đổi nghĩa trường của v1 tại chỗ.

Sự kiện có nhiều loại nên ta dùng `enum` với tag:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum LedgerEvent {
    LedgerEntryPosted(EntryPosted),
    LedgerEntryReversed(EntryReversed),
}
```

Tên biến thể thành giá trị của `"type"`, và trường này nằm cùng cấp với các trường khác. Đọc `"type": "LedgerEntryVoided"` cho lỗi `unknown variant` kèm danh sách biến thể hợp lệ. Phía C#, `[JsonPolymorphic(TypeDiscriminatorPropertyName = "type")]` cùng `[JsonDerivedType]` đọc được file golden của lab (thử tay trên .NET 10).

## Đọc JSON phải đi qua cửa kiểm: lỗi có vị trí ở đâu?

`derive(Deserialize)` thẳng trên struct có trường private vẫn dựng được struct (code sinh ra nằm cùng module), tức JSON đi vòng qua `try_new` và tạo được bút toán lệch. Lab chặn bằng `try_from`: đọc vào kiểu trung gian chỉ có hình dạng dữ liệu, rồi gọi `try_new` (`src/entry.rs`, rút gọn):

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "EntryWire")]
pub struct JournalEntry { /* trường private */ }

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct EntryWire {
    idempotency_key: String,
    postings: Vec<Posting>,
}

impl TryFrom<EntryWire> for JournalEntry {
    type Error = LedgerError;

    fn try_from(wire: EntryWire) -> Result<JournalEntry, LedgerError> {
        JournalEntry::try_new(&wire.idempotency_key, wire.postings)
    }
}
```

`AccountId` làm tương tự với `try_from = "String"`, nên mã rỗng bị từ chối lúc đọc. `serde_json::Error` cho biết `line()`, `column()` (từ 1; 0 là không có vị trí) và `classify()`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 280" role="img" aria-label="Hình 2. Bốn cửa kiểm khi đọc JSON vào JournalEntry: cú pháp, kiểu và tên trường, đủ trường, bất biến try_new. Ba cửa đầu báo lỗi có dòng và cột, cửa bất biến báo lỗi không có vị trí. Qua hết mới có JournalEntry hợp lệ."><defs><marker id="rust-serde-json-but-toan-decimal-thoi-gian-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="15" y="20" width="150" height="90" rx="10" class="f-phys-soft s-phys"/><text x="29" y="46" class="t-b">1 · Cú pháp</text><text x="29" y="70" class="t-small">JSON đúng ngữ pháp?</text><text x="29" y="88" class="t-small">(parser)</text><rect x="190" y="20" width="150" height="90" rx="10" class="f-phys-soft s-phys"/><text x="204" y="46" class="t-b">2 · Kiểu, tên</text><text x="204" y="70" class="t-small">amount là i64? có</text><text x="204" y="88" class="t-small">trường lạ không?</text><rect x="365" y="20" width="150" height="90" rx="10" class="f-phys-soft s-phys"/><text x="379" y="46" class="t-b">3 · Đủ trường</text><text x="379" y="70" class="t-small">có idempotencyKey,</text><text x="379" y="88" class="t-small">postings không?</text><rect x="540" y="20" width="150" height="90" rx="10" class="f-phys-soft s-phys"/><text x="554" y="46" class="t-b">4 · Bất biến</text><text x="554" y="70" class="t-small">try_new: tổng = 0,</text><text x="554" y="88" class="t-small">ít nhất 2 posting</text><rect x="715" y="20" width="150" height="90" rx="10" class="f-pk-soft s-pk"/><text x="729" y="46" class="t-b">5 · Giá trị</text><text x="729" y="70" class="t-small">JournalEntry hợp lệ</text><text x="729" y="88" class="t-small">mới được dùng</text><path d="M166 65 L188 65" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-serde-json-but-toan-decimal-thoi-gian-1-a)"/><path d="M341 65 L363 65" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-serde-json-but-toan-decimal-thoi-gian-1-a)"/><path d="M516 65 L538 65" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-serde-json-but-toan-decimal-thoi-gian-1-a)"/><path d="M691 65 L713 65" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-serde-json-but-toan-decimal-thoi-gian-1-a)"/><path d="M90 110 L90 150" class="f-none s-warn dash" stroke-width="2"/><path d="M265 110 L265 150" class="f-none s-warn dash" stroke-width="2"/><path d="M440 110 L440 150" class="f-none s-warn dash" stroke-width="2"/><path d="M615 110 L615 150" class="f-none s-warn dash" stroke-width="2"/><rect x="15" y="150" width="150" height="100" rx="10" class="f-warn-soft s-warn"/><text x="29" y="176" class="t-mono">(5, 5, Syntax)</text><text x="29" y="200" class="t-small">expected `,` or `]`</text><text x="29" y="220" class="t-small">có vị trí</text><rect x="190" y="150" width="150" height="100" rx="10" class="f-warn-soft s-warn"/><text x="204" y="176" class="t-mono">(4, 54, Data)</text><text x="204" y="200" class="t-small">invalid type</text><text x="204" y="220" class="t-small">có vị trí</text><rect x="365" y="150" width="150" height="100" rx="10" class="f-warn-soft s-warn"/><text x="379" y="176" class="t-mono">(7, 1, Data)</text><text x="379" y="200" class="t-small">missing field</text><text x="379" y="220" class="t-small">có vị trí</text><rect x="540" y="150" width="150" height="100" rx="10" class="f-warn-soft s-warn"/><text x="554" y="176" class="t-mono">(0, 0, Data)</text><text x="554" y="200" class="t-small">bút toán lệch,</text><text x="554" y="220" class="t-small">không có vị trí</text></svg>
</div>
<figcaption>Hình 2. Bốn cửa kiểm khi đọc <code>JournalEntry</code>. Ba cửa đầu do parser và visitor của serde phát hiện nên có dòng, cột; cửa 4 chạy sau khi đã đọc xong cả object nên không còn vị trí.</figcaption>
</figure>

Lab lấy thân bút toán sau (hằng `ENTRY_JSON` trong `src/main.rs`), sửa từng chỗ rồi in lỗi thật:

```json
{
  "idempotencyKey": "trip:TN-0001:capture",
  "postings": [
    { "account": "customer:CU-001", "amount": -25500 },
    { "account": "platform:revenue", "amount": 5100 },
    { "account": "driver:DR-001", "amount": 20400 }
  ]
}
```

| Sửa gì | `(line, column)` | `classify()` | Thông điệp |
| --- | --- | --- | --- |
| Bỏ dấu phẩy cuối dòng 4 | (5, 5) | `Syntax` | expected `,` or `]` |
| `"amount": "-25500"` | (4, 54) | `Data` | invalid type: string "-25500", expected i64 |
| Thêm `"fee": 1` vào posting | (5, 58) | `Data` | unknown field `fee`, expected `account` or `amount` |
| Xoá dòng `idempotencyKey` | (7, 1) | `Data` | missing field `idempotencyKey` |
| Đổi thành `idempotency_key` | (2, 19) | `Data` | unknown field `idempotency_key`, expected `idempotencyKey` or `postings` |
| Đổi `5100` thành `5000` | (0, 0) | `Data` | bút toán lệch, tổng bằng -100 VND |

Với lỗi kiểu, serde báo sau khi parser đọc xong token: cột 54 của dòng 4 là ký tự cuối của `"-25500"`. Với lỗi cú pháp, đó là chỗ parser nhận ra sai: dấu phẩy thiếu ở cuối dòng 4 được báo tại `{` đầu dòng 5. Cột đếm theo byte, nên chữ có dấu chiếm nhiều cột (đã thử: `ệ` tính 3). Hàng cuối là lỗi từ `try_new`, thuộc cả bút toán, nên không có dòng, cột.

Qua enum có `tag`, vị trí mất nhiều hơn: serde đệm cả object để tìm `"type"` rồi đọc biến thể từ bản đệm, nên lỗi kiểu, lỗi giá trị hay thiếu trường bên trong biến thể đều ra dòng 0. Ba sự kiện sai trong lab:

```text
schemaV 2    (dòng 0, cột 0, Data) schemaVersion 2 không được hỗ trợ, cần 1
type lạ      (dòng 2, cột 29, Data) unknown variant `LedgerEntryVoided`, expected `LedgerEntryPosted` or `LedgerEntryReversed` at line 2 column 29
amount chuỗi (dòng 0, cột 0, Data) invalid type: string "-25500", expected i64
```

Trong ba lỗi này chỉ `unknown variant` còn vị trí (lỗi cú pháp cũng vậy). Nên vị trí chỉ là phụ trợ: khi dòng và cột bằng 0, log kèm mã định danh message để tìm lại. Crate `serde_path_to_error` thêm đường dẫn trường (`postings[0].amount`) khi đọc thẳng struct; qua enum có `tag` nó cũng mất (đã thử).

## Với người từ C#: System.Text.Json ↔ serde

| System.Text.Json | serde |
| --- | --- |
| `JsonSerializerDefaults.Web` hoặc `JsonNamingPolicy.CamelCase`; `JsonSerializer` trơn giữ PascalCase | `#[serde(rename_all = "camelCase")]`, `rename` cho từng trường |
| Thuộc tính thiếu trong JSON: giá trị mặc định, im lặng (trừ `required`/`[JsonRequired]`) | Trường thiếu: lỗi `missing field`, trừ khi có `default` hoặc là `Option` |
| `JsonUnmappedMemberHandling.Disallow` | `#[serde(deny_unknown_fields)]` |
| `JsonConverter<T>` | `with = "…"`, `impl Serialize/Deserialize`, `try_from` |
| `JsonException.LineNumber`, `BytePositionInLine` (đếm từ 0), `Path` | `Error::line()`, `column()` (đếm từ 1); không có đường dẫn trường |
| Web defaults: đọc số từ chuỗi (`"0.20"` thành `decimal`), tên không phân biệt hoa thường | Chặt hơn: số từ chuỗi phải khai báo, `entryid` khác `entryId` |

Với trường thiếu, `System.Text.Json` mặc định khoan dung, serde mặc định chặt; với trường lạ, cả hai mặc định bỏ qua. Với tiền ta muốn chặt: phía C# bật `UnmappedMemberHandling` và đánh dấu `required` (thiếu thì ném `JsonException`, đã thử). Về tiền xem [record struct và `Money`](/articles/csharp-record-struct-money-quote) và [kiểu `decimal`](/articles/csharp-bien-kieu-du-lieu-decimal) của loạt C#.

## Chạy ví dụ và kiểm chứng

Từ gốc repository:

```bash
cargo run --manifest-path examples/toi-noi-rust/18-serde-json/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/18-serde-json/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/18-serde-json/Cargo.toml
```

Lệnh đầu in JSON ở mục đầu, rồi (bỏ các phần đã nêu):

```text
== Money ==
Money 25.500 đồng: 25500
dạng chuỗi:        {"amount":"25500"}
i64::MAX:          9223372036854775807
== Đọc lại (from_str) ==
round-trip bằng nhau: true
feeRate 0.20 nhân cước 25.500 = 5.100 VND
```

`-- --verify` kiểm JSON khớp file golden, round-trip, hai dạng tiền, từng cặp (dòng, cột) trong bảng và `schemaVersion: 2` bị từ chối; sai thì thoát mã 1. `cargo test` chạy 16 unit test (module từ lab 10), 15 test hợp đồng ở `tests/json_contract.rs` và 2 doc test. Test quan trọng nhất là `golden_round_trip_is_stable`: đọc file golden, ghi lại, ra đúng từng ký tự. Đưa cùng file cho test của service C# đọc là cách giữ hai phía không lệch nhau.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>JSON ở đây là chuỗi nhỏ trong bộ nhớ, đọc bằng <code>from_str</code>; chưa có đọc luồng, giới hạn kích thước payload, chữ ký hay mã hoá. Chỉ có <code>schemaVersion</code> 1, chưa có chiến lược nâng lên 2. Việc C# đọc được file golden là kết quả thử tay trên .NET 10; repo chưa có test C# đọc file này. Chưa có schema chung (OpenAPI, JSON Schema) và chưa có Kafka.</p>
</div>

## Bài tập

Producer cũ gửi payload không có `labels`; producer mới gửi `"labels": null`. Lab đọc trường hợp nào được, trường hợp nào lỗi? Sửa thế nào để đọc được cả hai, mà danh sách rỗng vẫn không được ghi ra?

<details><summary>Xem lời giải</summary><p>Thiếu <code>labels</code> đọc được nhờ <code>default</code>. <code>null</code> thì lỗi <code>invalid type: null, expected a sequence</code> (không có vị trí vì đi qua enum có tag): <code>default</code> chỉ lo trường vắng, không lo giá trị <code>null</code>. Thêm một hàm đọc:</p>
<pre><code class="language-rust">fn null_as_empty&lt;'de, D: Deserializer&lt;'de&gt;&gt;(d: D) -&gt; Result&lt;Vec&lt;String&gt;, D::Error&gt; {
    Ok(Option::&lt;Vec&lt;String&gt;&gt;::deserialize(d)?.unwrap_or_default())
}
// trên trường: #[serde(default, deserialize_with = "null_as_empty", skip_serializing_if = "Vec::is_empty")]</code></pre>
<p>Đã thử trong test nháp (hàm không có trong lab). Nới hợp đồng là quyết định về schema, nên ghi vào tài liệu hợp đồng chứ đừng chỉ sửa code.</p></details>

Bài trước: [bài 17](/articles/rust-thiserror-anyhow-xu-ly-loi-thuc-te). Bài tiếp: [bài 19](/articles/rust-proptest-property-test-bat-bien-so-cai).

## Tài liệu gốc

- [Serde: Attributes](https://serde.rs/attributes.html).
- [`serde_json::Error`](https://docs.rs/serde_json/1/serde_json/struct.Error.html).
- [`rust_decimal::serde`](https://docs.rs/rust_decimal/1/rust_decimal/serde/index.html).
- [jiff trên docs.rs](https://docs.rs/jiff/0.2/jiff/).
- [Tài liệu System.Text.Json](https://learn.microsoft.com/en-us/dotnet/standard/serialization/system-text-json/overview) của Microsoft.
- [Cargo: feature unification](https://doc.rust-lang.org/cargo/reference/features.html#feature-unification).
