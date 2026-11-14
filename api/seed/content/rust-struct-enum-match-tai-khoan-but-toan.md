---
slug: rust-struct-enum-match-tai-khoan-but-toan
title: "Struct, enum và pattern matching: mô hình tài khoản và bút toán"
summary: "Bài 05: mô hình tài khoản và bút toán bằng struct, enum có dữ liệu và match vét cạn; máy trạng thái thanh toán trả về Option, cùng if let, let else, Display, đối chiếu với class, record, enum của C#."
category: rust-beginner
level: beginner
tags: [rust, toi-noi, ledger, struct, enum, pattern-matching]
language: vi
---

Đây là bài 05 của lộ trình **Rust cơ bản qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Chuyến `TN-0001` đi 5 km, cước 25.500 VND: khách `CU-001` bị ghi nợ 25.500, tài xế `DR-001` được ghi có 20.400, nền tảng giữ 5.100. Trong code, số tiền của mỗi posting có dấu: âm là tiền ra khỏi tài khoản, dương là tiền vào, nên posting của khách là −25.500. Sổ cái cần một kiểu dữ liệu nói được điều đó, và cần từ chối ngay khi ai đó viết một bút toán không cân hoặc một lần chuyển trạng thái thanh toán không tồn tại.

Bài này dựng mô hình ấy bằng `struct`, `enum` và `match`. Bài 04 đã nói ai sở hữu giá trị; bài này nói giá trị có hình dạng gì. Lab chỉ dùng thư viện chuẩn, không có dependency.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p><code>PaymentState</code> có 6 biến thể. Bạn viết <code>match</code> chỉ với 5 nhánh và không có nhánh <code>_</code>. Chuyện gì xảy ra?</p>
<details><summary>Xem đáp án</summary><p>Chương trình không build được: rustc báo lỗi E0004, chỉ rõ biến thể nào chưa được xử lý. Không có chuyện chạy tới trạng thái bị bỏ sót rồi mới lỗi.</p></details>
</div>

## Một bút toán ba posting trông thế nào trong code?

Rust có ba dạng `struct`. Dạng có tên trường dùng cho dữ liệu nhiều thành phần như `Posting`. Dạng tuple struct bọc một giá trị để tạo kiểu riêng, gọi là newtype: `Money(i64)` khác `i64`, nên không ai cộng nhầm số dòng với số tiền. Dạng unit struct không có trường nào, chỉ dùng làm nơi gắn hành vi.

```rust
// src/main.rs (rút gọn: bỏ chú thích)
#[derive(Debug, Clone, Copy, PartialEq)]
struct Money(i64);

#[derive(Debug, Clone, PartialEq)]
struct AccountId(String);

struct Vixanh;

#[derive(Debug, Clone, PartialEq)]
struct Posting {
    account: String,
    amount: Money,
}

#[derive(Debug, Clone, PartialEq)]
struct JournalEntry {
    id: String,
    kind: EntryKind,
    postings: Vec<Posting>,
}
```

Dòng `#[derive(...)]` yêu cầu compiler sinh code mẫu. `Debug` cho phép in bằng `{:?}`; `Clone` cho phép `.clone()` tường minh; `PartialEq` cho phép so sánh `==` theo từng trường. Trong các struct trên, chỉ `Money` thêm `Copy`, vì kiểu này có 8 byte, sao chép rẻ và không sở hữu gì; `JournalEntry` chứa `String` và `Vec` nên không thể `Copy`, đúng với những gì bài 04 đã nói về move.

