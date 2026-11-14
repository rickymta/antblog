---
slug: rust-vec-hashmap-btreemap-lifetime-so-cai-bo-nho
title: "Vec, HashMap, BTreeMap và lifetime cơ bản: sổ cái trong bộ nhớ"
summary: "Bài 09: dựng sổ cái trong bộ nhớ bằng Vec, HashMap, BTreeMap, HashSet; dùng entry API, slice, range query và hiểu vì sao lifetime 'a xuất hiện khi hàm trả về tham chiếu."
category: rust-beginner
level: beginner
tags: [rust, toi-noi, ledger, collections, lifetime]
language: vi
---

Khi Payment báo `PaymentCaptured` cho chuyến `TN-0001`, service sổ cái phải trả lời ngay hai câu hỏi: «key `trip:TN-0001:capture` đã xử lý chưa?» và «`driver:DR-001` đang có bao nhiêu?». Hai câu hỏi cần hai cấu trúc dữ liệu khác nhau, và bút toán cần cái thứ ba để giữ thứ tự. Bài này dựng cả ba trong bộ nhớ, rồi giải thích lifetime qua những hàm trả về tham chiếu vào chính các cấu trúc đó.

Đây là bài 09 của lộ trình **Rust cơ bản qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Lab chỉ dùng thư viện chuẩn, không có dependency; tiền là `i64` đơn vị đồng để tập trung vào collection.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Một <code>Vec</code> có 4 phần tử. <code>v[7]</code> và <code>v.get(7)</code> khác nhau thế nào?</p>
<details><summary>Xem đáp án</summary><p><code>v[7]</code> làm chương trình panic; <code>v.get(7)</code> trả <code>None</code> và để caller quyết định. Lab in ra đúng thông điệp panic ở mục đầu tiên.</p></details>
</div>

## `Vec<T>` truy cập thế nào, và slice là gì?

`Vec<T>` là mảng cấp phát trên heap, tự lớn khi `push`. Truy cập theo chỉ số là O(1); tìm theo giá trị phải quét, O(n). Đoạn dưới (rút gọn từ `src/main.rs`) chạy cả những thao tác hay dùng:

```rust
let mut amounts: Vec<i64> = Vec::with_capacity(8); // cấp sẵn chỗ cho 8 phần tử
for a in [20_400, -25_500, 5_100, 9_600] {
    amounts.push(a);
}
// `get` trả Option, không panic; `v[i]` ngoài biên thì panic.
let get_missing = amounts.get(7).copied();
amounts.sort_by_key(|a| a.abs()); // sắp theo giá trị tuyệt đối
amounts.retain(|a| *a > 0); // giữ lại phần tử thỏa điều kiện
```

```text
Vec: len sau push = 4, get(7) = None
Vec: index ngoài biên -> panic: index out of bounds: the len is 4 but the index is 7
Vec: sort_by_key(abs) = [5100, 9600, 20400, -25500]; retain(>0) = [5100, 9600, 20400]
```

`with_capacity` chỉ cấp sẵn chỗ để tránh cấp phát lại; `len` vẫn là số phần tử thật. Panic nghĩa là giả định của lập trình viên đã sai; với dữ liệu từ ngoài vào, dùng `get` hoặc kiểm tra độ dài trước.

**Slice** `&[T]` là một «cửa sổ» mượn lên dãy phần tử liền nhau, gồm con trỏ và độ dài, không sở hữu dữ liệu. Hàm nhận `&[Posting]` dùng được với `&Vec<Posting>`, mảng hay một đoạn con; `&mut [T]` cho phép sửa tại chỗ nhưng không đổi độ dài:

```rust
fn sum_amounts(postings: &[Posting]) -> i64 {
    postings.iter().map(|p| p.amount).sum()
}

fn flip_signs(postings: &mut [Posting]) {
    for p in postings.iter_mut() {
        p.amount = -p.amount;
    }
}
```

## `HashMap` và `entry`: số dư theo tài khoản

Số dư cần tra theo tên tài khoản, không theo vị trí. `HashMap<K, V>` băm khoá để nhảy thẳng tới ô chứa giá trị: tra và chèn O(1) trung bình. Khoá phải implement `Eq + Hash`; `String` thoả cả hai. Thứ tự duyệt **không xác định** và có thể khác giữa hai lần chạy, nên lab sắp khoá trước khi in:

```rust
let mut rows: Vec<(&String, &i64)> = ledger.balances.iter().collect();
rows.sort_by_key(|(account, _)| account.as_str());
```

