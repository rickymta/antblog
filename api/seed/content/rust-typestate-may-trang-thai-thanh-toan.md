---
slug: rust-typestate-may-trang-thai-thanh-toan
title: "Typestate và máy trạng thái: hold → capture → settle không thể đi sai"
summary: "Bài 14: mô hình hóa sáu trạng thái thanh toán bằng enum và match trả Result, rồi bằng typestate Payment<S> với PhantomData để capture sai thứ tự không biên dịch được; cách nối với database và khi nào enum là đủ."
category: rust-beginner
level: beginner
tags: [rust, toi-noi, ledger, typestate, state-machine]
language: vi
---

Chuyến `TN-0001` kết thúc, VíXanh đã giữ 25.500 VND trên phương tiện thanh toán của khách (`AUTHORIZED`). Một handler gọi `capture` hai lần vì Kafka giao lại thông điệp. Một handler khác gọi `capture` cho khoản vừa tạo, chưa hề giữ tiền. Ở sổ cái, bút toán thu cước ghi nhầm không xóa được, chỉ sửa bằng bút toán đảo, nên ta muốn những lệnh gọi sai thứ tự bị chặn càng sớm càng tốt, tốt nhất là ngay khi biên dịch.

Đây là bài 14 của lộ trình **Rust cơ bản qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Bài dựng trên lab của [bài 10](/articles/rust-module-crate-cargo-test-clippy); sáu trạng thái đã xuất hiện ở [bài 05](/articles/rust-struct-enum-match-tai-khoan-but-toan), bút toán cân ở [bài 13](/articles/rust-double-entry-journal-entry-posting).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Với typestate, gọi <code>capture()</code> trên một khoản thanh toán chưa <code>authorize()</code> sẽ báo lỗi lúc chạy hay lúc biên dịch?</p>
<details><summary>Xem đáp án</summary><p><b>Lúc biên dịch.</b> <code>Payment&lt;Initiated&gt;</code> đơn giản là không có method <code>capture</code>, nên rustc báo E0599 trước khi có file thực thi. Với <code>enum</code>, cùng lỗi đó chỉ lộ ra khi chạy, dưới dạng một giá trị <code>Err</code>.</p></details>
</div>

## Enum PaymentState: bảng chuyển trạng thái nằm một chỗ

Cách quen thuộc nhất là một `enum` cho trạng thái, một `enum` cho sự kiện và một hàm quyết định cặp nào hợp lệ. Sáu trạng thái theo §9.1 kiến trúc Tới Nơi, năm sự kiện, 30 cặp ghép; chỉ 6 cặp có cạnh. Khác `Option` ở bài 05, hàm trả `Result` để lỗi nói rõ *vì sao* bị từ chối:

```rust
// src/state.rs
pub fn transition(from: PaymentState, event: PaymentEvent) -> Result<PaymentState, PaymentError> {
    use PaymentEvent::*;
    use PaymentState::*;
    match (from, event) {
        (Initiated, Authorize) => Ok(Authorized),
        (Authorized, Capture) => Ok(Captured),
        (Initiated | Authorized, Fail) => Ok(Failed),
        (Captured, RequestRefund) => Ok(RefundPending),
        (RefundPending, CompleteRefund) => Ok(Refunded),
        (from, event) => Err(PaymentError::InvalidTransition { from, event }),
    }
}
```

Nhánh cuối nuốt 24 cặp còn lại. Thêm trạng thái mới thì nhánh này vẫn biên dịch, giống điểm đánh đổi đã nêu ở bài 05. Bản ghi thanh toán giữ trạng thái trong trường private và chỉ đổi qua `apply`, nơi gắn việc ghi sổ vào đúng sự kiện:

```rust
// src/record.rs
pub fn apply(
    &mut self,
    event: PaymentEvent,
    ledger: &mut Ledger,
) -> Result<Option<String>, PaymentError> {
    let next = transition(self.state, event)?;
    // Nhớ ghi sổ cho đúng sự kiện là việc của người viết nhánh này; compiler không nhắc.
    let entry_id = match event {
        PaymentEvent::Capture => Some(ledger.post(capture_entry(&self.terms)?)?.id),
        PaymentEvent::CompleteRefund => Some(ledger.post(refund_entry(&self.terms)?)?.id),
        _ => None,
    };
    self.state = next;
    Ok(entry_id)
}
```

