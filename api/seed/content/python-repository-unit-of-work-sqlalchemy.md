---
slug: python-repository-unit-of-work-sqlalchemy
title: "Repository và Unit of Work với SQLAlchemy"
summary: "Bài 69: repository theo aggregate KycApplication, Unit of Work bao một Session cho mỗi command, optimistic concurrency và bẫy identity map giữ tham chiếu yếu, kiểm bằng PostgreSQL thật. Có lab chạy được."
category: python-advanced
level: advanced
tags: [python, toi-noi, sqlalchemy, repository, unit-of-work]
language: vi
---

Ở bài 68, hồ sơ KYC của [Tới Nơi](/articles/du-an-mau-toi-noi) sống trong một `dict`, và «transaction» chỉ là chụp ảnh `dict` rồi chép lại khi lỗi. Đủ để chứng minh thứ tự log, validate, transaction. Nhưng hồ sơ phải sống qua lần khởi động lại của service, và có lúc hai người duyệt cùng nạp một hồ sơ `needs_review`. Cần phân biệt hai tình huống: nếu request thứ hai đến *sau khi* request thứ nhất commit, nó nạp bản mới và domain từ chối; chỉ khi hai transaction *chồng nhau* mới cần cơ sở dữ liệu phân xử, và bộ nhớ không trả lời được «ai thắng, ai được báo».

Bài này đặt PostgreSQL thật vào chỗ của `dict`. **Repository** là cổng để nạp và lưu cả một aggregate; **Unit of Work** là ranh giới transaction mà mọi kho trong một command cùng dùng. Cả hai được kiểm trên PostgreSQL 17 thật, và một bẫy của identity map trong SQLAlchemy sẽ hiện ra khi test hai transaction chồng nhau chạy lần đầu.

Đây là bài 69 của lộ trình **Python nâng cao qua Tới Nơi**. Bạn cần đã đọc [bài 68](/articles/python-cqrs-command-query-handler-mediator), SQLAlchemy ở [bài 24](/articles/python-sqlalchemy-psycopg-postgresql-mapping-migration) và optimistic concurrency ở [bài 25](/articles/python-transaction-optimistic-concurrency-idempotency). Bài C# cùng chủ đề là [Repository và Unit of Work với EF Core](/articles/dotnet10-repository-unit-of-work).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Hồ sơ <code>KYC-0001</code> đang <code>needs_review</code>. RV-001 và RV-002 cùng nạp nó trong hai transaction đang mở. RV-001 duyệt và commit; RV-002, vẫn cầm bản đã nạp, ghi quyết định từ chối. Domain đã kiểm «chỉ <code>needs_review</code> mới được duyệt», và repository chỉ chạy <code>UPDATE ... WHERE application_id = ?</code>. Trạng thái cuối cùng là gì, và có ai nhận lỗi không?</p>
<details><summary>Xem đáp án</summary><p><b>Trạng thái cuối là <code>rejected</code> bởi RV-002, và không ai nhận lỗi.</b> Domain kiểm trên bản RV-002 đã nạp, bản đó vẫn ở <code>needs_review</code>; cơ sở dữ liệu không có gì để từ chối câu <code>UPDATE</code>. Quyết định của RV-001 biến mất (lost update). Chữa bằng một cột <code>version</code> đi kèm điều kiện ghi; ở bài này RV-002 nhận <code>ConcurrencyConflict</code> và RV-001 giữ <code>approved</code>. Nếu RV-002 gửi request <i>sau</i> khi RV-001 đã commit thì không có xung đột để bắt: request nạp bản mới và domain trả 409 <code>invalid-status-transition</code>.</p></details>
</div>

## Repository là cổng của aggregate, không phải bảng

Từ bài 67, application đã có một port `KycRepository`. Nó chỉ có năm thao tác, đúng những gì use case cần:

```python
# src/kyc_service/application/ports.py (trích)
class KycRepository(Protocol):
    """Lưu và nạp aggregate KycApplication."""

    def next_id(self) -> str: ...

    def add(self, application: KycApplication) -> None: ...

    def get(self, application_id: str) -> KycApplication | None: ...

    def save(self, application: KycApplication) -> None: ...

    def list_by_status(self, status: KycStatus) -> list[KycApplication]: ...
```

