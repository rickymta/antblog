---
slug: rust-ownership-move-borrow-but-toan
title: "Ownership, move và borrow: mỗi bút toán chỉ có một chủ"
summary: "Bài 04: ba quy tắc ownership, move, Copy và Clone, &T và &mut T qua hàm post(&mut Ledger, JournalEntry); đọc lỗi E0382, E0502, E0499 thật và hiểu vì sao bút toán đã vào sổ không bị sửa qua tham chiếu cũ."
category: rust-beginner
level: beginner
tags: [rust, toi-noi, ledger, ownership, borrow-checker]
language: vi
---

Đây là bài 04 của lộ trình **Rust cơ bản qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Sau khi chuyến `TN-0001` hoàn tất, service sổ cái nhận bút toán `JE-000001` (VíXanh thu 25.500 VND của khách) và ghi vào sổ. Câu hỏi của bài: sau khi ghi, ai còn được giữ và sửa bút toán đó?

Ở [bài 03](/articles/rust-kieu-so-tien-vnd-i64-decimal) ta đã chốt lưu tiền bằng `i64` đơn vị đồng. Hôm nay ta bọc nó trong newtype `Money(i64)`, dùng trong một `Ledger` nhỏ và học cơ chế khiến Rust khác C#: **ownership**. Ví dụ chỉ dùng thư viện chuẩn.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Sau <code>let b = a;</code>, biến <code>a</code> còn dùng được không?</p>
<details><summary>Xem đáp án</summary><p>Tùy kiểu. Với <code>i64</code> thì còn, vì giá trị được sao chép. Với <code>String</code> thì không: quyền sở hữu đã chuyển sang <code>b</code> và compiler báo lỗi nếu bạn dùng lại <code>a</code>.</p></details>
</div>

## Ba quy tắc ownership là gì?

Rust quản lý bộ nhớ không bằng garbage collector mà bằng ba quy tắc được compiler kiểm tra lúc build:

1. Mỗi giá trị có một **chủ** (owner): một biến, một field hoặc một phần tử của collection.
2. Tại một thời điểm chỉ có một chủ. Gán hoặc truyền đi thường **chuyển** quyền sở hữu (move).
3. Khi chủ ra khỏi scope, giá trị bị **drop**: bộ nhớ được giải phóng ngay tại đó.

Lab có kiểu `Tracked` cài `Drop` để nhìn thấy thời điểm drop:

```rust
// src/main.rs (rút gọn)
let _outer = Tracked(String::from("outer"));
{
    let _inner = Tracked(String::from("inner"));
    println!("  trong scope con");
}
println!("  đã ra scope con");
```

Kết quả: `trong scope con`, `drop inner`, `đã ra scope con`, rồi `drop outer` khi hàm kết thúc. Trong cùng một scope, biến khai báo sau bị drop trước: chế độ `--verify` tạo hai biến `verify-x` rồi `verify-y` trong một scope, in `drop verify-y` trước `drop verify-x` và assert cả hai đã được drop.

## Vì sao `i64` gán xong còn dùng, còn `String` thì không?

`i64` nằm trọn trên **stack**: 8 byte, sao chép rất rẻ. `String` gồm ba từ trên stack (con trỏ, độ dài, sức chứa) trỏ tới vùng **heap** chứa các byte chữ. Nếu gán `String` mà để cả hai biến cùng giữ con trỏ, cả hai sẽ cố giải phóng một vùng nhớ. Rust tránh điều đó bằng cách **move**: biến mới nhận quyền sở hữu, biến cũ không còn hợp lệ.

```rust
// src/main.rs (rút gọn)
let a: i64 = 25_500;
let b = a; // i64 là Copy: a vẫn dùng được
let id = String::from("JE-000001");
let moved = id; // String không Copy: quyền sở hữu chuyển sang `moved`
// Lỗi E0382 nếu bỏ comment: borrow of moved value: `id`
// println!("{id}");
```

Bỏ comment dòng cuối, `rustc` 1.98 báo (rút gọn):