Ghi sổ trước, đổi trạng thái sau: nếu `post` lỗi thì `?` thoát và `self.state` giữ nguyên. Thứ tự này thuộc về kỷ luật của người viết code. `capture_entry` trả bút toán ba dòng của bài 10: khách -25.500, nền tảng +5.100, tài xế +20.400, khóa `trip:TN-0001:capture` (lab gộp một bước cho gọn; bài 13 tách qua clearing). `refund_entry` đổi dấu cả ba dòng, khóa `trip:TN-0001:refund`.

Bảng này dễ test: lab duyệt đủ 30 cặp, đếm đúng 6 hợp lệ và 24 bị từ chối. Điểm yếu là lỗi chỉ lộ khi chạy: `record.apply(PaymentEvent::Capture, …)` trên bản ghi `INITIATED` vẫn biên dịch.

## Typestate: đưa trạng thái lên thành tham số kiểu

Typestate đưa trạng thái vào *kiểu*: thay vì một `Payment` có trường `state`, ta có `Payment<Initiated>`, `Payment<Authorized>`, `Payment<Captured>`, mỗi cái là một kiểu riêng. Trạng thái là struct rỗng (marker); `Payment<S>` giữ dữ liệu chung và một `PhantomData<S>`:

```rust
// src/typestate.rs (rút gọn: bỏ chú thích; macro marker! viết ở dưới)
pub trait State: sealed::Sealed {
    const KIND: PaymentState;
}

#[derive(Debug)]
pub struct Payment<S> {
    terms: PaymentTerms,
    _state: PhantomData<S>,
}
```

`PhantomData<S>` cho compiler biết struct "gắn với" `S` dù không chứa giá trị `S` nào; không có nó, Rust báo tham số `S` không được dùng. Nó không chiếm byte nào: `size_of::<Payment<Initiated>>()` và `size_of::<Payment<Captured>>()` cùng bằng 88 trên máy 64-bit (ba `String` 24 byte, hai `Money` 8 byte), đúng bằng kích thước `PaymentTerms`. Con số 88 phụ thuộc nền tảng, nên `--verify` chỉ kiểm hai kích thước bằng nhau. Marker sinh bằng macro cho gọn, mỗi marker khai báo `KIND` là giá trị `enum` tương ứng:

```rust
// src/typestate.rs
macro_rules! marker {
    ($name:ident, $kind:expr) => {
        #[derive(Debug)]
        pub struct $name;
        impl sealed::Sealed for $name {}
        impl State for $name {
            const KIND: PaymentState = $kind;
        }
    };
}

marker!(Initiated, PaymentState::Initiated);
marker!(Authorized, PaymentState::Authorized);
marker!(Captured, PaymentState::Captured);
marker!(Failed, PaymentState::Failed);
marker!(RefundPending, PaymentState::RefundPending);
marker!(Refunded, PaymentState::Refunded);
```

