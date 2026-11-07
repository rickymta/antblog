---
slug: python-clean-architecture-domain-application-infrastructure
title: "Clean Architecture trong Python: domain, application, infrastructure, api"
summary: "Bài 67: chia service hồ sơ KYC thành bốn lớp, dùng Protocol làm port, adapter giả ở infrastructure và import-linter giữ chiều phụ thuộc, kể cả khi ta cố ý phá. Có lab chạy được."
category: python-advanced
level: advanced
tags: [python, toi-noi, clean-architecture, import-linter]
language: vi
---

Ba service Python của [Tới Nơi](/articles/du-an-mau-toi-noi) đã chạy riêng lẻ: OCR đọc CCCD và GPLX, so khớp khuôn mặt cho điểm tương đồng, fraud-scoring cho điểm rủi ro. Giờ cần một service **hồ sơ tài xế** gọi cả ba theo thứ tự rồi đưa hồ sơ vào hàng chờ người duyệt. Cách nhanh nhất là viết thẳng vào route FastAPI: nhận ảnh, gọi `pytesseract`, gọi gRPC, ghi PostgreSQL, trả JSON. Nó chạy. Rồi sprint sau ta muốn đổi engine OCR, hoặc chỉ muốn test luật «hồ sơ không có đồng ý khuôn mặt thì không gọi kyc-face», và phát hiện test nào cũng cần Tesseract, mạng và cơ sở dữ liệu.

Bài này chia service thành bốn lớp để việc đổi OCR, đổi cơ sở dữ liệu và kiểm luật nghiệp vụ không kéo nhau theo. Quan trọng hơn, ta biến ranh giới thành thứ máy kiểm được: một import sai chiều làm lệnh `lint-imports` thất bại.

Đây là bài 67 của lộ trình **Python nâng cao qua Tới Nơi**, bài mở đầu phần 09. Bạn cần quen `Protocol` và tiêm phụ thuộc ([bài 14](/articles/python-abc-protocol-composition-dependency-injection)), bố cục `src` ([bài 20](/articles/python-dong-goi-ruff-pre-commit-cau-truc-du-an)) và bảng trạng thái hồ sơ KYC ([bài 41](/articles/python-kyc-tai-xe-bai-toan-du-lieu-metric)). Bài C# cùng chủ đề là [bài 31 của loạt .NET 10](/articles/dotnet10-clean-architecture-layers).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Use case gọi OCR qua một interface, còn adapter OCR nằm ở lớp <code>infrastructure</code> và hiện thực interface đó. Vậy lớp <code>application</code> có phải <code>import</code> lớp <code>infrastructure</code> không? Và trong Python, adapter có cần kế thừa interface không?</p>
<details><summary>Xem đáp án</summary><p><b>Không cho cả hai.</b> Interface (port) do <code>application</code> khai báo; <code>infrastructure</code> import <code>application</code>, không ngược lại. Với <code>typing.Protocol</code>, adapter chỉ cần có đúng phương thức, không cần import hay kế thừa port; mypy kiểm khớp cấu trúc. Chỉ lớp ngoài cùng, <code>api</code>, biết cả hai và ghép chúng lại. Bài sẽ chạy <code>lint-imports</code> và mypy để xem máy nói gì.</p></details>
</div>

## Bốn lớp và một chiều phụ thuộc

Lab `67-clean-architecture-layers` là một package `kyc_service` theo bố cục `src` của bài 20, chia bốn thư mục con:

```text
src/kyc_service/
  domain/          errors.py  kyc.py
  application/     errors.py  ports.py  use_cases.py  views.py
  infrastructure/  clocks.py  fake_services.py  fixtures.py  memory_repository.py
  api/             app.py  container.py  schemas.py
```

| Lớp | Giữ gì | Được import |
| --- | --- | --- |
| `domain` | `KycApplication`, `KycStatus`, quy tắc chuyển trạng thái, value object `OcrResult`, `FaceMatch`, `RiskScore` | thư viện chuẩn |
| `application` | port (`Protocol`), use case, DTO `ApplicationView` | `domain` |
| `infrastructure` | adapter giả cho OCR, khuôn mặt, điểm rủi ro; kho lưu trong bộ nhớ | `application`, `domain` |
| `api` | FastAPI, Problem Details, `build_container()` | cả ba lớp trong |

