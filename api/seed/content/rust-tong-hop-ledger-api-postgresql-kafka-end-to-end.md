---
slug: rust-tong-hop-ledger-api-postgresql-kafka-end-to-end
title: "Bài tổng hợp trung bình: ledger-api với PostgreSQL và Kafka chạy end-to-end"
summary: "Bài 40: gom P03–P04 thành service ledger-api chạy thật. TripCompleted từ Kafka thành bút toán, inbox và outbox cùng transaction, HTTP và gRPC trả số dư, tắt có thứ tự; test chờ sự kiện thật, phát lại và kill giữa chừng."
category: programming
level: intermediate
tags: [rust, toi-noi, ledger, kafka, postgresql, end-to-end]
language: vi
---

Sáng thứ Hai, nhóm Payment hỏi ba câu. Chuyến `TN-0001` hoàn thành lúc 08:30, cước 25.500 đồng: bao lâu sau thì số dư tài xế `DR-001` đúng 20.400? Nếu Kafka giao sự kiện đó hai lần thì sao? Nếu `ledger-api` chết giữa chừng thì sao? Hai mươi bài vừa qua, mỗi bài đã trả lời một mảnh. Bài này lắp các mảnh thành một service duy nhất và chạy thật với PostgreSQL và Kafka, để ba câu hỏi có ba test trả lời.

Đây là bài 40 của lộ trình **Rust trung bình qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Lab nằm trong một crate, dùng lại `ledger-core` của [bài 20](/articles/rust-tong-hop-ledger-core-cargo-doc-workspace) qua path dependency và chạy trên hạ tầng Docker Compose của loạt. Phần cần dùng của các bài 24–39 được viết lại gọn trong crate này; ngoài `ledger-core` không có path dependency nào.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Consumer đã commit transaction PostgreSQL ghi bút toán cho <code>TN-0001</code>, nhưng process chết trước khi commit offset Kafka. Khởi động lại thì trong sổ có mấy bút toán của chuyến này, và cái gì chặn bản thứ hai?</p>
<details><summary>Xem đáp án</summary><p>Một bút toán. Kafka giao lại message vì offset chưa commit; consumer thử ghi <code>inbox</code> với <code>event_id</code> đã có, thấy trùng, bỏ qua và commit offset. Lớp thứ hai là <code>idempotency_key</code> unique của <code>journal_entries</code>.</p></details>
</div>

## Hai mươi bài trước đã góp gì vào service này?

Cột giữa là điều bài đó thực sự dạy; cột phải là phần lab 40 lấy và phần bỏ. Phần bỏ quan trọng không kém: bài tổng hợp phải nói rõ cái gì chưa được lắp.

