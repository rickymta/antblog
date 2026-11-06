---
slug: rust-kafka-rdkafka-consumer-producer-schema
title: "Kafka với rdkafka: consume TripCompleted, produce LedgerEntryPosted"
summary: "Bài 36: rdkafka FutureProducer và StreamConsumer, commit offset sau khi ghi bút toán, at-least-once chạy thật, chọn key partition, rebalance và bỏ qua thông điệp hỏng không kẹt partition, trên Kafka thật."
category: rust
level: intermediate
tags: [rust, toi-noi, ledger, kafka, rdkafka, at-least-once]
language: vi
---

Chuyến `TN-0001` kết thúc, service Booking viết bằng C# phát `TripCompleted` lên Kafka. Service sổ cái đọc thông điệp, ghi bút toán thu cước 25.500 đồng, rồi phát `LedgerEntryPosted`. Có ba chỗ hỏng thật: process chết sau khi ghi mà chưa báo Kafka là xong; một bút toán chạm ba tài khoản trong khi thông điệp chỉ có một key; và một thông điệp hỏng chặn đứng cả partition.

Đây là bài 36 của lộ trình **Rust trung bình qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Lab chạy trên Kafka thật qua crate `rdkafka`. Sổ cái là `Ledger` trong bộ nhớ của `ledger-core` ([bài 20](/articles/rust-tong-hop-ledger-core-cargo-doc-workspace)), đứng thay cho PostgreSQL để bài chỉ nói về Kafka; ghi PostgreSQL cùng transaction với việc phát sự kiện là chuyện của [bài 37](/articles/rust-outbox-inbox-idempotent-consumer).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Consumer đã ghi bút toán nhưng chết trước khi commit offset. Khởi động lại thì bút toán bị ghi hai lần, hay bị mất?</p>
<details><summary>Xem đáp án</summary><p><b>Không hai lần, cũng không mất.</b> Offset chưa commit nên Kafka giao lại đúng thông điệp đó (at-least-once); lần ghi thứ hai gặp <code>idempotency_key</code> đã có nên chỉ là replay. Nếu commit <em>trước</em> khi ghi thì kết quả ngược lại: chết giữa chừng làm mất bút toán.</p></details>
</div>

## Commit offset trước hay sau khi ghi bút toán?

Offset đã commit là con trỏ «group đã xử lý xong tới đâu» trên từng partition, lưu ở Kafka. Có hai thứ tự nối việc ghi sổ với việc commit:

| Thứ tự | Chết giữa hai bước thì | Tên gọi |
|---|---|---|
| Commit rồi ghi | Thông điệp coi như xong, bút toán không có: mất tiền | at-most-once |
| Ghi rồi commit | Thông điệp đến lần nữa; ghi lại sẽ trùng nếu không có chốt | at-least-once |

Sổ cái chọn dòng dưới: trùng thì chặn được bằng `idempotency_key` ([bài 24](/articles/rust-transaction-isolation-append-only-idempotency-key)), còn mất thì không ai biết để sửa. Bài không hứa «exactly-once»: at-least-once cộng ghi idempotent cho hiệu ứng trên sổ cái đúng một lần.

`enable.auto.commit=false` trong `src/kafka.rs` tắt commit nền (mặc định bật, và có thể commit offset của thông điệp chưa xử lý xong). `enable.auto.offset.store` để mặc định không ảnh hưởng, vì lab commit tường minh từng thông điệp; muốn commit theo lô thì bật lại commit nền, đặt nó `false` và gọi `store_offset` sau khi ghi. `auto.offset.reset=earliest` cho group chưa có offset đọc từ đầu; với mặc định `latest`, consumer chết trước lần commit đầu tiên sẽ không nhận lại thông điệp đó. Producer bật `enable.idempotence` (kéo theo `acks=all`) để librdkafka gửi lại khi lỗi tạm thời mà không sinh bản trùng hay đảo thứ tự; chỉ bảo vệ trong một phiên producer, không thay việc khử trùng ở bên nhận.

```rust
// src/kafka.rs (rút gọn)
config
    .set("bootstrap.servers", &self.bootstrap)
    .set("group.id", group)
    .set("enable.auto.commit", "false")
    .set("auto.offset.reset", "earliest")
    .set("session.timeout.ms", "10000");

// producer
ClientConfig::new()
    .set("bootstrap.servers", &self.bootstrap)
    .set("enable.idempotence", "true")
    .set("message.timeout.ms", "15000")
    .create()
```