Trait `State` được "niêm phong" bằng `sealed::Sealed` (trait private): crate khác không thêm được trạng thái thứ bảy.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 1. Typestate của thanh toán: Payment Initiated gọi authorize để thành Authorized, capture để thành Captured và ghi bút toán, refund để thành RefundPending, settle_refund để thành Refunded và ghi bút toán đảo. fail đưa Initiated hoặc Authorized sang Failed. Method chỉ tồn tại ở trạng thái nguồn."><defs><marker id="rust-typestate-may-trang-thai-thanh-toan-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker><marker id="rust-typestate-may-trang-thai-thanh-toan-0-b" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs><rect x="12" y="80" width="140" height="56" rx="10" class="f-phys-soft s-phys"/><text x="24" y="104" class="t-b">Initiated</text><text x="24" y="124" class="t-small">vừa tạo</text><rect x="191" y="80" width="140" height="56" rx="10" class="f-phys-soft s-phys"/><text x="203" y="104" class="t-b">Authorized</text><text x="203" y="124" class="t-small">đã giữ tiền</text><rect x="370" y="80" width="140" height="56" rx="10" class="f-phys-soft s-phys"/><text x="382" y="104" class="t-b">Captured</text><text x="382" y="124" class="t-small">đã thu tiền</text><rect x="549" y="80" width="140" height="56" rx="10" class="f-phys-soft s-phys"/><text x="561" y="104" class="t-b">RefundPending</text><text x="561" y="124" class="t-small">chờ VíXanh hoàn</text><rect x="728" y="80" width="140" height="56" rx="10" class="f-warn-soft s-warn"/><text x="740" y="104" class="t-b">Refunded</text><text x="740" y="124" class="t-small">trạng thái cuối</text><path d="M82 80 Q170 20 259 80" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-typestate-may-trang-thai-thanh-toan-0-a)"/><text x="170" y="38" text-anchor="middle" class="t-mono">authorize()</text><path d="M261 80 Q349 20 438 80" class="f-none s-pk" stroke-width="2" marker-end="url(#rust-typestate-may-trang-thai-thanh-toan-0-b)"/><text x="349" y="38" text-anchor="middle" class="t-mono">capture()</text><path d="M440 80 Q528 20 617 80" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-typestate-may-trang-thai-thanh-toan-0-a)"/><text x="528" y="38" text-anchor="middle" class="t-mono">refund()</text><path d="M619 80 Q707 20 796 80" class="f-none s-pk" stroke-width="2" marker-end="url(#rust-typestate-may-trang-thai-thanh-toan-0-b)"/><text x="707" y="38" text-anchor="middle" class="t-mono">settle_refund()</text><rect x="106" y="196" width="130" height="56" rx="10" class="f-warn-soft s-warn"/><text x="118" y="220" class="t-b">Failed</text><text x="118" y="240" class="t-small">trạng thái cuối</text><path d="M82 136 L150 194" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-typestate-may-trang-thai-thanh-toan-0-a)"/><path d="M261 136 L192 194" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-typestate-may-trang-thai-thanh-toan-0-a)"/><text x="100" y="172" text-anchor="end" class="t-mono">fail()</text><text x="242" y="172" class="t-mono">fail()</text><rect x="330" y="176" width="538" height="96" rx="10" class="f-none s-line dash"/><text x="346" y="202" class="t-b">Mũi tên là method, nhận self theo giá trị</text><text x="346" y="224" class="t-small">Mỗi method chỉ có ở kiểu nguồn; gọi ở kiểu khác là E0599.</text><text x="346" y="244" class="t-small">Mũi tên màu nhấn: capture() và settle_refund() ghi bút toán vào sổ cái.</text><text x="346" y="262" class="t-small">Không đi ngược được: giá trị cũ đã bị move, không dùng lại.</text></svg>
</div>
<figcaption>Hình 1. Năm trạng thái chính và <code>Failed</code>. Mỗi mũi tên là một method; <b>Captured</b> không có <code>fail()</code> và <b>Initiated</b> không có <code>capture()</code> vì các method đó không tồn tại ở kiểu ấy.</figcaption>
</figure>

## Mỗi method chỉ tồn tại ở trạng thái hợp lệ

Điểm mấu chốt là các khối `impl` theo từng kiểu cụ thể: `authorize` nằm trong `impl Payment<Initiated>`, `capture` nằm trong `impl Payment<Authorized>`, nên chỉ kiểu đó mới gọi được. Mọi method đều nhận `self` theo giá trị (bài 04): sau `authorize()`, biến `Payment<Initiated>` cũ đã được chuyển đi và không dùng lại được. Hàm chuyển nội bộ `into_state` chỉ dựng lại struct với `PhantomData` kiểu khác:

```rust
// src/typestate.rs (rút gọn: bỏ chú thích)
impl Payment<Authorized> {
    pub fn capture(self, ledger: &mut Ledger) -> Result<(Payment<Captured>, String), PaymentError> {
        let entry_id = ledger.post(capture_entry(&self.terms)?)?.id;
        Ok((self.into_state(), entry_id))
    }
}

impl Payment<Captured> {
    pub fn refund(self) -> Payment<RefundPending> {
        self.into_state()
    }
}
```

Ở đây `capture` *tạo bút toán*: nó chỉ trả `Payment<Captured>` khi `ledger.post` thành công, kèm mã `JE-000001`. Khác với `apply`, người gọi không có chỗ nào để quên ghi sổ hay đổi trạng thái trước khi ghi. `settle_refund` làm tương tự với bút toán hoàn tiền; `refund()` chỉ đổi kiểu, vì khách mới yêu cầu còn VíXanh chưa hoàn.

Hủy giao dịch (`fail`) hợp lệ từ hai trạng thái. Thay vì viết hai `impl`, ta dùng trait bound với một trait đánh dấu:

```rust
// src/typestate.rs (rút gọn: ghép các đoạn)
pub trait Cancellable: State {}

impl Cancellable for Initiated {}
impl Cancellable for Authorized {}

impl<S: Cancellable> Payment<S> {
    pub fn fail(self) -> Payment<Failed> {
        self.into_state()
    }
}
```

`Payment<Captured>::fail()` không tồn tại, đúng với bảng chuyển ở phần trước: `(Captured, Fail)` không có cạnh.

## Lỗi compile thật: `capture` trên `Payment<Initiated>`

Đây là chương trình sai thứ tự, thử biên dịch trong một file nháp:

```rust
let payment = Payment::new(terms);
let _ = payment.capture(&mut ledger);
```

Thông điệp thật của rustc 1.98.1:

```text
error[E0599]: no method named `capture` found for struct `Payment<ledger_lab::Initiated>` in the current scope
  --> examples/err1.rs:14:21
   |
14 |     let _ = payment.capture(&mut ledger);
   |                     ^^^^^^^ method not found in `Payment<ledger_lab::Initiated>`
   |
   = note: the method was found for `Payment<ledger_lab::Authorized>`
```

Rustc còn chỉ ra kiểu có method đó, nên bước bị thiếu là `authorize()`. Thu hai lần trong cùng một đoạn code cũng bị chặn, nhờ `capture(self, …)` tiêu thụ giá trị (rút gọn, bỏ phần note):

```text
error[E0382]: use of moved value: `payment`
   --> examples/err2.rs:15:19
    |
 13 |     let payment = Payment::new(terms).authorize();
    |         ------- move occurs because `payment` has type `Payment<ledger_lab::Authorized>`, which does not implement the `Copy` trait
 14 |     let _first = payment.capture(&mut ledger);
    |                          -------------------- `payment` moved due to this method call
 15 |     let _second = payment.capture(&mut ledger);
    |                   ^^^^^^^ value used here after move
```

Hai đoạn này nằm trong lab dưới dạng doc test ```` ```compile_fail,E0599 ```` và ```` ```compile_fail,E0382 ````: nếu code đổi khiến chúng biên dịch được, `cargo test` báo đỏ. Nhưng như bài 10 đã lưu ý, doc test chỉ chứng minh "không biên dịch được"; trên stable, rustdoc không đối chiếu mã lỗi. Đổi tạm E0599 thành E0004, test vẫn pass. Thông điệp ở trên đến từ biên dịch file nháp, không phải từ doc test.

Chuỗi `Payment::new(terms).authorize().capture(&mut ledger)` biên dịch được vì kiểu biết trước ở từng bước. Nếu trạng thái chỉ biết lúc chạy (đọc từ database), ta cần bước tiếp theo.

## Typestate không lưu xuống database được: enum ở biên

Tham số kiểu `S` chỉ tồn tại lúc biên dịch. Cột `state` trong PostgreSQL hay JSON trong thông điệp Kafka chỉ chứa chuỗi `"CAPTURED"`. Vì vậy `enum PaymentState` vẫn cần ở *biên* hệ thống, còn typestate nằm *bên trong* logic đã biết trạng thái.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Hình 2. Ba biểu diễn của một khoản thanh toán: dòng database chỉ có chuỗi trạng thái, PaymentRecord giữ giá trị enum PaymentState lúc chạy, Payment S và AnyPayment giữ trạng thái trong kiểu. Đi lên bằng FromStr, From hoặc TryFrom, đi xuống bằng to_record; chuỗi lạ và nạp sai trạng thái là lỗi lúc chạy ở biên."><defs><marker id="rust-typestate-may-trang-thai-thanh-toan-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="20" width="210" height="130" rx="10" class="f-none s-muted dash"/><text x="34" y="48" class="t-b">Database, Kafka</text><text x="34" y="72" class="t-mono">state = 'CAPTURED'</text><text x="34" y="96" class="t-small">chỉ là chuỗi văn bản</text><text x="34" y="116" class="t-small">đọc được bởi mọi ngôn ngữ</text><rect x="335" y="20" width="210" height="130" rx="10" class="f-phys-soft s-phys"/><text x="349" y="48" class="t-b">PaymentRecord</text><text x="349" y="72" class="t-mono">state: PaymentState</text><text x="349" y="96" class="t-small">giá trị enum lúc chạy</text><text x="349" y="116" class="t-small">đổi qua apply()</text><rect x="650" y="20" width="210" height="130" rx="10" class="f-pk-soft s-pk"/><text x="664" y="48" class="t-b">Payment&lt;S&gt;</text><text x="664" y="72" class="t-mono">PhantomData&lt;S&gt;</text><text x="664" y="96" class="t-small">trạng thái nằm trong kiểu</text><text x="664" y="116" class="t-small">đổi qua method</text><text x="664" y="136" class="t-small">AnyPayment khi chưa biết S</text><path d="M232 62 L333 62" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-typestate-may-trang-thai-thanh-toan-1-a)"/><text x="283" y="54" text-anchor="middle" class="t-small">FromStr</text><path d="M333 112 L232 112" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-typestate-may-trang-thai-thanh-toan-1-a)"/><text x="283" y="130" text-anchor="middle" class="t-small">as_str()</text><path d="M547 62 L648 62" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-typestate-may-trang-thai-thanh-toan-1-a)"/><text x="597" y="54" text-anchor="middle" class="t-small">TryFrom · From</text><path d="M648 112 L547 112" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-typestate-may-trang-thai-thanh-toan-1-a)"/><text x="597" y="130" text-anchor="middle" class="t-small">to_record()</text><rect x="20" y="176" width="840" height="56" rx="8" class="f-warn-soft s-warn"/><text x="36" y="200" class="t-b">Lỗi ở biên là lỗi lúc chạy</text><text x="36" y="220" class="t-small">Chuỗi lạ: UnknownState. Nạp CAPTURED thành Payment&lt;Authorized&gt;: WrongState. Sau biên, kiểu giữ thứ tự gọi.</text></svg>
</div>
<figcaption>Hình 2. Hai chiều qua biên. Đi xuống (mũi tên trái) luôn thành công. Đi lên (mũi tên phải) trả <code>Result</code>, trừ <code>AnyPayment::from</code>: trạng thái nào cũng có biến thể tương ứng.</figcaption>
</figure>