| Bài | Bài đó dạy | Lab 40 lấy gì, bỏ gì |
| --- | --- | --- |
| [20](/articles/rust-tong-hop-ledger-core-cargo-doc-workspace) | `Money`, `JournalEntry`, `capture_entry`, `default_policy` | `store.rs::entry_for_trip` gọi nguyên, không sửa lab 20 |
| [21](/articles/rust-async-await-tokio-task-timeout-cancellation) | future lười, `join!`, `timeout`, `select!`, `spawn`, `CancellationToken`, `spawn_blocking` | `app.rs`: bốn `spawn`, ba token ngang hàng; `select!` ở consumer và relay. Không cần `join!`, `spawn_blocking` |
| [22](/articles/rust-send-sync-arc-mutex-channel-concurrency) | `Send`/`Sync`, `Arc<Mutex>`, `RwLock`, actor bằng `mpsc`, `oneshot`, `broadcast` | `Arc<Stats>`, `Arc<Readiness>` dùng atomic; không `Mutex`, không actor. `Notify` đánh thức relay là thứ mới, bài 22 chưa dạy |
| [23](/articles/rust-sqlx-postgresql-pool-migration-query) | `PgPool`, `sqlx::migrate!`, `query!`/`query_as!`, `.sqlx` offline | `Store::connect` dùng pool. Bỏ `migrate!` và `query!`: `raw_sql(schema.sql)` và `query` kiểm lúc chạy |
| [24](/articles/rust-transaction-isolation-append-only-idempotency-key) | transaction, `ON CONFLICT DO NOTHING`, write skew, retry `40001`, trigger append-only | `post_in_tx`, trigger trong `sql/schema.sql`. Chạy READ COMMITTED, không có retry `40001` |
| [25](/articles/rust-schema-so-cai-accounts-journal-postings-partition) | bốn bảng, constraint trigger kiểm cân lúc commit, `account_balances`, index, partition | `sql/schema.sql`, thêm `outbox` và `inbox`. Bỏ index tra cứu và bản partition |
| [26](/articles/rust-repository-unit-of-work-trait-transaction) | port `LedgerRepository`, Unit of Work, adapter giả và adapter thật | Bỏ cả ba: `Store` là struct sqlx duy nhất, không có trait |
| [27](/articles/rust-integration-test-postgresql-schema-rieng-ci) | fixture `TestDb`, schema `lab_<uuid>`, `#[sqlx::test]`, workflow CI | `harness.rs::LabEnv` cùng ý, thêm topic và group riêng, đặt `search_path` qua `PgConnectOptions::options`. Không có CI |
| [28](/articles/rust-redis-cache-so-du-rate-limit-lock-ttl) | cache-aside, token bucket, lock có TTL | Không dùng: số dư đọc thẳng `account_balances`, lab không cần Redis |
| [29](/articles/rust-config-secrets-tracing-structured-logging) | crate `config`, `SecretString`, span, `#[instrument]`, log JSON | `config.rs` đọc env bằng tay, `pg_url` là `String` thường, chưa có `SecretString`; `telemetry.rs` log JSON, span mở bằng `info_span!` |
| [30](/articles/rust-criterion-benchmark-do-bo-nho-10k-but-toan) | criterion, `dhat`, ghi 10.000 bút toán/giây | Không dùng: lab không đo throughput |
| [31](/articles/rust-axum-http-endpoint-ghi-but-toan-tra-so-du) | `Router`, extractor, `AppError`, `POST /v1/entries`, tra số dư | `http.rs` chỉ có `GET` số dư, `/healthz`, `/readyz`. Chưa có `POST /v1/entries`: ghi thủ công đi qua gRPC |
| [32](/articles/rust-validation-problem-details-openapi-utoipa) | validate, Problem Details, OpenAPI bằng `utoipa` | Không dùng: lỗi là JSON `{"error": …}`, chưa có OpenAPI |
| [33](/articles/rust-tower-middleware-timeout-request-id-jwt-rate-limit) | chồng chín lớp: request id, trace, timeout, JWT, phân quyền, giới hạn tải | Chỉ `SetRequestIdLayer`, `TraceLayer`, `PropagateRequestIdLayer`. Không timeout, JWT, rate limit |
| [34](/articles/rust-axum-api-test-oneshot-hop-dong-bff) | `oneshot`, server thật, ca 401/403/409/422, snapshot OpenAPI, file JSON chung với C# | Chỉ lấy đường server thật: test gọi cổng TCP bằng hyper. Không `oneshot`, snapshot, client C# |
| [35](/articles/rust-tonic-grpc-get-balance-post-entry-interop-dotnet) | `.proto`, `tonic-prost-build`, map `Status`, deadline, interceptor, client .NET | Cùng `ledger.proto`, `grpc.rs` map lỗi gọn. Không deadline, interceptor, client .NET; tài khoản tự mở theo tiền tố thay vì trả `FAILED_PRECONDITION` |
| [36](/articles/rust-kafka-rdkafka-consumer-producer-schema) | `StreamConsumer`, `FutureProducer`, commit sau khi ghi, chọn key, rebalance, bỏ qua tin hỏng | `consumer.rs`, `kafka.rs`; `post_rebalance` bật `/readyz`. Một partition nên không có ca chia lại |
| [37](/articles/rust-outbox-inbox-idempotent-consumer) | dual-write, outbox, relay `SKIP LOCKED`, inbox, DLQ, thứ tự nhiều worker | `store.rs`, `relay.rs`. `event_id` suy từ `tripId`, không đi qua header; chưa có DLQ |
| [38](/articles/rust-webhook-psp-hmac-chong-replay-xu-ly-tre-trung) | HMAC trên byte thô, cửa sổ thời gian, `external_ref`, webhook đến sớm, xoay khoá | Không dùng: chưa nối webhook VíXanh, chưa có `PaymentCaptured` |
| [39](/articles/rust-graceful-shutdown-health-readiness-backpressure) | tín hiệu, sáu bước tắt, `TaskTracker`, `/healthz`, `/readyz`, hàng đợi bounded, `pause`/`resume`, mã thoát | `app.rs::shutdown` năm bước, `main.rs::serve`, `http.rs`. Bỏ `readiness_grace`, hàng đợi và `pause`: consumer xử lý ngay trong vòng `recv`. `/readyz` không hỏi metadata Kafka; quá hạn thoát mã 1, không phải 3 |