Lần chờ thông điệp đi qua `recv_within`, bọc `StreamConsumer::recv()` trong `tokio::time::timeout` ([bài 21](/articles/rust-async-await-tokio-task-timeout-cancellation)): hết giờ là `LabError::Timeout`, không treo. Thiếu `TOI_NOI_KAFKA` hoặc không nối được broker thì `Lab::start` dừng với lỗi rõ.

```rust
// src/kafka.rs
pub async fn recv_within<'a, C: ConsumerContext + 'static>(
    consumer: &'a StreamConsumer<C>,
    wait: Duration,
    what: &str,
) -> Result<BorrowedMessage<'a>, LabError> {
    match tokio::time::timeout(wait, consumer.recv()).await {
        Err(_) => Err(LabError::Timeout {
            what: what.to_string(),
            wait,
        }),
        Ok(received) => Ok(received?),
    }
}
```

`recv()` trả `Err` chưa có nghĩa consumer đã chết. Subscribe ngay sau khi tạo topic, lúc metadata chưa đủ leader, cho `UnknownTopicOrPartition`; đọc tiếp thì hết. Vì vậy `create_topic` của lab chờ metadata rồi mới trả về.

## Nhận lại thông điệp thì sổ cái ghi mấy lần?

Kịch bản dựng đúng tình huống của callout. Topic đầu vào (logic là `trip.completed`, trong lab là `lab.36.<uuid>.trip-completed`) có 3 partition; `TN-0001` rơi vào partition 1, offset 0. Lần chạy 1 ghi bút toán, phát `LedgerEntryPosted`, rồi consumer bị drop trước khi commit. Lần chạy 2 dùng cùng group và cùng sổ cái (sổ sống lâu hơn consumer, như cơ sở dữ liệu sống lâu hơn process), và commit sau khi ghi:

