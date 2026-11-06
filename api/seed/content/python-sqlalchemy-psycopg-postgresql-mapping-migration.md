---
slug: python-sqlalchemy-psycopg-postgresql-mapping-migration
title: "SQLAlchemy 2 và psycopg 3 với PostgreSQL: mapping, migration Alembic"
summary: "Bài 24: SQLAlchemy 2 và psycopg 3 với PostgreSQL: mapping Mapped[], Session, select() và SQL sinh ra, migration Alembic có autogenerate và check, async engine. Có lab chạy thật trên schema riêng."
category: python
level: intermediate
tags: [python, toi-noi, sqlalchemy, postgresql, alembic]
language: vi
---

[Tới Nơi](/articles/du-an-mau-toi-noi) giao cho Python những service có dữ liệu riêng: chấm điểm gian lận, phân tích, OCR giấy tờ. Nhưng đến bài 23 mọi thứ ta viết vẫn sống trong bộ nhớ, tắt tiến trình là mất hết chuyến. Muốn một service Python giữ được chuyến, ta cần PostgreSQL, cùng loại database mà các bài C# đã dùng, và một cách nói chuyện với nó sao cho không phải viết SQL tay cho từng thao tác, đồng thời hình dạng bảng không trôi khỏi code theo thời gian.

Bài này dựng đúng phần đó bằng **SQLAlchemy 2** (ánh xạ và truy vấn), **psycopg 3** (driver) và **Alembic** (migration), chạy trên PostgreSQL dùng chung của loạt bài. Bài 25 sẽ dùng nền này để đặt chuyến an toàn khi nhiều người ghi cùng lúc.

Đây là bài 24 của lộ trình **Python trung bình qua Tới Nơi**. Bạn cần quen cấu trúc project và `pyproject.toml` ([bài 20](/articles/python-dong-goi-ruff-pre-commit-cau-truc-du-an)). Loạt C# có bài cùng chủ đề là [EF Core và PostgreSQL](/articles/dotnet10-efcore-postgresql-mapping-migration), cùng bài toán nhưng công cụ khác.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Bảng <code>trips</code> đã tạo. Bạn thêm một cột <code>rating</code> vào class <code>Trip</code> rồi chạy lại <code>Base.metadata.create_all(engine)</code>. Cột mới có xuất hiện trong bảng không?</p>
<details><summary>Xem đáp án</summary><p><b>Không.</b> <code>create_all</code> chỉ tạo những bảng chưa tồn tại và bỏ qua bảng đã có, nó không bao giờ <code>ALTER</code> bảng. Bài sẽ chạy đúng thí nghiệm này, rồi cho thấy Alembic giải quyết việc đó thế nào.</p></details>
</div>

## Ba lớp giữa class Trip và bảng trips

Khi người ta nói «dùng SQLAlchemy với PostgreSQL», thực ra có ba lớp xếp chồng, và biết lớp nào làm việc gì giúp đọc lỗi nhanh hơn.

