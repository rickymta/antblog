---
slug: rust-hexagonal-clean-workspace-crates-port-adapter
title: "Hexagonal/Clean trong Rust: workspace crates domain, application, infrastructure, api"
summary: "Bài 51: tách ledger-api thành bốn crate. Cargo cưỡng chế chiều phụ thuộc nhưng không cấm khai báo sai, nên cần test đọc cargo metadata. Port là trait, adapter bộ nhớ, sqlx và Kafka, feature kafka và khi nào chưa cần tách."
category: rust-advanced
level: advanced
tags: [rust, toi-noi, ledger, architecture, workspace, hexagonal]
language: vi
---

Sau [bài 40](/articles/rust-tong-hop-ledger-api-postgresql-kafka-end-to-end), `ledger-api` là một crate với mười ba module: `store.rs` nói chuyện với PostgreSQL, `kafka.rs` với broker, `http.rs` với axum. Ranh giới giữa chúng là quy ước ngầm. Không gì ngăn một dòng `use sqlx::…` chen vào bất kỳ module nào, vì cùng một crate thì `use` nào cũng biên dịch. P05 vừa thêm hold, hoàn tiền và đối soát, nên càng nhiều người sửa thì quy ước đó càng dễ vỡ.

Đây là bài 51 của lộ trình **Rust nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)**, bài mở đầu phần P06. Bài không dạy lại sqlx ([bài 23](/articles/rust-sqlx-postgresql-pool-migration-query)) hay Kafka ([bài 36](/articles/rust-kafka-rdkafka-consumer-producer-schema)). Câu hỏi là: ranh giới nào đáng để Cargo cưỡng chế, cưỡng chế được tới đâu, và trả giá gì. Lab dựng lại luồng chốt tiền chuyến `TN-0001` (capture rồi split, dạng hai bước của [bài 13](/articles/rust-double-entry-journal-entry-posting)) trong một workspace bốn crate.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Trong workspace bốn crate, một người viết <code>use sqlx::PgPool;</code> trong <code>domain</code> mà chưa sửa <code>Cargo.toml</code>. Chuyện gì xảy ra? Nếu họ thêm một dòng <code>sqlx</code> vào <code>Cargo.toml</code> của <code>domain</code> thì sao?</p>
<details><summary>Xem đáp án</summary><p>Lần đầu: lỗi biên dịch <code>E0432</code>, vì crate chưa được khai báo. Lần hai: build xanh, vì Cargo chỉ cưỡng chế điều đã khai báo chứ không cấm khai báo. Dòng thêm ở <code>Cargo.toml</code> hiện trong diff, và test kiến trúc của lab (đọc <code>cargo metadata</code>) chuyển đỏ.</p></details>
</div>

## Bốn crate, một chiều phụ thuộc

| Crate | Giữ gì | Phụ thuộc trong workspace |
|---|---|---|
| `domain` | `Trip`, hai bút toán capture và split, sự kiện `EntryPosted`; không async | không có |
| `application` | use case `SettleTrip`, `GetBalance` và ba port: `LedgerStore`, `EventPublisher`, `Clock` | `domain` |
| `infrastructure` | adapter bộ nhớ, `PgStore` (sqlx), `KafkaPublisher` (feature `kafka`), JSON trên dây | `application`, `domain` |
| `api` | router axum, composition root, kịch bản `--verify` | cả ba |

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Hình 1. Bốn crate xếp thành một chiều: api phụ thuộc infrastructure, infrastructure phụ thuộc application, application phụ thuộc domain. Mỗi mũi tên là một dòng trong Cargo.toml; không có mũi tên ngược."><defs><marker id="rust-hexagonal-clean-workspace-crates-port-adapter-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="24" width="180" height="96" rx="10" class="f-phys-soft s-phys"/><text x="34" y="52" class="t-b">api</text><text x="34" y="76" class="t-small">axum, composition root</text><text x="34" y="96" class="t-small">thấy cả bốn crate</text><rect x="240" y="24" width="180" height="96" rx="10" class="f-phys-soft s-phys"/><text x="254" y="52" class="t-b">infrastructure</text><text x="254" y="76" class="t-small">sqlx, Kafka, bộ nhớ</text><text x="254" y="96" class="t-small">cài đặt các port</text><rect x="460" y="24" width="180" height="96" rx="10" class="f-pk-soft s-pk"/><text x="474" y="52" class="t-b">application</text><text x="474" y="76" class="t-small">use case + port (trait)</text><text x="474" y="96" class="t-small">không tokio, không sqlx</text><rect x="680" y="24" width="180" height="96" rx="10" class="f-pk-soft s-pk"/><text x="694" y="52" class="t-b">domain</text><text x="694" y="76" class="t-small">Trip, hai bút toán</text><text x="694" y="96" class="t-small">ledger-core, thiserror</text><path d="M200 72 L238 72" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-hexagonal-clean-workspace-crates-port-adapter-0-a)"/><path d="M420 72 L458 72" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-hexagonal-clean-workspace-crates-port-adapter-0-a)"/><path d="M640 72 L678 72" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-hexagonal-clean-workspace-crates-port-adapter-0-a)"/><rect x="20" y="150" width="840" height="76" rx="10" class="f-pk-soft s-pk"/><text x="34" y="180" class="t-b">Mỗi mũi tên là một dòng trong Cargo.toml của crate ở đuôi mũi tên</text><text x="34" y="206" class="t-small">Không có mũi tên ngược: domain không gọi được application, application không gọi được sqlx</text></svg>
</div>
<figcaption>Hình 1. Chiều phụ thuộc của lab 51. Lõi (domain, application) nằm bên phải và không biết gì về bên trái.</figcaption>
</figure>