```rust
// src/scenarios.rs, lần chạy 2 (rút gọn)
let msg = recv_within(&consumer, WAIT, "TripCompleted ở lần chạy 2").await?;
let outcome = worker.handle(msg.payload()).await?;      // 1. ghi bút toán, phát sự kiện
consumer.commit_message(&msg, CommitMode::Sync)?;        // 2. chỉ khi bước 1 xong
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 1. Ba lần chạy cùng một group: lần 1 ghi bút toán rồi chết trước commit; lần 2 nhận lại đúng thông điệp, replay bút toán và commit offset 1; lần 3 không nhận gì."><defs><marker id="rust-kafka-rdkafka-consumer-producer-schema-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><text x="20" y="67" class="t-b">Lần chạy 1</text><text x="20" y="137" class="t-b">Lần chạy 2</text><text x="20" y="207" class="t-b">Lần chạy 3</text><rect x="140" y="40" width="230" height="44" rx="8" class="f-phys-soft s-phys"/><text x="154" y="59" class="t-b">1. Nhận offset 0</text><text x="154" y="75" class="t-small">TripCompleted TN-0001</text><rect x="395" y="40" width="230" height="44" rx="8" class="f-phys-soft s-phys"/><text x="409" y="59" class="t-b">2. Ghi JE-000001</text><text x="409" y="75" class="t-small">và phát LedgerEntryPosted</text><rect x="650" y="40" width="210" height="44" rx="8" class="f-warn-soft s-warn"/><text x="664" y="59" class="t-b">3. Chết, chưa commit</text><text x="664" y="75" class="t-small">group chưa có offset nào</text><path d="M370 62 L393 62" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-kafka-rdkafka-consumer-producer-schema-0-a)"/><path d="M625 62 L648 62" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-kafka-rdkafka-consumer-producer-schema-0-a)"/><rect x="140" y="110" width="230" height="44" rx="8" class="f-phys-soft s-phys"/><text x="154" y="129" class="t-b">1. Nhận lại offset 0</text><text x="154" y="145" class="t-small">cùng partition, cùng offset</text><rect x="395" y="110" width="230" height="44" rx="8" class="f-phys-soft s-phys"/><text x="409" y="129" class="t-b">2. Replay JE-000001</text><text x="409" y="145" class="t-small">sổ vẫn chỉ một bút toán</text><rect x="650" y="110" width="210" height="44" rx="8" class="f-pk-soft s-pk"/><text x="664" y="129" class="t-b">3. Commit offset 1</text><text x="664" y="145" class="t-small">chỉ sau khi ghi xong</text><path d="M370 132 L393 132" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-kafka-rdkafka-consumer-producer-schema-0-a)"/><path d="M625 132 L648 132" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-kafka-rdkafka-consumer-producer-schema-0-a)"/><rect x="140" y="180" width="230" height="44" rx="8" class="f-phys-soft s-phys"/><text x="154" y="199" class="t-b">1. Chờ 2 giây</text><text x="154" y="215" class="t-small">sau khi đã được gán partition</text><rect x="395" y="180" width="230" height="44" rx="8" class="f-pk-soft s-pk"/><text x="409" y="199" class="t-b">2. Không có gì đến</text><text x="409" y="215" class="t-small">offset 1 là thông điệp kế tiếp</text><path d="M370 202 L393 202" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-kafka-rdkafka-consumer-producer-schema-0-a)"/><rect x="20" y="244" width="840" height="46" rx="10" class="f-pk-soft s-pk"/><text x="36" y="264" class="t-b">Sổ cái: 1 bút toán, driver:DR-001 có 20.400 đồng, không phải 40.800.</text><text x="36" y="281" class="t-small">Topic đầu ra: 2 sự kiện cùng key, vì phát sự kiện nằm ngoài chốt idempotency của sổ cái.</text></svg>
</div>
<figcaption>Hình 1. Chết sau khi ghi, trước khi commit: Kafka giao lại, sổ cái nhận ra key cũ và không ghi thêm.</figcaption>
</figure>

Kết quả in ra (partition và offset là giá trị thật từ broker):

```text
gửi TripCompleted TN-0001: partition 1, offset 0
lần chạy 1 (partition 1, offset 0): ghi bút toán JE-000001; chết trước khi commit
offset đã commit của group trước lần chạy 2: None
lần chạy 2 (partition 1, offset 0): nhận lại, bút toán JE-000001 đã có; commit
offset đã commit sau lần chạy 2: Some(1)
lần chạy 3 nhận lại sau commit: false
bút toán trong sổ: 1; số dư driver:DR-001: 20400 đồng
LedgerEntryPosted trên topic đầu ra: 2 sự kiện
```

`Some(1)` là offset của thông điệp **sẽ đọc tiếp**: `commit_message` commit offset thông điệp cộng 1, và vì đó là con trỏ nên nó chốt luôn mọi offset thấp hơn của partition. Lần chạy 3 chờ được gán partition rồi mới đếm hai giây im lặng.

Nhưng topic đầu ra có **hai** sự kiện `LedgerEntryPosted` cùng key `trip:TN-0001:capture`. Replay vẫn phát lại vì lần phát trước có thể chưa tới nơi, nên bên nhận cũng phải khử trùng bằng `idempotencyKey`. Gốc là dual-write: ghi sổ và phát Kafka không chung transaction; bài 37 giải bằng outbox.

## Chọn key thế nào khi một bút toán chạm ba tài khoản?

Kafka chỉ bảo đảm thứ tự **trong một partition**. Producer chọn partition bằng hash của key, nên cùng key thì cùng partition, miễn là số partition không đổi (thêm partition cho topic đang chạy làm key đổi chỗ). Lab gửi thông điệp thử vào topic 6 partition theo ba cách:

```text
key = tripId, 12 chuyến: {1: 1, 2: 4, 3: 4, 4: 3}
key = platform:revenue, 12 bút toán: {2: 12}
ba tài khoản của bút toán trip:TN-0001:capture, key = tài khoản:
  customer:CU-001 -> partition 5
  driver:DR-001 -> partition 2
  platform:revenue -> partition 2
