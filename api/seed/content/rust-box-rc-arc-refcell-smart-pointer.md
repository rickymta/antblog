---
slug: rust-box-rc-arc-refcell-smart-pointer
title: "Box, Rc, Arc, RefCell và Cell: khi nào cần smart pointer"
summary: "Bài 16: dùng Box cho kiểu đệ quy và Box<dyn FeePolicy>, Rc + Weak cho cây tài khoản, RefCell và Cell cho sửa qua tham chiếu chung, Arc<Mutex> cho nhiều luồng; chọn theo ba câu hỏi."
category: programming
level: beginner
tags: [rust, toi-noi, ledger, smart-pointer, rc, refcell]
language: vi
---

Đến bài 15, sổ cái chỉ dùng ownership thường: `Ledger` sở hữu `Vec<JournalEntry>`, hàm mượn `&` hoặc `&mut`, mỗi giá trị đúng một chủ ([bài 04](/articles/rust-ownership-move-borrow-but-toan)). Đó là mặc định và nên giữ. Ba tình huống sau không vừa khuôn đó: một danh sách chính sách phí khác kiểu, một cây tài khoản mà nút nào cũng có cha lẫn con, và nhiều luồng cùng ghi vào một sổ.

Đây là bài 16 của lộ trình **Rust cơ bản qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Lab chỉ dùng thư viện chuẩn, mô hình `Money`, `AccountId`, `JournalEntry`, `Ledger` lấy từ [bài 10](/articles/rust-module-crate-cargo-test-clippy). Mỗi smart pointer dưới đây đều có một cái giá.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p><code>RefCell&lt;Ledger&gt;</code> có ngăn được hai chỗ cùng sửa sổ cái ngay lúc biên dịch không?</p>
<details><summary>Xem đáp án</summary><p><b>Không.</b> <code>RefCell</code> dời việc kiểm tra mượn sang lúc chạy. Vi phạm là panic <code>RefCell already borrowed</code>, hoặc <code>Err</code> nếu bạn dùng <code>try_borrow_mut</code>.</p></details>
</div>

## Dữ liệu nằm ở stack hay heap, và `Box` thêm gì?

Biến cục bộ nằm trên **stack**: kích thước biết lúc biên dịch, giải phóng khi hàm trả về. **Heap** dành cho dữ liệu có kích thước chỉ biết lúc chạy hoặc phải sống lâu hơn hàm tạo ra nó. `String` và `Vec` đã dùng heap sẵn: trên stack chỉ có ba từ (địa chỉ, độ dài, sức chứa), nên `size_of::<String>()` là 24 byte trên máy 64-bit.

`Box<T>` đặt một `T` lên heap, vẫn một chủ: khi `Box` ra khỏi phạm vi, giá trị bị `drop` và heap được trả lại, tất định như ở bài 01. Trên stack chỉ còn con trỏ 8 byte. Hai lý do để dùng:

**Kiểu đệ quy.** Công thức cước là một cây: tổng hoặc số nhỏ hơn (trần cước) của hai công thức con. Viết thẳng, compiler không biết `FareExpr` chiếm bao nhiêu byte:

```text
error[E0072]: recursive type `FareExpr` has infinite size
  --> src/boxed.rs:12:1
   |
12 | pub enum FareExpr {
   | ^^^^^^^^^^^^^^^^^
...
18 |     Sum(FareExpr, FareExpr),
   |         -------- recursive without indirection
   |
help: insert some indirection (e.g., a `Box`, `Rc`, or `&`) to break the cycle
```

Thêm `Box` thì mỗi nút con chỉ là một con trỏ, kích thước cố định (`src/boxed.rs`, rút gọn):

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FareExpr {
    /// Một khoản cố định, ví dụ 12.000 VND cho 2 km đầu.
    Fixed(Money),
    /// `count` đơn vị, mỗi đơn vị `unit`, ví dụ 3 km × 4.500 VND.
    PerUnit { unit: Money, count: i64 },
    /// Tổng hai công thức con.
    Sum(Box<FareExpr>, Box<FareExpr>),
    /// Số nhỏ hơn của hai công thức con (trần cước).
    Min(Box<FareExpr>, Box<FareExpr>),
}

