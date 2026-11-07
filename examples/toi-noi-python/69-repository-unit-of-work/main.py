"""Lab 69: Repository và Unit of Work trên PostgreSQL thật (schema `lab69`).

    uv run --project examples/toi-noi-python/69-repository-unit-of-work \
        examples/toi-noi-python/69-repository-unit-of-work/main.py [--verify | --cleanup | --keep]

Cần hạ tầng dùng chung:
    docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait
Mọi bảng nằm trong schema `lab69`; chạy xong schema bị xóa (trừ --keep).
"""

import argparse
import asyncio
import contextvars
import gc
import sys
import threading
import warnings
from collections.abc import Callable
from datetime import UTC, datetime
from typing import Any

from fastapi.testclient import TestClient
from pydantic import TypeAdapter
from sqlalchemy import create_engine, event, text
from sqlalchemy.engine import Engine
from sqlalchemy.exc import IntegrityError, OperationalError
from sqlalchemy.orm import Session

from kyc_service.api.app import create_app
from kyc_service.api.container import Container, FakeAdapterWarning, build_container
from kyc_service.application.behaviors import validation_behavior
from kyc_service.application.commands import RecordReviewDecision, SubmitDriverApplication
from kyc_service.application.errors import (
    ApplicationNotFound,
    ConcurrencyConflict,
    NestedUnitOfWork,
    UnitOfWorkNotStarted,
    ValidationFailed,
)
from kyc_service.application.handlers import GetApplicationHandler
from kyc_service.application.mediator import Mediator
from kyc_service.application.queries import GetApplication, ListReviewQueue
from kyc_service.domain.errors import InvalidStatusTransition
from kyc_service.domain.kyc import (
    DocumentImage,
    DocumentKind,
    FaceMatch,
    KycApplication,
    KycStatus,
    OcrResult,
    RiskScore,
)
from kyc_service.infrastructure.clocks import FixedClock
from kyc_service.infrastructure.fake_services import FakeFaceMatch, FakeOcr, FakeRiskScore
from kyc_service.infrastructure.sql.read_model import SqlAlchemyReadModel
from kyc_service.infrastructure.sql.schema import create_schema, drop_schema, reset_data
from kyc_service.infrastructure.sql.settings import COMPOSE_HINT, PG_URL, sqlalchemy_url
from kyc_service.infrastructure.sql.tables import ApplicationRow
from kyc_service.infrastructure.sql.unit_of_work import SqlAlchemyUnitOfWork, make_session_factory

NOW = datetime(2026, 11, 6, 3, 0, tzinfo=UTC)
CLOCK = FixedClock(NOW)


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


class TransactionTrace:
    """Ghi begin/commit/rollback THẬT mà SQLAlchemy báo cho kết nối PostgreSQL."""

    def __init__(self, engine: Engine) -> None:
        self.events: list[str] = []
        self._engine = engine
        self._hooks = {
            "begin": lambda conn: self.events.append("begin"),
            "commit": lambda conn: self.events.append("commit"),
            "rollback": lambda conn: self.events.append("rollback"),
        }
        for name, fn in self._hooks.items():
            event.listen(engine, name, fn)

    def close(self) -> None:
        for name, fn in self._hooks.items():
            event.remove(self._engine, name, fn)


def count_selects(engine: Engine, action: Callable[[], object]) -> int:
    seen: list[str] = []

    def on_execute(conn: Any, cursor: Any, statement: str, *rest: Any) -> None:
        if statement.lstrip().upper().startswith("SELECT"):
            seen.append(statement)

    event.listen(engine, "before_cursor_execute", on_execute)
    try:
        action()
    finally:
        event.remove(engine, "before_cursor_execute", on_execute)
    return len(seen)


def scalar(engine: Engine, sql: str) -> Any:
    with engine.connect() as conn:
        return conn.execute(text(sql)).scalar_one()


def new_uow(engine: Engine) -> SqlAlchemyUnitOfWork:
    return SqlAlchemyUnitOfWork(make_session_factory(engine))