Chiều xuống đơn giản: `to_record()` đọc `S::KIND`. Chiều lên có hai bước. Bước một là chuỗi thành enum, và chuỗi lạ phải thành lỗi, không đoán:

```rust
// src/state.rs
impl FromStr for PaymentState {
    type Err = PaymentError;

    fn from_str(raw: &str) -> Result<PaymentState, PaymentError> {
        PaymentState::ALL
            .into_iter()
            .find(|s| s.as_str() == raw)
            .ok_or_else(|| PaymentError::UnknownState(raw.to_string()))
    }
}
```

Bước hai là enum thành kiểu. Nếu người gọi biết mình cần trạng thái nào (endpoint `capture` cần đúng `Authorized`), dùng `TryFrom` generic; nếu chưa biết, dùng `AnyPayment`, một `enum` mà mỗi biến thể bọc một kiểu `Payment<…>` và `match` ra từng nhánh:

```rust
// src/typestate.rs
impl<S: State> TryFrom<PaymentRecord> for Payment<S> {
    type Error = PaymentError;

    fn try_from(record: PaymentRecord) -> Result<Payment<S>, PaymentError> {
        if record.state() != S::KIND {
            return Err(PaymentError::WrongState {
                expected: S::KIND,
                found: record.state(),
            });
        }
        Ok(Payment::from_terms(record.into_terms()))
    }
}
```