impl FareExpr {
    pub fn eval(&self) -> Result<Money, LedgerError> {
        match self {
            FareExpr::Fixed(m) => Ok(*m),
            FareExpr::PerUnit { unit, count } => {
                unit.checked_mul(*count).ok_or(LedgerError::Overflow)
            }
            FareExpr::Sum(a, b) => a
                .eval()?
                .checked_add(b.eval()?)
                .ok_or(LedgerError::Overflow),
            FareExpr::Min(a, b) => Ok(a.eval()?.min(b.eval()?)),
        }
    }
}
```

Chuyến `TN-0001` là `Sum(Fixed(12.000), PerUnit { unit: 4.500, count: 3 })`, cho 25.500 VND. Phép cộng và nhân vẫn là `checked_*` như bài 03.

**Struct lớn.** `RawStatement` trong lab là mảng 512 `Money` nằm thẳng trong struct, 4.096 byte. Về ngữ nghĩa, move là một phép `memcpy` các byte của giá trị: move `RawStatement` là 4.096 byte, move `Box<RawStatement>` là 8 byte và không chạm vào heap. Compiler có thể tối ưu bỏ bản sao, nhưng đó không phải cam kết của ngôn ngữ; đừng thêm `Box` vì thói quen, hãy đo trước.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Hình 1. Stack giữ giá trị nhỏ và con trỏ: Money 8 byte, Box tới RawStatement 8 byte, Box dyn FeePolicy 16 byte gồm địa chỉ dữ liệu và địa chỉ vtable. Heap giữ RawStatement 4096 byte và PercentFee. Vtable nằm ở bộ nhớ tĩnh."><defs><marker id="rust-box-rc-arc-refcell-smart-pointer-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="16" width="380" height="238" rx="12" class="f-none s-line dash"/><text x="36" y="40" class="t-b">Stack: kích thước biết lúc biên dịch</text><rect x="40" y="58" width="340" height="52" rx="8" class="f-phys-soft s-phys"/><text x="56" y="80" class="t-b">Money</text><text x="56" y="98" class="t-small">8 byte, giá trị nằm tại chỗ</text><rect x="40" y="122" width="340" height="52" rx="8" class="f-pk-soft s-pk"/><text x="56" y="144" class="t-b">Box&lt;RawStatement&gt;</text><text x="56" y="162" class="t-small">8 byte, chỉ địa chỉ</text><rect x="40" y="186" width="340" height="52" rx="8" class="f-pk-soft s-pk"/><text x="56" y="208" class="t-b">Box&lt;dyn FeePolicy&gt;</text><text x="56" y="226" class="t-small">16 byte: địa chỉ dữ liệu + địa chỉ vtable</text><rect x="480" y="16" width="380" height="238" rx="12" class="f-none s-line dash"/><text x="496" y="40" class="t-b">Heap và bộ nhớ tĩnh</text><rect x="500" y="58" width="340" height="52" rx="8" class="f-phys-soft s-phys"/><text x="516" y="80" class="t-b">RawStatement</text><text x="516" y="98" class="t-small">4.096 byte: 512 dòng Money (heap)</text><rect x="500" y="122" width="340" height="52" rx="8" class="f-phys-soft s-phys"/><text x="516" y="144" class="t-b">PercentFee { percent: 20 }</text><text x="516" y="162" class="t-small">dữ liệu của một chính sách (heap)</text><rect x="500" y="186" width="340" height="52" rx="8" class="f-warn-soft s-warn"/><text x="516" y="208" class="t-b">vtable của PercentFee</text><text x="516" y="226" class="t-small">bảng hàm name, fee; bộ nhớ tĩnh, không ở heap</text><path d="M380 148 L498 90" class="f-none s-pk" stroke-width="2" marker-end="url(#rust-box-rc-arc-refcell-smart-pointer-0-a)"/><path d="M380 202 L498 152" class="f-none s-pk" stroke-width="2" marker-end="url(#rust-box-rc-arc-refcell-smart-pointer-0-a)"/><path d="M380 224 L498 218" class="f-none s-pk dash" stroke-width="2" marker-end="url(#rust-box-rc-arc-refcell-smart-pointer-0-a)"/></svg>
</div>
<figcaption>Hình 1. <code>Box&lt;dyn Trait&gt;</code> là con trỏ béo: một địa chỉ tới dữ liệu, một địa chỉ tới vtable.</figcaption>
</figure>

Kích thước đo bằng `size_of` trên máy 64-bit chạy lab (Hình 1 lấy từ đây):

```text
  size_of::<Money>() = 8
  size_of::<Box<Money>>() = 8
  size_of::<RawStatement>() = 4096
  size_of::<Box<RawStatement>>() = 8
  size_of::<Box<dyn FeePolicy>>() = 16
  size_of::<&str>() = 16
  size_of::<String>() = 24
  size_of::<FareExpr>() = 24
  size_of::<Cell<i64>>() = 8
  size_of::<RefCell<i64>>() = 16
