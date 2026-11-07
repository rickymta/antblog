---
slug: rust-vi-sao-so-cai-toi-noi-chon-rust
title: "Vì sao sổ cái Tới Nơi viết bằng Rust khi backend chính là C#?"
summary: "Bài 01: quyết định kiến trúc vì sao riêng service sổ cái dùng Rust dù backend chính là C#/.NET 10; bốn tiêu chí có bằng chứng chạy được, cái giá phải trả, ranh giới hẹp và điều kiện để xét lại."
category: rust
level: beginner
tags: [rust, toi-noi, ledger, adr, kien-truc]
language: vi
---

Đây là bài 01 của lộ trình **Rust cơ bản qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Hồ sơ dự án ghi nhận khoảng 2 triệu chuyến mỗi ngày; giả sử mỗi chuyến ghi ít nhất một bút toán, sổ cái nhận hàng triệu bút toán mỗi ngày. Lệch một đồng trên một bút toán là lệch đối soát cuối ngày với cổng thanh toán, và rất khó tìm ra nó trong hàng triệu dòng.

Bạn đã theo loạt C#/.NET 10, nên câu hỏi hợp lý là: vì sao riêng service sổ cái lại dùng ngôn ngữ khác? Bài này là một **quyết định kiến trúc (ADR)**: nêu tiêu chí của đúng service này, có bằng chứng chạy được, nói rõ cái giá và lúc nào nên đổi ý.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Viết sổ cái bằng Rust thì sổ cái tự đúng nghiệp vụ, đúng hay sai?</p>
<details><summary>Xem đáp án</summary><p><b>Sai.</b> Sổ cái cần invariant (mỗi bút toán cân bằng, không sửa bút toán đã ghi), quy tắc làm tròn và đối soát với cổng thanh toán. Ngôn ngữ nào cũng phải tự viết các quy tắc đó. Rust chỉ giúp những invariant ấy khó bị vi phạm hơn ở tầng kiểu và bộ nhớ.</p></details>
</div>

## Sổ cái cần gì mà các service khác không cần?

Tài liệu kiến trúc Tới Nơi (mục 4.9, Payment & Wallet) có một khuyến nghị mạnh: «Ledger bất biến (append-only) cho các bút toán tài chính; không chỉ lưu một cột balance có thể ghi đè.» Mục 9.1 liệt kê sáu trạng thái thanh toán: `INITIATED`, `AUTHORIZED`, `CAPTURED`, `FAILED`, `REFUND_PENDING`, `REFUNDED`.

Từ đó, tiêu chí của service sổ cái là:

- **Tiền không được lặng lẽ sai.** Mọi phép cộng trừ hoặc đúng, hoặc báo lỗi rõ ràng.
- **Append-only và idempotent.** Bút toán đã ghi không bị sửa; ghi lần hai cùng `idempotency_key` (ví dụ `trip:TN-0001:capture`) trả về bút toán cũ.
- **Mọi trạng thái đều được xử lý.** Thêm trạng thái mới mà một chỗ quên xử lý là lỗi tiền bạc.
- **Độ trễ p99 và bộ nhớ dễ đoán.** Sổ cái nằm trên đường thanh toán; độ trễ đuôi tăng bất thường kéo cả luồng chờ.

Lưu ý: với khoảng 70 chuyến tạo mới mỗi giây lúc cao điểm (theo hồ sơ), **thông lượng thô không phải lý do** đổi ngôn ngữ: ở mức đó, giới hạn nhiều khả năng nằm ở database và mạng hơn là ở ngôn ngữ. Đây là lập luận, loạt này không đo C# trên tải sổ cái.

## C#/.NET 10 có đủ tốt để viết sổ cái không?

Có. C#/.NET 10 là backend chủ đạo của Tới Nơi: đặt chuyến, đơn hàng, điều phối đều viết bằng nó, và nó vẫn đủ để viết một sổ cái đúng. `decimal`, `long` với `checked`, record bất biến ([bài về Money](/articles/csharp-record-struct-money-quote)), transaction và idempotency ([bài về transaction](/articles/dotnet10-transaction-concurrency-idempotency)) đều dùng được; GC của .NET hiện đại thường có thời gian dừng ngắn và tinh chỉnh được ([bài về GC](/articles/dotnet10-allocation-gc-disposal)).

