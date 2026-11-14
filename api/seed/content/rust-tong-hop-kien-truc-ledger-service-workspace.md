---
slug: rust-tong-hop-kien-truc-ledger-service-workspace
title: "Bài tổng hợp kiến trúc: ledger service hoàn chỉnh trong workspace"
summary: "Bài 58: ledger-service là workspace năm crate chạy thật với PostgreSQL và Kafka. Chiều phụ thuộc kiểm bằng cargo tree, luồng TripCompleted đến số dư qua HTTP và gRPC, hợp đồng với C#, bảng P03–P06 và những gì chưa có."
category: rust-advanced
level: advanced
tags: [rust, toi-noi, ledger, hexagonal, workspace, end-to-end]
language: vi
---

Một kỹ sư mới nhận service sổ cái và hỏi ba câu. Crate nào được phép biết PostgreSQL? Ghi bút toán thủ công đi đường nào, ai được gọi? Còn thiếu gì trước khi lên môi trường thật? [Bài 40](/articles/rust-tong-hop-ledger-api-postgresql-kafka-end-to-end) trả lời một phần: service nằm trong một crate, chưa có `POST /v1/entries`, JWT, hold hay hoàn tiền. Mười bảy bài sau thêm từng mảnh. Bài này lắp chúng thành workspace năm crate và biến câu hỏi khi review thành lệnh chạy được.

Đây là bài 58 của lộ trình **Rust nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Lab dùng lại `ledger-core` của [bài 20](/articles/rust-tong-hop-ledger-core-cargo-doc-workspace) qua path dependency, và chỉ có path dependency đó; phần cần dùng của các lab 25, 32, 33, 35, 36, 37, 39, 40 được chép tối thiểu vào.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Một PR thêm <code>use tokio::time::sleep;</code> vào crate <code>domain</code> và thêm <code>tokio</code> vào <code>Cargo.toml</code> của nó để biên dịch được. Người review có thể bỏ sót dòng đó. Cái gì trong repo chắc chắn bắt được?</p>
<details><summary>Xem đáp án</summary><p>Luật của <code>ledger-domain</code> trong <code>arch::check_all</code>: nó chạy <code>cargo tree -p ledger-domain</code>, thấy <code>tokio</code> trong cây và làm <code>-- --verify</code> thoát mã 1 với thông điệp <code>ledger-domain kéo theo ["tokio"]</code>, đồng thời test <code>dependency_direction_holds</code> đỏ.</p></details>
</div>

## Bản đồ crate: ai được biết ai?

Hình 1 vẽ các cạnh phụ thuộc; mũi tên đi từ crate biết sang crate được biết. Điều đáng chú ý: không có cạnh nào giữa `api` và `infrastructure`, và chỉ binary `ledger-service` nối được cả hai.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 320" role="img" aria-label="Hình 1. Binary ledger-service phụ thuộc ledger-api và ledger-infrastructure; hai crate này cùng phụ thuộc ledger-application, rồi ledger-domain, rồi ledger-core của lab 20. Giữa ledger-api và ledger-infrastructure không có cạnh nào."><defs><marker id="rust-tong-hop-kien-truc-ledger-service-workspace-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="230" y="16" width="420" height="58" rx="8" class="f-warn-soft s-warn"/><text x="244" y="40" class="t-b">ledger-service (binary, composition root)</text><text x="244" y="60" class="t-small">lắp adapter vào port, chạy 4 task, tắt có thứ tự</text><rect x="40" y="112" width="300" height="62" rx="8" class="f-pk-soft s-pk"/><text x="54" y="136" class="t-b">ledger-api</text><text x="54" y="156" class="t-small">axum + tonic, JWT, Problem Details</text><rect x="540" y="112" width="300" height="62" rx="8" class="f-pk-soft s-pk"/><text x="554" y="136" class="t-b">ledger-infrastructure</text><text x="554" y="156" class="t-small">sqlx, rdkafka, outbox relay</text><rect x="290" y="190" width="300" height="50" rx="8" class="f-phys-soft s-phys"/><text x="304" y="212" class="t-b">ledger-application</text><text x="304" y="230" class="t-small">Ledger&lt;S&gt; và port LedgerStore</text><rect x="290" y="266" width="300" height="46" rx="8" class="f-phys-soft s-phys"/><text x="304" y="287" class="t-b">ledger-domain</text><text x="304" y="305" class="t-small">luật thuần: chuyến, hold, hoàn tiền</text><rect x="660" y="266" width="190" height="46" rx="8" class="f-none s-phys dash"/><text x="674" y="287" class="t-b">ledger-core</text><text x="674" y="305" class="t-small">lab 20, không sửa</text><path d="M300 74 L200 108" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-tong-hop-kien-truc-ledger-service-workspace-0-a)"/><path d="M580 74 L680 108" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-tong-hop-kien-truc-ledger-service-workspace-0-a)"/><path d="M190 174 L330 188" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-tong-hop-kien-truc-ledger-service-workspace-0-a)"/><path d="M690 174 L550 188" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-tong-hop-kien-truc-ledger-service-workspace-0-a)"/><path d="M440 240 L440 264" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-tong-hop-kien-truc-ledger-service-workspace-0-a)"/><path d="M590 289 L656 289" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-tong-hop-kien-truc-ledger-service-workspace-0-a)"/><path d="M344 143 L536 143" class="f-none s-warn dash" stroke-width="2"/><text x="380" y="134" class="t-small">không có cạnh nào</text><text x="370" y="164" class="t-small">kiểm bằng cargo tree</text></svg>
</div>
<figcaption>Hình 1. Chiều phụ thuộc của lab 58. Đường đứt giữa <code>api</code> và <code>infrastructure</code> là điều tiêu chí review kiểm; <code>ledger-service</code> khai báo cả bốn crate còn lại; <code>api</code> và <code>infrastructure</code> cũng khai báo trực tiếp <code>domain</code>.</figcaption>
</figure>

