---
slug: rust-ham-dieu-kien-vong-lap-iterator-so-du
title: "Hàm, điều kiện, vòng lặp và iterator: tính số dư từ dòng bút toán"
summary: "Bài 06: viết hàm có kiểu bắt buộc, phân biệt biểu thức và câu lệnh, dùng loop/while/for rồi iterator để tính cước và số dư từng tài khoản từ dòng bút toán. Có hai hình, lab chạy được và bài tập."
category: programming
level: beginner
tags: [rust, toi-noi, ledger, iterator, closure]
language: vi
---

Chuyến `TN-0001` vừa hoàn tất và sổ cái có ba dòng bút toán: khách trả 25.500, nền tảng nhận 5.100, tài xế `DR-001` nhận 20.400. Câu hỏi của màn hình ví tài xế rất ngắn: số dư của `driver:DR-001` là bao nhiêu? Câu trả lời là một phép duyệt dãy, lọc theo tài khoản rồi cộng lại. Hôm nay ta viết phép duyệt đó theo hai kiểu, vòng `for` và iterator, và xem chúng khác nhau ở đâu.

Đây là bài 06 của lộ trình **Rust cơ bản qua [Tới Nơi](/articles/du-an-mau-toi-noi)**, tiếp nối mô hình tài khoản và bút toán ở [bài 05](/articles/rust-struct-enum-match-tai-khoan-but-toan). Lab chỉ dùng thư viện chuẩn.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Hàm <code>fn fare(km: u32) -> i64 { 12_000 + i64::from(km) * 4_500; }</code> có biên dịch không?</p>
<details><summary>Xem đáp án</summary><p>Không. Dấu <code>;</code> ở cuối biến dòng cuối thành câu lệnh, nên thân hàm không còn giá trị và trả <code>()</code>, trong khi chữ ký hứa <code>i64</code>. Compiler báo E0308 và gợi ý bỏ dấu chấm phẩy.</p></details>
</div>

## Hàm trong Rust hứa gì ở chữ ký?

Chữ ký hàm là hợp đồng: mọi tham số và kiểu trả về đều phải ghi rõ, compiler không suy luận chúng từ thân hàm.

```rust
// src/main.rs
fn fare(distance_km: u32) -> i64 {
    let extra_km = distance_km.saturating_sub(INCLUDED_KM);
    BASE_FARE + i64::from(extra_km) * PER_KM
}
```