Với người quen .NET: `App::start` giống `WebApplication` cộng hai `BackgroundService` (xem [bài BackgroundService](/articles/dotnet10-backgroundservice-channel)); inbox và outbox cùng ý với [loạt .NET](/articles/dotnet10-outbox-inbox-idempotent-consumer).

## Một sự kiện đi qua những đâu?

Hình 1 vẽ đường đi của một `TripCompleted`. Booking (C#) phát lên Kafka; consumer đọc, và mọi thứ ghi vào PostgreSQL nằm trong **một** transaction. Relay đọc outbox và phát `LedgerEntryPosted`; HTTP và gRPC chỉ đọc bảng `account_balances`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 1. Booking phát TripCompleted lên Kafka; consumer ghi inbox, bút toán, số dư và outbox trong một transaction PostgreSQL rồi mới commit offset; relay đọc outbox và phát LedgerEntryPosted cho Payment; HTTP và gRPC đọc số dư từ cùng PostgreSQL."><defs><marker id="rust-tong-hop-ledger-api-postgresql-kafka-end-to-end-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="16" y="36" width="160" height="58" rx="8" class="f-phys-soft s-phys"/><text x="28" y="60" class="t-b">Booking (C#)</text><text x="28" y="80" class="t-small">phát TripCompleted</text><rect x="216" y="36" width="150" height="58" rx="8" class="f-phys-soft s-phys"/><text x="228" y="60" class="t-b">trip.completed</text><text x="228" y="80" class="t-small">topic, 1 partition</text><rect x="406" y="36" width="170" height="58" rx="8" class="f-pk-soft s-pk"/><text x="418" y="60" class="t-b">Consumer</text><text x="418" y="80" class="t-small">commit offset sau DB</text><rect x="616" y="20" width="248" height="196" rx="10" class="f-warn-soft s-warn"/><text x="630" y="46" class="t-b">Một transaction PostgreSQL</text><text x="630" y="72" class="t-small">1. inbox: INSERT event_id</text><text x="630" y="96" class="t-small">2. capture_entry (ledger-core)</text><text x="630" y="120" class="t-small">3. journal_entries + postings</text><text x="630" y="144" class="t-small">4. account_balances (cộng dồn)</text><text x="630" y="168" class="t-small">5. outbox: LedgerEntryPosted</text><text x="630" y="192" class="t-small">6. COMMIT</text><rect x="16" y="152" width="160" height="58" rx="8" class="f-phys-soft s-phys"/><text x="28" y="176" class="t-b">Payment (C#)</text><text x="28" y="196" class="t-small">đọc LedgerEntryPosted</text><rect x="216" y="152" width="150" height="58" rx="8" class="f-phys-soft s-phys"/><text x="228" y="176" class="t-b">entry-posted</text><text x="228" y="196" class="t-small">key: idempotencyKey</text><rect x="406" y="152" width="170" height="58" rx="8" class="f-pk-soft s-pk"/><text x="418" y="176" class="t-b">Relay</text><text x="418" y="196" class="t-small">phát rồi mới đánh dấu</text><rect x="616" y="236" width="248" height="50" rx="8" class="f-pk-soft s-pk"/><text x="630" y="258" class="t-b">HTTP + gRPC</text><text x="630" y="276" class="t-small">GetBalance đọc account_balances</text><path d="M176 65 L212 65" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-tong-hop-ledger-api-postgresql-kafka-end-to-end-0-a)"/><path d="M366 65 L402 65" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-tong-hop-ledger-api-postgresql-kafka-end-to-end-0-a)"/><path d="M576 65 L612 65" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-tong-hop-ledger-api-postgresql-kafka-end-to-end-0-a)"/><path d="M612 181 L580 181" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-tong-hop-ledger-api-postgresql-kafka-end-to-end-0-a)"/><path d="M402 181 L370 181" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-tong-hop-ledger-api-postgresql-kafka-end-to-end-0-a)"/><path d="M212 181 L180 181" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-tong-hop-ledger-api-postgresql-kafka-end-to-end-0-a)"/><path d="M740 234 L740 220" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-tong-hop-ledger-api-postgresql-kafka-end-to-end-0-a)"/><text x="16" y="248" class="t-small">Offset chỉ được commit sau bước 6. Chết trước đó thì Kafka giao lại,</text><text x="16" y="266" class="t-small">chết sau đó mà chưa commit offset thì inbox chặn bản giao lại.</text></svg>
</div>
<figcaption>Hình 1. Đường đi của một <code>TripCompleted</code>. Khung lớn là phần phải nguyên khối: bút toán, số dư và dòng outbox cùng có hoặc cùng không.</figcaption>
</figure>

Vài lựa chọn đáng chú ý. Consumer và relay không gọi nhau: consumer chỉ `notify_one()` một `Notify` để relay dậy sớm, còn lại relay tự quét outbox mỗi 200 ms. Số dư không có cache vì bài 28 đã nói Redis không là nguồn sự thật, và lab không cần nó. HTTP và gRPC dùng chung một `Store`, nên hai giao thức không thể trả hai số khác nhau. Key partition là `tripId` ở topic vào và `idempotencyKey` ở topic ra; bài 36 đã giải thích vì sao không lấy id tài khoản, khi một bút toán chạm nhiều tài khoản.

## Transaction nào ghi cả bốn thứ?

`apply_trip_completed` là trái tim của service. Mã dưới đây rút gọn từ `src/store.rs`:

```rust
// src/store.rs (rút gọn)
pub async fn apply_trip_completed(&self, ev: &TripCompleted) -> Result<Applied, StoreError> {
    let entry = entry_for_trip(ev)?; // ledger-core: 25.500 -> -25.500, 5.100, 20.400
    let mut tx = self.pool.begin().await?;
    let fresh = sqlx::query(
        "INSERT INTO inbox (event_id) VALUES ($1) ON CONFLICT (event_id) DO NOTHING",
    )
    .bind(ev.event_id())
    .execute(&mut *tx)
    .await?
    .rows_affected();
    if fresh == 0 {
        tx.rollback().await?;
        return Ok(Applied::Duplicate);
    }
    // bút toán, postings, account_balances, outbox: cùng `tx`
    let outcome = post_in_tx(&mut tx, &entry, &self.topic_out).await?;
    tx.commit().await?;
    // …
}
```

`entry_for_trip` gọi `capture_entry(&charge, &default_policy())` của `ledger-core`: phí 20% của 25.500 là 5.100 và tài xế nhận 20.400, ba posting cộng lại bằng 0. Service không tự tính tiền; nó chỉ ghép sự kiện vào kiểu của lõi. Trigger `postings_balanced` ở `schema.sql` là lớp bảo vệ thứ hai, kiểm lúc commit.

Mọi thứ trong `tx` tự rollback khi `tx` bị drop, kể cả dòng inbox. Đó là lý do inbox nằm **trong** transaction: nếu ghi inbox trước rồi mới ghi bút toán ở transaction khác, một lần chết giữa hai bước sẽ đánh dấu sự kiện là "đã xử lý" mà sổ không có gì.

`event_id` ở đây khác [bài 37](/articles/rust-outbox-inbox-idempotent-consumer). Bài đó mang nó trong header Kafka `event-id` (`ledger:<id>`, `trip-svc:A`). Lab 40 không gửi header, và hợp đồng payload không có `eventId`, nên `event_id()` lấy `TripCompleted:<tripId>` từ payload. Hệ quả: inbox khoá theo mã chuyến, không theo nội dung, và chặn trước khi tới `idempotency_key`. Thí nghiệm 6 của bài 37 (cùng chuyến, `event_id` khác, `idempotency_key` là lớp chặn) không tái hiện được ở lab này. Cùng chuyến mà khác cước cũng bị coi là bản trùng và bỏ qua, không báo lỗi; test `same_trip_with_other_fare_is_ignored_silently` ghi nhận điều đó.

Consumer chỉ commit offset sau khi hàm trả về `Ok`, hoặc khi lỗi là vĩnh viễn (bỏ qua, có log). Bảng dưới là các điểm chết có thể có và ai chịu trách nhiệm sửa:

| Chết ở đâu | Trạng thái | Phục hồi |
| --- | --- | --- |
| Trước `COMMIT` | DB không có gì, offset chưa commit | Kafka giao lại, xử lý như mới |
| Sau `COMMIT`, trước commit offset | Bút toán đã có | Giao lại, inbox chặn, commit offset |
| Sau `COMMIT`, relay chưa phát | Dòng outbox còn `published_at IS NULL` | Relay chạy lại phát tiếp |
| Relay gửi xong, chưa đánh dấu | `LedgerEntryPosted` có thể ra hai lần | Người nhận khử trùng theo `idempotencyKey` (key của message) |

Dòng cuối là at-least-once: lab **chưa có test** cho cửa sổ hẹp này, và người nhận phải tự khử trùng. Hai dòng đầu có test riêng, dòng thứ ba được kiểm gián tiếp (sự kiện vẫn về đủ), ở mục dưới.

## Tắt service cũng phải có thứ tự

Hình 2 là thứ tự `App::shutdown`. Ý tưởng: ngừng nhận việc mới trước, xả việc đang dở, rồi mới đóng cổng. Hai cổng đóng sau cùng vì chúng phải còn trả lời `/readyz` (503) trong lúc mọi thứ phía trước đang dừng.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Hình 2. Năm bước tắt theo thứ tự: readyz đổi 503, consumer dừng, relay xả outbox, đóng HTTP và gRPC, đóng pool. Tất cả nằm trong một hạn chung shutdown_timeout; quá hạn thì hủy cứng task còn lại."><defs><marker id="rust-tong-hop-ledger-api-postgresql-kafka-end-to-end-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><text x="10" y="28" class="t-b">Thứ tự tắt: ngừng nhận việc trước, đóng cổng sau cùng</text><rect x="10" y="44" width="140" height="100" rx="8" class="f-warn-soft s-warn"/><text x="22" y="70" class="t-b">1. readyz 503</text><text x="22" y="94" class="t-small">/healthz vẫn 200</text><text x="22" y="112" class="t-small">cổng còn mở</text><rect x="190" y="44" width="140" height="100" rx="8" class="f-pk-soft s-pk"/><text x="202" y="70" class="t-b">2. Consumer</text><text x="202" y="94" class="t-small">xong message dở</text><text x="202" y="112" class="t-small">commit offset</text><text x="202" y="130" class="t-small">ngừng nhận mới</text><rect x="370" y="44" width="140" height="100" rx="8" class="f-pk-soft s-pk"/><text x="382" y="70" class="t-b">3. Relay xả</text><text x="382" y="94" class="t-small">phát dòng còn lại</text><text x="382" y="112" class="t-small">đánh dấu đã phát</text><rect x="550" y="44" width="140" height="100" rx="8" class="f-phys-soft s-phys"/><text x="562" y="70" class="t-b">4. Đóng cổng</text><text x="562" y="94" class="t-small">HTTP và gRPC</text><text x="562" y="112" class="t-small">chờ request dở</text><rect x="730" y="44" width="140" height="100" rx="8" class="f-phys-soft s-phys"/><text x="742" y="70" class="t-b">5. Đóng pool</text><text x="742" y="94" class="t-small">tiến trình thoát</text><text x="742" y="112" class="t-small">mã 0</text><path d="M150 94 L186 94" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-tong-hop-ledger-api-postgresql-kafka-end-to-end-1-a)"/><path d="M330 94 L366 94" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-tong-hop-ledger-api-postgresql-kafka-end-to-end-1-a)"/><path d="M510 94 L546 94" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-tong-hop-ledger-api-postgresql-kafka-end-to-end-1-a)"/><path d="M690 94 L726 94" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-tong-hop-ledger-api-postgresql-kafka-end-to-end-1-a)"/><rect x="10" y="168" width="860" height="64" rx="10" class="f-none s-warn dash"/><text x="26" y="194" class="t-b">Một hạn chung: shutdown_timeout (mặc định 10 giây)</text><text x="26" y="216" class="t-small">Quá hạn thì task còn lại bị hủy cứng và báo timed_out = true; binary thoát mã khác 0.</text></svg>
</div>
<figcaption>Hình 2. Năm bước tắt chia nhau một hạn chung. Mỗi task có <code>CancellationToken</code> riêng nên thứ tự là có chủ ý.</figcaption>
</figure>