Hình 1 vẽ cả `JournalEntry` lẫn `AccountKind`, enum ở mục sau. Hộp trái là một giá trị `JournalEntry` chứa `Vec<Posting>`; hộp phải liệt kê bốn biến thể của loại tài khoản.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 1. JournalEntry là struct chứa id, kind và một Vec gồm ba Posting; AccountKind là enum có bốn biến thể, mỗi biến thể mang dữ liệu riêng."><rect x="20" y="20" width="520" height="262" rx="10" class="f-phys-soft s-phys"/><text x="36" y="46" class="t-b">struct JournalEntry</text><text x="36" y="70" class="t-mono">id: String = "JE-000001"</text><text x="36" y="92" class="t-mono">kind: EntryKind = Capture</text><text x="36" y="114" class="t-mono">postings: Vec&lt;Posting&gt;</text><rect x="36" y="126" width="488" height="42" rx="8" class="f-pk-soft s-pk"/><text x="48" y="152" class="t-mono">Posting { account: "customer:CU-001", amount: Money(-25500) }</text><rect x="36" y="174" width="488" height="42" rx="8" class="f-pk-soft s-pk"/><text x="48" y="200" class="t-mono">Posting { account: "driver:DR-001", amount: Money(20400) }</text><rect x="36" y="222" width="488" height="42" rx="8" class="f-pk-soft s-pk"/><text x="48" y="248" class="t-mono">Posting { account: "platform:revenue", amount: Money(5100) }</text><rect x="580" y="20" width="280" height="262" rx="10" class="f-warn-soft s-warn"/><text x="596" y="46" class="t-b">enum AccountKind</text><text x="596" y="76" class="t-mono">Customer(String)</text><text x="596" y="102" class="t-mono">Driver(String)</text><text x="596" y="128" class="t-mono">PlatformRevenue</text><text x="596" y="154" class="t-mono">PspClearing { provider }</text><text x="596" y="196" class="t-small">Mỗi giá trị là đúng một biến thể.</text><text x="596" y="216" class="t-small">Dữ liệu nằm trong biến thể mang nó,</text><text x="596" y="236" class="t-small">không có trường thừa cho các</text><text x="596" y="256" class="t-small">biến thể khác.</text></svg>
</div>
<figcaption>Hình 1. <b>struct</b> gom các trường cùng tồn tại; <b>enum</b> chọn một trong các biến thể, mỗi biến thể có dữ liệu riêng.</figcaption>
</figure>

## `&self`, `&mut self` và `self` khác nhau thế nào?

Method nằm trong khối `impl`. Tham số đầu tiên cho biết method dùng giá trị theo cách nào, và cách đó chính là ownership ở bài 04, được viết gọn lại. Hàm không có `self` là associated function, gọi bằng `JournalEntry::new(...)`, tương đương static factory của C#.

```rust
// src/main.rs (rút gọn: bỏ chú thích)
impl JournalEntry {
    fn new(id: &str, kind: EntryKind) -> JournalEntry {
        JournalEntry {
            id: id.to_string(),
            kind,
            postings: Vec::new(),
        }
    }

    fn add_posting(&mut self, account: &AccountKind, amount: Money) {
        self.postings.push(Posting {
            account: account.to_string(),
            amount,
        });
    }

    fn is_balanced(&self) -> bool {
        if self.postings.len() < 2 {
            return false;
        }
        let mut total: i64 = 0;
        for p in &self.postings {
            match total.checked_add(p.amount.0) {
                Some(next) => total = next,
                None => return false,
            }
        }
        total == 0
    }

    fn into_postings(self) -> Vec<Posting> {
        self.postings
    }
}
```

`&self` mượn chỉ đọc: `is_balanced` đọc được mọi posting mà không lấy đi quyền sở hữu. `&mut self` mượn có quyền sửa: `add_posting` đẩy thêm posting vào `Vec`. `self` lấy hẳn quyền sở hữu giá trị: sau `into_postings`, biến cũ không dùng được nữa. Thử dùng lại sẽ gặp đúng lỗi của bài 04 (thông điệp rút gọn):

```text
error[E0382]: borrow of moved value: `entry`
    |
429 |     let postings = entry.into_postings();
    |                          --------------- `entry` moved due to this method call
...
431 |     println!("{}", entry.id);
    |                    ^^^^^^^^ value borrowed here after move
note: `JournalEntry::into_postings` takes ownership of the receiver `self`, which moves `entry`
```

Đoạn gây lỗi chỉ còn là chú thích trong lab. Về `is_balanced`: bút toán cân khi có từ hai posting trở lên và tổng có dấu bằng 0. `checked_add` trả `Option<i64>`, nên tràn số cũng cho ra "không cân" thay vì quay vòng mà không báo gì.

## Enum của Rust khác enum của C# ở đâu?

