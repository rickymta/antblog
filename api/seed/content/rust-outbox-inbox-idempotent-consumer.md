---
slug: rust-outbox-inbox-idempotent-consumer
title: "Outbox, inbox và idempotent consumer trong Rust"
summary: "Bài 37: tái hiện dual-write mất và sinh sự kiện bằng tiến trình con thoát giữa hai bước, ghi outbox cùng transaction sqlx, relay SKIP LOCKED, inbox event_id, DLQ và thứ tự khi nhiều worker, chạy trên PostgreSQL và Kafka."
category: rust-intermediate
level: intermediate
tags: [rust, toi-noi, ledger, kafka, outbox, sqlx]
language: vi
---

Service ledger nhận `TripCompleted` của chuyến `TN-0001`, ghi bút toán `JE-000001` (khách -25.500, nền tảng +5.100, tài xế +20.400) rồi phải báo cho Payment và Admin BFF bằng `LedgerEntryPosted`. Hai việc nằm ở hai hệ thống, PostgreSQL và Kafka, và không có transaction nào bao cả hai.

Đây là bài 37 của lộ trình **Rust trung bình qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. [Bài 36](/articles/rust-kafka-rdkafka-consumer-producer-schema) đã dựng consumer và producer `rdkafka`, [bài 24](/articles/rust-transaction-isolation-append-only-idempotency-key) đã có transaction và `ON CONFLICT`.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Ghi bút toán vào PostgreSQL, commit, rồi gọi producer Kafka, mọi lỗi đều trả về bằng <code>?</code>. Còn đường nào làm bút toán đã ghi mà sự kiện không bao giờ tới Kafka?</p>
<details><summary>Xem đáp án</summary><p>Có. <code>?</code> chỉ chạy khi tiến trình còn sống. Bị kill hoặc mất điện đúng giữa <code>commit</code> và <code>send</code> thì không dòng nào kịp chạy. Lab cho tiến trình con thoát đúng chỗ đó rồi đếm.</p></details>
</div>

## Ghi DB rồi gửi Kafka sai ở đâu, và đảo thứ tự có đỡ không?

Dual-write là ghi hai nơi bằng hai lệnh liên tiếp. Lab thử cả hai thứ tự, đầu tiên là ghi DB, commit, rồi mới gửi:

```rust
// src/dual_write.rs (rút gọn: bỏ chú thích tài liệu)
pub async fn db_then_kafka(
    pool: &PgPool,
    producer: &FutureProducer,
    topic: &str,
    entry: &JournalEntry,
    die_between: bool,
) -> Result<()> {
    let mut tx = pool.begin().await?;
    let inserted = insert_entry(&mut tx, entry).await?;
    tx.commit().await?;
    if die_between {
        // Bút toán đã commit; sự kiện chưa rời process.
        std::process::exit(137);
    }
    if let Some(inserted) = inserted {
        let payload = serde_json::to_string(&entry_posted(&inserted, entry))?;
        producer
            .send(
                FutureRecord::to(topic)
                    .key(entry.idempotency_key())
                    .payload(&payload),
                Timeout::Never,
            )
            .await
            .map_err(|(err, _)| err)?;
    }
    Ok(())
}
```

Với `die_between = true`, tiến trình con gọi `std::process::exit(137)` ngay sau `commit`. Đây là giả lập, không phải `kill -9`: destructor không chạy nên `Drop` của producer không flush, nhưng hook `atexit` vẫn chạy, và 137 chỉ là mã lab tự chọn (process bị SIGKILL thật thì `ExitStatus::code()` trả `None`). Tiến trình cha đếm bút toán và thông điệp:

```text
-- ghi DB, commit, rồi mới gửi Kafka (tiến trình con chết giữa hai bước)
mã thoát của con        : Some(137)
bút toán trong DB       : 1
thông điệp trong Kafka  : 0
```

Thứ tự ngược lại gửi Kafka trước, chờ broker xác nhận, rồi mới ghi DB. Ở đây DB lỗi vì tài xế `DR-999` không có trong `accounts`:

```text
-- gửi Kafka (đã xác nhận), rồi mới ghi DB (DB lỗi)
SQLSTATE ghi DB         : 23503
thông điệp trong Kafka  : 1
bút toán trong DB       : 0
```