| Crate | Vai trò | Dependency trực tiếp (cargo tree, độ sâu 1) | Crate trong cây | Dòng Rust |
| --- | --- | --- | ---: | ---: |
| `ledger-domain` | chuyến thành hai bút toán, hold, hoàn tiền, sơ đồ tài khoản, `Clock` | `jiff`, `ledger-core`, `thiserror` | 16 | 786 |
| `ledger-application` | use case `Ledger<S>`, port `LedgerStore`, lỗi, `Readiness` | `jiff`, `ledger-domain`, `thiserror` | 17 | 583 |
| `ledger-infrastructure` | `PgStore`, consumer, relay, JSON trên dây | `sqlx`, `rdkafka`, `tokio`, `serde_json`, `tracing`… | 137 | 1.128 |
| `ledger-api` | handler axum, service tonic, JWT, Problem Details | `axum`, `tonic`, `jsonwebtoken`, `tower-http`… | 145 | 1.381 |
| `ledger-service` | composition root, harness, kịch bản kiểm | cả bốn crate trên | không tính | 1.912 |

Cột «crate trong cây» đếm cả chính crate đó; cột cuối gồm test nhỏ trong `src`. Số đo trên máy viết bài.

Điểm khác so với [bài 51](/articles/rust-hexagonal-clean-workspace-crates-port-adapter): ở đó `api` giữ luôn vai trò composition root nên phụ thuộc `infrastructure`, và test đọc `cargo metadata`. Ở đây root là binary riêng, nên `api` không có đường nào tới `sqlx`, và luật «API không biết dữ liệu nằm đâu» kiểm được bằng một lệnh. Cái giá phải trả là thêm một crate.

Port `LedgerStore` trả `impl Future<Output = …> + Send` thay vì `async fn`, nên `application` chỉ cần thư viện chuẩn, không `tokio`, không `async_trait`; adapter vẫn viết được `async fn`. Cái giá phải trả: `Ledger<S>` là kiểu generic theo `S`, tham số này lan tới `ApiState<S>` và từng handler, và trait không dùng được dưới dạng `dyn`.

## Tiêu chí review kiến trúc có chạy được không?

Mỗi tiêu chí dưới đây có cột «chạy bằng»; dòng nào chỉ người kiểm được thì ghi thẳng như vậy.

| Tiêu chí | Chạy bằng | Chứng cứ |
| --- | --- | --- |
| `domain` không kéo runtime, I/O hay tầng ngoài | `arch::check_all` (cargo tree) | 12 crate bị cấm, cây 16 crate sạch |
| `application` chỉ biết `domain` | `arch::check_all` | 10 crate bị cấm, cây 17 crate sạch |
| `infrastructure` không biết `api`; `api` không biết `sqlx`, `rdkafka` | `arch::check_all` | luật số 3 và 4 |
| Bộ kiểm không mù | phép đối chứng trong `check_all` | luật của `domain` áp lên `ledger-service` bắt 12 crate |
| Domain không đọc đồng hồ thật | `cargo clippy -D warnings` với `clippy.toml` | `Timestamp::now` bị cấm; chỉ `SystemClock` và `now_secs` (cấp token lab) được `allow` |
| Mọi lỗi có mã HTTP và gRPC | trình biên dịch | `match` vét cạn, không nhánh `_` ở `error_map.rs`, `grpc.rs` |
| Use case chạy không cần DB | test của `application` với adapter giả | 4 test, `tokio` chỉ là dev-dependency |
| Sổ không lệch, không sửa được kể cả khi bỏ qua ứng dụng | kịch bản `database_guards` | `23514` và `TN001` từ PostgreSQL |
| Số dư khoá theo thứ tự mã, không theo thứ tự dòng | kịch bản `balance_lock_order` | đảo thứ tự trong `apply_balances` thì đỏ với `55P03` |
| Lỗi nội bộ và khoá JWT không lộ ra ngoài | test `internal_errors_do_not_leak_details`, kịch bản SIGTERM | không dòng log nào chứa khoá |
| Handler chỉ giải mã, gọi use case, mã hoá | người đọc code | không máy nào kiểm |

Lõi bộ kiểm nằm ở `crates/ledger-service/src/arch.rs`. Mỗi luật là dữ liệu; việc chạy `cargo tree` và việc so tên là hai hàm riêng, nên có thể test riêng việc so tên mà không cần cargo:

```rust
// crates/ledger-service/src/arch.rs (rút gọn)
pub struct Rule {
    pub package: &'static str,
    pub forbidden: &'static [&'static str],
    pub why: &'static str,
}

Rule {
    package: "ledger-api",
    forbidden: &["sqlx", "rdkafka", "ledger-infrastructure"],
    why: "API gọi use case; nó không được biết dữ liệu nằm ở PostgreSQL hay Kafka",
},

pub fn violations(rule: &Rule, tree_output: &str) -> Vec<&'static str> {
    let present = crates_in_tree(tree_output);
    rule.forbidden
        .iter()
        .copied()
        .filter(|name| present.contains(*name))
        .collect()
}
```

