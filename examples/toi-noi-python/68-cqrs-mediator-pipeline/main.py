"""Lab 68: CQRS và mediator tự viết. Chạy demo, hoặc `--verify` để kiểm các kết quả trong bài.

    uv run --project examples/toi-noi-python/68-cqrs-mediator-pipeline \
        examples/toi-noi-python/68-cqrs-mediator-pipeline/main.py [--verify]
"""

import sys
import warnings
from collections.abc import Callable
from contextlib import suppress
from dataclasses import dataclass
from datetime import UTC, datetime
from typing import Any

from fastapi.testclient import TestClient
from pydantic import TypeAdapter

from kyc_service.api.app import create_app
from kyc_service.api.container import Container, build_container
from kyc_service.application.behaviors import transaction_behavior, validation_behavior
from kyc_service.application.commands import RecordReviewDecision, SubmitDriverApplication
from kyc_service.application.errors import ApplicationNotFound, ValidationFailed
from kyc_service.application.handlers import (
    GetApplicationHandler,
    SubmitDriverApplicationHandler,
)
from kyc_service.application.mediator import (
    Behavior,
    DuplicateHandler,
    HandlerNotFound,
    Mediator,
    Next,
)
from kyc_service.application.messages import Command, Message, Query
from kyc_service.application.queries import GetApplication, ListReviewQueue
from kyc_service.domain.errors import InvalidStatusTransition
from kyc_service.domain.kyc import KycApplication, RiskScore
from kyc_service.infrastructure.clocks import FixedClock
from kyc_service.infrastructure.fake_services import (
    FakeFaceMatch,
    FakeOcr,
    FakeRiskScore,
)
from kyc_service.infrastructure.memory_read_model import InMemoryReadModel
from kyc_service.infrastructure.memory_repository import InMemoryKycRepository
from kyc_service.infrastructure.memory_transactions import InMemoryTransactions

# Demo cố ý dùng adapter giả nên tắt cảnh báo của build_container(); một ca --verify riêng kiểm
# rằng cảnh báo vẫn được phát khi không tắt.
warnings.filterwarnings("ignore", message=r"build_container\(\) dùng adapter giả")

CLOCK = FixedClock(datetime(2026, 11, 6, 3, 0, tzinfo=UTC))


def submit_for(driver: str, *, consent: bool = True) -> SubmitDriverApplication:
    return SubmitDriverApplication(
        driver_id=driver,
        cccd_ref=f"synthetic/cccd/{driver}.png",
        gplx_ref=f"synthetic/gplx/{driver}.png",
        selfie_ref=f"synthetic/selfie/{driver}.png",
        face_consent=consent,
    )


class ExplodingRisk:
    """Lỗi lập trình bất ngờ (không phải PortFailure) trong lúc chấm điểm."""

    def score(self, application: KycApplication) -> RiskScore:
        raise RuntimeError("bug trong fraud-scoring adapter")


def transaction_events_for_invalid_command(*, transaction_first: bool) -> list[str]:
    """Gửi một command sai hình dạng qua pipeline hai behavior, theo hai thứ tự."""
    repo = InMemoryKycRepository()
    transactions = InMemoryTransactions(repo)
    behaviors = [validation_behavior(), transaction_behavior(transactions)]
    if transaction_first:
        behaviors.reverse()
    mediator = Mediator(behaviors)
    handler = SubmitDriverApplicationHandler(
        repo, FakeOcr(), FakeFaceMatch(), FakeRiskScore(), CLOCK
    )
    mediator.register(SubmitDriverApplication, handler)
    with suppress(ValidationFailed):
        mediator.send(SubmitDriverApplication("DR-1", "", "x", "y", True))
    return transactions.events


