---
slug: rust-result-option-toan-tu-hoi-loi-la-gia-tri
title: "Result, Option và toán tử ?: lỗi là giá trị, không phải exception"
summary: "Bài 07: Rust không có exception. Dùng Result, Option và ? để bút toán lệch tổng trả Err(Unbalanced), ghi trùng idempotency key trả bút toán cũ, và phân biệt lỗi nghiệp vụ với bug (panic)."
category: rust-beginner
level: beginner
tags: [rust, toi-noi, ledger, error-handling, result]
language: vi
---

Đây là bài 07 của lộ trình **Rust cơ bản qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Mỗi bài có một crate lab độc lập trong `examples/toi-noi-rust/`; bài này chỉ dùng thư viện chuẩn.

Service Payment gửi sang sổ cái một bút toán có hai posting: khách -25.500 và tài xế +25.000. Lệch 500 đồng. Sổ cái không được ghi bút toán này, cũng không được sập vì nó; nó phải trả về một câu trả lời rõ ràng để bên gọi biết cần sửa gì. Bài này xem Rust diễn đạt câu trả lời đó thế nào khi ngôn ngữ không có `try/catch`.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Hàm có kiểu trả về <code>Result&lt;i64, String&gt;</code> dùng <code>?</code> trên một <code>Option</code> thì có biên dịch được không?</p>
<details><summary>Xem đáp án</summary><p>Không. <code>?</code> không tự đổi <code>Option</code> thành <code>Result</code>: rustc báo E0277 và gợi ý <code>.ok_or(...)?</code> để bạn chọn lỗi trả về. Mục về toán tử <code>?</code> giải thích vì sao.</p></details>
</div>

## Rust không có exception, vậy lỗi đi đâu?

Rust không có `throw` và `catch`. Một hàm có thể thất bại nói điều đó ngay trong chữ ký bằng kiểu `Result<T, E>`: hoặc `Ok(T)` mang kết quả, hoặc `Err(E)` mang lỗi. Tương tự, `Option<T>` là `Some(T)` hoặc `None`. Cả hai là **enum** thường, giống enum bài 05; không có cơ chế đặc biệt nào đứng sau. Muốn lấy giá trị ra, bạn phải xử lý cả hai nhánh, vì `match` trên enum phải vét cạn.

Rust vẫn có `panic!`, dành cho bug chứ không phải luồng điều khiển thông thường (mục «Khi nào Err, khi nào panic?»). Đối chiếu với loạt C#:

| C# | Rust |
| --- | --- |
| `throw` và `try/catch` ([bài debug exception](/articles/csharp-debug-exception-bao-gia)) | trả `Err(e)`, caller `match` hoặc dùng `?` |
| Exception không xuất hiện trong chữ ký | `Result<T, E>` nằm ngay trong chữ ký |
| `string?`, `null` ([bài nullable](/articles/csharp-nullable-pattern-matching)) | `Option<T>`, không có null |
| `??`, `?.` | `unwrap_or`, `map`, `?` trên `Option` |

Đọc chữ ký `fn try_new(...) -> Result<JournalEntry, LedgerError>` là biết hàm có thể từ chối, và từ chối theo những cách nào.

## Option cho «không có», Result cho «có nhưng thất bại»

Tra số dư một tài khoản không tồn tại là chuyện bình thường, nên hàm trả `Option`. Đừng dùng giá trị canh như `-1` hay chuỗi rỗng: `-1` là một số dư hợp lệ của ví khách, và compiler không nhắc bạn kiểm tra nó.

```rust
// src/main.rs (rút gọn từ impl Ledger)
fn balance(&self, account: &str) -> Option<Money> {
    self.balances.get(account).copied().map(Money)
}

fn require_balance(&self, account: &str) -> Result<Money, LedgerError> {
    self.balance(account)
        .ok_or_else(|| LedgerError::UnknownAccount(account.to_string()))
}
```

`ok_or_else` đổi `None` thành `Err` khi caller coi tài khoản thiếu là lỗi. Kết quả thật của lab:

```text
balance(driver:DR-001) = Some(Money(20400))
balance(driver:DR-999) = None
require_balance(driver:DR-999) = Err(UnknownAccount("driver:DR-999"))
unwrap_or(Money(0)) chỉ để hiển thị: Money(0)
```