Một bên mất sự kiện, một bên có sự kiện ma: người nhận thấy bút toán mà sổ không có. Đảo thứ tự chỉ đổi kiểu hỏng.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Hình 1. Hai thứ tự dual-write và chỗ mỗi thứ tự hỏng: ghi DB rồi gửi thì mất sự kiện khi chết giữa chừng, gửi rồi ghi DB thì để lại sự kiện ma khi DB lỗi."><defs><marker id="rust-outbox-inbox-idempotent-consumer-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><text x="20" y="26" class="t-b">A. Ghi DB rồi gửi Kafka</text><rect x="20" y="38" width="240" height="72" rx="10" class="f-pk-soft s-pk"/><text x="36" y="68" class="t-b">1. INSERT và COMMIT</text><text x="36" y="92" class="t-small">bút toán đã bền trong PostgreSQL</text><path d="M260 74 L298 74" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-outbox-inbox-idempotent-consumer-0-a)"/><rect x="300" y="38" width="240" height="72" rx="10" class="f-warn-soft s-warn"/><text x="316" y="68" class="t-b">Tiến trình chết</text><text x="316" y="92" class="t-small">kill, hết bộ nhớ, mất điện</text><path d="M540 74 L578 74" class="f-none s-muted dash" stroke-width="2" marker-end="url(#rust-outbox-inbox-idempotent-consumer-0-a)"/><rect x="580" y="38" width="280" height="72" rx="10" class="f-none s-muted dash"/><text x="596" y="68" class="t-b">2. Gửi Kafka: không chạy</text><text x="596" y="92" class="t-small">sổ có bút toán, Kafka không có gì</text><text x="20" y="146" class="t-b">B. Gửi Kafka rồi ghi DB</text><rect x="20" y="158" width="240" height="72" rx="10" class="f-pk-soft s-pk"/><text x="36" y="188" class="t-b">1. Gửi Kafka, đã ack</text><text x="36" y="212" class="t-small">broker đã nhận thông điệp</text><path d="M260 194 L298 194" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-outbox-inbox-idempotent-consumer-0-a)"/><rect x="300" y="158" width="240" height="72" rx="10" class="f-warn-soft s-warn"/><text x="316" y="188" class="t-b">2. Ghi DB lỗi, ROLLBACK</text><text x="316" y="212" class="t-small">khoá ngoại, timeout, deadlock</text><path d="M540 194 L578 194" class="f-none s-muted dash" stroke-width="2" marker-end="url(#rust-outbox-inbox-idempotent-consumer-0-a)"/><rect x="580" y="158" width="280" height="72" rx="10" class="f-none s-muted dash"/><text x="596" y="188" class="t-b">Sự kiện ma</text><text x="596" y="212" class="t-small">người nhận thấy bút toán sổ không có</text></svg>
</div>
<figcaption>Hình 1. Hai thứ tự dual-write, hai kiểu hỏng. Ô nét đứt là việc không xảy ra (A) hoặc hậu quả (B).</figcaption>
</figure>

## Outbox: ghi ý định gửi cùng transaction với bút toán

Cách chữa là không gửi ngay. Ý định gửi trở thành một dòng trong bảng `outbox`, ghi cùng transaction với bút toán. Hai bảng nằm trong cùng một PostgreSQL nên commit là nguyên tử. Thêm hai bảng cạnh bốn bảng chuẩn của bài 25:

```sql
-- sql/schema.sql (trích)
CREATE TABLE outbox (
    id           bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    topic        text NOT NULL,
    key          text NOT NULL,
    payload      jsonb NOT NULL,
    created_at   timestamptz NOT NULL DEFAULT now(),
    published_at timestamptz
);
CREATE INDEX outbox_unpublished ON outbox (id) WHERE published_at IS NULL;

CREATE TABLE inbox (
    event_id     text PRIMARY KEY,
    processed_at timestamptz NOT NULL DEFAULT now()
);
```

Hàm ghi nhận `&mut PgConnection` nên người gọi giữ ranh giới transaction, như `insert_entry` ở bài 24:

```rust
// src/store.rs (rút gọn: bỏ chú thích tài liệu)
pub async fn post_with_outbox(
    conn: &mut PgConnection,
    entry: &JournalEntry,
    topic: &str,
) -> Result<Option<Inserted>, sqlx::Error> {
    let Some(inserted) = insert_entry(&mut *conn, entry).await? else {
        return Ok(None);
    };
    let event = entry_posted(&inserted, entry);
    let payload = serde_json::to_string(&event).expect("sự kiện luôn serialize được");
    enqueue(&mut *conn, topic, entry.idempotency_key(), &payload).await?;
    Ok(Some(inserted))
}
```

Hai lần ghi liên tiếp, lần sau cố ý lỗi khoá ngoại ở posting thứ ba, rồi một lượt relay:

```text
ghi thành công          : 1 bút toán, outbox 1 dòng (1 chưa gửi)
ghi lỗi giữa chừng      : SQLSTATE 23503, vẫn 1 bút toán, 1 dòng outbox
Kafka trước khi relay   : 0 thông điệp
relay một lượt          : gửi dòng [1], outbox còn 0 chưa gửi
Kafka sau relay         : key=trip:TN-0001:capture event-id=ledger:1
payload                 : {"entryId": "JE-000001", "postedAt": "2026-11-06T13:55:22.580882Z", "postings": [{"account": "customer:CU-001", "amountDong": -25500}, {"account": "platform:revenue", "amountDong": 5100}, {"account": "driver:DR-001", "amountDong": 20400}], "schemaVersion": 1, "idempotencyKey": "trip:TN-0001:capture"}
```

Lần lỗi không để lại gì: bút toán chèn dở rollback, dòng outbox chưa kịp ghi, số đếm giữ nguyên. `postedAt` đổi mỗi lần chạy. Cột `jsonb` không giữ thứ tự khoá và khoảng trắng; cần đúng từng byte, ví dụ khi ký thông điệp, thì lưu `text` hoặc `bytea`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Hình 2. Tuyến outbox, relay, Kafka, inbox: hai đầu là transaction nguyên tử trong một database, hai mũi tên giữa là at-least-once."><defs><marker id="rust-outbox-inbox-idempotent-consumer-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="40" width="165" height="130" rx="10" class="f-pk-soft s-pk"/><text x="34" y="68" class="t-b">Một transaction</text><text x="34" y="92" class="t-small">journal_entries</text><text x="34" y="112" class="t-small">postings</text><text x="34" y="132" class="t-small">outbox: chưa gửi</text><text x="34" y="152" class="t-small">cùng một COMMIT</text><path d="M185 105 L238 105" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-outbox-inbox-idempotent-consumer-1-a)"/><rect x="240" y="40" width="165" height="130" rx="10" class="f-phys-soft s-phys"/><text x="254" y="68" class="t-b">Relay (task)</text><text x="254" y="92" class="t-small">FOR UPDATE</text><text x="254" y="112" class="t-small">SKIP LOCKED</text><text x="254" y="132" class="t-small">gửi, đợi ack</text><text x="254" y="152" class="t-small">rồi published_at</text><path d="M405 105 L458 105" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-outbox-inbox-idempotent-consumer-1-a)"/><rect x="460" y="40" width="165" height="130" rx="10" class="f-phys-soft s-phys"/><text x="474" y="68" class="t-b">Kafka</text><text x="474" y="92" class="t-small">lab.37.&lt;uuid&gt;.*</text><text x="474" y="112" class="t-small">header event-id</text><text x="474" y="132" class="t-small">acks=all</text><text x="474" y="152" class="t-small">idempotent producer</text><path d="M625 105 L678 105" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-outbox-inbox-idempotent-consumer-1-a)"/><rect x="680" y="40" width="165" height="130" rx="10" class="f-pk-soft s-pk"/><text x="694" y="68" class="t-b">Một transaction</text><text x="694" y="92" class="t-small">inbox(event_id)</text><text x="694" y="112" class="t-small">hiệu ứng: số dư</text><text x="694" y="132" class="t-small">cùng một COMMIT</text><rect x="20" y="190" width="165" height="56" rx="10" class="f-none s-pk"/><text x="34" y="214" class="t-small">đúng một DB: cùng có</text><text x="34" y="232" class="t-small">hoặc cùng không</text><rect x="240" y="190" width="385" height="56" rx="10" class="f-warn-soft s-warn"/><text x="254" y="214" class="t-small">relay → Kafka: chết sau ack thì gửi lại</text><text x="254" y="232" class="t-small">Kafka → consumer: chưa commit offset thì giao lại</text><rect x="680" y="190" width="165" height="56" rx="10" class="f-none s-pk"/><text x="694" y="214" class="t-small">đúng một DB: cùng có</text><text x="694" y="232" class="t-small">hoặc cùng không</text></svg>
</div>
<figcaption>Hình 2. Hai đầu là transaction nguyên tử. Hai mũi tên giữa chỉ bảo đảm at-least-once.</figcaption>
</figure>

