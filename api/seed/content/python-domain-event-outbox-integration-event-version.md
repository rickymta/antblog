---
slug: python-domain-event-outbox-integration-event-version
title: "Domain event, outbox và version hợp đồng sự kiện"
summary: "Bài 71: tách domain event khỏi integration event, ghi outbox trong Unit of Work, dispatcher Kafka, nâng version v1 lên v2 tương thích ngược và quy ước tên trường snake_case cho consumer C#. Chạy thật trên PostgreSQL và Kafka."
category: python-advanced
level: advanced
tags: [python, toi-noi, outbox, kafka, domain-event]
language: vi
---

Khi hồ sơ tài xế của [Tới Nơi](/articles/du-an-mau-toi-noi) vào hàng chờ duyệt, một dịch vụ phía C# muốn thống kê hồ sơ đã chấm điểm theo lý do, và một service phân tích muốn đếm chúng theo ngày. (Chưa đủ để dựng bảng hàng chờ: sự kiện chỉ có cho hồ sơ đã chấm điểm đủ và không có sự kiện khi người duyệt quyết định, nên bảng dựng từ đây thiếu hồ sơ chưa có điểm và không bao giờ xoá dòng.) Bài 70 đã để lại domain event `KycApplicationScored`, nhưng nó chỉ đi vào bộ nhớ và có thể thành «sự kiện ma» khi COMMIT lỗi. Muốn sự kiện rời service đáng tin cậy, ta cần outbox của bài 39 đặt đúng chỗ trong Unit of Work của bài 69.

Việc khó hơn kỹ thuật: sự kiện là **hợp đồng** với đội viết C#, đọc JSON bằng thư viện của họ chứ không đọc mã Python của ta. Tên trường, cách nâng version và thứ tự nâng để consumer cũ không gãy đều phải được chốt và kiểm chứng. Bài làm cả hai: outbox trong transaction, và hợp đồng có version, kiểm trên PostgreSQL, Kafka và một consumer C# chạy thật.

Đây là bài 71 của lộ trình **Python nâng cao qua Tới Nơi**. Bạn cần đã đọc [bài 70](/articles/python-use-case-ho-so-tai-xe-end-to-end), outbox và inbox ở [bài 39](/articles/python-outbox-inbox-idempotent-consumer) và envelope có version ở [bài 38](/articles/python-kafka-aiokafka-producer-consumer-schema). Phía C# có [Outbox và Inbox bằng .NET 10](/articles/dotnet10-outbox-inbox-idempotent-consumer) và [Aggregate, domain event](/articles/dotnet10-ddd-bounded-context-aggregate); bài outbox C# ghi payload bằng `JsonSerializer` mặc định (tên PascalCase) vì phía đọc cũng là C#, còn ở đây phía đọc khác ngôn ngữ nên quy ước tên phải thành hợp đồng.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Ta thêm hai trường tuỳ chọn <code>face_score</code> và <code>ocr_min_confidence</code> vào payload và tăng <code>version</code> từ 1 lên 2. Consumer Python chưa nâng cấp kiểm <code>version in {1}</code> trước khi đọc payload (như bài 38). Nên triển khai producer trước hay consumer trước?</p>
<details><summary>Xem đáp án</summary><p><b>Consumer trước.</b> Thêm trường tuỳ chọn là tương thích về <i>cấu trúc</i>, nhưng consumer kiểm version trước khi đọc payload nên một consumer chỉ hiểu <code>{1}</code> từ chối mọi thông điệp version 2 (<code>unsupported_version</code>). Producer chỉ được phát v2 sau khi mọi consumer đã hiểu <code>{1, 2}</code>. Bài chạy cả hai consumer trên cùng một topic để xem kết quả.</p></details>
</div>

## Domain event và integration event là hai thứ khác nhau

**Domain event** thuộc về miền của service: `KycApplicationScored` là một dataclass Python, mang `ReviewReason` (enum), `datetime`, tên trường theo ý aggregate. Nó đổi tự do khi ta refactor miền. **Integration event** là thứ đi ra ngoài: một envelope JSON với tên trường, kiểu và `version` mà bên khác đã dựa vào. Hai thứ trông giống nhau hôm nay nhưng đổi vì lý do khác nhau, nên lab tách chúng và đặt một hàm duy nhất giữa hai bên:

