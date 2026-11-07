---
slug: python-outbox-inbox-idempotent-consumer
title: "Outbox, Inbox và idempotent consumer trong Python"
summary: "Bài 39: ghi chuyến và outbox trong một transaction, dispatcher thuê dòng bằng SKIP LOCKED rồi phát Kafka, consumer khử trùng bằng inbox. Tái hiện crash sinh thông điệp trùng trên PostgreSQL và Kafka thật."
category: python-intermediate
level: intermediate
tags: [python, toi-noi, outbox, kafka, idempotency]
language: vi
---

Khi một chuyến của [Tới Nơi](/articles/du-an-mau-toi-noi) kết thúc, ít nhất hai service phía sau cần biết: thanh toán ghi tiền chuyến cho tài xế, và `fraud-scoring` bằng Python chấm điểm cho chuyến vừa xong. Service chuyến giữ trạng thái `completed` trong PostgreSQL của nó, còn việc báo cho hai service kia đi qua Kafka. Ở cao điểm khoảng 70 chuyến mỗi giây, hai lỗi hiếm vẫn xảy ra hằng ngày. Một: chuyến đã `completed` mà sự kiện không bao giờ rời khỏi service, nên tài xế thiếu tiền và chuyến không được chấm điểm. Hai: cùng một sự kiện đến hai lần, và tài xế được trả hai lần.

Đây là bài 39 của lộ trình **Python trung bình qua Tới Nơi**. Bạn cần nắm transaction và khóa idempotency ở [bài 25](/articles/python-transaction-optimistic-concurrency-idempotency), cùng producer, consumer và commit offset ở [bài 38](/articles/python-kafka-aiokafka-producer-consumer-schema). Loạt C# có bài cùng chủ đề là [Outbox và Inbox bằng .NET 10 và PostgreSQL](/articles/dotnet10-outbox-inbox-idempotent-consumer): mô hình giống nhau, nhưng bài C# dùng publisher giả trong bộ nhớ và phát `TripCreated`, chưa có lease hay `SKIP LOCKED`, và bắt SQLSTATE 23505 của khóa unique, còn lab Python này chạy với Kafka thật, phát `TripCompleted`, và dùng `ON CONFLICT DO NOTHING`. Lab dựng cả hai phía trong một database cho gọn, với hai consumer là `payout` (ghi tiền chuyến) và `fraud-scoring` (xếp yêu cầu chấm điểm); ở production mỗi service có database riêng.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Service chuyến commit trạng thái <code>completed</code> vào PostgreSQL, rồi mới publish lên Kafka. Tiến trình bị kill đúng giữa hai bước. Khi khởi động lại, làm sao nó biết có một sự kiện chưa được phát?</p>
<details><summary>Xem đáp án</summary><p>Không biết được. Trạng thái <code>completed</code> không cho biết sự kiện đã publish hay chưa, và ý định «cần publish» chỉ nằm trong bộ nhớ của tiến trình đã chết. Muốn khởi động lại phát được phần còn thiếu, ý định đó phải được lưu bền <b>cùng transaction</b> với thay đổi nghiệp vụ. Đó là outbox.</p></details>
</div>

## Dual-write: hai nơi, một khoảng trống

PostgreSQL và Kafka là hai hệ thống độc lập, không có transaction chung. Dù ta ghi bên nào trước, giữa hai lần ghi luôn có một khoảng mà tiến trình có thể chết. `naive.py` của lab làm cả hai thứ tự và mô phỏng cái chết bằng một exception `SimulatedCrash`. Thứ tự thứ nhất commit database rồi mới publish; thứ tự thứ hai publish rồi mới commit:

```text
Commit rồi publish, chết giữa chừng: chuyến TN-0101 = completed, sự kiện trên topic = 0
Publish rồi commit, database lỗi: chuyến TN-0102 = in_transit, sự kiện trên topic = 1
```

Dòng đầu là **sự kiện mất**: chuyến đã xong nhưng không consumer nào được biết, và không có dấu vết nào để phát lại. Dòng hai là **sự kiện ma**: consumer nhận tin về một chuyến mà database chưa (và có thể không bao giờ) hoàn thành. Thử bọc `send` trong vòng retry cũng không chữa được, vì retry chỉ cứu lỗi mạng chớp nhoáng; khi tiến trình chết thì ý định gửi biến mất cùng bộ nhớ. Cách khác là đưa ý định gửi vào chính database, nơi nó sống sót qua mọi lần chết của tiến trình.

