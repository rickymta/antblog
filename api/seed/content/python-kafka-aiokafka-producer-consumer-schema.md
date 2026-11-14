---
slug: python-kafka-aiokafka-producer-consumer-schema
title: "Kafka với aiokafka: producer, consumer và schema sự kiện"
summary: "Bài 38: dùng aiokafka cho Tới Nơi: key theo khách để giữ thứ tự, consumer group, commit offset sau xử lý (at-least-once), tái hiện xử lý trùng và mất sự kiện, schema version trong envelope. Lab chạy thật."
category: python-intermediate
level: intermediate
tags: [python, toi-noi, kafka, aiokafka, event]
language: vi
---

Khi một chuyến được đặt, nhiều bên cần biết: fraud-scoring muốn chấm điểm, analytics muốn đếm, thông báo muốn nhắn khách. Nếu Dispatch của [Tới Nơi](/articles/du-an-mau-toi-noi) gọi từng bên một thì thêm một bên là phải sửa Dispatch, và một bên chậm kéo cả đường đặt chuyến chậm theo. Cách khác là Dispatch chỉ ghi một sự kiện `BookingCreated` vào Kafka rồi thôi; ai quan tâm thì đọc. Cái giá là những câu hỏi mà lời gọi trực tiếp không có: nếu fraud-scoring sập giữa chừng thì sự kiện đang xử lý ra sao, sự kiện của cùng một khách có đến theo thứ tự không, và khi hình dạng sự kiện đổi thì consumer cũ có gãy không.

Đây là bài 38 của lộ trình **Python trung bình qua Tới Nơi**. Bạn cần quen `async`/`await` ([bài 21](/articles/python-asyncio-coroutine-task-timeout-cancellation)) và hàng đợi cùng việc xử lý đồng thời ([bài 23](/articles/python-concurrency-lock-queue-semaphore)). Kafka chạy sẵn trong hạ tầng dùng chung (`localhost:9094`, xem `infra/README.md` của loạt bài). Mọi topic của lab có tiền tố `lab38.` và bị xoá khi chạy xong; bài dùng `aiokafka`, client Kafka bất đồng bộ cho `asyncio`.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Consumer xử lý xong một sự kiện thì tiến trình chết, trước khi kịp commit offset. Khởi động lại, sự kiện đó bị xử lý hai lần hay bị mất?</p>
<details><summary>Xem đáp án</summary><p><b>Bị xử lý hai lần</b>, nếu ta commit <i>sau</i> khi xử lý (at-least-once). Bị <b>mất</b> chỉ khi commit <i>trước</i> khi xử lý (at-most-once). Bài chạy cả hai và đếm: 5 lượt xử lý trùng với cách đầu, 1 sự kiện mất với cách sau. Vì vậy handler phải chịu được việc nhận lại cùng một sự kiện.</p></details>
</div>

## Topic, partition, key và consumer group

