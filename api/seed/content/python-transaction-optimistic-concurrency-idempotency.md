---
slug: python-transaction-optimistic-concurrency-idempotency
title: "Transaction, optimistic concurrency và idempotency: đặt chuyến an toàn"
summary: "Bài 25: Session.begin(), isolation level, cột version với UPDATE có điều kiện, khóa idempotency với UNIQUE và SQLSTATE 23505, retry có giới hạn. Tái hiện mất cập nhật và race bằng PostgreSQL thật."
category: python
level: intermediate
tags: [python, toi-noi, postgresql, transaction, idempotency]
language: vi
---

Lúc cao điểm, [Tới Nơi](/articles/du-an-mau-toi-noi) tạo khoảng 70 chuyến mỗi giây và sửa bảng `trips` khoảng 560 lần mỗi giây. Hai chuyện xảy ra thường xuyên trong dòng đó. Một: hai tài xế bấm nhận cùng một chuyến trong cùng một giây, mà chỉ một người được nhận. Hai: ứng dụng khách gửi yêu cầu đặt chuyến, mạng đứt đúng lúc phản hồi đang bay về, ứng dụng gửi lại, và nếu server không nhận ra đây là cùng một ý định thì khách có hai chuyến. Bài 24 cho ta đọc và ghi PostgreSQL bằng SQLAlchemy; bài này làm cho việc ghi **đúng** khi có nhiều người ghi cùng lúc.

Đây là bài 25 của lộ trình **Python trung bình qua Tới Nơi**. Bạn cần nắm `Session`, `select()` và mapping ở [bài 24](/articles/python-sqlalchemy-psycopg-postgresql-mapping-migration). Loạt C# có bài cùng chủ đề là [Transaction, optimistic concurrency và idempotency](/articles/dotnet10-transaction-concurrency-idempotency); bài này đi theo cách của SQLAlchemy.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Hai tài xế cùng mở chuyến <code>TN-0003</code>. Mỗi bên đọc chuyến, thấy <code>driver_id</code> còn trống, ghi tên mình rồi commit. Không có lỗi nào xảy ra ở cả hai phía. Cuối cùng chuyến thuộc về ai, và ai biết mình bị ghi đè?</p>
<details><summary>Xem đáp án</summary><p>Chuyến thuộc về <b>người commit sau</b>, và không ai biết: cả hai đều nhận «commit thành công». Mức cô lập mặc định của PostgreSQL không ngăn chuyện này vì mỗi bên kiểm tra trên dữ liệu đã đọc từ trước. Đây là <b>mất cập nhật</b> (lost update); bài sẽ tái hiện nó rồi chặn bằng cột <code>version</code>.</p></details>
</div>

## Ba vấn đề khác nhau thường bị gộp

Ba cơ chế dưới đây hay bị gọi chung là «an toàn khi ghi», nhưng mỗi cái chống một loại lỗi khác, và cái này không thay được cái kia.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 190" role="img" aria-label="Ba cơ chế và lỗi mỗi cơ chế chống: transaction chống ghi dở dang, cột version chống mất cập nhật, khóa idempotency chống request lặp."><defs><marker id="py25-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="30" width="270" height="130" rx="10" class="f-phys-soft s-phys"/><text x="36" y="60" class="t-b">Transaction</text><text x="36" y="86" class="t-small">Chống: ghi dở dang</text><text x="36" y="110" class="t-small">Chuyến và khóa cùng commit</text><text x="36" y="128" class="t-small">hoặc cùng rollback</text><rect x="310" y="30" width="270" height="130" rx="10" class="f-pk-soft s-pk"/><text x="326" y="60" class="t-b">Cột version</text><text x="326" y="86" class="t-small">Chống: mất cập nhật</text><text x="326" y="110" class="t-small">UPDATE ... WHERE version = v</text><text x="326" y="128" class="t-small">0 hàng nghĩa là có người chen</text><rect x="600" y="30" width="260" height="130" rx="10" class="f-warn-soft s-warn"/><text x="616" y="60" class="t-b">Khóa idempotency</text><text x="616" y="86" class="t-small">Chống: request bị gửi lặp</text><text x="616" y="110" class="t-small">UNIQUE (customer_id, key)</text><text x="616" y="128" class="t-small">23505 nghĩa là đã có kết quả</text></svg>
</div>
<figcaption>Hình 1. Mỗi cơ chế trả lời một câu hỏi khác nhau. Transaction giữ một nhóm thay đổi nguyên vẹn, version phát hiện người ghi dựa trên dữ liệu cũ, còn khóa idempotency nhận ra cùng một ý định đến hai lần.</figcaption>
</figure>