```python
# src/kyc_service/application/integration_events.py (trích)
def scored_envelope(
    event: KycApplicationScored, *, event_id: str, version: int = 2
) -> dict[str, Any]:
    """Domain event -> envelope. `version` chọn hình dạng payload (producer đang ở hợp đồng nào)."""
    if version not in SUPPORTED_VERSIONS:
        raise ValueError(f"version {version} chưa được định nghĩa")
    payload: dict[str, Any] = {
        "application_id": event.application_id,
        "driver_id": event.driver_id,
        "risk_score": event.risk_score,
        "review_reasons": [r.value for r in event.review_reasons],
    }
    if version >= 2:
        payload["face_score"] = event.face_score
        payload["ocr_min_confidence"] = event.ocr_min_confidence
    return {
        "event_id": event_id,
        "event_type": EVENT_TYPE,
        "occurred_at": event.occurred_at.isoformat(),
        "version": version,
        "payload": payload,
    }
```

Hàm này là chỗ **duy nhất** biết cả domain event lẫn hợp đồng ngoài. Đổi tên `risk_score` trong domain chỉ làm hàm này sửa một dòng, còn JSON trên dây không đổi; ngược lại, thêm trường ra ngoài là thay đổi có version, không phải chuyện refactor. Phần demo in cả hai dạng của cùng một sự kiện:

```text
== 1. Domain event và integration event: hai thứ khác nhau
  domain event KycApplicationScored (kiểu Python, nội bộ service):
    occurred_at=2026-11-06 03:00 UTC, application_id=KYC-0001
    risk_score=0.35, face_score=0.71
    review_reasons=('OCR_LOW_CONFIDENCE', 'FACE_LOW_SCORE', 'RISK_HIGH')  (enum ReviewReason)
  integration event v2 (JSON trên dây, hợp đồng với service khác):
    {
      "event_id": "00000000-0000-4000-8000-000000000001",
      "event_type": "KycApplicationScored",
      "occurred_at": "2026-11-06T03:00:00+00:00",
      "version": 2,
      "payload": {
        "application_id": "KYC-0001",
        "driver_id": "DR-002",
        "risk_score": 0.35,
        "review_reasons": [
          "ocr_low_confidence",
          "face_low_score",
          "risk_high"
        ],
        "face_score": 0.71,
        "ocr_min_confidence": 0.58
      }
    }
```

Envelope giữ đúng dạng của từ điển miền (bài 38 và 39): `event_id`, `event_type`, `occurred_at` ISO 8601 UTC, `version`, `payload`. Payload không có số giấy tờ hay họ tên: ranh giới giữa hai service không phải nơi để PII đi qua. Lý do vào hàng chờ đi dưới dạng **chuỗi** (`"risk_high"`), không phải enum của Python hay C#, để một lý do mới không làm consumer đang dùng enum đóng gãy.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 285" role="img" aria-label="Đường đi của sự kiện. Trong một transaction PostgreSQL: aggregate phát domain event, hàm scored_envelope ánh xạ sang integration event, outbox lưu dòng. Ngoài transaction: dispatcher đọc outbox và phát lên Kafka, từ đó consumer Python và consumer C# cùng đọc."><defs><marker id="py71-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="10" y="14" width="860" height="108" rx="10" class="f-none s-phys dash"/><rect x="20" y="30" width="190" height="64" rx="8" class="f-pk-soft s-pk"/><text x="34" y="56" class="t-b">Aggregate</text><text x="34" y="78" class="t-small">send_to_review</text><rect x="250" y="30" width="190" height="64" rx="8" class="f-pk-soft s-pk"/><text x="264" y="56" class="t-b">Domain event</text><text x="264" y="78" class="t-small">KycApplicationScored</text><rect x="480" y="30" width="190" height="64" rx="8" class="f-warn-soft s-warn"/><text x="494" y="56" class="t-b">scored_envelope</text><text x="494" y="78" class="t-small">sang integration event</text><rect x="710" y="30" width="150" height="64" rx="8" class="f-phys-soft s-phys"/><text x="724" y="56" class="t-b">Outbox</text><text x="724" y="78" class="t-small">một dòng mới</text><text x="20" y="112" class="t-small">cùng một transaction PostgreSQL (Unit of Work)</text><path d="M210 62 L248 62" class="f-none s-phys" stroke-width="2" marker-end="url(#py71-0-arrow)"/><path d="M440 62 L478 62" class="f-none s-phys" stroke-width="2" marker-end="url(#py71-0-arrow)"/><path d="M670 62 L708 62" class="f-none s-phys" stroke-width="2" marker-end="url(#py71-0-arrow)"/><rect x="710" y="165" width="150" height="70" rx="8" class="f-phys-soft s-phys"/><text x="724" y="192" class="t-b">Dispatcher</text><text x="724" y="214" class="t-small">ngoài transaction</text><rect x="480" y="165" width="190" height="70" rx="8" class="f-phys-soft s-phys"/><text x="494" y="192" class="t-b">Kafka</text><text x="494" y="214" class="t-small">topic .v1, key = mã hồ sơ</text><rect x="250" y="165" width="190" height="70" rx="8" class="f-pk-soft s-pk"/><text x="264" y="192" class="t-b">Consumer Python</text><text x="264" y="214" class="t-small">hiểu {1} hoặc {1, 2}</text><rect x="20" y="165" width="190" height="70" rx="8" class="f-pk-soft s-pk"/><text x="34" y="192" class="t-b">Consumer C#</text><text x="34" y="214" class="t-small">SnakeCaseLower, {1, 2}</text><path d="M785 124 L785 163" class="f-none s-phys" stroke-width="2" marker-end="url(#py71-0-arrow)"/><path d="M708 200 L672 200" class="f-none s-phys" stroke-width="2" marker-end="url(#py71-0-arrow)"/><path d="M478 200 L442 200" class="f-none s-phys" stroke-width="2" marker-end="url(#py71-0-arrow)"/><path d="M575 237 L575 262 L115 262 L115 238" class="f-none s-phys" stroke-width="2" marker-end="url(#py71-0-arrow)"/></svg>
</div>
<figcaption>Hình 1. Bên trong khung nét đứt là một transaction; từ outbox trở đi mọi bước đều nằm ngoài transaction và có thể lặp lại.</figcaption>
</figure>