- **psycopg 3** là driver: lớp duy nhất nói giao thức PostgreSQL, gửi câu SQL đi và nhận hàng về.
- **SQLAlchemy Core** gồm `Engine` (giữ connection pool) và bộ xây câu SQL từ biểu thức Python. Nó cho ta tạo `select(...)` thay vì nối chuỗi.
- **SQLAlchemy ORM** ánh xạ class sang bảng, và đưa `Session` vào: một đơn vị làm việc theo dõi object nào đã đổi để ghi xuống database.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 200" role="img" aria-label="Năm lớp từ trái sang phải: mô hình ORM, Session, Engine và pool, psycopg 3, PostgreSQL. Ba lớp đầu nằm trong SQLAlchemy."><defs><marker id="py24-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="8" y="26" width="534" height="156" rx="12" class="f-none s-pk dash"/><text x="24" y="48" class="t-b t-pk">SQLAlchemy (ORM và Core)</text><rect x="20" y="62" width="144" height="100" rx="10" class="f-pk-soft s-pk"/><text x="34" y="92" class="t-b">Mô hình ORM</text><text x="34" y="116" class="t-small">class Trip</text><text x="34" y="134" class="t-small">Mapped[...]</text><path d="M164 112 L194 112" class="f-none s-phys" stroke-width="2" marker-end="url(#py24-0-arrow)"/><rect x="198" y="62" width="144" height="100" rx="10" class="f-pk-soft s-pk"/><text x="212" y="92" class="t-b">Session</text><text x="212" y="116" class="t-small">Unit of Work</text><text x="212" y="134" class="t-small">identity map</text><path d="M342 112 L372 112" class="f-none s-phys" stroke-width="2" marker-end="url(#py24-0-arrow)"/><rect x="376" y="62" width="144" height="100" rx="10" class="f-pk-soft s-pk"/><text x="390" y="92" class="t-b">Engine + pool</text><text x="390" y="116" class="t-small">create_engine()</text><text x="390" y="134" class="t-small">kết nối tái dùng</text><path d="M520 112 L548 112" class="f-none s-phys" stroke-width="2" marker-end="url(#py24-0-arrow)"/><rect x="552" y="62" width="144" height="100" rx="10" class="f-phys-soft s-phys"/><text x="566" y="92" class="t-b">psycopg 3</text><text x="566" y="116" class="t-small">postgresql+psycopg</text><text x="566" y="134" class="t-small">giao thức PostgreSQL</text><path d="M696 112 L722 112" class="f-none s-phys" stroke-width="2" marker-end="url(#py24-0-arrow)"/><rect x="726" y="62" width="134" height="100" rx="10" class="f-phys-soft s-phys"/><text x="740" y="92" class="t-b">PostgreSQL 17</text><text x="740" y="116" class="t-small">schema lab24</text><text x="740" y="134" class="t-small">bảng, ràng buộc</text></svg>
</div>
<figcaption>Hình 1. Một lệnh đọc đi qua năm lớp. Ba lớp trong khung nét đứt là SQLAlchemy (ánh xạ, Session, Engine); psycopg 3 và PostgreSQL ở ngoài, nên lỗi kết nối hay lỗi ràng buộc nổi lên từ hai lớp đó.</figcaption>
</figure>

URL kết nối của lab lấy từ biến `TOINOI_PG_URL` và đổi tiền tố sang `postgresql+psycopg://`. Phần sau dấu `+` chọn driver. Ta ghi rõ thay vì dựa vào mặc định, vì mặc định đã đổi giữa các phiên bản: SQLAlchemy 2.0 mặc định là psycopg2, còn từ 2.1 URL `postgresql://` trần đã trỏ sang psycopg 3 (lab chạy trên 2.1.8, kiểm bằng `make_url(...).get_dialect()`). Dependency của lab (`pyproject.toml`): `sqlalchemy[asyncio]>=2.1`, `psycopg[binary]`, `alembic`; ta ghim từ 2.1 vì các kết quả trong bài đo trên 2.1.8. Gói `[binary]` là bản dựng sẵn; wheel cho một phiên bản Python mới có thể chưa kịp có, nhưng với 3.14.7 trên macOS ARM64 lab cài được.

## Mapping ORM 2.0: Mapped, mapped_column và schema riêng

Phong cách ORM 2.0 dùng **type annotation** để khai báo cột. `models.py` (trích):