Chỉ có transaction thì hai tài xế vẫn ghi đè nhau (mỗi người một transaction hợp lệ). Chỉ có version thì request lặp vẫn tạo hai chuyến (hai chuyến mới, không ai sửa chuyến nào). Chỉ có khóa idempotency thì hai tài xế vẫn nhận trùng chuyến. Phần dưới đi qua từng cái, với số liệu từ lab chạy trên PostgreSQL thật.

## Transaction với Session.begin(): tất cả hoặc không gì cả

Transaction gom nhiều thay đổi thành một đơn vị: hoặc tất cả được commit, hoặc không gì cả. Trong SQLAlchemy 2, `with Session(engine) as session, session.begin():` mở transaction, commit khi khối kết thúc êm và rollback khi có exception. Lab (`main.py`, trích) ghi một chuyến, cho một exception nổ ra, rồi hỏi database:

```python
try:
    with Session(engine) as session, session.begin():
        session.add(Trip(trip_id="TN-0001", customer_id="CU-001", fare=D("12000")))
        session.flush()  # câu INSERT đã gửi, nhưng chưa COMMIT
        seen_inside = session.scalar(select(func.count()).select_from(Trip))
        with Session(engine) as other:  # một kết nối khác nhìn vào
            seen_other = other.scalar(select(func.count()).select_from(Trip))
        raise RuntimeError("mô phỏng lỗi sau khi ghi chuyến")
except RuntimeError:
    pass
```

```text
Trong transaction thấy 1 chuyến; kết nối khác thấy 0; sau rollback còn 0
```

Ba con số kể trọn câu chuyện. Trong transaction, dòng vừa ghi đã thấy được. Kết nối khác **không** thấy dữ liệu chưa commit (không có dirty read, kể cả ở mức mặc định). Sau rollback dòng biến mất. Transaction của ta là cục bộ trong một database: khi ghi chuyến xong ta phải gọi một service khác (thanh toán, chấm điểm), đừng giữ transaction mở trong lúc chờ; transaction dài giữ khóa và làm bảng `trips` nghẽn. Bài 39 giải quyết bài toán «ghi database và phát sự kiện» bằng outbox.

**Isolation level** quyết định một transaction thấy thay đổi của transaction khác thế nào. PostgreSQL mặc định `read committed`, lab xác nhận bằng `SHOW transaction_isolation`. Ở mức này mỗi *câu lệnh* thấy dữ liệu đã commit tại thời điểm câu đó bắt đầu, nên hai lần đọc trong cùng transaction có thể ra hai kết quả. Ta thử với cước của một chuyến, trong khi một session khác sửa nó giữa hai lần đọc:

```text
READ COMMITTED : đọc lần 1 = 12000, đọc lần 2 = 14250 (non-repeatable read)
REPEATABLE READ: đọc lần 1 = 14250, đọc lần 2 = 14250, ghi chồng -> SQLSTATE 40001
```

Mức `REPEATABLE READ` (đặt bằng `session.connection(execution_options={"isolation_level": "REPEATABLE READ"})` trước câu lệnh đầu tiên của transaction) chụp một snapshot: hai lần đọc giống nhau dù bên ngoài đã sửa thành 15.000. Cái giá nằm ở dòng thứ hai: khi transaction này cố ghi chồng lên dòng đã bị người khác sửa sau snapshot, PostgreSQL từ chối với SQLSTATE `40001` (`serialization_failure`, thông báo `could not serialize access due to concurrent update`, lab in đúng dòng đó từ `exc.orig.diag.message_primary`) thay vì lặng lẽ ghi đè. Tài liệu PostgreSQL nói rõ ứng dụng ở mức này phải sẵn sàng **thử lại toàn bộ transaction**. `booking.py` có sẵn khung đó:

```python
for attempt in range(1, max_attempts + 1):
    try:
        with Session(engine) as session, session.begin():
            if isolation_level:
                # Phải đặt trước câu lệnh đầu tiên của transaction.
                session.connection(execution_options={"isolation_level": isolation_level})
            return work(session), attempt
    except DBAPIError as exc:
        sqlstate = getattr(exc.orig, "sqlstate", None)
        if sqlstate not in RETRYABLE_SQLSTATES or attempt == max_attempts:
            raise
```