## Outbox: ý định phát nằm trong cùng transaction

**Outbox** là một bảng trong database của service. Khi hoàn thành chuyến, ta ghi hai thứ trong **một transaction**: cập nhật `trips`, và thêm một dòng `outbox` mô tả sự kiện cần phát. Transaction commit thì cả hai tồn tại; rollback thì cả hai biến mất. Việc gửi ra Kafka được tách khỏi request, giao cho một tiến trình khác (dispatcher) đọc outbox và phát.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Luồng outbox và inbox. Service chuyến ghi trips và outbox trong một transaction. Dispatcher thuê dòng, publish rồi đánh dấu. Kafka chuyển thông điệp. Consumer ghi inbox và hiệu ứng trong một transaction. Hai cửa sổ trùng nằm sau publish và sau xử lý."><defs><marker id="py39-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="30" width="250" height="170" rx="10" class="f-pk-soft s-pk"/><text x="36" y="56" class="t-b">Service chuyến</text><text x="36" y="74" class="t-small">một transaction</text><rect x="36" y="90" width="218" height="44" rx="6" class="f-bg s-line"/><text x="48" y="117" class="t-small">trips: status = completed</text><rect x="36" y="146" width="218" height="44" rx="6" class="f-bg s-line"/><text x="48" y="173" class="t-small">outbox: dòng TripCompleted</text><path d="M270 115 L298 115" class="f-none s-phys" stroke-width="2" marker-end="url(#py39-0-arrow)"/><rect x="300" y="30" width="190" height="170" rx="10" class="f-phys-soft s-phys"/><text x="316" y="56" class="t-b">Dispatcher</text><text x="316" y="92" class="t-small">1. thuê dòng (lease)</text><text x="316" y="114" class="t-small">2. send_and_wait</text><text x="316" y="136" class="t-small">3. đánh dấu published_at</text><text x="316" y="172" class="t-small">ngoài mọi transaction</text><path d="M490 115 L518 115" class="f-none s-phys" stroke-width="2" marker-end="url(#py39-0-arrow)"/><rect x="520" y="60" width="120" height="110" rx="10" class="f-bg s-muted"/><text x="536" y="88" class="t-b">Kafka</text><text x="536" y="112" class="t-small">lab39.trip.</text><text x="536" y="128" class="t-small">completed.v1</text><text x="536" y="148" class="t-small">3 partition</text><path d="M640 115 L668 115" class="f-none s-phys" stroke-width="2" marker-end="url(#py39-0-arrow)"/><rect x="670" y="30" width="190" height="170" rx="10" class="f-pk-soft s-pk"/><text x="686" y="56" class="t-b">Consumer</text><text x="686" y="74" class="t-small">một transaction</text><rect x="686" y="90" width="158" height="44" rx="6" class="f-bg s-line"/><text x="698" y="117" class="t-small">inbox: INSERT</text><rect x="686" y="146" width="158" height="44" rx="6" class="f-bg s-line"/><text x="698" y="173" class="t-small">payout_lines: INSERT</text><rect x="300" y="226" width="250" height="56" rx="8" class="f-warn-soft s-warn"/><text x="314" y="250" class="t-b">Cửa sổ trùng 1</text><text x="314" y="268" class="t-small">đã publish, chưa kịp đánh dấu</text><rect x="610" y="226" width="250" height="56" rx="8" class="f-warn-soft s-warn"/><text x="624" y="250" class="t-b">Cửa sổ trùng 2</text><text x="624" y="268" class="t-small">đã xử lý, chưa commit offset</text><path d="M395 226 L395 202" class="f-none s-warn dash" stroke-width="1.5"/><path d="M765 226 L765 202" class="f-none s-warn dash" stroke-width="1.5"/></svg>
</div>
<figcaption>Hình 1. Hai nửa của đường đi: phía phát giữ ý định bền trong cùng transaction, phía nhận ghi dấu đã xử lý trong cùng transaction với hiệu ứng. Giữa hai nửa là các bước có thể lặp lại, và chúng sinh ra thông điệp trùng.</figcaption>
</figure>