Port nằm ở `application`, không ở `infrastructure`: bên cần dịch vụ là bên định nghĩa hình dạng của nó. [Bài 26](/articles/rust-repository-unit-of-work-trait-transaction) đặt port ở crate riêng `ledger-port`; ở đây port ở `application` vì use case là bên duy nhất gọi chúng. Một crate `ports` riêng chỉ đáng khi có hai crate use case dùng chung.

**`domain` dùng `ledger-core` hay bọc lại?** Dùng thẳng. `Money`, `AccountId` và `JournalEntry` của [bài 20](/articles/rust-tong-hop-ledger-core-cargo-doc-workspace) mang bất biến «bút toán cân», và `Trip::settlement` chỉ dựng hai bút toán bằng builder của nó. Bọc lại sẽ có hai kiểu `Money` cùng bất biến cộng thêm một lớp chuyển đổi, trong khi ta không định thay thư viện đó. Cái giá: đổi `Money` là sửa cả bốn crate, và `domain` không phải «không phụ thuộc gì», vì `ledger-core` kéo theo `serde` và `rust_decimal`. Điều lab cấm là runtime và IO, không phải mọi crate.

```rust
// crates/domain/src/trip.rs (rút gọn)
pub fn settlement(&self) -> Result<Settlement, TripError> {
    let fare = self.fare();
    let split = default_policy().split(fare)?;
    let clearing = AccountId::new("psp:vixanh:clearing")?;
    let revenue = AccountId::new("platform:revenue")?;
    let out = |m: Money| m.checked_neg().ok_or(LedgerError::Overflow);

    // Dương = tiền vào tài khoản, âm = tiền ra (quy ước sổ dòng tiền của loạt).
    let capture = JournalEntryBuilder::new(&step_key(&self.id, "capture"))
        .posting(self.customer.clone(), out(fare)?)
        .posting(clearing.clone(), fare)
        .build()?;
    let split_entry = JournalEntryBuilder::new(&step_key(&self.id, "split"))
        .posting(clearing, out(fare)?)
        .posting(revenue, split.platform_fee)
        .posting(self.driver.clone(), split.driver_net)
        .build()?;
    // ... trả Settlement { fare, platform_fee, driver_net, capture, split }
}
```

## Cargo chặn được gì, và không chặn được gì?

Ba lần thử trên `domain`, lần nào cũng chạy thật rồi hoàn lại. Lần một, thêm `use sqlx::PgPool;` mà không khai báo:

```text
error[E0432]: unresolved import `sqlx`
  --> crates/domain/src/lib.rs:15:5
   |
15 | use sqlx::PgPool;
   |     ^^^^ use of unresolved module or unlinked crate `sqlx`
   |
   = help: if you wanted to use a crate named `sqlx`, use `cargo add sqlx` to add it to your `Cargo.toml`
```

Lần hai, khai báo `sqlx` trong `crates/domain/Cargo.toml`: `cargo build -p domain` **xanh**. Cargo coi đó là ý định có chủ đích. Vì vậy chiều phụ thuộc cần thêm một test, và `cargo tree` đã cho thấy cây sạch của hai crate lõi:

```text
$ cargo tree -p domain --depth 1
domain v0.1.0 (…/domain)
├── ledger-core v0.1.0 (…/ledger-core)
└── thiserror v2.0.21
$ cargo tree -p domain | grep -cE "tokio|sqlx"
0
$ cargo tree -p application | grep -cE "tokio|sqlx"
0
```