class ExplodingRisk:
    """Lỗi lập trình bất ngờ (không phải PortFailure) trong lúc chấm điểm."""

    def score(self, application: KycApplication) -> RiskScore:
        raise RuntimeError("bug trong fraud-scoring adapter")


def fresh_container(engine: Engine, **kwargs: Any) -> Container:
    reset_data(engine)
    return build_container(engine, clock=CLOCK, log=lambda _: None, **kwargs)


def seed_review_queue(engine: Engine, count: int) -> None:
    """Thêm `count` hồ sơ needs_review trong MỘT transaction (nhanh hơn nộp từng hồ sơ)."""
    uow = new_uow(engine)
    with uow.begin():
        for _ in range(count):
            app = KycApplication.submit(
                application_id=uow.applications.next_id(),
                driver_id="DR-001",
                documents=(
                    DocumentImage(DocumentKind.CCCD, "c"),
                    DocumentImage(DocumentKind.GPLX, "g"),
                ),
                selfie_ref="s",
                face_consent=True,
                now=NOW,
            )
            app.record_ocr(
                (
                    OcrResult(DocumentKind.CCCD, {"id_number": "099090000183"}, 0.9),
                    OcrResult(DocumentKind.GPLX, {}, 0.9),
                )
            )
            app.record_face(FaceMatch(0.8))
            app.record_risk(RiskScore(0.2))
            app.send_to_review("seed")
            uow.applications.add(app)


def select_counts(engine: Engine) -> tuple[int, int]:
    """(read model, repository dựng aggregate): số câu SELECT để lấy hàng chờ duyệt."""
    c = build_container(engine, clock=CLOCK, log=lambda _: None)
    uow = new_uow(engine)

    def via_repository() -> None:
        with uow.begin():
            uow.applications.list_by_status(KycStatus.NEEDS_REVIEW)

    return (
        count_selects(engine, lambda: c.mediator.send(ListReviewQueue())),
        count_selects(engine, via_repository),
    )


BODY = {
    "driver_id": "DR-001",
    "cccd_ref": "synthetic/cccd/DR-001.png",
    "gplx_ref": "synthetic/gplx/DR-001.png",
    "selfie_ref": "synthetic/selfie/DR-001.png",
    "face_consent": True,
}
REVIEW = {"reviewer_id": "RV-001", "decision": "approved", "reason": "khớp"}