Không có `find(**filters)`, không có `query()` trả về `Session`, không có lớp `GenericRepository[T]` với `get_all`, `filter`, `delete`. Một repository tổng quát lặp lại bề mặt của ORM và để mọi lớp trên tự viết truy vấn, tức là lớp application phụ thuộc lại vào chi tiết lưu trữ qua cửa sau. Mỗi thao tác thêm vào là một quyết định có chủ ý: `list_by_status` có mặt vì handler cần, còn «liệt kê hồ sơ để hiển thị» là việc của phía đọc `ApplicationReadModel`.

Đơn vị lưu là **aggregate**: một hồ sơ gồm hàng gốc và các hàng con, nạp và ghi cùng nhau. `KycApplication` trong bộ nhớ là một dataclass; dưới PostgreSQL nó là ba bảng.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Aggregate KycApplication ở bên trái ánh xạ sang ba bảng ở bên phải. Mũi tên apply_to_row đi từ aggregate sang bảng, mũi tên to_domain đi ngược lại. Ba bảng là kyc_applications với version, kyc_documents hai hàng, kyc_ocr_results tối đa hai hàng."><defs><marker id="py69-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="40" width="250" height="210" rx="10" class="f-pk-soft s-pk"/><text x="36" y="68" class="t-b">Aggregate KycApplication</text><text x="36" y="100" class="t-small">application_id, driver_id, status</text><text x="36" y="124" class="t-small">documents: đúng hai giấy tờ</text><text x="36" y="148" class="t-small">ocr_results, face_match</text><text x="36" y="172" class="t-small">risk_score, note</text><text x="36" y="196" class="t-small">review (người duyệt)</text><text x="36" y="226" class="t-small">dataclass thuần, không biết ORM</text><path d="M270 120 L378 120" class="f-none s-phys" stroke-width="2" marker-end="url(#py69-0-arrow)"/><text x="282" y="110" class="t-mono t-small">apply_to_row</text><path d="M378 170 L272 170" class="f-none s-muted dash" stroke-width="2" marker-end="url(#py69-0-arrow)"/><text x="292" y="190" class="t-mono t-small">to_domain</text><path d="M384 71 L384 221" class="f-none s-phys" stroke-width="2"/><rect x="394" y="40" width="466" height="62" rx="8" class="f-phys-soft s-phys"/><text x="410" y="64" class="t-b">kyc_applications</text><text x="410" y="86" class="t-small">một hàng gốc: trạng thái, điểm, người duyệt, version</text><rect x="394" y="114" width="466" height="62" rx="8" class="f-phys-soft s-phys"/><text x="410" y="138" class="t-b">kyc_documents</text><text x="410" y="160" class="t-small">hai hàng: cccd và gplx, khóa chính (hồ sơ, loại)</text><rect x="394" y="188" width="466" height="62" rx="8" class="f-phys-soft s-phys"/><text x="410" y="212" class="t-b">kyc_ocr_results</text><text x="410" y="234" class="t-small">tối đa hai hàng; số giấy tờ lưu dạng đã che 099*******83</text><text x="20" y="282" class="t-small">Session và hàng ORM chỉ xuất hiện trong infrastructure; domain chỉ thấy dataclass.</text></svg>
</div>
<figcaption>Hình 1. Một aggregate, ba bảng: repository dịch hai chiều, và không rò hàng ORM ra ngoài.</figcaption>
</figure>

## Ánh xạ hai chiều: dataclass ở trong, hàng ORM ở ngoài

Các bảng khai báo bằng ORM kiểu SQLAlchemy 2 (`Mapped`, `mapped_column`). Hai điểm đáng đọc kỹ là cột `version` và hàng con:

```python
# src/kyc_service/infrastructure/sql/tables.py (trích)
    # Phiên bản cho optimistic concurrency: mỗi UPDATE thêm 1 và kèm `WHERE version = <đã nạp>`.
    version: Mapped[int] = mapped_column()

    __mapper_args__ = {"version_id_col": version}  # noqa: RUF012 - cách khai báo của SQLAlchemy

    # lazy="raise": chạm vào hàng con mà quên nạp (selectinload) thì lỗi ngay, không âm thầm
    # phát N câu SELECT. Mọi truy vấn phải nói rõ nó cần hàng con nào.
    documents: Mapped[list[DocumentRow]] = relationship(
        cascade="all, delete-orphan", lazy="raise", order_by="DocumentRow.kind"
    )
    ocr_results: Mapped[list[OcrRow]] = relationship(
        cascade="all, delete-orphan", lazy="raise", order_by="OcrRow.kind"
    )
```