Quy tắc là **mã nguồn chỉ import hướng vào trong**. Hình 1 vẽ chiều đó. Cần tách hai chiều dễ lẫn: lúc chạy, use case vẫn gọi tới adapter (đi ra ngoài); nhưng use case chỉ biết port nó tự khai báo, nên dòng `import` không bao giờ trỏ ra ngoài. Phụ thuộc mã nguồn bị đảo, lời gọi lúc chạy thì không.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Bốn lớp xếp từ ngoài vào trong: api, infrastructure, application, domain. Mũi tên import chỉ từ lớp ngoài sang lớp trong, không có mũi tên ngược."><defs><marker id="py67-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="40" width="185" height="120" rx="10" class="f-pk-soft s-pk"/><text x="36" y="72" class="t-b">api</text><text x="36" y="102" class="t-small">FastAPI, Problem</text><text x="36" y="122" class="t-small">Details, container</text><path d="M205 100 L243 100" class="f-none s-phys" stroke-width="2" marker-end="url(#py67-0-arrow)"/><rect x="245" y="40" width="185" height="120" rx="10" class="f-phys-soft s-phys"/><text x="261" y="72" class="t-b">infrastructure</text><text x="261" y="102" class="t-small">adapter giả,</text><text x="261" y="122" class="t-small">kho trong bộ nhớ</text><path d="M430 100 L468 100" class="f-none s-phys" stroke-width="2" marker-end="url(#py67-0-arrow)"/><rect x="470" y="40" width="185" height="120" rx="10" class="f-phys-soft s-phys"/><text x="486" y="72" class="t-b">application</text><text x="486" y="102" class="t-small">port (Protocol),</text><text x="486" y="122" class="t-small">use case, view</text><path d="M655 100 L693 100" class="f-none s-phys" stroke-width="2" marker-end="url(#py67-0-arrow)"/><rect x="695" y="40" width="165" height="120" rx="10" class="f-warn-soft s-warn"/><text x="711" y="72" class="t-b">domain</text><text x="711" y="102" class="t-small">KycApplication,</text><text x="711" y="122" class="t-small">KycStatus</text><path d="M120 190 L600 190" class="f-none s-muted dash" stroke-width="2" marker-end="url(#py67-0-arrow)"/><text x="120" y="222" class="t-small">api cũng import trực tiếp application và domain (bỏ qua một bậc vẫn đúng chiều)</text></svg>
</div>
<figcaption>Hình 1. Chiều import của mã nguồn: lớp ngoài biết lớp trong, lớp trong không biết lớp ngoài. Lõi (domain) không import lớp nào của service.</figcaption>
</figure>

Nếu bạn đã đọc loạt C#, đây là cùng ý với bốn project Domain, Application, Infrastructure, API; Python không có `ProjectReference` nên ranh giới chỉ là thư mục và thói quen, cho tới khi ta cho máy canh nó (mục «import-linter»).

## Domain: quy tắc không biết ai gọi nó

`domain/kyc.py` giữ bảng trạng thái của bài 41 và aggregate `KycApplication`. Hai quy tắc đáng nhìn lại: bước nào không đủ tin cậy đều có đường sang `needs_review`, và `approved`, `rejected` chỉ đến từ `needs_review`. Máy không tự duyệt hay từ chối một người. Khác bài 41, cạnh `risk_scored` → `approved` (tham số `auto_approve`) đã bị gỡ khỏi domain từ phần 09; muốn duyệt tự động về sau phải thêm lại bằng một quyết định nghiệp vụ có chủ ý.

```python
# src/kyc_service/domain/kyc.py (trích)
# Trạng thái -> các trạng thái được đi tiếp. Bước nào không đủ tin cậy đều có đường sang
# NEEDS_REVIEW; APPROVED và REJECTED chỉ đến từ NEEDS_REVIEW (máy không tự quyết định một người).
# Khác bài 41: cạnh RISK_SCORED -> APPROVED (`auto_approve`) đã bị gỡ khỏi domain từ phần 09.
ALLOWED: dict[KycStatus, frozenset[KycStatus]] = {
    S.SUBMITTED: frozenset({S.OCR_DONE, S.NEEDS_REVIEW}),
    S.OCR_DONE: frozenset({S.FACE_CHECKED, S.NEEDS_REVIEW}),
    S.FACE_CHECKED: frozenset({S.RISK_SCORED, S.NEEDS_REVIEW}),
    S.RISK_SCORED: frozenset({S.NEEDS_REVIEW}),
    S.NEEDS_REVIEW: frozenset({S.APPROVED, S.REJECTED}),
    S.APPROVED: frozenset(),
    S.REJECTED: frozenset(),
}


def advance(current: KycStatus, target: KycStatus) -> KycStatus:
    """Trả về `target` nếu chuyển hợp lệ, không thì ném InvalidStatusTransition."""
    if target not in ALLOWED[current]:
        raise InvalidStatusTransition(
            f"không được chuyển hồ sơ KYC từ {current.value} sang {target.value}"
        )
    return target
```