def demo(engine: Engine) -> None:
    print("Lưu ý: OCR, khuôn mặt, rủi ro là adapter GIẢ (bài 41 đến 61 viết bản thật).")
    print("== 1. Nộp hồ sơ qua pipeline: một transaction thật trên PostgreSQL")
    reset_data(engine)
    trace = TransactionTrace(engine)
    c = build_container(engine, clock=CLOCK, log=lambda line: print(f"  [log] {line}"))
    view = c.mediator.send(submit_for("DR-001"))
    print(f"  kết quả: {view.application_id} {view.status}; transaction: {trace.events}")
    for table in ("kyc_applications", "kyc_documents", "kyc_ocr_results"):
        print(f"  {table}: {scalar(engine, f'SELECT count(*) FROM lab69.{table}')} hàng")
    print(f"  version của hồ sơ: {scalar(engine, 'SELECT version FROM lab69.kyc_applications')}")
    trace.close()

    print("== 2. Số giấy tờ vào cơ sở dữ liệu ở dạng đã che")
    with engine.connect() as conn:
        row = conn.execute(
            text("SELECT fields FROM lab69.kyc_ocr_results WHERE kind = 'cccd'")
        ).scalar_one()
    print(f"  fields của CCCD trong bảng: id_number = {row['id_number']}")

    print("== 3. Lỗi bất ngờ giữa chừng: rollback, sequence để lại lỗ")
    reset_data(engine)
    trace = TransactionTrace(engine)
    boom = build_container(engine, risk=ExplodingRisk(), clock=CLOCK, log=lambda _: None)
    try:
        boom.mediator.send(submit_for("DR-001"))
    except RuntimeError as exc:
        print(f"  RuntimeError: {exc}")
    events = list(trace.events)
    trace.close()
    stored = scalar(engine, "SELECT count(*) FROM lab69.kyc_applications")
    print(f"  transaction: {events}; số hồ sơ: {stored}")
    ok = build_container(engine, clock=CLOCK, log=lambda _: None)
    print(f"  hồ sơ kế tiếp: {ok.mediator.send(submit_for('DR-001')).application_id}")

    print("== 4. Hai người duyệt cùng lúc: optimistic concurrency")
    reset_data(engine)
    first, second = new_uow(engine), new_uow(engine)
    ok.mediator.send(submit_for("DR-001"))
    with second.begin():
        stale = second.applications.get("KYC-0001")
        assert stale is not None
        with first.begin():
            winner = first.applications.get("KYC-0001")
            assert winner is not None
            winner.decide(reviewer_id="RV-001", decision=KycStatus.APPROVED, reason="khớp", now=NOW)
            first.applications.save(winner)
        version = scalar(engine, "SELECT version FROM lab69.kyc_applications")
        print(f"  RV-001 commit trước; version trong bảng: {version}")
        stale.decide(reviewer_id="RV-002", decision=KycStatus.REJECTED, reason="mờ", now=NOW)
        try:
            second.applications.save(stale)
        except ConcurrencyConflict as exc:
            print(f"  RV-002 ghi sau: ConcurrencyConflict: {exc}")
    final = scalar(engine, "SELECT status || ' bởi ' || reviewed_by FROM lab69.kyc_applications")
    print(f"  trạng thái cuối: {final}")

    print("== 5. Số câu SELECT để lấy hàng chờ duyệt, theo số hồ sơ")
    reset_data(engine)
    print("  hồ sơ  read model  repository (dựng aggregate)")
    have = 0
    for target in (5, 50, 520):
        seed_review_queue(engine, target - have)
        have = target
        read_model, repository = select_counts(engine)
        print(f"  {target:>5}  {read_model:>10}  {repository:>10}")

    print("== 6. Ràng buộc ở cơ sở dữ liệu: duyệt mà không có người duyệt")
    try:
        with engine.begin() as conn:
            conn.execute(text("UPDATE lab69.kyc_applications SET status = 'approved'"))
    except IntegrityError as exc:
        print(f"  IntegrityError: {str(exc.orig).splitlines()[0]}")

    print("== 7. Hai quyết định nối tiếp qua HTTP: domain chặn, không cần version")
    client = TestClient(create_app(fresh_container(engine)))
    client.post("/applications", json=BODY)
    first = client.post("/applications/KYC-0001/review", json=REVIEW)
    second = client.post(
        "/applications/KYC-0001/review",
        json={**REVIEW, "reviewer_id": "RV-002", "decision": "rejected"},
    )
    print(f"  RV-001: {first.status_code}; RV-002 (đến sau): {second.status_code}")
    print(f"  loại lỗi: {second.json()['type']}")

    print("== 8. Mã hồ sơ không ngừng ở KYC-9999")
    reset_data(engine)
    with engine.begin() as conn:
        conn.execute(text("ALTER SEQUENCE lab69.kyc_application_seq RESTART WITH 10000"))
    big = TestClient(create_app(build_container(engine, clock=CLOCK, log=lambda _: None)))
    created = big.post("/applications", json=BODY)
    print(f"  POST: {created.status_code} {created.headers['location']}")
    print(f"  GET:  {big.get(created.headers['location']).json()['status']}")


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

    def application(application_id: str = "KYC-0001", driver: str = "DR-001") -> KycApplication:
        return KycApplication.submit(
            application_id=application_id,
            driver_id=driver,
            documents=(
                DocumentImage(DocumentKind.CCCD, "c"),
                DocumentImage(DocumentKind.GPLX, "g"),
            ),
            selfie_ref="s",
            face_consent=True,
            now=NOW,
        )

    # --- repository ---
    @case("repository: add rồi get ở transaction khác trả lại đúng aggregate")
    def _() -> bool:
        reset_data(engine)
        c = build_container(engine, clock=CLOCK, log=lambda _: None)
        c.mediator.send(submit_for("DR-001"))
        uow = new_uow(engine)
        with uow.begin():
            loaded = uow.applications.get("KYC-0001")
        return (
            loaded is not None
            and loaded.status is KycStatus.NEEDS_REVIEW
            and loaded.driver_id == "DR-001"
            and [d.kind for d in loaded.documents] == [DocumentKind.CCCD, DocumentKind.GPLX]
            and loaded.face_match is not None
            and loaded.face_match.score == 0.88
            and len(loaded.ocr_results) == 2
        )

    @case("repository: số CCCD đầy đủ không có trong bảng, chỉ có dạng 099*******83")
    def _() -> bool:
        dump = scalar(engine, "SELECT string_agg(fields::text, ' ') FROM lab69.kyc_ocr_results")
        return "099090000183" not in dump and "099*******83" in dump

    @case(
        "repository: get hồ sơ lạ là None; save trước add là ValueError; add hai lần là ValueError"
    )
    def _() -> bool:
        uow = new_uow(engine)
        reset_data(engine)
        with uow.begin():
            missing = uow.applications.get("KYC-9999") is None
            unsaved = raises(ValueError, lambda: uow.applications.save(application()))
        twice = application()

        def add_twice() -> None:
            with uow.begin():
                uow.applications.add(twice)
                uow.applications.add(twice)

        return missing and unsaved and raises(ValueError, add_twice)

    @case("repository: list_by_status lọc đúng trạng thái và sắp theo mã hồ sơ")
    def _() -> bool:
        c = fresh_container(engine)
        for driver in ("DR-001", "DR-002", "DR-003"):
            c.mediator.send(submit_for(driver))
        c.mediator.send(RecordReviewDecision("KYC-0002", "RV-001", "rejected", "OCR mờ"))
        uow = new_uow(engine)
        with uow.begin():
            queue = uow.applications.list_by_status(KycStatus.NEEDS_REVIEW)
            rejected = uow.applications.list_by_status(KycStatus.REJECTED)
        return [a.application_id for a in queue] == ["KYC-0001", "KYC-0003"] and [
            a.application_id for a in rejected
        ] == ["KYC-0002"]

    @case("repository: mỗi lần save tăng version đúng 1")
    def _() -> bool:
        reset_data(engine)
        uow = new_uow(engine)
        with uow.begin():
            uow.applications.add(application())
        versions = [scalar(engine, "SELECT version FROM lab69.kyc_applications")]
        for _ in range(2):
            with uow.begin():
                loaded = uow.applications.get("KYC-0001")
                assert loaded is not None
                uow.applications.save(loaded)
            versions.append(scalar(engine, "SELECT version FROM lab69.kyc_applications"))
        return versions == [1, 2, 3]

    @case("hàng con: chạm vào quan hệ mà quên nạp thì lỗi ngay (lazy='raise'), không âm thầm N+1")
    def _() -> bool:
        reset_data(engine)
        uow = new_uow(engine)
        with uow.begin():
            uow.applications.add(application())
        with Session(engine) as session:
            row = session.get(ApplicationRow, "KYC-0001")
            assert row is not None
            return raises(Exception, lambda: row.ocr_results)

    @case("read model: hàng chờ 5 hồ sơ chỉ tốn 2 câu SELECT")
    def _() -> bool:
        c = fresh_container(engine)
        for n in range(1, 6):
            c.mediator.send(submit_for(f"DR-00{n % 3 + 1}"))
        return count_selects(engine, lambda: c.mediator.send(ListReviewQueue())) == 2

    # --- unit of work ---
    @case("uow: thoát khối bình thường là commit")
    def _() -> bool:
        reset_data(engine)
        uow = new_uow(engine)
        with uow.begin():
            uow.applications.add(application())
        return scalar(engine, "SELECT count(*) FROM lab69.kyc_applications") == 1

    @case("uow: exception rollback cả hai hồ sơ đã add; sequence không hoàn lại")
    def _() -> bool:
        reset_data(engine)
        uow = new_uow(engine)
        with raises_ctx(RuntimeError), uow.begin():
            uow.applications.add(application("KYC-0001", "DR-001"))
            uow.applications.add(application("KYC-0002", "DR-002"))
            uow.applications.next_id()
            raise RuntimeError("lỗi giữa chừng")
        rows = scalar(engine, "SELECT count(*) FROM lab69.kyc_applications")
        with uow.begin():
            next_id = uow.applications.next_id()
        return rows == 0 and next_id == "KYC-0002"

    @case("uow: dùng kho ngoài khối begin là UnitOfWorkNotStarted; begin lồng nhau bị chặn")
    def _() -> bool:
        uow = new_uow(engine)
        outside = raises(UnitOfWorkNotStarted, lambda: uow.applications)

        def nested() -> None:
            with uow.begin(), uow.begin():
                pass

        return outside and raises(NestedUnitOfWork, nested)

    @case("uow: kết nối được trả về pool sau cả commit lẫn rollback")
    def _() -> bool:
        uow = new_uow(engine)
        reset_data(engine)
        with uow.begin():
            uow.applications.add(application())
        with raises_ctx(RuntimeError), uow.begin():
            raise RuntimeError("x")
        return bool(engine.pool.checkedout() == 0)  # type: ignore[attr-defined]

    @case("uow: một đối tượng UoW, hai thread cùng lúc, hai Session riêng, cả hai commit")
    def _() -> bool:
        reset_data(engine)
        uow = new_uow(engine)
        barrier = threading.Barrier(2, timeout=10)
        sessions: list[int] = []
        errors: list[BaseException] = []

        def worker(app_id: str, driver: str) -> None:
            try:
                with uow.begin():
                    sessions.append(id(uow.session))
                    barrier.wait()
                    uow.applications.add(application(app_id, driver))
            except BaseException as exc:
                errors.append(exc)

        threads = [
            threading.Thread(target=worker, args=("KYC-0001", "DR-001")),
            threading.Thread(target=worker, args=("KYC-0002", "DR-002")),
        ]
        for t in threads:
            t.start()
        for t in threads:
            t.join()
        return (
            not errors
            and len(set(sessions)) == 2
            and scalar(engine, "SELECT count(*) FROM lab69.kyc_applications") == 2
        )

    # --- xung đột và ràng buộc ---
    @case("optimistic concurrency: người ghi sau nhận ConcurrencyConflict, người ghi trước thắng")
    def _() -> bool:
        reset_data(engine)
        seed = new_uow(engine)
        with seed.begin():
            seed.applications.add(application())
        first, second = new_uow(engine), new_uow(engine)
        conflict = False
        with second.begin():
            stale = second.applications.get("KYC-0001")
            assert stale is not None
            with first.begin():
                winner = first.applications.get("KYC-0001")
                assert winner is not None
                winner.send_to_review("thử")
                winner.decide(
                    reviewer_id="RV-001", decision=KycStatus.APPROVED, reason="a", now=NOW
                )
                first.applications.save(winner)
            stale.send_to_review("thử")
            stale.decide(reviewer_id="RV-002", decision=KycStatus.REJECTED, reason="b", now=NOW)
            conflict = raises(ConcurrencyConflict, lambda: second.applications.save(stale))
        who = scalar(engine, "SELECT reviewed_by FROM lab69.kyc_applications")
        return conflict and who == "RV-001"

    @case("identity map giữ tham chiếu yếu: bỏ hàng đã nạp thì get nạp lại version mới")
    def _() -> bool:
        reset_data(engine)
        seed = new_uow(engine)
        with seed.begin():
            seed.applications.add(application())
        with Session(engine) as session:
            row = session.get(ApplicationRow, "KYC-0001")
            assert row is not None
            v1 = row.version
            with engine.begin() as other:
                other.execute(text("UPDATE lab69.kyc_applications SET version = version + 1"))
            kept = session.get(ApplicationRow, "KYC-0001")
            same = kept is not None and kept.version == v1
            del row, kept
            gc.collect()
            again = session.get(ApplicationRow, "KYC-0001")
            return same and again is not None and again.version == v1 + 1

    @case("ràng buộc ở bảng: approved không có người duyệt bị cơ sở dữ liệu từ chối")
    def _() -> bool:
        c = fresh_container(engine)
        c.mediator.send(submit_for("DR-001"))
        try:
            with engine.begin() as conn:
                conn.execute(text("UPDATE lab69.kyc_applications SET status = 'approved'"))
        except IntegrityError as exc:
            return "ck_decision_has_reviewer" in str(exc.orig)
        return False

    # --- pipeline trên PostgreSQL ---
    @case(
        "pipeline: command hợp lệ là begin/commit thật; command sai hình dạng không chạm database"
    )
    def _() -> bool:
        c = fresh_container(engine)
        trace = TransactionTrace(engine)
        try:
            c.mediator.send(submit_for("DR-001"))
            ok = trace.events == ["begin", "commit"]
            trace.events.clear()
            bad = raises(
                ValidationFailed,
                lambda: c.mediator.send(SubmitDriverApplication("DR-1", "", "x", "y", True)),
            )
            return ok and bad and trace.events == []
        finally:
            trace.close()

    @case("pipeline: lỗi bất ngờ rollback, không hồ sơ mồ côi; hồ sơ kế tiếp là KYC-0002")
    def _() -> bool:
        reset_data(engine)
        boom = build_container(engine, risk=ExplodingRisk(), clock=CLOCK, log=lambda _: None)
        trace = TransactionTrace(engine)
        try:
            failed = raises(RuntimeError, lambda: boom.mediator.send(submit_for("DR-001")))
            rolled_back = trace.events[-1] == "rollback"
        finally:
            trace.close()
        empty = scalar(engine, "SELECT count(*) FROM lab69.kyc_applications") == 0
        healthy = build_container(engine, clock=CLOCK, log=lambda _: None)
        return (
            failed
            and rolled_back
            and empty
            and healthy.mediator.send(submit_for("DR-001")).application_id == "KYC-0002"
        )

    @case("pipeline: kyc-ocr tắt là lỗi lường trước, transaction commit, hồ sơ dừng ở submitted")
    def _() -> bool:
        c = fresh_container(engine, ocr=FakeOcr(available=False))
        trace = TransactionTrace(engine)
        try:
            view = c.mediator.send(submit_for("DR-001"))
            events = list(trace.events)
        finally:
            trace.close()
        return (view.status, view.pending_check) == ("submitted", "kyc-ocr") and events == [
            "begin",
            "commit",
        ]

    @case("pipeline: kyc-face tắt giữ kết quả OCR trong bảng; hồ sơ dừng ở ocr_done")
    def _() -> bool:
        c = fresh_container(engine, face=FakeFaceMatch(available=False))
        c.mediator.send(submit_for("DR-001"))
        return (
            c.mediator.send(GetApplication("KYC-0001")).status == "ocr_done"
            and scalar(engine, "SELECT count(*) FROM lab69.kyc_ocr_results") == 2
        )

    @case("pipeline: duyệt lần hai rollback, trạng thái giữ nguyên approved")
    def _() -> bool:
        c = fresh_container(engine)
        c.mediator.send(submit_for("DR-001"))
        c.mediator.send(RecordReviewDecision("KYC-0001", "RV-001", "approved", "khớp"))
        again = raises(
            InvalidStatusTransition,
            lambda: c.mediator.send(
                RecordReviewDecision("KYC-0001", "RV-001", "rejected", "đổi ý")
            ),
        )
        return again and c.mediator.send(GetApplication("KYC-0001")).status == "approved"

    # --- validation: handler nhận bản đã validate ---
    @case("validation: face_consent dạng chuỗi bị từ chối trước handler, không chạm database")
    def _() -> bool:
        c = fresh_container(engine)
        trace = TransactionTrace(engine)
        raw = SubmitDriverApplication("DR-001", "a", "b", "c", "false")  # type: ignore[arg-type]
        try:
            try:
                c.mediator.send(raw)
            except ValidationFailed as exc:
                codes = [(e.field, e.code) for e in exc.errors]
            else:
                return False
            return codes == [("face_consent", "bool_type")] and trace.events == []
        finally:
            trace.close()

    @case(
        "validation: handler nhận bản đã cắt khoảng trắng; mỗi kiểu thông điệp dựng bộ kiểm một lần"
    )
    def _() -> bool:
        padded = SubmitDriverApplication(
            "DR-001",
            "  synthetic/cccd/DR-001.png  ",
            "synthetic/gplx/DR-001.png",
            "synthetic/selfie/DR-001.png",
            True,
        )
        accepted = fresh_container(engine).mediator.send(padded).status == "needs_review"
        built: list[type] = []

        def counting(message_type: type) -> TypeAdapter[Any]:
            built.append(message_type)
            return TypeAdapter(message_type)

        mediator = Mediator([validation_behavior(counting)])
        read_model = SqlAlchemyReadModel(make_session_factory(engine))
        mediator.register(GetApplication, GetApplicationHandler(read_model))
        for _ in range(3):
            raises(ApplicationNotFound, lambda: mediator.send(GetApplication("KYC-9999")))
        return accepted and built == [GetApplication]

    # --- phạm vi Context của UoW ---
    @case("uow: task tạo trong begin() dùng chung Session, Thread trần thì UnitOfWorkNotStarted")
    def _() -> bool:
        uow = new_uow(engine)

        async def inner() -> int:
            return id(uow.session)

        async def scenario() -> list[int]:
            with uow.begin():
                return [
                    id(uow.session),
                    await asyncio.create_task(inner()),
                    await asyncio.to_thread(lambda: id(uow.session)),
                ]

        ids = asyncio.run(scenario())
        seen: list[str] = []

        def bare() -> None:
            try:
                _ = uow.session
            except UnitOfWorkNotStarted:
                seen.append("UnitOfWorkNotStarted")

        with uow.begin():
            t = threading.Thread(target=bare)
            t.start()
            t.join()
        return len(set(ids)) == 1 and seen == ["UnitOfWorkNotStarted"]

    @case("uow: thoát begin() ở Context khác là ValueError nhưng Session vẫn được đóng")
    def _() -> bool:
        uow = new_uow(engine)
        entered, leaving = contextvars.copy_context(), contextvars.copy_context()
        block = uow.begin()
        entered.run(block.__enter__)
        failed = raises(ValueError, lambda: leaving.run(block.__exit__, None, None, None))
        return failed and engine.pool.checkedout() == 0  # type: ignore[attr-defined]

    # --- phía đọc, adapter giả, khóa nối ---
    @case(
        "SELECT theo lô, không theo hồ sơ: 520 hồ sơ vẫn chỉ vài câu, read model ít hơn repository"
    )
    def _() -> bool:
        reset_data(engine)
        seed_review_queue(engine, 520)
        read_model, repository = select_counts(engine)
        return read_model < repository < 10

    @case("build_container cảnh báo khi dùng adapter giả, và im lặng khi truyền đủ ba adapter")
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

    @case("kết quả OCR lưu khóa nối sang document OCR (model_version, image_sha256)")
    def _() -> bool:
        fresh_container(engine).mediator.send(submit_for("DR-001"))
        rows = scalar(
            engine,
            "SELECT count(*) FROM lab69.kyc_ocr_results"
            " WHERE model_version = 'fake-ocr-1' AND length(image_sha256) = 64",
        )
        return rows == 2

    @case("mã hồ sơ vượt 9999: KYC-10000 được nhận qua POST và đọc lại qua GET")
    def _() -> bool:
        reset_data(engine)
        with engine.begin() as conn:
            conn.execute(text("ALTER SEQUENCE lab69.kyc_application_seq RESTART WITH 10000"))
        client = TestClient(create_app(build_container(engine, clock=CLOCK, log=lambda _: None)))
        created = client.post("/applications", json=BODY)
        got = client.get("/applications/KYC-10000")
        return (
            created.status_code == 201
            and created.headers["location"] == "/applications/KYC-10000"
            and got.status_code == 200
        )

    @case("pending_check còn đó khi đọc lại hồ sơ dừng giữa chừng (GET sau 202)")
    def _() -> bool:
        client = TestClient(create_app(fresh_container(engine, ocr=FakeOcr(available=False))))
        posted = client.post("/applications", json=BODY)
        got = client.get(posted.headers["location"])
        return posted.status_code == 202 and got.json()["pending_check"] == "kyc-ocr"

    @case("hai quyết định NỐI TIẾP qua HTTP: 200 rồi 409 do domain, không cần version")
    def _() -> bool:
        client = TestClient(create_app(fresh_container(engine)))
        client.post("/applications", json=BODY)
        first = client.post("/applications/KYC-0001/review", json=REVIEW)
        later = {**REVIEW, "reviewer_id": "RV-002", "decision": "rejected"}
        second = client.post("/applications/KYC-0001/review", json=later)
        return (
            first.status_code == 200
            and second.status_code == 409
            and second.json()["type"].endswith("invalid-status-transition")
        )

    # --- api ---
    body = BODY

    @case(
        "api: POST 201 + Location; container mới cùng database vẫn thấy hồ sơ; không lộ số đầy đủ"
    )
    def _() -> bool:
        c1 = fresh_container(engine)
        r = TestClient(create_app(c1)).post("/applications", json=body)
        c2 = TestClient(create_app(build_container(engine, clock=CLOCK, log=lambda _: None)))
        queue = c2.get("/review-queue").json()
        return (
            r.status_code == 201
            and r.headers["location"] == "/applications/KYC-0001"
            and "099090000183" not in r.text
            and [a["application_id"] for a in queue] == ["KYC-0001"]
        )

    @case("api: gửi kèm status=approved là 422 và không lưu gì")
    def _() -> bool:
        c = TestClient(create_app(fresh_container(engine)))
        r = c.post("/applications", json={**body, "status": "approved"})
        return (
            r.status_code == 422
            and scalar(engine, "SELECT count(*) FROM lab69.kyc_applications") == 0
        )

    @case("api: review hồ sơ submitted là 409; hồ sơ lạ là 404 problem+json")
    def _() -> bool:
        c = TestClient(create_app(fresh_container(engine, ocr=FakeOcr(available=False))))
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


class raises_ctx:
    """Context manager nuốt đúng một loại exception (và báo sai nếu không có exception)."""

    def __init__(self, exc: type[BaseException]) -> None:
        self._exc = exc

    def __enter__(self) -> None:
        return None

    def __exit__(
        self, exc_type: type[BaseException] | None, exc: BaseException | None, tb: object
    ) -> bool:
        return exc_type is not None and issubclass(exc_type, self._exc)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--verify", action="store_true", help="kiểm các kết quả in trong bài")
    parser.add_argument("--keep", action="store_true", help="giữ schema lab69 sau khi chạy")
    parser.add_argument("--cleanup", action="store_true", help="chỉ xóa schema lab69")
    args = parser.parse_args()
    warnings.simplefilter("ignore", FakeAdapterWarning)  # lab cố ý dùng adapter giả
    engine = make_engine()
    if args.cleanup:
        drop_schema(engine)
        print("đã xóa schema lab69")
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