Chỉ hai mã được thử lại: `40001` và `40P01` (deadlock). Mọi lỗi khác nổi lên ngay, và số lần thử bị chặn, vì retry vô hạn chỉ chuyển một sự cố nhỏ thành sự cố lớn. Trong lab, lần thử đầu bị một session khác chen vào nên dính `40001`, lần hai đọc snapshot mới và thành công: `run_in_transaction thử lại toàn bộ khi 40001: dùng 2 lần, cước cuối = 16500`. Retry phải chạy lại **cả** `work`, kể cả bước đọc, vì quyết định của lần đầu dựa trên dữ liệu đã cũ.

## Mất cập nhật: khi không ai báo lỗi

Quay lại câu hỏi đầu bài. Ở `read committed`, mỗi transaction đều hợp lệ, nhưng kiểu «đọc, kiểm tra trong Python, rồi ghi» vẫn sai. Lab dùng hai `Session` mô phỏng hai tài xế:

```python
with Session(engine) as a, Session(engine) as b:
    trip_a = a.get(Trip, "TN-0003")  # cả hai cùng thấy driver_id = None
    trip_b = b.get(Trip, "TN-0003")
    for who, session, trip in (("DR-001", a, trip_a), ("DR-002", b, trip_b)):
        if trip.driver_id is None:  # kiểm tra trên dữ liệu đã đọc: cả hai đều qua
            trip.driver_id = who
            session.commit()
```

```text
Không version: DR-001 commit thành công, DR-002 commit thành công; driver cuối cùng = DR-002
```

ORM chỉ phát `UPDATE trips SET driver_id=... WHERE trip_id=...`, điều kiện duy nhất là khóa chính, nên câu thứ hai đè lên câu thứ nhất mà không có gì để phản đối. Lỗi không nằm ở PostgreSQL mà ở chỗ kiểm tra «chưa có tài xế» diễn ra trước khi ghi, trên dữ liệu mà người khác có thể đã đổi. Đây là hình dạng check-then-act của bài [Lock, Queue, Semaphore](/articles/python-concurrency-lock-queue-semaphore), nhưng lock trong tiến trình không giúp được khi nhiều server ghi vào cùng một database.

## Optimistic concurrency: cột version

Cách chữa là đưa phần «kiểm tra» vào chính câu `UPDATE`. Bảng `trips` có cột `version` (bắt đầu bằng 1). Người ghi nhớ version đã đọc và chỉ ghi nếu nó **chưa đổi**; mỗi lần ghi tăng version. Gọi là *optimistic* vì ta không khóa gì khi đọc, ta lạc quan rằng ít khi có ai chen vào, và chỉ trả giá (đọc lại) khi có. `booking.py` (`accept_trip`, trích):

```python
for attempt in range(1, max_attempts + 1):
    with Session(engine) as session, session.begin():
        row = session.execute(
            select(Trip.driver_id, Trip.status, Trip.version).where(Trip.trip_id == trip_id)
        ).one()
        if row.driver_id is not None or row.status != TripStatus.SEARCHING:
            raise TripNotOpen(trip_id, row.driver_id, row.status)
        if before_write:
            before_write(attempt)  # chỗ test chèn một người ghi khác
        result = session.execute(
            update(Trip)
            .where(Trip.trip_id == trip_id, Trip.version == row.version)
            .values(driver_id=driver_id, status=TripStatus.ACCEPTED, version=Trip.version + 1)
        )
        if result.rowcount == 1:
            return attempt
    # rowcount == 0: version đã đổi, vòng sau đọc lại.
raise ConflictRetriesExhausted(f"{trip_id}: version đổi liên tục sau {max_attempts} lần thử")
```

