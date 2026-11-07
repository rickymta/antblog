"""Lab 70: use case nộp hồ sơ tài xế end-to-end trên PostgreSQL thật (schema `lab70`).

    uv run --project examples/toi-noi-python/70-use-case-ho-so-tai-xe-end-to-end \
        examples/toi-noi-python/70-use-case-ho-so-tai-xe-end-to-end/main.py [--verify]

Cần hạ tầng dùng chung:
    docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait
Mọi bảng nằm trong schema `lab70`; chạy xong schema bị xóa (trừ --keep). Ba service OCR, khuôn
mặt, rủi ro là adapter giả (bài 41-61 viết bản thật); không có ảnh nào được đọc trong lab này.
"""

import argparse
import sys
import threading
import warnings
from collections.abc import Callable
from datetime import UTC, datetime
from typing import Any

from fastapi.testclient import TestClient
from sqlalchemy import create_engine, event, text
from sqlalchemy.engine import Engine
from sqlalchemy.exc import OperationalError

from kyc_service.api.app import create_app
from kyc_service.api.container import Container, FakeAdapterWarning, build_container
from kyc_service.application.commands import (
    RecordReviewDecision,
    ResumeScreening,
    SubmitDriverApplication,
)
from kyc_service.application.errors import (
    ApplicationNotFound,
    ConcurrencyConflict,
    ValidationFailed,
)
from kyc_service.application.queries import GetApplication, ListReviewQueue
from kyc_service.domain.errors import DomainError, InvalidStatusTransition
from kyc_service.domain.events import KycApplicationScored
from kyc_service.domain.kyc import (
    DocumentImage,
    DocumentKind,
    FaceMatch,
    KycApplication,
    KycStatus,
    OcrResult,
    RiskScore,
)
from kyc_service.domain.policy import ReviewPolicy
from kyc_service.domain.review import ReviewReason as R
from kyc_service.infrastructure.clocks import FixedClock
from kyc_service.infrastructure.events import RecordingEventPublisher
from kyc_service.infrastructure.fake_services import FakeFaceMatch, FakeOcr, FakeRiskScore
from kyc_service.infrastructure.sql.schema import create_schema, drop_schema, reset_data
from kyc_service.infrastructure.sql.settings import COMPOSE_HINT, PG_URL, sqlalchemy_url

NOW = datetime(2026, 11, 6, 3, 0, tzinfo=UTC)
CLOCK = FixedClock(NOW)

# tài xế, có đồng ý khuôn mặt -> lý do vào hàng chờ mong đợi
DRIVERS: list[tuple[str, bool, tuple[R, ...]]] = [
    ("DR-001", True, (R.ROUTINE,)),
    ("DR-002", True, (R.OCR_LOW_CONFIDENCE, R.FACE_LOW_SCORE, R.RISK_HIGH)),
    ("DR-003", False, (R.NO_FACE_CONSENT,)),
    ("DR-004", True, (R.FACE_LOW_SCORE,)),
    ("DR-005", True, (R.OCR_LOW_CONFIDENCE,)),
]

# (ocr thấp nhất, khuôn mặt, rủi ro, có đồng ý) -> lý do; gồm các ca đúng ngưỡng
BOUNDARY_CASES: list[tuple[tuple[float, float | None, float | None, bool], tuple[R, ...]]] = [
    ((0.93, 0.88, 0.10, True), (R.ROUTINE,)),
    ((1.00, 1.00, 0.00, True), (R.ROUTINE,)),
    ((0.80, 0.88, 0.10, True), (R.ROUTINE,)),
    ((0.79, 0.88, 0.10, True), (R.OCR_LOW_CONFIDENCE,)),
    ((0.93, 0.75, 0.10, True), (R.ROUTINE,)),
    ((0.93, 0.74, 0.10, True), (R.FACE_LOW_SCORE,)),
    ((0.93, 0.88, 0.29, True), (R.ROUTINE,)),
    ((0.93, 0.88, 0.30, True), (R.RISK_HIGH,)),
    ((0.58, 0.71, 0.36, True), (R.OCR_LOW_CONFIDENCE, R.FACE_LOW_SCORE, R.RISK_HIGH)),
    ((0.99, None, None, False), (R.NO_FACE_CONSENT,)),
    ((0.50, None, None, False), (R.NO_FACE_CONSENT, R.OCR_LOW_CONFIDENCE)),
]


DECIDED = "status IN ('approved', 'rejected')"