`unwrap_or(Money(0))` an toàn khi chỉ để hiển thị. Đừng dùng nó để tính tiền: nó biến tài khoản không tồn tại thành số dư 0 và che mất lỗi nhập sai mã tài khoản.

## Lỗi nghiệp vụ của sổ cái là một enum

Lỗi có tên, có dữ liệu kèm theo, nên ta định nghĩa enum riêng. Mỗi biến thể là một tình huống mà caller có thể xử lý khác nhau:

```rust
// src/main.rs
#[derive(Debug, Clone, PartialEq, Eq)]
enum LedgerError {
    /// Tổng các posting khác 0; `diff` là tổng có dấu.
    Unbalanced {
        diff: i64,
    },
    /// Phép cộng i64 tràn.
    Overflow,
    UnknownAccount(String),
    DuplicateIdempotencyKey(String),
    /// Số tiền đọc từ chuỗi không hợp lệ (đến từ `ParseIntError` qua `From`).
    InvalidAmount(String),
}
```

Ngoài bốn biến thể nghiệp vụ, lab thêm `InvalidAmount` cho việc đọc số tiền từ chuỗi (mục `From` bên dưới). Để dùng với `Box<dyn Error>` và các thư viện, kiểu lỗi cần `Display` và `std::error::Error`. Bài này viết tay để thấy cơ chế; bài 17 sẽ dùng `thiserror` sinh các impl này.

```rust
// src/main.rs (rút gọn: bỏ ba nhánh UnknownAccount, DuplicateIdempotencyKey, InvalidAmount)
impl fmt::Display for LedgerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LedgerError::Unbalanced { diff } => write!(f, "bút toán lệch tổng: {diff} VND"),
            LedgerError::Overflow => write!(f, "tổng tiền tràn i64"),
            // ... ba biến thể còn lại viết tương tự
        }
    }
}

// Mọi method của Error đều có mặc định; đã có Debug + Display thì impl rỗng là đủ.
impl Error for LedgerError {}
```

Giờ đến cửa vào của sổ cái. `JournalEntry` chỉ tạo được qua `try_new`, nên mọi giá trị `JournalEntry` đang tồn tại đều đã cân tổng bằng 0:

```rust
// src/main.rs
impl JournalEntry {
    fn try_new(postings: Vec<Posting>) -> Result<JournalEntry, LedgerError> {
        let mut sum: i64 = 0;
        for p in &postings {
            // `?` ở đây: checked_add trả Option, ok_or đổi None thành Err(Overflow).
            sum = sum.checked_add(p.amount.0).ok_or(LedgerError::Overflow)?;
        }
        if sum != 0 {
            return Err(LedgerError::Unbalanced { diff: sum });
        }
        Ok(JournalEntry { postings })
    }
}
```

Dùng `checked_add` thay cho `+` là quyết định từ bài 03: phép `+` tràn sẽ panic ở bản debug và quay vòng ở bản release. Ở đây tràn là `Err(Overflow)`, ghi rõ trong chữ ký. Với bút toán lệch và bút toán tràn, lab in:

```text
== 2. Lệch tổng ==
Err: bút toán lệch tổng: -500 VND  (Debug: Unbalanced { diff: -500 })
== 3. Tràn số ==
Err: tổng tiền tràn i64  (Debug: Overflow)
```

## Toán tử ? làm gì và vì sao main phải trả Result?