## Outbox trong Unit of Work

Adapter `OutboxEventPublisher` hiện thực port `EventPublisher` của bài 70, nhưng thay vì cất event trong bộ nhớ, nó ghi một dòng `outbox` bằng **Session của Unit of Work đang mở**:

```python
# src/kyc_service/infrastructure/sql/outbox.py (trích)
    def publish(self, events: Sequence[DomainEvent]) -> None:
        session = self._uow.session  # ném UnitOfWorkNotStarted nếu gọi ngoài transaction
        for event in events:
            event_id = self._ids.new_id()  # đặt id một lần, lưu cùng dòng outbox
            session.add(
                OutboxRow(
                    event_id=event_id,
                    topic=self._topic,
                    message_key=partition_key(event),
                    envelope=to_envelope(event, event_id=event_id, version=self._emit_version),
                    created_at=self._clock.now(),
                )
            )
        session.flush()
```

Vì dùng chung Session, dòng outbox commit hoặc rollback cùng hồ sơ. Điều đáng chú ý là `handlers.py` của bài 71 **giống hệt** bài 70 từng dòng: handler vẫn gọi `self._events.publish(application.pull_events())`, và chỉ composition root đổi adapter. Đó là phần thưởng của việc để port ở lớp application. `event_id` được sinh một lần, lúc ghi, qua port `IdGenerator`, và lưu cùng dòng; không nơi nào sinh lại nó khi phát lại. Bảng `outbox` giống bài 39: `seq` cho thứ tự, `event_id` duy nhất, `published_at` rỗng nghĩa là chưa phát, `lease_until` và `attempts` cho dispatcher.

```python
# src/kyc_service/infrastructure/sql/tables.py (trích)
class OutboxRow(Base):
    """Ý định phát một sự kiện, ghi cùng transaction với thay đổi nghiệp vụ (xem bài 39).

    `seq` cho thứ tự phát; `event_id` là danh tính ổn định của sự kiện (consumer khử trùng bằng
    nó); `published_at` NULL nghĩa là chưa phát; `lease_until` là «thuê» của một dispatcher.
    """

    __tablename__ = "outbox"
    __table_args__ = (
        Index("ix_outbox_pending", "seq", postgresql_where=text("published_at IS NULL")),
    )

    seq: Mapped[int] = mapped_column(BigInteger, Identity(), primary_key=True)
    event_id: Mapped[str] = mapped_column(String(36), unique=True)
    topic: Mapped[str] = mapped_column(String(128))
    message_key: Mapped[str] = mapped_column(String(64))
    envelope: Mapped[dict[str, Any]] = mapped_column(JSONB)
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True))
    published_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True))
    lease_until: Mapped[datetime | None] = mapped_column(DateTime(timezone=True))
    attempts: Mapped[int] = mapped_column(default=0)
```

Hai kết quả chạy: sau khi nộp có một hồ sơ và một dòng outbox; còn khi COMMIT thất bại (cùng hook làm hỏng COMMIT như bài 70), không còn hồ sơ **và** không còn dòng outbox, nên không còn sự kiện ma.