```rust
// src/app.rs (rút gọn)
pub async fn shutdown(mut self) -> ShutdownReport {
    let deadline = tokio::time::Instant::now() + self.timeout;
    self.readiness.begin_shutdown();           // 1. /readyz -> 503
    self.tokens.consumer.cancel();             // 2. xong message dở rồi dừng
    clean &= join_before("consumer", consumer, deadline).await;
    self.tokens.relay.cancel();                // 3. xả outbox
    clean &= join_before("relay", self.relay, deadline).await;
    self.tokens.servers.cancel();              // 4. đóng HTTP và gRPC
    // …
}
```

Vì sao hai endpoint sức khỏe? `/healthz` không hỏi DB hay Kafka: nó chỉ nói process còn chạy, để một lần PostgreSQL chậm không khiến orchestrator giết nhầm. `/readyz` hỏi nhiều hơn: DB trả lời `SELECT 1`, consumer **đã được gán partition**, và chưa bắt đầu tắt. Điều kiện giữa nhận từ `post_rebalance` của `ConsumerContext`. Subscribe xong chưa có nghĩa là nhận được việc, nên cờ này chỉ bật khi broker thật sự giao partition.

`request_id` đi theo hai đường. HTTP: `SetRequestIdLayer` giữ `x-request-id` do client gửi hoặc sinh UUID, `TraceLayer` mở span `http` mang nó, `PropagateRequestIdLayer` trả lại trong response. gRPC: handler đọc metadata `x-request-id` rồi mở span `grpc`. Log JSON mang span ở khoá `span`, nên mọi dòng trong request đều có `request_id`.