Khi chạy lab (mục sau), nạp bản ghi `CAPTURED` thành `Payment<Authorized>` cho `Err(WrongState …)`, còn chuỗi `"AUTHORISED"` gõ sai cho `Err(UnknownState …)`.

Biên cũng là chỗ typestate hết tác dụng, theo hai nghĩa. Thứ nhất, `TryFrom` tin cột `state`: `PaymentRecord::restore(terms, PaymentState::Captured)` rồi `try_from` cho ra `Payment<Captured>` dù sổ cái chưa có bút toán nào. Thứ hai, Kafka giao lại thông điệp nghĩa là một lần nạp mới, một giá trị mới, nên E0382 không chặn được: đó là việc của trạng thái đã lưu và khóa idempotency.

Thiết kế `capture(self, …)` còn một cái giá: `self` bị tiêu thụ cả khi trả `Err`, nên nhánh lỗi không còn `Payment<Authorized>`. Không mất dữ liệu, vì bản ghi trong database vẫn là `AUTHORIZED`, nhưng muốn thử lại phải nạp lại. Thử lại chỉ có ích trong hai trường hợp. Một, lần trước đã ghi sổ mà chưa kịp lưu trạng thái: khóa `trip:TN-0001:capture` trả bút toán cũ, không ghi đôi (test `reload_and_retry_capture_is_idempotent`). Hai, lỗi tạm thời như mất kết nối database, thứ lab chưa có. Lỗi sổ cái trong lab đều xác định: thử lại ra đúng lỗi cũ và cần người xử lý (test `capture_error_consumes_value_and_retry_repeats_same_error`). Muốn giữ giá trị thì trả nó trong lỗi, `Result<_, (Payment<Authorized>, PaymentError)>`, đổi lại chữ ký nặng hơn; lab không chọn cách này.