Cập nhật số dư là mẫu «có thì cộng, chưa có thì tạo từ 0». `entry` làm đúng việc đó chỉ với một lần băm:

```rust
*self.balances.entry(p.account.clone()).or_insert(0) += p.amount;
```

`entry(key)` trả một ô có thể trống hoặc đầy; `or_insert(0)` điền 0 nếu trống rồi trả `&mut i64`; dấu `*` giải tham chiếu để cộng. Viết `if contains_key { get_mut } else { insert }` thì băm hai ba lần và dài hơn.

## `BTreeMap`, `HashSet` và `VecDeque` dùng khi nào?

`BTreeMap<K, V>` giữ khoá luôn được sắp, nên duyệt theo thứ tự và hỏi theo khoảng được. Giá phải trả là tra O(log n). Lab đặt bút toán vào `BTreeMap<String, JournalEntry>` với khoá `JE-000001`, `JE-000002`…:

```rust
fn entries_from<'a>(&'a self, from: &str) -> impl Iterator<Item = &'a JournalEntry> + 'a {
    // `range(from..)` với `&str` không biên dịch được (E0277); dùng String làm cận dưới.
    self.entries.range(from.to_string()..).map(|(_, e)| e)
}
```

Viết thẳng `range(from..)` với `from: &str` thì compiler suy ra kiểu cận là `&str` và báo `E0277: the trait bound String: Borrow<&str> is not satisfied`. Ép kiểu cận thành `str` bằng `range::<str, _>(from..)` thì gặp một E0277 khác, `the size for values of type str cannot be known at compilation time`, vì impl `RangeBounds<T> for RangeFrom<&T>` chỉ áp dụng cho `T` có kích thước biết trước. Tạo một `String` làm cận dưới là cách ngắn nhất. Còn `+ 'a` ở kiểu trả về nói iterator chỉ mượn `self`; bỏ đi thì edition 2024 coi iterator mượn cả `from`, và caller phải giữ chuỗi `from` sống đến khi dùng xong iterator. Một điểm cần nhớ: khoá `String` được so theo thứ tự chữ cái, nên `JE-9` đứng sau `JE-10`; phải đệm số 0 (`JE-{seq:06}`) thì thứ tự chữ cái mới trùng thứ tự thời gian.

`HashSet<String>` là tập hợp không trùng. `insert` trả `false` khi phần tử đã có, nên một lệnh vừa kiểm tra vừa đánh dấu: đây là bộ nhớ idempotency key của sổ cái. `VecDeque<T>` là hàng đợi hai đầu (`push_back`, `pop_front` đều O(1)); lab dùng nó làm hàng bút toán chờ ghi, vào trước ra trước.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 290" role="img" aria-label="Hình 1. Ba cấu trúc chính của sổ cái: Vec giữ thứ tự thêm, HashMap tra O(1) trung bình không có thứ tự, BTreeMap tra O(log n) và luôn sắp xếp. HashSet ghi nhớ idempotency key."><rect x="20" y="20" width="260" height="190" rx="10" class="f-phys-soft s-phys"/><text x="36" y="50" class="t-b">Vec&lt;Posting&gt;</text><text x="36" y="76" class="t-small">Mảng liền nhau, giữ thứ tự thêm</text><text x="36" y="100" class="t-small">Truy cập theo chỉ số: O(1)</text><text x="36" y="124" class="t-small">Tìm theo giá trị: O(n)</text><text x="36" y="148" class="t-small">Chỉ số ngoài biên: v[i] panic</text><text x="36" y="188" class="t-b">Dùng cho: các dòng posting</text><rect x="310" y="20" width="260" height="190" rx="10" class="f-pk-soft s-pk"/><text x="326" y="50" class="t-b">HashMap&lt;String, i64&gt;</text><text x="326" y="76" class="t-small">Băm khoá, nhảy tới ô chứa giá trị</text><text x="326" y="100" class="t-small">Tra, chèn: O(1) trung bình</text><text x="326" y="124" class="t-small">Thứ tự duyệt không xác định</text><text x="326" y="148" class="t-small">Khoá cần Eq + Hash</text><text x="326" y="188" class="t-b">Dùng cho: số dư tài khoản</text><rect x="600" y="20" width="260" height="190" rx="10" class="f-phys-soft s-phys"/><text x="616" y="50" class="t-b">BTreeMap&lt;String, Entry&gt;</text><text x="616" y="76" class="t-small">Cây cân bằng, khoá luôn sắp xếp</text><text x="616" y="100" class="t-small">Tra, chèn: O(log n)</text><text x="616" y="124" class="t-small">Duyệt có thứ tự, có range</text><text x="616" y="148" class="t-small">Khoá cần Ord</text><text x="616" y="188" class="t-b">Dùng cho: bút toán theo id</text><rect x="20" y="232" width="840" height="40" rx="10" class="f-warn-soft s-warn dash"/><text x="36" y="257" class="t-b">HashSet&lt;String&gt;<tspan class="t-small" dx="10">idempotency key đã thấy · insert trả false khi trùng · O(1) trung bình</tspan></text></svg>
</div>
<figcaption>Hình 1. Ba cấu trúc chính của sổ cái và <code>HashSet</code> ghi nhớ key. Chọn theo câu hỏi cần trả lời, không theo thói quen.</figcaption>
</figure>

