---
slug: python-integration-test-testcontainers-fixture
title: "Integration test với PostgreSQL/MongoDB thật: testcontainers và fixture"
summary: "Bài 28: dựng PostgreSQL và MongoDB thật bằng testcontainers, fixture scope session cho container, schema riêng cho mỗi test case, SKIP LOCKED và unique index. Có kết quả chạy thật và lab 19 test case chạy được."
category: python-intermediate
level: intermediate
tags: [python, toi-noi, pytest, testcontainers, integration-test]
language: vi
---

Tuần trước dịch vụ điều phối của [Tới Nơi](/articles/du-an-mau-toi-noi) gán chuyến TN-0001 cho hai tài xế cùng lúc. Bộ test vẫn xanh: kho chuyến trong test là một `dict` trong bộ nhớ, không có khoá dòng, không có ràng buộc unique, không có giao dịch song song. Cái dict không thể sai theo cách mà PostgreSQL sai. Test chỉ chứng minh code đúng với một bản giả mà ta tự viết, không chứng minh nó đúng với database thật. Lỗi thật nằm ở một câu `UPDATE` gán tài xế mà không kiểm tra trạng thái của chuyến; mục «Những điều chỉ database thật mới lộ ra» sửa nó và viết đúng test cho lỗi này.

Đây là bài 28 của lộ trình **Python trung bình qua Tới Nơi**. Bài dùng lại cách nối PostgreSQL ở [bài 24](/articles/python-sqlalchemy-psycopg-postgresql-mapping-migration) và MongoDB ở [bài 26](/articles/python-mongodb-pymongo-motor-document-ocr), nhưng lab tự chứa: code chỉ cần `psycopg` và `pymongo`, test cần thêm `testcontainers`. Bài này cần Docker đang chạy trên máy.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Bộ test của dịch vụ điều phối chạy trên SQLite in-memory và xanh cả 100 test case. Điều đó có chứng minh câu lệnh <code>SELECT ... FOR UPDATE SKIP LOCKED</code> ở production chạy đúng không?</p>
<details><summary>Xem đáp án</summary><p><b>Không.</b> SQLite không có cú pháp này, nên hoặc test không bao giờ chạy câu lệnh đó (đã thay bằng một cách khác chỉ có ở bản giả), hoặc nó bị lỗi cú pháp ngay. Bài sẽ chạy cùng câu lệnh trên cả SQLite lẫn PostgreSQL và xem chuyện gì xảy ra.</p></details>
</div>

## Test xanh trên bản giả chứng minh điều gì

Có ba cách quen thuộc để thay database trong test: một lớp giả trong bộ nhớ (dict, list), SQLite in-memory, hoặc thư viện giả lập (fake) của chính database. Cả ba nhanh và không cần cài gì. Cái giá là hành vi: ta kiểm tra code với một hệ thống khác PostgreSQL ở đúng những chỗ quan trọng nhất. Lab đặt cạnh nhau hai khác biệt cụ thể, chạy cùng một ý định trên hai engine (`tests/test_why_not_sqlite.py`, trích):

```python
def test_sqlite_has_no_skip_locked_but_postgresql_does(conn: psycopg.Connection) -> None:
    sql = "SELECT 1 FROM t WHERE x = 1 FOR UPDATE SKIP LOCKED"

    lite = sqlite3.connect(":memory:")
    lite.execute("CREATE TABLE t (x INTEGER)")
    with pytest.raises(sqlite3.OperationalError, match="syntax error"):
        lite.execute(sql)

    conn.execute("CREATE TEMP TABLE t (x integer)")
    conn.execute("BEGIN")
    assert conn.execute(sql).fetchall() == []                # PostgreSQL chạy được
    conn.execute("ROLLBACK")
```