`lazy="raise"` nghĩa là chạm vào `row.ocr_results` mà chưa nạp sẽ nổ ngay thay vì âm thầm phát thêm một câu `SELECT`: mọi truy vấn phải nói rõ nó cần hàng con nào bằng `selectinload`, nên không có N+1 mà không hay.

Việc dịch nằm trong hai hàm: `to_domain` dựng lại aggregate qua constructor (nên các bất biến của value object vẫn được kiểm), `apply_to_row` chép trạng thái aggregate lên hàng:

```python
# src/kyc_service/infrastructure/sql/repository.py (trích)
def protect(fields: Mapping[str, str]) -> dict[str, Any]:
    """Chốt thứ hai: `OcrResult` đã che số; đây chặn mọi đường khác ghi số rõ xuống bảng."""
    return {k: mask_id_number(v) if k in PII_FIELDS else v for k, v in fields.items()}
```

```python
# src/kyc_service/infrastructure/sql/repository.py (trích)
def apply_to_row(row: ApplicationRow, app: KycApplication) -> None:
    """Chép trạng thái aggregate lên hàng. SQLAlchemy chỉ phát UPDATE cho cột thật sự đổi."""
    row.driver_id = app.driver_id
    row.status = app.status.value
    row.selfie_ref = app.selfie_ref
    row.face_consent = app.face_consent
    row.submitted_at = app.submitted_at
    row.note = app.note
    row.face_score = app.face_match.score if app.face_match else None
    row.risk_value = app.risk_score.value if app.risk_score else None
    row.risk_reasons = list(app.risk_score.reasons) if app.risk_score else []
```

Hai quyết định ở đây cần nói thẳng. Thứ nhất là **số giấy tờ đầy đủ không bao giờ vào aggregate hay bảng**: `OcrResult` che `id_number` và `license_number` ngay khi được tạo, nên adapter rủi ro thấy `099*******83` cả lúc nộp lẫn lúc làm tiếp (bản đầu của lab che ở repository nên hai lúc ấy khác nhau). `protect` ở repository chỉ còn là chốt thứ hai. Hàm `mask_id_number` khác `mask_digits` của bài 47: ở đây chuỗi ngắn, lẫn chữ hay chữ số không phải ASCII bị che hết thành dấu `*` thay vì bị từ chối, để hồ sơ vẫn tới người duyệt. Ba hệ quả cần biết:

- `fraud-scoring` không còn số rõ để đối chiếu; muốn tìm hồ sơ trùng số phải dùng hash HMAC có khóa của bài 47.
- Che số chưa làm bảng hết nhạy cảm: bảng vẫn giữ ngày sinh và giới tính rõ, và bài 47 đếm được **10.000** ứng viên còn khớp với mặt nạ ba-hai cộng hai trường đó. Bảng vẫn là dữ liệu cá nhân, cần quyền truy cập và thời hạn lưu.
- Để nối một hàng với document OCR của bài 47 (khóa `(kyc_id, document_type, model_version, image_sha256)`), `kyc_ocr_results` giữ thêm `model_version` và `image_sha256`.

Thứ hai là **hai mô hình song song** (dataclass và hàng ORM) tốn thêm mã ánh xạ. «Imperative mapping» của SQLAlchemy ánh xạ thẳng dataclass vào bảng và bỏ được hai hàm trên; lab chọn mô hình tách để domain không dính vào cách ORM theo dõi đối tượng, và đó cũng là chỗ bẫy ở mục sau nằm.

## Unit of Work: một Session cho mỗi command

Port `UnitOfWork` thay `TransactionManager` của bài 68. Nó giữ `begin()` cũ và thêm `applications`, kho gắn với transaction đang mở:

```python
# src/kyc_service/application/ports.py (trích)
class UnitOfWork(Protocol):
    """Một transaction và các kho dùng chung nó. Thay `TransactionManager` của bài 68.

    `begin()` mở transaction; `applications` chỉ dùng được BÊN TRONG khối `with`, vì nó là kho
    gắn với transaction đang mở. Dùng ngoài khối thì ném `UnitOfWorkNotStarted`.
    """

    @property
    def applications(self) -> KycRepository: ...
```