`enum` của C# thực chất là một số nguyên có tên: nếu khai báo theo cùng thứ tự như enum Rust bên dưới, `PaymentState.Captured` mang giá trị `2`, và phép ép kiểu `(PaymentState)99` vẫn biên dịch được dù không có thành viên nào bằng 99. Muốn gắn dữ liệu cho từng trường hợp, C# thường dựng cây `abstract record` cùng các record con. Enum của Rust là kiểu tổng (sum type): mỗi biến thể có thể mang dữ liệu khác nhau, và không thể tạo một giá trị nằm ngoài danh sách.

```rust
// src/main.rs (rút gọn: bỏ chú thích)
enum AccountKind {
    Customer(String),
    Driver(String),
    PlatformRevenue,
    PspClearing { provider: String },
}

enum EntryKind {
    Capture,
    Refund,
    Reversal { of: String },
}

enum PaymentState {
    Initiated,
    Authorized,
    Captured,
    Failed,
    RefundPending,
    Refunded,
}
```

(Trong lab, mỗi enum đều có `#[derive(Debug, Clone, PartialEq)]`; `PaymentState` thêm `Copy` vì không mang dữ liệu.) `Customer(String)` là biến thể kiểu tuple, `PspClearing { provider }` có tên trường, `PlatformRevenue` không mang gì. `Reversal { of: String }` ghi bút toán nào bị đảo, đúng với nguyên tắc sổ cái chỉ thêm, không sửa.

Muốn in `customer:CU-001` thay vì `Customer("CU-001")`, ta cài đặt `Display`, trait dành cho `{}`:

```rust
impl fmt::Display for AccountKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AccountKind::Customer(id) => write!(f, "customer:{id}"),
            AccountKind::Driver(id) => write!(f, "driver:{id}"),
            AccountKind::PlatformRevenue => write!(f, "platform:revenue"),
            AccountKind::PspClearing { provider } => write!(f, "psp:{provider}:clearing"),
        }
    }
}
```

`Option<T>` cũng là một enum trong thư viện chuẩn, không phải cú pháp đặc biệt: `enum Option<T> { None, Some(T) }`. Vì vậy "không có giá trị" buộc phải được xử lý bằng `match` như mọi enum khác. Bài 07 sẽ đi sâu hơn, cùng với `Result`.

## `match` vét cạn: compiler không cho quên một trạng thái

`match` so một giá trị với các mẫu, theo thứ tự từ trên xuống, và **bắt buộc** phủ hết mọi khả năng. Hàm dưới cố ý không có nhánh `_`:

```rust
fn is_terminal(state: PaymentState) -> bool {
    match state {
        PaymentState::Initiated => false,
        PaymentState::Authorized => false,
        PaymentState::Captured => false,
        PaymentState::Failed => true,
        PaymentState::RefundPending => false,
        PaymentState::Refunded => true,
    }
}
```

Xoá nhánh `Refunded` và build lại, ta nhận được lỗi thật từ rustc 1.98 (rút gọn phần `help`):

```text
error[E0004]: non-exhaustive patterns: `PaymentState::Refunded` not covered
   --> src/main.rs:157:11
    |
157 |     match state {
    |           ^^^^^ pattern `PaymentState::Refunded` not covered
    |
note: `PaymentState` defined here
   --> src/main.rs:55:6
    |
 55 | enum PaymentState {
    |      ^^^^^^^^^^^^
...
 61 |     Refunded,
    |     -------- not covered
```

Hệ quả thực tế: ngày ai đó thêm trạng thái `Disputed` vào enum, mọi `match` không có `_` sẽ ngừng build cho tới khi được cập nhật. Với sổ cái, đó là danh sách việc cần xem lại mà compiler tự lập. Nhánh `_` (bắt mọi trường hợp còn lại) tiện nhưng làm tắt cơ chế này; chỉ dùng khi thật sự muốn gom hết phần còn lại vào một cách xử lý.

Ngoài phủ biến thể, mẫu còn hỗ trợ khoảng giá trị, guard và binding:

```rust
fn classify(amount: Money) -> String {
    match amount.0 {
        0 => "không đổi".to_string(),
        n if n < 0 => format!("ghi nợ {}", n.unsigned_abs()),
        small @ 1..=9_999 => format!("ghi có nhỏ {small}"),
        other => format!("ghi có {other}"),
    }
}
```