```python
class Base(DeclarativeBase):
    # Mọi bảng nằm trong schema lab24 để lab dọn được bằng DROP SCHEMA ... CASCADE.
    metadata = MetaData(schema=SCHEMA)
    # Cho phép viết Mapped[TripStatus] và Mapped[ServiceKind] thẳng trên annotation.
    type_annotation_map = {
        TripStatus: _enum_column(TripStatus),
        ServiceKind: _enum_column(ServiceKind),
    }


class Trip(Base):
    __tablename__ = "trips"
    __table_args__ = (Index("ix_trips_customer_created", "customer_id", "created_at"),)

    trip_id: Mapped[str] = mapped_column(String(16), primary_key=True)  # TN-0001
    customer_id: Mapped[str] = mapped_column(ForeignKey(f"{SCHEMA}.customers.customer_id"))
    driver_id: Mapped[str | None] = mapped_column(String(16))  # NULL tới khi có người nhận
    service: Mapped[ServiceKind]
    distance_km: Mapped[Decimal] = mapped_column(Numeric(7, 3))
    fare: Mapped[Decimal] = mapped_column(Numeric(12, 0))  # VND, không có phần lẻ
    status: Mapped[TripStatus] = mapped_column(default=TripStatus.REQUESTED)
    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), server_default=func.now()
    )
```

Một vài quyết định đáng nhìn kỹ, vì chúng là chỗ mapping thường sai âm thầm:

- **`Mapped[str | None]` là cột cho phép NULL**, còn `Mapped[str]` là `NOT NULL`. Annotation quyết định ràng buộc, không chỉ gợi ý cho trình soát kiểu. `driver_id` nullable vì chuyến mới chưa có tài xế.
- **Tiền và khoảng cách dùng `Numeric`, tương ứng `Decimal`**, không dùng `Float` (bài 02). Lab đọc lại `fare` ra `Decimal('14250')` và `distance_km` ra `Decimal('2.500')`: scale của cột được giữ.
- **`StrEnum` lưu thành VARCHAR chứa giá trị** (`"picked_up"`), không lưu tên thành viên (`"PICKED_UP"`). `values_callable` làm việc này, và `native_enum=False` tránh kiểu `ENUM` của PostgreSQL vốn khó sửa bằng migration khi thêm trạng thái. Đổi lại cột chỉ là VARCHAR(16) **không có ràng buộc CHECK**: ORM chặn giá trị lạ ở phía Python, nhưng một câu `INSERT` ghi thẳng `status = 'bogus'` vẫn lọt (lab thử và PostgreSQL nhận). Muốn database cũng chặn, ta thêm `CheckConstraint` riêng.
- **`DateTime(timezone=True)` là `timestamptz`**: PostgreSQL lưu một thời điểm tuyệt đối và trả về `datetime` có offset. Ta luôn ghi giờ UTC aware (bài 04); khi hiển thị đổi sang `Asia/Ho_Chi_Minh`.
- **Schema `lab24` đặt ở `MetaData`**, nên không class nào phải lặp lại. Khóa ngoại ta ghi đủ tên có schema (`lab24.customers.customer_id`).

`Trip` ở đây là **mô hình lưu trữ**, không phải `Trip` dataclass có hành vi và máy trạng thái của [bài 11](/articles/python-class-dataclass-property-encapsulation). Gộp hai thứ làm một là một lựa chọn kiến trúc; [bài 69](/articles/python-repository-unit-of-work-sqlalchemy) sẽ bàn việc tách repository khỏi domain.

## Session, select() và SQL sinh ra

`Session` là cửa vào của ORM. Ta ghi nhiều object trong một transaction rồi commit một lần. `main.py` (trích):

```python
with Session(engine) as session, session.begin():
    seed(session)  # session.add_all(customers); session.add_all(trips)
```

`session.begin()` mở transaction và commit khi thoát khối `with` thành công, rollback khi có exception; bài 25 đi sâu hơn. Truy vấn dùng `select()` kiểu 2.0, đọc như SQL:

```python
stmt = (
    select(Trip)
    .where(Trip.status == TripStatus.REQUESTED, Trip.customer_id == "CU-001")
    .order_by(Trip.created_at.desc())
)
print(str(stmt.compile(dialect=pg_psycopg.dialect())))
```