## `Ledger` giữ ba cấu trúc khớp nhau thế nào?

Ba cấu trúc gom vào một struct. Quy tắc bất biến: mỗi bút toán đã ghi có đúng một mục trong `entries`, một key trong `seen_keys`, và mọi posting của nó đã cộng vào `balances`.

```rust
#[derive(Default)]
struct Ledger {
    entries: BTreeMap<String, JournalEntry>, // sắp theo id (JE-000001, ...)
    balances: HashMap<String, i64>,          // số dư theo tài khoản
    seen_keys: HashSet<String>,              // idempotency key đã ghi
}

fn post(&mut self, entry: JournalEntry) -> Result<(), PostError> {
    // (rút gọn: kiểm tra có ít nhất 2 posting)
    let sum: i64 = entry.postings.iter().map(|p| p.amount).sum();
    if sum != 0 {
        return Err(PostError::Unbalanced(sum));
    }
    if !self.seen_keys.insert(entry.idempotency_key.clone()) {
        return Err(PostError::DuplicateKey(entry.idempotency_key));
    }
    for p in &entry.postings {
        *self.balances.entry(p.account.clone()).or_insert(0) += p.amount;
    }
    self.entries.insert(entry.id.clone(), entry);
    Ok(())
}
```

Thứ tự kiểm tra có chủ ý: kiểm tra cân bằng **trước** khi `insert` key. Nếu đảo lại, một bút toán lệch bị từ chối vẫn «đốt» key, và bản sửa đúng gửi lại sẽ bị coi là trùng.

Trong bộ nhớ, `post` chạy trên `&mut self`, nên không ai thấy trạng thái nửa vời. Với database thì khác: ba thay đổi này là ba bảng hoặc ba câu lệnh, có thể thất bại ở giữa chừng. Khi đó phải gói trong một transaction; phần đó thuộc loạt trung bình (bài về PostgreSQL), còn lý do idempotency xem [bài giao dịch và idempotency của loạt .NET](/articles/dotnet10-transaction-concurrency-idempotency).

## Lifetime: vì sao `pick` cần `'a` còn `largest_posting` thì không?

Mỗi tham chiếu trỏ vào dữ liệu có một chủ. Compiler phải chắc rằng tham chiếu không sống lâu hơn chủ. Phần lớn thời gian nó tự suy ra, gọi là lifetime elision. Với một tham chiếu vào và một tham chiếu ra, kết quả chỉ có thể mượn từ tham số đó:

```rust
fn largest_posting(entry: &JournalEntry) -> Option<&Posting> {
    entry.postings.iter().max_by_key(|p| p.amount.abs())
}
```

Hai tham chiếu vào thì khác: kết quả mượn từ `a` hay `b`? Compiler không đoán. Bỏ chú thích sẽ gặp lỗi thật:

```text
error[E0106]: missing lifetime specifier
 --> src/main.rs:2:38
  |
2 | fn pick(a: &Posting, b: &Posting) -> &Posting {
  |            --------     --------     ^ expected named lifetime parameter
  |
  = help: this function's return type contains a borrowed value, but the signature does not say whether it is borrowed from `a` or `b`
```

Ta nói rõ: cả hai đầu vào và kết quả cùng sống ít nhất trong khoảng `'a`.

```rust
fn pick<'a>(a: &'a Posting, b: &'a Posting) -> &'a Posting {
    if a.amount.abs() >= b.amount.abs() { a } else { b }
}
```