Bản SQLAlchemy mở một `Session` mới ở `begin()`, bọc nó trong `session.begin()` để thoát khối bình thường là commit, thoát bằng exception là rollback:

```python
# src/kyc_service/infrastructure/sql/unit_of_work.py (trích)
    @contextmanager
    def begin(self) -> Iterator[None]:
        if self._current.get() is not None:
            raise NestedUnitOfWork("begin() lồng nhau: mỗi command chỉ có một transaction")
        session = self._factory()
        token = self._current.set((session, SqlAlchemyKycRepository(session)))
        try:
            with session.begin():  # commit khi thoát bình thường, rollback khi có exception
                yield
        finally:
            try:
                self._current.reset(token)  # ValueError nếu thoát ở Context khác với lúc vào
            finally:
                session.close()  # dù reset lỗi, Session vẫn phải đóng để trả kết nối
```

Một đối tượng `SqlAlchemyUnitOfWork` được ghép vào mediator một lần và dùng chung cho mọi request, nên nó không giữ Session trong thuộc tính thường. Cặp (Session, kho) nằm trong một `ContextVar`, và phạm vi của nó là **Context**, không phải «thread hay task» nói chung:

- Mỗi request có Context riêng (FastAPI chạy endpoint đồng bộ trong thread pool, mỗi lần gọi trên một bản sao Context), nên hai request đồng thời có hai Session; test cho hai thread cùng đứng trong `begin()` bằng một `Barrier`.
- `asyncio.create_task` và `asyncio.to_thread` mở **bên trong** `begin()` chép Context, nên **dùng chung** Session của lời gọi cha; Session đồng bộ không an toàn khi chia cho thread khác.
- `threading.Thread` tự mở bên trong `begin()` không thừa kế Context: `uow.applications` ở đó ném `UnitOfWorkNotStarted`.
- Vào `begin()` ở Context này mà thoát ở Context khác là `ValueError`; `finally` lồng nhau vẫn đóng Session và trả kết nối.

Cả bốn ý có test. Vấn đề của async nằm chỗ khác: Session đồng bộ chặn event loop, nên code async cần `AsyncSession` (lab không đi vào). Tài liệu `contextvars` khuyên tạo `ContextVar` ở cấp module vì biến không được giải phóng cùng đối tượng chứa nó; lab giữ một biến cho mỗi UoW để hai UoW không dùng chung giá trị, và chỉ tạo một UoW lúc khởi động.

Hai chi tiết nữa có test: `begin()` lồng nhau ném `NestedUnitOfWork`, và dùng `uow.applications` ngoài khối `with` ném `UnitOfWorkNotStarted` thay vì âm thầm mở transaction. Phía application chỉ đổi hai chỗ: `transaction_behavior` gọi `uow.begin()`, và handler lấy `self._uow.applications` thay cho `self._repository`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 260" role="img" aria-label="Một command đi qua transaction_behavior gọi uow.begin, UnitOfWork mở Session và đặt vào ContextVar, handler dùng uow.applications để get và save, PostgreSQL chạy BEGIN rồi COMMIT. Thoát khối bình thường dẫn tới COMMIT; có exception dẫn tới ROLLBACK."><defs><marker id="py69-1-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="30" width="190" height="80" rx="10" class="f-warn-soft s-warn"/><text x="36" y="62" class="t-b">transaction_behavior</text><text x="36" y="90" class="t-small">gọi uow.begin()</text><rect x="250" y="30" width="190" height="80" rx="10" class="f-pk-soft s-pk"/><text x="266" y="62" class="t-b">UnitOfWork</text><text x="266" y="90" class="t-small">Session vào ContextVar</text><rect x="480" y="30" width="190" height="80" rx="10" class="f-pk-soft s-pk"/><text x="496" y="62" class="t-b">Handler</text><text x="496" y="90" class="t-small">uow.applications.get, save</text><rect x="710" y="30" width="150" height="80" rx="10" class="f-phys-soft s-phys"/><text x="726" y="62" class="t-b">PostgreSQL</text><text x="726" y="90" class="t-small">BEGIN ... COMMIT</text><path d="M210 70 L248 70" class="f-none s-phys" stroke-width="2" marker-end="url(#py69-1-arrow)"/><path d="M440 70 L478 70" class="f-none s-phys" stroke-width="2" marker-end="url(#py69-1-arrow)"/><path d="M670 70 L708 70" class="f-none s-phys" stroke-width="2" marker-end="url(#py69-1-arrow)"/><rect x="250" y="160" width="250" height="50" rx="8" class="f-phys-soft s-phys"/><text x="266" y="190" class="t-small">thoát khối bình thường: COMMIT</text><rect x="530" y="160" width="250" height="50" rx="8" class="f-warn-soft s-warn"/><text x="546" y="190" class="t-small">exception: ROLLBACK, ném tiếp</text><path d="M345 110 L345 158" class="f-none s-phys dash" stroke-width="2" marker-end="url(#py69-1-arrow)"/><path d="M575 110 L575 158" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py69-1-arrow)"/><text x="20" y="242" class="t-small">Session nằm trong ContextVar của đoạn mở begin(); task con tạo trong khối đó dùng chung Session.</text></svg>
</div>
<figcaption>Hình 2. Vòng đời một command: pipeline mở transaction, handler dùng kho của transaction đó, thoát khối quyết định commit hay rollback.</figcaption>
</figure>