5 thông điệp cùng key TN-0001: [(1, 1), (1, 2), (1, 3), (1, 4), (1, 5)]
thứ tự nhận lại: [1, 2, 3, 4, 5]
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 2. Key là id tài khoản: một bút toán thành ba thông điệp ở hai partition và tài khoản nền tảng dồn mọi bút toán vào một partition. Key là tripId: một bút toán là một thông điệp, các chuyến trải ra nhiều partition."><rect x="20" y="20" width="410" height="266" rx="10" class="f-warn-soft s-warn"/><text x="36" y="48" class="t-b t-warn">Key = id tài khoản</text><rect x="36" y="62" width="220" height="40" rx="8" class="f-surface s-warn"/><text x="48" y="80" class="t-b">Một bút toán, 3 posting</text><text x="48" y="95" class="t-small">trip:TN-0001:capture</text><text x="36" y="124" class="t-small">thành 3 thông điệp:</text><rect x="36" y="132" width="380" height="30" rx="6" class="f-surface s-warn"/><text x="48" y="152" class="t-b">customer:CU-001</text><text x="300" y="152" class="t-b">partition 5</text><rect x="36" y="168" width="380" height="30" rx="6" class="f-surface s-warn"/><text x="48" y="188" class="t-b">driver:DR-001</text><text x="300" y="188" class="t-b">partition 2</text><rect x="36" y="204" width="380" height="30" rx="6" class="f-surface s-warn"/><text x="48" y="224" class="t-b">platform:revenue</text><text x="300" y="224" class="t-b">partition 2</text><text x="36" y="256" class="t-small">Ba thông điệp, hai partition.</text><text x="36" y="272" class="t-small">Key platform:revenue: 12/12 bút toán vào partition 2.</text><rect x="450" y="20" width="410" height="266" rx="10" class="f-pk-soft s-pk"/><text x="466" y="48" class="t-b t-pk">Key = tripId</text><rect x="466" y="62" width="220" height="40" rx="8" class="f-surface s-pk"/><text x="478" y="80" class="t-b">Một bút toán, 3 posting</text><text x="478" y="95" class="t-small">trip:TN-0001:capture</text><text x="466" y="124" class="t-small">thành 1 thông điệp:</text><rect x="466" y="132" width="380" height="30" rx="6" class="f-surface s-pk"/><text x="478" y="152" class="t-b">TN-0001</text><text x="716" y="152" class="t-b">partition 1</text><rect x="466" y="168" width="380" height="30" rx="6" class="f-surface s-pk"/><text x="478" y="188" class="t-b">12 chuyến, 12 key</text><text x="716" y="188" class="t-b">4 partition</text><rect x="466" y="204" width="380" height="30" rx="6" class="f-surface s-pk"/><text x="478" y="224" class="t-b">5 thông điệp cùng key</text><text x="716" y="224" class="t-b">offset 1 đến 5</text><text x="466" y="256" class="t-small">Một bút toán là một sự kiện.</text><text x="466" y="272" class="t-small">Thứ tự giữ trong partition của nó.</text></svg>
</div>
<figcaption>Hình 2. Hai cách chọn key cho cùng một bút toán. Số partition và offset là kết quả đo trên topic 6 partition của lab.</figcaption>
</figure>

Key là id tài khoản thì hỏng theo hai cách.

- **Một thông điệp chỉ có một key.** Bút toán chạm `customer:CU-001`, `driver:DR-001` và `platform:revenue`. Chọn một tài khoản thì hai tài khoản còn lại không có thứ tự nào với bút toán. Phát ba thông điệp thì một bút toán thành ba mảnh ở hai partition.
- **Tài khoản nền tảng nằm trong mọi bút toán.** Với key `platform:revenue`, cả 12 bút toán vào partition 2. Mỗi partition chỉ một consumer đọc, nên thêm consumer cũng không chia được tải: hot partition.

`tripId` (và `idempotencyKey` cho topic `ledger.entry-posted`, với một lưu ý ở bài tập) có đúng tính chất cần: mọi sự kiện của một chuyến cùng partition và theo thứ tự (thu cước rồi mới hoàn tiền); năm thông điệp key `TN-0001` nằm ở partition 1, offset 1 đến 5, đến đúng thứ tự gửi. Cái giá là không có thứ tự giữa các chuyến; sổ cái chịu được vì số dư là tổng posting, cộng thứ tự nào cũng ra cùng kết quả. Quy tắc phụ thuộc thứ tự giữa các chuyến (hạn mức, chẳng hạn) cần thiết kế riêng.

Hash chạy ở client, không ở broker, nên hai client khác thuật toán có thể đưa cùng key vào hai partition. librdkafka mặc định `consistent_random` (CRC32), producer Java dùng murmur2. Cùng key, cùng topic, hai cấu hình:

