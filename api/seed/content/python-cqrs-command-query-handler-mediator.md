---
slug: python-cqrs-command-query-handler-mediator
title: "CQRS và mediator: command/query handler, pipeline không cần thư viện nặng"
summary: "Bài 68: tách command và query thành thông điệp có handler riêng, viết mediator dispatch theo kiểu và pipeline log, validate, transaction bằng hàm bọc, rồi so với MediatR. Có lab chạy được."
category: python-advanced
level: advanced
tags: [python, toi-noi, cqrs, mediator, clean-architecture]
language: vi
---

Hồ sơ tài xế của [Tới Nơi](/articles/du-an-mau-toi-noi) không chỉ đến từ một nơi. Route `POST /applications` nhận hồ sơ từ app tài xế; một worker đọc Kafka (bài 38) có thể làm tiếp hồ sơ khi service OCR sống lại; một công cụ dòng lệnh của nhóm vận hành cũng có thể cần nộp hồ sơ thử. Ở bài 67, mỗi use case là một lớp riêng và mỗi nơi gọi nó phải tự nhớ nghi thức quen thuộc: kiểm định dạng mã, ghi log, mở transaction, đóng transaction khi lỗi. Nghi thức nào bị quên ở một nơi thì nơi đó sai theo cách riêng.

Bài này gom nghi thức vào một chỗ. Mỗi ý định là một **thông điệp** (command hoặc query), mỗi thông điệp có đúng một **handler**, và một **mediator** nhỏ tìm handler theo kiểu rồi cho thông điệp đi qua một **pipeline** các lớp bọc: log, validate, transaction. Toàn bộ mediator dài chừng 60 dòng, không cần thư viện.

Đây là bài 68 của lộ trình **Python nâng cao qua Tới Nơi**. Bạn cần đã đọc [bài 67](/articles/python-clean-architecture-domain-application-infrastructure), nhớ decorator và context manager ([bài 17](/articles/python-decorator-context-manager)) và pydantic ([bài 15](/articles/python-pydantic-validation-settings)). Bài C# cùng chủ đề là [bài 32 của loạt .NET 10](/articles/dotnet10-cqrs-mediator-pipeline).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Pipeline có hai lớp bọc: <i>validate</i> (kiểm định dạng) và <i>transaction</i> (mở, commit hoặc rollback). Một command có mã tài xế sai định dạng đi vào pipeline. Nếu đặt transaction ở ngoài validate thay vì ở trong, kết quả cuối cùng với người gọi có khác không, và hệ thống có gì khác?</p>
<details><summary>Xem đáp án</summary><p><b>Người gọi vẫn nhận lỗi validate như cũ</b>, nhưng hệ thống làm thêm một việc vô ích: mở transaction rồi rollback cho mọi yêu cầu rác (<code>['begin', 'rollback']</code> thay vì danh sách rỗng). Với cơ sở dữ liệu thật, mỗi lần như vậy là một kết nối và một transaction bị chiếm. Bài chạy cả hai thứ tự để xem output.</p></details>
</div>

## Command, query và handler

CQRS (Command Query Responsibility Segregation) nghĩa là đường ghi và đường đọc được hiểu như hai việc khác nhau, mỗi việc có mô hình riêng. Nó **không** bắt buộc hai cơ sở dữ liệu hay event sourcing; lab của bài vẫn chạy trên một kho dữ liệu, chỉ khác là hai phía đi qua hai cổng khác nhau.

- **Command** là ý định thay đổi trạng thái: `SubmitDriverApplication` (nộp hồ sơ) và `RecordReviewDecision` (người duyệt quyết định). Command chạy trong transaction, đi qua aggregate `KycApplication`, trả `ApplicationView`.
- **Query** là nhu cầu đọc: `GetApplication` và `ListReviewQueue`. Query không đổi trạng thái, không mở transaction và trả thẳng view qua port phía đọc `ApplicationReadModel`, không dựng lại aggregate chỉ để hiển thị.

Mỗi thông điệp là một frozen dataclass kế thừa `Command[R]` hoặc `Query[R]`, trong đó `R` là kiểu kết quả (cú pháp generic của PEP 695). Các chú thích kiểu mô tả luôn hình dạng hợp lệ:

```python
# src/kyc_service/application/commands.py (trích)
ApplicationId = Annotated[str, StringConstraints(pattern=r"^KYC-[0-9]{4,}$")]
DriverId = Annotated[str, StringConstraints(pattern=r"^DR-[0-9]{3}$")]
ReviewerId = Annotated[str, StringConstraints(pattern=r"^RV-[0-9]{3}$")]
ImageRef = Annotated[str, StringConstraints(strip_whitespace=True, min_length=1)]
Reason = Annotated[str, StringConstraints(strip_whitespace=True, min_length=1)]


@dataclass(frozen=True)
class SubmitDriverApplication(Command[ApplicationView]):
    driver_id: DriverId
    cccd_ref: ImageRef
    gplx_ref: ImageRef
    selfie_ref: ImageRef
    face_consent: StrictBool  # đồng ý là quyết định pháp lý: "false" dạng chuỗi không được đoán


@dataclass(frozen=True)
class RecordReviewDecision(Command[ApplicationView]):
    application_id: ApplicationId
    reviewer_id: ReviewerId
    decision: Literal["approved", "rejected"]
    reason: Reason
```

Handler giữ nguyên thân của use case ở bài 67, chỉ đổi vỏ: mỗi handler là một lớp nhận port qua constructor và có đúng một phương thức `handle(message)`. Ví dụ phần đầu của handler nộp hồ sơ:

```python
# src/kyc_service/application/handlers.py (trích)
    def handle(self, command: SubmitDriverApplication) -> ApplicationView:
        application = KycApplication.submit(
            application_id=self._repository.next_id(),
            driver_id=command.driver_id,
            documents=(
                DocumentImage(DocumentKind.CCCD, command.cccd_ref),
                DocumentImage(DocumentKind.GPLX, command.gplx_ref),
            ),
            selfie_ref=command.selfie_ref,
            face_consent=command.face_consent,
            now=self._clock.now(),
        )
        self._repository.add(application)
```

Hình 1 vẽ hai đường. Cùng một mediator, hai phía đi hai hướng: command qua handler ghi rồi tới kho aggregate trong transaction; query qua handler đọc rồi tới read model.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 290" role="img" aria-label="Command SubmitDriverApplication và query GetApplication cùng đi vào một mediator. Command tới handler ghi rồi KycRepository; query tới handler đọc rồi ApplicationReadModel."><defs><marker id="py68-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="30" width="180" height="90" rx="10" class="f-pk-soft s-pk"/><text x="36" y="62" class="t-b">Command</text><text x="36" y="90" class="t-small">SubmitDriverApplication</text><rect x="20" y="170" width="180" height="90" rx="10" class="f-pk-soft s-pk"/><text x="36" y="202" class="t-b">Query</text><text x="36" y="230" class="t-small">GetApplication</text><rect x="230" y="30" width="180" height="230" rx="10" class="f-warn-soft s-warn"/><text x="246" y="62" class="t-b">Mediator</text><text x="246" y="100" class="t-small">dispatch theo kiểu</text><text x="246" y="124" class="t-small">pipeline: log, validate,</text><text x="246" y="148" class="t-small">transaction (command)</text><path d="M200 75 L228 75" class="f-none s-phys" stroke-width="2" marker-end="url(#py68-0-arrow)"/><path d="M200 215 L228 215" class="f-none s-phys" stroke-width="2" marker-end="url(#py68-0-arrow)"/><rect x="440" y="30" width="180" height="90" rx="10" class="f-phys-soft s-phys"/><text x="456" y="62" class="t-b">Handler ghi</text><text x="456" y="90" class="t-small">domain, rồi save</text><rect x="440" y="170" width="180" height="90" rx="10" class="f-phys-soft s-phys"/><text x="456" y="202" class="t-b">Handler đọc</text><text x="456" y="230" class="t-small">trả view trực tiếp</text><path d="M410 75 L438 75" class="f-none s-phys" stroke-width="2" marker-end="url(#py68-0-arrow)"/><path d="M410 215 L438 215" class="f-none s-phys" stroke-width="2" marker-end="url(#py68-0-arrow)"/><rect x="650" y="30" width="210" height="90" rx="10" class="f-phys-soft s-phys"/><text x="666" y="62" class="t-b">KycRepository</text><text x="666" y="90" class="t-small">aggregate KycApplication</text><rect x="650" y="170" width="210" height="90" rx="10" class="f-phys-soft s-phys"/><text x="666" y="202" class="t-b">ApplicationReadModel</text><text x="666" y="230" class="t-small">view, không dựng aggregate</text><path d="M620 75 L648 75" class="f-none s-phys" stroke-width="2" marker-end="url(#py68-0-arrow)"/><path d="M620 215 L648 215" class="f-none s-phys" stroke-width="2" marker-end="url(#py68-0-arrow)"/></svg>
</div>
<figcaption>Hình 1. Hai đường qua cùng một mediator: command đi qua aggregate trong transaction, query đi thẳng ra view.</figcaption>
</figure>