Một **topic** là nhật ký chỉ ghi thêm, chia thành các **partition**; mỗi bản ghi trong partition có một **offset** tăng dần từ 0. Thứ tự chỉ được bảo đảm **trong một partition**, không giữa các partition. Producer chọn partition bằng **key**: cùng key đi cùng partition. Với `BookingCreated` ta lấy key là `customer_id`, nên mọi sự kiện của một khách nằm trong cùng một partition và đến theo thứ tự đặt. **Consumer group** là một nhóm consumer chia nhau các partition: mỗi partition thuộc đúng một thành viên trong group tại một thời điểm, và mỗi group giữ offset đã commit riêng.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 340" role="img" aria-label="Producer băm key customer_id để chọn một trong ba partition. Partition 0 chứa khách CU-003, partition 1 chứa CU-001 và CU-002, partition 2 chứa CU-004, CU-005 và CU-006. Trong group lab38-fraud-scoring, consumer E đọc partition 0 và 2, consumer F đọc partition 1."><defs><marker id="py38-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="110" width="200" height="120" rx="10" class="f-pk-soft s-pk"/><text x="36" y="140" class="t-b">Producer</text><text x="36" y="166" class="t-small">key = customer_id</text><text x="36" y="186" class="t-small">murmur2(key) % 3</text><text x="36" y="206" class="t-small">acks=all, idempotent</text><rect x="320" y="24" width="300" height="76" rx="10" class="f-phys-soft s-phys"/><text x="336" y="52" class="t-b">partition 0</text><text x="336" y="76" class="t-small">CU-003: 5 sự kiện</text><rect x="320" y="132" width="300" height="76" rx="10" class="f-phys-soft s-phys"/><text x="336" y="160" class="t-b">partition 1</text><text x="336" y="184" class="t-small">CU-001, CU-002: 10 sự kiện</text><rect x="320" y="240" width="300" height="76" rx="10" class="f-phys-soft s-phys"/><text x="336" y="268" class="t-b">partition 2</text><text x="336" y="292" class="t-small">CU-004, CU-005, CU-006: 15 sự kiện</text><path d="M220 150 L320 62" class="f-none s-phys" stroke-width="2" marker-end="url(#py38-0-arrow)"/><path d="M220 170 L320 170" class="f-none s-phys" stroke-width="2" marker-end="url(#py38-0-arrow)"/><path d="M220 190 L320 278" class="f-none s-phys" stroke-width="2" marker-end="url(#py38-0-arrow)"/><rect x="690" y="70" width="170" height="200" rx="10" class="f-warn-soft s-warn"/><text x="706" y="98" class="t-b">group</text><text x="706" y="120" class="t-small">lab38-fraud-scoring</text><rect x="706" y="140" width="138" height="48" rx="6" class="f-surface s-line"/><text x="716" y="160" class="t-b">E</text><text x="716" y="178" class="t-small">partition 0, 2</text><rect x="706" y="200" width="138" height="48" rx="6" class="f-surface s-line"/><text x="716" y="220" class="t-b">F</text><text x="716" y="238" class="t-small">partition 1</text><path d="M620 62 L690 150" class="f-none s-warn" stroke-width="2" marker-end="url(#py38-0-arrow)"/><path d="M620 278 L690 178" class="f-none s-warn" stroke-width="2" marker-end="url(#py38-0-arrow)"/><path d="M620 170 L690 224" class="f-none s-warn" stroke-width="2" marker-end="url(#py38-0-arrow)"/></svg>
</div>
<figcaption>Hình 1. Key quyết định partition; group quyết định ai đọc partition nào (phân công E, F trong hình là của một lần chạy). Sự kiện cùng khách luôn nằm cùng một partition nên giữ thứ tự.</figcaption>
</figure>

Lab tạo topic `lab38.booking.created.v1` với 3 partition bằng admin client, rồi gửi 30 sự kiện (6 khách, mỗi khách 5 chuyến). Producer (trích `main.py`):

```python
producer = AIOKafkaProducer(
    bootstrap_servers=BOOTSTRAP,
    acks="all",  # chờ broker xác nhận
    enable_idempotence=True,  # gửi lại do lỗi mạng không tạo bản trùng trong partition
)
await producer.start()
for event, event_id in events:
    value = encode(event, event_id=event_id, occurred_at=event.requested_at)
    # key quyết định partition: cùng khách -> cùng partition -> giữ thứ tự theo khách
    futures.append(await producer.send(TOPIC, value, key=event.customer_id.encode()))
await producer.flush()
```

`send` trả về một future; lô được gửi nền và `flush` chờ tất cả xong, mỗi future cho ra partition và offset. `acks="all"` chờ mọi replica đồng bộ xác nhận (lab chỉ có một broker nên khác biệt chưa thể hiện); `enable_idempotence=True` làm broker bỏ qua bản gửi lại do lỗi mạng, nhưng chỉ trong một lần sống của producer, không thay thế được việc khử trùng lặp ở consumer. Output (trích):

```text
khách -> partition: {'CU-001': [1], 'CU-002': [1], 'CU-003': [0], 'CU-004': [2], 'CU-005': [2], 'CU-006': [2]}
số sự kiện mỗi partition: {0: 5, 1: 10, 2: 15}
12 sự kiện của CU-001, có key: partition [1]; không key: [0, 1, 2]
```