## Giữ version đã nạp: optimistic concurrency và bẫy identity map

Mapping có `version_id_col`, nên mỗi `UPDATE` mang điều kiện `WHERE version = <version đã nạp>` và đặt version mới. Nếu số hàng khớp khác một, SQLAlchemy ném `StaleDataError`; repository dịch nó thành `ConcurrencyConflict` của application, và API trả 409 `concurrent-update`. Repository còn gọi `flag_modified(row, "status")` trong `save` để mọi lần ghi aggregate đều tăng version dù chỉ hàng con đổi: aggregate là một đơn vị nhất quán, nên hai người cùng sửa nó, kể cả chỉ phần con, phải xung đột.

Bản đầu của lab mất kiểm tra này mà không báo gì: `save` gọi `session.get(ApplicationRow, id)` để lấy lại hàng, và test hai transaction chồng nhau đỏ với «DID NOT RAISE». Theo tài liệu SQLAlchemy, các đối tượng trong Session được giữ bằng **tham chiếu yếu**. Sau `get`, aggregate dựng xong thì không ai còn cầm hàng ORM, nó bị thu gom, và `session.get` lần hai nạp lại hàng với version **mới**, nên `WHERE version = <mới>` luôn khớp. Test cô lập hiện tượng:

```python
# tests/test_repository.py (trích)
def test_session_identity_map_forgets_unreferenced_rows(engine: Engine) -> None:
    """Lý do repository phải giữ hàng đã nạp: identity map của Session chỉ giữ tham chiếu yếu."""
    uow = new_uow(engine)
    with uow.begin():
        uow.applications.add(new_application())
    bump = text("UPDATE lab69.kyc_applications SET version = version + 1")
    with Session(engine) as session:
        row = session.get(ApplicationRow, "KYC-0001")
        assert row is not None
        loaded_version = row.version
        with engine.begin() as other:
            other.execute(bump)  # giao dịch khác đã commit version mới
        kept = session.get(ApplicationRow, "KYC-0001")  # `row` còn được cầm: không nạp lại
        assert kept is not None
        assert kept.version == loaded_version
        del row, kept
        gc.collect()
        reloaded = session.get(ApplicationRow, "KYC-0001")  # không ai cầm: nạp lại version mới
        assert reloaded is not None
        assert reloaded.version == loaded_version + 1
```

Chữa bằng cách để repository **giữ hàng đã nạp** suốt transaction: UoW tạo đúng một kho cho mỗi `begin()`, và kho cất mọi hàng nó nạp hay thêm vào `_rows`:

```python
# src/kyc_service/infrastructure/sql/repository.py (trích)
    def get(self, application_id: str) -> KycApplication | None:
        row = self._rows.get(application_id)
        if row is None:
            row = self._session.get(ApplicationRow, application_id, options=LOAD_ALL)
            if row is None:
                return None
            self._rows[application_id] = row
        return to_domain(row)

    def save(self, application: KycApplication) -> None:
        row = self._rows.get(application.application_id)
        if row is None:
            raise ValueError(f"hồ sơ {application.application_id} chưa được add hoặc get")
        apply_to_row(row, application)
        # Mọi lần ghi aggregate đều tăng version, kể cả khi chỉ hàng con đổi: aggregate là một
        # đơn vị nhất quán, hai người cùng sửa nó phải xung đột.
        flag_modified(row, "status")
        try:
            self._session.flush()
        except StaleDataError as exc:
            raise ConcurrencyConflict(application.application_id) from exc
```