Bảng `outbox` (`models.py`, trích) có đủ cột để dispatcher làm việc mà không cần trạng thái ngoài database:

```python
class OutboxEvent(Base):
    __tablename__ = "outbox"

    seq: Mapped[int] = mapped_column(BigInteger, Identity(), primary_key=True)  # thứ tự phát
    event_id: Mapped[str] = mapped_column(String(36), unique=True)  # danh tính ổn định của sự kiện
    topic: Mapped[str] = mapped_column(String(128))
    message_key: Mapped[str] = mapped_column(String(64))
    envelope: Mapped[dict[str, Any]] = mapped_column(JSONB)
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True))
    published_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True))  # NULL = chưa phát
    lease_until: Mapped[datetime | None] = mapped_column(DateTime(timezone=True))  # «thuê» của dispatcher
    attempts: Mapped[int] = mapped_column(default=0)
```

Và `complete_trip` (`outbox.py`, trích) ghi chuyến cùng dòng outbox:

```python
with Session(engine) as session, session.begin():
    trip = session.get(Trip, trip_id, with_for_update=True)
    if trip is None or trip.status != "in_transit":
        raise TripNotInTransit(trip_id)
    trip.status = "completed"
    trip.completed_at = now
    trip.fare = calculate_fare(trip.distance_km)
    event_id = str(uuid.uuid4())  # sinh MỘT lần, lưu cùng outbox; retry không bao giờ sinh id mới
    session.add(
        OutboxEvent(
            event_id=event_id,
            topic=TOPIC,
            message_key=trip.trip_id,  # cùng chuyến luôn vào cùng partition
            envelope=trip_completed_envelope(trip, event_id=event_id, occurred_at=now),
            created_at=now,
        )
    )
```

Ba chi tiết đáng nhìn. **Một**, `with_for_update=True` khóa dòng chuyến để hai lần hoàn thành cùng lúc không chen nhau; lần thứ hai thấy `completed` và nhận `TripNotInTransit`, nên không sinh dòng outbox thứ hai. Lab chỉ kiểm tuần tự (lần hai gọi sau lần một); an toàn khi đồng thời dựa vào khóa dòng của PostgreSQL, và lab không dựng cuộc đua đó. **Hai**, `event_id` được sinh đúng một lần và nằm trong dòng outbox; dispatcher về sau phát nguyên envelope đã lưu, nên dù phát lại bao nhiêu lần, mọi bản sao mang cùng một `event_id`. Điều này cốt lõi cho inbox ở phần sau. **Ba**, envelope theo từ điển miền (`event_id`, `event_type`, `occurred_at` UTC, `version`, `payload`), tiền `Decimal` đi qua JSON dưới dạng chuỗi, và key của thông điệp là `trip_id` để mọi sự kiện của một chuyến vào cùng partition.

Lab kiểm hai điều: lỗi xảy ra trước commit thì chuyến và dòng outbox cùng rollback, và hoàn thành 5 chuyến thì có đúng 5 dòng chờ phát.

```text
Lỗi trước commit: chuyến TN-0001 = in_transit, dòng outbox = 0 (cả hai cùng rollback)
Hoàn thành 5 chuyến: 5 chuyến completed, 5 dòng outbox chờ phát
```

## Dispatcher: thuê, phát, đánh dấu

Dispatcher là vòng lặp đọc các dòng `published_at IS NULL`, phát lên Kafka rồi đánh dấu đã phát. Nó có hai yêu cầu mâu thuẫn nhau: nhiều dispatcher chạy song song để chịu tải và chịu lỗi, nhưng không được giữ transaction mở trong lúc gọi Kafka (đúng lời khuyên ở bài 25: transaction dài giữ khóa và làm nghẽn bảng). Lab giải bằng **lease**: một câu `UPDATE` ngắn «thuê» một lô dòng, commit ngay, rồi phát ngoài transaction. `claim_batch` (`outbox.py`, trích):