Vì vậy đây **không phải xếp hạng ngôn ngữ**. Đây là lựa chọn của dự án mẫu cho một service có tiêu chí hẹp, và nó có điều kiện để bị đảo. Một đội khác, với SLO và kỹ năng khác, hoàn toàn có thể chọn C# cho cả sổ cái.

## Bốn tiêu chí và bằng chứng chạy được

Lab `01-vi-sao-rust` là một binary chỉ dùng thư viện chuẩn; mỗi tiêu chí có một đoạn code.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 240" role="img" aria-label="Hình 1. Bốn tiêu chí của sổ cái, mỗi tiêu chí gắn với một cơ chế của Rust và một bằng chứng trong lab; dải cảnh báo bên dưới nhắc rằng Rust không thay thế invariant nghiệp vụ."><rect x="20" y="25" width="196" height="125" rx="10" class="f-pk-soft s-pk"/><text x="36" y="53" class="t-b">Bất biến ở kiểu</text><text x="36" y="79" class="t-small">newtype Money(i64)</text><text x="36" y="99" class="t-small">checked_add trả None</text><text x="36" y="119" class="t-small">khi tràn i64</text><rect x="236" y="25" width="196" height="125" rx="10" class="f-pk-soft s-pk"/><text x="252" y="53" class="t-b">Vét cạn trạng thái</text><text x="252" y="79" class="t-small">match trên enum</text><text x="252" y="99" class="t-small">thiếu nhánh thì lỗi</text><text x="252" y="119" class="t-small">biên dịch E0004</text><rect x="452" y="25" width="196" height="125" rx="10" class="f-pk-soft s-pk"/><text x="468" y="53" class="t-b">Một chủ, một lần sửa</text><text x="468" y="79" class="t-small">&amp;mut là độc quyền</text><text x="468" y="99" class="t-small">đọc qua alias và sửa</text><text x="468" y="119" class="t-small">cùng lúc: lỗi E0502</text><rect x="668" y="25" width="196" height="125" rx="10" class="f-pk-soft s-pk"/><text x="684" y="53" class="t-b">Giải phóng tất định</text><text x="684" y="79" class="t-small">Drop chạy cuối phạm vi</text><text x="684" y="99" class="t-small">thứ tự ngược khai báo</text><text x="684" y="119" class="t-small">không chờ GC</text><rect x="20" y="172" width="844" height="48" rx="10" class="f-warn-soft s-warn dash"/><text x="36" y="201" class="t-b">Không thay thế: cân bằng bút toán, làm tròn, idempotency, đối soát</text></svg>
</div>
<figcaption>Hình 1. Bốn tiêu chí và cơ chế Rust đáp ứng chúng. Dải gạch đứt ở dưới là phần luôn thuộc về nghiệp vụ, bất kể ngôn ngữ.</figcaption>
</figure>

**(a) Bất biến ở kiểu.** Tiền là newtype bọc `i64` đơn vị đồng. Phép cộng chỉ có một cách là `checked_add`, trả `Option`: tràn thì `None`, không có ngoại lệ ngầm và cũng không wrap im lặng.

```rust
// src/main.rs (trích)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Money(i64);

impl Money {
    fn checked_add(self, other: Money) -> Option<Money> {
        self.0.checked_add(other.0).map(Money)
    }
}
```

Vì `Money` không cài `Add`, ghi `a + b` trên hai `Money` không biên dịch được; người viết buộc phải đi qua hàm kiểm tra tràn. C# làm được tương tự: không định nghĩa `operator +`, cộng trong ngữ cảnh `checked` (tràn ném `OverflowException`) hoặc trả `Money?`. Khác biệt chủ yếu là quy ước: ở Rust, khả năng tràn nằm ngay trong kiểu trả về, người gọi thấy và phải xử lý.

**(b) `match` vét cạn.** Sáu trạng thái thanh toán là một `enum`; hàm dưới đây không có nhánh `_`, nên mọi biến thể đều phải được liệt kê.

```rust
// src/main.rs (trích)
fn posts_ledger_entry(state: PaymentState) -> bool {
    match state {
        PaymentState::Captured | PaymentState::Refunded => true,
        PaymentState::Initiated
        | PaymentState::Authorized
        | PaymentState::Failed
        | PaymentState::RefundPending => false,
    }
}
```

Thêm biến thể `Disputed` vào enum rồi build lại, rustc từ chối (đây là thông điệp thật, rút gọn):