```text
error[E0382]: borrow of moved value: `id`
   --> src/main.rs:131:16
    |
128 |     let id = String::from("JE-000001");
    |         -- move occurs because `id` has type `String`, which does not implement the `Copy` trait
129 |     let moved = id;
    |                 -- value moved here
131 |     println!("{id}");
    |                ^^ value borrowed here after move
```

Thông điệp có đủ ba mốc: biến nào, bị move ở đâu, dùng lại ở đâu.

## Truyền bút toán vào hàm thì chuyện gì xảy ra?

Truyền một giá trị không `Copy` vào hàm cũng là move. Ta dùng điều này có chủ đích: bút toán đã `post` là của sổ cái, không còn của caller.

```rust
// src/main.rs
fn post(ledger: &mut Ledger, entry: JournalEntry) {
    ledger.entries.push(entry);
}

fn balance(ledger: &Ledger, account: &str) -> i64 {
    let mut total: i64 = 0;
    for entry in &ledger.entries {
        for posting in &entry.postings {
            if posting.account == account {
                total = total.checked_add(posting.amount.0).expect("tràn i64");
            }
        }
    }
    total
}
```

`post` nhận **ownership** của `entry` và mượn `ledger` để ghi. `balance` chỉ **mượn** sổ cái và tên tài khoản, không lấy gì làm của mình. Chữ ký hàm vì thế đã nói trước điều hàm được phép làm.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Hình 1. Biến entry ở main chuyển quyền sở hữu vào tham số của post, rồi vào Ledger.entries. Biến entry cũ ở caller không còn hợp lệ."><defs><marker id="rust-ownership-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker><marker id="rust-ownership-0-b" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs><rect x="20" y="30" width="230" height="80" rx="10" class="f-phys-soft s-phys"/><text x="36" y="60" class="t-b">main: entry</text><text x="36" y="85" class="t-small">sở hữu JE-000001</text><rect x="330" y="30" width="230" height="80" rx="10" class="f-phys-soft s-phys"/><text x="346" y="60" class="t-b">post(ledger, entry)</text><text x="346" y="85" class="t-small">tham số entry giữ bút toán</text><rect x="640" y="30" width="220" height="80" rx="10" class="f-pk-soft s-pk"/><text x="656" y="60" class="t-b">Ledger.entries</text><text x="656" y="85" class="t-small">chủ mới của JE-000001</text><path d="M250 70 L330 70" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-ownership-0-a)"/><text x="276" y="60" class="t-small">move</text><path d="M560 70 L640 70" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-ownership-0-a)"/><text x="588" y="60" class="t-small">push</text><path d="M135 110 L135 160" class="f-none s-warn dash" stroke-width="2" marker-end="url(#rust-ownership-0-b)"/><text x="148" y="142" class="t-small">sau move</text><rect x="20" y="160" width="230" height="70" rx="10" class="f-warn-soft s-warn dash"/><text x="36" y="190" class="t-b">entry ở caller</text><text x="36" y="212" class="t-small">không còn hợp lệ: E0382</text><text x="300" y="200" class="t-small">Compiler chặn lúc build, không đợi tới lúc chạy.</text></svg>
</div>
<figcaption>Hình 1. Move: quyền sở hữu đi theo mũi tên, biến cũ ở caller không còn dùng được.</figcaption>
</figure>

Trong lab, dòng `println!("{}", entry.id)` sau `post` được giữ ở dạng chú thích. Bỏ comment, rustc báo E0382 «borrow of moved value: `entry`» và chỉ ra tham số `entry` của `post` đang lấy ownership.

## Copy hay Clone: sao chép ngầm và sao chép tường minh khác gì?

`Copy` là sao chép **ngầm** bit-by-bit: chỉ dành cho kiểu mà chép bit là đủ, không sở hữu vùng heap nào. `Clone` là sao chép **tường minh** bằng `.clone()`, có thể tốn bộ nhớ vì phải nhân đôi cả vùng heap.

```rust
// src/main.rs (rút gọn)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Money(i64);

let fare = Money(25_500);
let same = fare; // Money derive Copy: fare vẫn dùng được
let copy = moved.clone(); // clone tường minh: sao chép cả vùng heap
```