```text
key TN-0001: consistent_random -> partition 1, murmur2_random -> partition 3
key TN-0002: consistent_random -> partition 3, murmur2_random -> partition 4
key TN-0003: consistent_random -> partition 3, murmur2_random -> partition 3
```

Service Java và service Rust cùng ghi một topic thì đặt `partitioner=murmur2_random` phía librdkafka.

## Consumer group chia partition ra sao khi thêm hoặc bớt consumer?

Trong một group, mỗi partition được giao cho tối đa một consumer tại một thời điểm: Kafka vừa chia tải vừa giữ thứ tự theo partition. Lab chạy hai consumer cùng group trên topic 4 partition, mỗi consumer một task tokio, có context ghi lại rebalance:

```rust
// src/scenarios.rs
impl ConsumerContext for RebalanceLog {
    fn post_rebalance(&self, _base: &BaseConsumer<Self>, rebalance: &Rebalance<'_>) {
        let text = match rebalance {
            Rebalance::Assign(tpl) => format!("gán {:?}", parts(tpl)),
            Rebalance::Revoke(tpl) => format!("thu hồi {:?}", parts(tpl)),
            Rebalance::Error(err) => format!("lỗi {err}"),
        };
        self.events.lock().unwrap().push(text);
    }
}
```

```text
4 partition. Consumer 1 một mình: [0, 1, 2, 3]
consumer 2 vào group: consumer 1 giữ [0, 1], consumer 2 nhận [2, 3]
12 chuyến, key = tripId: consumer 1 xử lý 6, consumer 2 xử lý 6; bút toán trong sổ: 12
consumer 1 rời group: consumer 2 nhận lại [0, 1, 2, 3]
rebalance của consumer 1: ["gán [0, 1, 2, 3]", "thu hồi [0, 1, 2, 3]", "gán [0, 1]"]
rebalance của consumer 2: ["gán [2, 3]", "thu hồi [2, 3]", "gán [0, 1, 2, 3]"]
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 280" role="img" aria-label="Hình 3. Một consumer giữ cả bốn partition; consumer thứ hai vào group thì chia mỗi bên hai partition; consumer đầu rời group thì consumer còn lại nhận lại cả bốn."><text x="20" y="36" class="t-b">1 consumer</text><rect x="28" y="52" width="96" height="34" rx="8" class="f-phys-soft s-phys"/><text x="32" y="73" class="t-b">consumer 1</text><rect x="128" y="52" width="34" height="34" rx="6" class="f-pk-soft s-pk"/><text x="138" y="73" class="t-b">p0</text><rect x="168" y="52" width="34" height="34" rx="6" class="f-pk-soft s-pk"/><text x="178" y="73" class="t-b">p1</text><rect x="208" y="52" width="34" height="34" rx="6" class="f-pk-soft s-pk"/><text x="218" y="73" class="t-b">p2</text><rect x="248" y="52" width="34" height="34" rx="6" class="f-pk-soft s-pk"/><text x="258" y="73" class="t-b">p3</text><text x="28" y="150" class="t-small">gán [0, 1, 2, 3]</text><text x="305" y="36" class="t-b">consumer 2 vào group</text><rect x="313" y="52" width="96" height="34" rx="8" class="f-phys-soft s-phys"/><text x="317" y="73" class="t-b">consumer 1</text><rect x="413" y="52" width="34" height="34" rx="6" class="f-pk-soft s-pk"/><text x="423" y="73" class="t-b">p0</text><rect x="453" y="52" width="34" height="34" rx="6" class="f-pk-soft s-pk"/><text x="463" y="73" class="t-b">p1</text><rect x="313" y="92" width="96" height="34" rx="8" class="f-phys-soft s-phys"/><text x="317" y="113" class="t-b">consumer 2</text><rect x="493" y="92" width="34" height="34" rx="6" class="f-pk-soft s-pk"/><text x="503" y="113" class="t-b">p2</text><rect x="533" y="92" width="34" height="34" rx="6" class="f-pk-soft s-pk"/><text x="543" y="113" class="t-b">p3</text><text x="313" y="146" class="t-small">consumer 1 giữ [0, 1],</text><text x="313" y="162" class="t-small">consumer 2 nhận [2, 3]</text><text x="600" y="36" class="t-b">consumer 1 rời group</text><rect x="608" y="52" width="96" height="34" rx="8" class="f-warn-soft s-warn dash"/><text x="612" y="73" class="t-b">consumer 1</text><text x="708" y="73" class="t-small">đã rời</text><rect x="608" y="92" width="96" height="34" rx="8" class="f-phys-soft s-phys"/><text x="612" y="113" class="t-b">consumer 2</text><rect x="708" y="92" width="34" height="34" rx="6" class="f-pk-soft s-pk"/><text x="718" y="113" class="t-b">p0</text><rect x="748" y="92" width="34" height="34" rx="6" class="f-pk-soft s-pk"/><text x="758" y="113" class="t-b">p1</text><rect x="788" y="92" width="34" height="34" rx="6" class="f-pk-soft s-pk"/><text x="798" y="113" class="t-b">p2</text><rect x="828" y="92" width="32" height="34" rx="6" class="f-pk-soft s-pk"/><text x="837" y="113" class="t-b">p3</text><text x="608" y="150" class="t-small">consumer 2 nhận lại [0, 1, 2, 3]</text><path d="M295 28 L295 160" class="f-none s-muted dash" stroke-width="1.5"/><path d="M590 28 L590 160" class="f-none s-muted dash" stroke-width="1.5"/><rect x="20" y="180" width="840" height="84" rx="10" class="f-warn-soft s-warn"/><text x="36" y="204" class="t-b">Rebalance kiểu eager: consumer bị thu hồi hết partition rồi mới nhận phần mới.</text><text x="36" y="224" class="t-small">Log của consumer 1: gán [0, 1, 2, 3], thu hồi [0, 1, 2, 3], gán [0, 1]. Trong khoảng đó nó không đọc gì.</text><text x="36" y="244" class="t-small">Thông điệp xử lý dở lúc partition đổi chủ sẽ được giao lại cho chủ mới: lại là at-least-once.</text></svg>
</div>
<figcaption>Hình 3. Partition đi theo thành viên của group; thêm hay bớt consumer đều kích hoạt rebalance.</figcaption>
</figure>

Khi consumer 2 vào, consumer 1 **thu hồi cả bốn** partition rồi mới nhận lại `[0, 1]`. Đó là kiểu eager, mặc định của librdkafka 2.12 (`group.protocol=classic`, `partition.assignment.strategy=range,roundrobin`): trong lúc rebalance không ai đọc gì. `cooperative-sticky` chỉ chuyển phần cần chuyển; lab không chạy nó. Consumer nào nhận nửa nào phụ thuộc id thành viên, nên đây là kết quả một lần chạy.

Số partition là trần song song (consumer thứ năm của topic 4 partition ngồi không, theo tài liệu Kafka). Partition đổi chủ lúc đang xử lý dở thì chủ mới nhận lại thông điệp chưa commit: ghi idempotent đã lo chuyện đó. Consumer không gọi `recv()` quá `max.poll.interval.ms` (mặc định 5 phút) cũng bị đẩy khỏi group, nên xử lý quá chậm cũng gây rebalance.

## Thông điệp hỏng hoặc version lạ có được làm kẹt partition không?

Sự kiện của loạt là JSON camelCase và luôn mang `schemaVersion`. Consumer đọc `schemaVersion` **trước**, chỉ đọc phần còn lại khi hiểu version đó. Trường lạ trong cùng version được bỏ qua, nên thêm trường tùy chọn không làm hỏng consumer cũ; đổi tên, xoá hoặc đổi nghĩa trường thì phải tăng version. Thông điệp v2 báo «version 2 chưa hỗ trợ», không phải «thiếu trường».

```rust
// src/events.rs
pub fn decode_trip_completed(bytes: &[u8]) -> Result<TripCompleted, DecodeError> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Header {
        schema_version: Option<u64>,
    }
    let header: Header = serde_json::from_slice(bytes)?;
    match header.schema_version {
        None => Err(DecodeError::MissingVersion),
        Some(v) if v == u64::from(SCHEMA_VERSION) => Ok(serde_json::from_slice(bytes)?),
        Some(found) => Err(DecodeError::UnsupportedVersion { found }),
    }
}
```

Lỗi là giá trị (`DecodeError`, như [bài 17](/articles/rust-thiserror-anyhow-xu-ly-loi-thuc-te)). Lab xếp năm thông điệp vào một partition: hợp lệ, JSON cụt, version 2, thiếu `fareDong`, rồi `TN-0002` hợp lệ. Consumer đúng log, bỏ qua và **vẫn commit**; consumer «ngây thơ» dừng khi gặp lỗi và không commit:

```text
consumer bỏ qua thông điệp xấu và vẫn commit:
  offset 0: ghi bút toán JE-000001
  offset 1: bỏ qua: JSON hỏng hoặc không khớp schema: EOF while parsing a string at line 1 column 45
  offset 2: bỏ qua: schemaVersion 2 chưa hỗ trợ (consumer hiểu 1)
  offset 3: bỏ qua: JSON hỏng hoặc không khớp schema: missing field `fareDong` at line 1 column 116
  offset 4: ghi bút toán JE-000002