Viết `match` cho mọi lời gọi sẽ rất dài. Toán tử `?` đặt sau một biểu thức `Result` làm hai việc: nếu `Ok(v)` thì lấy `v` và chạy tiếp; nếu `Err(e)` thì **return ngay** khỏi hàm hiện tại với `Err` đó (sau khi đổi kiểu lỗi qua `From`, xem mục sau). Hiểu gần đúng, `x?` là `match x { Ok(v) => v, Err(e) => return Err(From::from(e)) }`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 260" role="img" aria-label="Hình 1. Result là ngã ba Ok hoặc Err. Nhánh Ok tiếp tục chạy trong hàm; nhánh Err qua From rồi return sớm về caller."><defs><marker id="rust-result-option-toan-tu-hoi-loi-la-gia-tri-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="100" width="210" height="70" rx="10" class="f-phys-soft s-phys"/><text x="36" y="130" class="t-b">parse::&lt;i64&gt;()</text><text x="36" y="152" class="t-small">Result&lt;i64, ParseIntError&gt;</text><rect x="330" y="35" width="240" height="70" rx="10" class="f-pk-soft s-pk"/><text x="346" y="65" class="t-b">Ok(value)</text><text x="346" y="87" class="t-small">? lấy value, chạy tiếp dòng sau</text><rect x="330" y="175" width="240" height="70" rx="10" class="f-warn-soft s-warn"/><text x="346" y="205" class="t-b">Err(e)</text><text x="346" y="227" class="t-small">From::from(e) → LedgerError</text><rect x="650" y="35" width="210" height="70" rx="10" class="f-phys-soft s-phys"/><text x="666" y="65" class="t-b">Dòng tiếp theo</text><text x="666" y="87" class="t-small">hàm trả Ok(Money(value))</text><rect x="650" y="175" width="210" height="70" rx="10" class="f-phys-soft s-phys"/><text x="666" y="205" class="t-b">Caller của hàm</text><text x="666" y="227" class="t-small">nhận Err(LedgerError)</text><path d="M230 125 L330 70" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-result-option-toan-tu-hoi-loi-la-gia-tri-0-a)"/><path d="M230 150 L330 210" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-result-option-toan-tu-hoi-loi-la-gia-tri-0-a)"/><path d="M570 70 L650 70" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-result-option-toan-tu-hoi-loi-la-gia-tri-0-a)"/><path d="M570 210 L650 210" class="f-none s-warn dash" stroke-width="2" marker-end="url(#rust-result-option-toan-tu-hoi-loi-la-gia-tri-0-a)"/><text x="576" y="199" class="t-small">return sớm</text></svg>
</div>
<figcaption>Hình 1. Result là ngã ba Ok hoặc Err. Dấu <code>?</code> giữ nhánh Ok chạy tiếp và đưa nhánh Err về thẳng caller, sau khi đổi kiểu lỗi qua <code>From</code>.</figcaption>
</figure>

Vì `?` có thể return, hàm chứa nó phải có kiểu trả về nhận được lỗi đó. Thử dùng `?` trong `main` không có kiểu trả về:

```rust
// Lỗi E0277 nếu viết thế này (đoạn thử riêng, không nằm trong lab):
fn main() {
    let value: i64 = "25500".parse()?;
    println!("{value}");
}
```

```text
error[E0277]: the `?` operator can only be used in a function that returns `Result` or `Option` (or another type that implements `FromResidual`)
 --> src/main.rs:2:37
  |
1 | fn main() {
  | --------- this function should return `Result` or `Option` to accept `?`
2 |     let value: i64 = "25500".parse()?;
  |                                     ^ cannot use the `?` operator in a function that returns `()`
```

Cách sửa là cho `main` trả `Result<(), Box<dyn Error>>`. `Box<dyn Error>` nhận được mọi kiểu lỗi có `Error` impl (bao gồm `LedgerError`), nên `?` trong `main` dùng được với nhiều loại lỗi. Khi `main` trả `Err`, chương trình in `Error: ` kèm dạng Debug của lỗi rồi thoát với mã 1:

```rust
// src/main.rs (rút gọn nhánh --main-error)
fn main() -> Result<(), Box<dyn Error>> {
    match std::env::args().nth(1).as_deref() {
        Some("--main-error") => {
            parse_amount("hai muoi")?;
            println!("dòng này không bao giờ in");
            Ok(())
        }
        // ... các nhánh khác
        _ => demo(),
    }
}
```

```text
Error: InvalidAmount("invalid digit found in string")
```

`?` cũng dùng được trên `Option` trong hàm trả `Option`, nhưng không trộn hai loại: `?` trên `Option` trong hàm trả `Result` bị E0277 như câu hỏi đầu bài. Phải viết `.ok_or(...)?` để chọn lỗi.

## From: để ? đổi kiểu lỗi giúp bạn

`"25.500".parse::<i64>()` trả `Result<i64, ParseIntError>`, còn `parse_amount` của ta trả `Result<Money, LedgerError>`. Hai kiểu lỗi khác nhau. Nếu `LedgerError` không có `From<ParseIntError>`, rustc từ chối:

```text
error[E0277]: `?` couldn't convert the error to `LedgerError`
  --> src/main.rs:10:43
   |
 9 | fn parse_amount(text: &str) -> Result<Money, LedgerError> {
   |                                -------------------------- expected `LedgerError` because of this
10 |     let value = text.trim().parse::<i64>()?;
   |                             --------------^ the trait `From<ParseIntError>` is not implemented for `LedgerError`
```

Thêm impl `From` thì `?` tự gọi nó ở nhánh `Err`:

```rust
// src/main.rs
impl From<ParseIntError> for LedgerError {
    fn from(e: ParseIntError) -> Self {
        LedgerError::InvalidAmount(e.to_string())
    }
}

/// Đọc số tiền từ chuỗi. `?` tự đổi ParseIntError thành LedgerError nhờ `From`.
fn parse_amount(text: &str) -> Result<Money, LedgerError> {
    let value = text.trim().parse::<i64>()?;
    Ok(Money(value))
}
```

Cách viết thủ công cho cùng kết quả dùng `map_err`, tiện khi bạn không muốn (hoặc không thể) thêm `impl From`:

```rust
// src/main.rs
fn parse_amount_explicit(text: &str) -> Result<Money, LedgerError> {
    text.trim()
        .parse::<i64>()
        .map(Money)
        .map_err(|e| LedgerError::InvalidAmount(e.to_string()))
}
```

Để nối nhiều bước không cần `match`, dùng `and_then` (chạy bước tiếp nếu `Ok`) và `map` (đổi giá trị trong `Ok`). Ví dụ đọc một dòng `account=amount`:

```rust
// src/main.rs
fn parse_posting(line: &str) -> Result<Posting, LedgerError> {
    line.split_once('=')
        .ok_or_else(|| LedgerError::InvalidAmount(format!("thiếu dấu '=' trong {line:?}")))
        .and_then(|(account, text)| {
            parse_amount(text).map(|amount| Posting {
                account: account.trim().to_string(),
                amount,
            })
        })
}
```

```text
parse_amount("25500") = Ok(Money(25500))
parse_amount("25.500") = Err(InvalidAmount("invalid digit found in string"))
parse_posting("driver:DR-001=20400") = Ok(Posting { account: "driver:DR-001", amount: Money(20400) })
parse_posting("driver:DR-001") = Err(InvalidAmount("thiếu dấu '=' trong \"driver:DR-001\""))
```

Chuỗi «25.500» (có dấu chấm ngăn cách hàng nghìn) bị từ chối, đúng ý: phần lưu trữ chỉ nhận số nguyên đồng. Định dạng hiển thị có dấu chấm là việc của bài 08.

## Khi nào Err, khi nào panic?

Quy tắc dùng trong cả loạt: **`Err` cho tình huống nghiệp vụ dự kiến; `panic!` cho bug**, tức vi phạm một invariant mà code đúng không bao giờ tới được. Bút toán lệch tổng là chuyện sẽ xảy ra khi một service khác gửi sai, nên là `Err`. Một lời gọi `expect` thất bại trên giá trị mà chính code đảm bảo luôn có nghĩa là code sai, và không có cách xử lý nào hợp lý ngoài dừng lại để sửa.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 240" role="img" aria-label="Hình 2. Ranh giới giữa lỗi nghiệp vụ trả bằng Err và bug dừng bằng panic."><rect x="20" y="30" width="400" height="180" rx="10" class="f-pk-soft s-pk"/><text x="40" y="60" class="t-b">Err(LedgerError): tình huống dự kiến</text><text x="40" y="82" class="t-small">Caller quyết định: từ chối, báo lỗi, thử lại</text><text x="40" y="112" class="t-small">• bút toán lệch tổng, số tiền tràn i64</text><text x="40" y="136" class="t-small">• tài khoản không tồn tại</text><text x="40" y="160" class="t-small">• idempotency key bị ghi lần hai</text><text x="40" y="184" class="t-small">• chuỗi số tiền gõ sai</text><path d="M440 30 L440 210" class="f-none s-muted dash" stroke-width="2"/><rect x="460" y="30" width="400" height="180" rx="10" class="f-warn-soft s-warn"/><text x="480" y="60" class="t-b">panic!: bug của chính code</text><text x="480" y="82" class="t-small">Dừng thread, ghi log, sửa code</text><text x="480" y="112" class="t-small">• vi phạm invariant mà code không thể tới</text><text x="480" y="136" class="t-small">• expect trên giá trị code luôn đảm bảo có</text><text x="480" y="160" class="t-small">• chỉ số ngoài phạm vi do lỗi logic</text><text x="480" y="184" class="t-small">• tài khoản phải có từ lúc tạo sổ nhưng thiếu</text></svg>
</div>
<figcaption>Hình 2. Ranh giới của lỗi: dữ liệu và tình huống từ bên ngoài đi bằng <code>Err</code>; trạng thái mà code đã hứa không xảy ra mới được phép <code>panic!</code>.</figcaption>
</figure>

| Tình huống | Err hay panic | Lý do |
| --- | --- | --- |
| Bút toán -25.500 và +25.000 | `Err(Unbalanced { diff: -500 })` | Dữ liệu từ service khác; caller phải từ chối hoặc sửa |
| Cộng tiền vượt `i64::MAX` | `Err(Overflow)` | Sổ cái không được sai âm thầm; số liệu cực đoan có thể đến từ ngoài |
| Ghi lần hai cùng idempotency key | `Err(DuplicateIdempotencyKey)` | Retry là chuyện bình thường của hệ phân tán |
| Chuỗi số tiền «25.500» | `Err(InvalidAmount)` | Đầu vào của người dùng hoặc service ngoài |
| Key vừa báo trùng nhưng không có trong bảng `keys` | `panic!` qua `expect` | Chỉ xảy ra nếu code sai; không có cách xử lý hợp lý |
| Tài khoản phải có từ lúc khởi tạo sổ nhưng thiếu | `panic!` | Invariant của chính chương trình bị phá |

`unwrap()` và `expect("...")` lấy giá trị ra hoặc panic. Dùng chúng trong test, trong bản thử nhanh, hoặc khi bạn chứng minh được `None`/`Err` không thể xảy ra. Khi dùng cho trường hợp sau, viết lý do vào `expect` như trong lab; dòng đó sẽ là thông báo của lỗi:

```rust
// src/main.rs (rút gọn nhánh --panic-demo)
let ledger = Ledger::new(&["customer:CU-001"]);
let _ = ledger
    .balance("driver:DR-999")
    .expect("tài khoản phải được khai báo khi tạo sổ");
```

```text
thread 'main' (…) panicked at src/main.rs:489:18:
tài khoản phải được khai báo khi tạo sổ
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

Trong ngoặc là mã thread, đổi mỗi lần chạy nên bài lược đi; `489:18` là vị trí `expect` trong `src/main.rs` của lab. Chương trình thoát với mã 101. Đừng dùng `unwrap` trên dữ liệu bên ngoài (chuỗi từ API, dòng trong file): một đầu vào xấu sẽ làm sập cả service.

## Ghi lần hai cùng key: một lỗi mà caller xử lý được

`Ledger::post` trả `Err(DuplicateIdempotencyKey)` khi key `trip:TN-0001:capture` đã dùng. Hàm cũng tính số dư mới vào một bảng tạm trước, chỉ ghi vào sổ khi mọi posting hợp lệ; nên `?` đẩy lỗi `UnknownAccount` hay `Overflow` ra giữa chừng không để sổ nửa vời. `post_idempotent` bọc lại để caller nhận id của bút toán cũ:

```rust
// src/main.rs
fn post_idempotent(&mut self, key: &str, entry: JournalEntry) -> Result<EntryId, LedgerError> {
    match self.post(key, entry) {
        Err(LedgerError::DuplicateIdempotencyKey(k)) => {
            // Vừa được báo trùng thì key chắc chắn có trong bảng: nếu không, đó là bug.
            Ok(*self
                .keys
                .get(&k)
                .expect("key vừa báo trùng phải có trong bảng keys"))
        }
        other => other,
    }
}
```

Chỗ này cho thấy cả hai cách cùng tồn tại: `match` bắt đúng một biến thể `Err` mà ta xử lý được, còn `expect` giữ cho dòng vô lý (key vừa báo trùng lại không tồn tại) thành bug. Kết quả của lab:

```text
== 4. Idempotency ==
post lần 2: Err(DuplicateIdempotencyKey("trip:TN-0001:capture"))
post_idempotent lần 2: JE-000001 (entry cũ, sổ vẫn 1 bút toán)
```

Idempotency thật phải bền qua restart và đúng khi có hai request đồng thời; phần đó cần ràng buộc unique trong database, xem [bài transaction và idempotency của loạt .NET](/articles/dotnet10-transaction-concurrency-idempotency) và P03 của loạt Rust.

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
cargo run --manifest-path examples/toi-noi-rust/07-result-option/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/07-result-option/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/07-result-option/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/07-result-option/Cargo.toml -- --main-error
cargo run --manifest-path examples/toi-noi-rust/07-result-option/Cargo.toml -- --panic-demo
```

`--verify` assert các ca của bài và in:

```text
VERIFY OK (32 kiểm tra)
```

Các ca gồm `Unbalanced { diff: -500 }`, `Overflow` ở mức `try_new` lẫn số dư, `DuplicateIdempotencyKey` với số dư không đổi, `UnknownAccount` không để lại số dư sửa dở, `InvalidAmount` từ `From`. Kiểm tra sai thì `main` trả `Err`, in `Error: VERIFY FAIL: ...` và thoát mã 1. `cargo test` chạy lại chính các kiểm tra này. `--main-error` thoát mã 1 và `--panic-demo` thoát mã 101, cả hai là chủ ý.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Sổ cái ở đây nằm trong bộ nhớ, không có lưu trữ bền, không an toàn khi nhiều luồng cùng ghi, và key idempotency không có hạn dùng. <code>post_idempotent</code> trả bút toán cũ mà không so sánh nội dung: ở production, cùng key nhưng payload khác phải là một lỗi riêng. <code>try_new</code> chưa đòi tối thiểu hai posting (bài 13) và cộng tuần tự nên có thể báo <code>Overflow</code> sớm dù tổng cuối bằng 0. Trường của <code>JournalEntry</code> chưa được đóng gói (bài 10), và lỗi chưa mang <code>source()</code> hay ngữ cảnh (bài 17).</p></div>

**Bài tập:** thêm biến thể `LedgerError::EmptyEntry` cho bút toán không có posting nào hoặc chỉ có một posting. Bạn phải sửa những chỗ nào, và compiler giúp gì?

<details><summary>Xem lời giải</summary>
<p>Thêm biến thể vào enum, rồi chạy <code>cargo build</code>: rustc báo E0004 «non-exhaustive patterns: <code>&amp;LedgerError::EmptyEntry</code> not covered» tại <code>match self</code> trong <code>impl Display</code>, buộc bạn thêm nhánh, ví dụ <code>LedgerError::EmptyEntry =&gt; write!(f, "bút toán phải có ít nhất 2 posting")</code>. Cuối cùng thêm vào đầu <code>try_new</code>: <code>if postings.len() &lt; 2 { return Err(LedgerError::EmptyEntry); }</code>. Khi thử trên bản sao của lab, 32 kiểm tra cũ vẫn đạt vì mọi ca cũ đều có từ hai posting trở lên. Đó là lợi ích của enum vét cạn: thêm một loại lỗi thì compiler chỉ ra mọi nơi cần xử lý.</p>
</details>

Bài trước: [bài 06](/articles/rust-ham-dieu-kien-vong-lap-iterator-so-du). Bài tiếp: [bài 08](/articles/rust-string-str-ngay-gio-dinh-dang-tien).

## Tài liệu gốc

- [The Rust Book: Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html).
- [The Rust Book: Propagating Errors with ?](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html#a-shortcut-for-propagating-errors-the--operator).
- [std::result::Result](https://doc.rust-lang.org/std/result/enum.Result.html).
- [std::option::Option](https://doc.rust-lang.org/std/option/enum.Option.html).
- [std::error::Error](https://doc.rust-lang.org/std/error/trait.Error.html).
- [The Reference: Operator expressions, mục Question mark](https://doc.rust-lang.org/reference/expressions/operator-expr.html#r-expr.try).