```

## `Box<dyn Trait>`: một `Vec` chứa các chính sách phí khác kiểu

Phí nền tảng có nhiều cách tính: 20% cước, phí cố định cho chuyến khuyến mãi, 20% nhưng không quá 4.000. Ba kiểu khác nhau, cùng một danh sách. Generics ([bài 11](/articles/rust-trait-generics-chinh-sach-phi-lam-tron)) chọn kiểu lúc biên dịch, nên `Vec<P>` chỉ chứa một `P`. `dyn FeePolicy` không có kích thước cố định, phải đứng sau con trỏ: `Box<dyn FeePolicy>` là con trỏ 16 byte ở Hình 1, và `p.fee(..)` tra vtable lúc chạy, giống gọi qua tham chiếu interface của C#.

```rust
// src/policy.rs
pub trait FeePolicy {
    fn name(&self) -> &str;
    fn fee(&self, fare: Money) -> Result<Money, LedgerError>;
}

pub fn default_policies() -> Vec<Box<dyn FeePolicy>> {
    vec![
        Box::new(PercentFee { percent: 20 }),
        Box::new(FlatFee {
            amount: Money::from_dong(3_000),
        }),
        Box::new(CappedPercentFee {
            percent: 20,
            cap: Money::from_dong(4_000),
        }),
    ]
}
```

`PercentFee`, `FlatFee` và `CappedPercentFee` mỗi kiểu `impl FeePolicy` (trait rút gọn từ bài 11). Trên cước 25.500 VND, lab in:

```text
  phần trăm            5.100 VND
  cố định              3.000 VND
  phần trăm có trần    4.000 VND
```

Hàm nhận một chính sách chỉ cần `&dyn FeePolicy`: `capture_entry(.., policies[0].as_ref())` cho bút toán `-25.500 / +5.100 / +20.400`. Giá của `Box<dyn>`: một lần cấp phát heap mỗi chính sách và một lần gọi gián tiếp. Tập chính sách đóng và biết trước thì `enum` (bài 05) cũng giải được; chọn `dyn` khi tập kiểu mở rộng được.

## `Rc` và `Weak`: cây tài khoản có cha và con

`Rc<T>` cho nhiều chủ cùng giữ một giá trị trong **một luồng**. `Rc::clone` không copy dữ liệu, chỉ tăng bộ đếm; giá trị được giải phóng khi chủ cuối buông. Cây tài khoản cần điều đó: `platform` có hai con `platform:revenue` và `platform:fee`; cha giữ con bằng `Rc`, và chỗ khác cũng muốn giữ cùng nút.

Con trỏ về cha là chỗ dễ sai. Con giữ cha bằng `Rc` thì bộ đếm hai bên không bao giờ về 0 và bộ nhớ rò. `Weak<T>` trỏ tới nút nhưng không giữ nút sống; `upgrade()` trả `Option<Rc<T>>`, `None` nếu nút đã bị giải phóng (`src/tree.rs`, rút gọn):

```rust
pub struct AccountNode {
    id: AccountId,
    parent: Weak<AccountNode>,
    children: RefCell<Vec<Rc<AccountNode>>>,
}