Từ giờ `save` luôn ghi với version đã đọc. Demo cho hai người duyệt, trong đó RV-002 nạp trước, RV-001 commit trước, rồi RV-002 ghi:

```text
== 4. Hai người duyệt cùng lúc: optimistic concurrency
  RV-001 commit trước; version trong bảng: 3
  RV-002 ghi sau: ConcurrencyConflict: hồ sơ KYC-0001 vừa bị cập nhật bởi giao dịch khác
  trạng thái cuối: approved bởi RV-001
```

Điều này chỉ bắt hai transaction **chồng nhau**. Nếu RV-002 gửi request sau khi RV-001 đã commit, request nạp bản mới và chính domain từ chối; version không liên quan. Chạy hai quyết định nối tiếp qua HTTP:

```text
== 7. Hai quyết định nối tiếp qua HTTP: domain chặn, không cần version
  RV-001: 200; RV-002 (đến sau): 409
  loại lỗi: https://toinoi.example/problems/invalid-status-transition
```

Request thứ hai nhận 409 `invalid-status-transition`, không phải `concurrent-update`. Người duyệt nhìn màn hình cũ rồi bấm lúc sau cũng rơi vào trường hợp này, vì bản nạp lúc bấm đã mới. Muốn bắt «quyết định dựa trên bản đã xem», client phải gửi version đã xem (`ETag` và `If-Match`) và command mang theo; lab chưa làm.

Version là 3 vì mỗi lần `flush` ghi hàng là một version: `add` (1), `save` sau khi chấm điểm (2), quyết định của RV-001 (3). Đừng dùng số version làm số đếm nghiệp vụ.

## Phía đọc, N+1 và ràng buộc ở bảng

Query không đi qua Unit of Work. `SqlAlchemyReadModel` mở một Session ngắn cho mỗi lần đọc và chọn thẳng ra view, không dựng aggregate. Hàng chờ duyệt cần hai câu `SELECT` (hàng gốc, rồi OCR của cả danh sách), vì phía đọc không nạp `documents`; dựng cả aggregate tốn thêm một câu cho bảng giấy tờ. Số câu được đếm bằng event `before_cursor_execute` của engine, ở ba cỡ hàng chờ:

```text
== 5. Số câu SELECT để lấy hàng chờ duyệt, theo số hồ sơ
  hồ sơ  read model  repository (dựng aggregate)
      5           2           3
     50           2           3
    520           3           5
```

Số câu tăng theo **lô**, không theo hồ sơ: `selectinload` gửi một câu cho tối đa 500 khóa cha (theo tài liệu), nên 5 và 50 hồ sơ cùng cho 2 và 3 câu, còn 520 hồ sơ vượt một lô và cho 3 và 5. Hai thứ nữa nằm ở bảng, vì code có thể sai còn bảng thì không nên: `CheckConstraint` bắt buộc «có người duyệt khi và chỉ khi đã `approved` hoặc `rejected`», chốt thứ hai cho quy tắc domain đã kiểm; và mã hồ sơ sinh từ một `Sequence`, **không** rollback, nên transaction lỗi để lại lỗ trong dãy (mục 3 của output ở phần «Chạy và kiểm chứng»: hồ sơ kế tiếp là `KYC-0002`). Demo cố ghi `approved` mà không có người duyệt:

```text
== 6. Ràng buộc ở cơ sở dữ liệu: duyệt mà không có người duyệt
  IntegrityError: new row for relation "kyc_applications" violates check constraint "ck_decision_has_reviewer"
```

Mã hồ sơ không ngừng ở `KYC-9999`: domain và command nhận `KYC-[0-9]{4,}` (dùng `[0-9]` vì `\d` của `re` nhận cả chữ số Unicode như Arabic-Indic), và `next_id` in số `{n:04d}` nên sang năm chữ số khi sequence vượt 9999. Demo đặt sequence về 10000:

```text
== 8. Mã hồ sơ không ngừng ở KYC-9999
  POST: 201 /applications/KYC-10000
  GET:  needs_review
```

## Test bằng PostgreSQL thật, không bằng bản giả