def demo() -> None:
    print("== Pipeline của một command: log -> validate -> transaction -> handler")
    c = build_container(clock=CLOCK, log=lambda line: print(f"  [log] {line}"))
    view = c.mediator.send(submit_for("DR-001"))
    print(f"  kết quả: {view.application_id} {view.status}; transaction: {c.transactions.events}")

    print("== Command sai hình dạng: dừng ở validate, không mở transaction")
    c2 = build_container(clock=CLOCK, log=lambda line: print(f"  [log] {line}"))
    try:
        c2.mediator.send(SubmitDriverApplication("DR-1", "", "x", "y", True))
    except ValidationFailed as exc:
        for e in exc.errors:
            print(f"  lỗi: {e.field} ({e.code})")
    print(f"  transaction: {c2.transactions.events}; số hồ sơ đã lưu: {c2.repository.count()}")

    print("== Lỗi bất ngờ giữa chừng: rollback, không để lại hồ sơ mồ côi")
    c3 = build_container(
        risk=ExplodingRisk(), clock=CLOCK, log=lambda line: print(f"  [log] {line}")
    )
    try:
        c3.mediator.send(submit_for("DR-001"))
    except RuntimeError as exc:
        print(f"  RuntimeError: {exc}")
    print(f"  transaction: {c3.transactions.events}; số hồ sơ đã lưu: {c3.repository.count()}")

    print("== Đổi thứ tự hai behavior: command sai hình dạng có mở transaction không?")
    for first, label in ((False, "validate -> transaction"), (True, "transaction -> validate")):
        events = transaction_events_for_invalid_command(transaction_first=first)
        print(f"  {label}: transaction={events}")

    print("== Query không mở transaction")
    c.transactions.events.clear()
    queue = c.mediator.send(ListReviewQueue())
    print(
        f"  hàng chờ duyệt: {[v.application_id for v in queue]};"
        f" transaction: {c.transactions.events}"
    )


@dataclass(frozen=True)
class Ping(Query[str]):
    text: str


@dataclass(frozen=True)
class LoudPing(Ping):
    """Kiểu con của Ping, không có handler riêng."""


@dataclass(frozen=True)
class Bump(Command[int]):
    by: int


class PingHandler:
    def handle(self, message: Ping) -> str:
        return f"pong:{message.text}"


class BumpHandler:
    def __init__(self, trace: list[str]) -> None:
        self._trace = trace

    def handle(self, message: Bump) -> int:
        self._trace.append("handler")
        return message.by + 1


def recorder(name: str, trace: list[str]) -> Behavior:
    def behavior(message: Message[Any], next_step: Next) -> Any:
        trace.append(f"{name}:vào")
        try:
            return next_step(message)
        finally:
            trace.append(f"{name}:ra")

    return behavior