impl AccountNode {
    pub fn add_child(parent: &Rc<AccountNode>, id: AccountId) -> Rc<AccountNode> {
        let child = Rc::new(AccountNode {
            id,
            parent: Rc::downgrade(parent),
            children: RefCell::new(Vec::new()),
        });
        parent.children.borrow_mut().push(Rc::clone(&child));
        child
    }
}
```

`RefCell` quanh `children` là cách thêm con vào nút đã nằm trong `Rc`, giải thích ở mục sau. Dựng cây rồi đọc bộ đếm (Hình 2 giải thích từng số):

```text
  root: strong = 1, weak = 2
  revenue: strong = 2
  đường đi: platform > platform:revenue
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 2. Cây tài khoản: nút platform giữ hai nút con platform:revenue và platform:fee bằng Rc (đường liền); mỗi nút con trỏ về cha bằng Weak (đường đứt). Nút platform có strong 1 và weak 2."><defs><marker id="rust-box-rc-arc-refcell-smart-pointer-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="320" y="20" width="240" height="70" rx="10" class="f-pk-soft s-pk"/><text x="338" y="46" class="t-b">platform</text><text x="338" y="68" class="t-small">strong 1 · weak 2</text><rect x="100" y="170" width="260" height="70" rx="10" class="f-phys-soft s-phys"/><text x="118" y="196" class="t-b">platform:revenue</text><text x="118" y="218" class="t-small">strong 2: Vec cha + biến revenue</text><rect x="520" y="170" width="260" height="70" rx="10" class="f-phys-soft s-phys"/><text x="538" y="196" class="t-b">platform:fee</text><text x="538" y="218" class="t-small">strong 1: chỉ Vec của cha</text><path d="M340 90 L200 170" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-box-rc-arc-refcell-smart-pointer-1-a)"/><path d="M540 90 L680 170" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-box-rc-arc-refcell-smart-pointer-1-a)"/><path d="M300 170 L380 90" class="f-none s-warn dash" stroke-width="2" marker-end="url(#rust-box-rc-arc-refcell-smart-pointer-1-a)"/><path d="M580 170 L500 90" class="f-none s-warn dash" stroke-width="2" marker-end="url(#rust-box-rc-arc-refcell-smart-pointer-1-a)"/><path d="M40 272 L100 272" class="f-none s-phys" stroke-width="2"/><text x="112" y="276" class="t-small">Rc (mạnh): cha sở hữu con, tăng strong</text><path d="M460 272 L520 272" class="f-none s-warn dash" stroke-width="2"/><text x="532" y="276" class="t-small">Weak: con trỏ về cha, không giữ sống</text></svg>
</div>
<figcaption>Hình 2. Chủ sở hữu đi xuống (<code>Rc</code>), con trỏ ngược đi lên (<code>Weak</code>). Không có chu trình <code>Rc</code>, nên cây được giải phóng khi chủ cuối cùng buông.</figcaption>
</figure>

Lab chứng minh bằng một `Weak` quan sát. Hai nút giữ nhau bằng `Rc` mạnh: buông cả hai handle, `probe.upgrade().is_some()` vẫn `true`, nút còn sống tức là rò. Với cây dùng `Weak` cho cha, cùng phép thử trả `None` cho cả gốc lẫn con. Rò bộ nhớ không vi phạm an toàn bộ nhớ nên Rust không chặn.

## `RefCell` và `Cell`: sửa qua tham chiếu chung

`Rc<T>` chỉ cho mượn `&T`. Thử gọi một hàm `&mut self` trên `Rc<Ledger>`:

```text
error[E0596]: cannot borrow data in an `Rc` as mutable
 --> src/main.rs:8:5
  |
8 |     other.post(sample_entry().unwrap()).unwrap();
  |     ^^^^^ cannot borrow as mutable
  |
  = help: trait `DerefMut` is required to modify through a dereference, but it is not implemented for `Rc<Ledger>`
```

Đó là lý do có **interior mutability**: sửa bên trong giá trị đang chỉ có `&`. `RefCell<T>` cho `borrow()` (trả `Ref`) và `borrow_mut()` (trả `RefMut`). Luật vẫn như borrow checker: nhiều chỗ đọc, hoặc đúng một chỗ ghi; khác ở chỗ được đếm **lúc chạy**, và vi phạm là panic. Lỗi sau viết cố ý trong `src/shared.rs`:

```rust
pub type SharedLedger = Rc<RefCell<Ledger>>;

pub fn post_buggy(shared: &SharedLedger, entry: JournalEntry) -> Result<String, LedgerError> {
    let view = shared.borrow();
    let before = view.len();
    let id = shared.borrow_mut().post(entry)?.id;
    Ok(format!("{id} (trước đó có {before} bút toán)"))
}
```