Một chú ý trung thực: trong lab, phía ghi và phía đọc dùng chung một dict trong bộ nhớ, nên `InMemoryReadModel` chỉ dựng view từ chính kho đó. Việc tách thật nằm ở **cổng** (`ApplicationReadModel` là một port riêng); khi lên PostgreSQL ở bài 69, phía đọc có thể là một câu `SELECT` thẳng ra view mà không qua aggregate.

## Mediator: dispatch theo kiểu, chừng 60 dòng

Mediator giữ một dict từ **kiểu thông điệp** sang **pipeline đã dựng sẵn**. Hai kiểu hàm cho bước kế tiếp và lớp bọc, rồi `register` và `send`:

```python
# src/kyc_service/application/mediator.py (trích)
type Next = Callable[[Any], Any]
"""Bước kế tiếp trong pipeline: nhận thông điệp, trả kết quả."""

type Behavior = Callable[[Any, Next], Any]
"""Một lớp bọc quanh handler: nhận thông điệp và `next`, quyết định có gọi `next` hay không."""


class Handler[M, R](Protocol):
    def handle(self, message: M) -> R: ...
```

```python
# src/kyc_service/application/mediator.py (trích)
def _wrap(behavior: Behavior, next_step: Next) -> Next:
    # Hàm riêng (không viết lambda ngay trong vòng for) để mỗi lớp bọc giữ đúng
    # `behavior` và `next_step` của nó; closure trong vòng lặp sẽ bắt biến cuối cùng.
    return lambda message: behavior(message, next_step)


class Mediator:
    def __init__(self, behaviors: Sequence[Behavior] = ()) -> None:
        self._behaviors = tuple(behaviors)
        self._pipelines: dict[type, Next] = {}

    def register[M: Message[Any], R](self, message_type: type[M], handler: Handler[M, R]) -> None:
        """Gắn đúng một handler cho một kiểu thông điệp và dựng sẵn pipeline của nó."""
        if message_type in self._pipelines:
            raise DuplicateHandler(f"{message_type.__name__} đã có handler")
        pipeline: Next = handler.handle
        # Duyệt từ trong ra ngoài: behavior đầu danh sách bọc ngoài cùng.
        for behavior in reversed(self._behaviors):
            pipeline = _wrap(behavior, pipeline)
        self._pipelines[message_type] = pipeline

    def send[R](self, message: Message[R]) -> R:
        """Gửi thông điệp; dispatch theo kiểu chính xác, không theo kiểu cha."""
        pipeline = self._pipelines.get(type(message))
        if pipeline is None:
            raise HandlerNotFound(f"chưa đăng ký handler cho {type(message).__name__}")
        result: R = pipeline(message)
        return result
```

Vài chi tiết đáng đọc kỹ:

- **Dispatch theo kiểu chính xác**: `send` tra `type(message)`. Một lớp con của `GetApplication` không tự rơi về handler của lớp cha; thiếu handler là `HandlerNotFound` ngay khi gửi, và đăng ký hai handler cho một kiểu là `DuplicateHandler` ngay lúc khởi động. Cả hai có test.
- **Đăng ký tường minh, không quét package**: không reflection, không decorator tự đăng ký. Danh sách handler nằm ở composition root, đọc một lượt là biết service nhận được những thông điệp nào.
- **Pipeline dựng một lần lúc đăng ký**, không dựng lại mỗi lần gửi. Vòng `for` duyệt danh sách behavior từ cuối về đầu để behavior đầu danh sách bọc ngoài cùng.
- **`_wrap` là hàm riêng, không phải lambda trong vòng `for`.** Closure trong Python bắt *biến*, không bắt *giá trị*: viết `pipeline = lambda m: behavior(m, pipeline)` ngay trong vòng lặp thì cả `behavior` lẫn `pipeline` chỉ được tra lúc gọi. Lúc đó `pipeline` đã là chính lambda ngoài cùng nên nó gọi lại chính nó, và lời gọi đầu tiên kết thúc bằng `RecursionError` (ta thử riêng ngoài lab). Test `test_each_layer_keeps_its_own_next` giữ cho lỗi này không quay lại.

