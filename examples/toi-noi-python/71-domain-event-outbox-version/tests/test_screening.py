"""Use case nộp hồ sơ chạy toàn luồng trên PostgreSQL thật, với adapter giả cho ba service."""

import threading

import pytest
from fastapi.testclient import TestClient
from sqlalchemy import event, text
from sqlalchemy.engine import Engine
from sqlalchemy.exc import OperationalError

from kyc_service.api.app import create_app
from kyc_service.api.container import Container, build_container
from kyc_service.application.commands import RecordReviewDecision, ResumeScreening
from kyc_service.application.errors import ApplicationNotFound, ConcurrencyConflict
from kyc_service.application.queries import GetApplication
from kyc_service.domain.errors import InvalidStatusTransition
from kyc_service.domain.kyc import DocumentImage, DocumentKind, FaceMatch, OcrResult
from kyc_service.domain.policy import ReviewPolicy
from kyc_service.domain.review import ReviewReason as R
from kyc_service.infrastructure.clocks import FixedClock
from kyc_service.infrastructure.fake_services import FakeFaceMatch, FakeOcr, FakeRiskScore

from .conftest import NOW, outbox_rows, submit_for

TABLE = [
    # tài xế, consent -> trạng thái, lý do
    ("DR-001", True, (R.ROUTINE,)),
    ("DR-002", True, (R.OCR_LOW_CONFIDENCE, R.FACE_LOW_SCORE, R.RISK_HIGH)),
    ("DR-003", False, (R.NO_FACE_CONSENT,)),
    ("DR-004", True, (R.FACE_LOW_SCORE,)),
    ("DR-005", True, (R.OCR_LOW_CONFIDENCE,)),
]


@pytest.mark.parametrize(("driver", "consent", "reasons"), TABLE)
def test_every_driver_ends_in_needs_review_with_the_expected_reasons(
    container: Container, driver: str, consent: bool, reasons: tuple[R, ...]
) -> None:
    view = container.mediator.send(submit_for(driver, consent=consent))
    assert view.status == "needs_review"
    assert view.queue_reasons == tuple(r.value for r in reasons)
    assert view.reviewed_by is None
    assert view.pending_check is None


def test_outbox_rows_exist_only_for_fully_scored_applications(
    container: Container, engine: Engine
) -> None:
    for driver, consent, _ in TABLE:
        container.mediator.send(submit_for(driver, consent=consent))
    drivers = [r["envelope"]["payload"]["driver_id"] for r in outbox_rows(engine)]
    assert drivers == ["DR-001", "DR-002", "DR-004", "DR-005"]  # DR-003 chưa đồng ý: không có điểm


def test_a_stricter_policy_adds_reasons_but_never_a_decision(engine: Engine, clean: None) -> None:
    strict = ReviewPolicy(face_min_score=0.95)
    c = build_container(
        engine, emit_version=2, clock=FixedClock(NOW), policy=strict, log=lambda _: None
    )
    view = c.mediator.send(submit_for("DR-001"))
    assert (view.status, view.queue_reasons) == ("needs_review", ("face_low_score",))


def test_only_the_reviewer_reaches_approved(container: Container) -> None:
    container.mediator.send(submit_for("DR-001"))
    done = container.mediator.send(RecordReviewDecision("KYC-0001", "RV-001", "approved", "khớp"))
    assert (done.status, done.reviewed_by) == ("approved", "RV-001")


def test_resume_after_ocr_comes_back(engine: Engine, clean: None) -> None:
    ocr = FakeOcr(available=False)
    c = build_container(engine, emit_version=2, ocr=ocr, clock=FixedClock(NOW), log=lambda _: None)
    first = c.mediator.send(submit_for("DR-001"))
    assert (first.status, first.pending_check) == ("submitted", "kyc-ocr")
    assert outbox_rows(engine) == []
    ocr.available = True
    again = c.mediator.send(ResumeScreening("KYC-0001"))
    assert (again.status, again.pending_check) == ("needs_review", None)
    assert len(outbox_rows(engine)) == 1


def test_resume_continues_from_the_step_that_stopped(engine: Engine, clean: None) -> None:
    face = FakeFaceMatch(available=False)
    ocr = FakeOcr()
    c = build_container(
        engine, emit_version=2, ocr=ocr, face=face, clock=FixedClock(NOW), log=lambda _: None
    )
    assert c.mediator.send(submit_for("DR-001")).status == "ocr_done"
    face.available = True
    c.mediator.send(ResumeScreening("KYC-0001"))
    assert ocr.calls == 2  # OCR chỉ chạy lần đầu cho hai giấy tờ, không chạy lại khi làm tiếp