```text
error[E0004]: non-exhaustive patterns: `PaymentState::Disputed` not covered
  --> src/main.rs:31:11
   |
31 |     match state {
   |           ^^^^^ pattern `PaymentState::Disputed` not covered
```

Mọi `match` không có nhánh `_` trên enum này đều báo cùng lỗi, nên không chỗ nào bị quên. Trong C#, `switch` expression ([bài về pattern matching](/articles/csharp-nullable-pattern-matching)) không vét cạn cho cảnh báo CS8509, còn câu lệnh `switch` thì không cảnh báo; nâng cảnh báo thành lỗi (`WarningsAsErrors`) là cấu hình, không phải mặc định.

**(c) Ownership.** Giả sử ta đang đọc dòng đầu của một bút toán qua tham chiếu, trong khi code khác thêm dòng vào chính bút toán đó. Rust cấm vì `&mut` là độc quyền: không thể có tham chiếu sửa khi còn tham chiếu đọc đang sống. Đoạn dưới nằm trong lab dưới dạng chú thích; bỏ comment sẽ thấy lỗi E0502 thật:

```rust
// src/main.rs (lỗi E0502 nếu bỏ comment)
// let mut entry = vec![Money(-25_500), Money(5_100), Money(20_400)];
// let first = &entry[0];
// entry.push(Money(1));
// println!("{first:?}");
```

```text
error[E0502]: cannot borrow `entry` as mutable because it is also borrowed as immutable
   --> src/main.rs:99:5
    |
 98 |     let first = &entry[0];
    |                  ----- immutable borrow occurs here
 99 |     entry.push(Money(1));
    |     ^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
100 |     println!("{first:?}");
    |                ----- immutable borrow later used here
```

Lỗi bị chặn **khi biên dịch**, không đợi lúc chạy. Đây chưa phải append-only ở tầng database; bảo đảm đó nằm ở schema và quyền ghi.

**(d) `Drop` tất định.** Khi một giá trị ra khỏi phạm vi, Rust gọi `Drop` ngay tại đó, theo thứ tự ngược với thứ tự khai báo. Lab mở hai tài nguyên giả (`db_conn`, `entry_lock`) và ghi lại thứ tự giải phóng:

```rust
// src/main.rs (rút gọn)
{
    let _conn = Guard { name: "db_conn", log: Rc::clone(&log) };
    let _lock = Guard { name: "entry_lock", log: Rc::clone(&log) };
    log.borrow_mut().push("work done".to_string());
} // hết phạm vi: biến khai báo sau bị drop trước
```

Output thật của `cargo run`:

```text
1) 20.400 + 5.100 = Some(Money(25500))
1) i64::MAX + 1 = None
2) trạng thái ghi sổ: [Captured, Refunded]
3) thứ tự: work done -> drop entry_lock -> drop db_conn
```

Với tài nguyên như kết nối hay khóa, C# cũng giải phóng tất định bằng `using`/`IDisposable`. Khác biệt là ở Rust cả bộ nhớ heap cũng theo cùng quy tắc, không có bộ gom rác. Đó là cơ sở cho **xu hướng** độ trễ đuôi dễ đoán hơn và bộ nhớ thấp hơn; chỉ là xu hướng, **chưa đo**, vì allocator, khóa, I/O và Kafka vẫn gây trễ. Ta không đưa con số nào cho đến bài hiệu năng của lộ trình nâng cao.

## Cái giá của Rust là gì?

Mỗi lợi ích đều có giá, cần viết ra trước khi quyết định:

- **Đường cong học.** Ownership và borrow checker làm những tuần đầu chậm hơn đáng kể.
- **Tuyển dụng.** Người có kinh nghiệm Rust production ít hơn; service chỉ một hai người hiểu là rủi ro.
- **Thời gian build.** Biên dịch Rust thường lâu hơn, nhất là bản release.
- **Hệ sinh thái.** Với ORM hay UI, hệ sinh thái .NET rộng và trưởng thành hơn.
- **Hai toolchain.** CI, image, giám sát, quy trình bảo mật đều phải làm hai lần.

Cách giảm chi phí là giữ **ranh giới hẹp**: chỉ ledger và đối soát; giao tiếp với phần còn lại bằng hợp đồng gRPC và Kafka có version; một team nhỏ sở hữu trọn service.

## Rust nằm ở đâu trong hệ thống?