```python
pending = (
    select(OutboxEvent.seq)
    .where(OutboxEvent.published_at.is_(None))
    .where((OutboxEvent.lease_until.is_(None)) | (OutboxEvent.lease_until < now))
    .order_by(OutboxEvent.seq)
    .limit(limit)
    .with_for_update(skip_locked=True)
)
rows = session.execute(
    update(OutboxEvent)
    .where(OutboxEvent.seq.in_(pending))
    .values(lease_until=now + lease, attempts=OutboxEvent.attempts + 1)
    .returning(OutboxEvent.seq, OutboxEvent.event_id, OutboxEvent.topic,
               OutboxEvent.message_key, OutboxEvent.envelope)
).all()
return sorted((Claimed(*row) for row in rows), key=lambda c: c.seq)  # RETURNING không bảo đảm thứ tự
```

`FOR UPDATE SKIP LOCKED` làm câu truy vấn con bỏ qua các dòng mà transaction khác đang khóa, thay vì chờ. Hai dispatcher chạy cùng lúc nhận hai tập dòng rời nhau. Lab kiểm bằng cách mở hai session, để session A thuê 3 dòng mà chưa commit, rồi cho session B thuê khi A còn giữ khóa: B nhận 2 dòng còn lại, không trùng dòng nào. Tài liệu PostgreSQL cũng nói rõ `SKIP LOCKED` cho một góc nhìn không nhất quán về dữ liệu, nên chỉ hợp với bảng kiểu hàng đợi, không hợp cho truy vấn tổng quát. Sau khi commit, khóa dòng được nhả nhưng `lease_until` ở lại: dispatcher khác bỏ qua các dòng đang được thuê cho tới khi lease hết hạn. Nếu dispatcher chết sau khi thuê, các dòng không bị kẹt mãi; hết lease là có người khác nhặt.

`dispatch_once` (trích) nối ba bước, và thứ tự của chúng là điểm mấu chốt:

```python
with Session(engine) as session, session.begin():
    claimed = claim_batch(session, now=now, lease=lease, limit=batch)
# Transaction thuê đã commit; từ đây không giữ khóa hay transaction nào trong lúc gọi Kafka.
published: list[int] = []
for item in claimed:
    value = json.dumps(item.envelope, ensure_ascii=False, separators=(",", ":")).encode()
    await producer.send_and_wait(item.topic, value, key=item.key.encode())
    published.append(item.seq)
    if crash_after_publish is not None and len(published) >= crash_after_publish:
        raise SimulatedCrash("dispatcher chết sau khi phát ..., trước khi đánh dấu")
if published:
    with Session(engine) as session, session.begin():
        session.execute(update(OutboxEvent).where(OutboxEvent.seq.in_(published)).values(published_at=now))
```

Ta **phát trước, đánh dấu sau**. Nếu đảo thứ tự, chết giữa hai bước sẽ đánh dấu «đã phát» cho thông điệp chưa hề đi, tức là mất sự kiện, đúng lỗi ta đang chạy trốn. Với thứ tự này, kết cục xấu nhất là phát ít nhất hai lần. Đây là bảo đảm **at-least-once**: không mất, có thể trùng. Producer của lab bật `enable_idempotence=True`, nhưng đừng nhầm: nó chỉ khử trùng khi *chính producer* tự gửi lại một gói tin sau lỗi mạng. Hai lần `send` của hai lần chạy dispatcher là hai thông điệp khác nhau với Kafka.

Lab ép đúng cửa sổ đó bằng `crash_after_publish=3`, rồi cho dispatcher chạy lại ở ba thời điểm (thời gian `now` được truyền vào, nên không cần `sleep`):