```text
== 2. Outbox: hồ sơ và sự kiện cùng một transaction
  sau khi nộp: hồ sơ=1, outbox=1, published_at=None
  COMMIT thất bại giữa chừng (OperationalError)
  sau COMMIT lỗi: hồ sơ=0, outbox=0 (bài 70 để lại 1 sự kiện ma)
```

Hồ sơ chưa chấm điểm đủ (không đồng ý khuôn mặt, hoặc một service tắt) không có domain event nên không có dòng outbox; `ResumeScreening` chỉ ghi dòng khi chấm điểm xong (cả ba ca có test).

## Dispatcher: phát ngoài transaction và vì sao có thông điệp trùng

Dispatcher (cơ chế đã nói ở bài 39) làm ba bước mỗi vòng: thuê dòng chưa phát bằng một câu `UPDATE` dùng `FOR UPDATE SKIP LOCKED` và commit ngay, phát từng dòng ra Kafka, rồi đánh dấu `published_at`. Không giữ khoá hay transaction nào trong lúc gọi Kafka.

```python
# src/kyc_service/infrastructure/kafka/dispatcher.py (trích)
    async def dispatch_once(self, *, crash_after_publish: int | None = None) -> DispatchReport:
        now = self._clock.now()
        with self._factory() as session, session.begin():
            claimed = claim_batch(session, now=now, lease=self._lease, limit=self._batch)
        # Transaction thuê đã commit: từ đây không giữ khóa hay transaction nào khi gọi Kafka.
        published: list[int] = []
        for item in claimed:
            value = json.dumps(item.envelope, ensure_ascii=False, separators=(",", ":")).encode()
            await self._producer.send_and_wait(item.topic, value, key=item.key.encode())
            published.append(item.seq)
            if crash_after_publish is not None and len(published) >= crash_after_publish:
                raise SimulatedCrash(
                    f"chết sau khi phát {len(published)} thông điệp, trước khi đánh dấu"
                )
        if published:
            with self._factory() as session, session.begin():
                session.execute(
                    update(OutboxRow).where(OutboxRow.seq.in_(published)).values(published_at=now)
                )
        return DispatchReport(claimed=len(claimed), published=len(published))
```

Khoá Kafka là mã hồ sơ, nên các thông điệp của một hồ sơ vào cùng partition. Kafka chỉ giữ thứ tự **gửi** trong một partition, không giữ thứ tự tạo: hai dispatcher cùng chạy có thể gửi ngược, và dispatcher chết trước khi đánh dấu có thể gửi lại (kiểu 1, 2, 1, 2). Muốn có thứ tự thì chỉ một dispatcher cho mỗi khoá, hoặc consumer tự kiểm bằng `version` và `occurred_at`; lab có một sự kiện mỗi hồ sơ nên chưa cần. JSONB không giữ thứ tự khoá, nên thứ tự trường trên dây không thuộc hợp đồng. Chạy hai vòng liên tiếp cho ba hồ sơ, rồi mô phỏng dispatcher chết sau khi phát một thông điệp và trước khi đánh dấu:

```text
== 3. Dispatcher đẩy outbox ra Kafka, ngoài transaction
  vòng 1: thuê 3, phát 3
  vòng 2: thuê 0, phát 0
  trên topic: 3 thông điệp, key ['KYC-0001', 'KYC-0002', 'KYC-0003']
  dòng outbox đã đánh dấu: 3/3
```

```text
== 4. Dispatcher chết giữa phát và đánh dấu: cùng event_id đến hai lần
  chết sau khi phát 1 thông điệp, trước khi đánh dấu
  ngay sau đó (lease còn hiệu lực): thuê/phát = (0, 0)
  đồng hồ mô phỏng nhích 31 giây (lease hết hạn): thuê/phát = (2, 2)
  trên topic: 3 thông điệp, 2 event_id khác nhau
```