Aggregate đổi trạng thái qua các phương thức; mọi lần đổi đi qua `advance`, nên không có đường tắt:

```python
# src/kyc_service/domain/kyc.py (trích)
    def record_ocr(self, results: tuple[OcrResult, ...]) -> None:
        if sorted(r.kind for r in results) != sorted(d.kind for d in self.documents):
            raise DomainError("kết quả OCR phải đủ một kết quả cho mỗi giấy tờ của hồ sơ")
        self.status = advance(self.status, KycStatus.OCR_DONE)
        self.ocr_results = results

    def record_face(self, match: FaceMatch) -> None:
        self.status = advance(self.status, KycStatus.FACE_CHECKED)
        self.face_match = match

    def record_risk(self, score: RiskScore) -> None:
        self.status = advance(self.status, KycStatus.RISK_SCORED)
        self.risk_score = score

    def send_to_review(self, reason: str) -> None:
        self.status = advance(self.status, KycStatus.NEEDS_REVIEW)
        self.note = reason

    def decide(self, *, reviewer_id: str, decision: KycStatus, reason: str, now: datetime) -> None:
        """Người duyệt quyết định: chỉ NEEDS_REVIEW nhận, chỉ tới approved hoặc rejected."""
        _require_id(REVIEWER_ID, reviewer_id, "mã người duyệt")
        if decision not in (KycStatus.APPROVED, KycStatus.REJECTED):
            raise DomainError(
                f"quyết định phải là approved hoặc rejected, không phải {decision.value}"
            )
        if not reason.strip():
            raise DomainError("quyết định phải có lý do")
        self.status = advance(self.status, decision)
        self.review = ReviewDecision(reviewer_id, decision, reason, now)
```

Ba điểm cần để ý. Một hồ sơ gồm **đúng hai giấy tờ** (một CCCD, một GPLX); `submit()` từ chối hồ sơ thiếu, thừa hoặc hai giấy cùng loại. Kết quả OCR, điểm khuôn mặt, điểm rủi ro được lưu trong hồ sơ như **đầu vào cho người duyệt**: không có phương thức nào biến một điểm thành `approved`. Điểm khuôn mặt `FaceMatch` là cosine từ -1 đến 1 (miền của SFace; điểm âm là có thật), còn `RiskScore` là 0 đến 1; nguồn nào dùng thang khác (như 0 đến 100) thì adapter chia về 0 đến 1 trước khi vào domain. Mã hồ sơ nhận `KYC-` kèm từ bốn chữ số `[0-9]` trở lên, nên không ngừng nhận ở `KYC-10000`. Và domain không import gì ngoài thư viện chuẩn: không FastAPI, không pydantic, không biết «PostgreSQL» là gì. Nhờ vậy test domain (`tests/test_domain.py`, gồm cả ma trận 7 × 7 trạng thái) chạy không cần dựng thứ gì.

## Application: port là Protocol, use case chỉ biết port

Use case cần bốn thứ từ bên ngoài: đọc giấy tờ, so khuôn mặt, chấm điểm rủi ro, lưu hồ sơ. Lớp `application` khai báo chúng thành `Protocol`, đúng hình thức mà bài 14 đã dùng cho chính sách cước:

```python
# src/kyc_service/application/ports.py (trích)
class OcrPort(Protocol):
    """kyc-ocr: đọc các trường từ một ảnh giấy tờ."""

    def read(self, document: DocumentImage) -> OcrResult: ...


class FaceMatchPort(Protocol):
    """kyc-face: so ảnh chân dung trên giấy tờ với selfie."""

    def compare(self, document: DocumentImage, selfie_ref: str) -> FaceMatch: ...


class RiskScorePort(Protocol):
    """fraud-scoring: chấm điểm rủi ro cho cả hồ sơ."""

    def score(self, application: KycApplication) -> RiskScore: ...
```

`KycRepository` và `Clock` khai báo cùng cách trong `ports.py`. Điểm khác ABC: adapter không phải import hay kế thừa port. `FakeOcr` ở lớp ngoài chỉ cần có `read(document) -> OcrResult`. Nếu sai tên hoặc chữ ký, mypy ở chế độ `--strict` báo ngay tại chỗ ghép. Lab có sẵn `demo-errors/bad_adapter.py`, một adapter đặt tên phương thức là `read_document` rồi truyền vào `build_container`; `verify.sh` chạy mypy trên đúng file này và kỳ vọng đúng một lỗi:

```bash
uv run --project examples/toi-noi-python/67-clean-architecture-layers mypy --config-file examples/toi-noi-python/67-clean-architecture-layers/pyproject.toml examples/toi-noi-python/67-clean-architecture-layers/demo-errors/bad_adapter.py
```