Rust **chỉ** dành cho ledger và đối soát; ta không viết lại Booking hay Payment, hai service đó vẫn là C#. Ledger sở hữu một PostgreSQL riêng, không service nào ghi thẳng vào bảng của nó. Booking phát `TripCompleted`, Payment phát `PaymentCaptured` lên Kafka; ledger tiêu thụ, ghi bút toán rồi phát `LedgerEntryPosted`. Khi cần phản hồi ngay, Payment hoặc Admin BFF gọi gRPC `GetBalance` và `PostEntry`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 2. Ledger service viết bằng Rust và PostgreSQL riêng nằm trong một ranh giới; hai service C# là Booking và Payment giao tiếp với ledger qua Kafka và gRPC."><defs><marker id="rust-vi-sao-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="40" width="190" height="64" rx="10" class="f-pk-soft s-pk"/><text x="36" y="66" class="t-b">Booking / Trip</text><text x="36" y="88" class="t-small">C# · .NET 10</text><rect x="20" y="214" width="190" height="64" rx="10" class="f-pk-soft s-pk"/><text x="36" y="240" class="t-b">Payment</text><text x="36" y="262" class="t-small">C# · .NET 10</text><rect x="300" y="30" width="200" height="100" rx="10" class="f-phys-soft s-phys"/><text x="316" y="54" class="t-b">Apache Kafka</text><text x="316" y="75" class="t-small">TripCompleted</text><text x="316" y="93" class="t-small">PaymentCaptured</text><text x="316" y="111" class="t-small">LedgerEntryPosted</text><rect x="578" y="38" width="290" height="252" rx="12" class="f-none s-warn dash"/><text x="594" y="55" class="t-small">Ranh giới Rust: ledger + DB riêng</text><rect x="594" y="68" width="258" height="102" rx="10" class="f-pk-soft s-pk"/><text x="610" y="96" class="t-b">Ledger service</text><text x="610" y="118" class="t-small">Rust · ghi bút toán, đối soát</text><text x="610" y="138" class="t-small">append-only · idempotent</text><rect x="594" y="214" width="258" height="60" rx="10" class="f-phys-soft s-phys"/><text x="610" y="240" class="t-b">PostgreSQL của ledger</text><text x="610" y="260" class="t-small">chỉ ledger được ghi</text><path d="M210 72 L300 72" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-vi-sao-0-a)"/><path d="M210 232 L350 130" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-vi-sao-0-a)"/><path d="M500 88 L594 88" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-vi-sao-0-a)"/><path d="M594 118 L500 118" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-vi-sao-0-a)"/><path d="M210 262 L594 160" class="f-none s-pk dash" stroke-width="2" marker-end="url(#rust-vi-sao-0-a)"/><path d="M723 170 L723 214" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-vi-sao-0-a)"/><text x="512" y="80" class="t-small">tiêu thụ</text><text x="522" y="136" class="t-small">phát</text><text x="330" y="268" class="t-small">gRPC: GetBalance / PostEntry</text></svg>
</div>
<figcaption>Hình 2. Ledger Rust là một đảo nhỏ giữa các service C#. Mũi tên liền tới và từ Kafka là luồng bất đồng bộ; nét đứt xanh là gRPC (đồng bộ). Ledger không dùng chung database với ai.</figcaption>
</figure>

## Khi nào quyết định này sai?

Một ADR tốt nói được lúc nào nó hết hiệu lực. Nên xét lại khi:

- **Đội không giữ được kỹ năng Rust.** Người hiểu service rời đi, review chậm, sửa lỗi phải nhờ người ngoài.
- **Lượng bút toán nhỏ và SLO đã đạt.** Nếu .NET đáp ứng p99 và bộ nhớ theo **số đo** trên cùng tải, lợi ích của Rust không bù nổi chi phí hai toolchain.
- **Chi phí vận hành vượt lợi ích.** CI, bảo mật, on-call cho hai ngôn ngữ tốn hơn phần đạt được.
- **Ranh giới bị xói mòn.** Nếu Rust lan sang Booking, Payment hay báo cáo thì lập luận «đảo nhỏ» không còn đúng.

Bảng dưới so sánh theo tiêu chí của sổ cái.