`'a` không kéo dài sự sống của dữ liệu; nó chỉ **đặt tên** cho khoảng chung để compiler kiểm tra. Với phương thức có `&self`, kết quả mặc định mượn từ `self`, đó là lý do `balance(&self, account: &str) -> Option<&i64>` không cần `'a`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Hình 2. Tham chiếu r vẫn được dùng sau khi chủ p bị drop ở cuối khối, nên compiler báo E0597. Đoạn r còn dùng sau điểm drop là phần không hợp lệ."><text x="20" y="40" class="t-small">Thứ tự thực thi trong khối →</text><text x="20" y="85" class="t-b">Chủ</text><rect x="120" y="60" width="300" height="40" rx="8" class="f-phys-soft s-phys"/><text x="136" y="85" class="t-b">p: Posting (sống đến hết khối)</text><text x="20" y="155" class="t-b">Tham chiếu</text><rect x="240" y="130" width="180" height="40" rx="8" class="f-pk-soft s-pk"/><text x="256" y="155" class="t-b">r = &amp;p (hợp lệ)</text><rect x="420" y="130" width="240" height="40" rx="8" class="f-warn-soft s-warn dash"/><text x="436" y="155" class="t-b">r còn dùng, p đã chết</text><path d="M420 50 L420 200" class="f-none s-warn dash" stroke-width="2"/><text x="432" y="200" class="t-small">dấu } kết thúc khối: p bị drop</text><rect x="690" y="60" width="170" height="110" rx="10" class="f-warn-soft s-warn"/><text x="706" y="90" class="t-b">E0597</text><text x="706" y="114" class="t-small">`p` does not live</text><text x="706" y="134" class="t-small">long enough</text><text x="20" y="250" class="t-small">Quy tắc: tham chiếu không được sống lâu hơn chủ của nó.</text></svg>
</div>
<figcaption>Hình 2. Tham chiếu <code>r</code> vẫn được dùng sau khi chủ <code>p</code> bị drop: compiler báo E0597 ngay khi biên dịch, không đợi đến lúc chạy.</figcaption>
</figure>

Cùng quy tắc giải thích lỗi hay gặp nhất với collection: giữ tham chiếu vào map rồi sửa map.

```rust
let driver = balances.get("driver:DR-001");
balances.insert("platform:revenue".to_string(), 5_100);
println!("{driver:?}");
```

```text
error[E0502]: cannot borrow `balances` as mutable because it is also borrowed as immutable
5 |     let driver = balances.get("driver:DR-001");
  |                  -------- immutable borrow occurs here
6 |     balances.insert("platform:revenue".to_string(), 5_100);
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
7 |     println!("{driver:?}");
  |                ------ immutable borrow later used here
```

`get` trả `Option<&i64>`, trỏ thẳng vào ô trong bảng băm. `insert` có thể làm bảng cấp phát lại và dời ô, khi đó tham chiếu cũ treo lơ lửng. Rust chặn từ compile time. Cách sửa: sao chép giá trị ra (`.copied()`) trước khi `insert`, hoặc dùng xong tham chiếu rồi mới sửa. Các đoạn lỗi trên đã được tạo và chạy `rustc` thật, rồi bỏ khỏi lab để lab biên dịch được.

## Chọn cấu trúc nào, và C# có gì tương ứng?

| Thao tác | Vec | HashMap | BTreeMap |
| --- | --- | --- | --- |
| Tra theo chỉ số | O(1) | không có | không có |
| Tra theo khoá / tìm theo giá trị | O(n) quét | O(1) trung bình | O(log n) |
| Thêm cuối / chèn khoá mới | O(1) khấu hao | O(1) trung bình | O(log n) |
| Thứ tự duyệt | thứ tự thêm | không xác định | theo khoá |
| Truy vấn khoảng | sau khi sắp xếp | không | `range` |
| Yêu cầu với khoá | không có | `Eq + Hash` | `Ord` |

Đây là độ phức tạp tiệm cận, không phải thời gian đo: với vài chục phần tử, `Vec` quét thẳng thường không kém bảng băm. Chỉ đổi cấu trúc khi có câu hỏi truy vấn rõ ràng, và chỉ tối ưu khi có số đo (loạt nâng cao có bài đo hiệu năng).