## Kiểm bằng sự kiện thật, không dùng sleep cố định

Test không giả Kafka: nó phát `TripCompleted` thật lên topic `lab.40.<uuid>.trip.completed`, rồi đọc `LedgerEntryPosted` từ topic ra như một service khác. Hàm chờ có hạn chót chứ không ngủ một khoảng cố định:

```rust
// src/scenario.rs (rút gọn)
async fn event_for(&self, key: &str, within: Duration) -> Result<LedgerEntryPosted> {
    let deadline = Instant::now() + within;
    loop {
        let received = timeout_at(deadline, self.consumer.recv())
            .await
            .map_err(|_| anyhow!("không thấy LedgerEntryPosted của {key} sau {within:?}"))?;
        // Lỗi tạm của broker không làm test đỏ ngay: hạn chót mới là phán quyết.
        let Ok(msg) = received else { continue };
        let event: LedgerEntryPosted = serde_json::from_slice(msg.payload().context("payload rỗng")?)?;
        if event.idempotency_key == key {
            return Ok(event);
        }
    }
}
```

Từ đó có ba kịch bản:

- **Luồng đủ vòng.** Sau `LedgerEntryPosted` của `TN-0001` (`JE-000001`; ba posting `-25500`, `5100`, `20400`), số dư `driver:DR-001` là 20.400 qua cả HTTP lẫn gRPC; tài khoản lạ trả 404.
- **Phát lại.** Chứng minh "không có gì xảy ra" cần một tín hiệu dương. Test phát lại đúng sự kiện đó, rồi phát tiếp chuyến `TN-0002` làm cột mốc. Topic một partition giữ thứ tự, nên khi `LedgerEntryPosted` của `TN-0002` (`JE-000002`) về, bản phát lại đã được xử lý. Lúc đó sổ có đúng 2 bút toán, inbox có 2 dòng, một bút toán cho key `trip:TN-0001:capture`.
- **Chết giữa chừng.** Cấu hình có một `Fault` do test đặt. `AfterDbCommitBeforeOffset` làm consumer trả về ngay sau `COMMIT`, không commit offset; test gọi `App::kill` (hủy cứng mọi task), dựng `App` mới cùng consumer group và chờ kết quả. Bản chạy lại nhận message giao lại, inbox chặn (`duplicates == 1`, `processed == 0`), sổ vẫn một bút toán, số dư tài xế 13.200.