def test_risk_down_stops_at_face_checked_then_resumes(engine: Engine, clean: None) -> None:
    risk = FakeRiskScore(available=False)
    c = build_container(
        engine, emit_version=2, risk=risk, clock=FixedClock(NOW), log=lambda _: None
    )
    view = c.mediator.send(submit_for("DR-001"))
    assert (view.status, view.pending_check, view.face_score) == (
        "face_checked",
        "fraud-scoring",
        0.88,
    )
    risk.available = True
    assert c.mediator.send(ResumeScreening("KYC-0001")).status == "needs_review"


def test_still_down_on_resume_stays_pending(engine: Engine, clean: None) -> None:
    c = build_container(
        engine,
        emit_version=2,
        ocr=FakeOcr(available=False),
        clock=FixedClock(NOW),
        log=lambda _: None,
    )
    c.mediator.send(submit_for("DR-001"))
    again = c.mediator.send(ResumeScreening("KYC-0001"))
    assert (again.status, again.pending_check) == ("submitted", "kyc-ocr")


def test_unknown_image_and_out_of_range_face_score_are_expected_failures(
    engine: Engine, clean: None
) -> None:
    c = build_container(engine, emit_version=2, clock=FixedClock(NOW), log=lambda _: None)
    unknown = c.mediator.send(submit_for("DR-099"))
    assert (unknown.status, unknown.pending_check) == ("submitted", "kyc-ocr")
    liar = build_container(
        engine,
        emit_version=2,
        face=FakeFaceMatch(score_override=1.5),
        clock=FixedClock(NOW),
        log=lambda _: None,
    )
    lied = liar.mediator.send(submit_for("DR-001"))
    assert (lied.status, lied.pending_check) == ("ocr_done", "kyc-face")


def test_resume_refuses_finished_applications_and_unknown_ids(container: Container) -> None:
    container.mediator.send(submit_for("DR-001"))
    with pytest.raises(InvalidStatusTransition):
        container.mediator.send(ResumeScreening("KYC-0001"))
    with pytest.raises(ApplicationNotFound):
        container.mediator.send(ResumeScreening("KYC-9999"))


class ProbingOcr(FakeOcr):
    """Trong lúc gọi kyc-ocr, hỏi PostgreSQL xem có kết nối nào đang `idle in transaction`."""

    def __init__(self, engine: Engine) -> None:
        super().__init__()
        self._engine = engine
        self.seen: list[int] = []

    def read(self, document: DocumentImage) -> OcrResult:
        with self._engine.connect() as conn:
            n = conn.execute(
                text(
                    "SELECT count(*) FROM pg_stat_activity"
                    " WHERE datname = current_database() AND pid <> pg_backend_pid()"
                    " AND state = 'idle in transaction' AND query ILIKE '%lab71%'"
                )
            ).scalar_one()
        self.seen.append(n)
        return super().read(document)


def test_the_transaction_stays_open_while_kyc_ocr_is_being_called(
    engine: Engine, clean: None
) -> None:
    ocr = ProbingOcr(engine)
    c = build_container(engine, emit_version=2, ocr=ocr, clock=FixedClock(NOW), log=lambda _: None)
    c.mediator.send(submit_for("DR-001"))
    assert ocr.seen == [1, 1]  # cả hai lần gọi OCR đều diễn ra bên trong transaction đang mở


def test_application_and_outbox_row_fail_together_when_commit_fails(
    engine: Engine, clean: None
) -> None:
    c = build_container(engine, emit_version=2, clock=FixedClock(NOW), log=lambda _: None)

    def fail_on_commit(conn: object) -> None:
        raise OperationalError("COMMIT", {}, Exception("mất kết nối lúc commit"))

    event.listen(engine, "commit", fail_on_commit)
    try:
        with pytest.raises(OperationalError):
            c.mediator.send(submit_for("DR-001"))
    finally:
        event.remove(engine, "commit", fail_on_commit)
    with engine.connect() as conn:
        rows = conn.execute(text("SELECT count(*) FROM lab71.kyc_applications")).scalar_one()
    assert rows == 0
    assert outbox_rows(engine) == []  # khác bài 70: sự kiện cũng không đi đâu cả


def stored_version(engine: Engine) -> tuple[str, int]:
    with engine.connect() as conn:
        row = conn.execute(text("SELECT status, version FROM lab71.kyc_applications")).one()
    return row[0], row[1]


class WrongKindOcr:
    """Service trả đối tượng hợp lệ về kiểu nhưng sai nghĩa: hai CCCD cho hồ sơ CCCD + GPLX."""

    def read(self, document: DocumentImage) -> OcrResult:
        return OcrResult(DocumentKind.CCCD, {"id_number": "099090000183"}, 0.9)


def test_ocr_answer_that_breaks_a_domain_rule_is_the_services_fault_not_a_rollback(
    engine: Engine, clean: None
) -> None:
    c = build_container(
        engine, emit_version=2, ocr=WrongKindOcr(), clock=FixedClock(NOW), log=lambda _: None
    )
    view = c.mediator.send(submit_for("DR-001"))
    assert (view.status, view.pending_check) == ("submitted", "kyc-ocr")
    assert stored_version(engine)[0] == "submitted"  # hồ sơ được nhận, không mất