```text
Dispatcher chết sau 3 lần publish: topic có 3 thông điệp, outbox đánh dấu đã phát = 0
Chạy lại sau 5 giây (lease 30 giây chưa hết): thuê được 0 dòng
Chạy lại sau 60 giây: thuê 5, phát 5; topic có 8 thông điệp, 5 event_id khác nhau; đánh dấu đã phát = 5
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 230" role="img" aria-label="Bảng bốn cột theo thời gian: sau crash, sau 5 giây, sau 60 giây, sau consumer. Số thông điệp trên topic là 3, 3, 8, 8. Số dòng outbox đã đánh dấu là 0, 0, 5, 5. Số dòng tiền sau consumer là 5, ba thông điệp trùng bị bỏ."><rect x="20" y="20" width="230" height="40" class="f-bg s-line"/><rect x="250" y="20" width="150" height="40" class="f-bg s-line"/><rect x="400" y="20" width="150" height="40" class="f-bg s-line"/><rect x="550" y="20" width="150" height="40" class="f-bg s-line"/><rect x="700" y="20" width="150" height="40" class="f-bg s-line"/><text x="34" y="45" class="t-b">Thời điểm</text><text x="264" y="45" class="t-b">Sau crash</text><text x="414" y="45" class="t-b">Sau 5 giây</text><text x="564" y="45" class="t-b">Sau 60 giây</text><text x="714" y="45" class="t-b">Sau consumer</text><rect x="20" y="60" width="230" height="50" class="f-surface s-line"/><rect x="250" y="60" width="150" height="50" class="f-surface s-line"/><rect x="400" y="60" width="150" height="50" class="f-surface s-line"/><rect x="550" y="60" width="150" height="50" class="f-warn-soft s-warn"/><rect x="700" y="60" width="150" height="50" class="f-warn-soft s-warn"/><text x="34" y="89" class="t-small">Thông điệp trên topic</text><text x="264" y="89" class="t-b">3</text><text x="414" y="89" class="t-b">3</text><text x="564" y="89" class="t-b">8 (5 event_id)</text><text x="714" y="89" class="t-b">8 (5 event_id)</text><rect x="20" y="110" width="230" height="50" class="f-surface s-line"/><rect x="250" y="110" width="150" height="50" class="f-surface s-line"/><rect x="400" y="110" width="150" height="50" class="f-surface s-line"/><rect x="550" y="110" width="150" height="50" class="f-surface s-line"/><rect x="700" y="110" width="150" height="50" class="f-surface s-line"/><text x="34" y="139" class="t-small">Dòng outbox đã đánh dấu</text><text x="264" y="139" class="t-b">0</text><text x="414" y="139" class="t-b">0</text><text x="564" y="139" class="t-b">5</text><text x="714" y="139" class="t-b">5</text><rect x="20" y="160" width="230" height="50" class="f-surface s-line"/><rect x="250" y="160" width="150" height="50" class="f-surface s-line"/><rect x="400" y="160" width="150" height="50" class="f-surface s-line"/><rect x="550" y="160" width="150" height="50" class="f-surface s-line"/><rect x="700" y="160" width="150" height="50" class="f-phys-soft s-phys"/><text x="34" y="189" class="t-small">Dòng tiền cho tài xế</text><text x="264" y="189" class="t-small">chưa nhận</text><text x="414" y="189" class="t-small">chưa nhận</text><text x="564" y="189" class="t-small">chưa nhận</text><text x="714" y="189" class="t-b">5 (bỏ 3 trùng)</text></svg>
</div>
<figcaption>Hình 2. Một lần crash của dispatcher làm topic có 8 thông điệp cho 5 sự kiện. Lease chặn việc phát lại quá sớm, và inbox ở consumer biến 8 lần nhận thành 5 hiệu ứng.</figcaption>
</figure>

Kết quả thứ hai là lý do cần lease: chạy lại sau 5 giây, lô vẫn đang được thuê (lease 30 giây) nên dispatcher mới không động vào, tránh phát chồng lên một dispatcher có thể vẫn đang sống chỉ là chậm. Kết quả thứ ba là cái giá: 3 thông điệp đã đi trước khi chết được phát lần nữa, nên topic có 8 thông điệp nhưng chỉ 5 `event_id` khác nhau. Không có cách dispatcher nào tự loại bỏ chuyện này; việc khử trùng phải ở phía nhận.

## Inbox: consumer khử trùng bằng (consumer, event_id)

Thông điệp trùng không chỉ đến từ dispatcher. Consumer chết sau khi xử lý nhưng trước khi commit offset thì lần chạy sau nhận lại; rebalance consumer group cũng có thể giao lại. Vì vậy consumer luôn phải coi mỗi thông điệp có thể đến nhiều lần. **Inbox** là bảng ghi «consumer này đã xử lý sự kiện này», với khóa chính `(consumer, event_id)`. Điểm quyết định: dòng inbox và hiệu ứng nghiệp vụ phải nằm trong **cùng transaction**. `handle_trip_completed` (`inbox.py`, trích):

```python
envelope = json.loads(raw)
payload = envelope["payload"]
with Session(engine) as session, session.begin():
    inserted = session.execute(
        insert(InboxEntry)
        .values(consumer=consumer, event_id=envelope["event_id"], received_at=now)
        .on_conflict_do_nothing()
        .returning(InboxEntry.event_id)
    ).scalar_one_or_none()
    if inserted is None:
        return DUPLICATE  # (consumer, event_id) đã có: đã xử lý, bỏ qua
    session.add(PayoutLine(consumer=consumer, driver_id=payload["driver_id"],
                           trip_id=payload["trip_id"], amount=Decimal(payload["fare"]),
                           event_id=envelope["event_id"]))