Phép đối chứng cần thiết vì bộ kiểm luôn báo «sạch» (do parse sai đầu ra hoặc do cargo đổi định dạng) trông giống hệt bộ kiểm đúng. `check_all` áp luật của `domain` lên cây của `ledger-service`, nơi `tokio`, `sqlx`, `axum` có thật, và đòi bắt ít nhất tám trong 12 crate bị cấm. Lần chạy thật bắt cả 12.

Thử phá thật: thêm `tokio.workspace = true` vào `crates/domain/Cargo.toml` rồi chạy `-- --verify`:

```text
LỖI: kiểm chiều phụ thuộc: ledger-domain kéo theo ["tokio"]: domain là luật nghiệp vụ thuần: không runtime, không I/O, không biết tầng ngoài
```

Việc đọc đồng hồ thì `cargo tree` không thấy được, vì `jiff` vốn là dependency hợp lệ của `domain`. Phần này do clippy: `clippy.toml` ở gốc workspace cấm `jiff::Timestamp::now` và `std::time::SystemTime::now`. Thêm `Timestamp::now() >= hold.expires_at` vào `hold.rs` cho ra `error: use of a disallowed method jiff::Timestamp::now`, kèm ghi chú «dùng Clock truyền vào; chỉ SystemClock được gọi hàm này».

Giới hạn: bộ kiểm đọc các cạnh đã khai báo, không đọc ý định. Logic nghiệp vụ nằm nhầm trong handler HTTP vẫn qua mọi luật, nên dòng cuối của bảng vẫn cần người.

## Một TripCompleted đi qua những lớp nào?

Ba cổng vào (consumer, HTTP, gRPC) giải mã dữ liệu của mình thành lệnh thuần rồi gọi cùng `Ledger<S>`. Use case dựng hai bút toán bằng domain và giao cho port; mọi thứ cần ghi nằm trong một transaction của `PgStore` (Hình 2).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 320" role="img" aria-label="Hình 2. Consumer, HTTP và gRPC giải mã dữ liệu thành lệnh rồi gọi use case Ledger. Use case gọi port LedgerStore; PgStore ghi inbox, bút toán capture và split kèm outbox, payments và hold, số dư trong một transaction. Relay phát outbox lên topic entry-posted."><defs><marker id="rust-tong-hop-kien-truc-ledger-service-workspace-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="16" y="24" width="210" height="56" rx="8" class="f-pk-soft s-pk"/><text x="30" y="48" class="t-b">Consumer (infra)</text><text x="30" y="68" class="t-small">TripCompleted JSON → lệnh</text><rect x="16" y="104" width="210" height="56" rx="8" class="f-pk-soft s-pk"/><text x="30" y="128" class="t-b">HTTP (api)</text><text x="30" y="148" class="t-small">JWT → DTO → lệnh</text><rect x="16" y="184" width="210" height="56" rx="8" class="f-pk-soft s-pk"/><text x="30" y="208" class="t-b">gRPC (api)</text><text x="30" y="228" class="t-small">interceptor JWT → lệnh</text><rect x="286" y="80" width="190" height="112" rx="8" class="f-phys-soft s-phys"/><text x="300" y="106" class="t-b">Ledger&lt;S&gt;</text><text x="300" y="130" class="t-small">TripFare, plan_capture</text><text x="300" y="150" class="t-small">đồng hồ truyền vào</text><text x="300" y="170" class="t-small">(application, domain)</text><rect x="536" y="16" width="328" height="216" rx="10" class="f-warn-soft s-warn"/><text x="550" y="42" class="t-b">PgStore: một transaction</text><text x="550" y="70" class="t-small">1. inbox: INSERT event_id</text><text x="550" y="92" class="t-small">2. capture: khách −, clearing +</text><text x="550" y="114" class="t-small">3. split: clearing −, revenue +, driver +</text><text x="550" y="136" class="t-small">    mỗi bút toán kèm một dòng outbox</text><text x="550" y="158" class="t-small">4. payments và đóng hold của chuyến</text><text x="550" y="180" class="t-small">5. account_balances, theo thứ tự mã</text><text x="550" y="202" class="t-small">6. COMMIT</text><rect x="536" y="256" width="150" height="52" rx="8" class="f-pk-soft s-pk"/><text x="550" y="278" class="t-b">Relay</text><text x="550" y="296" class="t-small">outbox → Kafka</text><rect x="714" y="256" width="150" height="52" rx="8" class="f-phys-soft s-phys"/><text x="728" y="278" class="t-b">entry-posted</text><text x="728" y="296" class="t-small">key idempotencyKey</text><path d="M226 52 L282 112" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-tong-hop-kien-truc-ledger-service-workspace-1-a)"/><path d="M226 132 L282 136" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-tong-hop-kien-truc-ledger-service-workspace-1-a)"/><path d="M226 212 L282 160" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-tong-hop-kien-truc-ledger-service-workspace-1-a)"/><path d="M476 136 L532 136" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-tong-hop-kien-truc-ledger-service-workspace-1-a)"/><text x="488" y="126" class="t-small">port</text><path d="M610 232 L610 252" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-tong-hop-kien-truc-ledger-service-workspace-1-a)"/><path d="M686 282 L710 282" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-tong-hop-kien-truc-ledger-service-workspace-1-a)"/><text x="16" y="276" class="t-small">Adapter chỉ giải mã và gọi use case.</text><text x="16" y="296" class="t-small">Mọi luật nằm ở application và domain.</text></svg>
</div>
<figcaption>Hình 2. Một <code>TripCompleted</code> đi qua ba lớp. Khung lớn là phần phải nguyên khối: hai bút toán, số dư và hai dòng outbox cùng có hoặc cùng không.</figcaption>
</figure>