`distance_km: u32` nghĩa là số km nguyên, không âm. Gọi `fare(-1)` không biên dịch được: rustc báo ``E0600: cannot apply unary operator `-` to type `u32` ``. Bài [điều kiện, vòng lặp và phương thức trong C#](/articles/csharp-dieu-kien-vong-lap-phuong-thuc) chặn khoảng cách âm lúc chạy bằng exception; ở đây kiểu chặn trước một nửa. Nửa còn lại, khoảng cách quá lớn, vẫn cần kiểm tra bằng logic. Giá trị âm đến từ bên ngoài (JSON, database) phải được chuyển sang `u32` và xử lý khi chuyển thất bại, nội dung bài 07.

Hai điều khác với C# cần nhớ:

- Không có tham số mặc định. Muốn "tùy chọn" thì nói trong kiểu, ví dụ `surcharge: Option<i64>` và gọi `fare_with_surcharge(5, None)`. Phụ phí trong lab chỉ minh họa chữ ký, chưa là quy tắc nghiệp vụ của Tới Nơi.
- Không có overload theo số hay kiểu tham số cho hàm thường. Hai hàm khác nhau thì khác tên. Đa hình theo kiểu dùng trait, nằm ở phần sau của loạt bài.

## Dòng cuối không có dấu chấm phẩy nghĩa là gì?

Rust phân biệt **câu lệnh** (làm một việc, không có giá trị) và **biểu thức** (có giá trị). Một khối `{ ... }` là biểu thức; giá trị của nó là biểu thức cuối cùng **không có `;`**. Thêm `;` thì dòng đó thành câu lệnh và khối trả `()`.

```rust
// src/main.rs
let km_over = {
    let distance = 5_u32;
    distance - INCLUDED_KM
};
```

`km_over` là `3`. Với hàm, quy tắc giống hệt: dòng cuối của `fare` là giá trị trả về, không cần `return`. `return` vẫn dùng được để thoát sớm.

`if` cũng là biểu thức, nên hai nhánh phải trả cùng kiểu:

```rust
// src/main.rs
fn fare_label(distance_km: u32) -> &'static str {
    if distance_km <= INCLUDED_KM {
        "giá cơ bản"
    } else {
        "có km vượt"
    }
}
```

Điều kiện của `if` bắt buộc là `bool`: `if distance_km {` bị từ chối, không có chuyện số khác 0 là true. `else if` nối nhiều nhánh như thường.

## Cước viết bằng vòng lặp và bằng biểu thức

Rust có ba vòng lặp. `for x in iter` duyệt một iterator, `for i in 0..n` chạy từ 0 đến n-1 còn `0..=n` gồm cả n. `while` lặp khi điều kiện đúng. `loop` lặp mãi cho tới `break`, và `break` có thể mang giá trị ra ngoài.

Cách 1 duyệt từng km, đúng như câu mô tả quy tắc: mỗi km thứ ba trở đi cộng 4.500.

```rust
// src/main.rs
fn fare_loop(distance_km: u32) -> i64 {
    let mut total = BASE_FARE;
    for km in 1..=distance_km {
        if km > INCLUDED_KM {
            total += PER_KM;
        }
    }
    total
}
```

Cách 2 là `fare` ở trên: một phép tính, không biến `mut`. Ví dụ chọn quy ước rõ ràng, giống bài C#: chuyến 0 km vẫn thu giá cơ bản, nên `fare(0) = 12.000` (`--verify` kiểm giá trị này). `saturating_sub` cho `0` khi `distance_km` nhỏ hơn 2 thay vì tràn số.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 240" role="img" aria-label="Hình 1. Cước 5 km gồm 2 km đầu giá 12.000 và 3 km vượt mỗi km 4.500, tổng 25.500 VND."><rect x="44" y="30" width="312" height="70" rx="10" class="f-phys-soft s-phys"/><text x="60" y="58" class="t-b">2 km đầu: 12.000</text><text x="60" y="80" class="t-small">giá cơ bản, gồm km 1 và km 2</text><rect x="364" y="30" width="152" height="70" rx="10" class="f-pk-soft s-pk"/><text x="380" y="58" class="t-b">km 3: +4.500</text><text x="380" y="80" class="t-small">km vượt</text><rect x="524" y="30" width="152" height="70" rx="10" class="f-pk-soft s-pk"/><text x="540" y="58" class="t-b">km 4: +4.500</text><text x="540" y="80" class="t-small">km vượt</text><rect x="684" y="30" width="152" height="70" rx="10" class="f-pk-soft s-pk"/><text x="700" y="58" class="t-b">km 5: +4.500</text><text x="700" y="80" class="t-small">km vượt</text><path d="M40 118 L840 118" class="f-none s-line" stroke-width="2"/><path d="M40 112 L40 124 M360 112 L360 124 M520 112 L520 124 M680 112 L680 124 M840 112 L840 124" class="f-none s-line" stroke-width="2"/><text x="40" y="142" text-anchor="middle" class="t-small">0 km</text><text x="360" y="142" text-anchor="middle" class="t-small">2 km</text><text x="520" y="142" text-anchor="middle" class="t-small">3 km</text><text x="680" y="142" text-anchor="middle" class="t-small">4 km</text><text x="840" y="142" text-anchor="middle" class="t-small">5 km</text><rect x="40" y="162" width="796" height="62" rx="10" class="f-phys-soft s-phys"/><text x="60" y="188" class="t-b">fare(5) = 12.000 + 3 × 4.500 = 25.500 VND</text><text x="60" y="209" class="t-small">fare(2) = 12.000, không có km vượt · fare(0) = 12.000 theo quy ước luôn thu giá cơ bản</text></svg>
</div>
<figcaption>Hình 1. Hàm <code>fare</code>: 2 km đầu trả giá cơ bản, mỗi km vượt cộng 4.500. Cả hai cách viết cho cùng kết quả.</figcaption>
</figure>

`loop` với `break` mang giá trị hợp khi chưa biết trước số vòng. Ví dụ: tìm km nhỏ nhất làm cước vượt 30.000.

```rust
// src/main.rs
fn first_km_over_loop(limit: i64) -> u32 {
    let mut km = 0;
    loop {
        if fare(km) > limit {
            break km;
        }
        km += 1;
    }
}
```

Kết quả là 7, vì `fare(6) = 30.000` chưa vượt còn `fare(7) = 34.500`. Lab có thêm bản `while fare(km) <= limit` cho cùng kết quả; `while` gọn hơn khi điều kiện dừng nằm gọn trong một biểu thức.

## iter, iter_mut hay into_iter: ai giữ dữ liệu?

Iterator là thứ có phương thức `next()` trả `Some(phần_tử)` hoặc `None` khi hết. Ba cách lấy iterator từ `Vec<Posting>` khác nhau ở quyền sở hữu, tiếp nối bài 04:

| Cách gọi | Phần tử nhận được | Sau khi duyệt |
| --- | --- | --- |
| `postings.iter()` hay `for p in &postings` | `&Posting` (mượn đọc) | `postings` dùng tiếp được |
| `postings.iter_mut()` hay `for p in &mut postings` | `&mut Posting` (mượn sửa) | dùng tiếp được, dữ liệu đã đổi |
| `postings.into_iter()` hay `for p in postings` | `Posting` (lấy quyền sở hữu) | `postings` đã bị move |

Sổ cái append-only không sửa bút toán đã ghi, nên `iter_mut` trong lab chạy trên **bản sao** để dựng bút toán đảo `JE-000002`:

```rust
// src/main.rs
let mut reversal = postings.clone();
for p in reversal.iter_mut() {
    p.entry = "JE-000002".to_string();
    p.amount = -p.amount;
}
```

Cộng số dư driver qua cả `JE-000001` và `JE-000002` được 0. Còn `into_iter` biến dãy thành đầu vào của một đường ống rồi bỏ nó đi. Dùng lại biến sau đó là lỗi thật. Thử riêng trong một crate nhỏ (bản rút gọn), rustc báo:

```text
error[E0382]: borrow of moved value: `postings`
 --> src/main.rs:6:32
  |
5 |     let amounts = postings.into_iter().map(|p| p.amount).collect::<Vec<_>>();
  |                            ----------- `postings` moved due to this method call
6 |     println!("{amounts:?} {}", postings.len());
  |                                ^^^^^^^^ value borrowed here after move
```

Lab giữ dòng này dưới dạng chú thích để build xanh.

## Số dư của một tài khoản: vòng for rồi iterator

Dòng bút toán trong lab là phiên bản gọn của `Posting` ở bài 05: `amount` là `i64` đơn vị đồng có dấu thay cho `Money` (cùng quy ước: âm là tiền ra khỏi tài khoản, dương là tiền vào), và có thêm mã bút toán `entry`, để ví dụ tập trung vào vòng lặp:

```rust
// src/main.rs (rút gọn)
#[derive(Debug, Clone)]
struct Posting {
    entry: String,
    account: String,
    amount: i64,
}
```

Bản vòng `for`:

```rust
// src/main.rs
fn balance_of_loop(postings: &[Posting], account: &str) -> i64 {
    let mut balance = 0;
    for p in postings {
        if p.account == account {
            balance += p.amount;
        }
    }
    balance
}
```

Bản iterator làm đúng việc đó:

```rust
// src/main.rs
fn balance_of(postings: &[Posting], account: &str) -> i64 {
    postings
        .iter()
        .filter(|p| p.account == account)
        .map(|p| p.amount)
        .sum::<i64>()
}
```

`&[Posting]` là tham chiếu mượn tới một dãy; bài 09 nói kỹ về slice. `|p| p.amount` là **closure**: hàm vô danh, tham số nằm giữa hai dấu `|`. `filter` nhận closure trả `bool`; `map` nhận closure biến mỗi phần tử thành giá trị mới. Closure đọc được biến xung quanh nó, ở đây là `account`. Cú pháp `.sum::<i64>()` chỉ rõ kiểu tổng, vì `sum` có thể trả nhiều kiểu số khác nhau. Ở vị trí trả về của `balance_of`, chữ ký `-> i64` cũng đủ để suy ra, nhưng `let total = ….sum();` đứng một mình bị rustc từ chối với E0283 `type annotations needed`.

Cùng họ adapter còn vài công cụ lab đã chạy: `fold(0_i64, |acc, p| acc + p.amount)` cộng dồn với giá trị khởi đầu tự chọn, `count()` đếm phần tử, `max_by_key(|p| p.amount)` trả `Option` của phần tử lớn nhất, `enumerate()` thêm số thứ tự và `zip` ghép hai dãy theo cặp. Cuối cùng `collect::<Vec<_>>()` gom kết quả vào `Vec`; dấu `_` để compiler suy kiểu phần tử.

```rust
// src/main.rs
fn balances(postings: &[Posting]) -> Vec<(&'static str, i64)> {
    ACCOUNTS
        .iter()
        .map(|&account| (account, balance_of(postings, account)))
        .collect::<Vec<_>>()
}
```

## Iterator lazy: closure chạy lúc nào?

Adapter như `filter` hay `map` chỉ **mô tả** một đường ống, chưa chạy gì. Công việc bắt đầu khi một bộ tiêu thụ (`sum`, `count`, `collect`, `for`) kéo phần tử qua `next()`. Mỗi tầng kéo tầng trước, nên một phần tử đi hết đường ống rồi mới tới phần tử sau.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Hình 2. Đường ống iterator lazy: dữ liệu chảy từ Vec qua iter, filter, map tới sum; yêu cầu next() chạy ngược từ sum về Vec."><defs><marker id="rust-ham-dieu-kien-vong-lap-iterator-so-du-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker><marker id="rust-ham-dieu-kien-vong-lap-iterator-so-du-0-b" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs><rect x="30" y="40" width="140" height="80" rx="10" class="f-phys-soft s-phys"/><text x="42" y="75" class="t-b">Vec&lt;Posting&gt;</text><text x="42" y="98" class="t-small">3 posting</text><rect x="200" y="40" width="140" height="80" rx="10" class="f-phys-soft s-phys"/><text x="212" y="75" class="t-b">iter()</text><text x="212" y="98" class="t-small">mượn từng &amp;Posting</text><rect x="370" y="40" width="140" height="80" rx="10" class="f-phys-soft s-phys"/><text x="382" y="75" class="t-b">filter</text><text x="382" y="98" class="t-small">giữ amount &gt; 0</text><rect x="540" y="40" width="140" height="80" rx="10" class="f-phys-soft s-phys"/><text x="552" y="75" class="t-b">map</text><text x="552" y="98" class="t-small">lấy p.amount</text><rect x="710" y="40" width="140" height="80" rx="10" class="f-pk-soft s-pk"/><text x="722" y="75" class="t-b">sum::&lt;i64&gt;()</text><text x="722" y="98" class="t-small">kéo hết, cộng dồn</text><path d="M170 80 L200 80" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-ham-dieu-kien-vong-lap-iterator-so-du-0-a)"/><path d="M340 80 L370 80" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-ham-dieu-kien-vong-lap-iterator-so-du-0-a)"/><path d="M510 80 L540 80" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-ham-dieu-kien-vong-lap-iterator-so-du-0-a)"/><path d="M680 80 L710 80" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-ham-dieu-kien-vong-lap-iterator-so-du-0-a)"/><path d="M780 120 L780 160 L100 160 L100 124" class="f-none s-pk dash" stroke-width="2" marker-end="url(#rust-ham-dieu-kien-vong-lap-iterator-so-du-0-b)"/><text x="440" y="190" text-anchor="middle" class="t-b">next() đi ngược: sum kéo map, map kéo filter, filter kéo iter, iter lấy phần tử từ Vec</text><text x="440" y="214" text-anchor="middle" class="t-small">Dữ liệu chảy sang phải. Chưa có sum, count hay collect thì không ai gọi next() và closure không chạy lần nào.</text></svg>
</div>
<figcaption>Hình 2. Iterator là đường ống kéo: bộ tiêu thụ ở cuối quyết định khi nào có phần tử đi qua.</figcaption>
</figure>