def test_api_turns_that_service_fault_into_202_not_422(engine: Engine, clean: None) -> None:
    c = build_container(
        engine, emit_version=2, ocr=WrongKindOcr(), clock=FixedClock(NOW), log=lambda _: None
    )
    r = TestClient(create_app(c)).post(
        "/applications",
        json={
            "driver_id": "DR-001",
            "cccd_ref": "a",
            "gplx_ref": "b",
            "selfie_ref": "c",
            "face_consent": True,
        },
    )
    assert (r.status_code, r.json()["pending_check"]) == (202, "kyc-ocr")


def test_resume_while_the_service_stays_down_writes_and_calls_again_every_time(
    engine: Engine, clean: None
) -> None:
    face = FakeFaceMatch(available=False)
    c = build_container(
        engine, emit_version=2, face=face, clock=FixedClock(NOW), log=lambda _: None
    )
    c.mediator.send(submit_for("DR-001"))
    seen = [stored_version(engine)]
    for _ in range(3):
        c.mediator.send(ResumeScreening("KYC-0001"))
        seen.append(stored_version(engine))
    # Mỗi lần làm tiếp gọi service thêm một lần và ghi lại hồ sơ (version tăng) dù không có gì mới.
    assert seen == [("ocr_done", 2), ("ocr_done", 3), ("ocr_done", 4), ("ocr_done", 5)]
    assert face.calls == 4


class MeetingFace(FakeFaceMatch):
    """Cả hai lần làm tiếp phải cùng đứng trong cuộc gọi kyc-face trước khi một bên được đi tiếp."""

    def __init__(self) -> None:
        super().__init__(available=False)
        self.barrier = threading.Barrier(2, timeout=30)
        self.armed = False

    def compare(self, document: DocumentImage, selfie_ref: str) -> FaceMatch:
        result = super().compare(document, selfie_ref)
        if self.armed:
            self.barrier.wait()
        return result


def test_two_concurrent_resumes_call_the_service_twice_but_only_one_commits(
    engine: Engine, clean: None
) -> None:
    face = MeetingFace()
    c = build_container(
        engine, emit_version=2, face=face, clock=FixedClock(NOW), log=lambda _: None
    )
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
    assert sorted(outcomes) == ["ConcurrencyConflict", "needs_review"]
    assert face.calls == 2  # xung đột chỉ chặn việc GHI, không chặn việc GỌI service
    assert len(outbox_rows(engine)) == 1  # nhưng chỉ một sự kiện


def test_submitting_twice_creates_two_applications(engine: Engine, clean: None) -> None:
    """Giới hạn đã biết: chưa có khóa idempotency (bài 25), nộp lại tạo hồ sơ thứ hai."""
    c = build_container(engine, emit_version=2, clock=FixedClock(NOW), log=lambda _: None)
    first = c.mediator.send(submit_for("DR-001"))
    second = c.mediator.send(submit_for("DR-001"))
    assert (first.application_id, second.application_id) == ("KYC-0001", "KYC-0002")


class ProbingFace(FakeFaceMatch):
    def __init__(self, engine: Engine) -> None:
        super().__init__(available=False)
        self._engine = engine
        self.seen: list[int] = []

    def compare(self, document: DocumentImage, selfie_ref: str) -> FaceMatch:
        if self.available:
            with self._engine.connect() as conn:
                found = conn.execute(
                    text(
                        "SELECT count(*) FROM pg_stat_activity"
                        " WHERE datname = current_database() AND pid <> pg_backend_pid()"
                        " AND state = 'idle in transaction' AND query ILIKE '%lab71%'"
                    )
                ).scalar_one()
            self.seen.append(found)
        return super().compare(document, selfie_ref)


def test_resume_still_runs_the_whole_chain_inside_one_transaction(
    engine: Engine, clean: None
) -> None:
    face = ProbingFace(engine)
    c = build_container(
        engine, emit_version=2, face=face, clock=FixedClock(NOW), log=lambda _: None
    )
    c.mediator.send(submit_for("DR-001"))
    face.available = True
    c.mediator.send(ResumeScreening("KYC-0001"))
    assert face.seen == [1]  # đang gọi kyc-face mà transaction của lần làm tiếp vẫn mở


def test_pending_check_is_still_there_on_a_later_get(engine: Engine, clean: None) -> None:
    c = build_container(
        engine,
        emit_version=2,
        face=FakeFaceMatch(available=False),
        clock=FixedClock(NOW),
        log=lambda _: None,
    )
    c.mediator.send(submit_for("DR-001"))
    assert c.mediator.send(GetApplication("KYC-0001")).pending_check == "kyc-face"