Phía use case chỉ có vài dòng, vì luật đã ở domain:

```rust
// crates/application/src/service.rs (rút gọn: bỏ chú thích)
pub async fn on_trip_completed(&self, cmd: &TripCompletedCmd) -> Result<Applied, Error> {
    let trip = TripFare::new(
        &cmd.trip_id,
        &cmd.customer_id,
        &cmd.driver_id,
        cmd.fare_dong,
    )?;
    let plan = plan_capture(&trip, self.fee.as_ref())?;
    self.store
        .apply_trip(&cmd.event_id, &trip, &plan, self.clock.now())
        .await
}
```

`plan_capture` của `domain` dựng hai bút toán của [bài 41](/articles/rust-bai-toan-so-cai-chart-of-accounts-dong-tien-chuyen): `trip:TN-0001:capture` (khách -25.500, clearing +25.500) và `trip:TN-0001:split` (clearing -25.500, doanh thu +5.100, tài xế +20.400). Sau bước chia, `psp:vixanh:clearing` về đúng 0, và kịch bản kiểm điều đó qua cả HTTP lẫn gRPC.

Adapter ghép các việc vào một transaction. Đoạn dưới rút gọn từ `PgStore::apply_trip`:

```rust
// crates/infrastructure/src/pg.rs (rút gọn)
let mut deltas = BTreeMap::new();
let mut created = Vec::new();
for entry in [&plan.capture, &plan.split] {
    if let Written::Created(posted) =
        write_entry(&mut tx, entry, &self.topic_out, &mut deltas).await?
    {
        created.push(posted);
    }
}
// payments, rồi đóng hold của chuyến nếu có
apply_balances(&mut tx, &deltas).await?;
tx.commit().await.db()?;
```

`write_entry` chỉ gom thay đổi số dư vào `deltas`; `apply_balances` cập nhật `account_balances` một lần ở cuối, theo thứ tự mã tài khoản. Một bút toán thủ công chạm `platform:revenue` rồi `psp:vixanh:clearing`, còn bước chia chạm clearing rồi revenue; khoá theo hai thứ tự khác nhau là công thức dẫn đến deadlock. Kịch bản `balance_lock_order` kiểm thứ tự này: một phiên giữ khoá `platform:revenue`, bút toán ghi clearing trước revenue bị chặn (thấy qua `pg_blocking_pids`), trong khi `FOR UPDATE NOWAIT` trên clearing vẫn qua. Đảo thứ tự trong `apply_balances` thì kịch bản đỏ với `55P03`. Lab không dựng deadlock thật; race condition đó được trình bày ở [bài 53](/articles/rust-aggregate-invariant-optimistic-locking-for-update).

Lắp ráp nằm ở một chỗ, trong `app.rs` của binary:

```rust
// crates/ledger-service/src/app.rs (rút gọn)
let ledger = Arc::new(Ledger::new(
    store.clone(),
    clock,
    Arc::new(default_policy()),
));
let router = http::router(ApiState {
    ledger: ledger.clone(),
    auth: auth.clone(),
    ready: readiness.clone(),
});
let service = ledger_api::grpc::service(ledger, auth);
```

HTTP và gRPC cầm cùng một `Arc<Ledger<PgStore>>`, nên không thể trả hai số khác nhau. Tắt service theo năm bước của [bài 39](/articles/rust-graceful-shutdown-health-readiness-backpressure): `/readyz` đổi sang 503, consumer xử lý xong message đang dở, relay xả outbox, đóng hai cổng, đóng pool.

## Hợp đồng với service C#: HTTP, gRPC và sự kiện