Lab đo điều này bằng một bộ đếm trong closure của `filter`:

```rust
// src/main.rs
let mut checked = 0;
{
    let _lazy = postings.iter().filter(|p| {
        checked += 1;
        p.amount > 0
    });
    // Không gọi sum/collect: closure chưa chạy lần nào.
}
println!("sau khi chỉ dựng filter: closure đã chạy {checked} lần");
```

Output thật: `sau khi chỉ dựng filter: closure đã chạy 0 lần`. Sau khi thêm `.count()` vào một đường ống mới, bộ đếm lên 3 và `count` trả 2 posting dương. Nếu bạn viết `v.iter().map(...)` rồi bỏ đó, rustc cảnh báo ``unused `Map` that must be used`` kèm ghi chú `iterators are lazy and do nothing unless consumed`.

Đây cũng là ý của [LINQ và deferred execution](/articles/csharp-linq-deferred-execution) trong C#: `Where` và `Select` mô tả truy vấn, `ToList` hay `Sum` mới thực thi. Điểm khác: iterator Rust bị tiêu thụ trong một lượt, vì `sum` lấy `self` theo giá trị. Muốn duyệt lần hai thì gọi `iter()` lại, không có chuyện một biến truy vấn chạy lại nhiều lần như `IEnumerable`. Iterator không hứa nhanh hơn vòng `for`; chọn theo độ rõ, đo khi cần.