Người đã đọc [bài generics và collection của loạt C#](/articles/csharp-generics-collections) sẽ thấy quen: `List<T>` ứng với `Vec<T>`, `Dictionary<K, V>` với `HashMap<K, V>`, `SortedDictionary<K, V>` với `BTreeMap<K, V>`, `Queue<T>` với `VecDeque<T>`. Khác biệt đáng nhớ: chỉ số ngoài biên ném `ArgumentOutOfRangeException` trong C#, còn Rust panic; C# thường tra bằng `TryGetValue`, Rust trả `Option<&V>` và buộc ta xử lý nhánh thiếu.

## Chạy ví dụ và kiểm chứng

Từ gốc repository:

```bash
cargo run --manifest-path examples/toi-noi-rust/09-vec-hashmap-lifetime/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/09-vec-hashmap-lifetime/Cargo.toml -- --verify
```

Ba chuyến `TN-0001`..`TN-0003` có cước 25.500, 12.000 và 34.500; mỗi bút toán trừ khách, cộng 80% cho tài xế và 20% cho nền tảng. Một bút toán thứ tư gửi lại key `trip:TN-0001:capture`. Kết quả chạy thật (lược năm dòng đầu: ba dòng `Vec` đã in ở trên và hai dòng `Slice`):

```text
post JE-000001 -> Ok(())
post JE-000002 -> Ok(())
post JE-000003 -> Ok(())
post JE-000004 -> Err(DuplicateKey("trip:TN-0001:capture"))
  customer:CU-001        -25500
  customer:CU-002        -12000
  customer:CU-003        -34500
  driver:DR-001           57600
  platform:revenue        14400
Tổng mọi số dư = 0
range("JE-000002"..) -> ["JE-000002", "JE-000003"]
largest_posting(JE-000001) = Some(Posting { account: "customer:CU-001", amount: -25500 })
pick = Posting { account: "driver:DR-001", amount: 20400 }
lệch 1 đồng -> Err(Unbalanced(1)); sửa rồi ghi lại -> Ok(())
VERIFY OK
```

`--verify` kiểm các giá trị này bằng so sánh thật: tài xế 57.600 (20.400 + 9.600 + 27.600), nền tảng 14.400, tổng bằng 0, key trùng bị từ chối, `range` trả 2 bút toán, ba cấu trúc khớp nhau. Sai điều nào thì in `VERIFY FAIL` và thoát với mã 1.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Đây là sổ cái mất sạch khi tiến trình dừng: không có lưu trữ, không transaction, không khoá khi nhiều luồng cùng ghi. Phép cộng <code>+=</code> trên <code>i64</code> chưa dùng <code>checked_add</code> (xem bài 03), phí 20% là phép rút gọn <code>fare / 5</code>: chia nguyên cắt phần lẻ, chỉ đúng vì ba cước mẫu chia hết cho 5 (cước 25.503 sẽ bị cắt mất 0,6 đồng thay vì làm tròn theo quy tắc của bài 03), và <code>TooFewPostings</code>, <code>Unbalanced</code> mới là kiểm tra tối thiểu. Production cần database, <code>checked_*</code> và kiểm thử đồng thời.</p>
</div>

## Bài tập

Thêm hàm `entries_between(&self, from: &str, to: &str)` trả `Vec<&JournalEntry>` gồm các bút toán có id từ `from` đến `to` (gồm cả hai đầu). Gợi ý: `range` nhận cả `RangeInclusive`. Hàm có ba tham chiếu vào: ghi `'a` ở đâu, hay không cần ghi?

<details><summary>Xem lời giải</summary>
<pre><code class="language-rust">fn entries_between(&amp;self, from: &amp;str, to: &amp;str) -&gt; Vec&lt;&amp;JournalEntry&gt; {
    self.entries
        .range(from.to_string()..=to.to_string())
        .map(|(_, e)| e)
        .collect()
}</code></pre>
<p>Không cần ghi <code>'a</code>. Với phương thức có <code>&amp;self</code>, elision gán cho mọi tham chiếu trong kết quả lifetime của <code>self</code>, dù <code>from</code> và <code>to</code> cũng là tham chiếu. Điều này đúng ý ta: kết quả mượn từ sổ cái, không mượn từ hai chuỗi cận. Hàm có trong <code>src/main.rs</code> của lab, và <code>--verify</code> kiểm <code>entries_between("JE-000001", "JE-000002")</code> trả 2 bút toán.</p>
</details>

Bài trước: [bài 08](/articles/rust-string-str-ngay-gio-dinh-dang-tien). Bài tiếp: [bài 10](/articles/rust-module-crate-cargo-test-clippy).

## Tài liệu gốc

- [The Rust Book: Common Collections](https://doc.rust-lang.org/book/ch08-00-common-collections.html).
- [The Rust Book: Validating References with Lifetimes](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html).
- [std::collections](https://doc.rust-lang.org/std/collections/index.html) và [HashMap::entry](https://doc.rust-lang.org/std/collections/struct.HashMap.html#method.entry).
- [BTreeMap::range](https://doc.rust-lang.org/std/collections/struct.BTreeMap.html#method.range).
- [Reference: lifetime elision](https://doc.rust-lang.org/reference/lifetime-elision.html).