Ledger sở hữu database riêng; Booking, Payment và Admin BFF (C#) chỉ nói chuyện qua ba kênh sau.

| Kênh | Bên dùng | Hợp đồng | Bằng chứng |
| --- | --- | --- | --- |
| Kafka `toi-noi.trip.completed` | Booking (C#) phát | `TripCompleted` JSON camelCase, `schemaVersion: 1`, trường lạ bị bỏ qua; sai dạng hoặc phiên bản khác 1 thì ghi log và bỏ qua | `flow_and_replay` |
| Kafka `toi-noi.ledger.entry-posted` | Payment, Admin đọc | `LedgerEntryPosted`, key `idempotencyKey`; **mỗi bút toán một sự kiện**, nên một chuyến ra hai sự kiện | `flow_and_replay` |
| HTTP `/v1` | Admin BFF | `POST /v1/entries`, `GET …/balance`, `GET …/available`, `POST /v1/holds`, `POST /v1/trips/{id}/refunds`; lỗi `application/problem+json` | `http_and_grpc_contract`, `hold_and_refund` |
| gRPC `ledger.v1` | Payment | `GetBalance`, `PostEntry` theo `ledger.proto` của bài 35, giữ nguyên | `http_and_grpc_contract` |

Hai thay đổi so với bài 40 mà service C# phải biết. **Một chuyến ra hai sự kiện thay vì một**: `trip:<id>:capture` chỉ còn khách và clearing, doanh thu và tài xế chuyển sang `trip:<id>:split`. Consumer đọc thu nhập tài xế từ sự kiện capture sẽ không thấy gì, consumer đếm «một chuyến, một sự kiện» sẽ sai. Khi topic có nhiều partition, hai key khác nhau có thể rơi vào hai partition nên không được giả định capture đến trước split ([bài 36](/articles/rust-kafka-rdkafka-consumer-producer-schema)). Dù sao cũng phải khử trùng lặp theo `idempotencyKey`, vì relay là at-least-once.

**Tên topic mặc định cũng đổi**: bài 40 dùng tiền tố `ledger` (`ledger.trip.completed`, `ledger.ledger.entry-posted`), còn ở đây `LEDGER_TOPIC_PREFIX` mặc định là `toi-noi`. Service giữ tên cũ không nhận lỗi nào, chỉ không còn nhận được sự kiện. Trên hệ đang chạy, đổi tên phải theo ba bước của [bài 55](/articles/rust-hop-dong-su-kien-xuyen-ngon-ngu-protobuf-version): ghi song song, chuyển consumer, dọn topic cũ; lab không dựng các bước đó, và tên này chưa có hậu tố `.v1` mà bài 55 nêu là quy ước.

Xác thực chung cho HTTP và gRPC: Bearer JWT ký HS256, `aud` là `ledger-api`, bắt buộc `exp`, `aud`, `sub`, khoá dài tối thiểu 32 byte và không được ghi vào log. Vai trò `finance` ghi và đọc, `cs` chỉ đọc. Kiểm quyền nằm ở extractor `Authorized<P>` và interceptor của tonic, nên lỗi 401, 403 trả về trước lỗi đọc body. Các lỗi được ánh xạ như sau, cả hai phía đều là `match` vét cạn:

| Lỗi | HTTP | gRPC |
| --- | --- | --- |
| thiếu hoặc sai token | 401 + `WWW-Authenticate` | `Unauthenticated` |
| thiếu vai trò | 403 | `PermissionDenied` |
| dữ liệu sai, bút toán lệch, tài khoản ngoài sơ đồ | 422 | `InvalidArgument` |
| cùng key khác nội dung | 409 | `AlreadyExists` |
| không có tài khoản hoặc chuyến | 404 | `NotFound` |
| không đủ khả dụng, hoàn vượt mức | 422 | `FailedPrecondition` |
| JSON hỏng | 400 | không áp dụng |
| hạ tầng tạm thời | 503 | `Unavailable` |
| lỗi lưu trữ | 500, không kèm chi tiết | `Internal` |

Bút toán ghi bằng HTTP rồi gửi lại bằng gRPC với cùng key sẽ trả về cùng `JE-000001` với `replayed = true`. Client .NET của [bài 35](/articles/rust-tonic-grpc-get-balance-post-entry-interop-dotnet) dùng cùng `ledger.proto`, nhưng server này đòi JWT thay vì token cố định, nên client phải gửi header `authorization` mới gọi được. Tôi chưa chạy client .NET với server này.

`PaymentCaptured` có trong hợp đồng chung nhưng lab này chưa tiêu thụ: `TripCompleted` là nguồn duy nhất của bút toán chuyến.

## Hold và hoàn tiền: đường chính đi qua đâu?

**Hold** không ghi bút toán. `POST /v1/holds` khoá hàng `account_balances` của ví, cộng các hold `ACTIVE` chưa hết hạn, và từ chối nếu `balance - held` không đủ. Khi `TripCompleted` đến, `apply_trip` khoá hold `trip:<id>:hold` và đóng nó trong cùng transaction với hai bút toán. Việc hết hạn do domain quyết định, và domain nhận giờ như một đối số:

```rust
// crates/domain/src/hold.rs (rút gọn: bỏ chú thích)
pub fn settle(&self, now: Timestamp, fare: Money) -> Settlement {
    if now >= self.expires_at {
        Settlement::Expired
    } else {
        Settlement::Captured {
            captured: self.amount.min(fare),
        }
    }
}
```

Kịch bản nạp ví 100.000, giữ 30.000 (khả dụng 70.000); hold thứ hai 80.000 bị 422 `insufficient-available`. Chuyến `TN-0001` xong: hold `CAPTURED` 25.500, ví còn 74.500. Giữ thêm 30.000, cho `ManualClock` chạy thêm 901 giây thì hold đó không còn bị trừ vào số khả dụng, không `sleep`; chuyến `TN-0002` đóng nó thành `EXPIRED`.

**Hoàn tiền một phần** khoá dòng `payments` của chuyến, kiểm phần còn hoàn được rồi ghi bút toán `trip:TN-0001:refund:<n>`: khách nhận lại, nền tảng hoàn phần phí theo đúng tỷ lệ lúc thu, tài xế hoàn phần còn lại. Hoàn 10.000 là khách +10.000, doanh thu -2.000, tài xế -8.000; hoàn nốt 15.500 đưa `payments` sang `REFUNDED`, và phần của `TN-0001` ở doanh thu cùng ví tài xế về đúng 0. Phần phí mỗi lần là hiệu của `fee_target` sau và trước lần đó, tính theo tổng luỹ kế như [bài 43](/articles/rust-refund-reversal-chargeback-but-toan-dao-maker-checker), bằng số nguyên `i128`, nên nhiều lần hoàn nhỏ vẫn cộng đúng 5.100:

```rust
// crates/domain/src/refund.rs (rút gọn: bỏ chú thích)
fn fee_target(cumulative: i64, fee: i64, captured: i64) -> Option<i64> {
    let numerator = i128::from(cumulative) * i128::from(fee) * 2 + i128::from(captured);
    let target = numerator / (i128::from(captured) * 2);
    i64::try_from(target).ok()
}
```

Đây không phải thuật toán largest remainder của [bài 46](/articles/rust-lam-tron-chia-tien-largest-remainder-khong-mat-dong): chia hai phần thì một phần là phần dư của phần kia.

## Các bài P03–P06 đã góp gì vào service này?

Cột phải là phần lab 58 lấy hoặc bỏ; phần bỏ quan trọng không kém phần lấy.

| Bài | Bài đó dạy | Lab 58 lấy gì, bỏ gì |
| --- | --- | --- |
| [21](/articles/rust-async-await-tokio-task-timeout-cancellation) | `async`, `select!`, `CancellationToken` | `app.rs`: bốn `spawn`, ba token; `select!` ở consumer |
| [22](/articles/rust-send-sync-arc-mutex-channel-concurrency) | `Send`, `Sync`, `Arc`, channel | `+ Send` ở port; `Arc<Ledger<S>>` chung; atomic ở `Readiness`, `Stats` |
| [23](/articles/rust-sqlx-postgresql-pool-migration-query) | `PgPool`, migration, `query!` | `PgStore::connect`; bốn file SQL nạp bằng `raw_sql`, chưa `migrate!` |
| [24](/articles/rust-transaction-isolation-append-only-idempotency-key) | transaction, `ON CONFLICT`, append-only | `write_entry` (so nội dung khi key trùng), `03_append_only.sql`; không retry `40001` |
| [25](/articles/rust-schema-so-cai-accounts-journal-postings-partition) | bốn bảng, trigger cân bút toán | `01_schema.sql`, `02_balance_check.sql` giữ nguyên từng byte; bỏ index và partition |
| [26](/articles/rust-repository-unit-of-work-trait-transaction) | port, Unit of Work, adapter giả | `LedgerStore`; mỗi phương thức một transaction; adapter giả ở test của `application` |
| [27](/articles/rust-integration-test-postgresql-schema-rieng-ci) | schema `lab_<uuid>`, fixture | `LabEnv`: schema, topic, group, khoá JWT riêng cho mỗi test case; không có CI |
| [28](/articles/rust-redis-cache-so-du-rate-limit-lock-ttl) | cache, rate limit, lock | không dùng |
| [29](/articles/rust-config-secrets-tracing-structured-logging) | cấu hình, secret, tracing | `Config::from_lookup` gom lỗi; `Secret` che khoá; log JSON có `request_id` |
| [30](/articles/rust-criterion-benchmark-do-bo-nho-10k-but-toan) | criterion, `dhat` | không dùng, lab không đo thông lượng |
| [31](/articles/rust-axum-http-endpoint-ghi-but-toan-tra-so-du) | `Router`, extractor | router, extractor; `POST /v1/entries` có từ bản này |
| [32](/articles/rust-validation-problem-details-openapi-utoipa) | validate, Problem Details, OpenAPI | `problem.rs`, `validate.rs`, `error_map.rs`; bỏ OpenAPI |
| [33](/articles/rust-tower-middleware-timeout-request-id-jwt-rate-limit) | tower, JWT, phân quyền | request id, `Authorized<P>`, vai trò; bỏ timeout và rate limit |
| [34](/articles/rust-axum-api-test-oneshot-hop-dong-bff) | `oneshot`, hợp đồng với BFF | chỉ lấy đường server thật qua TCP; bỏ `oneshot` và snapshot OpenAPI |
| [35](/articles/rust-tonic-grpc-get-balance-post-entry-interop-dotnet) | `.proto`, `Status`, interceptor | `ledger.proto` giữ nguyên; interceptor kiểm JWT thay token cố định |
| [36](/articles/rust-kafka-rdkafka-consumer-producer-schema) | consumer, producer, commit sau khi ghi | `consumer.rs`, `kafka.rs`; một partition |
| [37](/articles/rust-outbox-inbox-idempotent-consumer) | outbox, inbox, relay | `write_entry` ghi outbox, `relay.rs`, inbox trong `apply_trip`; chưa có DLQ |
| [38](/articles/rust-webhook-psp-hmac-chong-replay-xu-ly-tre-trung) | webhook, HMAC, chống replay | không dùng |
| [39](/articles/rust-graceful-shutdown-health-readiness-backpressure) | tín hiệu, năm bước tắt, `/readyz` | `App::shutdown`, `/healthz`, `/readyz`; bỏ hàng đợi và `pause` |
| [40](/articles/rust-tong-hop-ledger-api-postgresql-kafka-end-to-end) | service một crate end-to-end | nền của lab này; mới: workspace, JWT, hai bút toán, hold, hoàn tiền |
| [41](/articles/rust-bai-toan-so-cai-chart-of-accounts-dong-tien-chuyen) | sơ đồ tài khoản, dòng hai bước | `chart.rs` tám mẫu; `plan_capture` hai bước; clearing về 0 |
| [42](/articles/rust-hold-capture-void-so-du-kha-dung) | hold, capture, void | `hold.rs`, `place_hold`, capture một phần qua `settle`; bỏ void và job hết hạn |
| [43](/articles/rust-refund-reversal-chargeback-but-toan-dao-maker-checker) | hoàn tiền, đảo, chargeback | `refund.rs`, khoá dòng `payments`, làm tròn luỹ kế; bỏ đảo toàn phần, chargeback, maker-checker |
| [44](/articles/rust-vi-tai-xe-chia-tien-payout-theo-lo) | ví tài xế, payout | không dùng |
| [45](/articles/rust-khuyen-mai-tro-gia-but-toan-nhieu-ben-ngan-sach) | khuyến mãi, ngân sách | không dùng |
| [46](/articles/rust-lam-tron-chia-tien-largest-remainder-khong-mat-dong) | largest remainder | không dùng; `fee_target` chỉ chia hai phần |
| [47](/articles/rust-doi-soat-psp-sao-ke-csv-khop-lech-bao-cao) | đối soát sao kê | không dùng |
| [48](/articles/rust-so-du-theo-thoi-diem-snapshot-dong-ky) | snapshot, đóng kỳ | không dùng |
| [49](/articles/rust-event-sourcing-co-chung-muc-projection-replay) | projection, replay | `account_balances` là projection cùng transaction; không replay |
| [50](/articles/rust-audit-append-only-hash-chain-phan-quyen-pii) | `REVOKE`, hash chain, PII | không dùng |
| [51](/articles/rust-hexagonal-clean-workspace-crates-port-adapter) | workspace hexagonal | cấu trúc crate và luật phụ thuộc; root là binary riêng, `cargo tree` thay `cargo metadata`, bỏ feature `kafka` |
| [52](/articles/rust-command-handler-pipeline-domain-event-khong-framework) | command, pipeline, domain event | không dùng pipeline, bus hay domain event; `Ledger<S>` là các phương thức thường, trường hợp «một hàm thẳng là đủ» của bài đó |
| [53](/articles/rust-aggregate-invariant-optimistic-locking-for-update) | aggregate, `version`, `FOR UPDATE` | khoá dòng ở `place_hold`, `refund`; cột `version` chỉ tăng, không dùng optimistic lock, không retry |
| [54](/articles/rust-use-case-thanh-toan-chuyen-end-to-end) | use case thanh toán đầu-cuối | hai bút toán trong `apply_trip`; bỏ aggregate `TripPayment` và thứ tự đến tuỳ ý; không tiêu thụ `PaymentCaptured` |
| [55](/articles/rust-hop-dong-su-kien-xuyen-ngon-ngu-protobuf-version) | hợp đồng sự kiện, version | JSON camelCase `schemaVersion: 1`; không protobuf cho sự kiện, không fixture cho consumer đa ngôn ngữ, không registry |
| [56](/articles/rust-feature-flag-config-migration-expand-contract) | flag, expand/contract | không dùng; `migrate` chỉ nạp SQL khi chưa có bảng |
| [57](/articles/rust-cargo-mutants-fuzz-insta-kiem-thu-nang-cao) | mutation, fuzz, `insta`, giờ giả | `ManualClock` theo cùng nguyên tắc; chưa chạy mutation, fuzz hay `insta` trên lab này |

## Chạy ví dụ và kiểm chứng

Hạ tầng và biến môi trường theo `examples/toi-noi-rust/infra/README.md`. Từ gốc repository:

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
export TOI_NOI_KAFKA='127.0.0.1:9092'
cargo run --manifest-path examples/toi-noi-rust/58-ledger-service-workspace/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/58-ledger-service-workspace/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/58-ledger-service-workspace/Cargo.toml
cargo tree --manifest-path examples/toi-noi-rust/58-ledger-service-workspace/Cargo.toml -p ledger-api -e normal --depth 1
```

Lệnh đầu kể lại từng kịch bản (lần build đầu lâu vì `rdkafka` biên dịch `librdkafka` từ nguồn):

```text
== Chiều phụ thuộc (cargo tree)
   ledger-domain: sạch (16 crate trong cây, cấm 12)
   ledger-application: sạch (17 crate trong cây, cấm 10)
   ledger-infrastructure: sạch (137 crate trong cây, cấm 6)
   ledger-api: sạch (145 crate trong cây, cấm 3)
   đối chứng: luật của domain áp lên ledger-service bắt được 12 crate
== TripCompleted -> hai bút toán -> số dư, phát lại, tắt
   readyz 200 khi consumer đã được gán partition
   capture JE-000001 [("customer:CU-001", -25500), ("psp:vixanh:clearing", 25500)]
   split   JE-000002 [("psp:vixanh:clearing", -25500), ("platform:revenue", 5100), ("driver:DR-001", 20400)]
   driver 20400, revenue 5100, customer -25500, clearing 0 (HTTP và gRPC khớp)
   phát lại TripCompleted TN-0001: inbox chặn, vẫn 4 bút toán, 2 bút toán cho TN-0001
   shutdown có trật tự, outbox rỗng
== POST /v1/entries và gRPC PostEntry với JWT
   401 (không token, sai audience, hết hạn, HS512, rác) và 403 (cs ghi) đều là problem+json
   201 JE-000001, replay 200, cùng key khác nội dung 409, lệch 422; sự kiện đã phát
   422 gom 3 lỗi theo trường, 400 JSON hỏng, 422 tài khoản ngoài sơ đồ, x-request-id được trả lại
   gRPC: Unauthenticated, PermissionDenied, replay chung với HTTP, AlreadyExists, InvalidArgument, NotFound
== Hold, capture, hết hạn theo đồng hồ giả, hoàn tiền
   nạp 100000, hold 30000 → khả dụng 70000; hold 80000 bị 422 insufficient-available
   TripCompleted TN-0001: hold CAPTURED 25500, phần dư 4500 nhả; ví còn 74500, khả dụng 74500
   đồng hồ giả +901 giây: hold hết hạn, khả dụng về 74500; TN-0002 đóng hold EXPIRED, ví còn 62500
   hoàn 10000 (201), replay 200, khác số 409, vượt 422, chuyến lạ 404, cs 403; hoàn nốt 15500 → REFUNDED
   sau hoàn đủ TN-0001: tài xế 9600, doanh thu 2400, ví khách 88000 (chỉ còn TN-0002)
== Trigger cân bút toán, append-only, CHECK
   bút toán cân qua; lệch tổng và thiếu posting bị 23514 (check_violation) lúc COMMIT
   UPDATE, DELETE, TRUNCATE trên sổ đều bị TN001 (append-only)
   CHECK: hold 0 đồng và hoàn vượt số đã thu đều bị 23514, kể cả khi bỏ qua khóa dòng
== Thứ tự khóa account_balances
   bút toán ghi clearing trước revenue, nhưng khóa revenue trước; revenue bị giữ thì clearing vẫn trống
   nhả khóa: bút toán ghi xong, revenue 200
== SIGTERM tiến trình thật
   tiến trình con trả 20400 qua HTTP (x-request-id được trả lại) và gRPC, có JWT
   SIGTERM: thoát mã 0, 13 dòng log JSON, có request_id và 'shutdown complete', không lộ khóa JWT
VERIFY OK
```

`-- --verify` chạy đúng các kịch bản đó, chỉ in tên và `VERIFY OK`; sai thì thoát mã 1. `cargo test` chạy 34 unit test và 8 test end-to-end. Số dòng log của tiến trình con đổi mỗi lần chạy. Không kịch bản nào assert thời gian: giờ nghiệp vụ đi qua `ManualClock`, việc chờ sự kiện dùng hạn chót, stdout của tiến trình con đọc tới EOF thay vì đoán bằng `sleep`. Mỗi test case có schema `lab_<uuid>`, topic `lab.58.<uuid>.*`, group, khoá JWT và cổng `127.0.0.1:0` riêng.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Đây là service để học, không phải bản deploy được. Khoá JWT là chuỗi dùng chung, chưa có JWKS hay xoay khoá; topic một partition; relay là một task trong cùng process; <code>migrate</code> chỉ nạp SQL khi chưa có bảng; sự kiện hỏng bị bỏ qua và ghi log. Kịch bản chạy trên PostgreSQL và Kafka local.</p></div>

## Những gì chưa có

Liệt kê thẳng, theo thứ tự việc phải làm trước khi vận hành:

- **Profiling.** Chưa đo throughput, p99, bộ nhớ hay flamegraph; chưa có con số nào về 10.000 bút toán mỗi giây trên service này.
- **Container.** Chưa có Dockerfile, image, kiểm tra non-root hay kích thước.
- **CI.** Chưa có pipeline nào chạy `clippy`, test, `cargo audit`, `cargo deny` hay dựng PostgreSQL service container.
- **Deploy.** Chưa có secrets manager, autoscale hay health probe thật trên orchestrator.
- **OpenTelemetry.** Log JSON có `request_id` nhưng chưa có trace, chưa lan truyền `traceparent` qua gRPC metadata hay header Kafka, chưa có metric. Năm mục đến đây thuộc phần P07, hiệu năng và vận hành.
- **Nghiệp vụ lược.** Hold chưa có void và job hết hạn; hoàn tiền chưa có đảo toàn phần, chargeback, maker-checker; chưa có payout, khuyến mãi, đối soát VíXanh, đóng kỳ, hash chain; chưa nhận `PaymentCaptured` hay webhook.
- **API và đua.** Chưa có OpenAPI, timeout, rate limit; READ COMMITTED không retry `40001`; chưa dựng deadlock thật; chưa có DLQ; tên topic chưa có `.v1`; `event_id` suy từ `tripId` nên sửa cước cùng chuyến bị bỏ qua mà không báo gì như ở bài 40.

**Bài tập:** Một PR thêm hàm `fn is_expired_now(hold: &ActiveHold) -> bool { Timestamp::now() >= hold.expires_at }` vào `crates/domain/src/hold.rs`. Luật nào của bộ kiểm bắt nó, và nếu gỡ `clippy.toml` thì cái gì còn bắt?

<details><summary>Xem lời giải</summary><p>Không luật <code>cargo tree</code> nào bắt: <code>jiff</code> là dependency hợp lệ của <code>domain</code>, nên cây không đổi. <code>cargo clippy -D warnings</code> với <code>clippy.toml</code> cấm <code>jiff::Timestamp::now</code> bắt được (<code>use of a disallowed method</code>). Gỡ <code>clippy.toml</code> thì test <code>settle_depends_only_on_the_clock_argument</code> vẫn xanh, vì nó chỉ chứng minh <code>settle</code> nhận <code>now</code> làm đối số, không chứng minh không hàm nào khác đọc giờ thật. Chỉ còn người review.</p></details>

Bài trước: [bài 57](/articles/rust-cargo-mutants-fuzz-insta-kiem-thu-nang-cao). Bài tiếp: [bài 59](/articles/rust-bo-nho-khong-gc-stack-heap-allocator-so-voi-dotnet).

## Tài liệu gốc

- [The Cargo Book: Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html).
- [The Cargo Book: cargo tree](https://doc.rust-lang.org/cargo/commands/cargo-tree.html).
- [Clippy: disallowed_methods](https://rust-lang.github.io/rust-clippy/master/index.html#disallowed_methods).
- [docs.rs: tonic](https://docs.rs/tonic).
- [docs.rs: jsonwebtoken](https://docs.rs/jsonwebtoken).
- [PostgreSQL: CREATE TRIGGER](https://www.postgresql.org/docs/current/sql-createtrigger.html).