Bản trong bộ nhớ của bài 68 không thể phát hiện bẫy identity map, ràng buộc CHECK hay lỗ trong sequence. Test của bài chạy trên PostgreSQL của hạ tầng chung, trong schema `lab69`; fixture tạo schema một lần cho cả phiên, và nếu không nối được thì báo bằng lệnh khởi động chứ không bỏ qua:

```python
# tests/conftest.py (trích)
@pytest.fixture(scope="session")
def engine() -> Iterator[Engine]:
    eng = create_engine(sqlalchemy_url(), pool_pre_ping=True)
    try:
        with eng.connect() as conn:
            conn.execute(text("SELECT 1"))
    except OperationalError:
        pytest.fail(f"không nối được PostgreSQL tại {PG_URL} ({COMPOSE_HINT})", pytrace=False)
    create_schema(eng)
    yield eng
    drop_schema(eng)
```

Một fixture khác gắn listener `begin`, `commit`, `rollback` của engine, nên test khẳng định trên transaction của cơ sở dữ liệu chứ không trên một danh sách tự ghi.

## Chạy và kiểm chứng

Cần uv, Python 3.14 và PostgreSQL của hạ tầng chung (`docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait`). Từ gốc repository:

```bash
uv run --project examples/toi-noi-python/69-repository-unit-of-work examples/toi-noi-python/69-repository-unit-of-work/main.py
```

```text
== 1. Nộp hồ sơ qua pipeline: một transaction thật trên PostgreSQL
  [log] > SubmitDriverApplication
  [log] < SubmitDriverApplication ok
  kết quả: KYC-0001 needs_review; transaction: ['begin', 'commit']
  kyc_applications: 1 hàng
  kyc_documents: 2 hàng
  kyc_ocr_results: 2 hàng
  version của hồ sơ: 2
== 2. Số giấy tờ vào cơ sở dữ liệu ở dạng đã che
  fields của CCCD trong bảng: id_number = 099*******83
== 3. Lỗi bất ngờ giữa chừng: rollback, sequence để lại lỗ
  RuntimeError: bug trong fraud-scoring adapter
  transaction: ['begin', 'rollback']; số hồ sơ: 0
  hồ sơ kế tiếp: KYC-0002
```

(trích: các mục 4 tới 8 của output đã in ở các mục trên.) Lab in lời nhắc ba service là adapter giả, và `build_container()` phát `FakeAdapterWarning` (một `UserWarning`) nêu tên port dùng adapter giả. Schema `lab69` bị xóa khi chạy xong. Kiểm các kết quả trên:

```bash
uv run --project examples/toi-noi-python/69-repository-unit-of-work examples/toi-noi-python/69-repository-unit-of-work/main.py --verify
```

```text
OK 33/33
```

33 ca: repository, Unit of Work (kể cả phạm vi Context), xung đột và ràng buộc, pipeline trên PostgreSQL, validation, cảnh báo adapter giả, khóa nối OCR, `KYC-10000`, `pending_check` khi đọc lại và các ca HTTP. Chuỗi đầy đủ gồm ruff, mypy `--strict`, import-linter với bốn hợp đồng (mới: lõi không import `sqlalchemy` hay `psycopg`), pytest và `--verify`:

```bash
sh examples/toi-noi-python/69-repository-unit-of-work/verify.sh
```

```text
== 1. ruff format --check
40 files already formatted
== 2. ruff check
All checks passed!
== 3. mypy --strict
Success: no issues found in 38 source files
== 4. import-linter
---------
Contracts
---------
Analyzed 45 files, 102 dependencies.
------------------------------------
Bốn lớp: api > infrastructure > application > domain KEPT
Lõi không import framework web KEPT
Domain không import pydantic KEPT
Lõi không import thư viện cơ sở dữ liệu KEPT
Contracts: 4 kept, 0 broken.
== 5. pytest
........................................................................ [ 52%]
................................................................         [100%]
136 passed in 19.07s
== 6. main.py --verify
OK 33/33
OK 6/6
```