Hai dòng đầu cho thấy hai điều. Mỗi khách đúng một partition, nhưng **phân bố không đều**: hash trên sáu khoá dồn ba khách vào partition 2 (15 sự kiện, gấp ba partition 0). Đây là mầm của partition nóng; key càng ít giá trị hoặc càng lệch thì càng nặng. Dòng thứ ba là lý do cần key: không key, 12 sự kiện của cùng một khách rải ra cả ba partition, và thứ tự theo khách mất. Cần thêm một lưu ý khi nhiều ngôn ngữ cùng ghi vào một topic: aiokafka băm key bằng murmur2, giống client Java; còn librdkafka (nền của client C#) mặc định dùng CRC32. Producer C# của Dispatch phải đặt `partitioner` thành `murmur2_random` (trong Confluent.Kafka là `Partitioner.Murmur2Random`), nếu không cùng một khách có thể rơi vào hai partition tuỳ bên gửi. Ta thử riêng với librdkafka (nền của client C#, gọi qua binding Python) trên cùng topic ba partition và sáu khách:

| Cách chọn partition | CU-001 … CU-006 |
| --- | --- |
| aiokafka (murmur2) | 1, 1, 0, 2, 2, 2 |
| librdkafka `murmur2_random` | 1, 1, 0, 2, 2, 2 |
| librdkafka `consistent_random` (mặc định) | 0, 2, 1, 0, 1, 2 |

Hai hàng đầu khớp nhau, hàng mặc định thì lệch. Ta chưa chạy chính client C#; kết luận dựa trên việc nó dùng cùng librdkafka.

Cuối cùng, đo cho thấy gọi `send_and_wait` cho từng sự kiện chậm hơn nhiều so với gửi theo lô:

```text
tải máy lúc đo (load average 1 phút): 52.1
send_and_wait 200 lần: 140 ms (1434 sự kiện/giây)
send + flush 5000 lần: 67 ms (74393 sự kiện/giây)
```

Máy Apple M1 Max, macOS, Kafka một broker trong Docker cùng máy; dòng đầu là tải máy do lab tự in (load average 1 phút 52 vì nhiều tác vụ khác đang chạy), nên chỉ đọc theo bậc độ lớn. `send_and_wait` mỗi lần chờ một vòng đi-về; gửi theo lô gom nhiều bản ghi vào một yêu cầu (`linger_ms=10` cho phép chờ tối đa 10 ms để gom). Dùng `send_and_wait` khi cần biết chắc từng sự kiện đã ghi (ví dụ trước khi trả lời khách); gửi theo lô khi thông lượng quan trọng.

## Envelope và schema version

Sự kiện đi trên dây là JSON bọc trong một **envelope**: `event_id`, `event_type`, `occurred_at` (ISO UTC), `version` và `payload`. `event_id` cho phép khử trùng lặp; `version` cho phép hình dạng payload tiến hoá. Version 1 có `trip_id`, `customer_id`, `service`, `distance_km` (chuỗi, như `Money`) và `requested_at`; version 2 thêm `device_id`. Phần giải mã (trích `envelope.py`):

```python
def decode(raw: bytes) -> Decoded:
    try:
        env = json.loads(raw)
    except (UnicodeDecodeError, json.JSONDecodeError) as exc:
        raise EnvelopeError("invalid_json", str(exc)) from None
    if env.get("event_type") != EVENT_TYPE:
        raise EnvelopeError("wrong_event_type", repr(env.get("event_type")))
    version = env.get("version")
    if type(version) is not int or version not in SUPPORTED:
        # kiểm TRƯỚC khi đọc payload: payload của version lạ có thể đã đổi nghĩa
        raise EnvelopeError("unsupported_version", repr(version))
    ...
        device_id=p.get("device_id"),  # v1 không có -> None; field lạ khác bị bỏ qua
```

Ba quy tắc nằm trong ba dòng này. **Người đọc khoan dung**: bỏ qua field lạ và coi field tuỳ chọn vắng là `None`, nên thêm field là thay đổi tương thích. **Kiểm version trước payload**, vì payload của version lạ có thể đổi nghĩa mà ta không biết. **Từ chối có lý do**: `EnvelopeError.reason` là mã ổn định cho log và metric, không phải câu văn. Ta gửi năm thông điệp vào một topic riêng rồi đọc bằng consumer không qua group (`assign` và `seek_to_beginning`, không phải chờ cân bằng nhóm):

```text
offset 0: v1 ok device_id=None
offset 1: v2 ok device_id=DEV-0001
offset 2: v2 ok device_id=DEV-0001
offset 3: rejected unsupported_version
offset 4: rejected invalid_json
version méo kiểu (list, dict, chuỗi, true) và payload không phải object: ['unsupported_version', 'unsupported_version', 'unsupported_version', 'unsupported_version', 'invalid_payload']
```

Offset 2 là bản v2 có thêm field `coupon` mà consumer không biết: vẫn đọc được. Offset 3 là version 3, offset 4 là chuỗi không phải JSON. Hai thông điệp bị từ chối nhưng consumer phải **commit vượt qua chúng và đi tiếp**: một poison message không đọc nổi mà không bao giờ được commit sẽ chặn cả partition. Lab chỉ đếm và bỏ qua; nơi lưu poison message (ví dụ một topic riêng cho thông điệp hỏng) do bạn tự thiết kế, còn dead-letter của bài 40 là cho job nền trong Redis, không phải cho Kafka. Phải kiểm tra kiểu trước khi tra cứu: `version` có thể là list, dict hay `true` do bên gửi lỗi, nên `decode` chỉ nhận đúng `int` (`true` bằng 1 trong Python, không được lọt). Dòng cuối của output trên là năm thông điệp méo kiểu như vậy, đều bị từ chối bằng `EnvelopeError` thay vì ném `TypeError` ra ngoài. Schema registry với Avro, Protobuf hoặc JSON Schema là cách quản lý hợp đồng chặt hơn; lab dùng một file `envelope.py` làm hợp đồng và kiểm bằng `--verify`. Một quy tắc triển khai đi kèm: khi bắt đầu phát version mới, **consumer phải biết version đó trước producer**, vì sự kiện đã vào Kafka thì không rút lại được.

## Consumer group và commit offset

Consumer của lab (trích `main.py`):

```python
def make_consumer(group: str) -> AIOKafkaConsumer:
    return AIOKafkaConsumer(
        TOPIC,
        bootstrap_servers=BOOTSTRAP,
        group_id=group,
        enable_auto_commit=False,  # ta tự commit SAU khi xử lý xong: at-least-once
        auto_offset_reset="earliest",  # group mới thì đọc từ đầu
    )


async def commit_through(consumer, rec) -> None:
    """Commit offset TIẾP THEO của partition: Kafka lưu 'sẽ đọc từ đâu', không phải 'đã đọc xong'."""
    await consumer.commit({TopicPartition(rec.topic, rec.partition): rec.offset + 1})
```

`enable_auto_commit` mặc định là `True`: aiokafka commit vị trí đã đọc theo chu kỳ (mặc định 5 giây), không biết ta đã xử lý xong chưa. Phải tắt đi và tự commit thì mới kiểm soát được thời điểm commit. Offset được commit là offset của bản ghi **sẽ đọc tiếp**, nên là `offset + 1`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Hai dải offset từ 0 đến 7. Dải trên, commit sau xử lý: offset 0 đến 2 đã xử lý và commit, offset 3 và 4 đã xử lý nhưng chưa commit nên sẽ được đọc lại. Dải dưới, commit trước xử lý: offset 0 đến 2 đã xong, offset 3 đã commit nhưng chưa xử lý nên bị mất."><rect x="200" y="50" width="56" height="44" rx="6" class="f-phys-soft s-phys"/><text x="222" y="77" class="t-mono">0</text><rect x="264" y="50" width="56" height="44" rx="6" class="f-phys-soft s-phys"/><text x="286" y="77" class="t-mono">1</text><rect x="328" y="50" width="56" height="44" rx="6" class="f-phys-soft s-phys"/><text x="350" y="77" class="t-mono">2</text><rect x="392" y="50" width="56" height="44" rx="6" class="f-warn-soft s-warn"/><text x="414" y="77" class="t-mono">3</text><rect x="456" y="50" width="56" height="44" rx="6" class="f-warn-soft s-warn"/><text x="478" y="77" class="t-mono">4</text><rect x="520" y="50" width="56" height="44" rx="6" class="f-surface s-line"/><text x="542" y="77" class="t-mono">5</text><rect x="584" y="50" width="56" height="44" rx="6" class="f-surface s-line"/><text x="606" y="77" class="t-mono">6</text><rect x="648" y="50" width="56" height="44" rx="6" class="f-surface s-line"/><text x="670" y="77" class="t-mono">7</text><text x="20" y="70" class="t-b">Commit sau xử lý</text><text x="20" y="88" class="t-small">(at-least-once)</text><text x="200" y="120" class="t-small">đã commit: 3. Sập sau khi xử lý 3 và 4: khởi động lại đọc từ 3,</text><text x="200" y="138" class="t-small">nên 3 và 4 được xử lý hai lần.</text><rect x="200" y="190" width="56" height="44" rx="6" class="f-phys-soft s-phys"/><text x="222" y="217" class="t-mono">0</text><rect x="264" y="190" width="56" height="44" rx="6" class="f-phys-soft s-phys"/><text x="286" y="217" class="t-mono">1</text><rect x="328" y="190" width="56" height="44" rx="6" class="f-phys-soft s-phys"/><text x="350" y="217" class="t-mono">2</text><rect x="392" y="190" width="56" height="44" rx="6" class="f-none s-dead dash"/><text x="414" y="217" class="t-mono t-dead">3</text><rect x="456" y="190" width="56" height="44" rx="6" class="f-surface s-line"/><text x="478" y="217" class="t-mono">4</text><rect x="520" y="190" width="56" height="44" rx="6" class="f-surface s-line"/><text x="542" y="217" class="t-mono">5</text><rect x="584" y="190" width="56" height="44" rx="6" class="f-surface s-line"/><text x="606" y="217" class="t-mono">6</text><rect x="648" y="190" width="56" height="44" rx="6" class="f-surface s-line"/><text x="670" y="217" class="t-mono">7</text><text x="20" y="210" class="t-b">Commit trước xử lý</text><text x="20" y="228" class="t-small">(at-most-once)</text><text x="200" y="260" class="t-small">đã commit: 4, rồi sập trước khi xử lý 3: khởi động lại đọc từ 4,</text><text x="200" y="278" class="t-small">nên sự kiện 3 không bao giờ được xử lý.</text><rect x="20" y="302" width="16" height="16" rx="3" class="f-phys-soft s-phys"/><text x="44" y="315" class="t-small">đã xử lý và commit</text><rect x="260" y="302" width="16" height="16" rx="3" class="f-warn-soft s-warn"/><text x="284" y="315" class="t-small">đã xử lý, chưa commit</text><rect x="520" y="302" width="16" height="16" rx="3" class="f-none s-dead dash"/><text x="544" y="315" class="t-small">đã commit, chưa xử lý</text></svg>
</div>
<figcaption>Hình 2. Vị trí commit nằm lệch so với vị trí xử lý một khoảng; sập trong khoảng đó thì sự kiện hoặc bị lặp (commit sau) hoặc bị mất (commit trước).</figcaption>
</figure>

Lab tái hiện cả hai. Ca đầu (commit sau xử lý): consumer A xử lý 12 sự kiện và commit từng cái, xử lý thêm 5 rồi «sập» (đóng consumer mà không commit gì thêm); consumer B cùng group khởi động và đọc nốt:

```python
for _ in range(12):  # 12 sự kiện đầu: xử lý rồi commit
    rec = await asyncio.wait_for(a.getone(), 10)
    a_seen.append(handle(rec))
    await commit_through(a, rec)
for _ in range(5):  # 5 sự kiện tiếp: xử lý xong nhưng "sập" trước khi commit
    rec = await asyncio.wait_for(a.getone(), 10)
    a_seen.append(handle(rec))
```

```text
consumer A: xử lý 17, đã commit 12, rồi sập
consumer B (cùng group): nhận 18; tổng lượt xử lý 35 cho 30 sự kiện khác nhau; xử lý trùng 5
```

B bắt đầu từ offset đã commit, nên nhận 30 − 12 = 18 sự kiện, trong đó 5 sự kiện A đã xử lý. Không mất sự kiện nào; đổi lại là có 5 lượt trùng. Ca thứ hai đảo thứ tự: commit trước, xử lý sau, sập ở sự kiện thứ ba:

```text
đã xử lý 29 / 30; sự kiện mất: 1
```

Với dữ liệu tài chính hay chấm điểm, mất sự kiện thường tệ hơn trùng, nên lựa chọn thông dụng là at-least-once cộng handler **idempotent**: cùng `event_id` đến lần thứ hai không tạo thêm hiệu ứng. Lab cho thấy khử trùng lặp theo `event_id` đưa 35 lượt về đúng 30 sự kiện; cách làm bền (bảng inbox với khoá duy nhất) là bài 39. Một chi tiết sát thực tế: lab «sập» bằng cách đóng consumer, nên consumer rời group ngay và B vào được ngay. Nếu tiến trình bị kill thật thì broker chỉ biết sau `session_timeout_ms` (mặc định của aiokafka là 10 giây), và trong lúc đó các partition của nó đứng yên.

## Chia việc trong group

Hai consumer cùng group chia partition cho nhau. Lab khởi động E và F cùng lúc, chờ phân công ổn định rồi cho cả hai đọc hết:

```text
phân công (đổi theo lần chạy): {'E': [0, 2], 'F': [1]}
số sự kiện mỗi consumer: {'E': 20, 'F': 10}
```

Hai tập partition rời nhau và hợp lại đủ ba; mỗi sự kiện được đúng một consumer xử lý. Phân công cụ thể **đổi theo từng lần chạy** (lần khác E có thể nhận partition 1 còn F nhận 0 và 2); lab chỉ kiểm bất biến, và số sự kiện mỗi consumer luôn bằng tổng các partition được giao. Ở lần chạy này E giữ hai partition, trong đó có partition 2 là partition nóng, nên nhận nhiều gấp đôi. Từ đây rút ra giới hạn: số consumer có việc trong một group không vượt quá số partition (consumer thứ tư trong group ba partition ngồi không), và thêm consumer không giải quyết được vấn đề của một partition nóng vì nó vẫn chỉ có một người đọc. Mỗi lần thành viên vào hoặc rời, group **cân bằng lại** và việc đọc tạm dừng; khi nhóm đang rỗng, broker chờ thêm một khoảng thời gian trước khi chia việc cho thành viên đầu tiên. Lab đọc cấu hình đó qua admin client (dòng `broker group.initial.rebalance.delay.ms = 3000` ở đầu output) và đo `start()` của consumer đầu tiên: 3,01 giây. Cộng thêm vài giây chờ không còn tin mới để biết đã đọc hết, đó là lý do ba phần dùng consumer group mất 9,1 giây, 9,1 giây và 8,0 giây trong tổng khoảng 30 giây (các dòng `[… giây]` do lab in). Thứ tự theo khách vẫn giữ: lab kiểm rằng trong mỗi consumer, các chuyến của cùng một khách được xử lý theo thứ tự tăng dần.

## Chạy và kiểm chứng

Cần uv, Python 3.14 và Kafka trong hạ tầng dùng chung đang chạy. Phiên bản đã dùng: Python 3.14.7, aiokafka 0.15.0, broker Kafka 3.9.0. Từ thư mục gốc repository:

```bash
docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait
uv run --project examples/toi-noi-python/38-kafka-aiokafka-producer-consumer examples/toi-noi-python/38-kafka-aiokafka-producer-consumer/main.py --verify
```

```text
OK 26/26
```

26 ca gồm: 5 ca producer (mỗi khách đúng một partition, tổng 30 sự kiện, offset liên tục trên ba partition); 2 ca key và không key; 1 ca đo gửi theo lô; 6 ca schema (5 qua Kafka, 1 cho envelope méo kiểu); 4 ca at-least-once; 1 ca commit trước xử lý; 6 ca chia partition và thứ tự; 1 ca topic đã được xoá sạch. Mất khoảng 30 giây. Khi chưa có Kafka, `--verify` in `FAIL: không nối được Kafka tại <url> (docker compose ...)` và thoát mã 1. Để chắc `--verify` đỏ khi nên đỏ, ta thử phá: trong bản sao ở thư mục tạm, bỏ `key=` khi gửi.

```text
SAI: mỗi khách đúng một partition: nhận False, kỳ vọng True
SAI: E: thứ tự chuyến theo khách giữ nguyên: nhận False, kỳ vọng True
FAIL 24/26
```

Hai ca hỏng đúng chỗ ta nói: không key thì khách rải nhiều partition và thứ tự theo khách mất. `FAIL 24/26` nghĩa là 24 ca đạt trên 26. Các con số mili giây phụ thuộc máy; ánh xạ khách sang partition và số lượt trùng lặp lại được vì seed cố định và hash của key xác định; phân công partition cho E và F thì không.

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| Consumer mới mất vài giây mới nhận bản ghi đầu | cân bằng nhóm; broker đợi `group.initial.rebalance.delay.ms` khi nhóm rỗng |
| Cùng khách bị xử lý không theo thứ tự | producer có `key` không; mọi producer của topic có cùng thuật toán chọn partition không |
| Xử lý trùng sau khi deploy hoặc restart | bình thường với at-least-once; handler có khử trùng lặp theo `event_id` không |
| `CommitFailedError` | group đã cân bằng lại khi ta xử lý quá lâu; xử lý nhanh hơn hoặc chỉnh `max_poll_interval_ms` |
| Offset của một partition không bao giờ tăng | có poison message chưa được commit vượt qua; từ chối có lý do rồi commit |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>Một broker, replication factor 1, không TLS và không xác thực: production có nhiều broker, <code>min.insync.replicas</code> lớn hơn 1 và kết nối được bảo vệ. Idempotent producer chỉ chống trùng trong một lần sống của producer; ghi vào Kafka và ghi vào database vẫn là hai lần ghi riêng (khoảng trống mà bài 39 xử lý bằng outbox). Lab khử trùng lặp bằng danh sách trong bộ nhớ, không bền. «Sập» là đóng consumer, không phải kill tiến trình, và 30 sự kiện là quá nhỏ để bàn về lag hay thông lượng. Số đo trên một máy bận, chỉ để thấy bậc độ lớn. Điểm fraud mà consumer tính ra (bài 37) chỉ là đầu vào cho người duyệt, không phải quyết định.</p>
</div>

**Bài tập 1:** Vì sao `commit_through` commit `offset + 1`? Nếu commit `offset`, điều gì xảy ra khi consumer khởi động lại?

**Bài tập 2:** Ta muốn phát `BookingCreated` version 3 có thêm `pickup_city` bắt buộc ở payload. Cần đổi gì ở consumer và theo thứ tự triển khai nào để consumer cũ không gãy?

<details><summary>Xem lời giải</summary><p>(1) Kafka lưu offset của bản ghi sẽ đọc <b>tiếp theo</b>. Commit <code>offset</code> thay vì <code>offset + 1</code> nghĩa là «hãy đọc lại từ chính bản ghi vừa xử lý», nên mỗi lần khởi động lại mỗi partition lặp đúng một sự kiện cuối, kể cả khi không có sự cố nào. (2) Trước hết thêm 3 vào <code>SUPPORTED</code> và đọc <code>pickup_city</code> cho version 3 (version cũ không có thì coi là <code>None</code>); deploy <b>mọi consumer trước</b>, rồi mới cho producer phát version 3. Nếu producer đi trước, consumer cũ gặp <code>unsupported_version</code> và sự kiện bị từ chối. Field bắt buộc mới không tương thích với producer cũ, nên giai đoạn chuyển tiếp consumer phải chịu cả hai: version 1 và 2 không có <code>pickup_city</code>, version 3 có. Chỉ khi không còn producer nào phát version cũ mới được coi nó là bắt buộc ở phía nhận.</p></details>

Bài trước: [bài 37](/articles/python-grpc-fraud-scoring-service). Bài tiếp: [bài 39](/articles/python-outbox-inbox-idempotent-consumer).

## Tài liệu gốc

- [aiokafka: Consumer client](https://aiokafka.readthedocs.io/en/stable/consumer.html).
- [aiokafka: Producer client](https://aiokafka.readthedocs.io/en/stable/producer.html).
- [aiokafka: API documentation](https://aiokafka.readthedocs.io/en/stable/api.html).
- [Apache Kafka: Message delivery semantics](https://kafka.apache.org/39/documentation/#design_deliverysemantics).
- [Apache Kafka: Consumer configs](https://kafka.apache.org/39/documentation/#consumerconfigs).
- [librdkafka: Configuration (partitioner)](https://github.com/confluentinc/librdkafka/blob/master/CONFIGURATION.md).