## Relay lấy việc bằng SKIP LOCKED, gửi, rồi mới đánh dấu

Relay là một task đọc các dòng `published_at IS NULL`, gửi Kafka, đợi broker xác nhận rồi đặt `published_at`. Đánh dấu sau khi gửi là điểm mấu chốt (bài tập cuối bài nói vì sao).

```rust
// src/relay.rs (rút gọn: bỏ phần dựng bản ghi Kafka và móc thử nghiệm)
const CLAIM_LOCKED: &str = "SELECT id, topic, key, payload::text AS payload FROM outbox \
     WHERE published_at IS NULL \
       AND ($2::bigint = 0 OR (('x' || substr(md5(key), 1, 7))::bit(28)::int) % $2 = $3) \
     ORDER BY id LIMIT $1 FOR UPDATE SKIP LOCKED";

// trong relay_once:
let mut tx = pool.begin().await?;
let rows = sqlx::query(sql)
    .bind(cfg.batch)
    .bind(shard_count)
    .bind(shard_index)
    .fetch_all(&mut *tx)
    .await?;
// ... xếp từng dòng vào producer, rồi đợi ack theo đúng thứ tự ...
sqlx::query("UPDATE outbox SET published_at = now() WHERE id = ANY($1)")
    .bind(&ids)
    .execute(&mut *tx)
    .await?;
tx.commit().await?;
```