Lần ba, cho `domain` khai báo `application`: Cargo từ chối trước khi biên dịch gì (đường dẫn rút gọn):

```text
error: cyclic package dependency: package `application v0.1.0 (…)` depends on itself. Cycle:
package `application v0.1.0 (…)`
    ... which satisfies path dependency `application` (locked to 0.1.0) of package `domain v0.1.0 (…)`
    ... which satisfies path dependency `domain` (locked to 0.1.0) of package `application v0.1.0 (…)`
    ... which satisfies path dependency `application` (locked to 0.1.0) of package `api v0.1.0 (…)`
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 2. Ba lần thử trên domain. Dùng sqlx chưa khai báo: rustc báo E0432. Khai báo sqlx trong domain: build xanh nhưng test kiến trúc đỏ. Khai báo application trong domain: cargo từ chối vì vòng phụ thuộc."><defs><marker id="rust-hexagonal-clean-workspace-crates-port-adapter-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="24" width="270" height="76" rx="10" class="f-phys-soft s-phys"/><text x="34" y="54" class="t-b">1 · use sqlx::PgPool</text><text x="34" y="80" class="t-small">chưa khai báo trong Cargo.toml</text><rect x="305" y="24" width="270" height="76" rx="10" class="f-phys-soft s-phys"/><text x="319" y="54" class="t-b">2 · khai báo sqlx</text><text x="319" y="80" class="t-small">trong crates/domain/Cargo.toml</text><rect x="590" y="24" width="270" height="76" rx="10" class="f-phys-soft s-phys"/><text x="604" y="54" class="t-b">3 · khai báo application</text><text x="604" y="80" class="t-small">trong domain: tạo vòng phụ thuộc</text><path d="M155 100 L155 130" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-hexagonal-clean-workspace-crates-port-adapter-1-a)"/><path d="M440 100 L440 130" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-hexagonal-clean-workspace-crates-port-adapter-1-a)"/><path d="M725 100 L725 130" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-hexagonal-clean-workspace-crates-port-adapter-1-a)"/><rect x="20" y="130" width="270" height="60" rx="10" class="f-warn-soft s-warn"/><text x="34" y="157" class="t-b">rustc E0432</text><text x="34" y="177" class="t-small">unresolved import `sqlx`</text><rect x="305" y="130" width="270" height="60" rx="10" class="f-pk-soft s-pk"/><text x="319" y="157" class="t-b">cargo build: xanh</text><text x="319" y="177" class="t-small">domain biên dịch cùng sqlx</text><rect x="590" y="130" width="270" height="60" rx="10" class="f-warn-soft s-warn"/><text x="604" y="157" class="t-b">cargo từ chối</text><text x="604" y="177" class="t-small">cyclic package dependency</text><path d="M440 190 L440 220" class="f-none s-warn" stroke-width="2" marker-end="url(#rust-hexagonal-clean-workspace-crates-port-adapter-1-a)"/><rect x="305" y="220" width="270" height="60" rx="10" class="f-warn-soft s-warn dash"/><text x="319" y="247" class="t-b">test kiến trúc: đỏ</text><text x="319" y="267" class="t-small">domain kéo tokio, sqlx, mio</text><text x="34" y="240" class="t-small">Chặn ở trình biên dịch,</text><text x="34" y="260" class="t-small">khi chưa khai báo.</text><text x="604" y="240" class="t-small">Chặn ở Cargo, trước cả</text><text x="604" y="260" class="t-small">khi biên dịch.</text></svg>
</div>
<figcaption>Hình 2. Cargo chặn thứ chưa khai báo và thứ gây ra vòng phụ thuộc. Khai báo sai chiều nhưng vẫn hợp lệ thì cần test hoặc review.</figcaption>
</figure>

Test nằm ở `crates/api/tests/architecture.rs` và không đọc mã nguồn: nó chạy `cargo metadata`, dựng bao đóng phụ thuộc (kể cả gián tiếp) của từng crate rồi so với danh sách cấm:

```rust
// crates/api/tests/architecture.rs (rút gọn)
const FORBIDDEN_IN_CORE: [&str; 9] =
    ["tokio", "sqlx", "rdkafka", "axum", "hyper", "tonic", "tower", "mio", "reqwest"];