`FOR UPDATE SKIP LOCKED` là cách để hai tài xế nhận hai chuyến khác nhau mà không chờ nhau. SQLite không có nó, nên bản test SQLite hoặc phải bỏ đoạn đó, hoặc bọc lại bằng một cách khác chỉ tồn tại trong test. Khác biệt thứ hai là kiểu dữ liệu. Với bảng SQLite thường, cột khai báo `NUMERIC` nhận cả chuỗi `'abc'` (kiểu chỉ là «affinity», một gợi ý); PostgreSQL từ chối bằng `InvalidTextRepresentation`. Công bằng mà nói, SQLite từ 3.37 có bảng `STRICT` chặn chuỗi trong cột số, và lab có kiểm tra điều đó. Nhưng bảng `STRICT` chỉ có các kiểu `INT`, `INTEGER`, `REAL`, `TEXT`, `BLOB`, `ANY`: không có `numeric(12,0)` cho tiền hay `timestamptz` cho thời điểm UTC, hai kiểu mà bảng `trips` của ta dùng.

Đây không phải lời chê SQLite, một database xuất sắc cho việc của nó. Vấn đề là khi test dùng SQLite thay PostgreSQL, nó trả lời câu hỏi «code chạy được với SQLite không», trong khi production hỏi câu khác. Phân tầng cho gọn: unit test (bài 10) cho logic thuần, **integration test** cho đoạn code chạm database, và integration test chỉ có giá trị khi database là chính loại sẽ chạy ở production, cùng phiên bản chính.

## testcontainers: database thật, vòng đời do pytest giữ