`FOR UPDATE` khoá các hàng đã nhận, `SKIP LOCKED` làm relay khác bỏ qua hàng đang bị khoá thay vì chờ. Khoá này giữ suốt lúc gửi Kafka, tới tận `commit`. Transaction ghi bút toán không bị kéo theo, vì nó chỉ `INSERT` dòng mới. Giá phải trả nằm ở relay: mỗi batch chiếm một kết nối của pool, và transaction đã có XID nên khi Kafka chậm tới `message.timeout.ms` (lab đặt 10 giây), `VACUUM` cũng không dọn được tuple chết mới hơn nó. Vì thế batch phải nhỏ và producer phải có hạn chót. Muốn transaction ngắn thì đổi sang lease (cột `claimed_until`): commit trước, gửi sau, đổi lại phải chọn hạn lease. Điều kiện `md5` dùng ở mục thứ tự; `$2 = 0` là không chia.

Hai relay chạy song song trên 10 dòng, mỗi relay một lượt 5 dòng. A nhận xong thì đứng lại, B bắt đầu đúng lúc đó, B gửi xong A mới đi tiếp. Lịch dùng `tokio::sync::Notify`, không dùng giờ, nên không đổi theo tải máy. Kết quả với `SELECT` thường, rồi với `SKIP LOCKED`:

```text
== 4a. Hai relay song song, SELECT không khóa ==
relay A nhận            : [1, 2, 3, 4, 5]
relay B nhận            : [1, 2, 3, 4, 5]
Kafka                   : 10 thông điệp, 5 event-id khác nhau
== 4b. Hai relay song song, FOR UPDATE SKIP LOCKED ==
relay A nhận            : [1, 2, 3, 4, 5]
relay B nhận            : [6, 7, 8, 9, 10]
Kafka                   : 10 thông điệp, 10 event-id khác nhau
xả thêm 100 dòng        : hai worker gửi 100, Kafka thêm 100 thông điệp, 100 event-id mới
```

Có khoá thì không trùng, kể cả khi hai worker xả thêm 100 dòng. Nhưng khoá chỉ chặn trùng giữa các relay còn sống. Relay chết sau khi gửi thì khác: kết nối đóng, PostgreSQL rollback, khoá được nhả, các dòng vẫn `published_at IS NULL`. `process::exit` đóng socket nên PostgreSQL biết gần như ngay (lab vẫn thử lại có hạn chót). Máy mất điện hay đứt mạng thì server chưa hay gì, khoá nằm lại tới khi TCP keepalive báo (`tcp_keepalives_idle`), và trong lúc đó `SKIP LOCKED` im lặng bỏ qua các dòng ấy. Relay con của lab thoát ngay sau khi Kafka xác nhận:

```text
== 3a. Relay chết sau khi gửi, consumer KHÔNG có inbox ==
mã thoát của relay con  : Some(137)
sau khi con chết        : Kafka 3 thông điệp, outbox 3 dòng chưa đánh dấu
sau khi relay chạy lại  : Kafka 6 thông điệp, 3 event-id khác nhau
thứ tự outbox id        : [1, 2, 3, 1, 2, 3]
```

Mỗi dòng tới hai lần với cùng header `event-id` (`ledger:1` đến `ledger:3`, lấy từ id dòng outbox nên ổn định giữa các lần gửi). `enable.idempotence` không giúp ở đây: nó chỉ chống trùng do retry trong một phiên producer, relay chạy lại là phiên mới. Đó là at-least-once, không phải lỗi: PostgreSQL và Kafka không có transaction chung.

## Inbox biến giao lại thành xử lý một lần

Consumer chặn bản trùng bằng bảng `inbox`. Hợp đồng sự kiện chung của loạt không có `eventId` trong payload, nên lab này mang nó trong header. Mỗi thông điệp được xử lý trong một transaction: chèn `event_id` trước, rồi làm hiệu ứng, cùng một commit.

```rust
// src/consumer.rs (rút gọn: bỏ nhánh TripCompleted)
let mut tx = pool.begin().await?;
if use_inbox {
    let inserted = sqlx::query(
        "INSERT INTO inbox (event_id) VALUES ($1) ON CONFLICT (event_id) DO NOTHING",
    )
    .bind(event_id)
    .execute(&mut *tx)
    .await?
    .rows_affected();
    if inserted == 0 {
        tx.rollback().await?;
        return Ok(Outcome::Duplicate);
    }
}
let outcome = match handler {
    // Handler::TripCompleted { .. } => bút toán và dòng outbox
    Handler::EntryPosted => {
        let event = parse_entry_posted(payload)?;
        for posting in &event.postings {
            sqlx::query(
                "INSERT INTO account_balances (account_id, balance_dong, version) \
                 VALUES ($1, $2, 1) \
                 ON CONFLICT (account_id) DO UPDATE SET \
                     balance_dong = account_balances.balance_dong + EXCLUDED.balance_dong, \
                     version = account_balances.version + 1, \
                     updated_at = now()",
            )
            .bind(&posting.account)
            .bind(posting.amount_dong)
            .execute(&mut *tx)
            .await?;
        }
        Outcome::Applied
    }
};
tx.commit().await?;
```

Ba điều làm cách này đúng. Một, `PRIMARY KEY` là chốt chặn cuối: tám task cùng nhận một `event_id` thì chỉ một task tạo hiệu ứng (lab đo `applied 1, duplicates 7`). Hai, hiệu ứng lỗi thì dòng inbox cũng rollback, nên lần giao lại sau vẫn được xử lý. Ba, inbox và hiệu ứng phải cùng database: nếu hiệu ứng là gọi API ngoài, dual-write quay lại ở phía consumer.

Kiểm bằng số dư. Consumer hạ nguồn cộng số tiền có dấu của `LedgerEntryPosted` vào `account_balances`. Sau kịch bản relay chết, sáu thông điệp tới consumer, một lần không inbox, một lần có:

```text
== 3a. Relay chết sau khi gửi, consumer KHÔNG có inbox ==
consumer                : applied 6, duplicates 0
số dư                   : driver:DR-001 122400, customer:CU-001 -153000, platform:revenue 30600
== 3b. Relay chết sau khi gửi, consumer CÓ inbox ==
consumer                : applied 3, duplicates 3
số dư                   : driver:DR-001 61200, customer:CU-001 -76500, platform:revenue 15300
```

Thiếu inbox, `driver:DR-001` phình thành 122.400 thay vì 61.200, trong khi `journal_entries` vẫn đúng 3 bút toán.

Inbox không thay `idempotency_key`; hai lớp chặn hai thứ khác nhau. `event_id` chặn cùng một thông điệp giao lại. `idempotency_key` chặn cùng một sự thật nghiệp vụ đến bằng thông điệp khác, ví dụ service C# phát lại `TripCompleted` với `event_id` mới. Thí nghiệm gửi ba thông điệp cho `TN-0001`, hai cái mang `event_id` `trip-svc:A`, một cái `trip-svc:B`:

```text
== 6. Inbox: A, A, B (B cùng chuyến TN-0001 nhưng event_id khác) ==
consumer                : applied 1, duplicates 1, entry_replays 1
DB                      : 1 bút toán, outbox 1 dòng, inbox 2 dòng
```