Vòng hai không có gì để phát: dòng đã được đánh dấu. Trong ca chết giữa chừng, `lease_until` chặn mọi dispatcher khác trong 30 giây; khi đồng hồ mô phỏng (`ManualClock`, không chờ thật) nhích 31 giây, lease hết hạn và dòng chưa đánh dấu được phát lại: **3 thông điệp trên topic nhưng chỉ 2 `event_id`**. Đó là at-least-once, cái giá của việc không có transaction chung giữa PostgreSQL và Kafka. `enable_idempotence` của producer không giúp: nó chỉ khử trùng lặp khi chính producer thử lại một gói tin, còn hai vòng dispatcher là hai lần `send` khác nhau trong mắt nó. Khử trùng lặp thuộc về **consumer** bằng `event_id` (inbox, bài 39).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 230" role="img" aria-label="Ba giai đoạn nâng version. Giai đoạn một: producer vẫn phát v1, consumer mới đọc 1 và 2, consumer cũ vẫn đọc v1. Giai đoạn hai: producer phát v2, consumer cũ từ chối v2. Giai đoạn ba cho thay đổi không tương thích: topic mới .v2, phát song song, consumer chuyển dần."><defs><marker id="py71-1-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="30" width="260" height="170" rx="10" class="f-pk-soft s-pk"/><text x="36" y="58" class="t-b">1. Nâng consumer trước</text><text x="36" y="92" class="t-small">producer vẫn phát v1</text><text x="36" y="116" class="t-small">consumer mới đọc {1, 2}</text><text x="36" y="140" class="t-small">consumer cũ vẫn đọc v1</text><text x="36" y="180" class="t-small">an toàn: chưa có v2 trên topic</text><rect x="310" y="30" width="260" height="170" rx="10" class="f-pk-soft s-pk"/><text x="326" y="58" class="t-b">2. Rồi mới nâng producer</text><text x="326" y="92" class="t-small">producer phát v2</text><text x="326" y="116" class="t-small">consumer mới đọc v1 và v2</text><text x="326" y="140" class="t-small">consumer cũ từ chối v2</text><text x="326" y="180" class="t-small">chỉ làm khi hết consumer cũ</text><rect x="600" y="30" width="260" height="170" rx="10" class="f-warn-soft s-warn"/><text x="616" y="58" class="t-b">3. Đổi vỡ: topic mới</text><text x="616" y="92" class="t-small">topic mới: ...scored.v2</text><text x="616" y="116" class="t-small">producer phát song song hai topic</text><text x="616" y="140" class="t-small">consumer chuyển dần, rồi tắt .v1</text><text x="616" y="180" class="t-small">đổi tên, xoá trường, đổi nghĩa</text><path d="M282 115 L308 115" class="f-none s-phys" stroke-width="2" marker-end="url(#py71-1-arrow)"/><path d="M572 115 L598 115" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py71-1-arrow)"/></svg>
</div>
<figcaption>Hình 2. Thêm trường tuỳ chọn đi qua hai bước theo thứ tự; thay đổi không tương thích phải mở topic mới và chạy song song.</figcaption>
</figure>

## Version hợp đồng: v1 sang v2

Hợp đồng có hai phiên bản. Version 1 gồm `application_id`, `driver_id`, `risk_score`, `review_reasons`. Version 2 thêm hai trường **tuỳ chọn** `face_score` và `ocr_min_confidence`. Quy tắc của lab, giống bài 38, gồm ba ý: `version` là số nguyên chỉ tăng; thêm trường tuỳ chọn là thay đổi tương thích (consumer bỏ qua trường lạ, trường vắng là `None`); đổi tên, xoá trường hoặc đổi nghĩa là thay đổi vỡ và phải mở topic mới với hậu tố `.v2`. Hậu tố của topic là «thế hệ tương thích», còn `version` trong envelope là bản sửa trong thế hệ đó; vì vậy ở hình 2, bước 3 phát song song hai topic. Quy tắc này do lab chọn: hệ có Schema Registry kiểm tính tương thích bằng máy, còn ở đây hợp đồng được giữ bằng golden file và test. Phiên bản producer phát (`emit_version`) là cấu hình triển khai **bắt buộc**, không có mặc định: quên đặt thì lỗi chứ không âm thầm phát bản cũ hay mới. Hai golden file `scored.v1.json` và `scored.v2.json` viết tay và được so với đầu ra của `scored_envelope`; test kiểm v2 chỉ **thêm** hai trường:

```python
# tests/test_contract.py (trích)
def test_v2_only_adds_optional_fields_to_v1() -> None:
    v1 = json.loads(golden(1))
    v2 = json.loads(golden(2))
    assert {k: v for k, v in v2["payload"].items() if k in v1["payload"]} == v1["payload"]
    assert set(v2["payload"]) - set(v1["payload"]) == {"face_score", "ocr_min_confidence"}
    assert {k: v for k, v in v2.items() if k not in ("version", "payload")} == {
        k: v for k, v in v1.items() if k not in ("version", "payload")
    }
```

Consumer đọc theo quy tắc «kiểm version trước, mới đọc payload», vì payload của version lạ có thể đã đổi nghĩa:

```python
# consumers/scored_consumer.py (trích)
    version = env.get("version")
    # Kiểm kiểu trước: True == 1 lọt qua `in`, list/dict không băm được.
    if type(version) is not int or version not in supported:
        # Kiểm TRƯỚC khi đọc payload: payload của version lạ có thể đã đổi nghĩa.
        raise EnvelopeError("unsupported_version", repr(version))
```