Một kiểu chỉ được `Copy` khi mọi field đều `Copy` và kiểu đó không cài `Drop`. `Money(i64)` đủ điều kiện, nên truyền tiền đi lại không để lại "xác" biến đã move. `Posting` chứa `String` nên không `Copy`; `Vec` thì chỉ `Clone`.

Ta cố ý **không** derive `Clone` cho `JournalEntry`. rustc gợi ý clone khi gặp E0382, nhưng một bút toán nhân bản sẽ có hai bản cùng `id` và cùng `idempotency_key`. Muốn sao chép thì phải viết ra, và người review sẽ thấy.

## `&T` và `&mut T`: ai được đọc, ai được ghi?

Thay vì chuyển chủ, ta có thể **mượn** (borrow) bằng tham chiếu. Quy tắc: tại một thời điểm, hoặc có **nhiều** `&T` (chỉ đọc), hoặc có **đúng một** `&mut T` (đọc và ghi), không bao giờ cả hai. Tham chiếu sống đến lần dùng cuối cùng của nó, không đến hết scope.

```rust
// src/main.rs (rút gọn)
let r1 = &ledger;
let r2 = &ledger; // nhiều &T cùng lúc: hợp lệ
println!(
    "hai người đọc thấy {} và {}",
    balance(r1, "psp:vixanh:clearing"),
    balance(r2, "psp:vixanh:clearing")
);
// r1, r2 không còn được dùng, nên &mut bên dưới hợp lệ (NLL).
post(&mut ledger, split_entry()); // một người ghi duy nhất
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 290" role="img" aria-label="Hình 2. Bên trái ba tham chiếu &amp;T cùng đọc một Ledger. Bên phải một tham chiếu &amp;mut T duy nhất được ghi; tham chiếu &amp;mut thứ hai bị chặn bởi E0499."><defs><marker id="rust-ownership-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker><marker id="rust-ownership-1-b" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs><text x="20" y="30" class="t-b">&amp;T: nhiều người đọc cùng lúc</text><rect x="20" y="50" width="140" height="44" rx="8" class="f-phys-soft s-phys"/><text x="34" y="77" class="t-b">r1 = &amp;ledger</text><rect x="20" y="110" width="140" height="44" rx="8" class="f-phys-soft s-phys"/><text x="34" y="137" class="t-b">r2 = &amp;ledger</text><rect x="20" y="170" width="140" height="44" rx="8" class="f-phys-soft s-phys"/><text x="34" y="197" class="t-b">balance(..)</text><rect x="250" y="90" width="150" height="120" rx="10" class="f-pk-soft s-pk"/><text x="266" y="145" class="t-b">Ledger</text><text x="266" y="168" class="t-small">chủ sở hữu</text><path d="M160 72 L250 120" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-ownership-1-a)"/><path d="M160 132 L250 150" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-ownership-1-a)"/><path d="M160 192 L250 180" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-ownership-1-a)"/><path d="M440 20 L440 270" class="f-none s-line dash"/><text x="470" y="30" class="t-b">&amp;mut T: một người ghi, không ai khác</text><rect x="470" y="105" width="180" height="90" rx="8" class="f-warn-soft s-warn"/><text x="486" y="145" class="t-b">w = &amp;mut ledger</text><text x="486" y="168" class="t-small">post(w, entry)</text><rect x="710" y="90" width="150" height="120" rx="10" class="f-pk-soft s-pk"/><text x="726" y="145" class="t-b">Ledger</text><text x="726" y="168" class="t-small">chủ sở hữu</text><path d="M650 150 L710 150" class="f-none s-warn" stroke-width="2" marker-end="url(#rust-ownership-1-b)"/><rect x="470" y="225" width="180" height="44" rx="8" class="f-warn-soft s-warn dash"/><text x="484" y="252" class="t-small">w2: bị chặn (E0499)</text><path d="M650 247 L710 205" class="f-none s-warn dash" stroke-width="2"/></svg>
</div>
<figcaption>Hình 2. Nhiều mũi tên đọc cùng tồn tại được; chỉ một mũi tên ghi, và khi nó tồn tại thì không có mũi tên nào khác tới cùng Ledger.</figcaption>
</figure>

Hai lỗi kinh điển, đều được tạo thật trong lab rồi để lại dạng chú thích. Giữ một tham chiếu đọc rồi `post`:

```rust
// src/main.rs, demo_borrow (đang là chú thích)
let last = last_entry(&ledger);
post(&mut ledger, split_entry());
println!("{:?}", last);
```

```text
error[E0502]: cannot borrow `ledger` as mutable because it is also borrowed as immutable
   --> src/main.rs:192:10
    |