#[test]
fn core_crates_do_not_reach_runtime_or_io_crates() {
    let g = Graph::load(&[]);
    for core in ["domain", "application"] {
        let reach = g.closure(core);
        let leaked: Vec<&str> = FORBIDDEN_IN_CORE.iter().copied()
            .filter(|f| reach.contains(*f)).collect();
        assert!(leaked.is_empty(), "`{core}` kéo theo {leaked:?}; dependency trực tiếp: {:?}", g.edges[core]);
    }
    // Đối chứng: bộ quét không mù. `infrastructure` phải thấy sqlx và tokio.
    let infra = g.closure("infrastructure");
    assert!(infra.contains("sqlx") && infra.contains("tokio"));
}
```

Với dòng `sqlx` thêm vào `domain`, test đỏ và chỉ đúng nơi sửa; `mio` là phụ thuộc gián tiếp mà không ai khai báo:

```text
`domain` kéo theo ["tokio", "sqlx", "mio"]; dependency trực tiếp: {"ledger-core", "sqlx", "thiserror"}
```

Test thứ nhất cùng file ghim các cạnh của bảng ở trên. Với bốn crate này, cạnh ngược đã bị Cargo chặn bằng lỗi vòng, nên test đó chủ yếu có giá trị khi workspace có thêm crate. Phần đối chứng giữ nguyên tắc của [bài 50](/articles/rust-audit-append-only-hash-chain-phan-quyen-pii): test xanh chỉ có nghĩa khi bộ quét chứng minh được nó nhìn thấy cái cần tìm.

## Port là trait, adapter cắm vào ở composition root

Ba port của `application`, rút gọn. `-> impl Future + Send` thay cho `async fn` trần vì lý do đã giải thích ở bài 26: use case generic vẫn `tokio::spawn` được ở `api`, trong khi `application` không cần phụ thuộc tokio.

```rust
// crates/application/src/ports.rs (rút gọn)
pub trait LedgerStore: Send + Sync {
    fn post(&self, entry: &JournalEntry)
        -> impl Future<Output = Result<PostOutcome, StoreError>> + Send;
    fn balance(&self, account: &AccountId)
        -> impl Future<Output = Result<Money, StoreError>> + Send;
}
pub trait EventPublisher: Send + Sync {
    fn publish(&self, event: &EntryPosted)
        -> impl Future<Output = Result<(), PublishError>> + Send;
}
pub trait Clock: Send + Sync {
    fn now_unix(&self) -> i64;
}
```

`StoreError` có biến thể `Storage(Box<dyn Error + Send + Sync>)` cho mọi lỗi hạ tầng, nên port không biết `sqlx::Error` tồn tại. `infrastructure` có ba loại adapter. `MemoryStore` và `MemoryPublisher` luôn được biên dịch. `PgStore` bao một `PgPool` và dùng `INSERT … ON CONFLICT (idempotency_key) DO NOTHING RETURNING id`, đọc lại và so nội dung khi trùng key, trên đúng schema và trigger cân bút toán của [bài 25](/articles/rust-schema-so-cai-accounts-journal-postings-partition). `KafkaPublisher` chỉ có một việc:

```rust
// crates/infrastructure/src/kafka.rs (gộp bớt dòng)
impl EventPublisher for KafkaPublisher {
    async fn publish(&self, event: &EntryPosted) -> Result<(), PublishError> {
        let payload = wire::to_json(event)?;
        self.producer
            .send(
                FutureRecord::to(&self.topic).key(&event.idempotency_key).payload(&payload),
                Duration::from_secs(10),
            )
            .await
            .map_err(|(err, _)| PublishError(Box::new(err)))?;
        Ok(())
    }
}
```

JSON camelCase với `schemaVersion: 1` do `wire.rs` của `infrastructure` dựng, nên đổi tên trường JSON không chạm luật nghiệp vụ. Như bài 26, bộ test hợp đồng viết một lần và chạy cho từng adapter, nên `MemoryStore` không thể «dễ tính» hơn PostgreSQL mà không bị phát hiện:

```rust
// crates/api/tests/scenarios.rs (rút gọn)
async fn store_contract<S: LedgerStore>(store: &S) { /* post, replay, xung đột key, tài khoản lạ, số dư */ }