```text
demo-errors/bad_adapter.py:12: error: Argument "ocr" to "build_container" has incompatible type "WrongOcr"; expected "OcrPort | None"  [arg-type]
Found 1 error in 1 file (checked 1 source file)
```

Use case là lớp nhận port qua constructor (tiêm phụ thuộc, bài 14) và điều phối. Nó không chứa quy tắc nghiệp vụ, vì quy tắc nằm ở domain, và không biết HTTP:

```python
# src/kyc_service/application/use_cases.py (trích)
    def execute(self, request: SubmitDriverApplicationRequest) -> ApplicationView:
        application = KycApplication.submit(
            application_id=self._repository.next_id(),
            driver_id=request.driver_id,
            documents=(
                DocumentImage(DocumentKind.CCCD, request.cccd_ref),
                DocumentImage(DocumentKind.GPLX, request.gplx_ref),
            ),
            selfie_ref=request.selfie_ref,
            face_consent=request.face_consent,
            now=self._clock.now(),
        )
        self._repository.add(application)  # nhận hồ sơ trước: service ngoài hỏng cũng không mất

        pending: str | None = None
        try:
            application.record_ocr(tuple(self._ocr.read(d) for d in application.documents))
            if application.face_consent:
                application.record_face(
                    self._face.compare(application.id_document, application.selfie_ref)
                )
                application.record_risk(self._risk.score(application))
                application.send_to_review("kết quả kiểm tra tự động là đầu vào cho người duyệt")
            else:
                # Bảng chuyển cho phép OCR_DONE -> NEEDS_REVIEW, không cho nhảy sang điểm rủi ro.
                application.send_to_review("chưa có đồng ý so khớp khuôn mặt")
        except PortFailure as failure:
            pending = failure.service  # hồ sơ dừng ở trạng thái đã đạt, chờ kiểm lại sau
        self._repository.save(application)
        return ApplicationView.of(application, pending_check=pending)
```

Ba quyết định trong đoạn này. **Nhận hồ sơ trước, kiểm sau**: `add()` chạy trước khi gọi service ngoài, nên OCR tắt không làm mất hồ sơ. **Lỗi lường trước đi qua `PortFailure`**: adapter dịch mọi lỗi của service (tắt, trả dữ liệu sai miền) về một kiểu chung có tên service; use case giữ hồ sơ ở trạng thái đã đạt và báo `pending_check`, API trả `202 Accepted` thay vì `201`. **Bảng chuyển quyết định đường đi**: nếu tài xế chưa đồng ý so khớp khuôn mặt, hồ sơ đi `ocr_done` rồi `needs_review`; use case không gọi `kyc-face` và `advance` cũng không cho nhảy sang điểm rủi ro.

Use case ở bài này cố ý mỏng: nó luôn kết thúc ở `needs_review`. Ngưỡng điểm, domain event và bảng ca nằm ở bài 70.

## Infrastructure và api: nơi duy nhất biết adapter cụ thể

Adapter làm hai việc: gọi service thật (ở lab là tra bảng cố định) và **dịch lỗi của service về ngôn ngữ của port**, để application không cần biết service nói giao thức gì.

```python
# src/kyc_service/infrastructure/fake_services.py (trích)
class FakeOcr:
    def __init__(self, *, available: bool = True) -> None:
        self.available = available
        self.calls = 0

    def read(self, document: DocumentImage) -> OcrResult:
        self.calls += 1
        if not self.available:
            raise ServiceUnavailable("kyc-ocr")
        try:
            kind, fields, confidence = OCR_FIXTURES[document.image_ref]
            if kind is not document.kind:
                raise InvalidServiceResponse(
                    "kyc-ocr", f"ảnh {document.image_ref} không phải {document.kind}"
                )
            return OcrResult(kind, fields, confidence)
        except KeyError as exc:
            raise InvalidServiceResponse("kyc-ocr", f"không có ảnh {document.image_ref}") from exc
        except DomainError as exc:
            raise InvalidServiceResponse("kyc-ocr", str(exc)) from exc
```

Adapter thật cho OCR, khuôn mặt và fraud nằm ở phần 05 đến 07; cơ sở dữ liệu thật ở bài 69. Adapter giả chỉ để chạy luồng và test: dữ liệu là tổng hợp (CCCD giả bắt đầu `099`), không phải mô hình. Kho `InMemoryKycRepository` trả **bản sao** như một cơ sở dữ liệu thật: sửa object đã nạp mà không `save` thì không có gì đổi, nên test không vô tình dựa vào việc hai biến cùng trỏ một object.

