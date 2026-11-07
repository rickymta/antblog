"""Repository trên PostgreSQL thật: round trip, mặt nạ PII, ràng buộc, xung đột, số câu SELECT."""

import gc

import pytest
from sqlalchemy import event, text
from sqlalchemy.engine import Engine
from sqlalchemy.exc import IntegrityError, InvalidRequestError
from sqlalchemy.orm import Session

from kyc_service.api.container import Container
from kyc_service.application.errors import ConcurrencyConflict, UnitOfWorkNotStarted
from kyc_service.application.queries import ListReviewQueue
from kyc_service.application.views import ApplicationView
from kyc_service.domain.kyc import (
    DocumentImage,
    DocumentKind,
    FaceMatch,
    KycApplication,
    KycStatus,
    OcrResult,
    RiskScore,
)
from kyc_service.domain.review import ReviewReason
from kyc_service.infrastructure.sql.tables import ApplicationRow
from kyc_service.infrastructure.sql.unit_of_work import SqlAlchemyUnitOfWork, make_session_factory

from .conftest import NOW, submit_for

pytestmark = pytest.mark.usefixtures("clean")

ID_NUMBER = "099090000183"


def new_uow(engine: Engine) -> SqlAlchemyUnitOfWork:
    return SqlAlchemyUnitOfWork(make_session_factory(engine))


def new_application(application_id: str = "KYC-0001", driver: str = "DR-001") -> KycApplication:
    return KycApplication.submit(
        application_id=application_id,
        driver_id=driver,
        documents=(DocumentImage(DocumentKind.CCCD, "c"), DocumentImage(DocumentKind.GPLX, "g")),
        selfie_ref="s",
        face_consent=True,
        now=NOW,
    )


def screened(application: KycApplication) -> KycApplication:
    application.record_ocr(
        (
            OcrResult(
                DocumentKind.CCCD, {"id_number": ID_NUMBER, "full_name": "Nguyễn Văn A"}, 0.97
            ),
            OcrResult(DocumentKind.GPLX, {"license_number": "990000000001"}, 0.93),
        )
    )
    application.record_face(FaceMatch(0.88))
    application.record_risk(RiskScore(0.1, ("ví dụ",)))
    application.send_to_review((ReviewReason.ROUTINE,), now=NOW)
    return application


def test_round_trip_keeps_the_aggregate(engine: Engine) -> None:
    uow = new_uow(engine)
    original = new_application()
    with uow.begin():
        uow.applications.add(original)  # INSERT ở trạng thái submitted
        screened(original)
        uow.applications.save(original)  # UPDATE + thay hàng con OCR
    loaded = fetch(engine, "KYC-0001")
    assert loaded is not None
    assert ApplicationView.of(loaded) == ApplicationView.of(original)
    assert loaded.documents == original.documents
    assert loaded.status is KycStatus.NEEDS_REVIEW
    assert loaded.queue_reasons == (ReviewReason.ROUTINE,)
    assert loaded.note == original.note
    assert loaded.risk_score == original.risk_score
    assert loaded.face_match == original.face_match


def test_full_id_number_never_reaches_the_database(engine: Engine) -> None:
    uow = new_uow(engine)
    with uow.begin():
        uow.applications.add(screened(new_application()))
    with engine.connect() as conn:
        stored = conn.execute(
            text("SELECT string_agg(fields::text, ' ') FROM lab70.kyc_ocr_results")
        )
        dump = stored.scalar_one()
    assert ID_NUMBER not in dump
    assert "099*******83" in dump
    loaded = fetch(engine, "KYC-0001")
    assert loaded is not None
    assert loaded.ocr_results[0].fields["id_number"] == "099*******83"


def test_get_unknown_and_save_before_add(engine: Engine) -> None:
    uow = new_uow(engine)
    with uow.begin():
        assert uow.applications.get("KYC-9999") is None
        with pytest.raises(ValueError, match="chưa được add"):
            uow.applications.save(new_application())


def test_add_twice_is_rejected(engine: Engine) -> None:
    uow = new_uow(engine)
    app = new_application()
    with pytest.raises(ValueError, match="đã tồn tại"), uow.begin():
        uow.applications.add(app)
        uow.applications.add(app)


def test_list_by_status_filters_and_orders(engine: Engine) -> None:
    uow = new_uow(engine)
    with uow.begin():
        repo = uow.applications
        for n in (3, 1, 2):
            app = new_application(f"KYC-000{n}", f"DR-00{n}")
            repo.add(app if n == 2 else screened(app))
    with uow.begin():
        queue = uow.applications.list_by_status(KycStatus.NEEDS_REVIEW)
        fresh = uow.applications.list_by_status(KycStatus.SUBMITTED)
    assert [a.application_id for a in queue] == ["KYC-0001", "KYC-0003"]
    assert [a.application_id for a in fresh] == ["KYC-0002"]