Bản A thứ hai bị inbox chặn. Bản B qua inbox rồi bị `ON CONFLICT` của bài 24 chặn, và không sinh dòng outbox thứ hai. Còn một đường trùng nữa: consumer chết sau khi xử lý nhưng trước khi commit offset. Lab giả lập bằng cách đóng consumer mà không commit:

```text
== 8. Chết trước khi commit offset ==
lần 1 (không commit)    : applied 1, offset đã commit -1
lần 2 (cùng group)      : applied 0, duplicates 1, offset đã commit 1
bút toán                : 1
```

Lần hai inbox chặn, rồi offset mới được commit.

## Thông điệp hỏng: DLQ để partition không kẹt

Một thông điệp luôn lỗi mà consumer cứ thử lại mãi sẽ chặn mọi thông điệp phía sau cùng partition. Lab thử mỗi thông điệp tối đa 3 lần, hết lượt thì gửi sang `lab.37.<uuid>.dlq` kèm lý do, đợi xác nhận rồi mới commit offset:

```rust
// src/consumer.rs (rút gọn: trong run_consumer)
for attempt in 1..=cfg.max_attempts {
    stats.attempts += 1;
    let result = match event_id.as_deref() {
        None => Err(ProcessError::MissingEventId),
        Some(id) => process_message(pool, &cfg.handler, cfg.use_inbox, id, payload).await,
    };
    match result {
        Ok(done) => {
            outcome = Some(done);
            break;
        }
        Err(err) => {
            last_error = err.to_string();
            if attempt < cfg.max_attempts {
                tokio::time::sleep(cfg.retry_delay).await;
            }
        }
    }
}
// outcome = None: dead_letter(...).await?, rồi commit_message
```

Năm thông điệp cùng một partition:

```text
== 9. DLQ: tốt, JSON hỏng, tốt, tài xế không tồn tại, tốt ==
consumer                : applied 3, dead_lettered 2, tổng lần thử 9
DB                      : 3 bút toán, 3 dòng inbox
offset đã commit        : 5 (topic có 5 thông điệp)
DLQ                     : x-source=lab.37.<uuid>.trip-completed/0/1 x-attempts=3 event-id=trip-svc:2
                          x-error=payload không đọc được: key must be a string at line 1 column 2
DLQ                     : x-source=lab.37.<uuid>.trip-completed/0/3 x-attempts=3 event-id=trip-svc:4
                          x-error=error returned from database: insert or update on table "postings" violates foreign key constraint "postings_account_id_fkey" ...
```

Offset đã commit là 5: consumer đi qua cả hai thông điệp hỏng nên partition không kẹt. Tổng 9 lần thử: 3 × 1 + 2 × 3. Thông điệp hỏng không để lại dòng inbox vì transaction của nó đã rollback, nên sau khi sửa nguyên nhân (thêm tài khoản `DR-999`) có thể phát lại từ DLQ. DLQ giữ nguyên byte gốc và thêm `x-error`, `x-attempts`, `x-source`. Gửi DLQ xong mà chết trước khi commit thì thông điệp vào DLQ hai lần, nên DLQ cũng cần xử lý idempotent. Đi DLQ cũng là chấp nhận mất thứ tự: thông điệp sau cùng key vẫn chạy tiếp trong khi bản hỏng nằm chờ.

## Thứ tự: nhiều worker đảo những gì?

Một relay lấy theo `ORDER BY id` thì giữ thứ tự id trong partition, như thí nghiệm relay chết. Có một lỗ lab không đo: id cấp lúc `INSERT`, không phải lúc `COMMIT`, nên hai transaction ghi song song có thể commit ngược id và relay gửi id lớn trước. Hai worker thì đảo rõ hơn: mỗi worker giữ một batch, batch gửi xong trước tới trước. Lab dùng 6 dòng xen kẽ hai key. Worker A nhận id 1, 2, 3 rồi đứng lại, worker B nhận 4, 5, 6 và gửi xong trước:

```text
== 5a. Thứ tự với hai worker, không chia ==
hai key                 : trip:TN-0003:capture / trip:TN-0001:capture
thứ tự outbox id        : [4, 5, 6, 1, 2, 3]
mỗi key đúng thứ tự     : false
toàn topic đúng thứ tự  : false
```