Mediator này là phiên bản giảng giải. Nó không có notification (một thông điệp, nhiều handler), không streaming, không async và không chẩn đoán đăng ký; bản async sẽ cần `await` ở mỗi lớp bọc.

## Pipeline: ba lớp bọc, mỗi lớp một việc

Một behavior là hàm `(message, next_step) -> kết quả`. Gọi `next_step(message)` là cho thông điệp đi tiếp; không gọi là chặn lại. Đó chính là hình dạng của decorator ở bài 17, chỉ khác là danh sách lớp bọc được ghép lúc chạy và áp dụng cho mọi handler.

```python
# src/kyc_service/application/behaviors.py (trích)
def logging_behavior(log: Callable[[str], None]) -> Behavior:
    """Ghi vào, ra, hoặc lỗi của mọi thông điệp. Ghi TÊN thông điệp, không ghi nội dung (PII)."""

    def behavior(message: Any, next_step: Next) -> Any:
        name = type(message).__name__
        log(f"> {name}")
        try:
            result = next_step(message)
        except Exception as exc:
            log(f"! {name} {type(exc).__name__}")
            raise
        log(f"< {name} ok")
        return result

    return behavior
```

Logging ghi **tên** thông điệp chứ không ghi nội dung, vì nội dung có số giấy tờ và đường dẫn ảnh. Validation dùng chính chú thích kiểu trên dataclass:

```python
# src/kyc_service/application/behaviors.py (trích)
def validation_behavior(
    adapter_factory: Callable[[type], TypeAdapter[Any]] = TypeAdapter,
) -> Behavior:
    """Kiểm hình dạng thông điệp theo chú thích kiểu của dataclass, gom MỌI lỗi một lượt.

    Handler nhận bản ĐÃ VALIDATE (đã cắt khoảng trắng, đã ép về đúng kiểu), không phải thông
    điệp gốc của người gọi. Chỉ kiểm hình dạng; quy tắc nghiệp vụ như «hai giấy tờ khác loại»
    vẫn nằm ở domain. `adapter_factory` để test đếm số lần dựng bộ kiểm.
    """
    adapters: dict[type, TypeAdapter[Any]] = {}

    def behavior(message: Any, next_step: Next) -> Any:
        message_type = type(message)
        adapter = adapters.get(message_type)
        if adapter is None:  # dựng đúng một lần cho mỗi kiểu thông điệp
            adapter = adapters[message_type] = adapter_factory(message_type)
        try:
            validated = adapter.validate_python(asdict(message))
        except ValidationError as exc:
            # Không chép `input` của pydantic vào lỗi: có thể là dữ liệu nhạy cảm.
            raise ValidationFailed(
                [
                    FieldError(".".join(str(p) for p in e["loc"]), e["type"], e["msg"])
                    for e in exc.errors()
                ]
            ) from None
        return next_step(validated)

    return behavior
```

`TypeAdapter` của pydantic dựng bộ kiểm **đúng một lần** cho mỗi kiểu thông điệp (tra `adapters.get` trước, dựng sau; bản đầu viết `setdefault(kiểu, TypeAdapter(kiểu))` dựng lại bộ kiểm ở mọi lần gửi, và một ca `--verify` đếm số lần dựng để giữ lỗi đó khỏi quay lại). `asdict` đưa dataclass về dict để kiểm, và điểm quan trọng: **handler nhận bản đã validate** (`next_step(validated)`), không phải thông điệp thô của người gọi. Nhờ vậy khoảng trắng thừa quanh đường dẫn ảnh đã được cắt, kiểu đã đúng; nếu chuyển tiếp bản thô, validate chỉ là hình thức, handler vẫn ăn dữ liệu chưa qua kiểm. Lỗi được gom hết một lượt (mã sai và ảnh rỗng cùng lúc cho hai lỗi) và **không** chép giá trị đầu vào vào lỗi, vì đó có thể là dữ liệu nhạy cảm. Behavior này chỉ kiểm *hình dạng*; quy tắc như «hai giấy tờ khác loại» vẫn ở domain. Ưu điểm so với kiểm trong API: worker Kafka và công cụ dòng lệnh gửi cùng command cũng được kiểm giống hệt, vì chúng không đi qua FastAPI. Lớp API chỉ còn kiểm kiểu JSON và từ chối trường lạ (`extra="forbid"`). Trường `face_consent` dùng `StrictBool` ở cả hai nơi: đồng ý là quyết định pháp lý nên chuỗi `"false"` bị từ chối (`bool_type`) chứ không bị đoán thành `False` (hay, tệ hơn, thành `True` vì chuỗi không rỗng).