Điều kiện `Trip.version == row.version` làm việc đúng ngay ở mức `read committed` nhờ cách PostgreSQL xử lý `UPDATE`: nếu dòng đích đã bị một transaction khác sửa, câu `UPDATE` chờ transaction đó xong, rồi **đánh giá lại** `WHERE` trên phiên bản mới của dòng. Version đã tăng thì điều kiện không khớp nữa và câu lệnh ảnh hưởng 0 hàng. Ta kiểm bằng `result.rowcount`: bằng 1 nghĩa là ta thắng, bằng 0 nghĩa là có người chen vào.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 290" role="img" aria-label="Hai làn thời gian. Cả hai tài xế đọc version 1. Tài xế thứ hai UPDATE trước và thành công, version thành 2. UPDATE của tài xế thứ nhất ảnh hưởng 0 hàng, anh ta đọc lại và nhận TripNotOpen."><defs><marker id="py25-1-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><text x="20" y="92" class="t-b">DR-001</text><text x="20" y="202" class="t-b">DR-002</text><rect x="80" y="55" width="148" height="64" rx="8" class="f-pk-soft s-pk"/><text x="94" y="82" class="t-b">Đọc</text><text x="94" y="102" class="t-small">version = 1</text><rect x="554" y="55" width="148" height="64" rx="8" class="f-warn-soft s-warn"/><text x="568" y="82" class="t-b">UPDATE ... v=1</text><text x="568" y="102" class="t-small">0 hàng: bị chen</text><rect x="712" y="55" width="148" height="64" rx="8" class="f-pk-soft s-pk"/><text x="726" y="82" class="t-b">Đọc lại</text><text x="726" y="102" class="t-small">TripNotOpen</text><rect x="238" y="165" width="148" height="64" rx="8" class="f-pk-soft s-pk"/><text x="252" y="192" class="t-b">Đọc</text><text x="252" y="212" class="t-small">version = 1</text><rect x="396" y="165" width="148" height="64" rx="8" class="f-phys-soft s-phys"/><text x="410" y="192" class="t-b">UPDATE ... v=1</text><text x="410" y="212" class="t-small">1 hàng, v thành 2</text><path d="M470 165 L470 140 L628 140 L628 123" class="f-none s-warn dash" stroke-width="1.5" marker-end="url(#py25-1-arrow)"/><path d="M80 262 L860 262" class="f-none s-muted" stroke-width="1.5" marker-end="url(#py25-1-arrow)"/><text x="80" y="282" class="t-small">thời gian</text></svg>
</div>
<figcaption>Hình 2. Người ghi sau dựa trên version cũ nên không ghi được; bước đọc lại cho thấy chuyến đã có tài xế và kết quả nghiệp vụ là từ chối, không phải thử mù lần nữa.</figcaption>
</figure>

Lab chạy ba tình huống bằng điểm chèn `before_write`, nên thứ tự xen kẽ luôn giống nhau, không có `sleep`:

```text
Có version: DR-001 nhận TripNotOpen (driver hiện tại DR-002); driver cuối = DR-002, version = 2
Xung đột không liên quan: nhận được ở lần thử 2; driver = DR-001, version = 3
Version đổi liên tục: ConflictRetriesExhausted sau 3 lần; driver = None
```

Dòng đầu là tình huống của câu hỏi mở bài, giờ có kết cục đúng: DR-002 ghi trước, DR-001 bị 0 hàng, đọc lại thấy chuyến đã có chủ và nhận `TripNotOpen`, một **kết quả nghiệp vụ** để báo «chuyến đã có người nhận», không phải lý do để thử tiếp. Dòng hai là xung đột vô hại: ai đó sửa cước làm version tăng, DR-001 bị 0 hàng, đọc lại thấy chuyến vẫn trống và nhận được ở lần hai; đây là lý do retry có ích. Dòng ba cho thấy giới hạn số lần thử: version đổi liên tục thì sau ba lần ta dừng và báo lỗi.

Hai quy tắc rút ra từ đó. Một: **retry là đọc lại rồi quyết định lại**, không phát lại câu `UPDATE` cũ (đó chính là cách ghi đè mù). Hai: mọi đường ghi vào bảng đều phải đổi version, chỉ cần một đường quên là cơ chế mất tác dụng với đường đó; lab tự tăng `version=Trip.version + 1` trong từng `UPDATE`.

SQLAlchemy cũng làm hộ phần này nếu ta khai báo `version_id_col` trong mapper (`VersionedTrip` ở `models.py`): ORM thêm `AND version = <cũ>` vào mọi `UPDATE`, tự tăng version, và ném `StaleDataError` khi 0 hàng khớp. Lab chạy hai session sửa cùng một dòng và ghi lại: `version_id_col: người thứ hai nhận StaleDataError; driver = DR-001, version = 2`. Khóa bi quan (`SELECT ... FOR UPDATE`, `with_for_update()`) chặn người khác từ lúc đọc, hợp khi tranh chấp cao; lab không chạy nó.