Chỗ ghép là `api/container.py`, composition root. Port nào không truyền thì dùng adapter giả **và phát cảnh báo** `UserWarning` nêu tên các port đó: một bản chạy thật quên truyền adapter OCR sẽ đọc giấy tờ bằng bảng tra cố định, nên việc dùng adapter giả không được âm thầm.

```python
# src/kyc_service/api/container.py (trích)
def build_container(
    *,
    repository: KycRepository | None = None,
    ocr: OcrPort | None = None,
    face: FaceMatchPort | None = None,
    risk: RiskScorePort | None = None,
    clock: Clock | None = None,
) -> Container:
    """Port nào không truyền thì dùng adapter giả và CẢNH BÁO; test truyền adapter riêng.

    Không fallback âm thầm: một bản chạy thật mà quên truyền adapter OCR sẽ đọc giấy tờ bằng
    bảng tra cố định, nên việc dùng adapter giả phải lộ ra.
    """
    fallbacks = [
        name
        for name, given in (
            ("repository", repository),
            ("ocr", ocr),
            ("face", face),
            ("risk", risk),
        )
        if given is None
    ]
    if fallbacks:
        warnings.warn(
            f"build_container() dùng adapter giả cho: {', '.join(fallbacks)}. "
            "Chỉ cho demo và test.",
            UserWarning,
            stacklevel=2,
        )
    repository = repository or InMemoryKycRepository()
    clock = clock or SystemClock()
    return Container(
        submit=SubmitDriverApplicationUseCase(
            repository, ocr or FakeOcr(), face or FakeFaceMatch(), risk or FakeRiskScore(), clock
        ),
        review=ReviewApplicationUseCase(repository, clock),
        get=GetApplicationUseCase(repository),
    )
```

Hình 2 vẽ cả hai chiều với `OcrPort`: use case gọi qua port, adapter thỏa port, và chỉ composition root gặp cả hai. Test muốn một service tắt thì truyền `ocr=FakeOcr(available=False)`; không patch, không mock.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 290" role="img" aria-label="Use case gọi OcrPort, FakeOcr thỏa OcrPort. Composition root trong api ghép FakeOcr vào use case lúc khởi động bằng hai mũi tên nét đứt."><defs><marker id="py67-1-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="30" width="250" height="100" rx="10" class="f-phys-soft s-phys"/><text x="36" y="62" class="t-b">Use case (application)</text><text x="36" y="92" class="t-small">gọi self._ocr.read(...)</text><text x="36" y="112" class="t-small">không biết adapter nào</text><path d="M270 80 L313 80" class="f-none s-phys" stroke-width="2" marker-end="url(#py67-1-arrow)"/><rect x="315" y="30" width="250" height="100" rx="10" class="f-warn-soft s-warn"/><text x="331" y="62" class="t-b">OcrPort (Protocol)</text><text x="331" y="92" class="t-small">read(document)</text><text x="331" y="112" class="t-small">-> OcrResult</text><path d="M610 80 L567 80" class="f-none s-phys" stroke-width="2" marker-end="url(#py67-1-arrow)"/><rect x="610" y="30" width="250" height="100" rx="10" class="f-phys-soft s-phys"/><text x="626" y="62" class="t-b">FakeOcr (infrastructure)</text><text x="626" y="92" class="t-small">thỏa port theo cấu trúc</text><text x="626" y="112" class="t-small">dịch lỗi về PortFailure</text><rect x="20" y="190" width="840" height="80" rx="10" class="f-pk-soft s-pk"/><text x="36" y="222" class="t-b">api/container.py: build_container()</text><text x="36" y="248" class="t-small">ghép adapter vào use case lúc khởi động; test ghép adapter khác bằng tham số</text><path d="M145 190 L145 132" class="f-none s-pk dash" stroke-width="2" marker-end="url(#py67-1-arrow)"/><path d="M735 190 L735 132" class="f-none s-pk dash" stroke-width="2" marker-end="url(#py67-1-arrow)"/></svg>
</div>
<figcaption>Hình 2. Use case gọi port, adapter thỏa port, hai bên chỉ gặp nhau ở chỗ ghép. Mũi tên từ use case tới port là lời gọi lúc chạy; mũi tên từ FakeOcr tới port là quan hệ «thỏa port» theo cấu trúc, không phải lời gọi; nét đứt là việc ghép lúc khởi động.</figcaption>
</figure>

Lớp `api` còn một việc: dịch use case sang HTTP. `POST /applications` trả 201 (hoặc 202 khi một service chưa kiểm được), `409` khi người duyệt quyết định sai trạng thái, `404` khi không có hồ sơ, lỗi theo Problem Details như [bài 32](/articles/python-fastapi-validation-problem-details-openapi). Body dùng `extra="forbid"`: client gửi kèm `"status": "approved"` bị `422 extra_forbidden` thay vì được tự đặt trạng thái. Số giấy tờ ra ngoài chỉ ở dạng mặt nạ `099*******83`.