BODY_070 = {
    "driver_id": "DR-001",
    "cccd_ref": "a",
    "gplx_ref": "b",
    "selfie_ref": "c",
    "face_consent": True,
}


def make_engine() -> Engine:
    engine = create_engine(sqlalchemy_url(), pool_pre_ping=True)
    try:
        with engine.connect() as conn:
            conn.execute(text("SELECT 1"))
    except OperationalError:
        print(f"FAIL: không nối được PostgreSQL tại {PG_URL} ({COMPOSE_HINT})")
        sys.exit(1)
    return engine


def submit_for(driver: str, *, consent: bool = True) -> SubmitDriverApplication:
    return SubmitDriverApplication(
        driver_id=driver,
        cccd_ref=f"synthetic/cccd/{driver}.png",
        gplx_ref=f"synthetic/gplx/{driver}.png",
        selfie_ref=f"synthetic/selfie/{driver}.png",
        face_consent=consent,
    )


def scalar(engine: Engine, sql: str) -> Any:
    with engine.connect() as conn:
        return conn.execute(text(sql)).scalar_one()


def fresh(engine: Engine, **kwargs: Any) -> Container:
    reset_data(engine)
    return build_container(engine, clock=CLOCK, log=lambda _: None, **kwargs)


def received(c: Container) -> list[Any]:
    assert isinstance(c.events, RecordingEventPublisher)
    return list(c.events.received)


def scored_application(
    ocr_min: float, face: float | None, risk: float | None, *, consent: bool
) -> KycApplication:
    app = KycApplication.submit(
        application_id="KYC-0001",
        driver_id="DR-001",
        documents=(DocumentImage(DocumentKind.CCCD, "c"), DocumentImage(DocumentKind.GPLX, "g")),
        selfie_ref="s",
        face_consent=consent,
        now=NOW,
    )
    app.record_ocr(
        (OcrResult(DocumentKind.CCCD, {}, ocr_min), OcrResult(DocumentKind.GPLX, {}, 1.0))
    )
    if face is not None:
        app.record_face(FaceMatch(face))
    if risk is not None:
        app.record_risk(RiskScore(risk))
    return app


def reasons_for(case: tuple[float, float | None, float | None, bool]) -> tuple[R, ...]:
    ocr_min, face, risk, consent = case
    return ReviewPolicy().reasons(scored_application(ocr_min, face, risk, consent=consent))


def fmt(value: float | None) -> str:
    return "-" if value is None else f"{value:.2f}"


class ProbingOcr(FakeOcr):
    """Trong lúc gọi kyc-ocr, hỏi PostgreSQL xem có kết nối nào đang `idle in transaction`."""

    def __init__(self, engine: Engine) -> None:
        super().__init__()
        self._engine = engine
        self.seen: list[int] = []

    def read(self, document: DocumentImage) -> OcrResult:
        with self._engine.connect() as conn:
            found = conn.execute(
                text(
                    "SELECT count(*) FROM pg_stat_activity"
                    " WHERE datname = current_database() AND pid <> pg_backend_pid()"
                    " AND state = 'idle in transaction' AND query ILIKE '%lab70%'"
                )
            ).scalar_one()
        self.seen.append(found)
        return super().read(document)


def commit_fails(engine: Engine) -> Callable[[], None]:
    """Gắn một hook làm COMMIT thất bại; trả hàm gỡ hook."""

    def hook(conn: object) -> None:
        raise OperationalError("COMMIT", {}, Exception("mất kết nối lúc commit"))

    event.listen(engine, "commit", hook)
    return lambda: event.remove(engine, "commit", hook)


class WrongKindOcr:
    """Service trả đối tượng hợp lệ về kiểu nhưng sai nghĩa: hai CCCD cho hồ sơ CCCD + GPLX."""

    def read(self, document: DocumentImage) -> OcrResult:
        return OcrResult(DocumentKind.CCCD, {"id_number": "099090000183"}, 0.9)


class MeetingFace(FakeFaceMatch):
    """Cả hai lần làm tiếp phải cùng đứng trong cuộc gọi kyc-face trước khi một bên đi tiếp."""

    def __init__(self) -> None:
        super().__init__(available=False)
        self.barrier = threading.Barrier(2, timeout=30)
        self.armed = False

    def compare(self, document: DocumentImage, selfie_ref: str) -> FaceMatch:
        result = super().compare(document, selfie_ref)
        if self.armed:
            self.barrier.wait()
        return result