Hai consumer cùng đọc một topic gồm thông điệp v1, v2 và một thông điệp version 3 giả:

```text
== 5. Nâng version v1 -> v2: consumer cũ và mới đọc cùng một topic
  thông điệp  consumer cũ {1}                   consumer mới {1, 2}
  version 1   đọc được v1                       đọc được v1
  version 2   từ chối (unsupported_version)     đọc được v2
  version 3   từ chối (unsupported_version)     từ chối (unsupported_version)
```

Consumer cũ chỉ hiểu `{1}` nên từ chối cả v2 lẫn v3; consumer mới hiểu `{1, 2}` đọc được v1 và v2 (trường vắng của v1 là `None`) và vẫn từ chối v3. Từ đó có hai hệ quả cho việc triển khai. Một: nâng consumer **trước**, nâng producer sau, và chỉ nâng producer khi không còn consumer cũ. Hai: consumer cũ không «hỏng», nó từ chối có lý do ổn định (`unsupported_version`) và phải commit offset qua các thông điệp đó thay vì kẹt (bài 38).

## Quy ước tên trường: snake_case, và consumer C# phải nói rõ

Hợp đồng cần một quyết định mà không bên nào tự nhiên đúng: tên trường là `snake_case` hay `camelCase`? Lab chọn **snake_case**, ở cả envelope lẫn payload. Lý do thứ nhất: envelope trong từ điển miền và các sự kiện từ bài 38, 39 (`event_id`, `occurred_at`) đã là snake_case; đổi tên trường trên dây là thay đổi vỡ với mọi consumer hiện có. Lý do thứ hai: chi phí chuyển sang đầu C#, nơi nó rẻ và được kiểm: một dòng cấu hình `JsonNamingPolicy.SnakeCaseLower`. Loạt Rust của Tới Nơi chọn camelCase cho payload sổ cái giữa Rust và C#; đó là hợp đồng khác, và không có quy ước đúng chung. Điều bắt buộc là mỗi hợp đồng ghi rõ quy ước và có test từ phía consumer.

Consumer C# của lab là một chương trình nhỏ, không có gói NuGet ngoài, đọc các thông điệp mà lab Python ghi ra và in kết quả. Cấu hình và kiểu đọc (hai dòng `Respect...` chống nhận thông điệp thiếu trường mà không báo, xem bên dưới):

```csharp
// csharp-consumer/Program.cs (trích)
var wire = new JsonSerializerOptions
{
    PropertyNamingPolicy = JsonNamingPolicy.SnakeCaseLower,
    UnmappedMemberHandling = JsonUnmappedMemberHandling.Skip, // trường lạ trong cùng version: bỏ qua
    // Hai dòng dưới chống «nhận thông điệp thiếu trường mà không báo»: tham số constructor không có
    // giá trị mặc định là BẮT BUỘC, và null vào kiểu không-nullable là lỗi.
    RespectRequiredConstructorParameters = true,
    RespectNullableAnnotations = true,
};

```

```csharp
// csharp-consumer/Program.cs (trích)
record Envelope(Guid EventId, string EventType, DateTimeOffset OccurredAt, int Version, ScoredPayload Payload);

record ScoredPayload(
    string ApplicationId,
    string DriverId,
    double RiskScore,
    string[] ReviewReasons,
    double? FaceScore = null, // có từ version 2: vắng ở v1 là null
    double? OcrMinConfidence = null);
```

`ScoredPayload` dùng `string[]` cho lý do. Hai trường của v2 mặc định `null`, nên vắng ở v1 là hợp lệ; mọi tham số còn lại **bắt buộc** nhờ `RespectRequiredConstructorParameters`, và `RespectNullableAnnotations` biến `null` vào kiểu không-nullable thành lỗi. Chương trình còn kiểm `event_type` và bắt `JsonException` thành `invalid_payload`. Không có các chốt đó, bản đầu nhận thông điệp thiếu `risk_score` với giá trị 0, nhận cả `event_type` khác, và nổ `NullReferenceException` khi thiếu `payload`. Các thông điệp đã đi qua Kafka được đưa cho chương trình; sau đó nó thử đọc lại **không** đặt naming policy:

```text
== 6. Consumer C# đọc cùng các thông điệp bằng System.Text.Json
  đọc được v1: KYC-0001 risk_score=0.09 review_reasons=['routine'] face_score=None
  đọc được v2: KYC-0002 risk_score=0.35 review_reasons=['ocr_low_confidence', 'face_low_score', 'risk_high'] face_score=0.71
  từ chối v3: unsupported_version
  không đặt naming policy: event_id = 00000000-0000-0000-0000-000000000000, payload null = True
```