## import-linter: biến ranh giới thành test

Thư mục tách đôi không ngăn ai viết `from kyc_service.infrastructure... import ...` trong domain vào lúc 11 giờ đêm. `import-linter` đọc đồ thị import của package và kiểm các hợp đồng khai báo trong `pyproject.toml`:

```toml
# pyproject.toml (trích)
# ---- Kiến trúc: import-linter đọc cấu hình ở đây -------------------------------------------
[tool.importlinter]
root_package = "kyc_service"
include_external_packages = true  # để luật "domain không import fastapi" thấy được thư viện ngoài

# Hợp đồng 1: chiều phụ thuộc. Lớp trên được import lớp dưới, không ngược lại.
[[tool.importlinter.contracts]]
name = "Bốn lớp: api > infrastructure > application > domain"
type = "layers"
containers = ["kyc_service"]
layers = ["api", "infrastructure", "application", "domain"]
# exhaustive: mọi module con trực tiếp của kyc_service phải thuộc một lớp. Thêm
# `kyc_service/workers.py` mà không xếp lớp thì hợp đồng vỡ, thay vì lặng lẽ không kiểm.
exhaustive = true

# Hợp đồng 2: lõi (domain, application) không biết framework web hay thư viện validate.
[[tool.importlinter.contracts]]
name = "Lõi không import framework"
type = "forbidden"
source_modules = ["kyc_service.domain", "kyc_service.application"]
forbidden_modules = ["fastapi", "starlette", "pydantic"]
```

Hợp đồng `layers` nói lớp trên được import lớp dưới, không ngược lại; theo tài liệu, hợp đồng này tính cả import gián tiếp qua module trung gian. Hai dòng `containers` và `exhaustive = true` đáng chú ý: chúng bắt mọi module con trực tiếp của `kyc_service` phải thuộc một trong bốn lớp (xem phần giới hạn bên dưới). Hợp đồng `forbidden` cấm lõi import `fastapi`, `starlette`, `pydantic` (cần `include_external_packages = true` để thấy thư viện ngoài). Chạy trên code thật, từ thư mục lab hoặc truyền `--config`:

```text
Analyzed 29 files, 46 dependencies.
-----------------------------------

Bốn lớp: api > infrastructure > application > domain KEPT
Lõi không import framework KEPT

Contracts: 2 kept, 0 broken.
```

Một hợp đồng chỉ có giá trị nếu nó **có thể vỡ**. Trong bản sao ở thư mục tạm, ta thêm một dòng đầu `domain/kyc.py` (import kho trong bộ nhớ) và một dòng đầu `application/use_cases.py` (`import fastapi`), rồi chạy lại:

```text
Analyzed 29 files, 48 dependencies.
-----------------------------------

Bốn lớp: api > infrastructure > application > domain BROKEN
Lõi không import framework BROKEN

Contracts: 0 kept, 2 broken.


----------------
Broken contracts
----------------

Bốn lớp: api > infrastructure > application > domain
----------------------------------------------------

kyc_service.domain is not allowed to import kyc_service.infrastructure:

- kyc_service.domain.kyc -> kyc_service.infrastructure.memory_repository (l.1)


Lõi không import framework
--------------------------

kyc_service.application is not allowed to import fastapi:

-   kyc_service.application.use_cases -> fastapi (l.1)
```

Bộ test của lab làm đúng việc đó tự động (`tests/test_architecture.py`): một test chạy trên code thật, bốn test phá có chủ ý trên bản sao rồi kỳ vọng `returncode == 1`. Nếu có ai sửa cấu hình cho hợp đồng lỏng đi, test «phải vỡ» sẽ đỏ.

Hợp đồng `layers` bản đầu của lab chỉ liệt kê bốn lớp, và khi ta thử thêm `kyc_service/workers.py` (import `api` và `infrastructure`, một module nằm ngoài bốn lớp) thì vẫn «2 kept»: module lạ lọt khỏi hợp đồng mà không ai hay. Vì vậy hợp đồng hiện dùng `containers` kèm `exhaustive = true`; cùng thử nghiệm đó cho:

```text
Analyzed 30 files, 48 dependencies.
-----------------------------------

Bốn lớp: api > infrastructure > application > domain BROKEN
Lõi không import framework KEPT

Contracts: 1 kept, 1 broken.


----------------
Broken contracts
----------------

Bốn lớp: api > infrastructure > application > domain
----------------------------------------------------

The following modules are not listed as layers:

- kyc_service.workers

(Since this contract is marked as 'exhaustive', every child of every container 
must be declared as a layer.)
```