`n if n < 0` là guard: nhánh chỉ khớp khi điều kiện sau `if` đúng. `small @ 1..=9_999` vừa kiểm tra khoảng vừa giữ lại giá trị trong biến `small`. Một tên biến đứng riêng như `other` bắt phần còn lại; nếu không cần giá trị, viết `_`. Lưu ý compiler vét cạn cho cả số nguyên: nhánh cuối là nhánh giúp phủ hết mọi giá trị của `i64`.

## Máy trạng thái thanh toán: chuyển không hợp lệ là `None`

Trạng thái thanh toán đi theo các cạnh ở Hình 2: `INITIATED` sang `AUTHORIZED` rồi `CAPTURED`; `FAILED` đến được từ `INITIATED` hoặc `AUTHORIZED`; `CAPTURED` sang `REFUND_PENDING` rồi `REFUNDED`. Mọi cặp trạng thái và sự kiện khác đều bị từ chối.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 2. Máy trạng thái thanh toán: INITIATED sang AUTHORIZED sang CAPTURED; INITIATED và AUTHORIZED sang FAILED; CAPTURED sang REFUND_PENDING sang REFUNDED. FAILED và REFUNDED là trạng thái cuối. Hàm next trả Some cho cạnh hợp lệ và None cho mọi cặp còn lại."><defs><marker id="rust-struct-enum-match-tai-khoan-but-toan-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="40" width="150" height="70" rx="10" class="f-phys-soft s-phys"/><text x="34" y="70" class="t-b">INITIATED</text><text x="34" y="92" class="t-small">vừa tạo yêu cầu</text><rect x="230" y="40" width="150" height="70" rx="10" class="f-phys-soft s-phys"/><text x="244" y="70" class="t-b">AUTHORIZED</text><text x="244" y="92" class="t-small">đã giữ tiền</text><rect x="440" y="40" width="150" height="70" rx="10" class="f-phys-soft s-phys"/><text x="454" y="70" class="t-b">CAPTURED</text><text x="454" y="92" class="t-small">đã thu tiền</text><rect x="125" y="200" width="150" height="70" rx="10" class="f-warn-soft s-warn"/><text x="139" y="230" class="t-b">FAILED</text><text x="139" y="252" class="t-small">trạng thái cuối</text><rect x="440" y="200" width="150" height="70" rx="10" class="f-phys-soft s-phys"/><text x="454" y="230" class="t-b">REFUND_PENDING</text><text x="454" y="252" class="t-small">chờ VíXanh hoàn</text><rect x="700" y="200" width="150" height="70" rx="10" class="f-warn-soft s-warn"/><text x="714" y="230" class="t-b">REFUNDED</text><text x="714" y="252" class="t-small">trạng thái cuối</text><path d="M170 75 L228 75" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-struct-enum-match-tai-khoan-but-toan-1-a)"/><text x="200" y="66" text-anchor="middle" class="t-small">Authorize</text><path d="M380 75 L438 75" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-struct-enum-match-tai-khoan-but-toan-1-a)"/><text x="410" y="66" text-anchor="middle" class="t-small">Capture</text><path d="M515 110 L515 198" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-struct-enum-match-tai-khoan-but-toan-1-a)"/><text x="527" y="158" class="t-small">RequestRefund</text><path d="M590 235 L698 235" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-struct-enum-match-tai-khoan-but-toan-1-a)"/><text x="644" y="226" text-anchor="middle" class="t-small">CompleteRefund</text><path d="M95 110 L150 198" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-struct-enum-match-tai-khoan-but-toan-1-a)"/><text x="90" y="160" text-anchor="end" class="t-small">Fail</text><path d="M305 110 L250 198" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-struct-enum-match-tai-khoan-but-toan-1-a)"/><text x="292" y="160" class="t-small">Fail</text><rect x="640" y="20" width="220" height="110" rx="10" class="f-pk-soft s-pk"/><text x="656" y="46" class="t-mono">next(state, event)</text><text x="656" y="64" class="t-mono">-&gt; Option&lt;PaymentState&gt;</text><text x="656" y="92" class="t-small">Some(s): cạnh hợp lệ</text><text x="656" y="112" class="t-small">None: cặp còn lại (nhánh _)</text></svg>
</div>
<figcaption>Hình 2. Sáu trạng thái và các cạnh hợp lệ. Cạnh không vẽ, như <b>INITIATED + Capture</b> hay <b>REFUNDED + Authorize</b>, được <code>next</code> trả về <code>None</code>.</figcaption>
</figure>