Dòng cuối là điều đáng sợ nhất: không đặt `SnakeCaseLower`, `JsonSerializer` **không báo lỗi**. Nó khớp tên theo nguyên văn, `event_id` không khớp `EventId`, nên mọi thành viên vắng trong JSON nhận giá trị mặc định (`Guid` rỗng, `Payload` null) và consumer âm thầm xử lý một sự kiện trống. Test C# giữ cả ca này. Năm thông điệp hỏng (thiếu `risk_score`, sai `event_type`, thiếu `payload`, `payload` null, `application_id` null) bị C# và consumer Python từ chối cùng lý do:

```text
== 7. Thông điệp hỏng: C# và Python từ chối cùng lý do
  thông điệp            C#                  Python
  thiếu risk_score      invalid_payload     invalid_payload
  sai event_type        wrong_event_type    wrong_event_type
  thiếu payload         invalid_payload     invalid_payload
  payload null          invalid_payload     invalid_payload
  application_id null   invalid_payload     invalid_payload
```

Verify còn so từng trường C# đọc được với consumer Python trên cùng thông điệp. Điểm đi dưới dạng số thực JSON vì không phải tiền; số tiền đi dưới dạng chuỗi như bài 39.

## Chạy và kiểm chứng

Cần uv, Python 3.14, hạ tầng chung (PostgreSQL và Kafka, `docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait`) và .NET SDK 10 cho phần consumer C#. Từ gốc repository:

```bash
uv run --project examples/toi-noi-python/71-domain-event-outbox-version examples/toi-noi-python/71-domain-event-outbox-version/main.py
```