Code biên dịch sạch. Chạy `cargo run -- --panic-demo` (Rust 1.98.1):

```text
thread 'main' (…) panicked at src/shared.rs:25:21:
RefCell already borrowed
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

Nguyên nhân: guard `view` có `Drop`, nên sống đến hết hàm chứ không chỉ đến lần dùng cuối. Lúc dòng 25 xin mượn ghi, một mượn đọc vẫn đang mở.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Hình 3. Dòng thời gian của hai hàm. post_buggy: guard đọc view sống đến cuối hàm nên borrow_mut ở giữa gây panic. post_fixed: guard đọc tạm hết ở dấu chấm phẩy, sau đó borrow_mut chạy bình thường."><text x="20" y="50" class="t-b">post_buggy</text><rect x="150" y="30" width="700" height="36" rx="8" class="f-phys-soft s-phys"/><text x="166" y="53" class="t-small">view = shared.borrow(): guard đọc sống đến hết hàm</text><rect x="400" y="86" width="250" height="36" rx="8" class="f-warn-soft s-warn"/><text x="416" y="109" class="t-small">borrow_mut(): panic vì đang đọc</text><path d="M400 66 L400 86" class="f-none s-warn dash" stroke-width="2"/><path d="M650 66 L650 86" class="f-none s-warn dash" stroke-width="2"/><text x="20" y="170" class="t-b">post_fixed</text><rect x="150" y="150" width="190" height="36" rx="8" class="f-phys-soft s-phys"/><text x="166" y="173" class="t-small">borrow().len()</text><text x="346" y="173" class="t-b">;</text><rect x="380" y="150" width="270" height="36" rx="8" class="f-pk-soft s-pk"/><text x="396" y="173" class="t-small">borrow_mut().post(entry)</text><text x="150" y="216" class="t-small">Guard tạm hết sống ở dấu chấm phẩy, nên mượn ghi sau đó không còn va chạm.</text></svg>
</div>
<figcaption>Hình 3. Cùng hai lời gọi, khác ở lúc guard đọc hết sống. Compiler không thấy khác biệt; <code>RefCell</code> thấy lúc chạy.</figcaption>
</figure>

Hai cách sửa. Thu hẹp guard: `let before = shared.borrow().len();` để guard tạm hết ở dấu `;` (`post_fixed`, in `JE-000001 (trước đó có 0 bút toán)`). Hoặc trả lỗi thay vì panic: `shared.try_borrow_mut().map_err(|_| LedgerError::Busy)?` (`post_try`, in `Err(Busy)` khi có `borrow()` đang mở).

**`Cell<T>`** nhẹ hơn. `get` sao chép giá trị ra (nên cần `T: Copy`), `set` ghi đè; không bao giờ phát tham chiếu vào bên trong, nên không thể vi phạm luật mượn và không cần kiểm tra lúc chạy. Hợp với bộ đếm hay cờ. Lab đếm lượt tra cứu số dư dù `balance` chỉ nhận `&self` (`src/shared.rs`, rút gọn):

```rust
pub struct MeteredLedger {
    ledger: Ledger,
    lookups: Cell<u32>,
}

impl MeteredLedger {
    pub fn balance(&self, account: &AccountId) -> Result<Money, LedgerError> {
        // `Cell<u32>`: u32 là Copy, nên `get`/`set` sao chép giá trị, không phát tham chiếu.
        self.lookups.set(self.lookups.get() + 1);
        self.ledger.balance(account)
    }
}
```

Sau hai lần gọi, `lookups()` trả 2. Cái giá của `RefCell` thấy ở kích thước: `size_of::<Cell<i64>>()` là 8, còn `RefCell<i64>` là 16 vì có thêm cờ đếm mượn. Cả hai đều **không** `Sync`: không chia sẻ được giữa các luồng.

## `Arc<Mutex<_>>`: khi các chủ nằm ở nhiều luồng

`Arc<T>` là `Rc<T>` với bộ đếm nguyên tử, an toàn giữa các luồng; đổi lại mỗi lần tăng giảm bộ đếm đắt hơn. Compiler không cho dùng nhầm `Rc` (rút gọn):

```text
error[E0277]: `Rc<std::sync::Mutex<Ledger>>` cannot be sent between threads safely
  --> src/main.rs:13:27
   |