Một cái giá cần nói rõ: `application` giờ import pydantic. Đó là lựa chọn có ý thức của lab, vì pydantic ở đây chỉ mô tả hình dạng dữ liệu, không phải framework web. Hợp đồng import-linter ghi đúng lựa chọn ấy: `domain` bị cấm pydantic, `application` thì được nhưng vẫn bị cấm `fastapi` và `starlette`.

```python
# src/kyc_service/application/behaviors.py (trích)
def transaction_behavior(transactions: TransactionManager) -> Behavior:
    """Chỉ command mới mở transaction; query đi thẳng."""

    def behavior(message: Any, next_step: Next) -> Any:
        if not isinstance(message, Command):
            return next_step(message)
        with transactions.begin():
            return next_step(message)

    return behavior
```

Transaction chỉ bọc **command**; query đi thẳng. `transactions.begin()` là context manager: thoát khối bình thường là commit, thoát bằng exception là rollback. Bản trong bộ nhớ (`InMemoryTransactions`) chụp ảnh các hàng khi vào và hoàn lại khi lỗi; bài 69 thay bằng Unit of Work trên `Session` thật.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Thông điệp đi vào qua logging, validation, transaction rồi handler; kết quả đi ngược ra. Mỗi lớp có một nhánh lỗi: ghi log rồi ném tiếp, dừng trước khi mở transaction, rollback rồi ném tiếp."><defs><marker id="py68-1-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="30" width="180" height="90" rx="10" class="f-warn-soft s-warn"/><text x="36" y="62" class="t-b">logging</text><text x="36" y="90" class="t-small">ghi vào, ra, lỗi</text><rect x="240" y="30" width="180" height="90" rx="10" class="f-warn-soft s-warn"/><text x="256" y="62" class="t-b">validation</text><text x="256" y="90" class="t-small">kiểm hình dạng</text><rect x="460" y="30" width="180" height="90" rx="10" class="f-warn-soft s-warn"/><text x="476" y="62" class="t-b">transaction</text><text x="476" y="90" class="t-small">chỉ bọc command</text><rect x="680" y="30" width="180" height="90" rx="10" class="f-phys-soft s-phys"/><text x="696" y="62" class="t-b">handler</text><text x="696" y="90" class="t-small">domain, save</text><path d="M200 60 L238 60" class="f-none s-phys" stroke-width="2" marker-end="url(#py68-1-arrow)"/><path d="M420 60 L458 60" class="f-none s-phys" stroke-width="2" marker-end="url(#py68-1-arrow)"/><path d="M640 60 L678 60" class="f-none s-phys" stroke-width="2" marker-end="url(#py68-1-arrow)"/><path d="M238 100 L200 100" class="f-none s-muted dash" stroke-width="2" marker-end="url(#py68-1-arrow)"/><path d="M458 100 L420 100" class="f-none s-muted dash" stroke-width="2" marker-end="url(#py68-1-arrow)"/><path d="M678 100 L640 100" class="f-none s-muted dash" stroke-width="2" marker-end="url(#py68-1-arrow)"/><rect x="20" y="170" width="180" height="60" rx="10" class="f-bg s-line"/><text x="36" y="196" class="t-small">ghi tên lỗi,</text><text x="36" y="214" class="t-small">ném tiếp</text><rect x="240" y="170" width="180" height="60" rx="10" class="f-bg s-line"/><text x="256" y="196" class="t-small">ValidationFailed:</text><text x="256" y="214" class="t-small">chưa mở transaction</text><rect x="460" y="170" width="180" height="60" rx="10" class="f-bg s-line"/><text x="476" y="196" class="t-small">lỗi bất ngờ:</text><text x="476" y="214" class="t-small">rollback, ném tiếp</text><path d="M110 120 L110 168" class="f-none s-pk dash" stroke-width="2" marker-end="url(#py68-1-arrow)"/><path d="M330 120 L330 168" class="f-none s-pk dash" stroke-width="2" marker-end="url(#py68-1-arrow)"/><path d="M550 120 L550 168" class="f-none s-pk dash" stroke-width="2" marker-end="url(#py68-1-arrow)"/></svg>
</div>
<figcaption>Hình 2. Thông điệp đi vào theo mũi tên liền, kết quả đi ra theo nét đứt ở hàng trên; ba nhánh nét đứt đi xuống là ba cách pipeline xử lý lỗi.</figcaption>
</figure>