return PROCESSED
```

`INSERT ... ON CONFLICT DO NOTHING RETURNING` làm việc của «thử ghi, bắt SQLSTATE 23505» ở bài 25 mà không cần bắt exception: không chèn được nghĩa là đã có. Hai trường hợp hỏng giữa chừng đều an toàn nhờ cùng transaction. Nếu chết trước commit thì cả inbox lẫn hiệu ứng cùng mất, thông điệp đến lại sẽ được xử lý đúng. Nếu ghi inbox ở transaction riêng và commit trước, một lỗi sau đó để lại dấu «đã xử lý» cho hiệu ứng chưa hề có, và sự kiện bị bỏ vĩnh viễn. Phần `consume` trong `kafka_io.py` thì commit offset **sau** khi xử lý (`enable_auto_commit=False`), đúng quy ước at-least-once của bài 38. Vì `handle_trip_completed` dùng SQLAlchemy đồng bộ, lab gọi nó qua `asyncio.to_thread` để không chặn vòng lặp sự kiện của consumer.

Khóa inbox gồm cả tên consumer, vì cùng một sự kiện được nhiều service xử lý, và mỗi service cần dấu riêng. Lab cho hai consumer `payout` và `fraud-scoring` đọc cùng topic, mỗi bên xử lý đủ 5 sự kiện. Để thấy inbox làm gì, lab chạy trước một consumer **không có inbox** trên cùng 8 thông điệp:

```text
Consumer không inbox: nhận 8 thông điệp, ghi 8 dòng tiền; 3 chuyến bị trả hai lần
Consumer có inbox: processed=5, duplicate=3; dòng tiền = 5, dòng inbox = 5
Consumer thứ hai 'fraud-scoring': processed=5, duplicate=3; tổng dòng inbox = 10, yêu cầu chấm điểm = 5
```

Không có inbox, 3 chuyến bị trả hai lần. Có inbox, 8 lần nhận cho 5 hiệu ứng: 3 lần được nhận diện là trùng và bỏ qua. Ba kịch bản lỗi còn lại cho thấy bảo đảm giữ vững khi consumer chết đúng chỗ khó:

```text
Consumer chết sau khi xử lý, trước khi commit offset: dòng tiền = 6
Khởi động lại cùng group: nhận lại thông điệp, kết quả = ['duplicate']; dòng tiền = 6
Lỗi sau khi ghi hiệu ứng: (inbox, tiền) = (0, 0); xử lý lại = processed
```

Consumer xử lý chuyến thứ 6 rồi «chết» trước commit offset. Cùng group khởi động lại, nhận đúng thông điệp đó và nhận ra trùng: số dòng tiền vẫn là 6. Ca cuối: lỗi xảy ra sau khi ghi hiệu ứng nhưng trước commit, cả inbox lẫn tiền cùng rollback (0, 0), và lần xử lý lại thành công. Ta không tuyên bố «exactly-once delivery»: thông điệp vẫn được giao trùng (topic có 8 cho 5 sự kiện). Cái ta có là **hiệu ứng trong database của consumer xảy ra đúng một lần**, nhờ at-least-once cộng khử trùng. Kafka có transaction cho luồng đọc-rồi-ghi bên trong Kafka, nhưng không bao trùm PostgreSQL của ta. Khi hiệu ứng nằm ngoài database (gọi API thanh toán, gửi email), một hàng inbox không đủ: dependency đó cần idempotency key của riêng nó, hoặc ta tiếp tục dùng outbox cho bước kế tiếp.

## Chạy và kiểm chứng

Cần PostgreSQL và Kafka của hạ tầng dùng chung đã chạy. Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/39-outbox-inbox-idempotent-consumer examples/toi-noi-python/39-outbox-inbox-idempotent-consumer/main.py --verify
```