def run_verify() -> int:
    cases: list[tuple[str, Callable[[], bool]]] = []

    def case(name: str) -> Callable[[Callable[[], bool]], Callable[[], bool]]:
        def register(fn: Callable[[], bool]) -> Callable[[], bool]:
            cases.append((name, fn))
            return fn

        return register

    def raises(exc: type[Exception], fn: Callable[[], object]) -> bool:
        try:
            fn()
        except exc:
            return True
        return False

    def fresh(events: list[str] | None = None, **kwargs: Any) -> Container:
        sink = events.append if events is not None else (lambda _: None)
        return build_container(clock=CLOCK, log=sink, **kwargs)

    # --- mediator ---
    @case("dispatch theo kiểu chính xác; kiểu con không rơi về handler của lớp cha")
    def _() -> bool:
        m = Mediator()
        m.register(Ping, PingHandler())
        return m.send(Ping("a")) == "pong:a" and raises(
            HandlerNotFound, lambda: m.send(LoudPing("a"))
        )

    @case("thông điệp chưa đăng ký: HandlerNotFound; đăng ký hai lần: DuplicateHandler")
    def _() -> bool:
        m = Mediator()
        first = raises(HandlerNotFound, lambda: m.send(Ping("a")))
        m.register(Ping, PingHandler())
        return first and raises(DuplicateHandler, lambda: m.register(Ping, PingHandler()))

    @case("behavior đầu danh sách bọc ngoài cùng: A, B, C, handler, C, B, A")
    def _() -> bool:
        trace: list[str] = []
        m = Mediator([recorder("A", trace), recorder("B", trace), recorder("C", trace)])
        m.register(Bump, BumpHandler(trace))
        m.send(Bump(1))
        return trace == ["A:vào", "B:vào", "C:vào", "handler", "C:ra", "B:ra", "A:ra"]

    @case("behavior không gọi next thì handler không chạy")
    def _() -> bool:
        trace: list[str] = []
        m = Mediator([lambda message, next_step: -1])
        m.register(Bump, BumpHandler(trace))
        return m.send(Bump(1)) == -1 and trace == []

    # --- pipeline thật ---
    @case("command: log vào/ra, transaction begin/commit")
    def _() -> bool:
        events: list[str] = []
        c = fresh(events)
        c.mediator.send(submit_for("DR-001"))
        return events == [
            "> SubmitDriverApplication",
            "< SubmitDriverApplication ok",
        ] and c.transactions.events == [
            "begin",
            "commit",
        ]

    @case("query không mở transaction")
    def _() -> bool:
        c = fresh()
        c.mediator.send(submit_for("DR-001"))
        c.transactions.events.clear()
        c.mediator.send(GetApplication("KYC-0001"))
        return c.transactions.events == []

    @case("command sai hình dạng: gom 2 lỗi, không transaction, không lưu, log thấy lỗi")
    def _() -> bool:
        events: list[str] = []
        c = fresh(events)
        try:
            c.mediator.send(SubmitDriverApplication("DR-1", "", "x", "y", True))
        except ValidationFailed as exc:
            fields = [e.field for e in exc.errors]
        else:
            return False
        return (
            fields == ["driver_id", "cccd_ref"]
            and c.transactions.events == []
            and c.repository.count() == 0
            and events[-1] == "! SubmitDriverApplication ValidationFailed"
        )

    @case("validate ngoài transaction: command sai không mở transaction; đảo lại thì có")
    def _() -> bool:
        return transaction_events_for_invalid_command(transaction_first=False) == [] and (
            transaction_events_for_invalid_command(transaction_first=True) == ["begin", "rollback"]
        )

    @case("handler nhận bản đã validate: khoảng trắng thừa quanh đường dẫn ảnh đã được cắt")
    def _() -> bool:
        raw = SubmitDriverApplication(
            "DR-001",
            "  synthetic/cccd/DR-001.png  ",
            "synthetic/gplx/DR-001.png",
            "synthetic/selfie/DR-001.png",
            True,
        )
        return fresh().mediator.send(raw).status == "needs_review"

    @case('face_consent="false" dạng chuỗi bị validation chặn, handler không chạy')
    def _() -> bool:
        c = fresh()
        raw = SubmitDriverApplication(
            "DR-001",
            "synthetic/cccd/DR-001.png",
            "synthetic/gplx/DR-001.png",
            "synthetic/selfie/DR-001.png",
            "false",  # type: ignore[arg-type]
        )
        try:
            c.mediator.send(raw)
        except ValidationFailed as exc:
            codes = [(e.field, e.code) for e in exc.errors]
        else:
            return False
        return codes == [("face_consent", "bool_type")] and c.repository.count() == 0

    @case("validation dựng TypeAdapter đúng một lần cho mỗi kiểu thông điệp (3 lần gửi)")
    def _() -> bool:
        built: list[type] = []

        def counting(message_type: type) -> TypeAdapter[Any]:
            built.append(message_type)
            return TypeAdapter(message_type)

        c = fresh()
        m = Mediator([validation_behavior(counting)])
        m.register(GetApplication, GetApplicationHandler(InMemoryReadModel(c.repository)))
        for _ in range(3):
            raises(ApplicationNotFound, lambda: m.send(GetApplication("KYC-9999")))
        return built == [GetApplication]

    @case("mã hồ sơ KYC-10000 vẫn hợp lệ về hình dạng (không ngừng ở KYC-9999)")
    def _() -> bool:
        return raises(
            ApplicationNotFound, lambda: fresh().mediator.send(GetApplication("KYC-10000"))
        )

    @case("kyc-face trả cosine âm -0.4: tới người duyệt, điểm rủi ro nằm trong 0..1")
    def _() -> bool:
        view = fresh(face=FakeFaceMatch(score_override=-0.4)).mediator.send(submit_for("DR-001"))
        return view.face_score == -0.4 and view.risk_score is not None and 0 <= view.risk_score <= 1

    @case("build_container() không truyền adapter: cảnh báo dùng adapter giả")
    def _() -> bool:
        with warnings.catch_warnings(record=True) as caught:
            warnings.simplefilter("always")
            build_container(clock=CLOCK)
        return any("adapter giả cho: repository, ocr, face, risk" in str(w.message) for w in caught)

    @case("lỗi validation không chứa giá trị đầu vào")
    def _() -> bool:
        c = fresh()
        try:
            c.mediator.send(submit_for("BI-MAT-12345"))
        except ValidationFailed as exc:
            return "BI-MAT-12345" not in repr(exc.errors)
        return False

    @case("lỗi bất ngờ: rollback, không hồ sơ mồ côi (lab 67 còn để lại)")
    def _() -> bool:
        c = fresh(risk=ExplodingRisk())
        return (
            raises(RuntimeError, lambda: c.mediator.send(submit_for("DR-001")))
            and c.repository.count() == 0
            and c.transactions.events == ["begin", "rollback"]
        )

    @case("rollback không hoàn lại bộ đếm mã: hồ sơ kế tiếp là KYC-0002")
    def _() -> bool:
        repo = InMemoryKycRepository()
        boom = fresh(repository=repo, risk=ExplodingRisk())
        raises(RuntimeError, lambda: boom.mediator.send(submit_for("DR-001")))
        ok = fresh(repository=repo)
        return ok.mediator.send(submit_for("DR-001")).application_id == "KYC-0002"

    @case("kyc-ocr tắt: lỗi đã lường trước được handler xử lý, transaction commit")
    def _() -> bool:
        c = fresh(ocr=FakeOcr(available=False))
        view = c.mediator.send(submit_for("DR-001"))
        return (
            (view.status, view.pending_check) == ("submitted", "kyc-ocr")
            and c.transactions.events == ["begin", "commit"]
            and c.repository.count() == 1
        )

    @case("kyc-face tắt: giữ kết quả OCR, dừng ở ocr_done")
    def _() -> bool:
        view = fresh(face=FakeFaceMatch(available=False)).mediator.send(submit_for("DR-001"))
        return (view.status, view.pending_check, len(view.ocr)) == ("ocr_done", "kyc-face", 2)

    @case("không đồng ý khuôn mặt: kyc-face không được gọi, vẫn vào needs_review")
    def _() -> bool:
        face = FakeFaceMatch()
        view = fresh(face=face).mediator.send(submit_for("DR-003", consent=False))
        return face.calls == 0 and view.status == "needs_review" and view.face_score is None

    @case(
        "người duyệt approved từ needs_review; quyết định lần hai rollback, trạng thái giữ nguyên"
    )
    def _() -> bool:
        c = fresh()
        c.mediator.send(submit_for("DR-001"))
        ok = c.mediator.send(RecordReviewDecision("KYC-0001", "RV-001", "approved", "khớp"))
        c.transactions.events.clear()
        second = raises(
            InvalidStatusTransition,
            lambda: c.mediator.send(
                RecordReviewDecision("KYC-0001", "RV-001", "rejected", "đổi ý")
            ),
        )
        return (
            ok.status == "approved"
            and second
            and c.transactions.events == ["begin", "rollback"]
            and c.mediator.send(GetApplication("KYC-0001")).status == "approved"
        )

    @case("approved/rejected chỉ từ needs_review: hồ sơ submitted bị từ chối")
    def _() -> bool:
        c = fresh(ocr=FakeOcr(available=False))
        c.mediator.send(submit_for("DR-001"))
        return raises(
            InvalidStatusTransition,
            lambda: c.mediator.send(
                RecordReviewDecision("KYC-0001", "RV-001", "approved", "ép duyệt")
            ),
        )

    @case("quyết định ngoài approved/rejected bị validation chặn")
    def _() -> bool:
        c = fresh()
        c.mediator.send(submit_for("DR-001"))
        bad = RecordReviewDecision("KYC-0001", "RV-001", "needs_review", "x")  # type: ignore[arg-type]
        return raises(ValidationFailed, lambda: c.mediator.send(bad))

    @case("hồ sơ không tồn tại: ApplicationNotFound")
    def _() -> bool:
        return raises(
            ApplicationNotFound, lambda: fresh().mediator.send(GetApplication("KYC-9999"))
        )

    @case("hàng chờ duyệt chỉ có needs_review, theo mã hồ sơ")
    def _() -> bool:
        c = fresh()
        for driver in ("DR-001", "DR-002", "DR-003"):
            c.mediator.send(submit_for(driver))
        c.mediator.send(RecordReviewDecision("KYC-0002", "RV-001", "rejected", "OCR mờ"))
        return [v.application_id for v in c.mediator.send(ListReviewQueue())] == [
            "KYC-0001",
            "KYC-0003",
        ]

    # --- api ---
    body = {
        "driver_id": "DR-001",
        "cccd_ref": "synthetic/cccd/DR-001.png",
        "gplx_ref": "synthetic/gplx/DR-001.png",
        "selfie_ref": "synthetic/selfie/DR-001.png",
        "face_consent": True,
    }

    def api(**kwargs: Any) -> TestClient:
        return TestClient(create_app(fresh(**kwargs)))

    @case("api: POST -> 201, Location, không lộ số giấy tờ đầy đủ; GET và review-queue thấy hồ sơ")
    def _() -> bool:
        c = api()
        r = c.post("/applications", json=body)
        queue = c.get("/review-queue").json()
        return (
            r.status_code == 201
            and r.headers["location"] == "/applications/KYC-0001"
            and "099090000183" not in r.text
            and [a["application_id"] for a in queue] == ["KYC-0001"]
        )

    @case("api: kyc-ocr tắt -> 202 Accepted")
    def _() -> bool:
        r = api(ocr=FakeOcr(available=False)).post("/applications", json=body)
        return r.status_code == 202 and r.json()["pending_check"] == "kyc-ocr"

    @case('api: face_consent="false" dạng chuỗi -> 422 bool_type')
    def _() -> bool:
        r = api().post("/applications", json={**body, "face_consent": "false"})
        return r.status_code == 422 and r.json()["errors"][0]["code"] == "bool_type"

    @case("api: gửi kèm status=approved -> 422 extra_forbidden")
    def _() -> bool:
        r = api().post("/applications", json={**body, "status": "approved"})
        return r.status_code == 422 and r.json()["errors"][0]["code"] == "extra_forbidden"

    @case("api: mã tài xế sai -> 422 do behavior validation, field body.driver_id")
    def _() -> bool:
        r = api().post("/applications", json={**body, "driver_id": "driver-1"})
        e = r.json()["errors"][0]
        return r.status_code == 422 and (e["field"], e["code"]) == (
            "body.driver_id",
            "string_pattern_mismatch",
        )

    @case("api: review hồ sơ submitted -> 409; hồ sơ lạ -> 404 problem+json")
    def _() -> bool:
        c = api(ocr=FakeOcr(available=False))
        c.post("/applications", json=body)
        review = {"reviewer_id": "RV-001", "decision": "approved", "reason": "ép"}
        conflict = c.post("/applications/KYC-0001/review", json=review)
        missing = c.get("/applications/KYC-9999")
        return (
            conflict.status_code == 409
            and conflict.json()["type"].endswith("invalid-status-transition")
            and missing.status_code == 404
            and missing.headers["content-type"].startswith("application/problem+json")
        )

    passed = 0
    for name, fn in cases:
        try:
            ok = fn()
        except Exception as exc:  # ca kiểm lỗi: in lý do thay vì dừng cả bộ
            print(f"SAI: {name} ({type(exc).__name__}: {exc})")
            continue
        if ok:
            passed += 1
        else:
            print(f"SAI: {name}")
    if passed == len(cases):
        print(f"OK {passed}/{len(cases)}")
        return 0
    print(f"FAIL {passed}/{len(cases)}")
    return 1


if __name__ == "__main__":
    if "--verify" in sys.argv[1:]:
        sys.exit(run_verify())
    demo()