#[tokio::test]
async fn memory_store_keeps_the_contract() { store_contract(&MemoryStore::new()).await; }
#[tokio::test]
async fn pg_store_keeps_the_contract() { /* LabDb::setup, store_contract(&PgStore::new(pool)), cleanup */ }
```

Composition root là nơi duy nhất thấy cả port lẫn adapter. `Services` generic trên ba port, nên ghép sai kiểu là lỗi biên dịch, không phải lỗi lúc chạy:

```rust
// crates/api/src/compose.rs
impl<S: LedgerStore, P: EventPublisher, C: Clock> Services<S, P, C> {
    pub fn new(store: Arc<S>, publisher: Arc<P>, clock: Arc<C>) -> Services<S, P, C> {
        Services {
            balance: GetBalance::new(Arc::clone(&store)),
            settle: SettleTrip::new(store, publisher, clock),
        }
    }
}
```

`--verify` ghép ba tổ hợp (bộ nhớ; `PgStore` với publisher bộ nhớ; `PgStore` với `KafkaPublisher` trên topic riêng), gọi cùng một use case qua HTTP bằng `oneshot` và đòi cùng kết quả. Đồng hồ cố định ở 2026-11-09T01:30:00Z nên JSON không phụ thuộc giờ máy (rút gọn: bỏ phần lặp của tổ hợp 2):

```text
== 1. Bộ nhớ: MemoryStore + MemoryPublisher
POST /trips/settle -> 200 OK
  trip:TN-0001:capture   JE-000001  replayed=false
  trip:TN-0001:split     JE-000002  replayed=false
  customer:CU-001         -25500
  psp:vixanh:clearing          0
  platform:revenue          5100
  driver:DR-001            20400
POST lần hai: replayed=[true, true]
GET /accounts/driver:DR-001/balance -> 200 OK {"account":"driver:DR-001","balanceDong":20400}
POST distanceKm=0 -> 422 Unprocessable Entity application/problem+json
sự kiện đã phát: 2 (lần hai không phát thêm)
  {"schemaVersion":1,"entryId":"JE-000001","idempotencyKey":"trip:TN-0001:capture","postings":[{"account":"customer:CU-001","amountDong":-25500},{"account":"psp:vixanh:clearing","amountDong":25500}],"postedAt":"2026-11-09T01:30:00Z"}
== 2. PostgreSQL: PgStore + MemoryPublisher
... (cùng số dư, cùng replay)
JSON sự kiện giống hệt bản bộ nhớ: true
đã dọn schema lab_<uuid>: true
== 3. PostgreSQL + Kafka: PgStore + KafkaPublisher
đọc lại từ Kafka: khoá=["trip:TN-0001:capture", "trip:TN-0001:split"]
JSON trên Kafka giống hệt bản bộ nhớ: true
VERIFY OK
```

Số dư `psp:vixanh:clearing` về 0 sau bước split: tiền đi qua clearing nhưng không nằm lại.

## Feature flag của crate: build không cần librdkafka

`rdkafka` bọc librdkafka (mã C), nên là phụ thuộc nặng nhất của workspace. Lab đặt nó sau feature `kafka` của `infrastructure`, và `api` chuyển tiếp feature đó xuống:

```toml
# crates/infrastructure/Cargo.toml
[features]
default = ["kafka"]
kafka = ["dep:rdkafka"]
[dependencies]
rdkafka = { version = "0.39.0", optional = true }

# crates/api/Cargo.toml
[features]
default = ["kafka"]
kafka = ["infrastructure/kafka"]
```

Ở gốc workspace, `infrastructure` được khai báo `default-features = false`, nên chỉ `api` quyết định có bật `kafka` hay không. Module `kafka` có `#[cfg(feature = "kafka")]`; hàm `scenario::kafka` của `api` có hai bản `cfg`, bản tắt chỉ in một dòng. Đồ thị đổi theo:

```text
$ cargo tree -p api -i rdkafka
rdkafka v0.39.0
└── infrastructure v0.1.0
    └── api v0.1.0
$ cargo tree -p api --no-default-features -i rdkafka
error: package ID specification `rdkafka` did not match any packages
```

Test thứ ba trong `architecture.rs` kiểm cả hai chiều bằng `cargo metadata --no-default-features`: có `rdkafka` khi mặc định, không còn khi tắt, còn `sqlx` thì giữ. Build, `--verify` và `cargo test` với `--no-default-features` đều xanh; `--verify` ghi rõ «bỏ qua vì build không có feature `kafka`». Hai điều cần cẩn thận: feature của Cargo cộng dồn trong toàn đồ thị, chỉ cần một crate bật `infrastructure/kafka` thì `rdkafka` sẽ vào build; và `cargo build -p infrastructure` dùng `default` của chính crate đó, tức có Kafka.

Build nguội với thư mục `target` mới: `cargo build --no-default-features` biên dịch 153 crate, `cargo build` biên dịch 169. Mười sáu crate thêm là `rdkafka`, `rdkafka-sys`, `libz-sys`, `cc`, `pkg-config`, `num_enum` và các phụ thuộc của chúng. Bài không ghi thời gian build nguội: máy đang tải cao, và hai đợt đo cùng cấu hình lệch nhau gần gấp đôi.

## Tách crate làm build nhanh hơn bao nhiêu?