Đây là **giả lập** cái chết, không phải `kill -9` ở thời điểm ngẫu nhiên; đổi lại nó chết đúng khe hở mà ta muốn kiểm, lần nào cũng vậy. Cái chết thật được kiểm ở chỗ khác: test chạy binary `ledger-api serve` như tiến trình con, gửi `SIGTERM` bằng lệnh `kill`, và kiểm thoát mã 0, log JSON có span `http` mang đúng `request_id` mà test gửi.

Khi cố ý phá, test đỏ đúng chỗ. Đưa `commit_message` lên trước `handle`, cả hai test chết giữa chừng đỏ sau hạn 30 giây: `không thấy LedgerEntryPosted của trip:TN-0003:capture` và `quá 30s mà chưa thấy: giao lại bị inbox chặn`. Kafka không giao lại message đã commit offset, nên bút toán mất vĩnh viễn.

Một chi tiết đáng nhớ ở ca `BeforeDbCommit`: sau khi chạy lại, bút toán của `TN-0003` mang mã `JE-000002`, không phải `JE-000001`. Lần thử bị rollback đã dùng `1` của cột identity, và sequence PostgreSQL không hoàn lại. Replay qua `PostEntry` cũng đốt một id, vì `ON CONFLICT DO NOTHING` chỉ xét sau khi giá trị mặc định đã được lấy. Khoảng trống trong dãy id không phải lỗi; đừng bao giờ coi mã bút toán là số thứ tự liền để đối soát.