def test_next_id_follows_the_sequence_and_is_not_rolled_back(engine: Engine) -> None:
    uow = new_uow(engine)
    with pytest.raises(RuntimeError), uow.begin():
        assert uow.applications.next_id() == "KYC-0001"
        raise RuntimeError("lỗi sau khi lấy mã")
    with uow.begin():
        assert uow.applications.next_id() == "KYC-0002"  # sequence không rollback


def test_every_save_bumps_the_version(engine: Engine) -> None:
    uow = new_uow(engine)
    app = new_application()
    with uow.begin():
        uow.applications.add(app)
    for expected in (2, 3):
        with uow.begin():
            loaded = uow.applications.get("KYC-0001")
            assert loaded is not None
            uow.applications.save(loaded)
        with engine.connect() as conn:
            version = conn.execute(text("SELECT version FROM lab70.kyc_applications")).scalar_one()
        assert version == expected


def test_second_writer_gets_concurrency_conflict_and_first_wins(engine: Engine) -> None:
    first, second = new_uow(engine), new_uow(engine)
    with first.begin():
        first.applications.add(screened(new_application()))
    with second.begin():
        stale = second.applications.get("KYC-0001")  # B nạp trước
        assert stale is not None
        with first.begin():  # A nạp, quyết định, commit trước
            winner = first.applications.get("KYC-0001")
            assert winner is not None
            winner.decide(reviewer_id="RV-001", decision=KycStatus.APPROVED, reason="khớp", now=NOW)
            first.applications.save(winner)
        stale.decide(reviewer_id="RV-002", decision=KycStatus.REJECTED, reason="mờ", now=NOW)
        with pytest.raises(ConcurrencyConflict):
            second.applications.save(stale)
    final = fetch(engine, "KYC-0001")
    assert final is not None
    assert final.review is not None
    assert (final.status, final.review.reviewer_id) == (KycStatus.APPROVED, "RV-001")


def fetch(engine: Engine, application_id: str) -> KycApplication | None:
    uow = new_uow(engine)
    with uow.begin():
        return uow.applications.get(application_id)


def test_session_identity_map_forgets_unreferenced_rows(engine: Engine) -> None:
    """Lý do repository phải giữ hàng đã nạp: identity map của Session chỉ giữ tham chiếu yếu."""
    uow = new_uow(engine)
    with uow.begin():
        uow.applications.add(new_application())
    bump = text("UPDATE lab70.kyc_applications SET version = version + 1")
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


def test_database_refuses_a_decision_without_reviewer(engine: Engine) -> None:
    uow = new_uow(engine)
    with uow.begin():
        uow.applications.add(screened(new_application()))
    with pytest.raises(IntegrityError, match="ck_decision_has_reviewer"), engine.begin() as conn:
        conn.execute(text("UPDATE lab70.kyc_applications SET status = 'approved'"))


def test_child_rows_cannot_be_read_without_asking_for_them(engine: Engine) -> None:
    uow = new_uow(engine)
    with uow.begin():
        uow.applications.add(new_application())
    with Session(engine) as session:
        row = session.get(ApplicationRow, "KYC-0001")  # không có options nạp hàng con
        assert row is not None
        with pytest.raises(InvalidRequestError, match="lazy='raise'"):
            _ = row.ocr_results


def test_repository_is_unusable_outside_a_unit_of_work(engine: Engine) -> None:
    uow = new_uow(engine)
    with pytest.raises(UnitOfWorkNotStarted):
        _ = uow.applications


def count_selects(engine: Engine, action: object) -> int:
    seen: list[str] = []

    def on_execute(conn, cursor, statement, parameters, context, executemany):  # type: ignore[no-untyped-def]
        if statement.lstrip().upper().startswith("SELECT"):
            seen.append(statement)

    event.listen(engine, "before_cursor_execute", on_execute)
    try:
        assert callable(action)
        action()
    finally:
        event.remove(engine, "before_cursor_execute", on_execute)
    return len(seen)


def test_review_queue_read_model_uses_two_selects_for_five_applications(
    container: Container, engine: Engine
) -> None:
    for n in range(1, 6):
        container.mediator.send(submit_for(f"DR-00{n % 3 + 1}"))
    selects = count_selects(engine, lambda: container.mediator.send(ListReviewQueue()))
    assert selects == 2  # hàng gốc + OCR của cả danh sách; không N+1