191 |     let last = last_entry(&ledger);
    |                           ------- immutable borrow occurs here
192 |     post(&mut ledger, split_entry());
    |          ^^^^^^^^^^^ mutable borrow occurs here
193 |     println!("{:?}", last);
    |                      ---- immutable borrow later used here
```

Và hai người ghi cùng lúc:

```text
error[E0499]: cannot borrow `ledger` as mutable more than once at a time
   --> src/main.rs:197:14
    |
196 |     let w1 = &mut ledger;
    |              ----------- first mutable borrow occurs here
197 |     let w2 = &mut ledger;
    |              ^^^^^^^^^^^ second mutable borrow occurs here
198 |     post(w1, split_entry());
    |          -- first borrow later used here
```

## Vì sao bút toán đã ghi không thể bị sửa qua một tham chiếu cũ?

Quy tắc thật là: **không có alias khi có mutation**. Nếu một người đang đọc `last` (trỏ vào phần tử trong `Vec<JournalEntry>`) trong khi `post` gọi `push`, `Vec` có thể phải cấp phát lại vùng nhớ lớn hơn, và `last` sẽ trỏ vào bộ nhớ đã giải phóng. Rust loại bỏ tình huống đó từ gốc: không cho cả hai cùng tồn tại.

Với sổ cái, hệ quả hữu ích hơn. Bút toán vào sổ thuộc về `Ledger`. Bên ngoài chỉ có thể nhận `&JournalEntry` để đọc, và những tham chiếu đó phải hết hạn trước khi ai đó giữ `&mut Ledger`. Không tồn tại "tham chiếu cũ" nào sửa được bút toán sau lưng sổ cái. Lưu ý giới hạn: borrow checker không tự biến bút toán thành bất biến. Trong lab, mọi thứ nằm chung một module, nên ai giữ `&mut Ledger` vẫn gán thẳng được vào `ledger.entries`. Tính append-only đến từ API: chỉ lộ `&JournalEntry` và không có hàm sửa. Bài 10 sẽ để field private trong module riêng để compiler giữ luôn phần đó.

Đối chiếu với C# ([bài về record](/articles/csharp-record-struct-money-quote)): biến kiểu reference luôn có thể alias cùng một object; `record` với property `init` chỉ chặn gán lại property, tức bất biến **nông**, nên `List` bên trong record vẫn bị sửa qua mọi alias. Sửa `List<T>` trong lúc `foreach` thì ném `InvalidOperationException` **lúc chạy**. Rust đưa cả hai thành lỗi **lúc build**, đổi lại bạn phải nghĩ về chủ và người mượn từ đầu.

Một lỗi cùng họ là tham chiếu treo (dangling). Lab giữ ví dụ ngắn ở dạng chú thích, lifetime đầy đủ để dành cho bài 09:

```rust
// Lỗi E0106 nếu bỏ comment: missing lifetime specifier
// fn dangle() -> &String {
//     let s = String::from("JE-000001");
//     &s
// }
```

`s` bị drop khi hàm kết thúc, nên tham chiếu tới nó sẽ treo. rustc từ chối và gợi ý trả `String` sở hữu.

## Đọc thông điệp rustc: gặp lỗi nào, sửa theo hướng nào?

| Thông điệp rustc | Nghĩa | Sửa |
| --- | --- | --- |
| E0382 `borrow of moved value: entry` | Giá trị đã chuyển chủ (move); biến cũ không còn hợp lệ | Truyền `&entry` nếu hàm chỉ đọc; lấy trước thứ cần (`entry.id.clone()`); `.clone()` cả giá trị chỉ khi thật sự cần hai bản |
| E0502 `cannot borrow ledger as mutable because it is also borrowed as immutable` | Còn tham chiếu `&` được dùng sau đó mà đã đòi `&mut` | Dùng xong phần đọc trước khi ghi, hoặc copy giá trị cần giữ (`map(\|e\| e.id.clone())`) |
| E0499 `cannot borrow ... as mutable more than once at a time` | Hai `&mut` cùng sống tới `Ledger` | Chỉ giữ một `&mut`, ghi tuần tự |
| E0106 `missing lifetime specifier` | Trả tham chiếu mà không có gì để mượn | Trả giá trị sở hữu (`String`); lifetime ở bài 09 |
| E0515 `cannot return reference to local variable` | Đã ghi lifetime (ví dụ `&'static String`) nhưng vẫn trả tham chiếu tới biến cục bộ sắp bị drop | Trả giá trị sở hữu thay vì `&s` |