## Khi nào enum là đủ?

Typestate không rẻ: nhiều kiểu hơn, nhiều `impl` hơn, và vẫn cần enum ở biên.

| Tiêu chí | Enum + `transition` | Typestate |
| --- | --- | --- |
| Trạng thái đọc từ database ở mỗi lệnh gọi | Hợp: giá trị đã là enum | Cần `TryFrom`/`AnyPayment` ở biên |
| Số trạng thái và cạnh thay đổi thường xuyên | Hợp: sửa một bảng | Mỗi trạng thái mới là kiểu và `impl` mới |
| Cạnh phụ thuộc dữ liệu (số tiền, thời hạn hold) | Hợp: `match` có guard | Khó: phải kiểm bằng giá trị, trả `Result` |
| Sai thứ tự gây thiệt hại lớn, gọi trong cùng một hàm | Test và review phải bắt | Hợp: sai thì không biên dịch |

Với Tới Nơi, quy tắc tạm dùng ở lab: **enum ở biên và trong bảng chuyển, typestate cho đoạn mã nghiệp vụ ngắn đã biết trạng thái** (ví dụ luồng capture rồi settle trong một use case). Không nên bọc cả service bằng typestate: khoản thanh toán sống qua nhiều request, mỗi request nạp lại từ database. Một bài ở cấp nâng cao sẽ quay lại hold có thời hạn; ở đó cạnh phụ thuộc thời gian, loại cạnh typestate không chứa nổi.

Với người từ C#: có thể bắt chước `Payment<TState>` bằng generic và extension method trên `Payment<Authorized>`, nhưng C# không có move: sau `var captured = authorized.Capture()`, biến `authorized` vẫn dùng được, nên kiểu không chặn được thu hai lần. Phần enum thì C# có `switch` expression (xem [bài về nullable và pattern matching](/articles/csharp-nullable-pattern-matching)): không bao hết trường hợp là cảnh báo CS8509, còn Rust với `match` thiếu biến thể là lỗi E0004. Điểm riêng của Rust ở đây là move, nhờ nó typestate chặn được dùng lại giá trị cũ.

## Chạy ví dụ và kiểm chứng

Từ gốc repository:

```bash
cargo run --manifest-path examples/toi-noi-rust/14-typestate-thanh-toan/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/14-typestate-thanh-toan/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/14-typestate-thanh-toan/Cargo.toml
```

Lệnh đầu chạy cùng một chuyến qua hai cách, rồi qua biên:

```text
== Cách 1: enum PaymentState, kiểm tra lúc chạy ==
trạng thái đầu: INITIATED
Capture khi INITIATED: Err(không thể Capture khi đang ở INITIATED)
Authorize -> AUTHORIZED
Capture -> CAPTURED (bút toán JE-000001)
Capture lần hai: Err(không thể Capture khi đang ở CAPTURED)
Số bút toán: 1; tài xế: 20.400 VND

== Cách 2: typestate Payment<S>, kiểm tra lúc biên dịch ==
kiểu: Payment<Initiated>, state() = INITIATED
kiểu: Payment<Authorized>, state() = AUTHORIZED
kiểu: Payment<Captured>, state() = CAPTURED (bút toán JE-000001)
khách -25.500 VND, nền tảng 5.100 VND, tài xế 20.400 VND
size_of Payment<Initiated> = 88, Payment<Captured> = 88

== Ở biên: lưu xuống và nạp lên ==
lưu: state = "CAPTURED"
đọc "AUTHORISED": Err(UnknownState("AUTHORISED"))
nạp CAPTURED thành Payment<Authorized>: Some(WrongState { expected: Authorized, found: Captured })
hoàn tiền -> REFUNDED (bút toán JE-000002)
tài xế: 0 VND; tổng toàn sổ: 0 VND

cặp hợp lệ: 6/30
```