(trích: sáu đoạn demo đã in ở các mục trên, theo đúng thứ tự `1` đến `6`; lần đầu `dotnet run` build project C# nên chậm hơn.) Schema `lab71` và các topic `lab71.*` bị xoá khi chạy xong. Kiểm các kết quả trên:

```bash
uv run --project examples/toi-noi-python/71-domain-event-outbox-version examples/toi-noi-python/71-domain-event-outbox-version/main.py --verify
```

```text
OK 21/21
```

21 ca: outbox, hợp đồng (golden file, v2 chỉ thêm trường, consumer từ chối thông điệp sai), dispatcher và Kafka (phát, chết giữa chừng, hai dispatcher, thuê dòng, thứ tự nâng version), một ca HTTP và ba ca C# (đọc đúng, từ chối thông điệp hỏng, mặc định âm thầm khi thiếu naming policy). Chuỗi đầy đủ gồm ruff, mypy `--strict`, import-linter với năm hợp đồng (mới so với bài 69: lõi không import `aiokafka`), pytest và `--verify`:

```bash
sh examples/toi-noi-python/71-domain-event-outbox-version/verify.sh
```

```text
== 1. ruff format --check
58 files already formatted
== 2. ruff check
All checks passed!
== 3. mypy --strict
Success: no issues found in 56 source files
== 4. import-linter
---------
Contracts
---------
Analyzed 59 files, 147 dependencies.
------------------------------------
Bốn lớp: api > infrastructure > application > domain KEPT
Lõi không import framework web KEPT
Domain không import pydantic KEPT
Lõi không import thư viện cơ sở dữ liệu KEPT
Lõi không import thư viện Kafka KEPT
Contracts: 5 kept, 0 broken.
== 5. pytest
........................................................................ [ 32%]
........................................................................ [ 65%]
........................................................................ [ 97%]
.....                                                                    [100%]
221 passed in 69.92s (0:01:09)
== 6. main.py --verify
OK 21/21
OK 6/6
```

(trích: bỏ phần hình ASCII in đầu của import-linter.) Python 3.14.7, SQLAlchemy 2.1.8, psycopg 3.3.8, aiokafka 0.15.0, PostgreSQL 17.11, Kafka 3.9.0, .NET SDK 10.0.401, import-linter 2.16, mypy 2.4.1, pytest 9.2.0, ruff 0.17.4 (theo `uv.lock`). Output do một lần chạy chung với nhiều tác vụ khác của máy (load average đo bằng `uptime` sau loạt chạy cuối: 336,90; 334,81; 362,57) nên số giây của pytest chỉ để tham khảo. Không có `dotnet` thì `--verify` dừng với lý do rõ ràng; thêm `--no-csharp` để bỏ ba ca C# (khi đó kết quả là `OK 18/18`), và `pytest -m "not csharp"` bỏ các test C#.

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| C# nhận sự kiện nhưng mọi trường là mặc định (`Guid` rỗng, `Payload` null), hoặc `invalid_payload` hàng loạt | Đã đặt `JsonNamingPolicy.SnakeCaseLower` chưa; tên trường trên dây là snake_case |
| Consumer cũ ghi `unsupported_version` sau khi producer nâng version | Thứ tự triển khai: nâng consumer trước; xem `emit_version` (cấu hình bắt buộc, không mặc định) của producer |
| Cùng `event_id` xuất hiện hai lần trên topic | Dispatcher chết giữa lúc phát và lúc đánh dấu (at-least-once); consumer phải khử trùng lặp theo `event_id` |
| Dòng outbox có `published_at` rỗng mãi | Dispatcher có chạy không; `lease_until` còn hiệu lực không; cột `attempts` có tăng không |
| Có hồ sơ nhưng không có dòng outbox | Hồ sơ chưa chấm điểm đủ (không đồng ý khuôn mặt hoặc service tắt): chưa có domain event |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>Dispatcher chạy một vòng mỗi lần gọi (không vòng lặp nền, backoff hay dead-letter) và gọi SQLAlchemy đồng bộ trong coroutine nên chặn event loop. Bảng outbox chỉ tăng: production cần dọn dòng đã phát. Không có inbox (xem bài 39) và không có Schema Registry; consumer C# chỉ được kiểm với các hình dạng thông điệp ở đây. Sự kiện chưa đủ để dựng hàng chờ (xem đoạn mở bài). Payload không có số giấy tờ hay họ tên, nhưng điểm khuôn mặt và rủi ro vẫn nhạy cảm, nên quyền đọc topic cần kiểm soát; điểm và lý do chỉ là đầu vào cho người duyệt. Mọi dữ liệu là dữ liệu tổng hợp.</p>
</div>

**Bài tập:** (1) Ta muốn thêm trường `driver_city` mà producer luôn gửi. Nó là version 3 hay vẫn là 2, consumer được phép coi nó bắt buộc từ lúc nào, và vì sao consumer vẫn phải đọc được thông điệp cũ không có nó? (2) Dispatcher chết sau khi phát và trước khi đánh dấu, consumer C# nhận cùng `event_id` hai lần. Khử trùng lặp nằm ở đâu, và vì sao `enable_idempotence` của producer không làm việc đó?

<details><summary>Xem lời giải</summary><p>(1) Thêm trường là thay đổi payload nên nâng <code>version</code> (thành 3); chỉ gọi nó «bắt buộc» với thông điệp từ version 3 trở đi: consumer đòi <code>driver_city</code> khi <code>version &gt;= 3</code>, còn v1 và v2 vắng trường vẫn hợp lệ. Consumer phải đọc được thông điệp cũ vì trên topic còn v1 và v2 chưa tiêu thụ hoặc chưa hết thời gian lưu, và consumer mới có thể phải đọc lại từ đầu. Đòi trường cho mọi version là thay đổi vỡ. (2) Ở <b>consumer</b>, bằng bảng inbox khoá (consumer, <code>event_id</code>) ghi cùng transaction với hiệu ứng (bài 39). <code>enable_idempotence</code> chỉ khử trùng lặp khi chính producer thử lại một gói tin; hai vòng dispatcher là hai lần <code>send</code> độc lập, với producer là hai thông điệp khác nhau. Cái giữ danh tính là <code>event_id</code>, sinh một lần lúc ghi outbox.</p></details>

Bài trước: [bài 70](/articles/python-use-case-ho-so-tai-xe-end-to-end). Bài tiếp: [bài 72](/articles/python-profiling-cprofile-py-spy-tracemalloc).

## Tài liệu gốc

- [aiokafka: tài liệu](https://aiokafka.readthedocs.io/en/stable/).
- [Apache Kafka 3.9: tài liệu](https://kafka.apache.org/39/documentation.html).
- [PostgreSQL 17: SELECT, mục The Locking Clause (SKIP LOCKED)](https://www.postgresql.org/docs/17/sql-select.html).
- [.NET: Customize property names and values with System.Text.Json](https://learn.microsoft.com/en-us/dotnet/standard/serialization/system-text-json/customize-properties).
- [.NET: JsonNamingPolicy.SnakeCaseLower](https://learn.microsoft.com/en-us/dotnet/api/system.text.json.jsonnamingpolicy.snakecaselower).
- [.NET: Handle unmapped members during deserialization](https://learn.microsoft.com/en-us/dotnet/standard/serialization/system-text-json/missing-members).
