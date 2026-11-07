---
slug: rust-trait-generics-chinh-sach-phi-lam-tron
title: "Trait, generics và trait bound: chính sách phí và làm tròn thay thế được"
summary: "Bài 11: tách phí nền tảng và cách làm tròn thành hai trait, viết generic có trait bound, thêm chính sách phí thứ hai mà không sửa hàm dựng bút toán; so static với dynamic dispatch và đối chiếu interface C#."
category: rust-beginner
level: beginner
tags: [rust, toi-noi, ledger, trait, generics]
language: vi
---

Phí nền tảng của chuyến `TN-0001` là 20% của 25.500, tức 5.100 đồng. Ở bài 10 con số này viết cứng trong code. Giả sử quý sau Tới Nơi thêm một chính sách phí cố định cho chuyến ngắn, rồi một chính sách theo bậc cước. Nếu tỷ lệ 20% và cách làm tròn nằm ngay trong hàm dựng bút toán, mỗi chính sách mới là một lần sửa hàm đó, và một lần nguy cơ làm lệch sổ. Bài này tách hai thứ biến đổi (cách tính phí, cách làm tròn) thành hai trait, để hàm dựng bút toán không phải đổi.

Đây là bài 11 của lộ trình **Rust cơ bản qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Lab mở rộng thư viện của [bài 10](/articles/rust-module-crate-cargo-test-clippy) và dùng `rust_decimal` như ở [bài 03](/articles/rust-kieu-so-tien-vnd-i64-decimal) để làm tròn nửa đồng.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Hàm <code>fn capture_entry&lt;F: FeePolicy&gt;(trip: &amp;TripCharge, policy: &amp;F)</code> có nhận được <code>&amp;dyn FeePolicy</code> (mượn từ một <code>Box&lt;dyn FeePolicy&gt;</code>) không?</p>
<details><summary>Xem đáp án</summary><p><b>Không, nếu để nguyên như vậy.</b> Tham số kiểu <code>F</code> mặc định phải có kích thước biết lúc biên dịch (<code>Sized</code>), còn <code>dyn FeePolicy</code> thì không. Rustc báo E0277 và gợi ý thêm <code>?Sized</code>. Mục «Static dispatch hay dynamic dispatch» có thông điệp lỗi.</p></details>
</div>

## Trait là gì, và nó khác interface C# ở đâu?

Trait là một hợp đồng: tập phương thức mà một kiểu hứa có. `impl Trait for Kiểu` là lời hứa đó. Một phương thức có thể có thân hàm ngay trong trait, gọi là default method, và mọi kiểu implement trait đều có sẵn nó.