13 |       let h = thread::spawn(move || {
   |               ------------- ^------
   ...
   = help: within `{closure@src/main.rs:13:27: 13:34}`, the trait `Send` is not implemented for `Rc<std::sync::Mutex<Ledger>>`
```

`Arc` chỉ cho `&T`, nên để ghi cần thêm `Mutex<T>`: dữ liệu nằm *bên trong* khóa, chỉ chạm được qua `lock()` trả guard, và guard nhả khóa khi hết phạm vi. Bốn chuyến, mỗi chuyến ghi từ một luồng (`src/threads.rs`, bỏ chú thích):

```rust
pub fn capture_in_threads(trips: usize) -> Result<Ledger, LedgerError> {
    let shared = Arc::new(Mutex::new(Ledger::new()));
    let mut handles = Vec::new();
    for n in 1..=trips {
        let shared = Arc::clone(&shared);
        handles.push(thread::spawn(move || -> Result<(), LedgerError> {
            let entry = trip_entry(n)?;
            let mut ledger = shared.lock().map_err(|_| LedgerError::Poisoned)?;
            ledger.post(entry)?;
            Ok(())
        }));
    }
    for handle in handles {
        handle.join().map_err(|_| LedgerError::Poisoned)??;
    }
    let mutex = Arc::try_unwrap(shared).expect("mọi luồng đã join, chỉ còn một chủ");
    mutex.into_inner().map_err(|_| LedgerError::Poisoned)
}
```

`lock()` trả `Err` nếu luồng khác từng panic khi giữ khóa (poisoning); lab đổi nó thành `LedgerError::Poisoned`. Kết quả không phụ thuộc thứ tự luồng: 4 bút toán, `platform:revenue` bằng 20.400 VND (4 × 5.100), tổng toàn sổ bằng 0; mã `JE-00000x` tùy luồng nào vào trước.

## Chọn loại nào: ba câu hỏi và đối chiếu với C#

Câu đầu tiên: có thật cần không? Mặc định vẫn là ownership thường cộng borrow. Khi không đủ, hỏi ba câu: **một chủ hay nhiều chủ; một luồng hay nhiều luồng; có cần sửa qua tham chiếu chung không**.

| Tình huống | Chọn | Giá phải trả |
| --- | --- | --- |
| Một chủ; kiểu đệ quy hoặc `dyn Trait` | `Box<T>`, `Box<dyn Trait>` | Cấp phát heap; `dyn` gọi gián tiếp |
| Nhiều chủ, một luồng | `Rc<T>` (+ `Weak` liên kết ngược) | Chu trình `Rc` bị rò |
| Nhiều chủ, nhiều luồng | `Arc<T>` | Đếm nguyên tử |
| Sửa qua `&`, một luồng | `RefCell<T>` | Kiểm tra lúc chạy, có thể panic |
| Sửa qua `&`, một luồng, kiểu `Copy` nhỏ | `Cell<T>` | Không phát tham chiếu vào trong |
| Sửa qua `&`, nhiều luồng | `Mutex<T>` (đi với `Arc`) | Luồng khác phải chờ; poisoning |

Với người từ C#, xem thêm [bài về allocation, GC và dọn tài nguyên](/articles/dotnet10-allocation-gc-disposal):

| C# / .NET | Rust |
| --- | --- |
| `class` là tham chiếu do GC quản, chia sẻ tự do, không phải nghĩ ai là chủ | Mặc định một chủ; chia sẻ phải ghi ra kiểu (`Rc`, `Arc`) |
| GC truy vết, dọn được cả chu trình | `Rc` chỉ đếm, chu trình bị rò; `Weak<T>` gần `WeakReference<T>` |
| `lock (obj)`: khóa tách khỏi dữ liệu | `Mutex<T>`: dữ liệu nằm trong khóa |
| `IDisposable` + `using`: người gọi phải viết `using`; bộ nhớ thường do GC dọn | `Drop` tự chạy khi chủ cuối buông; chu trình `Rc` thì không chạy |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p><code>Rc&lt;RefCell&lt;Ledger&gt;&gt;</code> và bốn luồng ghi cùng một <code>Mutex</code> là minh họa cơ chế, không phải thiết kế production. Service thật nên cho đúng một nơi sở hữu <code>Ledger</code> và truyền <code>&amp;mut Ledger</code> hoặc nhận lệnh qua hàng đợi. Phí ở đây là phần trăm nguyên, làm tròn nửa lên, chưa dùng <code>rust_decimal</code> như bài 03; sổ cái chỉ nằm trong bộ nhớ.</p>
</div>

## Chạy ví dụ và kiểm chứng

Từ gốc repository:

```bash
cargo run --manifest-path examples/toi-noi-rust/16-box-rc-arc-refcell/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/16-box-rc-arc-refcell/Cargo.toml -- --verify
cargo run --manifest-path examples/toi-noi-rust/16-box-rc-arc-refcell/Cargo.toml -- --panic-demo
cargo test --manifest-path examples/toi-noi-rust/16-box-rc-arc-refcell/Cargo.toml
```

Lệnh đầu in từng nhóm ví dụ. Phần chưa trích ở trên:

```text
  cước TN-0001 = 25.500 VND (cây sâu 2)
  cước có trần 20.000 = 20.000 VND
  chu trình Rc mạnh bị rò: true; cây có Weak được giải phóng: true
  post_fixed: JE-000001 (trước đó có 0 bút toán)
  post_try khi đang có người đọc: Err(Busy)
  post_buggy panic: RefCell already borrowed
  số dư driver:DR-001 = 20.400 VND, số lần tra cứu = 2
  số bút toán = 4
  platform:revenue = 20.400 VND
  tổng toàn sổ = 0 VND
```

`-- --verify` kiểm các giá trị trên và bắt panic thật của `post_buggy` bằng `catch_unwind`: đúng thì in `VERIFY OK`, thoát 0; sai thì thoát 1. Bước này chỉ so phần `already borrowed`, vì thông điệp có thể đổi theo phiên bản Rust. `-- --panic-demo` để panic không bắt, thoát mã 101. `cargo test` chạy 33 unit test và 2 doc test.

**Bài tập:** (1) Trong `post_buggy`, thêm `drop(view);` ngay sau dòng `let before = view.len();`. Hàm còn panic không? (2) Đổi `Arc<Mutex<Ledger>>` thành `Arc<RefCell<Ledger>>` rồi đưa vào `thread::spawn`. Compiler nói gì, và vì sao?

<details><summary>Xem lời giải</summary><p>(1) Không còn panic: <code>drop(view)</code> nhả guard đọc trước khi <code>borrow_mut()</code>; hàm trả <code>JE-000001</code> với <code>before = 0</code>. (2) Lỗi E0277: <code>RefCell&lt;Ledger&gt;</code> cannot be shared between threads safely. <code>RefCell</code> không cài <code>Sync</code> vì đếm mượn không nguyên tử, và <code>Arc&lt;T&gt;</code> chỉ <code>Send</code> khi <code>T</code> là <code>Send + Sync</code>. Compiler gợi ý <code>RwLock</code>; <code>Mutex</code> cũng đúng cho việc này.</p></details>

Bài trước: [bài 15](/articles/rust-closure-iterator-adapter-doi-soat-bo-nho). Bài tiếp: [bài 17](/articles/rust-thiserror-anyhow-xu-ly-loi-thuc-te).

## Tài liệu gốc

- [The Rust Book: Smart Pointers](https://doc.rust-lang.org/book/ch15-00-smart-pointers.html).
- [The Rust Book: RefCell\<T\> and the Interior Mutability Pattern](https://doc.rust-lang.org/book/ch15-05-interior-mutability.html).
- [The Rust Book: Shared-State Concurrency](https://doc.rust-lang.org/book/ch16-03-shared-state.html).
- [std::rc](https://doc.rust-lang.org/std/rc/index.html) và [std::cell](https://doc.rust-lang.org/std/cell/index.html).
- [std::sync::Arc](https://doc.rust-lang.org/std/sync/struct.Arc.html) và [std::sync::Mutex](https://doc.rust-lang.org/std/sync/struct.Mutex.html).
- [The Rust Reference: Trait objects](https://doc.rust-lang.org/reference/types/trait-object.html).