Muốn thêm một nơi chạy mới (worker, CLI) ta phải xếp nó vào một lớp, thường là cùng lớp `api` như một composition root khác. Giới hạn cần nói thẳng: import-linter chỉ thấy **câu lệnh import**. Nó không thấy domain gọi `datetime.now()` (phụ thuộc đồng hồ qua thư viện chuẩn), không thấy use case nhận `dict` thay vì kiểu domain, và không biết adapter có dịch lỗi đúng hay không. Phần đó vẫn cần test hành vi.

## Chạy và kiểm chứng

Cần uv và Python 3.14; không cần Docker, không cần mạng sau lần resolve đầu. Từ gốc repository:

```bash
uv run --project examples/toi-noi-python/67-clean-architecture-layers examples/toi-noi-python/67-clean-architecture-layers/main.py
```

Demo chạy ba hồ sơ với giờ cố định (trích, `face` là điểm tương đồng, `risk` là điểm rủi ro, cả hai chỉ để người duyệt tham khảo):

```text
== Ba hồ sơ, mọi service sẵn sàng
KYC-0001 DR-001 needs_review id=099*******83   face=0.88 risk=0.09
    ghi chú: kết quả kiểm tra tự động là đầu vào cho người duyệt
KYC-0002 DR-002 needs_review id=099*******71   face=0.71 risk=0.35
    lý do rủi ro: OCR độ tin cậy thấp (0.58)
    lý do rủi ro: điểm khuôn mặt thấp (0.71)
    ghi chú: kết quả kiểm tra tự động là đầu vào cho người duyệt
KYC-0003 DR-003 needs_review id=099*******45   face=-    risk=-   
    ghi chú: chưa có đồng ý so khớp khuôn mặt
== Người duyệt RV-001 quyết định KYC-0001, rồi thử quyết định lần hai
KYC-0001 DR-001 approved     id=099*******83   face=0.88 risk=0.09
    ghi chú: kết quả kiểm tra tự động là đầu vào cho người duyệt
    bị từ chối: không được chuyển hồ sơ KYC từ approved sang rejected
== kyc-ocr tắt: hồ sơ vẫn được nhận
KYC-0001 DR-001 submitted    id=-              face=-    risk=-    pending=kyc-ocr
```

KYC-0002 có độ tin cậy OCR thấp và điểm khuôn mặt thấp nên hai lý do xuất hiện, nhưng hồ sơ vẫn chỉ vào hàng chờ; KYC-0003 chưa đồng ý khuôn mặt nên `face` và `risk` trống. Dòng cuối cho thấy OCR tắt vẫn nhận hồ sơ. Lệnh kiểm các kết quả trên:

```bash
uv run --project examples/toi-noi-python/67-clean-architecture-layers examples/toi-noi-python/67-clean-architecture-layers/main.py --verify
```

```text
OK 23/23
```

23 ca gồm: ba hồ sơ đều dừng ở `needs_review`, mặt nạ số giấy tờ, không đồng ý khuôn mặt thì `kyc-face` không được gọi, ba kiểu service tắt hoặc nói dối (OCR tắt, face tắt, face trả điểm 1,7), điểm cosine âm vẫn tới người duyệt, cảnh báo adapter giả, thứ tự trạng thái (`approved` chỉ từ `needs_review`, quyết định lần hai bị từ chối), đầu vào nói dối (ảnh GPLX nộp vào ô CCCD, `status` tự đặt, hai CCCD) và 6 ca qua HTTP (thêm ca `face_consent` gửi dạng chuỗi `"false"` bị từ chối thay vì đoán). Cả chuỗi ruff, mypy `--strict`, mypy phải bắt adapter sai chữ ký, import-linter, pytest và `--verify` nằm trong `verify.sh`:

```bash
sh examples/toi-noi-python/67-clean-architecture-layers/verify.sh
```

```text
== 1. ruff format --check
26 files already formatted
== 2. ruff check
All checks passed!
== 3. mypy --strict
Success: no issues found in 24 source files
== 4. mypy phải bắt adapter sai chữ ký (demo-errors)
examples/toi-noi-python/67-clean-architecture-layers/demo-errors/bad_adapter.py:12: error: Argument "ocr" to "build_container" has incompatible type "WrongOcr"; expected "OcrPort | None"  [arg-type]
Found 1 error in 1 file (checked 1 source file)
== 5. import-linter
---------
Contracts
---------
Analyzed 29 files, 46 dependencies.
-----------------------------------
Bốn lớp: api > infrastructure > application > domain KEPT
Lõi không import framework KEPT
Contracts: 2 kept, 0 broken.
== 6. pytest
........................................................................ [ 74%]
.........................                                                [100%]
97 passed in 24.77s
== 7. main.py --verify
OK 23/23
OK 7/7
```