Hàm `next` khớp trên một tuple `(state, event)`. Mỗi nhánh là một cạnh trong hình; mẫu `Initiated | Authorized` gộp hai trạng thái nguồn cho cùng sự kiện `Fail`:

```rust
fn next(state: PaymentState, event: PaymentEvent) -> Option<PaymentState> {
    use PaymentEvent::*;
    use PaymentState::*;
    match (state, event) {
        (Initiated, Authorize) => Some(Authorized),
        (Authorized, Capture) => Some(Captured),
        (Initiated | Authorized, Fail) => Some(Failed),
        (Captured, RequestRefund) => Some(RefundPending),
        (RefundPending, CompleteRefund) => Some(Refunded),
        _ => None,
    }
}
```

Ở đây nhánh `_` là chủ ý: số cặp (6 trạng thái × 5 sự kiện = 30) lớn, trong khi chỉ có 6 cặp hợp lệ, phần còn lại thống nhất là "không hợp lệ". Việc trả `Option` thay vì lặng lẽ giữ nguyên trạng thái buộc người gọi phải quyết định làm gì với lần chuyển trạng thái sai. Ở bài 07, `None` này sẽ được nâng thành `Err` có lý do.

## Khi nào dùng `if let` và `let else` thay cho `match`?

Khi chỉ có một biến thể cần xử lý, `match` đầy đủ dài dòng hơn mức cần thiết. `if let` khớp một mẫu và bỏ qua phần còn lại; `let else` lấy giá trị ra khỏi mẫu, còn nhánh `else` bắt buộc phải thoát (`return`, `continue`, `break`, `panic!`):

```rust
fn reversal_target(kind: &EntryKind) -> Option<&str> {
    if let EntryKind::Reversal { of } = kind {
        Some(of)
    } else {
        None
    }
}

fn describe_account(text: &str) -> String {
    let Some(kind) = parse_account(text) else {
        return format!("không nhận ra tài khoản '{text}'");
    };
    format!("tài khoản hợp lệ: {kind}")
}
```

Cái giá của `if let`: bạn không còn được kiểm tra vét cạn. Nếu mọi biến thể đều quan trọng, như `is_terminal` ở trên, hãy giữ `match`. `parse_account` trong lab dùng thêm toán tử `?` để trả `None` sớm; đó là nội dung bài 07, tạm hiểu là "thiếu thì thoát".

## Đối chiếu với class, record và enum của C#

| Nhu cầu | C# | Rust |
|---|---|---|
| Gom dữ liệu | `class` hoặc `record` ([bài record](/articles/csharp-record-struct-money-quote)) | `struct` có tên trường |
| So sánh theo giá trị | `record` tự sinh; `class` phải tự viết | `#[derive(PartialEq)]` |
| Hành vi và che dữ liệu | `class` với property ([bài class](/articles/csharp-class-property-encapsulation)) | `impl`; trường private theo module |
| Tập trạng thái cố định | `enum` là số nguyên, không mang dữ liệu | `enum` mỗi biến thể có dữ liệu riêng |
| Thiếu một trường hợp | `switch` expression thiếu trường hợp chỉ bị cảnh báo CS8509 (trừ khi bật warnings-as-errors) | `match` thiếu nhánh là lỗi E0004 |
| Không có giá trị | `null` hoặc nullable ([bài pattern](/articles/csharp-nullable-pattern-matching)) | `Option<T>` |

Pattern matching của C# mạnh và đọc tương tự, nhưng được áp dụng trên `enum` không mang dữ liệu hoặc cây kế thừa mở, nên compiler khó biết bạn đã phủ hết hay chưa. Enum đóng của Rust cho compiler đủ thông tin để buộc điều đó. Điều này không làm C# kém đi; điều đó chỉ khiến Rust phù hợp với một bài toán cụ thể: tập trạng thái tiền cố định, mỗi trạng thái cần được xử lý có chủ đích.

## Chạy ví dụ và kiểm chứng

Từ gốc repository:

```bash
cargo run --manifest-path examples/toi-noi-rust/05-struct-enum-match/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/05-struct-enum-match/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/05-struct-enum-match/Cargo.toml
```