Lab dùng schema `lab39` và hai topic `lab39.trip.completed.v1`, `lab39.naive.trip.completed.v1` (3 partition), tất cả bị xóa khi chạy xong. Output đầy đủ lần chạy cuối:

```text
Commit rồi publish, chết giữa chừng: chuyến TN-0101 = completed, sự kiện trên topic = 0
Publish rồi commit, database lỗi: chuyến TN-0102 = in_transit, sự kiện trên topic = 1
Lỗi trước commit: chuyến TN-0001 = in_transit, dòng outbox = 0 (cả hai cùng rollback)
Hoàn thành 5 chuyến: 5 chuyến completed, 5 dòng outbox chờ phát
Dispatcher chết sau 3 lần publish: topic có 3 thông điệp, outbox đánh dấu đã phát = 0
Chạy lại sau 5 giây (lease 30 giây chưa hết): thuê được 0 dòng
Chạy lại sau 60 giây: thuê 5, phát 5; topic có 8 thông điệp, 5 event_id khác nhau; đánh dấu đã phát = 5
Consumer không inbox: nhận 8 thông điệp, ghi 8 dòng tiền; 3 chuyến bị trả hai lần
Consumer có inbox: processed=5, duplicate=3; dòng tiền = 5, dòng inbox = 5
Consumer thứ hai 'fraud-scoring': processed=5, duplicate=3; tổng dòng inbox = 10, yêu cầu chấm điểm = 5
Consumer chết sau khi xử lý, trước khi commit offset: dòng tiền = 6
Khởi động lại cùng group: nhận lại thông điệp, kết quả = ['duplicate']; dòng tiền = 6
Lỗi sau khi ghi hiệu ứng: (inbox, tiền) = (0, 0); xử lý lại = processed
Hai dispatcher thuê đồng thời (limit 3): 3 và 2 dòng, không trùng nhau: True
Đã dọn: DROP SCHEMA lab39 CASCADE, xóa topic lab39.trip.completed.v1 và lab39.naive.trip.completed.v1
OK 21/21
```

`--verify` chạy 21 kiểm tra, in `OK 21/21` khi đạt; sai thì in tên ca, `FAIL n/m` (n là số ca đạt) và exit 1; thiếu PostgreSQL hoặc Kafka thì in `FAIL: không nối được ...` và exit 1. Ta chạy lab ba lần liên tiếp, cả ba lần `OK 21/21`; các ca không dùng `sleep` để chờ cửa sổ lỗi mà dựng chúng bằng điểm chèn và thời gian truyền vào. Để biết `--verify` có bắt được lỗi thật, ta thử phá ba chỗ: tắt inbox theo mặc định (`FAIL 15/21`), nới điều kiện lease (`FAIL 19/21`), và bỏ nhánh `DUPLICATE` (`FAIL 15/21`). Phiên bản: Python 3.14.7, SQLAlchemy 2.1.8, psycopg 3.3.8, aiokafka 0.15.0, PostgreSQL 17, Kafka 3.9 (một broker).

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| Chuyến `completed` mà consumer không bao giờ nhận | Có dòng outbox cho chuyến không; `published_at` còn NULL và `lease_until` đã qua chưa; dispatcher có đang chạy không |
| Outbox ngày càng nhiều dòng `published_at IS NULL` | `send_and_wait` có lỗi không; `attempts` tăng mãi; lease quá dài sau một lần crash |
| Tiền chuyến bị trả hai lần | Inbox có cùng transaction với hiệu ứng không; `event_id` có ổn định giữa các lần phát không |
| Có dòng inbox nhưng không có hiệu ứng | Có commit inbox trước khi ghi hiệu ứng không; hiệu ứng nằm ngoài database |
| Dispatcher thứ hai không phát gì | Các dòng đang được thuê hoặc đang bị khóa; so `lease_until` với giờ hiện tại |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>Lab chạy một PostgreSQL, một broker Kafka, một tiến trình dispatcher gọi tay từng vòng; chưa có vòng poll liên tục, chưa có <code>LISTEN/NOTIFY</code> để giảm độ trễ, và <code>attempts</code> chỉ được đếm chứ chưa dùng để backoff khi Kafka lỗi. Nhiều dispatcher thuê các lô khác nhau có thể phát lệch thứ tự giữa các lô, nên thứ tự theo chuyến chỉ giữ được khi mỗi chuyến chỉ có một sự kiện hoặc khi ta phân vùng việc thuê theo key; lab chỉ phát một sự kiện cho mỗi chuyến. Outbox và inbox đều lớn dần: cần xóa dòng đã phát (outbox) và dòng cũ (inbox), và thời hạn giữ inbox phải dài hơn khoảng thời gian một thông điệp còn có thể được giao lại. Lab dùng giờ của ứng dụng cho <code>lease_until</code>; production cần chọn một nguồn giờ nhất quán giữa các dispatcher. Consumer chưa xử lý thông điệp lỗi định dạng (poison message), nên một thông điệp hỏng sẽ chặn partition cho tới khi có chỗ cất nó (ví dụ một topic dead-letter riêng). Bài 40 chỉ làm dead-letter cho job của worker, không cho thông điệp Kafka. Lab cũng đặt cả phía phát lẫn phía nhận trong một database để dễ kiểm chứng, và chưa kiểm hợp đồng <code>version</code> của envelope với consumer C#. Tiền chuyến ở đây là dữ liệu giả, không có thanh toán thật.</p>
</div>