E0106 và E0515 là hai mặt của cùng một ý: tham chiếu không được sống lâu hơn chủ của nó.

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
cargo run --manifest-path examples/toi-noi-rust/04-ownership-borrow/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/04-ownership-borrow/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/04-ownership-borrow/Cargo.toml
```

Lệnh đầu in (chọn lọc, đúng thứ tự thật):

```text
trước post: JE-000001 (khóa trip:TN-0001:capture) có 2 posting
số dư psp:vixanh:clearing = 25500
số dư customer:CU-001 = -25500
hai người đọc thấy 25500 và 25500
sổ có 2 bút toán
driver:DR-001 = 20400
platform:revenue = 5100
psp:vixanh:clearing = 0
```

Lệnh `--verify` assert số dư `psp:vixanh:clearing` sau khi post `JE-000001` là 25.500, tài xế nhận 20.400, phí nền tảng 5.100, rồi in `VERIFY OK`.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Sổ cái ở đây chỉ nằm trong bộ nhớ một tiến trình. Chưa có PostgreSQL, chưa kiểm <code>idempotency_key</code> trùng, <code>post</code> chưa kiểm tổng bút toán bằng 0 (chỉ <code>--verify</code> kiểm) và chưa trả <code>Result</code> khi lỗi. Ownership chống alias trong một tiến trình; nó không thay transaction hay khóa idempotency khi nhiều instance cùng ghi sổ.</p>
</div>

**Bài tập:** (1) Bỏ comment khối E0502 trong `demo_borrow`, sửa để build xanh mà không đổi `post`. (2) Vì sao lab không derive `Clone` cho `JournalEntry`?

<details><summary>Xem lời giải</summary><p>(1) Lấy thứ cần giữ ra thành giá trị sở hữu trước khi ghi, ví dụ <code>let last = last_entry(&amp;ledger).map(|e| e.id.clone());</code>, hoặc in <code>last</code> trước khi gọi <code>post</code>. Khi đó tham chiếu <code>&amp;</code> hết hạn trước <code>&amp;mut</code>, nên E0502 biến mất. (2) Nhân bản một bút toán tạo hai bản cùng <code>id</code> và <code>idempotency_key</code>, trái với ý "mỗi bút toán chỉ có một chủ". Nếu cần bản sao, hãy viết hàm có tên rõ nghĩa (ví dụ bút toán đảo) thay vì để <code>.clone()</code> xuất hiện tình cờ.</p></details>

Bài trước: [bài 03](/articles/rust-kieu-so-tien-vnd-i64-decimal). Bài tiếp: [bài 05](/articles/rust-struct-enum-match-tai-khoan-but-toan).

## Tài liệu gốc

- [The Rust Programming Language: Understanding Ownership](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html).
- [References and Borrowing](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html).
- [Rust Reference: Destructors](https://doc.rust-lang.org/reference/destructors.html).
- [std::marker::Copy](https://doc.rust-lang.org/std/marker/trait.Copy.html).
- [std::clone::Clone](https://doc.rust-lang.org/std/clone/trait.Clone.html).
- [Error codes index: E0382](https://doc.rust-lang.org/error_codes/E0382.html).