## Chạy ví dụ và kiểm chứng

Hạ tầng và biến môi trường theo `examples/toi-noi-rust/infra/README.md`. Từ gốc repository:

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
export TOI_NOI_KAFKA='127.0.0.1:9092'
cargo run --manifest-path examples/toi-noi-rust/40-ledger-api-e2e/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/40-ledger-api-e2e/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/40-ledger-api-e2e/Cargo.toml
```

Lần build đầu lâu vì `rdkafka` biên dịch `librdkafka` từ nguồn (cần trình biên dịch C), và `build.rs` dùng `protoc-bin-vendored` nên máy không cần cài `protoc`. Lệnh đầu kể lại từng kịch bản; số mili giây và số dòng log thay đổi mỗi lần (số đo trên máy viết bài, không phải SLA):

```text
== Luồng TripCompleted -> số dư, phát lại, tắt
   readyz 200 khi consumer đã được gán partition
   LedgerEntryPosted JE-000001 [("customer:CU-001", -25500), ("platform:revenue", 5100), ("driver:DR-001", 20400)]
   số dư driver:DR-001 = 20400, platform:revenue = 5100, customer:CU-001 = -25500 (HTTP và gRPC khớp)
   phát lại TripCompleted TN-0001: inbox chặn, vẫn 2 bút toán, 1 bút toán cho key TN-0001
   shutdown trong 219 ms, outbox rỗng
== Chết sau commit DB, trước commit offset
   process chết (AfterDbCommitBeforeOffset): DB có 1 bút toán, offset chưa commit
   chạy lại: trip:TN-0003:capture -> 1 bút toán JE-000001, số dư tài xế 13200
== Chết trước commit DB
   process chết (BeforeDbCommit): DB có 0 bút toán, offset chưa commit
   chạy lại: trip:TN-0003:capture -> 1 bút toán JE-000002, số dư tài xế 13200
== gRPC PostEntry
   PostEntry JE-000001: replay, AlreadyExists, InvalidArgument đúng mã; sự kiện đã phát
== SIGTERM tiến trình thật
   tiến trình con trả 20400 qua HTTP (x-request-id được trả lại) và gRPC
   SIGTERM: thoát mã 0, 14 dòng log JSON, có request_id và 'shutdown complete'