class ProbingFace(FakeFaceMatch):
    """Trong lúc gọi kyc-face, đếm kết nối `idle in transaction` của lab."""

    def __init__(self, engine: Engine) -> None:
        super().__init__(available=False)
        self._engine = engine
        self.seen: list[int] = []

    def compare(self, document: DocumentImage, selfie_ref: str) -> FaceMatch:
        if self.available:
            with self._engine.connect() as conn:
                self.seen.append(
                    conn.execute(
                        text(
                            "SELECT count(*) FROM pg_stat_activity"
                            " WHERE datname = current_database() AND pid <> pg_backend_pid()"
                            " AND state = 'idle in transaction' AND query ILIKE '%lab70%'"
                        )
                    ).scalar_one()
                )
        return super().compare(document, selfie_ref)


def state(engine: Engine) -> tuple[str, int]:
    with engine.connect() as conn:
        row = conn.execute(text("SELECT status, version FROM lab70.kyc_applications")).one()
    return row[0], row[1]


def resume_concurrently(engine: Engine) -> tuple[list[str], int, int]:
    """Hai ResumeScreening cùng lúc: (kết quả từng bên, số lần gọi kyc-face, số event)."""
    reset_data(engine)
    face = MeetingFace()
    c = build_container(engine, face=face, clock=CLOCK, log=lambda _: None)
    c.mediator.send(submit_for("DR-001"))
    face.available, face.armed, face.calls = True, True, 0
    outcomes: list[str] = []

    def resume() -> None:
        try:
            outcomes.append(c.mediator.send(ResumeScreening("KYC-0001")).status)
        except ConcurrencyConflict:
            outcomes.append("ConcurrencyConflict")

    threads = [threading.Thread(target=resume) for _ in range(2)]
    for t in threads:
        t.start()
    for t in threads:
        t.join()
    return sorted(outcomes), face.calls, len(received(c))


def resume_while_down(engine: Engine) -> tuple[list[tuple[str, int]], int]:
    reset_data(engine)
    face = FakeFaceMatch(available=False)
    c = build_container(engine, face=face, clock=CLOCK, log=lambda _: None)
    c.mediator.send(submit_for("DR-001"))
    seen = [state(engine)]
    for _ in range(3):
        c.mediator.send(ResumeScreening("KYC-0001"))
        seen.append(state(engine))
    return seen, face.calls