## Thứ tự ghép là hành vi

Composition root ghép pipeline bằng một danh sách, và thứ tự trong danh sách **là** hành vi của service:

```python
# src/kyc_service/api/container.py (trích)
    # Thứ tự là hành vi: log bọc ngoài cùng (thấy cả lỗi validate), validate chạy TRƯỚC khi
    # mở transaction (thông điệp sai không tốn transaction), transaction sát handler nhất.
    mediator = Mediator(
        [logging_behavior(log), validation_behavior(), transaction_behavior(transactions)]
    )
    mediator.register(
        SubmitDriverApplication,
        SubmitDriverApplicationHandler(
            repository, ocr or FakeOcr(), face or FakeFaceMatch(), risk or FakeRiskScore(), clock
        ),
    )
    mediator.register(RecordReviewDecision, RecordReviewDecisionHandler(repository, clock))
    mediator.register(GetApplication, GetApplicationHandler(read_model))
    mediator.register(ListReviewQueue, ListReviewQueueHandler(read_model))
    return Container(mediator, repository, transactions)
```

Log ở ngoài cùng nên thấy cả lỗi validate. Hàm `build_container()` còn một việc: port nào không truyền thì dùng adapter giả **và phát `UserWarning`** nêu tên port, để một bản chạy thật quên truyền adapter không âm thầm chạy bằng bảng tra cố định. Validate đứng trước transaction nên thông điệp sai không tốn transaction. Lab chạy một command sai định dạng qua hai thứ tự (trích từ phần demo):

```text
validate -> transaction: transaction=[]
transaction -> validate: transaction=['begin', 'rollback']
```

Với `validate -> transaction` danh sách sự kiện rỗng; đảo lại thì mở transaction rồi rollback, đúng như câu hỏi đầu bài. Đây là lý do test kiểm cả `transactions.events`, không chỉ kiểm kết quả trả về.

Hai loại lỗi được xử lý khác nhau có chủ ý:

- **Lỗi lường trước** (`PortFailure`: service ngoài tắt hoặc trả dữ liệu sai miền) do handler bắt, giữ hồ sơ ở trạng thái đã đạt và trả `pending_check`. Handler không ném lên, nên transaction **commit**: hồ sơ đã nhận phải được lưu. Test: OCR tắt cho `['begin', 'commit']` và đúng một hồ sơ trong kho.
- **Lỗi bất ngờ** (bug) cứ ném lên. Transaction **rollback**, không còn hồ sơ mồ côi. Đây là ca mà bài 67 để lại hồ sơ `submitted`; ở lab này kho trống sau lỗi và log ghi `! SubmitDriverApplication RuntimeError`.

Một chi tiết về rollback: bản trong bộ nhớ hoàn lại các hàng nhưng **không** hoàn lại bộ đếm mã, giống sequence của PostgreSQL vốn không rollback. Sau một lần lỗi, hồ sơ kế tiếp là `KYC-0002`, không phải `KYC-0001`; lab có ca kiểm điều này để ai nối cơ sở dữ liệu thật không bất ngờ vì «lỗ hổng» trong dãy mã.

## So với MediatR bên C#

Bài C# dùng một dispatcher cùng ý tưởng. Đây là chỗ giống và khác với thư viện MediatR mà nhiều dự án .NET dùng:

| | MediatR (C#) | Mediator của lab (Python) |
| --- | --- | --- |
| Khai báo yêu cầu | `IRequest<TResponse>` | `Command[R]`, `Query[R]` (dataclass) |
| Handler | `IRequestHandler<TRequest, TResponse>` | lớp có `handle(message)` (Protocol `Handler`) |
| Tìm handler | qua DI container theo kiểu generic | dict `type -> pipeline`, đăng ký tay |
| Lớp bọc | `IPipelineBehavior<TRequest, TResponse>` | hàm `(message, next_step)` |
| Thứ tự lớp bọc | theo thứ tự đăng ký vào container | thứ tự danh sách truyền vào `Mediator` |
| Notification (một-nhiều) | có (`INotification`) | không; sự kiện đi bằng outbox ở bài 71 |

Python không có DI container mặc định và hàm là giá trị thường, nên cùng ý tưởng gọn hơn nhiều. Đổi lại ta tự làm phần thư viện làm hộ: thư viện .NET thường quét assembly để tự đăng ký handler, còn ở đây phải tự `register` từng cái (quên thì cả hai cách đều hỏng lúc gửi, chỉ khác nơi báo), và lab không có async, không có notification. Nếu dự án thật cân nhắc thư viện mediator, hãy xem tính năng, giấy phép và vòng đời hiện hành của nó thay vì bài này.

Cũng đừng dùng mediator khi không cần. Với bốn thông điệp như lab, gọi thẳng handler cũng chạy được; mediator đáng giá khi **nhiều handler dùng chung nghi thức** và **nhiều nơi gửi** (API, worker, CLI). Nếu chỉ cần transaction quanh vài hàm, decorator ở bài 17 là đủ.

## Chạy và kiểm chứng

Cần uv và Python 3.14; không cần Docker. Từ gốc repository:

```bash
uv run --project examples/toi-noi-python/68-cqrs-mediator-pipeline examples/toi-noi-python/68-cqrs-mediator-pipeline/main.py
```

```text
== Pipeline của một command: log -> validate -> transaction -> handler
  [log] > SubmitDriverApplication
  [log] < SubmitDriverApplication ok
  kết quả: KYC-0001 needs_review; transaction: ['begin', 'commit']
== Command sai hình dạng: dừng ở validate, không mở transaction
  [log] > SubmitDriverApplication
  [log] ! SubmitDriverApplication ValidationFailed
  lỗi: driver_id (string_pattern_mismatch)
  lỗi: cccd_ref (string_too_short)
  transaction: []; số hồ sơ đã lưu: 0
== Lỗi bất ngờ giữa chừng: rollback, không để lại hồ sơ mồ côi
  [log] > SubmitDriverApplication
  [log] ! SubmitDriverApplication RuntimeError
  RuntimeError: bug trong fraud-scoring adapter
  transaction: ['begin', 'rollback']; số hồ sơ đã lưu: 0
== Đổi thứ tự hai behavior: command sai hình dạng có mở transaction không?
  validate -> transaction: transaction=[]
  transaction -> validate: transaction=['begin', 'rollback']
== Query không mở transaction
  [log] > ListReviewQueue
  [log] < ListReviewQueue ok
  hàng chờ duyệt: ['KYC-0001']; transaction: []
```

Năm đoạn demo cho thấy: command thành công đi qua đủ ba lớp bọc; command sai hình dạng dừng ở validate với hai lỗi, không mở transaction; lỗi bất ngờ rollback và kho trống; đổi thứ tự hai lớp bọc thì thấy transaction mở thừa; query không mở transaction. Kiểm các kết quả trên:

```bash
uv run --project examples/toi-noi-python/68-cqrs-mediator-pipeline examples/toi-noi-python/68-cqrs-mediator-pipeline/main.py --verify
```

```text
OK 31/31
```

31 ca: 4 ca mediator (dispatch theo kiểu chính xác, thiếu hoặc trùng handler, thứ tự các lớp bọc, lớp bọc chặn `next`), 21 ca pipeline (log và transaction của command, query không transaction, validate gom lỗi và không lộ đầu vào, thứ tự validate–transaction, rollback khi lỗi bất ngờ, bộ đếm mã sau rollback, ba tình huống service tắt hoặc chưa đồng ý khuôn mặt, quyết định của người duyệt kể cả quyết định lần hai và hồ sơ chưa tới `needs_review`, hàng chờ duyệt, handler nhận bản đã validate, `face_consent` dạng chuỗi bị chặn, số lần dựng bộ kiểm, mã hồ sơ `KYC-10000`, cosine âm, cảnh báo adapter giả) và 6 ca qua HTTP. Chuỗi đầy đủ gồm ruff, mypy `--strict`, import-linter với ba hợp đồng, pytest và `--verify`:

```bash
sh examples/toi-noi-python/68-cqrs-mediator-pipeline/verify.sh
```

```text
== 1. ruff format --check
34 files already formatted
== 2. ruff check
All checks passed!
== 3. mypy --strict
Success: no issues found in 32 source files
== 4. import-linter
---------
Contracts
---------
Analyzed 38 files, 83 dependencies.
-----------------------------------
Bốn lớp: api > infrastructure > application > domain KEPT
Lõi không import framework web KEPT
Domain không import pydantic KEPT
Contracts: 3 kept, 0 broken.
== 5. pytest
........................................................................ [ 64%]
.......................................                                  [100%]
111 passed in 17.28s
== 6. main.py --verify
OK 31/31
OK 6/6
```

Python 3.14.7, FastAPI 0.162.0, pydantic 2.14.3, import-linter 2.16, mypy 2.4.1, pytest 9.2.0, ruff 0.17.4 (theo `uv.lock`). Kết quả OCR, điểm khuôn mặt và điểm rủi ro trong các view vẫn chỉ là đầu vào cho người duyệt: `approved` và `rejected` chỉ đến từ `needs_review` qua `RecordReviewDecision` của `RV-001`.

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| `HandlerNotFound` dù đã định nghĩa handler | Đã `register` đúng kiểu thông điệp chưa; kiểu con không dùng handler của kiểu cha |
| `RecursionError` ngay lời gọi `send` đầu tiên | `_wrap` bị thay bằng lambda trong vòng `for`: closure bắt biến, mỗi lớp phải giữ `next_step` riêng |
| Command sai vẫn mở transaction | Thứ tự trong danh sách behavior: validate phải đứng trước transaction |
| Log có dòng «vào» nhưng handler không chạy | Một behavior quên gọi `next_step(message)` (chặn lại) hoặc trả kết quả sớm |
| Rollback nhưng dữ liệu vẫn còn | Lỗi đã được handler bắt (nên commit); chỉ exception ném ra khỏi handler mới rollback |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>Mediator đồng bộ, trong một process: nó không phải message broker, không bền, không giao lại, và không tạo transaction chung với service ngoài. Gửi command trong process không đảm bảo một sự kiện được phát ra khi commit; việc đó cần outbox (bài 71). <code>InMemoryTransactions</code> chỉ chụp và hoàn lại một dict, không thay cho transaction cơ sở dữ liệu, không an toàn khi nhiều worker cùng chạy và không chống ghi đồng thời. Số liệu trong demo là dữ liệu tổng hợp.</p>
</div>

**Bài tập:** (1) Thêm behavior đo thời gian xử lý của mỗi thông điệp vào pipeline. Đặt ở đâu trong danh sách để số đo gồm cả thời gian validate nhưng không gồm thời gian ghi log? (2) `ListReviewQueue` có nên đi qua `transaction_behavior` để «cho chắc» không? Nêu một hậu quả nếu có.

<details><summary>Xem lời giải</summary><p>(1) Đặt <b>ngay sau logging</b>, trước validation: danh sách <code>[logging_behavior(log), timing_behavior(...), validation_behavior(), transaction_behavior(...)]</code>. Behavior đầu danh sách bọc ngoài cùng, nên thời gian đo gồm validate, transaction và handler nhưng không gồm lần ghi log vào và log ra. Dùng <code>time.perf_counter()</code>, ghi trong <code>finally</code> để lỗi cũng được đo. (2) Không nên. Query không đổi trạng thái; bọc transaction chỉ chiếm kết nối và mở rộng phạm vi khoá cho việc không cần, đồng thời làm mờ ranh giới command/query. Đó là lý do <code>transaction_behavior</code> kiểm <code>isinstance(message, Command)</code>.</p></details>

Bài trước: [bài 67](/articles/python-clean-architecture-domain-application-infrastructure). Bài tiếp: [bài 69](/articles/python-repository-unit-of-work-sqlalchemy).

## Tài liệu gốc

- [PEP 695: Type Parameter Syntax](https://peps.python.org/pep-0695/).
- [Python 3.14: contextlib](https://docs.python.org/3.14/library/contextlib.html).
- [Python 3.14: dataclasses](https://docs.python.org/3.14/library/dataclasses.html).
- [Pydantic: TypeAdapter](https://docs.pydantic.dev/latest/concepts/type_adapter/).
- [Azure Architecture Center: CQRS pattern](https://learn.microsoft.com/en-us/azure/architecture/patterns/cqrs).