## Idempotency: khóa bền và UNIQUE

Idempotent nghĩa là gửi cùng một ý định nhiều lần cho cùng một kết quả bền, không tạo thêm hiệu ứng. Với đặt chuyến, client sinh một **idempotency key** cho mỗi ý định (một giá trị ngẫu nhiên giữ nguyên khi gửi lại), server lưu key cùng kết quả trong database. Bảng `booking_requests` có ràng buộc `UNIQUE (customer_id, idempotency_key)` và hai cột `service`, `distance_km` để so khớp nội dung ý định. `fare` được tính bằng `calculate_fare(distance_km, service=service)` (miền 0 đến 100 km, như bài 03) ngay đầu hàm, nên khoảng cách ngoài miền bị từ chối trước khi tốn một số sequence nào. Quan trọng nhất là hai điều: chuyến và khóa được ghi trong **cùng một transaction** (không có «có khóa mà mất chuyến»), và quyết định cuối cùng thuộc về ràng buộc UNIQUE, không thuộc về câu `SELECT` kiểm tra trước. `book_trip` (trích):

```python
try:
    with Session(engine) as session, session.begin():
        trip_id = f"TN-{session.scalar(TRIP_SEQ.next_value()):04d}"
        session.add(Trip(trip_id=trip_id, customer_id=customer_id, service=service,
                         status=TripStatus.REQUESTED, fare=fare))
        session.flush()  # chuyến phải có trước khi khóa trỏ khóa ngoại tới nó
        session.add(BookingRequest(customer_id=customer_id, idempotency_key=key, service=service,
                                   distance_km=distance_km, trip_id=trip_id))
    return BookingResult(trip_id, replayed=False)
except IntegrityError as exc:
    orig = exc.orig
    if getattr(orig, "sqlstate", None) != UNIQUE_VIOLATION or (
        orig.diag.constraint_name != "uq_booking_requests_customer_key"
    ):
        raise
    conflict = f"{orig.sqlstate}: {orig.diag.message_primary}"
```

Câu `except` ở đây hẹp có chủ ý: chỉ nhận **đúng** SQLSTATE `23505` (`unique_violation`) của **đúng** ràng buộc khóa idempotency. `IntegrityError` bao trùm cả khóa ngoại sai, `NOT NULL` và những lỗi khác, và nuốt hết chúng sẽ che mất bug thật. `exc.orig` là exception gốc của psycopg, có `.sqlstate` và `.diag.constraint_name`. Khi đúng là 23505, transaction của ta đã rollback (kể cả chuyến ứng viên), người thắng đã commit, nên ta mở session mới đọc lại khóa của họ và trả đúng kết quả đó. Trước bước ghi, hàm còn có một lần `SELECT` đọc nhanh: nếu khóa đã có thì trả luôn; nhưng nó chỉ là tối ưu cho trường hợp thường gặp, còn tính đúng nằm ở UNIQUE.

Kiểm bằng các ca sau. Gửi hai lần cùng khóa, rồi cùng khóa nhưng đổi nội dung, rồi một khoảng cách ngoài miền, rồi hai request chạy đồng thời. Với ca đồng thời, `threading.Barrier(2)` giữ cả hai thread **sau** bước kiểm tra «chưa có» và **trước** khi ghi, để cả hai chắc chắn cùng thấy khóa chưa tồn tại:

```text
Gửi req-1 hai lần: lần 1 replayed=False, lần 2 replayed=True, cùng chuyến: True
Cùng khóa req-1 nhưng 6 km: IdempotencyConflict
Cùng khóa req-1 nhưng đổi dịch vụ sang food: IdempotencyConflict
Đặt 100,5 km: khoảng cách ngoài miền 0..100 km: 100.5
Thông báo PostgreSQL cho request thua: 23505: duplicate key value violates unique constraint "uq_booking_requests_customer_key"
Hai request req-2 đồng thời: 1 tạo chuyến, 1 bị 23505 rồi đọc lại; cùng trip_id: True
Trong database: 2 bản ghi khóa, 2 chuyến sinh bởi book_trip
Sequence đã cấp tới số 102 nhưng chỉ có 2 chuyến: ca thua đã rollback chuyến ứng viên, số của nó không được dùng lại
```