def demo(engine: Engine) -> None:
    print("Lưu ý: OCR, khuôn mặt, rủi ro là adapter GIẢ (bài 41 đến 61 viết bản thật).")
    print("== 1. Năm hồ sơ qua use case thật: máy luôn dừng ở needs_review")
    c = fresh(engine)
    print("  hồ sơ   tài xế  trạng thái     ocr   khuôn mặt  rủi ro  lý do vào hàng chờ")
    for driver, consent, _ in DRIVERS:
        v = c.mediator.send(submit_for(driver, consent=consent))
        ocr = min((o.confidence for o in v.ocr), default=None)
        head = f"  {v.application_id}  {v.driver_id}  {v.status:<13}"
        scores = f"{fmt(ocr)}  {fmt(v.face_score):>9}  {fmt(v.risk_score):>6}"
        print(f"{head}  {scores}  {', '.join(v.queue_reasons)}")
    decided = scalar(engine, f"SELECT count(*) FROM lab70.kyc_applications WHERE {DECIDED}")
    print(f"  approved/rejected do máy: {decided}")

    print("== 2. Domain event KycApplicationScored (chỉ hồ sơ đã chấm điểm đủ)")
    for e in received(c):
        assert isinstance(e, KycApplicationScored)
        reasons = ",".join(r.value for r in e.review_reasons)
        print(f"  {e.application_id} {e.driver_id} rủi ro {e.risk_score:.2f} -> {reasons}")
    print(f"  tổng: {len(received(c))} event cho 5 hồ sơ (DR-003 chưa đồng ý nên không có điểm)")

    print("== 3. Ca biên của ngưỡng (ocr 0,80; khuôn mặt 0,75; rủi ro từ 0,30)")
    print("  ocr   khuôn mặt  rủi ro  đồng ý  lý do")
    for case, _ in BOUNDARY_CASES:
        ocr_min, face, risk, consent = case
        why = ", ".join(r.value for r in reasons_for(case))
        agree = "có" if consent else "không"
        print(f"  {fmt(ocr_min)}  {fmt(face):>9}  {fmt(risk):>6}  {agree:<6}  {why}")

    print("== 4. kyc-face tắt rồi sống lại: dừng ở ocr_done, ResumeScreening làm tiếp")
    face = FakeFaceMatch(available=False)
    ocr = FakeOcr()
    c4 = fresh(engine, ocr=ocr, face=face)
    first = c4.mediator.send(submit_for("DR-001"))
    print(f"  nộp: {first.status}, chờ {first.pending_check}; event: {len(received(c4))}")
    face.available = True
    again = c4.mediator.send(ResumeScreening("KYC-0001"))
    print(f"  làm tiếp: {again.status}; event: {len(received(c4))}; OCR đã gọi {ocr.calls} lần")

    print("== 5. Transaction vẫn mở trong lúc gọi kyc-ocr")
    probe = ProbingOcr(engine)
    fresh(engine, ocr=probe).mediator.send(submit_for("DR-001"))
    print(f"  kết nối idle in transaction thấy được trong hai lần gọi OCR: {probe.seen}")

    print("== 6. Event đã giao trong transaction, rồi COMMIT thất bại")
    c6 = fresh(engine)
    undo = commit_fails(engine)
    try:
        c6.mediator.send(submit_for("DR-001"))
    except OperationalError:
        print("  OperationalError lúc commit")
    finally:
        undo()
    rows = scalar(engine, "SELECT count(*) FROM lab70.kyc_applications")
    print(f"  hồ sơ trong bảng: {rows}; event đã nhận: {len(received(c6))}")

    print("== 7. Chỉ người duyệt mới tới approved")
    c7 = fresh(engine)
    c7.mediator.send(submit_for("DR-002"))
    done = c7.mediator.send(RecordReviewDecision("KYC-0001", "RV-001", "approved", "khớp mặt"))
    print(f"  {done.application_id}: {done.status} bởi {done.reviewed_by}")

    print("== 8. Làm tiếp vẫn chạy cả chuỗi trong MỘT transaction")
    reset_data(engine)
    face8 = ProbingFace(engine)
    c8 = build_container(engine, face=face8, clock=CLOCK, log=lambda _: None)
    c8.mediator.send(submit_for("DR-001"))
    face8.available = True
    c8.mediator.send(ResumeScreening("KYC-0001"))
    print(f"  kết nối idle in transaction thấy được lúc ResumeScreening gọi kyc-face: {face8.seen}")

    print("== 9. Hai ResumeScreening cùng lúc: gọi service hai lần, ghi một lần")
    outcomes, calls, events = resume_concurrently(engine)
    print(f"  kết quả: {outcomes}; kyc-face bị gọi {calls} lần; event: {events}")

    print("== 10. Làm tiếp khi service vẫn tắt: version tăng, service bị gọi mỗi lần")
    versions, face_calls = resume_while_down(engine)
    print(f"  (trạng thái, version) sau nộp và ba lần làm tiếp: {versions}")
    print(f"  kyc-face bị gọi {face_calls} lần")

    print("== 11. Nộp hai lần: hai hồ sơ (chưa có khóa idempotency)")
    c11 = fresh(engine)
    ids = [c11.mediator.send(submit_for("DR-001")).application_id for _ in range(2)]
    print(f"  mã hồ sơ: {ids}")

    print("== 12. OCR trả lời sai nghĩa: lỗi của service, hồ sơ vẫn được nhận")
    c12 = fresh(engine, ocr=WrongKindOcr())
    wrong = c12.mediator.send(submit_for("DR-001"))
    print(f"  {wrong.application_id}: {wrong.status}, chờ {wrong.pending_check}")

    print("== 13. Điểm khuôn mặt âm (cosine) tới người duyệt")
    c13 = fresh(engine, face=FakeFaceMatch(score_override=-0.4))
    neg = c13.mediator.send(submit_for("DR-001"))
    print(f"  khuôn mặt {neg.face_score}, rủi ro {neg.risk_score}, lý do {neg.queue_reasons}")