Cargo biên dịch lại crate có file đổi và mọi crate phụ thuộc vào nó. Thêm một dòng chú thích vào `domain` vẫn biên dịch lại cả bốn crate. Cách đo: `touch` một file trong từng crate rồi `cargo build`; các crate ngoài workspace không biên dịch lại.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 3. Số crate biên dịch lại khi sửa từng crate: sửa domain biên dịch lại cả bốn, sửa application ba, sửa infrastructure hai, sửa api một. Các crate bên ngoài workspace không biên dịch lại."><text x="250" y="34" class="t-b">domain</text><text x="380" y="34" class="t-b">application</text><text x="510" y="34" class="t-b">infrastructure</text><text x="640" y="34" class="t-b">api</text><text x="20" y="75" class="t-b">sửa domain</text><rect x="250" y="50" width="110" height="40" rx="8" class="f-warn-soft s-warn"/><text x="262" y="75" class="t-small">biên dịch lại</text><rect x="380" y="50" width="110" height="40" rx="8" class="f-warn-soft s-warn"/><text x="392" y="75" class="t-small">biên dịch lại</text><rect x="510" y="50" width="110" height="40" rx="8" class="f-warn-soft s-warn"/><text x="522" y="75" class="t-small">biên dịch lại</text><rect x="640" y="50" width="110" height="40" rx="8" class="f-warn-soft s-warn"/><text x="652" y="75" class="t-small">biên dịch lại</text><text x="20" y="125" class="t-b">sửa application</text><rect x="250" y="100" width="110" height="40" rx="8" class="f-phys-soft s-phys"/><text x="262" y="125" class="t-small">dùng lại</text><rect x="380" y="100" width="110" height="40" rx="8" class="f-warn-soft s-warn"/><text x="392" y="125" class="t-small">biên dịch lại</text><rect x="510" y="100" width="110" height="40" rx="8" class="f-warn-soft s-warn"/><text x="522" y="125" class="t-small">biên dịch lại</text><rect x="640" y="100" width="110" height="40" rx="8" class="f-warn-soft s-warn"/><text x="652" y="125" class="t-small">biên dịch lại</text><text x="20" y="175" class="t-b">sửa infrastructure</text><rect x="250" y="150" width="110" height="40" rx="8" class="f-phys-soft s-phys"/><text x="262" y="175" class="t-small">dùng lại</text><rect x="380" y="150" width="110" height="40" rx="8" class="f-phys-soft s-phys"/><text x="392" y="175" class="t-small">dùng lại</text><rect x="510" y="150" width="110" height="40" rx="8" class="f-warn-soft s-warn"/><text x="522" y="175" class="t-small">biên dịch lại</text><rect x="640" y="150" width="110" height="40" rx="8" class="f-warn-soft s-warn"/><text x="652" y="175" class="t-small">biên dịch lại</text><text x="20" y="225" class="t-b">sửa api</text><rect x="250" y="200" width="110" height="40" rx="8" class="f-phys-soft s-phys"/><text x="262" y="225" class="t-small">dùng lại</text><rect x="380" y="200" width="110" height="40" rx="8" class="f-phys-soft s-phys"/><text x="392" y="225" class="t-small">dùng lại</text><rect x="510" y="200" width="110" height="40" rx="8" class="f-phys-soft s-phys"/><text x="522" y="225" class="t-small">dùng lại</text><rect x="640" y="200" width="110" height="40" rx="8" class="f-warn-soft s-warn"/><text x="652" y="225" class="t-small">biên dịch lại</text><text x="780" y="75" class="t-b">4 crate</text><text x="780" y="125" class="t-b">3 crate</text><text x="780" y="175" class="t-b">2 crate</text><text x="780" y="225" class="t-b">1 crate</text><rect x="20" y="256" width="840" height="34" rx="8" class="f-pk-soft s-pk"/><text x="34" y="278" class="t-small">sqlx, axum, tokio, rdkafka và mọi crate ngoài workspace: dùng lại trong cả bốn trường hợp</text></svg>
</div>
<figcaption>Hình 3. Sửa càng gần lõi thì càng nhiều crate phải biên dịch lại.</figcaption>
</figure>

```text
sửa file                       crate biên dịch lại   CPU user + sys (trung vị 5 lần)
crates/domain/src/trip.rs              4              1,7 s
crates/application/src/settle.rs       3              1,6 s
crates/infrastructure/src/pg.rs        2              1,5 s
crates/api/src/http.rs                 1              1,2 s
```

