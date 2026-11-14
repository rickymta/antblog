---
slug: rust-command-handler-pipeline-domain-event-khong-framework
title: "Command handler, pipeline và domain event không cần framework"
summary: "Bài 52: trait Command có Output liên kết, dispatch generic bắt thiếu handler lúc biên dịch, pipeline gồm validate, trace, transaction với thứ tự chứng minh bằng log, domain event ghi outbox cùng transaction, chi phí của Box, so với mediator của loạt C#."
category: rust-advanced
level: advanced
tags: [rust, toi-noi, ledger, cqrs, pipeline, domain-event]
language: vi
---

Handler thu cước của service sổ cái lúc đầu chỉ dựng bút toán `trip:TN-0001:capture` rồi ghi. Sau đó nó thêm bước kiểm tra đầu vào, log, transaction và sự kiện `LedgerEntryPosted`. Đặt sai một dòng ở bước "phát sự kiện" là service báo cho cả hệ thống về một bút toán chưa từng tồn tại. Bài này tách các việc đó thành các lớp có thứ tự bằng trait và generic, không framework, rồi chạy các kịch bản hỏng để xem mỗi lớp làm gì.

Đây là bài 52 của lộ trình **Rust nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. [Bài 51](/articles/rust-hexagonal-clean-workspace-crates-port-adapter) chia workspace theo chiều phụ thuộc; bài này ở tầng application. Lab dùng lại `ledger-core` của [bài 20](/articles/rust-tong-hop-ledger-core-cargo-doc-workspace), schema và trigger cân bút toán của [bài 25](/articles/rust-schema-so-cai-accounts-journal-postings-partition), outbox của [bài 37](/articles/rust-outbox-inbox-idempotent-consumer). Đối chiếu: [CQRS và Mediator của loạt C#](/articles/dotnet10-cqrs-mediator-pipeline).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Command <code>VoidHold</code> chưa có handler nào. Gửi nó qua pipeline sẽ lỗi lúc biên dịch hay lúc chạy? Và nếu ai đó quên bọc lớp <code>Transactional</code> quanh handler thì sao?</p>
<details><summary>Xem đáp án</summary><p>Thiếu handler là lỗi biên dịch <code>E0277</code>, chương trình không ra binary. Quên lớp <code>Transactional</code> thì trình biên dịch không biết: handler chạy rồi báo <code>NoTransaction</code> lúc chạy. Hệ thống kiểu bắt được "có handler", chưa bắt được "có đủ lớp".</p></details>
</div>

## Command mang theo kiểu kết quả, handler là trait generic

**Command** là một ý định thay đổi trạng thái, ví dụ `CapturePayment { trip_id, customer, driver, fare_dong }`. Lab khai báo hai trait (`src/command.rs`, rút gọn bỏ chú thích):

```rust
pub trait Command: Send + 'static {
    type Output: Send + 'static;
    const NAME: &'static str;
}

pub trait CommandHandler<C: Command>: Send + Sync {
    fn handle(&self, cmd: C, ctx: &mut Ctx)
        -> impl Future<Output = Result<C::Output, AppError>> + Send;
}
```

`Output` là kiểu liên kết nên nằm ở command: `CapturePayment` quyết định kết quả là `CaptureReceipt`, người viết handler không chọn được. `CommandHandler<C>` là trait có tham số, nên một struct (`LedgerHandlers`) cài đặt nó cho nhiều command. Dạng `-> impl Future + Send` thay cho `async fn` trần là do giới hạn `Send` ở [bài 26](/articles/rust-repository-unit-of-work-trait-transaction).

Handler trả sai kiểu kết quả thì bị chặn ngay. Cho `Ping` (kết quả `u32`) trả `String`, rustc 1.98.1 báo (rút gọn):

```text
error[E0271]: expected `impl Future<Output = Result<String, AppError>>` to be a future that
resolves to `Result<u32, AppError>`, but it resolves to `Result<String, AppError>`
```

Đoạn thử nằm trong doc test `compile_fail,E0271` ở `src/lib.rs`. Trên stable, `compile_fail` chỉ kiểm code không biên dịch, không đối chiếu mã lỗi: đổi thẻ thành `E9999` doc test vẫn xanh. Mã lỗi trong bài lấy từ lần chạy rustc riêng. `&mut Ctx` là ngữ cảnh của đúng một lần gọi: clock, transaction đang mở, sự kiện chờ thu về, log thứ tự. Việc kiểm tra đầu vào dùng trait riêng `Validate`.

## Thiếu handler: lỗi lúc biên dịch hay lúc chạy?

Lab cố ý khai báo `VoidHold` mà không viết handler. Gửi nó qua pipeline thật thì rustc dừng (trích, rút gọn):

```text
error[E0277]: the trait bound `LedgerHandlers: CommandHandler<VoidHold>` is not satisfied
help: `LedgerHandlers` implements trait `CommandHandler<C>`
     impl CommandHandler<CapturePayment> for LedgerHandlers
     impl CommandHandler<PostRawEntry> for LedgerHandlers
  = note: required for `Validated<Traced<Transactional<LedgerHandlers>>>`
          to implement `CommandHandler<VoidHold>`
```

Thông điệp lỗi liệt kê cả các command `LedgerHandlers` đã cài đặt. Đối chứng là `DynBus`, một `HashMap<TypeId, …>` tra handler lúc chạy như container. Cùng lời gọi biên dịch bình thường, rồi chạy ra:

```text
DynBus.send(VoidHold) biên dịch được, chạy ra: không có handler cho VoidHold
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 230" role="img" aria-label="Hình 1. Cùng một lời gọi thiếu handler: dispatch generic dừng ở rustc với E0277; bus tra cứu lúc chạy biên dịch được và chỉ lỗi NoHandler khi dòng đó chạy."><defs><marker id="rs52-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="15" width="250" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="41" class="t-b">Generic: viết lời gọi</text><text x="34" y="65" class="t-small">handle(VoidHold { .. }, ctx)</text><text x="34" y="85" class="t-small">trên LedgerPipeline</text><path d="M270 57 L315 57" class="f-none s-phys" stroke-width="2" marker-end="url(#rs52-0-a)"/><rect x="315" y="15" width="250" height="85" rx="10" class="f-phys-soft s-phys"/><text x="329" y="41" class="t-b">rustc tìm impl</text><text x="329" y="65" class="t-small">CommandHandler&lt;VoidHold&gt;</text><text x="329" y="85" class="t-small">cho LedgerHandlers: không có</text><path d="M565 57 L610 57" class="f-none s-phys" stroke-width="2" marker-end="url(#rs52-0-a)"/><rect x="610" y="15" width="250" height="85" rx="10" class="f-pk-soft s-pk"/><text x="624" y="41" class="t-b">E0277, dừng ở build</text><text x="624" y="65" class="t-small">không ra binary</text><text x="624" y="85" class="t-small">lỗi nằm ở chỗ viết code</text><rect x="20" y="130" width="250" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="156" class="t-b">DynBus: viết lời gọi</text><text x="34" y="180" class="t-small">bus.send(VoidHold { .. })</text><text x="34" y="200" class="t-small">tra theo TypeId</text><path d="M270 172 L315 172" class="f-none s-phys" stroke-width="2" marker-end="url(#rs52-0-a)"/><rect x="315" y="130" width="250" height="85" rx="10" class="f-phys-soft s-phys"/><text x="329" y="156" class="t-b">rustc chấp nhận</text><text x="329" y="180" class="t-small">send&lt;C: Command&gt; chỉ đòi</text><text x="329" y="200" class="t-small">C: Command, không hỏi handler</text><path d="M565 172 L610 172" class="f-none s-phys" stroke-width="2" marker-end="url(#rs52-0-a)"/><rect x="610" y="130" width="250" height="85" rx="10" class="f-warn-soft s-warn"/><text x="624" y="156" class="t-b">Chạy: NoHandler</text><text x="624" y="180" class="t-small">HashMap không có TypeId</text><text x="624" y="200" class="t-small">chỉ lộ khi dòng này chạy</text></svg>
</div>
<figcaption>Hình 1. Cùng một lời gọi thiếu handler. Dispatch generic bắt ở lúc biên dịch; bus tra cứu lúc chạy chỉ bắt khi dòng gọi thật sự được thực thi.</figcaption>
</figure>

Gửi command chưa có handler qua pipeline thì không ra được binary. Ranh giới của sự bảo đảm cũng đo được: test `handler_without_transaction_layer_fails_at_runtime` gọi `LedgerHandlers` trần, không có `Transactional`, và nhận `AppError::NoTransaction` lúc chạy. Trình biên dịch kiểm "có impl", không kiểm "đủ lớp"; lab giảm rủi ro bằng cách chỉ lắp pipeline ở một hàm (`ledger_pipeline`).

## Pipeline là các struct bọc nhau, thứ tự nằm trong kiểu

Mỗi lớp là một struct generic cài đặt chính `CommandHandler<C>` bằng cách gọi handler bên trong, nên cả chuỗi vẫn là một handler. Lớp validate (`src/layers.rs`):

```rust
pub struct Validated<H>(pub H);

impl<C, H> CommandHandler<C> for Validated<H>
where
    C: Command + Validate,
    H: CommandHandler<C>,
{
    async fn handle(&self, cmd: C, ctx: &mut Ctx) -> Result<C::Output, AppError> {
        if let Err(reason) = cmd.validate() {
            ctx.log(format_args!("validate: từ chối {} ({reason})", C::NAME));
            return Err(AppError::Validation(reason));
        }
        ctx.log(format_args!("validate: ok"));
        self.0.handle(cmd, ctx).await
    }
}
```

`Traced` và `Transactional` cùng khuôn. Pipeline chuẩn là một alias kiểu:

```rust
pub type LedgerPipeline = Validated<Traced<Transactional<LedgerHandlers>>>;
```

Log chứng minh thứ tự này. Thu cước `TN-0001` (phí 5.100, tài xế 20.400) ra:

```text
validate: ok
trace: bắt đầu CapturePayment
tx: begin
handler: CapturePayment trip=TN-0001
handler: đã INSERT journal_entries JE-000001
handler: aggregate sinh 1 sự kiện, chuyển cho ctx
tx: outbox +1 dòng
tx: commit
trace: kết thúc CapturePayment ok
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 240" role="img" aria-label="Hình 2. Bốn lớp lồng nhau Validated, Traced, Transactional, Handler; lệnh đi vào theo mũi tên liền, kết quả đi ra theo mũi tên đứt; cả bốn dùng chung một Ctx của một lần gọi."><defs><marker id="rs52-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="25" width="180" height="90" rx="10" class="f-phys-soft s-phys"/><text x="34" y="52" class="t-b">Validated</text><text x="34" y="76" class="t-small">chặn lệnh sai</text><text x="34" y="96" class="t-small">trước khi chạm DB</text><rect x="240" y="25" width="180" height="90" rx="10" class="f-phys-soft s-phys"/><text x="254" y="52" class="t-b">Traced</text><text x="254" y="76" class="t-small">log lúc vào</text><text x="254" y="96" class="t-small">và lúc ra</text><rect x="460" y="25" width="180" height="90" rx="10" class="f-pk-soft s-pk"/><text x="474" y="52" class="t-b">Transactional</text><text x="474" y="76" class="t-small">begin, thu event,</text><text x="474" y="96" class="t-small">outbox, commit/rollback</text><rect x="680" y="25" width="180" height="90" rx="10" class="f-phys-soft s-phys"/><text x="694" y="52" class="t-b">Handler</text><text x="694" y="76" class="t-small">domain + SQL</text><text x="694" y="96" class="t-small">ctx.raise(event)</text><path d="M200 55 L240 55" class="f-none s-phys" stroke-width="2" marker-end="url(#rs52-1-a)"/><path d="M420 55 L460 55" class="f-none s-phys" stroke-width="2" marker-end="url(#rs52-1-a)"/><path d="M640 55 L680 55" class="f-none s-phys" stroke-width="2" marker-end="url(#rs52-1-a)"/><path d="M240 90 L200 90" class="f-none s-line dash" stroke-width="2" marker-end="url(#rs52-1-a)"/><path d="M460 90 L420 90" class="f-none s-line dash" stroke-width="2" marker-end="url(#rs52-1-a)"/><path d="M680 90 L640 90" class="f-none s-line dash" stroke-width="2" marker-end="url(#rs52-1-a)"/><rect x="20" y="140" width="840" height="60" rx="10" class="f-warn-soft s-warn"/><text x="36" y="166" class="t-b">Ctx của một lần gọi</text><text x="36" y="188" class="t-small">clock · transaction đang mở · sự kiện chờ thu về · log thứ tự</text><text x="20" y="226" class="t-small">Kiểu của cả chuỗi: Validated&lt;Traced&lt;Transactional&lt;LedgerHandlers&gt;&gt;&gt;. Mũi tên liền: lệnh vào. Đứt: kết quả ra.</text></svg>
</div>
<figcaption>Hình 2. Bốn lớp lồng nhau dùng chung một <code>Ctx</code>. Thứ tự gọi là thứ tự lồng của kiểu.</figcaption>
</figure>

Đổi thứ tự là đổi một dòng lắp ráp, log đổi theo. Lệnh có cước 0 đi qua `Validated` ở ngoài cùng chỉ để lại một dòng `validate: từ chối …`, không có `tx: begin`, không có dòng `trace`. Lắp `Traced(Validated(…))` thì lệnh đó để lại ba dòng, từ `trace: bắt đầu` tới `trace: kết thúc CapturePayment lỗi`. Bài C# đặt telemetry bọc ngoài validation và nêu lý do: yêu cầu bị từ chối vẫn được tính vào metric ở biên application. Lab mặc định ngược lại; test `layer_order_is_visible_in_the_log` khoá lựa chọn đó.

## Lỗi ở lớp nào thì rollback ra sao?

Sau kịch bản thành công (1 bút toán, 1 dòng outbox), lab chạy ba kịch bản hỏng và đọc lại hai bảng ngoài mọi transaction:

| Kịch bản | Hỏng ở lớp | Log thấy | Sau đó trong DB |
|---|---|---|---|
| Cước 0 | validate | một dòng `validate: từ chối …` | không đổi |
| Khách `CU-404` | handler, lỗi khoá ngoại `23503` | `tx: rollback, bỏ 0 sự kiện chưa ghi` | không đổi; header `JE-000003` đã INSERT bị huỷ |
| Bút toán lệch qua `PostRawEntry` | COMMIT, trigger `23514` | `tx: outbox +1 dòng`, rồi `tx: commit lỗi (SQLSTATE 23514), PostgreSQL đã rollback` | không đổi; dòng outbox vừa ghi cũng mất |

Kịch bản thứ ba đáng đọc kỹ. Trigger `postings_balanced` của bài 25 là `DEFERRABLE INITIALLY DEFERRED`, nên chỉ chạy lúc COMMIT. Handler ghi xong, `Transactional` thu sự kiện và ghi dòng outbox, mọi câu lệnh đều thành công; lỗi chỉ đến ở COMMIT. Thiết kế phát sự kiện "sau khi handler xong" đã nói dối ở thời điểm đó. `PostRawEntry` mô phỏng một bug lọt qua aggregate; trigger là tuyến phòng thủ cuối.

`Transactional` gọi `tx.rollback().await` tường minh thay vì dựa vào `Drop` (lý do ở bài 26). Mã `JE-000003` tới `JE-000005` không bao giờ vào sổ vì sequence `identity` không hoàn tác; `JE-000002` đã bị lần gửi lại đốt mất (`ON CONFLICT` lấy số trước khi phát hiện trùng, xem bài 26). Gửi lại đúng `TN-0001` cho thấy phía ngược lại: key đã có thì handler không ghi, không sinh sự kiện, log ghi `tx: outbox +0 dòng`.

## Domain event: aggregate sinh, pipeline thu về, outbox giữ

**Domain event** là sự kiện do chính aggregate ghi nhận khi trạng thái đổi. Aggregate `TripEntry` bọc bút toán thu cước và giữ sự kiện trong bộ nhớ; nó không biết outbox, transaction hay Kafka (`src/domain.rs`, rút gọn):

```rust
pub fn mark_posted(&mut self, entry_id: &str, posted_at: &str) {
    // ... dựng LedgerEntryPosted từ các posting của bút toán
    self.events.push(DomainEvent::EntryPosted(LedgerEntryPosted { /* ... */ }));
}

pub fn take_events(&mut self) -> Vec<DomainEvent> {
    std::mem::take(&mut self.events)
}
```

Sự kiện được ghi nhận sau khi database cấp mã bút toán, vì payload cần `entryId`. Sự kiện sinh ra trước commit nhưng chỉ nằm ở RAM. Handler chuyển cho `ctx.raise(events)`; khi handler xong, `Transactional` thu về và ghi outbox trong chính transaction đó (`src/layers.rs`, đoạn giữa hai nhánh):

```rust
let events = ctx.take_events();
for event in &events {
    sqlx::query("INSERT INTO outbox (topic, key, payload) VALUES ($1, $2, $3::jsonb)")
        .bind(event.topic())
        .bind(event.key())
        .bind(event.payload())
        .execute(&mut *tx)
        .await?;
}
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 280" role="img" aria-label="Hình 3. Hai ca của cùng bốn bước: commit thành công thì event bền trong outbox; commit lỗi thì rollback kéo theo cả dòng outbox, không event nào ra ngoài."><defs><marker id="rs52-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><text x="20" y="24" class="t-b">Commit thành công</text><rect x="20" y="36" width="185" height="70" rx="10" class="f-phys-soft s-phys"/><text x="34" y="62" class="t-b">handler</text><text x="34" y="86" class="t-small">aggregate ghi event</text><path d="M205 71 L235 71" class="f-none s-phys" stroke-width="2" marker-end="url(#rs52-2-a)"/><rect x="235" y="36" width="185" height="70" rx="10" class="f-phys-soft s-phys"/><text x="249" y="62" class="t-b">thu về</text><text x="249" y="86" class="t-small">ctx.take_events()</text><path d="M420 71 L450 71" class="f-none s-phys" stroke-width="2" marker-end="url(#rs52-2-a)"/><rect x="450" y="36" width="185" height="70" rx="10" class="f-phys-soft s-phys"/><text x="464" y="62" class="t-b">ghi outbox</text><text x="464" y="86" class="t-small">INSERT cùng transaction</text><path d="M635 71 L665 71" class="f-none s-phys" stroke-width="2" marker-end="url(#rs52-2-a)"/><rect x="665" y="36" width="185" height="70" rx="10" class="f-pk-soft s-pk"/><text x="679" y="62" class="t-b">COMMIT</text><text x="679" y="86" class="t-small">event bền, relay gửi sau</text><text x="20" y="144" class="t-b">Commit lỗi (trigger 23514)</text><rect x="20" y="156" width="185" height="70" rx="10" class="f-phys-soft s-phys"/><text x="34" y="182" class="t-b">handler</text><text x="34" y="206" class="t-small">aggregate ghi event</text><path d="M205 191 L235 191" class="f-none s-phys" stroke-width="2" marker-end="url(#rs52-2-a)"/><rect x="235" y="156" width="185" height="70" rx="10" class="f-phys-soft s-phys"/><text x="249" y="182" class="t-b">thu về</text><text x="249" y="206" class="t-small">ctx.take_events()</text><path d="M420 191 L450 191" class="f-none s-phys" stroke-width="2" marker-end="url(#rs52-2-a)"/><rect x="450" y="156" width="185" height="70" rx="10" class="f-phys-soft s-phys"/><text x="464" y="182" class="t-b">ghi outbox</text><text x="464" y="206" class="t-small">INSERT cùng transaction</text><path d="M635 191 L665 191" class="f-none s-phys" stroke-width="2" marker-end="url(#rs52-2-a)"/><rect x="665" y="156" width="185" height="70" rx="10" class="f-warn-soft s-warn"/><text x="679" y="182" class="t-b">COMMIT lỗi</text><text x="679" y="206" class="t-small">rollback, mất outbox</text><text x="20" y="258" class="t-small">Pipeline sai phát event ở bước "thu về": ca dưới đã đẩy ra ngoài 1 event về JE-000005 không tồn tại.</text></svg>
</div>
<figcaption>Hình 3. Cùng bốn bước, hai kết cục. Event chỉ ra khỏi process qua outbox sau COMMIT; trước đó chỉ là dữ liệu trong cùng transaction.</figcaption>
</figure>

Phản chứng nằm ở `NaivePublish`, lớp chỉ để đối chứng: đặt bên trong `Transactional`, phát sự kiện ngay khi handler xong, tức trước commit. Cùng lệnh bút toán lệch chạy qua pipeline đó:

```text
handler: đã ghi 2 dòng cho JE-000005, tổng -100
naive: phát 1 sự kiện ra ngoài (commit chưa chạy)
tx: outbox +0 dòng
tx: commit lỗi (SQLSTATE 23514), PostgreSQL đã rollback
trace: kết thúc PostRawEntry lỗi
(DB: 1 bút toán, 1 dòng outbox)
sự kiện đã phát ra ngoài: 1 (entryId JE-000005)
```

Consumer nhận `LedgerEntryPosted` về `JE-000005` trong khi sổ không có bút toán đó. Pipeline chuẩn trong cùng hoàn cảnh để outbox có đúng một dòng, của `TN-0001`, `published=false` vì lab không chạy relay:

```text
ledger.entry-posted key=trip:TN-0001:capture published=false
{"entryId":"JE-000001","idempotencyKey":"trip:TN-0001:capture","postedAt":"2026-11-06T02:20:00Z","postings":[{"account":"customer:CU-001","amountDong":-25500},{"account":"platform:revenue","amountDong":5100},{"account":"driver:DR-001","amountDong":20400}],"schemaVersion":1}
```

Payload có ba dòng vì lab dùng `capture_entry` của `ledger-core`, dạng rút gọn của bài 20; dạng chuẩn hai bước qua clearing là của bài 13 và 51. `postedAt` lấy từ clock đưa vào `Ctx`, nên payload lặp lại được từng byte. Bài C# nhắc rằng mediator không phải message broker; ở đây cũng vậy: `ctx.events` là RAM, độ bền đến từ dòng outbox, việc gửi lên Kafka là của relay ở bài 37.

## `async fn` trong trait và `Box`: tốn gì, cần ở đâu?

Đo bằng đếm, không bằng thời gian: kích thước future lấy bằng `size_of_val` trước khi poll, poll tay bằng `Waker::noop()`. Số lần cấp phát đếm bằng allocator bọc `System` (`src/counting.rs`). Đó là nơi duy nhất có `unsafe`, vì `GlobalAlloc` là unsafe trait: code chỉ chuyển tiếp nguyên lời gọi sang `System` và đếm vào biến thread-local không cấp phát, nên hợp đồng của `System` vẫn giữ. Debug và release cho cùng số ở bảng này.

| Đường gọi | Kích thước future | Cấp phát trong 1.000 lần gọi |
|---|---|---|
| `Echo` trần | 24 byte | 0 |
| `Validated<Echo>` | 56 byte | 0 |
| `Validated<Traced<Echo>>` | 88 byte | 0 |
| `DynBus::send` (handler đóng hộp) | 32 byte, cộng future đã `Box` | 1.000 (đúng một `Box` mỗi lần) |

`Pin<Box<dyn Future>>` là con trỏ béo 16 byte, còn future nằm trên heap. Chuỗi generic không cấp phát ở đường đo này vì trình biên dịch biết toàn bộ kiểu. `Box` cần ở chỗ phải xoá kiểu, ở đây là cất các handler khác kiểu vào một `HashMap`. `async fn` trong trait không dùng được sau `dyn` (E0038, bài 26), nên `DynBus` dùng closure trả future đã đóng hộp. Tổ hợp lớp bằng generic không cần `Box`.

Future của `CapturePayment` thì khác. Qua pipeline thật là 3.248 byte ở bản debug và 9.344 byte ở bản release; hàm thẳng `capture_direct` (mục sau) là 2.952 và 9.048. Hiệu hai bản đều là 296 byte, nhưng hai đường có biến cục bộ khác nhau, nên đó là chênh lệch giữa hai cách viết, không phải giá riêng của một lớp. Phần lớn kích thước là phần chung của hai đường, trong đó có future của sqlx; lab không truy nguyên vì sao release lớn hơn debug. Số tuyệt đối đổi theo bản build và rustc (1.98.1, máy viết bài, không phải SLA). Lab không đo thời gian.

## So với mediator của loạt C#, và khi nào một hàm thẳng là đủ

[Bài 32 của loạt C#](/articles/dotnet10-cqrs-mediator-pipeline) tự viết một dispatcher nhỏ để dạy pattern, không dùng thư viện MediatR và không hứa thay thế nó. Vì vậy bảng dưới chỉ đối chiếu với dispatcher đó; bài này không nói gì về MediatR:

| Việc | Bài C# | Lab này |
|---|---|---|
| Gắn kiểu kết quả | `IRequest<TResponse>`, `IRequestHandler<TRequest,TResponse>` | `Command::Output`, `CommandHandler<C>` |
| Tìm handler | `GetRequiredService` trong `SendAsync`, lúc gọi | impl trait, trình biên dịch tìm lúc build |
| Xếp pipeline | behavior lấy từ DI, vòng `Reverse` dựng delegate | struct lồng nhau, thứ tự là kiểu |
| Telemetry và validation | telemetry bọc ngoài validation | validate ngoài mặc định, đổi một dòng |
| Transaction | không bao mọi query; commit qua UoW | một lớp `Transactional` cho mọi command của lab |

Bài đó không dựng kịch bản thiếu đăng ký handler, nên ở đây không khẳng định thông điệp lỗi của container; theo code, việc tra cứu nằm trong `SendAsync`, nên lỗi lộ muộn nhất là lúc lời gọi chạy. Lab này chưa có phía đọc.

Lớp này tốn gì: ba trait (`Command`, `CommandHandler`, `Validate`), một kiểu `Ctx`, ba lớp generic, tên kiểu dài, và `Ctx` là ngữ cảnh chứa transaction mà handler lấy ra dùng, một phụ thuộc ngầm mà kiểu không ép được. So với một hàm thẳng `capture_direct` (`src/direct.rs`, 68 dòng cả chú thích): validate, `begin`, ghi, thu sự kiện, ghi outbox, `commit` trong một thân hàm, lỗi thoát bằng `?`. Test `direct_function_matches_pipeline` chạy hai bản trên hai schema riêng và khẳng định cùng biên lai, cùng payload outbox. Hàm thẳng đủ khi chỉ có một hoặc hai use case ghi. Pipeline đáng dùng khi nhiều command cùng cần khung `begin`, outbox, `commit`, vì mỗi hàm thẳng sẽ chép lại khung đó và có bản sẽ lệch đi; hoặc khi cần ghép lớp khác nhau theo loại command.

## Chạy ví dụ và kiểm chứng

PostgreSQL của `infra/` phải đang chạy. Từ thư mục gốc repository:

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
cargo run --manifest-path examples/toi-noi-rust/52-command-pipeline/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/52-command-pipeline/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/52-command-pipeline/Cargo.toml
```

Lệnh đầu in năm phần ứng với năm mục trên. `--verify` kiểm mọi giá trị bài nêu (log từng ca, SQLSTATE `23503` và `23514`, số dòng sau mỗi ca, payload outbox, 0 cấp phát ở đường generic, đúng 1.000 ở `DynBus`) rồi in `VERIFY OK`; thiếu `TOI_NOI_PG_URL` thì thoát mã 1. `cargo test` chạy 7 test (3 test cần PostgreSQL, schema `lab_<uuid>` riêng, dọn cuối test case) cùng 3 doc test, trong đó 2 là `compile_fail`. Một test cho bốn lệnh cùng key chạy song song trên runtime nhiều thread qua `tokio::spawn`, chứng minh future của pipeline là `Send`, không lệnh nào lỗi và chỉ có một bút toán. Không test nào assert thời gian.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Lab không có relay: dòng outbox ở lại <code>published=false</code> (gửi Kafka là bài 37). Chỉ một loại sự kiện, chưa versioning. Replay chỉ so theo key, không so nội dung như bài 24 và 26; <code>account_balances</code> không được cập nhật. Chưa có phía đọc, kịch bản huỷ giữa chừng, timeout hay thử lại khi xung đột (bài 53). <code>PostRawEntry</code>, <code>NaivePublish</code> chỉ để dựng kịch bản hỏng; <code>DynBus</code> không nối với pipeline thật.</p>
</div>

**Bài tập:** viết `impl CommandHandler<VoidHold> for LedgerHandlers` trả `Ok(())`. Bài kiểm nào của lab chuyển đỏ, bài nào vẫn xanh, và vì sao đỏ ở đây là tin tốt?

<details><summary>Xem lời giải</summary>
<p>Pipeline giờ biên dịch được với <code>VoidHold</code>, nên doc test <code>compile_fail,E0277</code> ở <code>src/lib.rs</code> đỏ với thông điệp "Test compiled successfully, but it's marked <code>compile_fail</code>" (đã chạy thử trên lab). Cả 7 test tích hợp vẫn xanh, kể cả <code>dyn_bus_reports_missing_handler_at_runtime</code>, vì <code>DynBus</code> có bảng đăng ký riêng. Đỏ là tin tốt: doc test thật sự kiểm được "thiếu handler thì không biên dịch"; việc cần làm là đổi nó sang command khác chưa có handler.</p>
</details>

Bài trước: [bài 51](/articles/rust-hexagonal-clean-workspace-crates-port-adapter). Bài tiếp: [bài 53](/articles/rust-aggregate-invariant-optimistic-locking-for-update).

## Tài liệu gốc

- [The Rust Reference: Associated items](https://doc.rust-lang.org/reference/items/associated-items.html).
- [The Rust Reference: Dyn compatibility](https://doc.rust-lang.org/reference/items/traits.html#dyn-compatibility).
- [std::future::Future](https://doc.rust-lang.org/std/future/trait.Future.html).
- [std::alloc::GlobalAlloc](https://doc.rust-lang.org/std/alloc/trait.GlobalAlloc.html).
- [sqlx::Transaction](https://docs.rs/sqlx/latest/sqlx/struct.Transaction.html).
- [PostgreSQL: CREATE TRIGGER (constraint trigger hoãn tới COMMIT)](https://www.postgresql.org/docs/current/sql-createtrigger.html).