Đảo xảy ra cả trong một key chứ không chỉ giữa các key: `trip:TN-0003:capture` có các dòng 1, 3, 5 nhưng topic thấy 5, 1, 3. Cách chữa là chia hàng theo key để mỗi key chỉ qua một worker: điều kiện `md5(key)` chia lấy dư trong câu `SELECT` ở trên phải bằng phần của worker. Điều kiện kèm theo, lab không tự kiểm: mỗi phần chỉ một worker tại một thời điểm và số phần không đổi lúc chạy.

```text
== 5b. Thứ tự với hai worker, chia theo md5(key) ==
hai key                 : trip:TN-0003:capture / trip:TN-0001:capture
thứ tự outbox id        : [2, 4, 6, 1, 3, 5]
mỗi key đúng thứ tự     : true
toàn topic đúng thứ tự  : false
```

Mỗi key giữ thứ tự, còn thứ tự giữa các key không được bảo đảm; với Kafka điều đó chấp nhận được vì thứ tự chỉ có nghĩa trong một partition (bài 36).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 240" role="img" aria-label="Hình 3. Hai relay không chia theo key: worker A nhận dòng 1 đến 3 nhưng dừng, worker B nhận dòng 4 đến 6 và gửi trước nên topic có thứ tự 4 5 6 1 2 3."><defs><marker id="rust-outbox-inbox-idempotent-consumer-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><text x="20" y="47" class="t-b">Worker A</text><rect x="120" y="20" width="250" height="44" rx="10" class="f-pk-soft s-pk"/><text x="136" y="47" class="t-b">nhận id 1, 2, 3 rồi dừng</text><path d="M370 42 L598 42" class="f-none s-muted dash" stroke-width="2" marker-end="url(#rust-outbox-inbox-idempotent-consumer-2-a)"/><text x="400" y="34" class="t-small">đang giữ khoá dòng 1, 2, 3</text><rect x="600" y="20" width="260" height="44" rx="10" class="f-warn-soft s-warn"/><text x="616" y="47" class="t-b">gửi id 1, 2, 3 (muộn)</text><text x="20" y="111" class="t-b">Worker B</text><rect x="290" y="84" width="300" height="44" rx="10" class="f-pk-soft s-pk"/><text x="306" y="111" class="t-b">nhận id 4, 5, 6 và gửi ngay</text><text x="20" y="197" class="t-b">Topic</text><rect x="120" y="170" width="56" height="44" rx="8" class="f-pk-soft s-pk"/><text x="148" y="197" class="t-b" text-anchor="middle">4</text><rect x="186" y="170" width="56" height="44" rx="8" class="f-pk-soft s-pk"/><text x="214" y="197" class="t-b" text-anchor="middle">5</text><rect x="252" y="170" width="56" height="44" rx="8" class="f-pk-soft s-pk"/><text x="280" y="197" class="t-b" text-anchor="middle">6</text><rect x="318" y="170" width="56" height="44" rx="8" class="f-warn-soft s-warn"/><text x="346" y="197" class="t-b" text-anchor="middle">1</text><rect x="384" y="170" width="56" height="44" rx="8" class="f-warn-soft s-warn"/><text x="412" y="197" class="t-b" text-anchor="middle">2</text><rect x="450" y="170" width="56" height="44" rx="8" class="f-warn-soft s-warn"/><text x="478" y="197" class="t-b" text-anchor="middle">3</text><text x="540" y="186" class="t-b">Thứ tự trong topic: 4 5 6 1 2 3</text><text x="540" y="206" class="t-small">key TN-0003 (id 1, 3, 5) thấy 5, 1, 3: bị đảo</text></svg>
</div>
<figcaption>Hình 3. Batch của worker gửi nhanh hơn tới trước. Chia hàng theo md5(key) giữ thứ tự trong từng key.</figcaption>
</figure>

## So với bài outbox của loạt C#