def run_verify(engine: Engine) -> int:
    cases: list[tuple[str, Callable[[], bool]]] = []

    def case(name: str) -> Callable[[Callable[[], bool]], Callable[[], bool]]:
        def register(fn: Callable[[], bool]) -> Callable[[], bool]:
            cases.append((name, fn))
            return fn

        return register

    def raises(exc: type[BaseException], fn: Callable[[], object]) -> bool:
        try:
            fn()
        except exc:
            return True
        return False

    # --- bảng ca của use case ---
    for driver, consent, reasons in DRIVERS:
        label = ", ".join(r.value for r in reasons)

        @case(f"use case {driver} (đồng ý={consent}): needs_review, lý do {label}")
        def _(
            driver: str = driver, consent: bool = consent, reasons: tuple[R, ...] = reasons
        ) -> bool:
            c = fresh(engine)
            v = c.mediator.send(submit_for(driver, consent=consent))
            return (
                v.status == "needs_review"
                and v.queue_reasons == tuple(r.value for r in reasons)
                and v.reviewed_by is None
                and v.pending_check is None
            )

    @case("máy không bao giờ tự duyệt: sau 5 hồ sơ không có approved hay rejected")
    def _() -> bool:
        c = fresh(engine)
        for driver, consent, _ in DRIVERS:
            c.mediator.send(submit_for(driver, consent=consent))
        return scalar(engine, f"SELECT count(*) FROM lab70.kyc_applications WHERE {DECIDED}") == 0

    @case("bảng ngưỡng 11 ca (gồm đúng ngưỡng): lý do khớp, và hồ sơ nào cũng vào needs_review")
    def _() -> bool:
        for case_, expected in BOUNDARY_CASES:
            if reasons_for(case_) != expected:
                return False
            ocr_min, face, risk, consent = case_
            app = scored_application(ocr_min, face, risk, consent=consent)
            app.send_to_review(ReviewPolicy().reasons(app), now=NOW)
            if app.status is not KycStatus.NEEDS_REVIEW:
                return False
        return True

    @case("ngưỡng chặt hơn chỉ thêm lý do (DR-001 thêm face_low_score), vẫn không có quyết định")
    def _() -> bool:
        reset_data(engine)
        strict = ReviewPolicy(face_min_score=0.95)
        c = build_container(engine, clock=CLOCK, policy=strict, log=lambda _: None)
        v = c.mediator.send(submit_for("DR-001"))
        return (v.status, v.queue_reasons) == ("needs_review", ("face_low_score",))

    @case("ngưỡng ngoài miền 0..1 (kể cả NaN) bị từ chối khi tạo chính sách")
    def _() -> bool:
        return raises(DomainError, lambda: ReviewPolicy(face_min_score=1.2)) and raises(
            DomainError, lambda: ReviewPolicy(risk_review_at=float("nan"))
        )

    @case("vào hàng chờ mà không có lý do nào bị domain từ chối")
    def _() -> bool:
        app = scored_application(0.9, 0.9, 0.1, consent=True)
        return raises(DomainError, lambda: app.send_to_review((), now=NOW))

    # --- domain event ---
    @case("KycApplicationScored: đúng một event, đủ số liệu và lý do")
    def _() -> bool:
        c = fresh(engine)
        c.mediator.send(submit_for("DR-002"))
        (e,) = received(c)
        return e == KycApplicationScored(
            occurred_at=NOW,
            application_id="KYC-0001",
            driver_id="DR-002",
            risk_score=0.35,
            face_score=0.71,
            ocr_min_confidence=0.58,
            review_reasons=(R.OCR_LOW_CONFIDENCE, R.FACE_LOW_SCORE, R.RISK_HIGH),
        )

    @case("không đồng ý khuôn mặt: không gọi kyc-face, không có điểm nên không có event")
    def _() -> bool:
        face = FakeFaceMatch()
        c = fresh(engine, face=face)
        v = c.mediator.send(submit_for("DR-003", consent=False))
        return face.calls == 0 and v.face_score is None and received(c) == []

    @case("event chỉ phát một lần: nạp lại hồ sơ từ cơ sở dữ liệu thì không còn event chờ")
    def _() -> bool:
        c = fresh(engine)
        c.mediator.send(submit_for("DR-001"))
        with c.uow.begin():
            loaded = c.uow.applications.get("KYC-0001")
        return loaded is not None and loaded.pull_events() == ()

    # --- service tắt và làm tiếp ---
    @case(
        "kyc-ocr tắt: dừng ở submitted, không event; sống lại thì ResumeScreening tới needs_review"
    )
    def _() -> bool:
        ocr = FakeOcr(available=False)
        c = fresh(engine, ocr=ocr)
        first = c.mediator.send(submit_for("DR-001"))
        ocr.available = True
        again = c.mediator.send(ResumeScreening("KYC-0001"))
        return (
            (first.status, first.pending_check) == ("submitted", "kyc-ocr")
            and (again.status, again.pending_check) == ("needs_review", None)
            and len(received(c)) == 1
        )

    @case("kyc-face tắt: giữ OCR ở ocr_done; làm tiếp không gọi lại OCR")
    def _() -> bool:
        face = FakeFaceMatch(available=False)
        ocr = FakeOcr()
        c = fresh(engine, ocr=ocr, face=face)
        first = c.mediator.send(submit_for("DR-001"))
        face.available = True
        c.mediator.send(ResumeScreening("KYC-0001"))
        return (first.status, len(first.ocr)) == ("ocr_done", 2) and ocr.calls == 2

    @case("fraud-scoring tắt: dừng ở face_checked giữ điểm khuôn mặt; làm tiếp tới needs_review")
    def _() -> bool:
        risk = FakeRiskScore(available=False)
        c = fresh(engine, risk=risk)
        first = c.mediator.send(submit_for("DR-001"))
        risk.available = True
        again = c.mediator.send(ResumeScreening("KYC-0001"))
        return (first.status, first.pending_check, first.face_score) == (
            "face_checked",
            "fraud-scoring",
            0.88,
        ) and again.status == "needs_review"

    @case("làm tiếp khi service vẫn tắt: vẫn pending, không đổi trạng thái")
    def _() -> bool:
        c = fresh(engine, ocr=FakeOcr(available=False))
        c.mediator.send(submit_for("DR-001"))
        again = c.mediator.send(ResumeScreening("KYC-0001"))
        return (again.status, again.pending_check) == ("submitted", "kyc-ocr")

    @case("service nói lời sai (ảnh lạ, điểm 1,5): lỗi lường trước, hồ sơ dừng chứ không hỏng")
    def _() -> bool:
        unknown = fresh(engine).mediator.send(submit_for("DR-099"))
        liar = fresh(engine, face=FakeFaceMatch(score_override=1.5))
        lied = liar.mediator.send(submit_for("DR-001"))
        return (unknown.status, unknown.pending_check) == ("submitted", "kyc-ocr") and (
            lied.status,
            lied.pending_check,
        ) == ("ocr_done", "kyc-face")

    @case("làm tiếp hồ sơ đã needs_review bị từ chối; hồ sơ lạ là ApplicationNotFound")
    def _() -> bool:
        c = fresh(engine)
        c.mediator.send(submit_for("DR-001"))
        return raises(
            InvalidStatusTransition, lambda: c.mediator.send(ResumeScreening("KYC-0001"))
        ) and raises(ApplicationNotFound, lambda: c.mediator.send(ResumeScreening("KYC-9999")))

    # --- đường biên của transaction và sự kiện ---
    @case("trong lúc gọi kyc-ocr, kết nối PostgreSQL ở trạng thái idle in transaction")
    def _() -> bool:
        probe = ProbingOcr(engine)
        fresh(engine, ocr=probe).mediator.send(submit_for("DR-001"))
        return probe.seen == [1, 1]

    @case("event đã giao trong transaction nhưng COMMIT thất bại: không hồ sơ, vẫn có event")
    def _() -> bool:
        c = fresh(engine)
        undo = commit_fails(engine)
        try:
            failed = raises(OperationalError, lambda: c.mediator.send(submit_for("DR-001")))
        finally:
            undo()
        rows = scalar(engine, "SELECT count(*) FROM lab70.kyc_applications")
        return failed and rows == 0 and len(received(c)) == 1

    @case("lý do vào hàng chờ được lưu và đọc lại đúng qua repository và read model")
    def _() -> bool:
        c = fresh(engine)
        c.mediator.send(submit_for("DR-002"))
        with c.uow.begin():
            loaded = c.uow.applications.get("KYC-0001")
        stored = scalar(engine, "SELECT queue_reasons::text FROM lab70.kyc_applications")
        view = c.mediator.send(ListReviewQueue())[0]
        return (
            loaded is not None
            and loaded.queue_reasons == (R.OCR_LOW_CONFIDENCE, R.FACE_LOW_SCORE, R.RISK_HIGH)
            and "risk_high" in stored
            and view.queue_reasons == ("ocr_low_confidence", "face_low_score", "risk_high")
        )

    @case("số CCCD đầy đủ không có trong bảng sau use case; chỉ có dạng 099*******83")
    def _() -> bool:
        fresh(engine).mediator.send(submit_for("DR-001"))
        dump = scalar(engine, "SELECT string_agg(fields::text, ' ') FROM lab70.kyc_ocr_results")
        return "099090000183" not in dump and "099*******83" in dump

    # --- người duyệt ---
    @case("RV-001 duyệt hồ sơ needs_review; quyết định lần hai bị từ chối, trạng thái giữ nguyên")
    def _() -> bool:
        c = fresh(engine)
        c.mediator.send(submit_for("DR-002"))
        done = c.mediator.send(RecordReviewDecision("KYC-0001", "RV-001", "approved", "khớp"))
        again = raises(
            InvalidStatusTransition,
            lambda: c.mediator.send(
                RecordReviewDecision("KYC-0001", "RV-001", "rejected", "đổi ý")
            ),
        )
        return (
            (done.status, done.reviewed_by) == ("approved", "RV-001")
            and again
            and c.mediator.send(GetApplication("KYC-0001")).status == "approved"
        )

    @case("approved/rejected chỉ từ needs_review: duyệt hồ sơ submitted bị từ chối")
    def _() -> bool:
        c = fresh(engine, ocr=FakeOcr(available=False))
        c.mediator.send(submit_for("DR-001"))
        return raises(
            InvalidStatusTransition,
            lambda: c.mediator.send(RecordReviewDecision("KYC-0001", "RV-001", "approved", "ép")),
        )

    # --- đường biên của ResumeScreening và của service ---
    @case("làm tiếp chạy cả chuỗi trong một transaction: kyc-face được gọi khi idle in transaction")
    def _() -> bool:
        reset_data(engine)
        face = ProbingFace(engine)
        c = build_container(engine, face=face, clock=CLOCK, log=lambda _: None)
        c.mediator.send(submit_for("DR-001"))
        face.available = True
        c.mediator.send(ResumeScreening("KYC-0001"))
        return face.seen == [1]

    @case("hai làm tiếp cùng lúc: needs_review và ConcurrencyConflict, service gọi 2 lần, 1 event")
    def _() -> bool:
        return resume_concurrently(engine) == (["ConcurrencyConflict", "needs_review"], 2, 1)

    @case("làm tiếp khi service vẫn tắt: version 2, 3, 4, 5 và service bị gọi 4 lần")
    def _() -> bool:
        versions, calls = resume_while_down(engine)
        return versions == [("ocr_done", v) for v in (2, 3, 4, 5)] and calls == 4

    @case("nộp hai lần cho cùng tài xế tạo hai hồ sơ KYC-0001, KYC-0002 (giới hạn đã biết)")
    def _() -> bool:
        c = fresh(engine)
        ids = [c.mediator.send(submit_for("DR-001")).application_id for _ in range(2)]
        return ids == ["KYC-0001", "KYC-0002"]

    @case("OCR trả hai CCCD cho một hồ sơ: lỗi của service, hồ sơ giữ ở submitted (không 422)")
    def _() -> bool:
        c = fresh(engine, ocr=WrongKindOcr())
        view = c.mediator.send(submit_for("DR-001"))
        rows = scalar(engine, "SELECT count(*) FROM lab70.kyc_applications")
        http = TestClient(create_app(fresh(engine, ocr=WrongKindOcr()))).post(
            "/applications", json={**BODY_070, "driver_id": "DR-001"}
        )
        return (
            (view.status, view.pending_check) == ("submitted", "kyc-ocr")
            and rows == 1
            and (http.status_code == 202)
        )

    @case("điểm khuôn mặt âm tới người duyệt: needs_review, face_low_score, rủi ro trong 0..1")
    def _() -> bool:
        c = fresh(engine, face=FakeFaceMatch(score_override=-0.4))
        view = c.mediator.send(submit_for("DR-001"))
        return (
            view.status == "needs_review"
            and view.face_score == -0.4
            and view.risk_score is not None
            and 0.0 <= view.risk_score <= 1.0
            and "face_low_score" in view.queue_reasons
        )

    @case("điểm khuôn mặt ngoài -1..1 là lỗi service: hồ sơ dừng ở ocr_done")
    def _() -> bool:
        view = fresh(engine, face=FakeFaceMatch(score_override=1.5)).mediator.send(
            submit_for("DR-001")
        )
        return (view.status, view.pending_check) == ("ocr_done", "kyc-face")

    @case("pending_check còn khi đọc lại hồ sơ dừng giữa chừng (GET), suy từ trạng thái")
    def _() -> bool:
        c = fresh(engine, face=FakeFaceMatch(available=False))
        c.mediator.send(submit_for("DR-001"))
        return c.mediator.send(GetApplication("KYC-0001")).pending_check == "kyc-face"

    @case("face_consent dạng chuỗi bị từ chối trước handler, không có hồ sơ nào được tạo")
    def _() -> bool:
        face = FakeFaceMatch()
        c = fresh(engine, face=face)
        raw = SubmitDriverApplication("DR-001", "a", "b", "c", "false")  # type: ignore[arg-type]
        failed = raises(ValidationFailed, lambda: c.mediator.send(raw))
        rows = scalar(engine, "SELECT count(*) FROM lab70.kyc_applications")
        return failed and rows == 0 and face.calls == 0

    @case("mã hồ sơ vượt 9999: KYC-10000 đi hết luồng tới needs_review")
    def _() -> bool:
        reset_data(engine)
        with engine.begin() as conn:
            conn.execute(text("ALTER SEQUENCE lab70.kyc_application_seq RESTART WITH 10000"))
        c = build_container(engine, clock=CLOCK, log=lambda _: None)
        view = c.mediator.send(submit_for("DR-001"))
        return (view.application_id, view.status) == ("KYC-10000", "needs_review")

    @case("build_container cảnh báo khi dùng adapter giả, im lặng khi truyền đủ ba adapter")
    def _() -> bool:
        with warnings.catch_warnings(record=True) as caught:
            warnings.simplefilter("always")
            build_container(engine, clock=CLOCK, log=lambda _: None)
            loud = [w for w in caught if issubclass(w.category, FakeAdapterWarning)]
            caught.clear()
            build_container(
                engine,
                ocr=FakeOcr(),
                face=FakeFaceMatch(),
                risk=FakeRiskScore(),
                clock=CLOCK,
                log=lambda _: None,
            )
            quiet = [w for w in caught if issubclass(w.category, FakeAdapterWarning)]
        return (
            len(loud) == 1
            and "adapter giả cho: ocr, face, risk" in str(loud[0].message)
            and not quiet
        )

    # --- api ---
    body = {
        "driver_id": "DR-002",
        "cccd_ref": "synthetic/cccd/DR-002.png",
        "gplx_ref": "synthetic/gplx/DR-002.png",
        "selfie_ref": "synthetic/selfie/DR-002.png",
        "face_consent": True,
    }

    @case("api: POST 201, queue_reasons có trong phản hồi, không lộ số giấy tờ đầy đủ")
    def _() -> bool:
        c = TestClient(create_app(fresh(engine)))
        r = c.post("/applications", json=body)
        queue = c.get("/review-queue").json()
        return (
            r.status_code == 201
            and r.json()["queue_reasons"] == ["ocr_low_confidence", "face_low_score", "risk_high"]
            and "099190000271" not in r.text
            and [a["application_id"] for a in queue] == ["KYC-0001"]
        )

    @case("api: kyc-ocr tắt -> 202; POST /resume khi đã sống lại -> 200 needs_review")
    def _() -> bool:
        ocr = FakeOcr(available=False)
        c = TestClient(create_app(fresh(engine, ocr=ocr)))
        first = c.post("/applications", json=body)
        ocr.available = True
        again = c.post("/applications/KYC-0001/resume")
        return (
            first.status_code == 202
            and again.status_code == 200
            and again.json()["status"] == "needs_review"
        )

    @case("api: resume hồ sơ đã needs_review -> 409; hồ sơ lạ -> 404; gửi status=approved -> 422")
    def _() -> bool:
        c = TestClient(create_app(fresh(engine)))
        c.post("/applications", json=body)
        done = c.post("/applications/KYC-0001/resume")
        missing = c.post("/applications/KYC-9999/resume")
        forged = c.post("/applications", json={**body, "status": "approved"})
        return (done.status_code, missing.status_code, forged.status_code) == (409, 404, 422)

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


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--verify", action="store_true", help="kiểm các kết quả in trong bài")
    parser.add_argument("--keep", action="store_true", help="giữ schema lab70 sau khi chạy")
    parser.add_argument("--cleanup", action="store_true", help="chỉ xóa schema lab70")
    args = parser.parse_args()
    warnings.simplefilter("ignore", FakeAdapterWarning)  # lab cố ý dùng adapter giả
    engine = make_engine()
    if args.cleanup:
        drop_schema(engine)
        print("đã xóa schema lab70")
        return 0
    create_schema(engine)
    try:
        if args.verify:
            return run_verify(engine)
        demo(engine)
        return 0
    finally:
        if not args.keep:
            drop_schema(engine)
        engine.dispose()


if __name__ == "__main__":
    sys.exit(main())