offset đã commit: Some(5); bút toán trong sổ: 2
consumer ngây thơ (dừng khi gặp lỗi), khởi động hai lần:
  dừng ở offset 1: JSON hỏng hoặc không khớp schema: EOF while parsing a string at line 1 column 45
  dừng ở offset 1: JSON hỏng hoặc không khớp schema: EOF while parsing a string at line 1 column 45
offset đã commit: Some(1); bút toán trong sổ: 1
```

Consumer ngây thơ khởi động lại bao nhiêu lần cũng dừng đúng offset 1; `TN-0002` nằm sau đó và không bao giờ được ghi. Đó là poison message: thử lại bao nhiêu lần cũng không thành.

Chính sách tạm thời của bài là **log, bỏ qua, commit**. Bỏ qua âm thầm là mất dữ liệu, nên log phải đủ để tìm lại (topic, partition, offset, lý do); lab chỉ in ra màn hình, service thật dùng log có cấu trúc ([bài 29](/articles/rust-config-secrets-tracing-structured-logging)). Cất thông điệp xấu sang topic riêng để xử lý sau, DLQ, là bài 37.

Lỗi hạ tầng (cơ sở dữ liệu tạm chết) thì khác: phải thử lại, và chỉ không commit thì chưa đủ. Vị trí đọc nằm trong bộ nhớ consumer nên thông điệp kế tiếp vẫn đến, và commit nó sẽ che luôn thông điệp lỗi:

```text
offset 0: lỗi giả lập, không commit
thông điệp kế tiếp vẫn đến: offset 1; commit nó
offset đã commit: Some(2)
seek về offset 0: nhận offset 0
khởi động lại không seek: nhận offset 2
```

Vậy phải thử lại tại chỗ, `seek` về offset đó, hoặc thoát để group giao lại từ offset đã commit, trước khi đụng tới thông điệp sau.

## So với Confluent.Kafka của .NET thì sao?

Confluent.Kafka, client Kafka phổ biến của .NET, cũng bọc librdkafka (lab dùng librdkafka 2.12.1 đi kèm `rdkafka-sys`). Vì vậy tên cấu hình gần như nguyên: `EnableAutoCommit`, `AutoOffsetReset`, `EnableIdempotence`, `Partitioner` ứng với `enable.auto.commit`, `auto.offset.reset`, `enable.idempotence`, `partitioner`, và mặc định cũng là `consistent_random`. Một lần kiểm riêng ngoài lab bằng Confluent.Kafka 2.12.0, topic 6 partition, cho đúng các partition Rust in ở trên với cả hai cấu hình (cùng key UTF-8, cùng số partition). Producer Java thì bài không chạy.

Khác nhau ở mô hình lập trình. Phía .NET, `IConsumer<TKey, TValue>.Consume(...)` chặn một thread (nhận timeout hoặc `CancellationToken`) nên vòng đọc thường nằm trong `BackgroundService` ([bài .NET về BackgroundService và Channel](/articles/dotnet10-backgroundservice-channel)), và `consumer.Commit(result)` là commit thủ công, cũng lưu offset cộng 1. Phía Rust, `recv()` là future, hủy được bằng `tokio::time::timeout`. `ProduceAsync` ứng với `FutureProducer::send`. Ý tưởng «ghi rồi commit, chặn trùng bằng khóa» giống hệt, xem [Outbox và Inbox trong .NET](/articles/dotnet10-outbox-inbox-idempotent-consumer). Phần mô hình lập trình dựa vào tài liệu của hai thư viện.

## Chạy ví dụ và kiểm chứng

Kafka chạy bằng Docker Compose ở `examples/toi-noi-rust/infra` (một broker `apache/kafka:4.1.0`, KRaft). Từ gốc repository:

```bash
docker compose -f examples/toi-noi-rust/infra/docker-compose.yml up -d --wait
export TOI_NOI_KAFKA='127.0.0.1:9092'
cargo run --manifest-path examples/toi-noi-rust/36-kafka-rdkafka/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/36-kafka-rdkafka/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/36-kafka-rdkafka/Cargo.toml
```

`--verify` chạy cả sáu kịch bản, kiểm các giá trị trong bài và in `VERIFY OK`, thoát mã 1 nếu sai. `cargo test` có bốn unit test giải mã (không cần broker) và bảy test trên Kafka thật, trong đó một test kiểm thiếu biến môi trường và không có broker. Mỗi ca tạo topic `lab.36.<uuid>.*` và group `lab-36-<uuid>*` rồi xoá đúng những thứ đó bằng `AdminClient`. Một lần `--verify` mất khoảng 25 giây; tám tiến trình song song, ba vòng, đều qua, lâu nhất 50 giây (số đo trên máy viết bài, không phải SLA). Chờ thông điệp và chờ gán partition có hạn 30 giây, nên broker bận thì chậm hoặc fail rõ; trường hợp broker chết giữa chừng chưa được thử. Lần build đầu lâu vì `rdkafka-sys` biên dịch librdkafka kèm theo (cần trình biên dịch C và `make`).

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Sổ cái trong bộ nhớ, mất khi process thoát. «Chết» ở đây là drop consumer; <code>kill -9</code> thì group phải đợi <code>session.timeout.ms</code> mới thu hồi partition. Một broker, replication 1, không TLS hay SASL. Commit đồng bộ từng thông điệp an toàn nhưng chậm, và chặn một worker của tokio trong lúc chờ broker. Chưa có DLQ, schema registry hay version 2 thật.</p>
</div>

**Bài tập:** bút toán đảo của `trip:TN-0001:capture` có key `trip:TN-0001:capture:reversal` và cũng chạm ba tài khoản. Nếu topic `ledger.entry-posted` dùng `idempotencyKey` làm key partition, điều gì có thể xảy ra với thứ tự giữa bút toán gốc và bút toán đảo? Sửa thế nào?

<details><summary>Xem lời giải</summary><p>Hai chuỗi key khác nhau thì hash khác nhau, nên hai sự kiện có thể rơi vào hai partition và bên nhận có thể thấy bút toán đảo trước bút toán gốc (lập luận, lab không chạy trường hợp này). Cách sửa: chọn key là định danh ổn định của chuỗi sự kiện (ở đây là <code>tripId</code>) thay vì khóa idempotency của từng bút toán; hoặc để bên nhận chịu được sự kiện đến sớm bằng cách giữ lại rồi thử lại.</p></details>

Bài trước: [bài 35](/articles/rust-tonic-grpc-get-balance-post-entry-interop-dotnet). Bài tiếp: [bài 37](/articles/rust-outbox-inbox-idempotent-consumer).

## Tài liệu gốc

- [docs.rs: crate rdkafka 0.39](https://docs.rs/rdkafka/0.39.0/rdkafka/) (`FutureProducer`, `StreamConsumer`, `AdminClient`).
- [docs.rs: StreamConsumer](https://docs.rs/rdkafka/0.39.0/rdkafka/consumer/stream_consumer/struct.StreamConsumer.html).
- [librdkafka: bảng cấu hình](https://github.com/confluentinc/librdkafka/blob/master/CONFIGURATION.md) (`enable.auto.commit`, `partitioner`, `enable.idempotence`).
- [Apache Kafka: message delivery semantics](https://kafka.apache.org/41/design/design/#message-delivery-semantics).
- [Apache Kafka: consumer groups và offset](https://kafka.apache.org/41/getting-started/introduction/).
- [Confluent: .NET client](https://docs.confluent.io/kafka-clients/dotnet/current/overview.html).