## Chọn vòng lặp hay iterator?

| Vòng lặp | Iterator | Khi nào |
| --- | --- | --- |
| `for p in &postings { if … { total += p.amount } }` | `.iter().filter(…).map(…).sum::<i64>()` | Tính một giá trị từ cả dãy: iterator ngắn và không cần biến `mut` |
| `for` kèm `return` hay `break` sớm | `.all(…)`, `.any(…)` | Kiểm tra điều kiện trên dãy: dùng hàm có sẵn, dừng ngay khi đủ kết luận |
| `loop { … break giá_trị }` | không có tương đương gọn | Chưa biết số vòng, cần thử tới khi thỏa |
| `for` có tác dụng phụ (in, ghi log) | tránh nhồi vào closure của `map` | Việc cần thứ tự và hiệu ứng rõ ràng: viết bằng `for` |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Số dư ở đây là phép duyệt qua <code>Vec</code> trong bộ nhớ, với tài khoản cho trước và <code>i64</code> trần thay cho <code>Money</code>. Chưa có chặn tràn số (<code>sum</code> trên <code>i64</code> theo quy tắc tràn số ở bài 03, sổ cái thật phải dùng <code>checked_add</code>), chưa có map tài khoản → số dư, chưa có lưu trữ và đồng thời. Bút toán trong lab bỏ <code>psp:vixanh:clearing</code> cho gọn. Sổ cái production không tính lại số dư bằng cách duyệt toàn bộ dòng mỗi lần: các bài sau về schema và projection xử lý việc này.</p>
</div>