[Bài 26 của loạt C#](/articles/dotnet10-outbox-inbox-idempotent-consumer) dựng cùng khuôn: outbox cùng commit, gửi rồi mới đánh dấu, inbox có unique key, không tuyên bố exactly-once. Khác nhau:

| Việc | Bài C# (EF Core) | Bài này (sqlx) |
|---|---|---|
| Ghi cùng commit | Một `SaveChanges` với dữ liệu và outbox | Một `Transaction`; hàm nhận `&mut PgConnection` |
| Chặn trùng ở consumer | Inbox khoá `consumer/EventId`, bắt 23505 rồi kiểm | `INSERT ... ON CONFLICT DO NOTHING`, đọc `rows_affected` |
| Nhiều worker | Một dispatcher, chưa claim | `FOR UPDATE SKIP LOCKED`, chia theo key |
| Transaction lúc gửi | Không: đọc, `PublishAsync`, rồi `SaveChanges` từng dòng | Có: giữ khoá hàng suốt batch |
| Tái hiện chết giữa hai bước | Cờ test ném exception | Tiến trình con `process::exit(137)` |
| Broker | `IPublisher` ghi trong memory | Kafka thật, `acks=all`, producer idempotent |
| Thông điệp hỏng | Chưa có | DLQ sau 3 lần |


<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Một broker Kafka, phần lớn thí nghiệm dùng topic một partition, không đo thông lượng. Relay chưa có backoff. Chưa dọn outbox và inbox cũ. Chưa phân loại lỗi tạm thời và vĩnh viễn, nên lỗi DB tạm thời cũng có thể đẩy một thông điệp tốt vào DLQ; chưa có công cụ phát lại. <code>account_balances</code> do consumer hạ nguồn cập nhật để thấy hậu quả của bản trùng; ở service thật số dư thường cập nhật ngay trong transaction ghi bút toán, và mỗi service có inbox riêng. <code>process::exit(137)</code> giả lập kill: socket đóng ngay, nên không thay được thử nghiệm <code>kill -9</code>, mất điện hay đứt mạng.</p></div>

## Chạy ví dụ và kiểm chứng

Cần PostgreSQL và Kafka của `examples/toi-noi-rust/infra`. Lần build đầu biên dịch librdkafka từ nguồn (cần `make` và trình biên dịch C), mất vài phút. Từ gốc repository:

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
export TOI_NOI_KAFKA='127.0.0.1:9092'
cargo run --manifest-path examples/toi-noi-rust/37-outbox-inbox/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/37-outbox-inbox/Cargo.toml
```

`--verify` in các khối kết quả ở trên rồi kiểm từng giá trị; đạt thì in `VERIFY OK` và thoát mã 0. `cargo test` chạy 12 test tích hợp, mỗi test có schema `lab_<uuid>` và topic `lab.37.<uuid>.*` riêng, xoá khi xong.

**Bài tập:** nếu relay đánh dấu `published_at` rồi mới gửi (đánh dấu và commit trước, gửi sau), chết giữa hai bước thì hậu quả là gì? Và vì sao không giải bài toán trùng bằng cách sinh `event_id` mới ở mỗi lần gửi lại?

<details><summary>Xem lời giải</summary><p>Dòng đã có <code>published_at</code> nhưng chưa tới Kafka, và relay không bao giờ nhặt lại: sự kiện mất, đúng lỗi dual-write chuyển vào bảng outbox. Gửi trước, đánh dấu sau chỉ đổi mất thành trùng, và trùng thì inbox xử lý được. <code>event_id</code> mới ở mỗi lần gửi lại làm inbox coi bản gửi lại là sự kiện khác, hết chặn trùng. Vì vậy <code>event_id</code> phải bắt nguồn từ dòng outbox (<code>ledger:&lt;id&gt;</code>) và giữ nguyên giữa các lần gửi.</p></details>

Bài trước: [bài 36](/articles/rust-kafka-rdkafka-consumer-producer-schema). Bài tiếp: [bài 38](/articles/rust-webhook-psp-hmac-chong-replay-xu-ly-tre-trung).

## Tài liệu gốc

- [PostgreSQL: SELECT, mục The Locking Clause (`FOR UPDATE`, `SKIP LOCKED`)](https://www.postgresql.org/docs/current/sql-select.html#SQL-FOR-UPDATE-SHARE).
- [PostgreSQL: INSERT, mục ON CONFLICT Clause](https://www.postgresql.org/docs/current/sql-insert.html).
- [Apache Kafka: Message Delivery Semantics](https://kafka.apache.org/documentation/#semantics).
- [docs.rs: rdkafka, `FutureProducer`](https://docs.rs/rdkafka/0.39.0/rdkafka/producer/struct.FutureProducer.html).
- [docs.rs: sqlx, `Transaction`](https://docs.rs/sqlx/0.9.0/sqlx/struct.Transaction.html).
- [Rust std: `std::process::exit`](https://doc.rust-lang.org/std/process/fn.exit.html).