Ca «cùng khóa khác nội dung» bị từ chối bằng `IdempotencyConflict`: khóa đã dùng cho 5 km `ride` mà request mới đòi 6 km, hoặc đổi sang `food`, là lỗi của client, không được trả lại chuyến cũ như thể không có gì. Ca đồng thời: một thread tạo chuyến, thread kia bị 23505 rồi trả về chuyến của người thắng, cuối cùng chỉ có 2 chuyến cho 2 khóa. Dòng cuối là chi tiết hay bị quên: `sequence` của PostgreSQL không rollback, nên số `101` hoặc `102` của ca thua bị bỏ lại và mã chuyến có khoảng trống. Mã chuyến cần **duy nhất**, không cần **liên tục**; nếu nghiệp vụ đòi liên tục (hóa đơn chẳng hạn) thì phải thiết kế riêng.

Cách khác là `INSERT ... ON CONFLICT DO NOTHING` rồi kiểm số hàng ảnh hưởng. Dù chọn cách nào, lớp quyết định phải là ràng buộc của database: `SELECT` rồi mới `INSERT` luôn có khe hở giữa hai bước.

## Chạy và kiểm chứng

Cần PostgreSQL của hạ tầng dùng chung đã chạy. Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/25-transaction-optimistic-idempotency examples/toi-noi-python/25-transaction-optimistic-idempotency/main.py --verify
```

Lab dùng schema `lab25` (tạo bằng `create_all` cho gọn vì trọng tâm là transaction, bài 24 đã nói về Alembic) và `DROP SCHEMA lab25 CASCADE` khi xong. Output đầy đủ lần chạy cuối:

```text
Isolation mặc định của PostgreSQL: read committed
Trong transaction thấy 1 chuyến; kết nối khác thấy 0; sau rollback còn 0
READ COMMITTED : đọc lần 1 = 12000, đọc lần 2 = 14250 (non-repeatable read)
REPEATABLE READ: đọc lần 1 = 14250, đọc lần 2 = 14250, ghi chồng -> SQLSTATE 40001
Thông báo PostgreSQL cho 40001: could not serialize access due to concurrent update
run_in_transaction thử lại toàn bộ khi 40001: dùng 2 lần, cước cuối = 16500
Không version: DR-001 commit thành công, DR-002 commit thành công; driver cuối cùng = DR-002
Có version: DR-001 nhận TripNotOpen (driver hiện tại DR-002); driver cuối = DR-002, version = 2
Xung đột không liên quan: nhận được ở lần thử 2; driver = DR-001, version = 3
Version đổi liên tục: ConflictRetriesExhausted sau 3 lần; driver = None
version_id_col: người thứ hai nhận StaleDataError; driver = DR-001, version = 2
Gửi req-1 hai lần: lần 1 replayed=False, lần 2 replayed=True, cùng chuyến: True
Cùng khóa req-1 nhưng 6 km: IdempotencyConflict
Thông báo PostgreSQL cho request thua: 23505: duplicate key value violates unique constraint "uq_booking_requests_customer_key"
Hai request req-2 đồng thời: 1 tạo chuyến, 1 bị 23505 rồi đọc lại; cùng trip_id: True
Trong database: 2 bản ghi khóa, 2 chuyến sinh bởi book_trip
Sequence đã cấp tới số 102 nhưng chỉ có 2 chuyến: ca thua đã rollback chuyến ứng viên, số của nó không được dùng lại
Đã dọn: DROP SCHEMA lab25 CASCADE
OK 24/24
```

`--verify` chạy 24 kiểm tra, in `OK 24/24` khi đạt; sai thì in tên ca, `FAIL n/m` (n là số ca đạt) và exit 1; chưa bật PostgreSQL thì in `FAIL: không nối được PostgreSQL tại <url> ...` và exit 1. Ta chạy lab nhiều lần liên tiếp và lần nào cũng `OK 24/24`: các ca xen kẽ được dựng bằng điểm chèn và `Barrier`, nên kết quả không phụ thuộc may rủi về thời gian. Phiên bản: Python 3.14.7, SQLAlchemy 2.1.8, psycopg 3.3.8, PostgreSQL 17.11.

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| Hai người cùng «nhận» chuyến mà không lỗi nào | `UPDATE` có điều kiện version chưa; có kiểm `rowcount` sau `UPDATE` không |
| Retry xong dữ liệu bị ghi đè | Retry có đọc lại và quyết định lại không, hay phát lại `UPDATE` cũ |
| `could not serialize access due to concurrent update` (SQLSTATE `40001`) | Đang ở `REPEATABLE READ`; thử lại toàn bộ transaction, có giới hạn số lần |
| Có hai chuyến cho một khóa idempotency | Có ràng buộc UNIQUE ở database không; chuyến và khóa có cùng một transaction không |
| `except IntegrityError` nuốt cả lỗi khóa ngoại | Chỉ bắt đúng SQLSTATE `23505` và đúng `constraint_name` |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>Lab chạy trên một PostgreSQL một node, trong một tiến trình với hai thread, không có HTTP và không có service nào khác trong transaction. Production còn phải lo: key lấy từ đâu và hết hạn khi nào (retention), lưu cả phản hồi gốc để trả lại y hệt thay vì tạo lại, xác thực để <code>customer_id</code> đáng tin cậy, và nhất là bài toán ghi database rồi phát sự kiện ở bài 39 (không nối chúng bằng một transaction dài). Retry trong lab chạy lại ngay, không có backoff (chờ tăng dần cộng nhiễu ngẫu nhiên), còn production cần để nhiều client không cùng thử lại một lúc. Lab cũng chưa dùng <code>SERIALIZABLE</code>, chưa đo tranh chấp ở tải 560 <code>UPDATE</code> mỗi giây, và không chứng minh PostgreSQL xếp lịch khóa theo đúng một thứ tự cố định: điểm chèn chỉ dựng thứ tự xen kẽ của hai transaction, còn bên trong, PostgreSQL tự quyết. Số liệu chuyến và tài xế là dữ liệu giả.</p>
</div>

**Bài tập:** (1) Giả sử `book_trip` chỉ dùng `SELECT` để kiểm tra khóa và bảng không có ràng buộc UNIQUE. Hai server xử lý hai request cùng khóa đồng thời có thể tạo hai chuyến không? (2) Cùng khóa `req-1` nhưng do khách `CU-002` gửi: kết quả là gì, và điều đó đòi hỏi gì ở tầng xác thực?

<details><summary>Xem lời giải</summary><p>(1) <b>Có.</b> Cả hai cùng chạy <code>SELECT</code> khi chưa có khóa, cả hai cùng thấy «chưa có», rồi cả hai <code>INSERT</code> thành công, vì không có gì ngăn. Khe hở giữa kiểm tra và ghi luôn tồn tại với <code>SELECT</code> trước <code>INSERT</code>; chỉ ràng buộc UNIQUE của database đóng được khe đó, bất kể có bao nhiêu server. (2) Phạm vi UNIQUE là <code>(customer_id, idempotency_key)</code> nên <code>CU-002</code> có bản ghi khóa riêng và tạo chuyến riêng, hai khách không dùng chung khóa. Điều đó chỉ đúng khi <code>customer_id</code> đến từ danh tính đã xác thực, không phải từ nội dung request: nếu client tự khai <code>customer_id</code>, họ có thể thử đoán khóa của người khác.</p></details>

Bài trước: [bài 24](/articles/python-sqlalchemy-psycopg-postgresql-mapping-migration). Bài tiếp: [bài 26](/articles/python-mongodb-pymongo-motor-document-ocr).

## Tài liệu gốc

- [PostgreSQL 17: Transaction Isolation](https://www.postgresql.org/docs/17/transaction-iso.html).
- [PostgreSQL 17: Error Codes (SQLSTATE)](https://www.postgresql.org/docs/17/errcodes-appendix.html).
- [PostgreSQL 17: Constraints (UNIQUE)](https://www.postgresql.org/docs/17/ddl-constraints.html).
- [PostgreSQL 17: INSERT ... ON CONFLICT](https://www.postgresql.org/docs/17/sql-insert.html).
- [SQLAlchemy 2.1: Transactions and Connection Management](https://docs.sqlalchemy.org/en/21/orm/session_transaction.html).
- [SQLAlchemy 2.1: Configuring a Version Counter](https://docs.sqlalchemy.org/en/21/orm/versioning.html).