## Chạy ví dụ và kiểm chứng

Từ gốc repository:

```bash
cargo run --manifest-path examples/toi-noi-rust/06-ham-vong-lap-iterator/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/06-ham-vong-lap-iterator/Cargo.toml -- --verify
```

Output thật của lệnh đầu (rút gọn, chỉ giữ phần số dư; các dòng cước và lazy đã trích ở trên):

```text
#1 JE-000001  -25500 customer:CU-001
#2 JE-000001    5100 platform:revenue
#3 JE-000001   20400 driver:DR-001
số dư driver:DR-001 = 20400
  customer:CU-001     -25500
  platform:revenue      5100
  driver:DR-001        20400
tổng tiền vào = 25500, tổng mọi posting = 0
posting lớn nhất: driver:DR-001 (20400)
đảo: số dư driver sau JE-000002 = 0
```

Lệnh sau in thêm `VERIFY OK (11 kiểm tra)` và thoát với mã 0; kiểm tra nào sai thì in `FAIL: <tên>` ra stderr và thoát với mã 1. Các kiểm tra gồm cước 0, 2, 5, 7 km, `fare_loop` trùng `fare` từ 0 đến 50 km, `loop` trùng `while`, số dư từng tài khoản và tổng mọi posting bằng 0.

**Bài tập:** viết `fn fare_between(from_km: u32, to_km: u32) -> i64` trả tổng cước **phần km vượt** của các km từ `from_km` đến `to_km` (gồm cả hai đầu), chỉ bằng một iterator, không có biến `mut`. Với `from_km = 1`, `to_km = 5` kết quả phải là 13.500. Sau đó thử: nếu bỏ `.sum::<i64>()` ở `balance_of`, compiler phàn nàn gì?

<details><summary>Xem lời giải</summary>

```rust
fn fare_between(from_km: u32, to_km: u32) -> i64 {
    (from_km..=to_km)
        .filter(|&km| km > INCLUDED_KM)
        .map(|_| PER_KM)
        .sum::<i64>()
}
```

Các km 3, 4, 5 vượt 2 km nên có 3 × 4.500 = 13.500. Nếu `from_km` lớn hơn `to_km` thì dải rỗng và kết quả là 0. Nếu bỏ `.sum::<i64>()` ở `balance_of`, thân hàm có kiểu `Map<Filter<…>, …>` chứ không phải `i64`; compiler báo E0308 vì kiểu không khớp chữ ký, và đường ống chưa tiêu thụ không chạy gì.

</details>

Bài trước: [bài 05](/articles/rust-struct-enum-match-tai-khoan-but-toan). Bài tiếp: [bài 07](/articles/rust-result-option-toan-tu-hoi-loi-la-gia-tri).

## Tài liệu gốc

- [The Rust Programming Language: Functions](https://doc.rust-lang.org/book/ch03-03-how-functions-work.html).
- [The Rust Programming Language: Control Flow](https://doc.rust-lang.org/book/ch03-05-control-flow.html).
- [The Rust Programming Language: Closures](https://doc.rust-lang.org/book/ch13-01-closures.html).
- [The Rust Programming Language: Processing a Series of Items with Iterators](https://doc.rust-lang.org/book/ch13-02-iterators.html).
- [std::iter::Iterator](https://doc.rust-lang.org/std/iter/trait.Iterator.html).
- [The Rust Reference: Expressions](https://doc.rust-lang.org/reference/expressions.html).