(trích: bỏ phần hình ASCII in đầu của import-linter.) Python 3.14.7, SQLAlchemy 2.1.8, psycopg 3.3.8, PostgreSQL 17.11, FastAPI 0.162.0, pydantic 2.14.3, import-linter 2.16, mypy 2.4.1, pytest 9.2.0, ruff 0.17.4 (theo `uv.lock`). Output do một lần chạy chung với nhiều tác vụ khác của máy (load average 1, 5, 15 phút đo bằng `uptime` sau khi chạy: 299,68; 344,73; 374,10), nên các số giây chỉ để tham khảo và bài không dùng chúng làm kết luận.

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| Hai người ghi cùng hồ sơ, không ai nhận `ConcurrencyConflict` | Repository có giữ hàng đã nạp suốt transaction không; `session.get` lần hai nạp lại version mới (tham chiếu yếu) |
| `InvalidRequestError ... lazy='raise'` | Truy vấn quên `selectinload` cho hàng con mà code đang đọc |
| `UnitOfWorkNotStarted` | Code dùng `uow.applications` ngoài khối `with uow.begin()` |
| Mã hồ sơ nhảy số (`KYC-0001` rồi `KYC-0003`) | Sequence không rollback: có transaction lỗi ở giữa; bình thường, đừng «sửa» |
| Hai request đồng thời thấy chung một Session | Session đang nằm trong thuộc tính thường của UoW thay vì `ContextVar` |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>Lab dùng <code>create_all</code>, production dùng migration (bài 24). Một PostgreSQL, không đo pool, không read replica. <code>ConcurrencyConflict</code> trả cho client dưới dạng 409 và lab không thử lại; chưa có <code>If-Match</code> cho quyết định dựa trên bản đã xem. Mặt nạ số giấy tờ chưa phải chính sách PII (bài 47), và bảng vẫn giữ ngày sinh, giới tính rõ. <code>ContextVar</code> đúng với code đồng bộ; async cần <code>AsyncSession</code>. Handler vẫn gọi OCR, khuôn mặt, điểm rủi ro bên trong transaction; bài 70 đo hệ quả đó. Chưa có outbox. Dữ liệu là dữ liệu tổng hợp.</p>
</div>

**Bài tập:** (1) Thêm quy tắc «một tài xế chỉ có một hồ sơ chưa quyết định». Lấy hồ sơ đang mở của một tài xế nên đặt ở repository hay ở read model, và ngoài mã Python còn cần gì ở bảng? (2) `RecordReviewDecisionHandler` gặp `ConcurrencyConflict`. Nên tự nạp lại và thử quyết định lần nữa, hay trả 409? Nêu hậu quả của mỗi lựa chọn.

<details><summary>Xem lời giải</summary><p>(1) Quy tắc nằm trong lệnh ghi (nộp hồ sơ phải từ chối khi đã có hồ sơ mở), nên repository cần thêm một thao tác trả <b>aggregate</b> như <code>get_open_for_driver(driver_id)</code>; read model chỉ phục vụ hiển thị. Hai request cùng lúc vẫn lọt qua mã Python, nên bảng cần chỉ mục <b>unique một phần</b> trên <code>driver_id</code> với điều kiện <code>status</code> chưa phải <code>approved</code> hoặc <code>rejected</code>. (2) Trả 409. Tự thử lại sẽ nạp bản mới (đã <code>approved</code>) và domain từ chối, nên chỉ thêm một lượt nạp thừa; thử lại chỉ đáng với lệnh không xung đột về nghiệp vụ và làm lại được an toàn. Người duyệt thứ hai cần biết mình đến sau.</p></details>

Bài trước: [bài 68](/articles/python-cqrs-command-query-handler-mediator). Bài tiếp: [bài 70](/articles/python-use-case-ho-so-tai-xe-end-to-end).

## Tài liệu gốc

- [SQLAlchemy 2.1: Configuring a Version Counter](https://docs.sqlalchemy.org/en/21/orm/versioning.html).
- [SQLAlchemy 2.1: Session state management, gồm «Session Referencing Behavior»](https://docs.sqlalchemy.org/en/21/orm/session_state_management.html).
- [SQLAlchemy 2.1: Relationship Loading Techniques (selectinload, raiseload)](https://docs.sqlalchemy.org/en/21/orm/queryguide/relationships.html).
- [SQLAlchemy 2.1: ORM Mapped Class Configuration (declarative và imperative)](https://docs.sqlalchemy.org/en/21/orm/mapping_styles.html).
- [Python 3.14: contextvars](https://docs.python.org/3.14/library/contextvars.html).
- [PostgreSQL 17: Sequence Manipulation Functions (nextval không rollback)](https://www.postgresql.org/docs/17/functions-sequence.html).