| Tiêu chí | C#/.NET 10 | Rust | Ghi chú cho sổ cái |
| --- | --- | --- | --- |
| Tiền đúng | `decimal`, `long` với `checked` | `Money(i64)` với `checked_add` | Cả hai làm được; C# cần bật `checked`, Rust đặt tràn vào kiểu trả về |
| Mọi trạng thái được xử lý | `switch` expression thiếu nhánh: cảnh báo CS8509 | `match` thiếu nhánh: lỗi E0004 | Cảnh báo C# có thể nâng thành lỗi bằng cấu hình |
| Quản lý bộ nhớ | GC thế hệ, pause thường ngắn | Ownership, `Drop`, không GC | Rust dễ đoán hơn theo xu hướng; cần đo, không dùng khẩu hiệu |
| Hệ sinh thái | Rất rộng (ORM, web, UI) | Nhỏ hơn cho ORM và UI | Ledger không cần UI; ORM không bắt buộc |
| Người và tuyển dụng | Nhiều người biết, onboarding nhanh | Học lâu hơn, ít người có kinh nghiệm | Rủi ro lớn nhất của quyết định này |
| Build và vận hành | Build nhanh hơn, một toolchain với hệ thống | Build chậm hơn, thêm toolchain | Chi phí cố định, trả hằng ngày |

## Lộ trình 68 bài đi qua những gì?

Lộ trình có 3 cấp độ, 7 phần, 68 bài. **Cơ bản** (20 bài): ngôn ngữ Rust và thư viện `ledger-core` với bất biến double-entry. **Trung bình** (20 bài): service bất đồng bộ với tokio, sqlx, axum, tonic và Kafka. **Nâng cao** (28 bài): nghiệp vụ sổ cái đầy đủ, kiến trúc, hiệu năng có số đo và vận hành.

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository (cần Rust stable cài bằng rustup, edition 2024):

```bash
cargo run --manifest-path examples/toi-noi-rust/01-vi-sao-rust/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/01-vi-sao-rust/Cargo.toml -- --verify
```

Lệnh đầu in ba bằng chứng trên. Lệnh sau `assert` từng giá trị (25.500, tràn là `None`, chỉ `Captured` và `Refunded` ghi sổ, thứ tự drop) và in `VERIFY OK`; sai thì dừng với exit code khác 0.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Lab chỉ chứng minh cơ chế ngôn ngữ trên vài dòng code. Nó chưa có bút toán đủ hai phía, idempotency, database, Kafka hay gRPC, và không nói gì về hiệu năng. Các con số tải (2 triệu chuyến mỗi ngày, 70 chuyến mỗi giây) là số hư cấu của dự án mẫu; chọn ngôn ngữ cho service thật cần số đo trên tải thật.</p>
</div>

**Bài tập:** liệt kê ba service khác của Tới Nơi (ngoài sổ cái) và lập luận có nên dùng Rust cho từng cái không, theo cùng bốn tiêu chí.

<details><summary>Xem lời giải</summary>
<p><b>Location ingest (nhận vị trí tài xế):</b> định hướng của dự án là Go. Việc chính là rất nhiều kết nối, worker song song và backpressure; mất một bản ghi vị trí không làm lệch tiền. Bốn tiêu chí của sổ cái không phải áp lực chính.</p>
<p><b>Fraud scoring:</b> định hướng là Python, vì pipeline dữ liệu và mô hình có sẵn thư viện; độ tin cậy nằm ở đánh giá mô hình, không ở kiểu hay bộ nhớ.</p>
<p><b>Booking:</b> giữ C#. Đây là backend nghiệp vụ chủ đạo, cần aggregate, transaction, CQRS, hệ sinh thái .NET và đội đã có kỹ năng; viết lại sang Rust chỉ thêm chi phí hai toolchain.</p>
<p>Điểm chung: chọn theo tiêu chí của từng service và khả năng của đội, không theo xếp hạng ngôn ngữ.</p>
</details>

Bài tiếp: [bài 02](/articles/rust-cargo-tu-source-code-den-but-toan-dau-tien).

## Tài liệu gốc

- [The Rust Programming Language (The Book)](https://doc.rust-lang.org/book/).
- [Ownership và borrowing trong The Book](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html).
- [Tham chiếu `i64::checked_add`](https://doc.rust-lang.org/std/primitive.i64.html#method.checked_add).
- [Trait `Drop`](https://doc.rust-lang.org/std/ops/trait.Drop.html).
- [Mã lỗi E0004](https://doc.rust-lang.org/error_codes/E0004.html) và [E0502](https://doc.rust-lang.org/error_codes/E0502.html).