Nhìn SQL sinh ra là kỹ năng bắt buộc khi dùng ORM. Có hai cách. `stmt.compile(dialect=...)` (lab dùng `psycopg.dialect()` từ `sqlalchemy.dialects.postgresql`) cho chuỗi SQL không cần kết nối; `create_engine(url, echo=True)` in từng câu thật mỗi lần chạy (đó chỉ là cách bật logger `sqlalchemy.engine` ở mức INFO, lab bật logger đó nhưng gom lại để đếm). Output của lab (trích, đã bỏ khoảng trắng cuối dòng):

```text
SELECT lab24.trips.trip_id, lab24.trips.customer_id, lab24.trips.driver_id, lab24.trips.service, lab24.trips.distance_km, lab24.trips.fare, lab24.trips.status, lab24.trips.created_at
FROM lab24.trips
WHERE lab24.trips.status = %(status_1)s AND lab24.trips.customer_id = %(customer_id_1)s::VARCHAR ORDER BY lab24.trips.created_at DESC
Tham số gửi kèm: {'status_1': 'requested', 'customer_id_1': 'CU-001'}
```

Giá trị không bao giờ nằm trong chuỗi SQL mà đi riêng dưới dạng tham số (`%(status_1)s`), nên người dùng gõ `"'; DROP TABLE trips; --"` cũng chỉ là một chuỗi dữ liệu. Phần `::VARCHAR` sau tham số là do dialect psycopg trong SQLAlchemy 2.1.8 thêm vào để PostgreSQL biết kiểu của tham số được gửi tách khỏi câu lệnh.

Hai hành vi của `Session` mà người mới hay vấp:

- **Identity map.** Trong một session, cùng khóa chính là cùng một object. Gọi `session.get(Trip, "TN-0001")` lần hai trả về chính object cũ và không phát thêm câu SQL nào (lab đếm: 0 câu).
- **Unit of Work.** Đổi `first.status = TripStatus.SEARCHING` chưa chạm database. Đến lúc `flush` (tự động trước truy vấn kế tiếp hoặc lúc commit), ORM so với trạng thái ban đầu và phát đúng một `UPDATE lab24.trips SET status=%(status)s WHERE lab24.trips.trip_id = ...`, chỉ có cột đã đổi.

Cuối cùng là bẫy kinh điển **N+1**. Quan hệ `Customer.trips` mặc định tải lười: chạm `.trips` mới phát SQL. Với 3 khách, vòng lặp đọc `.trips` từng người tốn `1 + 3 = 4` câu; thêm `.options(selectinload(Customer.trips))` còn 2 câu (một lấy khách, một lấy mọi chuyến của họ). `selectinload` không phải một câu duy nhất: nó chia khóa thành lô tối đa 500 giá trị mỗi câu `IN`. Lab thêm 1.498 khách để có 1.501 và đếm được `1 + 4 = 5` câu (một lấy khách, bốn lô). Với 25.000 tài xế trực tuyến, đó là khoảng 51 câu thay vì 25.001 vòng khứ hồi của tải lười.

## Alembic: migration là lịch sử lược đồ có thể review