Lệnh đầu in kết quả sau:

```text
== struct, tuple struct, unit struct ==
Money(25500) AccountId("customer:CU-001")
psp:vixanh:clearing
== enum có dữ liệu và Display ==
customer:CU-001
driver:DR-001
platform:revenue
psp:vixanh:clearing
== bút toán ==
customer:CU-001 -25500
driver:DR-001 +20400
platform:revenue +5100
JE-000001 balanced = true
thu tiền
JE-000002 đảo của Some("JE-000001"), balanced = true
đảo bút toán JE-000001
hoàn tiền
tài khoản hợp lệ: driver:DR-001
không nhận ra tài khoản 'DR-001'
== máy trạng thái thanh toán ==
Initiated --Authorize--> Authorized
Authorized --Capture--> Captured
Captured --Authorize--> None
== match: guard và @ ==
Money(0) -> không đổi
Money(-25500) -> ghi nợ 25500
Money(5100) -> ghi có nhỏ 5100
Money(20400) -> ghi có 20400
```

Lệnh `--verify` kiểm tra 23 điều kiện, gồm bút toán `JE-000001` cân (−25.500 + 20.400 + 5.100 = 0), bút toán lệch bị từ chối, bút toán đảo cân, các cạnh hợp lệ và không hợp lệ của máy trạng thái; kết thúc bằng `VERIFY OK` và mã thoát 0; sai một điều kiện thì in `VERIFY FAILED` và mã thoát 1. Ba test trong `cargo test` kiểm tra lại cùng các hành vi cốt lõi.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Đây là mô hình trong bộ nhớ để học cú pháp. Chưa có validation mã tài khoản, chưa có loại tiền, chưa kiểm tra số posting tối đa, chưa có <code>idempotency_key</code>, chưa lưu xuống PostgreSQL, và <code>next</code> chưa cho biết vì sao việc chuyển trạng thái bị từ chối. <code>Posting.account</code> là <code>String</code> nên vẫn nhận mã sai chính tả; bài sau sẽ thay bằng kiểu chặt hơn. Máy trạng thái thật còn phải xử lý sự kiện trùng và đến muộn từ cổng thanh toán.</p>
</div>

**Bài tập:** thêm trạng thái `Disputed` vào `PaymentState`, rồi xem `cargo build` báo gì. Sau đó cho `is_terminal` trả `false` với trạng thái mới và cho phép chuyển `Captured` sang `Disputed` bằng sự kiện `OpenDispute`. Hàm nào trong lab bắt buộc bạn sửa, hàm nào không?

<details><summary>Xem lời giải</summary><p>Ngay sau khi thêm biến thể, build lỗi E0004 ở <code>is_terminal</code>, hàm không có nhánh <code>_</code>; thêm <code>PaymentState::Disputed =&gt; false</code> là xong. <code>next</code> vẫn build vì có nhánh <code>_</code>, nhưng khi đó <code>Captured + OpenDispute</code> sẽ trả <code>None</code> cho tới khi bạn thêm biến thể <code>OpenDispute</code> vào <code>PaymentEvent</code> và nhánh <code>(Captured, OpenDispute) =&gt; Some(Disputed)</code>. Đây chính là điểm đánh đổi của nhánh <code>_</code>: tiện, nhưng compiler không nhắc bạn xem lại.</p></details>

Bài trước: [bài 04](/articles/rust-ownership-move-borrow-but-toan). Bài tiếp: [bài 06](/articles/rust-ham-dieu-kien-vong-lap-iterator-so-du).

## Tài liệu gốc

- [The Rust Book: Using Structs to Structure Related Data](https://doc.rust-lang.org/book/ch05-00-structs.html).
- [The Rust Book: Enums and Pattern Matching](https://doc.rust-lang.org/book/ch06-00-enums.html).
- [The Rust Book: Patterns and Matching](https://doc.rust-lang.org/book/ch19-00-patterns.html).
- [Rust Reference: Match expressions](https://doc.rust-lang.org/reference/expressions/match-expr.html).
- [std::option::Option](https://doc.rust-lang.org/std/option/enum.Option.html).
- [std::fmt::Display](https://doc.rust-lang.org/std/fmt/trait.Display.html).