VERIFY OK
```

`-- --verify` chạy đúng các kịch bản đó, chỉ in tên và `VERIFY OK`; sai thì thoát mã 1. `cargo test` chạy 7 unit test và 9 test end-to-end, gồm cả hai ca chết, `rollback_leaves_no_trace_in_any_table` (rollback xoá sạch năm bảng) và `cleanup_removes_schema_and_topics` (dọn xong không còn schema hay topic). Mỗi test có schema `lab_<uuid>`, topic `lab.40.<uuid>.*`, consumer group và cổng `127.0.0.1:0` riêng nên chạy song song được; `setup` chờ topic có leader rồi mới chạy, vì `create_topics` trả về trước khi metadata kịp lan. Một dòng log JSON của tiến trình con cho request đọc số dư:

```json
{"fields":{"latency":"23 ms","message":"finished processing request","status":200},"level":"INFO","span":{"method":"GET","name":"http","path":"/v1/accounts/driver:DR-001/balance","request_id":"req-e2e-0001"},"target":"tower_http::trace::on_response","timestamp":"2026-11-06T13:58:55.898182Z"}
```

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Đây là service để học, không phải bản có thể deploy. Topic chỉ một partition nên thứ tự là toàn cục; relay là một task trong cùng process; <code>migrate</code> chỉ chạy <code>schema.sql</code> khi chưa có bảng; kịch bản chết là giả lập; sự kiện hỏng bị bỏ qua và ghi log.</p></div>

## Điều lab chưa có

Liệt kê thẳng:

- **Xác thực và phân quyền.** Không JWT, không phân quyền Finance/CS, không rate limit. Mọi endpoint mở trên `127.0.0.1`.
- **Hợp đồng API đầy đủ.** Chưa có OpenAPI, Problem Details, `POST /v1/entries`, chưa có client .NET gọi vào.
- **Đối soát.** Chưa đọc sao kê VíXanh, chưa có `PaymentCaptured` hay webhook, chưa so sổ với bên ngoài.
- **Hold và capture.** Lab ghi thẳng một bút toán thu cước ba dòng khi chuyến xong; chưa có `AUTHORIZED`, giữ tiền, hoàn tiền, payout.
- **Partition.** Một partition; với nhiều partition thì thứ tự chỉ còn trong từng partition, và việc chọn key phải tính tới đó.
- **Sự kiện hỏng và backpressure.** Chưa có DLQ. Consumer xử lý ngay trong vòng `recv`, nên DB chết lâu hơn `max.poll.interval.ms` thì bị đá khỏi group ([bài 39](/articles/rust-graceful-shutdown-health-readiness-backpressure)).
- **Deploy và vận hành.** Chưa container, CI, migration thật, metric, OpenTelemetry, đo tải.
- **Sửa cước.** Cùng `TN-0001` với cước mới bị bỏ qua im lặng, như đã nói ở mục transaction. Sửa đúng nghĩa cần bút toán đảo cộng bút toán mới.

Các mục trên thuộc phần P05, nghiệp vụ sổ cái, và các phần sau ở lộ trình Rust nâng cao.

**Bài tập:** `TripCompleted` của `TN-0001` đến lần thứ hai với `fareDong = 26000` (sửa cước). Lab làm gì với nó, số dư `driver:DR-001` là bao nhiêu, và lớp nào của hệ thống đáng lẽ phải lên tiếng?

<details><summary>Xem lời giải</summary><p>Lab bỏ qua im lặng: <code>event_id</code> là <code>TripCompleted:TN-0001</code> đã có trong inbox nên <code>apply_trip_completed</code> trả <code>Duplicate</code> trước khi đụng tới sổ; số dư tài xế vẫn 20.400. Lớp thứ hai (<code>idempotency_key</code> unique, kèm so sánh nội dung ở <code>replayed</code>) có thể báo <code>IdempotencyConflict</code> nhưng không bao giờ được chạm tới. Cách xử lý đúng không nằm ở inbox: dùng <code>eventId</code> riêng cho từng sự kiện, phát hiện cước đổi, và ghi bút toán đảo cho bản cũ rồi bút toán mới. Test <code>same_trip_with_other_fare_is_ignored_silently</code> ghi nhận hành vi hiện tại.</p></details>

Bài trước: [bài 39](/articles/rust-graceful-shutdown-health-readiness-backpressure). Bài tiếp: phần P05, nghiệp vụ sổ cái, ở lộ trình Rust nâng cao.

## Tài liệu gốc

- [docs.rs: rdkafka](https://docs.rs/rdkafka), gồm `StreamConsumer` và `FutureProducer`.
- [docs.rs: axum](https://docs.rs/axum) và [tonic](https://docs.rs/tonic).
- [docs.rs: sqlx](https://docs.rs/sqlx), phần `Transaction` và `PgConnectOptions::options`.
- [docs.rs: tokio-util](https://docs.rs/tokio-util/latest/tokio_util/sync/struct.CancellationToken.html): `CancellationToken`.
- [Tokio: Graceful Shutdown](https://tokio.rs/tokio/topics/shutdown).
- [Apache Kafka: Message Delivery Semantics](https://kafka.apache.org/documentation/#semantics).