Số crate là kết quả tất định; thời gian thì không. Thời gian thực tế (wall-clock) dao động mạnh vì máy tải cao, nên bảng dùng CPU, chỉ lệch khoảng 0,1 s giữa hai đợt đo. Đây là số đo trên máy viết bài, không phải SLA. Sửa `domain` tốn xấp xỉ 1,4 lần sửa `api`, chênh lệch nhỏ vì cả bốn crate chỉ khoảng 1.900 dòng kể cả test. Cái lợi này chỉ lớn khi `infrastructure` và `api` nặng hơn nhiều so với lõi, và khi các lần sửa thường nằm ở phía ngoài. Lab không đo cách gộp thành một crate duy nhất, nên không so được với cách đó.

## So với Clean Architecture của loạt C#

[Bài 31 của loạt C#](/articles/dotnet10-clean-architecture-layers) có cùng bốn tầng và cùng ý: tham chiếu hướng vào lõi, composition root biết implementation. Trong cột C#, `ProjectReference`, test đọc assembly và DI container lấy từ bài đó; ba dòng còn lại (vòng phụ thuộc, `InternalsVisibleTo`, đăng ký theo môi trường) là hiểu biết chung về công cụ .NET, chưa chạy trong loạt này.

| Khía cạnh | C# và .NET 10 | Rust (lab này) |
|---|---|---|
| Cạnh phụ thuộc | `ProjectReference` trong `.csproj` | `[dependencies]` trong `Cargo.toml`, phiên bản khai báo một chỗ bằng `workspace.dependencies` |
| Vòng phụ thuộc | project reference vòng không build được | Cargo từ chối với `cyclic package dependency` |
| Che nội bộ | `internal`, mở cho assembly chọn lọc bằng `InternalsVisibleTo` | `pub(crate)`; không có cách mở cho riêng một crate, nên muốn crate khác dùng thì phải `pub` |
| Kiểm kiến trúc | test đọc các assembly được tham chiếu | test đọc bao đóng phụ thuộc từ `cargo metadata`, kể cả phụ thuộc gián tiếp |
| Ghép adapter | DI container, resolve lúc chạy | generic ở composition root, compiler kiểm lúc biên dịch |
| Chọn adapter theo cấu hình | đăng ký khác nhau theo môi trường | feature lúc biên dịch, hoặc `match` ở composition root |

Dòng `pub(crate)` đáng chú ý nhất: `step_key` trong `domain` là `pub(crate)`. Nếu `domain` tạm mở module `trip` và `infrastructure` gọi `domain::trip::step_key`, rustc từ chối:

```text
error[E0603]: function `step_key` is private
  --> crates/infrastructure/src/clock.rs:32:19
   |
32 |     domain::trip::step_key("TN-0001", "capture")
   |                   ^^^^^^^^ private function
note: the function `step_key` is defined here
  --> crates/domain/src/trip.rs:48:1
   |
48 | pub(crate) fn step_key(trip_id: &str, step: &str) -> String {
```

Không có «friend crate», nên test của lõi sống trong chính crate (`#[cfg(test)]`) hoặc trong `tests/` chỉ thấy API `pub`. Về composition root, generic cho kiểu cụ thể và lỗi ghép sai hiện lúc biên dịch; đổi lại, chọn adapter theo cấu hình lúc chạy cần thêm một `match`, vì port dùng `impl Future` không dùng được với `dyn` (E0038, xem bài 26).

## Khi nào chưa cần tách crate, và đi từ bài 40 lên thế nào?

Tách crate có cái giá cụ thể. Lab có năm file `Cargo.toml`, tổng 123 dòng. Mọi thứ dùng xuyên crate đều thành `pub`, nên bề mặt API tăng. Phải có re-export (`pub use ledger_core::…` ở `domain`), và phải cân nhắc từng phụ thuộc nên đặt ở crate nào. Đổi lại là ba thứ đã chứng minh ở trên: Cargo từ chối thứ chưa khai báo và vòng, test đọc được bao đóng phụ thuộc, `--no-default-features` bỏ hẳn một nhóm crate nặng.

Chưa cần tách khi: một binary, mỗi port chỉ có một adapter, một đội nhỏ review mọi PR, và chưa có lý do thứ hai. Khi đó module trong một crate cộng quy ước là đủ; bài 40 là ví dụ. Dấu hiệu nên tách: có binary thứ hai dùng lõi, build của adapter bắt đầu cản việc sửa lõi, có adapter thay được thật (bộ nhớ cho test, Kafka tắt được), hoặc đã có một lần `sqlx` lọt vào logic nghiệp vụ. [Bài 58](/articles/rust-tong-hop-kien-truc-ledger-service-workspace) đi tiếp bằng cách tách composition root thành binary riêng.

Đường đi từ bài 40, làm bước rẻ trước (lab này dựng mới, nên đây là đề xuất, chưa được chạy thử):

1. Gom phần thuần (dựng bút toán, tính phí) khỏi `store.rs` và `consumer.rs` thành module không `use` crate IO.
2. Rút các trait ra thành port và để use case generic trên chúng, như bài 26 đã làm.
3. Chỉ khi có lý do thứ hai, đổi từng module thành crate, từ lõi ra ngoài.
4. Thêm test kiến trúc ngay từ crate đầu tiên.

## Chạy ví dụ và kiểm chứng

Cần PostgreSQL và Kafka của lab đang chạy ([bài 27](/articles/rust-integration-test-postgresql-schema-rieng-ci)). Thiếu biến hoặc không kết nối được thì `--verify` thoát mã 1. Từ gốc repository:

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
export TOI_NOI_KAFKA='127.0.0.1:9092'
cargo run --manifest-path examples/toi-noi-rust/51-hexagonal-workspace/Cargo.toml                 # chạy thử bộ nhớ, không cần hạ tầng
cargo run --manifest-path examples/toi-noi-rust/51-hexagonal-workspace/Cargo.toml -- --verify     # bộ nhớ, PostgreSQL, Kafka thật
cargo test --manifest-path examples/toi-noi-rust/51-hexagonal-workspace/Cargo.toml
cargo build --no-default-features --manifest-path examples/toi-noi-rust/51-hexagonal-workspace/Cargo.toml   # không có rdkafka
cargo tree --manifest-path examples/toi-noi-rust/51-hexagonal-workspace/Cargo.toml -p domain      # không có tokio, sqlx
```

`--verify` in các khối kết quả ở trên rồi kiểm từng giá trị. `cargo test` chạy 10 test: ba test kiến trúc, ba test kịch bản và hợp đồng trên PostgreSQL và Kafka thật, bốn test đơn vị. Không test nào assert thời gian.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Use case ghi bút toán rồi mới phát sự kiện, không qua outbox. Nếu <code>publish</code> lỗi sau khi bút toán đã ghi, lần chạy lại thấy <code>replayed = true</code> và không phát lại, tức là sự kiện bị mất; bài toán này là của <a href="/articles/rust-outbox-inbox-idempotent-consumer">bài 37</a>. Adapter Kafka chỉ có producer; chưa có consumer, gRPC, xác thực hay hợp đồng lỗi đầy đủ của bài 32. Test kiến trúc chỉ biết tên crate, không thấy một <code>use</code> sai tầng trong cùng crate, và cần cache registry vì chạy <code>cargo metadata --offline</code>. Số đo CPU là một máy, profile dev, tải cao. <code>domain</code> mỏng vì <code>ledger-core</code> đã là lõi.</p></div>

**Bài tập:** thêm adapter gRPC `GetBalance` (bài 35) vào workspace. Crate nào phải sửa `Cargo.toml`, có cần port mới không, và test kiến trúc phải đổi gì nếu đặt adapter trong crate mới tên `grpc`?

<details><summary>Xem lời giải</summary><p>gRPC là adapter phía vào, cùng loại với router axum, nên nằm ở <code>api</code> (hoặc một crate mới cùng tầng), và <code>tonic</code>, <code>prost</code> chỉ vào <code>Cargo.toml</code> của crate đó. Không cần port mới: use case <code>GetBalance</code> đã có. Nếu tách crate <code>grpc</code>, thêm nó vào <code>MEMBERS</code> và thêm kỳ vọng <code>grpc: {application, domain}</code> trong test hướng phụ thuộc; <code>tonic</code> đã nằm trong danh sách cấm của lõi.</p></details>

Bài trước: [bài 50](/articles/rust-audit-append-only-hash-chain-phan-quyen-pii). Bài tiếp: [bài 52](/articles/rust-command-handler-pipeline-domain-event-khong-framework).

## Tài liệu gốc

- [The Cargo Book: Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html).
- [The Cargo Book: Features](https://doc.rust-lang.org/cargo/reference/features.html).
- [The Cargo Book: `cargo metadata`](https://doc.rust-lang.org/cargo/commands/cargo-metadata.html).
- [The Rust Reference: Visibility and privacy](https://doc.rust-lang.org/reference/visibility-and-privacy.html).
- [docs.rs: rdkafka](https://docs.rs/rdkafka/0.39.0/rdkafka/).
- [docs.rs: axum](https://docs.rs/axum/0.8.9/axum/).