`create_all` chỉ tạo bảng chưa có, nên không đủ khi lược đồ thay đổi. **Alembic** lưu mỗi thay đổi thành một file **revision** nằm trong git, có `upgrade()` và `downgrade()`, và ghi revision hiện tại vào bảng `alembic_version` của database. Cách làm việc là: sửa `models.py`, để Alembic **so** models với database thật và đề xuất migration, đọc kỹ, sửa tay nếu cần, rồi áp dụng.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Vòng đời migration: models.py được Alembic so với database để sinh file revision, người review sửa, upgrade head áp dụng vào PostgreSQL, alembic check so lại models với database để báo lệch."><defs><marker id="py24-1-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="50" width="210" height="100" rx="10" class="f-pk-soft s-pk"/><text x="36" y="82" class="t-b">models.py</text><text x="36" y="106" class="t-small">Base.metadata</text><text x="36" y="124" class="t-small">Mapped[] của Trip</text><path d="M230 100 L336 100" class="f-none s-phys" stroke-width="2" marker-end="url(#py24-1-arrow)"/><text x="244" y="90" class="t-small">autogenerate</text><rect x="340" y="50" width="210" height="100" rx="10" class="f-warn-soft s-warn"/><text x="356" y="82" class="t-b">Revision 0002</text><text x="356" y="106" class="t-small">migrations/versions/*.py</text><text x="356" y="124" class="t-small">người review sửa tay</text><path d="M550 100 L656 100" class="f-none s-phys" stroke-width="2" marker-end="url(#py24-1-arrow)"/><text x="564" y="90" class="t-small">upgrade head</text><rect x="660" y="50" width="200" height="100" rx="10" class="f-phys-soft s-phys"/><text x="676" y="82" class="t-b">PostgreSQL</text><text x="676" y="106" class="t-small">bảng lab24.*</text><text x="676" y="124" class="t-small">alembic_version = 0002</text><path d="M760 150 L760 196 L125 196 L125 154" class="f-none s-muted dash" stroke-width="1.5" marker-end="url(#py24-1-arrow)"/><text x="260" y="222" class="t-small">alembic check: so lại models.py với database thật, báo nếu còn lệch</text></svg>
</div>
<figcaption>Hình 2. Migration đi một chiều từ models tới database, còn nhánh nét đứt là vòng kiểm tra ngược để biết mapping và lược đồ thật còn khớp nhau không.</figcaption>
</figure>

Dựng khung bằng `alembic init migrations` cho ra `alembic.ini`, thư mục `migrations/` với `env.py`, `script.py.mako` và `versions/`. Lab giữ phần lớn mặc định và chỉnh vài chỗ có chủ ý. Trong `alembic.ini`, `script_location = %(here)s/migrations` và `prepend_sys_path = %(here)s` dùng `%(here)s` (thư mục chứa file ini) nên lệnh chạy được từ gốc repository, không phụ thuộc thư mục làm việc. URL **không** nằm trong ini mà do `env.py` lấy từ `settings.py`, tức cùng nguồn `TOINOI_PG_URL` với ứng dụng. `migrations/env.py` (trích):

```python
target_metadata = Base.metadata


def include_name(name: str | None, type_: str, parent_names: dict) -> bool:
    """Chỉ xét schema lab24; bỏ qua mọi schema khác trong database dùng chung."""
    if type_ == "schema":
        return name == SCHEMA
    return True


def _configure(**kwargs) -> None:
    context.configure(
        target_metadata=target_metadata,
        include_schemas=True,
        include_name=include_name,
        version_table_schema=SCHEMA,  # bảng alembic_version cũng nằm trong lab24
        **kwargs,
    )
```

Ba tùy chọn cùng làm một việc: giữ Alembic trong đúng schema `lab24`. Database dùng chung có schema của các lab khác, và nếu thiếu `include_name`, autogenerate sẽ so cả chúng và đề xuất xóa những bảng không thuộc về ta. Revision đầu tiên được sinh bằng `alembic revision --autogenerate -m "create customers and trips" --rev-id 0001`; log cho thấy nó phát hiện đúng hai bảng mới:

```text
INFO  [alembic.autogenerate.compare.tables] Detected added table 'lab24.customers'
INFO  [alembic.autogenerate.compare.tables] Detected added table 'lab24.trips'
```

File sinh ra được đọc lại và gọn hóa, vì autogenerate chỉ là **bản nháp**. Nó không đoán được ý định (đổi tên cột trông giống xóa một cột rồi thêm cột khác), không thấy mọi thay đổi (ví dụ giá trị `server_default` mặc định không được so sánh), và không biết dữ liệu hiện có. Revision `0001` (trích):

```python
def upgrade() -> None:
    op.create_table(
        "customers",
        sa.Column("customer_id", sa.String(length=16), nullable=False),
        sa.Column("full_name", sa.String(length=100), nullable=False),
        sa.PrimaryKeyConstraint("customer_id"),
        schema="lab24",
    )
    op.create_table("trips", ...)  # trích: cột như models.py, khóa ngoại tới customers


def downgrade() -> None:
    op.drop_table("trips", schema="lab24")
    op.drop_table("customers", schema="lab24")
```

Sau đó ta thêm `Index(...)` vào `Trip`, chạy autogenerate lần nữa và được revision `0002` với `Detected added index 'ix_trips_customer_created'`. Để áp dụng migration, chạy từ gốc repository:

```bash
uv run --project examples/toi-noi-python/24-sqlalchemy-postgresql-alembic \
  alembic -c examples/toi-noi-python/24-sqlalchemy-postgresql-alembic/alembic.ini upgrade head
```

```text
INFO  [alembic.runtime.migration] Running upgrade  -> 0001, create customers and trips
INFO  [alembic.runtime.migration] Running upgrade 0001 -> 0002, index trips by customer and created_at
```

(trích: đã bỏ hai dòng `Context impl` và `Will assume transactional DDL`). `alembic current` in `0002 (head)`. Muốn bỏ hẳn schema khi thử xong, chạy `main.py --cleanup` (lệnh này `DROP SCHEMA lab24 CASCADE`). Alembic coi DDL của PostgreSQL là transactional, và theo mặc định cả lệnh `upgrade` là **một** transaction: nếu `0002` lỗi thì bảng của `0001` cũng bị rollback, không để lại nửa chừng (đặt `transaction_per_migration=True` trong `context.configure` nếu muốn mỗi revision một transaction). Thêm `--sql` vào `upgrade` sẽ in SQL của các revision mà không kết nối, để đưa cho người review database trước khi chạy thật. Lưu ý output đó không có `CREATE SCHEMA lab24`, vì chỉ chế độ online của `env.py` tạo schema: khi chạy kịch bản offline, schema phải có sẵn.

Lệnh quan trọng cuối cùng là `alembic check`: nó so `models.py` với database sau khi áp dụng và thoát với mã khác 0 nếu còn lệch, nên rất hợp để chạy trong CI. Lab gọi nó ở dạng `command.check(cfg)`. Khi ta thử thêm `rating: Mapped[int | None] = mapped_column(SmallInteger)` vào `Trip` mà quên viết migration, nó báo `New upgrade operations detected: [('add_column', 'lab24', 'trips', Column('rating', SmallInteger(), ...))]`, và `create_all` thì im lặng bỏ qua cột mới: chính xác điều câu hỏi đầu bài nói.

## Async engine: cùng mô hình, quy tắc khác

Service Python của Tới Nơi sẽ là FastAPI (bài 31), nên cần biết phiên bản bất đồng bộ. URL không đổi (`postgresql+psycopg://` dùng được cho cả hai kiểu, SQLAlchemy tự chọn biến thể async bên dưới), chỉ đổi cách tạo engine và session. `main.py` (trích):

```python
engine = create_async_engine(sqlalchemy_url())
async with AsyncSession(engine) as session:
    trips = (await session.scalars(select(Trip).order_by(Trip.trip_id))).all()
    customer = await session.scalar(select(Customer).where(Customer.customer_id == "CU-001"))
    customer.trips  # tải lười cần I/O ngầm, mà coroutine không được phép chặn
```

Có hai điều lab ghi lại. Thứ nhất, `sqlalchemy.ext.asyncio` cần thư viện `greenlet`; lần chạy đầu của ta thiếu nó và dừng với `ImportError: The SQLAlchemy asyncio module requires that the Python 'greenlet' library is installed`, nên dependency phải khai báo là `sqlalchemy[asyncio]`. Thứ hai, chạm `customer.trips` trong ngữ cảnh async ném `MissingGreenlet` (SQLAlchemy bọc nó trong `StatementError`, lỗi gốc nằm ở `.orig`). Tải lười tức là «phát SQL ở chỗ không có dấu hiệu gì trong code», mà trong coroutine mọi I/O phải có `await` (bài 21). Cách đúng là nói trước cho truy vấn tải luôn quan hệ: `.options(selectinload(Customer.trips))`; lab xác nhận cách này trả `TN-0001,TN-0002` cho `CU-001`.

Chọn async khi cả service đã async; nó không làm một câu SQL chạy nhanh hơn, chỉ cho luồng khác tiếp tục trong lúc chờ. Với script hay job tuần tự, engine đồng bộ đơn giản và ít bẫy hơn.

## Chạy và kiểm chứng

Cần PostgreSQL của hạ tầng dùng chung đã chạy (`docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait`). Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/24-sqlalchemy-postgresql-alembic examples/toi-noi-python/24-sqlalchemy-postgresql-alembic/main.py --verify
```

`main.py` bắt đầu bằng `DROP SCHEMA IF EXISTS lab24 CASCADE`, chạy migration bằng API của Alembic, thực hiện các bước ở trên, rồi dọn schema. Output cuối cùng (trích):

```text
alembic upgrade head -> revision hiện tại 0002, head của script 0002
Cột trips trong PostgreSQL: {'trip_id': 'VARCHAR(16)', ..., 'fare': 'NUMERIC(12, 0)', ..., 'created_at': 'TIMESTAMP WITH TIME ZONE'}
Đã thêm 3 khách và 4 chuyến bằng một transaction.
TN-0001: fare=Decimal('14250'), distance=Decimal('2.500'), status=<TripStatus.REQUESTED: 'requested'>, created_at=2026-11-06 10:00 +07
flush sau khi đổi status: UPDATE lab24.trips SET status=%(status)s WHERE lab24.trips.trip_id = %(lab24_trips_trip_id)s::VARCHAR
3 khách, đọc .trips của từng người: lazy = 4 câu SQL, selectinload = 2 câu
1501 khách với selectinload: 5 câu SQL (1 câu lấy khách + các lô tối đa 500 khóa)
INSERT thẳng status='bogus' vào VARCHAR: PostgreSQL nhận ('bogus'), vì không có CHECK
Chuyến của khách không tồn tại -> IntegrityError, SQLSTATE 23503
Async: 4 chuyến; tải lười trong async -> MissingGreenlet; selectinload -> TN-0001,TN-0002
downgrade -1 -> revision 0001, index còn: []
upgrade head -> revision 0002, index còn: ['ix_trips_customer_created']
Đã dọn: DROP SCHEMA lab24 CASCADE
OK 27/27
```

Vài điểm đáng đọc. Phần «Cột trips trong PostgreSQL» là kết quả hỏi **catalog của database** (`inspect(engine)`), không hỏi lại model: đây là cách chứng minh mapping và bảng thật khớp. Khóa ngoại sai (`CU-999`) không bị Python chặn mà bị PostgreSQL từ chối với SQLSTATE `23503`, và `exc.orig.sqlstate` cho ta mã đó; ràng buộc nằm ở database chứ không chỉ ở code. Cặp `downgrade -1` rồi `upgrade head` cho thấy migration đi được hai chiều và dữ liệu còn nguyên (lab đếm lại 4 chuyến). `--verify` chạy 27 kiểm tra; đạt thì in `OK 27/27`, sai thì in tên ca và `FAIL n/m` (n là số ca đạt) với exit 1. Nếu chưa bật PostgreSQL, nó in `FAIL: không nối được PostgreSQL tại <url> ...` và exit 1. Phiên bản đã chạy: Python 3.14.7, SQLAlchemy 2.1.8, psycopg 3.3.8, Alembic 1.20.1, PostgreSQL 17.11.

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| Thêm cột vào model nhưng bảng không có cột đó | Đã tạo và áp dụng migration chưa; `alembic check` báo gì |
| `autogenerate` đề xuất `drop_table` cho bảng không phải của bạn | Có `include_name` giới hạn schema chưa; database có schema của dự án khác không |
| `ImportError ... 'greenlet' library is installed` | Khai báo `sqlalchemy[asyncio]`, không chỉ `sqlalchemy` |
| `MissingGreenlet` khi đọc quan hệ trong `async def` | Tải quan hệ trước bằng `selectinload`/`joinedload` thay vì chạm thuộc tính sau |
| Một màn hình danh sách phát hàng trăm câu SQL giống nhau | Bật log SQL, đếm câu; N+1 do tải lười trong vòng lặp |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>Lab dựng và xóa schema <code>lab24</code> mỗi lần chạy, còn production không bao giờ <code>DROP SCHEMA</code>: migration chỉ đi tới, có kế hoạch rollback và được review SQL, đặc biệt với bảng <code>trips</code> hàng trăm triệu dòng, nơi một <code>ALTER TABLE</code> hay <code>CREATE INDEX</code> thông thường có thể khóa bảng (<code>CREATE INDEX CONCURRENTLY</code> không chạy được trong một transaction nên phải đặt trong <code>autocommit_block()</code> của Alembic). Ví dụ này cũng chưa tinh chỉnh pool kết nối cho tải 70 chuyến mỗi giây, chưa quản lý quyền (tài khoản <code>toinoi</code> chỉ dùng cho lab), và để migration chạy một lần trước khi các replica khởi động chứ không để mỗi replica tự chạy lúc start. Dữ liệu khách và chuyến là dữ liệu giả.</p>
</div>

**Bài tập:** (1) Thêm cột `rating` (`SmallInteger`, cho phép NULL) vào `Trip`, sinh revision `0003` bằng autogenerate và cho biết `downgrade()` phải làm gì. (2) Nếu thay vào đó muốn `rating` là `NOT NULL` trên bảng đã có 200.000 chuyến, migration an toàn gồm những bước nào?

<details><summary>Xem lời giải</summary><p>(1) Autogenerate sinh <code>op.add_column('trips', sa.Column('rating', sa.SmallInteger(), nullable=True), schema='lab24')</code> cho <code>upgrade()</code>, và <code>op.drop_column('trips', 'rating', schema='lab24')</code> cho <code>downgrade()</code>: ta đã chạy thí nghiệm này trong bản sao của lab. Cần lưu ý downgrade xóa cột là mất dữ liệu cột đó. (2) Thêm cột <code>NOT NULL</code> ngay sẽ lỗi vì 200.000 dòng cũ chưa có giá trị. Cách an toàn: thêm cột cho phép NULL; điền giá trị cho dòng cũ (<code>UPDATE</code>, với bảng lớn thì theo lô); khi đã không còn NULL mới đặt <code>NOT NULL</code> (<code>op.alter_column(..., nullable=False)</code>), hoặc đặt <code>server_default</code> nếu có một giá trị mặc định đúng nghiệp vụ. Mỗi bước là một thay đổi nhỏ có thể review.</p></details>

Bài trước: [bài 23](/articles/python-concurrency-lock-queue-semaphore). Bài tiếp: [bài 25](/articles/python-transaction-optimistic-concurrency-idempotency).

## Tài liệu gốc

- [SQLAlchemy 2.1: ORM Quick Start](https://docs.sqlalchemy.org/en/21/orm/quickstart.html).
- [SQLAlchemy 2.1: Transactions and Connection Management (Session)](https://docs.sqlalchemy.org/en/21/orm/session_transaction.html).
- [SQLAlchemy 2.1: Asynchronous I/O (asyncio)](https://docs.sqlalchemy.org/en/21/orm/extensions/asyncio.html).
- [SQLAlchemy 2.1: Relationship Loading Techniques](https://docs.sqlalchemy.org/en/21/orm/loading_relationships.html).
- [Alembic: Auto Generating Migrations](https://alembic.sqlalchemy.org/en/latest/autogenerate.html).
- [psycopg 3: Installation](https://www.psycopg.org/psycopg3/docs/basic/install.html).