**Bài tập:** (1) Giả sử dispatcher sinh `event_id` mới ở mỗi lần phát lại thay vì dùng id lưu trong outbox. Inbox còn chặn được thông điệp trùng không? (2) Thêm ràng buộc `UNIQUE (trip_id)` vào `payout_lines` có thay được inbox không?

<details><summary>Xem lời giải</summary><p>(1) <b>Không.</b> Inbox nhận diện bằng <code>(consumer, event_id)</code>; mỗi bản sao mang một id mới trông như một sự kiện mới, nên cả ba bản sao đều được xử lý. Vì vậy <code>event_id</code> phải sinh một lần, lưu cùng outbox, và dispatcher chỉ phát lại đúng giá trị đã lưu. (2) <b>Không thay hẳn, nhưng nên có cả hai.</b> Ràng buộc đó chặn trả tiền hai lần cho một chuyến ở mức hiệu ứng và là lớp phòng thủ thứ hai tốt, nhưng nó chỉ hợp khi hiệu ứng có khóa tự nhiên. Hiệu ứng như «cộng tiền vào số dư» hoặc «tăng bộ đếm» không có khóa tự nhiên để đặt UNIQUE; inbox bảo vệ chúng vì nó khử trùng theo sự kiện, không theo dạng hiệu ứng. Ngoài ra, vi phạm UNIQUE phát sinh exception và rollback giữa chừng, trong khi inbox cho phép bỏ qua êm.</p></details>

Bài trước: [bài 38](/articles/python-kafka-aiokafka-producer-consumer-schema). Bài tiếp: [bài 40](/articles/python-worker-arq-scheduling-retry-dead-letter).

## Tài liệu gốc

- [Pattern: Transactional outbox (microservices.io)](https://microservices.io/patterns/data/transactional-outbox.html).
- [Pattern: Idempotent Consumer (microservices.io)](https://microservices.io/patterns/communication-style/idempotent-consumer.html).
- [PostgreSQL 17: SELECT, mệnh đề locking (FOR UPDATE, SKIP LOCKED)](https://www.postgresql.org/docs/17/sql-select.html).
- [PostgreSQL 17: INSERT ... ON CONFLICT](https://www.postgresql.org/docs/17/sql-insert.html).
- [SQLAlchemy 2.1: Dialect PostgreSQL, INSERT ... ON CONFLICT](https://docs.sqlalchemy.org/en/21/dialects/postgresql.html).
- [aiokafka: tài liệu chính thức](https://aiokafka.readthedocs.io/en/stable/).
- [Apache Kafka 3.9: tài liệu, mục Message Delivery Semantics](https://kafka.apache.org/39/documentation/).