| Việc | C# | Rust |
| --- | --- | --- |
| Khai báo hợp đồng | `interface IFeePolicy` | `trait FeePolicy` |
| Gắn hợp đồng vào kiểu | Ở chỗ khai báo: `class X : IFeePolicy` | Chỗ riêng: `impl FeePolicy for X`, có thể ở module khác |
| Phương thức mặc định | Default interface method (C# 8) | Default method trong trait |
| Ràng buộc generic | `where T : IFeePolicy` | `T: FeePolicy` (trait bound) |
| Biến có kiểu hợp đồng | `IFeePolicy p`: gọi qua interface lúc chạy (JIT có thể devirtualize) | Chọn: generic hoặc `impl Trait` (tĩnh), `&dyn Trait` hoặc `Box<dyn Trait>` (động) |
| Mã sinh ra cho generic | Runtime sinh bản riêng cho từng value type, dùng chung một bản cho mọi reference type | Compiler sinh bản riêng cho từng kiểu cụ thể lúc biên dịch (monomorphization) |

Vì `impl` đứng riêng, Rust có quy tắc mồ côi (orphan rule): chỉ được implement trait cho kiểu nếu trait hoặc kiểu là của crate mình. Theo quy tắc này, chính sách `TieredFee` trong `src/main.rs` implement trait của thư viện được, còn bạn không thể implement trait của thư viện chuẩn cho kiểu của thư viện khác. Chi tiết generic C# ở [bài về generics và collections](/articles/csharp-generics-collections); so sánh trên chỉ để định hướng.

## RoundingPolicy và FeePolicy: hai trait của sổ cái

`RoundingPolicy` trả lời câu hỏi "nửa đồng làm tròn thế nào". Phương thức bắt buộc là `round_to_dong`; `round_ratio` là default method, dùng lại nó:

```rust
// src/policy.rs (rút gọn chú thích)
pub trait RoundingPolicy {
    /// Tên hiển thị của chính sách.
    fn name(&self) -> &'static str;

    /// Phương thức bắt buộc: làm tròn về đồng; `Overflow` nếu không vừa `i64`.
    fn round_to_dong(&self, amount: Decimal) -> Result<Money, LedgerError>;

    /// Default method: nhân cước với tỷ lệ rồi làm tròn.
    fn round_ratio(&self, base: Money, rate: Decimal) -> Result<Money, LedgerError> {
        let raw = Decimal::from(base.dong())
            .checked_mul(rate)
            .ok_or(LedgerError::Overflow)?;
        self.round_to_dong(raw)
    }
}
```

Hai cách triển khai nằm cạnh nhau. Quy ước của Tới Nơi là `MidpointAwayFromZero`; bản thứ hai (banker's rounding) có để thấy sự khác nhau ở đúng nửa đồng:

```rust
// src/policy.rs (rút gọn: bỏ derive và hàm phụ round_with)
pub struct HalfAwayFromZero;

impl RoundingPolicy for HalfAwayFromZero {
    fn name(&self) -> &'static str { "nửa ra xa 0" }
    fn round_to_dong(&self, amount: Decimal) -> Result<Money, LedgerError> {
        round_with(RoundingStrategy::MidpointAwayFromZero, amount)
    }
}
// NearestEven: giống hệt, dùng RoundingStrategy::MidpointNearestEven.
```

`FeePolicy` có một phương thức bắt buộc, `fee`, và một default method, `split`, chia cước giữa nền tảng và tài xế. Trait chỉ ghi được kiểu của hợp đồng, không ghi được "phí không âm và không vượt cước". Điều kiện đó nằm ở default method, nên mọi chính sách, kể cả chính sách do người khác viết, đi qua cùng một bước kiểm:

```rust
// src/policy.rs (rút gọn: bớt chú thích, gộp dòng cuối)
pub trait FeePolicy {
    fn name(&self) -> String;

    /// Phương thức bắt buộc: phí nền tảng của một khoản cước.
    fn fee(&self, fare: Money) -> Result<Money, LedgerError>;

    /// Default method: chia cước và kiểm hợp đồng (0 <= phí <= cước).
    fn split(&self, fare: Money) -> Result<FeeSplit, LedgerError> {
        let fee = self.fee(fare)?;
        if fee < Money::ZERO || fee > fare {
            return Err(LedgerError::InvalidFee { fee, fare });
        }
        let driver_net = fare.checked_sub(fee).ok_or(LedgerError::Overflow)?;
        Ok(FeeSplit { platform_fee: fee, driver_net })
    }
}
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 1. Ba chính sách phí PercentageFee, FlatFee và TieredFee cùng implement trait FeePolicy; hàm capture_entry chỉ phụ thuộc vào trait nên thêm chính sách mới không phải sửa nó."><defs><marker id="rust-trait-generics-chinh-sach-phi-lam-tron-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><text x="20" y="28" class="t-b">impl FeePolicy for …</text><rect x="20" y="46" width="240" height="56" rx="8" class="f-phys-soft s-phys"/><text x="34" y="68" class="t-b">PercentageFee&lt;R&gt;</text><text x="34" y="88" class="t-small">R: RoundingPolicy · policy.rs</text><rect x="20" y="112" width="240" height="56" rx="8" class="f-phys-soft s-phys"/><text x="34" y="134" class="t-b">FlatFee</text><text x="34" y="154" class="t-small">phí cố định · src/policy.rs</text><rect x="20" y="178" width="240" height="56" rx="8" class="f-warn-soft s-warn"/><text x="34" y="200" class="t-b">TieredFee</text><text x="34" y="220" class="t-small">phí theo bậc · src/main.rs</text><rect x="330" y="46" width="230" height="188" rx="10" class="f-pk-soft s-pk"/><text x="346" y="76" class="t-b">trait FeePolicy</text><text x="346" y="108" class="t-small">bắt buộc: name, fee</text><text x="346" y="132" class="t-small">mặc định: split</text><text x="346" y="168" class="t-small">split kiểm 0 &lt;= phí &lt;= cước</text><text x="346" y="190" class="t-small">cho mọi chính sách</text><rect x="630" y="46" width="230" height="188" rx="10" class="f-phys-soft s-phys"/><text x="646" y="76" class="t-b">capture_entry&lt;F&gt;</text><text x="646" y="108" class="t-small">F: FeePolicy + ?Sized</text><text x="646" y="132" class="t-small">gọi policy.split(cước)</text><text x="646" y="168" class="t-small">trả JournalEntry cân</text><text x="646" y="190" class="t-small">không sửa khi thêm chính sách</text><path d="M260 74 L328 120" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-trait-generics-chinh-sach-phi-lam-tron-0-a)"/><path d="M260 140 L328 140" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-trait-generics-chinh-sach-phi-lam-tron-0-a)"/><path d="M260 206 L328 160" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-trait-generics-chinh-sach-phi-lam-tron-0-a)"/><path d="M630 140 L562 140" class="f-none s-phys dash" stroke-width="2" marker-end="url(#rust-trait-generics-chinh-sach-phi-lam-tron-0-a)"/><rect x="20" y="254" width="840" height="36" rx="8" class="f-none s-line dash"/><text x="36" y="277" class="t-small">Mũi tên liền: "implement". Mũi tên đứt: "chỉ phụ thuộc vào". Hộp khác màu (TieredFee) nằm ngoài thư viện.</text></svg>
</div>
<figcaption>Hình 1. Chính sách phí cắm vào trait; hàm dựng bút toán chỉ nhìn thấy trait.</figcaption>
</figure>

## Generic và trait bound: PercentageFee ghép chính sách bằng kiểu

`PercentageFee` cần một cách làm tròn. Thay vì viết cứng, nó nhận tham số kiểu `R` ràng buộc bởi `RoundingPolicy`, và `impl<R: RoundingPolicy> FeePolicy for PercentageFee<R>` chỉ viết một lần cho mọi `R`:

```rust
// src/policy.rs (rút gọn: bỏ derive và chú thích)
pub struct PercentageFee<R: RoundingPolicy> {
    rate: Decimal,
    rounding: R,
}

impl<R: RoundingPolicy> FeePolicy for PercentageFee<R> {
    fn name(&self) -> String {
        let percent = (self.rate * Decimal::from(100)).normalize();
        format!("{percent}% · {}", self.rounding.name())
    }

    fn fee(&self, fare: Money) -> Result<Money, LedgerError> {
        self.rounding.round_ratio(fare, self.rate)
    }
}
```

Chính sách mặc định của Tới Nơi dựng bằng `PercentageFee::new(Decimal::new(20, 2), HalfAwayFromZero)`. Hàm `default_policy()` trả về nó dưới dạng `impl FeePolicy`: người gọi chỉ biết "có một `FeePolicy`", không cần gõ ra tên `PercentageFee<HalfAwayFromZero>`. `impl Trait` ở vị trí tham số là cú pháp gọn của hàm generic: `fn summarize(policy: &impl FeePolicy, …)` tương đương `fn summarize<P: FeePolicy>(policy: &P, …)`.

Nếu quên trait bound, lỗi nằm ngay chỗ dùng. Thử viết `fn total_fee<T>(policy: &T, fare: Money) -> … { policy.fee(fare) }` cho ra:

```text
error[E0599]: no method named `fee` found for reference `&T` in the current scope
  --> src/capture.rs:84:12
   |
84 |     policy.fee(fare)
   |            ^^^ method not found in `&T`
   |
   = help: items from traits can only be used if the type parameter is bounded by the trait
help: the following trait defines an item `fee`, perhaps you need to restrict type parameter `T` with it:
   |
83 | pub fn total_fee<T: FeePolicy>(policy: &T, fare: Money) -> Result<Money, LedgerError> {
   |                   +++++++++++
```

Hàm generic Rust được kiểm ngay lúc định nghĩa: chỉ phương thức có trong trait bound mới gọi được, không cần chờ tới lúc có kiểu cụ thể.

## Thêm chính sách phí thứ hai mà không sửa code tính bút toán

Đây là hàm dựng bút toán thu cước. Nó chỉ biết trait `FeePolicy`:

```rust
// src/capture.rs (rút gọn: bỏ chú thích tài liệu)
pub fn capture_entry<F>(trip: &TripCharge, policy: &F) -> Result<JournalEntry, LedgerError>
where
    F: FeePolicy + ?Sized,
{
    let split = policy.split(trip.fare)?;
    let platform = AccountId::new("platform:revenue")?;
    let charge = trip.fare.checked_neg().ok_or(LedgerError::Overflow)?;
    JournalEntry::try_new(
        &format!("trip:{}:capture", trip.trip_id),
        vec![
            Posting::new(trip.customer.clone(), charge),
            Posting::new(platform, split.platform_fee),
            Posting::new(trip.driver.clone(), split.driver_net),
        ],
    )
}
```

Bút toán vẫn đi qua `JournalEntry::try_new` của bài 10, nên dù chính sách trả gì, tổng posting phải bằng 0 mới ghi được. Giờ thêm chính sách thứ hai, phí cố định theo bậc cước. Nó nằm trong `src/main.rs`, tức ngoài thư viện, và `capture_entry` không đổi dòng nào:

```rust
// src/main.rs
struct TieredFee;

impl FeePolicy for TieredFee {
    fn name(&self) -> String {
        "cố định theo bậc".to_string()
    }

    fn fee(&self, fare: Money) -> Result<Money, LedgerError> {
        let dong = match fare.dong() {
            ..20_000 => 3_000,
            20_000..50_000 => 4_500,
            _ => 8_000,
        };
        Ok(Money::from_dong(dong))
    }
}
```

Chạy cùng chuyến 25.500 đồng qua năm chính sách (hai phần trăm 5,5% chỉ khác cách làm tròn, vì 25.500 × 0,055 = 1.402,5):

```text
  20% · nửa ra xa 0      phí  5.100 VND  tài xế  20.400 VND
  5.5% · nửa ra xa 0     phí  1.403 VND  tài xế  24.097 VND
  5.5% · nửa về số chẵn  phí  1.402 VND  tài xế  24.098 VND
  cố định 3.000 VND      phí  3.000 VND  tài xế  22.500 VND
  cố định theo bậc       phí  4.500 VND  tài xế  21.000 VND
```

Trong mọi dòng, phí cộng phần tài xế đúng bằng 25.500. Một chính sách sai hợp đồng thì sao? Test trong `tests/capture.rs` dựng `GreedyFee` trả phí 25.501 cho cước 25.500; `capture_entry` trả `Err(InvalidFee { fee, fare })` và không có gì được ghi.

## Static dispatch hay dynamic dispatch: `Box<dyn Trait>` khi nào?

Có hai cách gọi phương thức của trait. Với generic (`<F: FeePolicy>`, `&impl FeePolicy`), compiler sinh một bản hàm riêng cho từng kiểu cụ thể: monomorphization, dispatch tĩnh. Ở lab, `instantiated_for::<F>` in ra `type_name` của bản được sinh, và có hai bản khác nhau:

```text
  ledger_lab::policy::PercentageFee<ledger_lab::policy::HalfAwayFromZero>
  ledger_lab::policy::FlatFee
```

Với `&dyn FeePolicy` hoặc `Box<dyn FeePolicy>`, chỉ có một bản code, và lời gọi đi qua vtable (bảng hàm) lúc chạy: dispatch động. Con trỏ phải mang thêm địa chỉ vtable nên là "fat pointer" gồm hai word. Lab đo được trên máy 64 bit:

```text
kích thước con trỏ: &FlatFee = 8 byte, &dyn FeePolicy = 16 byte, Box<dyn FeePolicy> = 16 byte
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 2. Bên trái, hàm generic được sinh thành hai bản riêng cho PercentageFee và FlatFee. Bên phải, con trỏ dyn gồm con trỏ dữ liệu và con trỏ vtable, vtable trỏ tới các hàm của FlatFee."><defs><marker id="rust-trait-generics-chinh-sach-phi-lam-tron-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><path d="M440 20 L440 284" class="f-none s-line dash"/><text x="20" y="34" class="t-b">Static dispatch · generic</text><rect x="20" y="62" width="240" height="56" rx="8" class="f-phys-soft s-phys"/><text x="34" y="84" class="t-b">capture_entry&lt;F&gt;</text><text x="34" y="104" class="t-small">một định nghĩa, F: FeePolicy</text><rect x="20" y="160" width="190" height="56" rx="8" class="f-pk-soft s-pk"/><text x="34" y="182" class="t-b">bản 1</text><text x="34" y="202" class="t-small">F = PercentageFee&lt;…&gt;</text><rect x="226" y="160" width="190" height="56" rx="8" class="f-pk-soft s-pk"/><text x="240" y="182" class="t-b">bản 2</text><text x="240" y="202" class="t-small">F = FlatFee</text><path d="M110 118 L110 158" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-trait-generics-chinh-sach-phi-lam-tron-1-a)"/><path d="M200 118 L300 158" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-trait-generics-chinh-sach-phi-lam-tron-1-a)"/><text x="20" y="246" class="t-small">Gọi thẳng vào bản đúng kiểu, compiler có thể inline.</text><text x="20" y="266" class="t-small">Mỗi bản là code riêng nên binary lớn hơn.</text><text x="470" y="34" class="t-b">Dynamic dispatch · dyn</text><text x="470" y="58" class="t-small">&amp;dyn FeePolicy · 16 byte</text><rect x="470" y="68" width="90" height="44" rx="6" class="f-warn-soft s-warn"/><text x="482" y="86" class="t-b">data</text><text x="482" y="104" class="t-small">8 byte</text><rect x="560" y="68" width="90" height="44" rx="6" class="f-pk-soft s-pk"/><text x="572" y="86" class="t-b">vtable</text><text x="572" y="104" class="t-small">8 byte</text><rect x="470" y="170" width="110" height="44" rx="8" class="f-surface s-phys"/><text x="482" y="188" class="t-b">FlatFee</text><text x="482" y="206" class="t-small">{ amount }</text><rect x="600" y="150" width="260" height="120" rx="8" class="f-pk-soft s-pk"/><text x="614" y="174" class="t-b">vtable của FlatFee</text><text x="614" y="196" class="t-small">drop · size · align</text><text x="614" y="216" class="t-small">name → FlatFee::name</text><text x="614" y="236" class="t-small">fee → FlatFee::fee</text><text x="614" y="256" class="t-small">split → bản mặc định</text><path d="M515 112 L515 168" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-trait-generics-chinh-sach-phi-lam-tron-1-a)"/><path d="M630 112 L630 148" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-trait-generics-chinh-sach-phi-lam-tron-1-a)"/></svg>
</div>
<figcaption>Hình 2. Hai cách gọi cùng một trait: nhân bản code theo kiểu (trái) hoặc tra vtable lúc chạy (phải). Bố cục vtable là chi tiết của compiler, không được đảm bảo.</figcaption>
</figure>

Dispatch tĩnh bị giới hạn ở một điểm: mỗi chỗ gọi biết đúng một kiểu. `policy_catalog()` cần một `Vec` chứa chính sách khác kiểu nhau, nên phải dùng `Vec<Box<dyn FeePolicy>>`:

```rust
// src/main.rs
fn policy_catalog() -> Vec<Box<dyn FeePolicy>> {
    vec![
        Box::new(default_policy()),
        Box::new(PercentageFee::new(Decimal::new(55, 3), HalfAwayFromZero)),
        Box::new(PercentageFee::new(Decimal::new(55, 3), NearestEven)),
        Box::new(FlatFee::new(Money::from_dong(3_000))),
        Box::new(TieredFee),
    ]
}
```

Để `capture_entry` nhận được `&*policy` (mượn `dyn FeePolicy` trong `Box`), nó phải có `?Sized`. Bỏ `?Sized` đi, lab không biên dịch (`cargo test`, rút gọn):

```text
error[E0277]: the size for values of type `dyn policy::FeePolicy` cannot be known at compilation time
  --> src/capture.rs:77:44
   |
77 |         let entry = capture_entry(&trip(), &*boxed)?;
   |                     -------------          ^^^^^^^ doesn't have a size known at compile-time
   |
   = help: the trait `Sized` is not implemented for `dyn policy::FeePolicy`
help: consider relaxing the implicit `Sized` restriction
   |
30 |     F: FeePolicy + ?Sized,
   |                  ++++++++
```

Không phải trait nào cũng dùng được với `dyn`. Trait có phương thức generic, như `fn convert<T: Into<i64>>(&self, x: T)`, không thể lập vtable (vì mỗi `T` cần một bản hàm). Rustc báo E0038 (rút gọn):

```text
error[E0038]: the trait `Convert` is not dyn compatible
1 | trait Convert { fn convert<T: Into<i64>>(&self, x: T) -> i64; }
  |       -------      ^^^^^^^ ...because method `convert` has generic type parameters
  |       |
  |       this trait is not dyn compatible...
```

`FeePolicy` không có phương thức generic nên dùng được.

Cách chọn: ưu tiên generic hoặc `impl Trait` khi kiểu cụ thể biết lúc biên dịch (mặc định của lab: một chính sách cho một service). Dùng `Box<dyn Trait>` khi cần một tập hợp chứa nhiều kiểu, khi chính sách chọn lúc chạy từ cấu hình, hoặc khi muốn tránh nhân bản code. Chi phí của dispatch động là một lần gọi gián tiếp và thường không inline được; với tính phí mỗi chuyến, chi phí này không đáng kể.

## Chạy ví dụ và kiểm chứng

Từ gốc repository:

```bash
cargo run --manifest-path examples/toi-noi-rust/11-trait-generics-phi/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/11-trait-generics-phi/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/11-trait-generics-phi/Cargo.toml
```

Lệnh đầu in ba phần: ghi bút toán bằng chính sách mặc định, bảng năm chính sách (đã trích ở trên), rồi đo dispatch. Phần đầu:

```text
== Chính sách mặc định: 20% · nửa ra xa 0
JE-000001 trip:TN-0001:capture (3 posting, replayed = false)
  customer:CU-001    -25.500 VND
  platform:revenue     5.100 VND
  driver:DR-001       20.400 VND
Tổng toàn sổ: 0 VND
```

`-- --verify` kiểm các số đó, kể cả phí theo bậc 4.500 và kích thước con trỏ, rồi in `VERIFY OK` (exit code 0). `cargo test` chạy 26 unit test, 3 integration test và 2 doc test; clippy với `-D warnings` và `cargo fmt --check` sạch. Lưu ý: `type_name` chỉ để chẩn đoán, định dạng của nó không được đảm bảo ổn định giữa các phiên bản compiler.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Chính sách phí trong lab là hàm thuần của cước: chưa có phí theo thành phố, theo giờ cao điểm, khuyến mãi hay thuế (phần P05 mới mở rộng). Tỷ lệ và bậc được viết cứng trong code, không đọc từ cấu hình hay database, và chưa có phiên bản chính sách theo thời gian: khi đổi phí, bút toán cũ phải còn tra được chính sách đã dùng. Hàm <code>split</code> chỉ kiểm 0 &lt;= phí &lt;= cước; hợp đồng đầy đủ của một chính sách thật (đơn điệu, không đổi giữa hai lần gọi cùng đầu vào) chưa được kiểm.</p>
</div>

**Bài tập:** thêm `CappedPercentageFee<R>`: phí theo tỷ lệ nhưng không quá 5.000 đồng. Gợi ý: dùng lại `PercentageFee<R>` thay vì viết lại phép nhân. Với cước 25.500 và tỷ lệ 20% thì phí và phần tài xế là bao nhiêu? `capture_entry` có phải sửa không?

<details><summary>Xem lời giải</summary><p>Bọc một <code>PercentageFee&lt;R&gt;</code> và một <code>cap: Money</code>, rồi implement <code>FeePolicy</code>: <code>fee</code> gọi <code>self.inner.fee(fare)?</code> và trả <code>.min(self.cap)</code> (<code>Money</code> có <code>Ord</code> nên <code>min</code> dùng được, như <code>FlatFee</code> đã làm với cước). Cước 25.500 × 20% = 5.100 vượt mức trần, nên phí là 5.000 và tài xế nhận 20.500. <code>capture_entry</code> không phải sửa, chỉ thêm kiểu mới và một khối <code>impl</code> (gồm <code>name</code> và <code>fee</code>). Phương thức <code>split</code> tự kiểm hợp đồng, nên không cần viết lại bước kiểm.</p></details>

Bài trước: [bài 10](/articles/rust-module-crate-cargo-test-clippy). Bài tiếp: [bài 12](/articles/rust-newtype-value-object-money-account-id).

## Tài liệu gốc

- [The Rust Book: Generic Types, Traits, and Lifetimes](https://doc.rust-lang.org/book/ch10-00-generics.html).
- [The Rust Book: Defining Shared Behavior with Traits](https://doc.rust-lang.org/book/ch10-02-traits.html).
- [The Rust Book: Using Trait Objects to Abstract over Shared Behavior](https://doc.rust-lang.org/book/ch18-02-trait-objects.html).
- [The Rust Reference: Trait objects](https://doc.rust-lang.org/reference/types/trait-object.html).
- [The Rust Reference: Traits (dyn compatibility)](https://doc.rust-lang.org/reference/items/traits.html#dyn-compatibility).
- [rust_decimal trên docs.rs: RoundingStrategy](https://docs.rs/rust_decimal/latest/rust_decimal/enum.RoundingStrategy.html).