Python 3.14.7, FastAPI 0.162.0, pydantic 2.14.3, import-linter 2.16, mypy 2.4.1, pytest 9.2.0, ruff 0.17.4 (theo `uv.lock`).

Một ca trong `--verify` ghi nhận **giới hạn của lab này**: khi risk adapter ném lỗi lập trình bất ngờ (không phải `PortFailure`), hồ sơ `KYC-0001` đã được `add()` và còn lại ở `submitted` trong kho, không có kết quả OCR. Bốn lớp sạch không tự cho ta transaction. Bài 68 thêm nó dưới dạng một behavior quanh mỗi command.

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| `lint-imports` báo `Could not read any configuration.` | Chạy từ thư mục lab, hoặc truyền `--config <lab>/pyproject.toml` khi đứng ở gốc repository |
| mypy báo `expected "OcrPort \| None"` | Tên phương thức hoặc chữ ký của adapter khác port |
| Hợp đồng `layers` báo `not listed as layers` | Một module mới ở gốc `kyc_service` chưa xếp vào lớp nào; đặt nó vào một lớp hoặc khai báo thêm lớp |
| Hợp đồng `layers` vỡ ở một import «vô hại» | Đọc chuỗi import mà báo cáo in ra; import gián tiếp qua module trung gian cũng bị tính |
| `forbidden` không bắt được `import fastapi` | Đã bật `include_external_packages = true` chưa |
| Test dùng nhầm kho: sửa object mà không thấy đổi | Kho trả bản sao; gọi `save()` sau khi sửa |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>Bốn lớp không đảm bảo thiết kế tốt: một service nhỏ làm đúng một việc đôi khi không cần cả bốn thư mục, và chia lớp quá sớm chỉ thêm chi phí ánh xạ giữa các kiểu. Lab dùng adapter giả tra bảng cố định và kho trong bộ nhớ: không bền, không an toàn khi chạy nhiều worker (mỗi worker có kho riêng, hồ sơ nộp ở worker này không thấy ở worker kia). Điểm số trong demo là số giả để chạy luồng, không phải kết quả của mô hình và không nên đọc thành tỉ lệ production. Quyết định cuối cùng về hồ sơ luôn là của người duyệt.</p>
</div>

**Bài tập:** (1) Thêm một `Notifier` Protocol vào `application/ports.py` để use case báo cho tài xế khi hồ sơ vào hàng chờ; adapter thật gửi SMS. Lớp nào được import gì, và hợp đồng nào cần sửa? (2) Một đồng nghiệp muốn `domain/kyc.py` dùng `datetime.now(UTC)` cho `submitted_at` cho tiện. `lint-imports` có báo không? Vì sao lab truyền `now` vào `submit()`?

<details><summary>Xem lời giải</summary><p>(1) <code>Notifier</code> là port nên nằm trong <code>application/ports.py</code>; adapter SMS nằm trong <code>infrastructure/</code> và import <code>application</code> (nếu cần dùng <code>PortFailure</code>); chỉ <code>api/container.py</code> ghép chúng. Không hợp đồng nào phải sửa: <code>layers</code> và <code>forbidden</code> đã bao mọi module trong từng lớp; chỉ khi thêm một lớp mới ngang hàng với bốn lớp này mới phải khai báo thêm. (2) <b>Không báo</b>: <code>datetime</code> là thư viện chuẩn, import-linter không có gì để cấm. Nhưng domain mất tính xác định, test phải patch đồng hồ. Truyền <code>now</code> từ use case (lấy từ port <code>Clock</code>) giữ domain thuần và cho test dùng <code>FixedClock</code>.</p></details>

Bài trước: [bài 66](/articles/python-notebook-den-production-code). Bài tiếp: [bài 68](/articles/python-cqrs-command-query-handler-mediator).

## Tài liệu gốc

- [Import Linter: Layers](https://import-linter.readthedocs.io/en/stable/contract_types/layers/).
- [Import Linter: Forbidden](https://import-linter.readthedocs.io/en/stable/contract_types/forbidden/).
- [Python 3.14: typing.Protocol](https://docs.python.org/3.14/library/typing.html#typing.Protocol).
- [PEP 544: Protocols, structural subtyping](https://peps.python.org/pep-0544/).
- [uv: build backend](https://docs.astral.sh/uv/concepts/build-backend/).
- [FastAPI: Bigger Applications](https://fastapi.tiangolo.com/tutorial/bigger-applications/).