`-- --verify` kiểm các giá trị này và in `VERIFY OK`; sai thì in `VERIFY FAIL: …` và thoát với mã 1. `cargo test` chạy 37 unit test, 2 integration test (có test hai cách cho cùng một sổ cái) và 5 doc test, trong đó 3 là `compile_fail`.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Đây là mô hình để học typestate, không phải service thanh toán. Không có database: <code>to_record</code> và <code>TryFrom</code> chỉ minh họa biên. <code>authorize()</code> không gọi VíXanh và chưa ghi gì vào sổ. Phí 5.100 được truyền vào. Không có đồng thời: hai request cùng nạp một bản ghi <code>AUTHORIZED</code> rồi cùng capture chỉ an toàn nhờ idempotency key, chưa có khóa hay phiên bản dòng. Chưa có thời hạn hold, capture một phần, hay <code>REFUND_PENDING</code> thất bại.</p>
</div>

## Bài tập

Thêm trạng thái `Voided` (hủy giữ tiền, cuối) đến được từ `Authorized` bằng sự kiện `Void`. Với mỗi cách (enum và typestate), nói compiler bắt buộc bạn sửa những chỗ nào và chỗ nào bạn có thể quên.

<details><summary>Xem lời giải</summary><p>Cả hai cách đều phải thêm <code>Voided</code> vào <code>PaymentState</code> (typestate cần nó cho <code>KIND</code>). Việc đó làm <code>as_str</code> và <code>AnyPayment::from</code> báo E0004 vì không có nhánh <code>_</code>: compiler chỉ ra những chỗ phải sửa. Mảng <code>ALL</code> thì không: quên thêm <code>Voided</code> vào đó, code vẫn biên dịch, chỉ <code>FromStr</code> từ chối chuỗi <code>"VOIDED"</code>. <b>Enum:</b> thêm <code>Void</code> vào <code>PaymentEvent</code>; <code>transition</code> vẫn biên dịch nhờ nhánh cuối, nên nếu quên <code>(Authorized, Void) =&gt; Ok(Voided)</code> thì sự kiện luôn bị từ chối lúc chạy. Test đếm cặp đỏ ngay khi <code>ALL</code> có thêm phần tử, vì nó kiểm 6 hợp lệ và 24 bị từ chối; sửa xong phải là 7 và 35 trên 42 cặp. <b>Typestate:</b> thêm <code>marker!(Voided, PaymentState::Voided)</code> và <code>impl Payment&lt;Authorized&gt; { fn void(self) -&gt; Payment&lt;Voided&gt; }</code>. Không có bảng nào để quên, nhưng cũng không có gì nhắc bạn viết <code>void()</code>. Chỗ dễ quên ở cả hai cách: migration cho cột <code>state</code>.</p></details>

Bài trước: [bài 13](/articles/rust-double-entry-journal-entry-posting). Bài tiếp: [bài 15](/articles/rust-closure-iterator-adapter-doi-soat-bo-nho).

## Tài liệu gốc

- [The Rust Book: Implementing an Object-Oriented Design Pattern](https://doc.rust-lang.org/book/ch18-03-oo-design-patterns.html), mục «Encoding States and Behavior as Types».
- [The Rust Book: Generic Data Types](https://doc.rust-lang.org/book/ch10-01-syntax.html), mục «In Method Definitions» về `impl` cho một kiểu cụ thể.
- [std::marker::PhantomData](https://doc.rust-lang.org/std/marker/struct.PhantomData.html).
- [The Rust Reference: Generic parameters](https://doc.rust-lang.org/reference/items/generics.html).
- [Rustdoc Book: Documentation tests](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html).
- [Error index: E0599](https://doc.rust-lang.org/error_codes/E0599.html).