[testcontainers](https://testcontainers-python.readthedocs.io/en/latest/) dựng container Docker từ trong test, lấy cổng ngẫu nhiên, đợi đến khi database nhận kết nối, rồi xoá container khi xong. Fixture PostgreSQL của lab (`tests/conftest.py`, trích):

```python
PG_IMAGE = "pgvector/pgvector:pg17"  # cùng image với infra/compose.yml


@pytest.fixture(scope=_container_scope)
def pg_dsn() -> Iterator[str]:
    """DSN của PostgreSQL. Có TOINOI_PG_URL thì dùng compose sẵn; không thì dựng container."""
    if url := os.environ.get("TOINOI_PG_URL"):
        yield url
        return
    started = time.perf_counter()
    # driver=None: URL dạng postgresql://... (psycopg tự hiểu), không phải postgresql+psycopg2://
    with PostgresContainer(PG_IMAGE, driver=None) as pg:
        print(f"\n[lab28] PostgreSQL container sẵn sàng sau {time.perf_counter() - started:.2f} s")
        yield pg.get_connection_url()
```

Fixture của MongoDB giống hệt, dùng `MongoDbContainer("mongo:8")`. Ba điểm đáng chú ý:

- **Cùng image với production và compose.** Lab dùng `pgvector/pgvector:pg17`, đúng image của `infra/compose.yml`. Nếu production chạy PostgreSQL 17 mà test chạy bản 12, ta lại kiểm chứng một database khác.
- **`yield` bên trong `with`.** Khối `with` giữ container sống suốt thời gian fixture; pytest chạy phần sau `yield` khi hết scope, container bị dừng và xoá.
- **Cổng ngẫu nhiên.** `get_connection_url()` trả URL với cổng host do Docker cấp, nên hai lần chạy song song (hai nhánh CI, hai người cùng máy) không trùng cổng.

Ở testcontainers-python 4.15 (bản lab dùng), module cũ như `testcontainers.postgres` vẫn còn nhưng in `DeprecationWarning` và trỏ sang `testcontainers.community.postgres`; lab import đường dẫn mới.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="pytest gọi testcontainers, testcontainers điều khiển Docker Engine dựng container PostgreSQL và MongoDB với cổng ngẫu nhiên; một container Ryuk riêng canh tiến trình test và thường xoá container sau khoảng 10 giây khi tiến trình chết, không bảo đảm."><defs><marker id="py28-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker><marker id="py28-0-warn" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs><rect x="20" y="40" width="170" height="74" rx="10" class="f-pk-soft s-pk"/><text x="36" y="72" class="t-b">pytest</text><text x="36" y="96" class="t-small">fixture scope session</text><rect x="240" y="40" width="190" height="74" rx="10" class="f-pk-soft s-pk"/><text x="256" y="72" class="t-b">testcontainers</text><text x="256" y="96" class="t-small">start, đợi sẵn sàng, stop</text><rect x="480" y="40" width="150" height="74" rx="10" class="f-phys-soft s-phys"/><text x="496" y="72" class="t-b">Docker Engine</text><text x="496" y="96" class="t-small">socket cục bộ</text><rect x="680" y="24" width="180" height="42" rx="8" class="f-phys-soft s-phys"/><text x="694" y="50" class="t-small">PostgreSQL, cổng ngẫu nhiên</text><rect x="680" y="88" width="180" height="42" rx="8" class="f-phys-soft s-phys"/><text x="694" y="114" class="t-small">MongoDB, cổng ngẫu nhiên</text><path d="M190 77 L238 77" class="f-none s-phys" stroke-width="2" marker-end="url(#py28-0-arrow)"/><path d="M430 77 L478 77" class="f-none s-phys" stroke-width="2" marker-end="url(#py28-0-arrow)"/><path d="M630 60 L678 47" class="f-none s-phys" stroke-width="2" marker-end="url(#py28-0-arrow)"/><path d="M630 94 L678 108" class="f-none s-phys" stroke-width="2" marker-end="url(#py28-0-arrow)"/><rect x="240" y="170" width="390" height="64" rx="10" class="f-warn-soft s-warn"/><text x="256" y="196" class="t-b">Ryuk: container canh gác</text><text x="256" y="216" class="t-small">giữ kết nối tới tiến trình test; kết nối đứt thì xoá container</text><path d="M555 170 L555 116" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py28-0-warn)"/></svg>
</div>
<figcaption>Hình 1. Đường đi từ fixture tới container. Ryuk là lưới an toàn có giới hạn: nếu tiến trình test bị giết giữa chừng, nó thường dọn container sau khoảng 10 giây, nhưng không bảo đảm.</figcaption>
</figure>

Để chạy trên hạ tầng compose có sẵn thay vì dựng container (nhanh hơn khi chạy lặp), fixture đọc `TOINOI_PG_URL` và `TOINOI_MONGO_URL`, hai biến của hạ tầng chung. Test không đổi một dòng nào, chỉ nguồn database đổi, và vì mỗi test case có schema riêng (mục sau), compose không bị dữ liệu test làm bẩn.

## Fixture scope: container đắt, schema rẻ

Nguyên tắc chọn scope cho fixture hạ tầng: **thứ đắt thì để scope rộng, thứ rẻ mà phải sạch thì scope hẹp**. Container mất khoảng 1,5 giây để dựng, nên chọn scope `session`. Schema chỉ là một câu `CREATE SCHEMA`, nên dựng mới cho từng test case (`scope="function"`, mặc định):

```python
@pytest.fixture
def pg_schema(pg_dsn: str) -> Iterator[str]:
    """Một schema mới cho mỗi ca test, có bảng từ DDL; xóa cả schema khi ca xong."""
    name = f"t_{uuid.uuid4().hex[:10]}"
    with psycopg.connect(pg_dsn, autocommit=True) as admin:
        admin.execute(f'CREATE SCHEMA "{name}"')
        admin.execute(f'SET search_path TO "{name}"')
        admin.execute(DDL)
    try:
        yield name
    finally:
        with psycopg.connect(pg_dsn, autocommit=True) as admin:
            admin.execute(f'DROP SCHEMA "{name}" CASCADE')
```

Fixture `connect` mở các kết nối với `options=-c search_path=<schema>`, nên `TripRepository` viết `FROM trips` mà không cần tiền tố, và test case nào cũng có thể dùng mã `TN-0001` mà không đụng nhau. Test case cần hai phiên (khoá dòng) gọi `connect()` hai lần. Với MongoDB, mỗi test case dùng một database tên `t_<hex>` và `drop_database` khi xong.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 310" role="img" aria-label="Một dải container PostgreSQL và MongoDB dựng một lần cho cả phiên test; bên dưới ba test case, mỗi test case tạo một schema riêng, chạy rồi xoá schema đó."><defs><marker id="py28-1-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="16" width="840" height="100" rx="10" class="f-phys-soft s-phys"/><text x="36" y="42" class="t-b">scope session: container dựng một lần cho cả phiên, khoảng 1,5 s mỗi loại</text><rect x="36" y="58" width="390" height="44" rx="8" class="f-bg s-phys"/><text x="52" y="85" class="t-mono">PostgreSQL pgvector/pgvector:pg17</text><rect x="452" y="58" width="392" height="44" rx="8" class="f-bg s-phys"/><text x="468" y="85" class="t-mono">MongoDB mongo:8</text><path d="M150 116 L150 154" class="f-none s-phys" stroke-width="2" marker-end="url(#py28-1-arrow)"/><path d="M440 116 L440 154" class="f-none s-phys" stroke-width="2" marker-end="url(#py28-1-arrow)"/><path d="M730 116 L730 154" class="f-none s-phys" stroke-width="2" marker-end="url(#py28-1-arrow)"/><rect x="20" y="156" width="260" height="110" rx="10" class="f-pk-soft s-pk"/><text x="36" y="182" class="t-b">ca test 1</text><text x="36" y="206" class="t-mono">CREATE SCHEMA t_3f9a1c</text><text x="36" y="226" class="t-small">chạy ca, bảng trips trống</text><text x="36" y="248" class="t-mono">DROP SCHEMA ... CASCADE</text><rect x="310" y="156" width="260" height="110" rx="10" class="f-pk-soft s-pk"/><text x="326" y="182" class="t-b">ca test 2</text><text x="326" y="206" class="t-mono">CREATE SCHEMA t_b07e42</text><text x="326" y="226" class="t-small">chạy ca, bảng trips trống</text><text x="326" y="248" class="t-mono">DROP SCHEMA ... CASCADE</text><rect x="600" y="156" width="260" height="110" rx="10" class="f-pk-soft s-pk"/><text x="616" y="182" class="t-b">ca test 3</text><text x="616" y="206" class="t-mono">CREATE SCHEMA t_91d5c8</text><text x="616" y="226" class="t-small">chạy ca, bảng trips trống</text><text x="616" y="248" class="t-mono">DROP SCHEMA ... CASCADE</text><text x="20" y="294" class="t-small">Container đắt nên dùng chung; schema rẻ (khoảng 0,02 s) nên tạo mới cho từng ca.</text></svg>
</div>
<figcaption>Hình 2. Hai tầng vòng đời. Tầng trên dùng chung giữa mọi test case; tầng dưới tách dữ liệu từng test case, nên thứ tự chạy test không ảnh hưởng kết quả.</figcaption>
</figure>

Lab cho phép đo chính xác cái giá của việc chọn sai scope. `_container_scope` là một *dynamic scope*: pytest cho phép tham số `scope` là hàm trả về tên scope, và lab đọc biến `LAB28_SCOPE`:

```python
def _container_scope(fixture_name: str, config: pytest.Config) -> str:
    """Scope động: mặc định `session`; đặt LAB28_SCOPE=function để đo giá của sai lầm kinh điển."""
    return os.environ.get("LAB28_SCOPE", "session")
```

Cùng 19 test case, container dựng một lần cho cả phiên: khoảng 7 giây. Container dựng lại cho từng test case: từ 39 đến 48 giây (số đo ở mục «Chạy và kiểm chứng»). Đây là lý do nhiều bộ integration test bị bỏ khỏi quy trình hằng ngày: không phải vì chúng chậm bản chất, mà vì chúng dựng lại thứ đắt không cần thiết.

## Những điều chỉ database thật mới lộ ra

Chạy trên PostgreSQL thật cho phép test kiểm tra đúng các ràng buộc mà schema đang đảm nhiệm. Bảng `trips` trong `trips_db.py` (trích):

```sql
CREATE TABLE trips (
    trip_id      text PRIMARY KEY CHECK (trip_id ~ '^TN-[0-9]{4}$'),
    ...
    distance_km  numeric(6, 3) NOT NULL CHECK (distance_km BETWEEN 0 AND 100),
    fare_vnd     numeric(12, 0) NOT NULL CHECK (fare_vnd >= 0),
    status       text NOT NULL DEFAULT 'requested' CHECK (status IN (...)),
    created_at   timestamptz NOT NULL DEFAULT now()
);
-- Một tài xế chỉ có một chuyến đang chạy: ràng buộc nằm ở database, không chỉ ở code.
CREATE UNIQUE INDEX one_active_trip_per_driver
    ON trips (driver_id)
    WHERE driver_id IS NOT NULL
      AND status IN ('accepted', 'arriving', 'picked_up', 'in_transit');
```

Chỉ mục unique có điều kiện (partial index) là chỗ đáng giá nhất. Chỉ mục này biến quy tắc «tài xế bận không được nhận thêm chuyến» thành điều database tự đảm bảo ngay cả khi hai tiến trình của dịch vụ gán chuyến cùng lúc. Test kiểm tra cả hai phía của quy tắc, bị chặn khi bận và được nhận khi chuyến trước đã hoàn thành:

```python
def test_driver_cannot_hold_two_active_trips(trips: TripRepository) -> None:
    queue_trip(trips, "TN-0001")
    queue_trip(trips, "TN-0002")
    trips.assign_driver("TN-0001", "DR-001")

    with pytest.raises(DriverBusy):
        trips.assign_driver("TN-0002", "DR-001")

    after = trips.get("TN-0002")
    assert (after.driver_id, after.status) == (None, "searching")   # lệnh bị từ chối không để lại dấu vết
```

Lỗi nêu ở đầu bài nằm ở chỗ khác: bản đầu của `assign_driver` chỉ viết `UPDATE trips SET driver_id = ..., status = 'accepted' WHERE trip_id = ...`, nên gọi lần hai cho tài xế khác ghi đè lên chuyến đã nhận mà không báo lỗi. Bản sửa đưa trạng thái xuất phát vào `WHERE` và kiểm tra số dòng bị ảnh hưởng (`trips_db.py`, trích):

```python
def assign_driver(self, trip_id: str, driver_id: str) -> None:
    try:
        self._transition(trip_id, "searching", "accepted", driver_id=driver_id)
    except pg_errors.UniqueViolation as exc:
        raise DriverBusy(f"{driver_id} đang có chuyến chưa xong") from exc

def _transition(self, trip_id, from_status, to_status, driver_id=None) -> None:
    cur = self._conn.execute(
        "UPDATE trips SET status = %s, driver_id = COALESCE(%s, driver_id)"
        " WHERE trip_id = %s AND status = %s",
        (to_status, driver_id, trip_id, from_status),
    )
    if cur.rowcount != 1:
        raise InvalidTransition(f"{trip_id}: không thể chuyển {from_status} -> {to_status}")
```

Test case `test_second_driver_cannot_overwrite_an_accepted_trip` tái hiện đúng lỗi (nhận bằng DR-001 rồi thử gán DR-002, kỳ vọng `InvalidTransition` và tài xế vẫn là DR-001), và `test_cannot_assign_a_driver_before_the_trip_is_searching` kiểm tra trạng thái `requested` chưa được gán. Lab chỉ cài đặt luồng rút gọn requested, searching, accepted; các trạng thái sau (arriving, picked_up, in_transit, completed) test đặt thẳng bằng SQL. Việc cài đủ 8 trạng thái và mọi bước chuyển hợp lệ thuộc về dịch vụ thật.

Với hai tài xế gọi nhận chuyến cùng lúc, lab mở hai kết nối thật, mỗi kết nối một giao dịch đang mở, đúng như hai tiến trình dịch vụ:

```python
def test_skip_locked_gives_two_drivers_two_different_trips(connect) -> None:
    a, b = connect(), connect()
    queue_trip(TripRepository(a), "TN-0001")
    queue_trip(TripRepository(a), "TN-0002", "4")

    with a.transaction():
        first = TripRepository(a).claim_next_searching("DR-001")
        # Giao dịch của a còn mở, TN-0001 đang bị khóa; b không chờ mà lấy chuyến kế tiếp.
        with b.transaction():
            second = TripRepository(b).claim_next_searching("DR-002")

    assert (first, second) == ("TN-0001", "TN-0002")
```

Test kế tiếp trong lab là test phản chứng: cùng tình huống nhưng dùng `FOR UPDATE` thuần, kết nối thứ hai phải chờ khoá và (nhờ `lock_timeout = '300ms'`) bị `LockNotAvailable`. Không có database thật thì không thể viết được cả hai test này.

MongoDB cũng có bẫy riêng. Lần chạy đầu của `test_created_at_comes_back_timezone_aware`, khi fixture tạo client với tham số mặc định, đã đỏ (trích):

```text
>       assert doc["created_at"].tzinfo is not None
E       assert None is not None
E        +  where None = datetime.datetime(2026, 11, 6, 11, 19, 32, 72000).tzinfo
```

PyMongo mặc định trả `datetime` *naive* dù giá trị lưu là UTC. Quy ước của loạt bài là thời điểm luôn aware, nên fixture tạo client với `tz_aware=True`, và test case này xanh. Một kho giả trong bộ nhớ sẽ trả về chính cái `datetime` ta đã nhét vào, aware hay không là do ta, nên lỗi này không bao giờ lộ ra. Tương tự, `test_unique_index_rejects_second_document_of_same_type` kiểm tra `DuplicateKeyError` với mã 11000 do chỉ mục unique `(kyc_id, document_type)` sinh ra: hành vi của server, không phải của thư viện client.

## Chạy và kiểm chứng

Lab ở `examples/toi-noi-python/28-integration-test-testcontainers/`. Cần Docker đang chạy; lần đầu cần mạng để kéo image `pgvector/pgvector:pg17`, `mongo:8` và image Ryuk.

```bash
uv run --project examples/toi-noi-python/28-integration-test-testcontainers pytest examples/toi-noi-python/28-integration-test-testcontainers -q -s --durations=3
```

Output của lần chạy cuối (Python 3.14.7, pytest 9.2.0, testcontainers 4.15.0, psycopg 3.3.8, pymongo 4.19.0):

```text
[lab28] MongoDB container sẵn sàng sau 1.55 s
....
[lab28] PostgreSQL container sẵn sàng sau 1.49 s
.............
============================= slowest 3 durations ==============================
3.13s setup    tests/test_ocr_mongodb.py::test_save_and_read_back
1.53s setup    tests/test_trips_postgres.py::test_roundtrip_keeps_decimal_and_aware_time
0.47s teardown tests/test_why_not_sqlite.py::test_sqlite_has_no_skip_locked_but_postgresql_does
19 passed in 6.00s
```

Hai test case đầu tiên của mỗi database gánh chi phí dựng container; các test case sau có setup 0,02 đến 0,03 giây và teardown 0,00 đến 0,01 giây (theo `--durations=0`). Chạy với compose có sẵn (`docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait`, rồi đặt `TOINOI_PG_URL` và `TOINOI_MONGO_URL`):

```text
19 passed in 0.96s
```

Ba lần đo của mỗi chế độ trên máy macOS Apple silicon 10 lõi (dòng cuối của pytest, không kể thời gian khởi động `uv`). Các lần đo chạy lúc máy bận (`uptime` báo load average 16,43 21,69 37,68 cho 1, 5, 15 phút trên 10 lõi), vì vậy chỉ nên đọc theo bậc độ lớn: compose nhanh hơn container `session` cỡ một bậc, `session` nhanh hơn `function` cỡ sáu đến bảy lần:

| Chế độ | `19 passed in ...` |
| --- | --- |
| compose có sẵn | 0,91 s; 0,88 s; 0,96 s |
| testcontainers, container `session` | 6,97 s; 6,89 s; 7,89 s |
| testcontainers, container `function` (`LAB28_SCOPE=function`) | 48,32 s; 39,07 s; 39,17 s |

Sau lần chạy, `\dn` trên PostgreSQL compose không còn schema `t_...`, `listDatabases` của MongoDB chỉ còn `admin`, `config`, `local`, và `docker ps -a` không còn container nào của test, kể cả Ryuk.

Để chắc test không xanh một cách hão, ta cố tình làm hỏng code. Bỏ điều kiện `AND status IN (...)` khỏi chỉ mục unique (biến nó thành «một tài xế chỉ một chuyến, mãi mãi»), chạy lại trên compose (trích):

```text
FAILED tests/test_trips_postgres.py::test_driver_is_free_again_after_trip_completes
1 failed, 18 passed in 0.91s
```

Đúng một test case đỏ, đúng test case kiểm «tài xế rảnh lại sau khi hoàn thành chuyến». Test case `test_driver_cannot_hold_two_active_trips` vẫn xanh vì chỉ mục hỏng chặn nhiều hơn mức cần, không phải ít hơn: hai phía của quy tắc cần hai test.

## CI và những giới hạn

File `github-actions-example.yml` trong lab là ví dụ minh hoạ, **chưa chạy** trên GitHub trong khuôn khổ bài này. Cấu hình của nó đơn giản vì testcontainers tự dựng hạ tầng: một job `pytest` trên runner có Docker daemon, cài uv và Python 3.14, rồi chạy lệnh pytest như trên (thêm `--locked` để `uv.lock` lệch thì job đỏ). File đã có `permissions: contents: read` và `timeout-minutes`, và phiên bản action kiểm tra lần cuối ngày 06/11/2026. Cách thứ hai là *service container* của CI (khai báo `pgvector/pgvector:pg17` trong workflow, đặt `TOINOI_PG_URL`), nhanh hơn nhưng gắn với một nền tảng CI; fixture của lab hỗ trợ cả hai nhờ biến môi trường. [Bài 75](/articles/python-ci-cd-github-actions-pytest-ruff-mypy) sẽ làm kỹ phần CI/CD.

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| pytest treo ở setup của ca đầu, không có container mới nào xuất hiện | Testcontainers 4.15 cần ảnh `testcontainers/ryuk:0.8.1`. Thử `docker pull testcontainers/ryuk:0.8.1`; nếu lệnh kẹt thì kiểm tra credential helper của Docker. Cách xử lý tạm: đặt `RYUK_CONTAINER_IMAGE=testcontainers/ryuk:<bản đã có>` (`docker images \| grep ryuk`) |
| Sau khi giết tiến trình test (`kill -9`), `docker ps` còn container | Ryuk thường dọn sau khoảng 10 giây nhưng không bảo đảm. Liệt kê bằng `docker ps -a --filter label=org.testcontainers=true`, rồi `docker rm -f` đúng container của phiên đã chết; đừng xoá container của phiên khác đang chạy |
| Không kết nối được tới Docker ngay khi chạy | Docker Desktop/daemon đã chạy chưa; `docker ps` có trả lời không |
| `datetime` đọc từ MongoDB không có `tzinfo` | Tạo `MongoClient(..., tz_aware=True)` |
| Chạy trên compose, lần sau còn schema `t_...` | Test case bị giết giữa chừng trước khi `finally` chạy; `DROP SCHEMA ... CASCADE` thủ công |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>19 test case chỉ chạm các câu lệnh ta chọn, không chứng minh mọi truy vấn của dịch vụ đúng. Một container đơn lẻ không có replica, failover, kết nối bị ngắt hay độ trễ mạng, nên không thay thế được kiểm thử chịu lỗi. Schema được dựng từ chuỗi <code>DDL</code> trong code; production nên dựng bằng chính migration (Alembic, bài 24) để test kiểm luôn migration. Test song song (như <code>pytest-xdist</code>) không được thử ở đây: schema riêng cho mỗi test case là điều kiện cần, chưa phải đủ. Luồng trạng thái trong lab là bản rút gọn của 8 trạng thái chuyến. Các số thời gian đo trên một máy cụ thể, và ở lần chạy sau image đã nằm trong cache của Docker. Tên hex của schema (<code>t_3f9a1c</code> trong hình) chỉ là minh hoạ.</p>
</div>

**Bài tập:** (1) Viết một test case chứng minh rằng khi chỉ còn **một** chuyến `searching` và đã có một tài xế đang giữ khoá chuyến đó, tài xế thứ hai nhận về `None` (không chờ, không nhận trùng). (2) Vì sao fixture `connect` đóng kết nối trong phần sau `yield` mà không để `DROP SCHEMA` lo?

<details><summary>Xem lời giải</summary><p>(1) Dựng một chuyến, mở giao dịch ở kết nối `a` và nhận chuyến đó, rồi gọi kết nối `b` khi giao dịch của `a` còn mở:</p>

```python
def test_second_driver_gets_none_when_only_one_trip_is_searching(connect) -> None:
    a, b = connect(), connect()
    TripRepository(a).create("TN-0001", "CU-001", "ride", D("3"), D("16500"))
    TripRepository(a).start_searching("TN-0001")

    with a.transaction():
        first = TripRepository(a).claim_next_searching("DR-001")
        with b.transaction():
            second = TripRepository(b).claim_next_searching("DR-002")

    assert (first, second) == ("TN-0001", None)
```

<p>Test case này đã chạy: <code>1 passed</code>. (2) <code>DROP SCHEMA ... CASCADE</code> chạy trên một kết nối quản trị khác và cần khoá loại ACCESS EXCLUSIVE trên các bảng của schema; nếu một kết nối của test case còn giữ giao dịch dở, lệnh xoá phải chờ nó. Pytest dọn fixture theo thứ tự ngược với lúc dựng: <code>connect</code> phụ thuộc <code>pg_schema</code> nên được dọn trước, kết nối đóng xong rồi schema mới bị xoá. Nếu không đóng, mỗi test case còn rò thêm kết nối tới server dùng chung.</p></details>

Bài trước: [bài 27](/articles/python-redis-cache-rate-limit-geo). Bài tiếp: [bài 29](/articles/python-polars-pandas-parquet-phan-tich-chuyen).

## Tài liệu gốc

- [testcontainers-python: documentation](https://testcontainers-python.readthedocs.io/en/latest/).
- [pytest: How to use fixtures (scope, dynamic scope, teardown)](https://docs.pytest.org/en/stable/how-to/fixtures.html).
- [PostgreSQL 17: SELECT, mục The Locking Clause (FOR UPDATE, SKIP LOCKED)](https://www.postgresql.org/docs/17/sql-select.html).
- [PostgreSQL 17: Partial Indexes](https://www.postgresql.org/docs/17/indexes-partial.html).
- [SQLite: Datatypes In SQLite](https://www.sqlite.org/datatype3.html).
- [SQLite: STRICT Tables](https://www.sqlite.org/stricttables.html).
- [PyMongo: Dates and Times](https://www.mongodb.com/docs/languages/python/pymongo-driver/current/data-formats/dates-and-times/).
