"""Lab 71: domain event, outbox, dispatcher Kafka, version hợp đồng (schema `lab71`).

    uv run --project examples/toi-noi-python/71-domain-event-outbox-version \
        examples/toi-noi-python/71-domain-event-outbox-version/main.py [--verify] [--no-csharp]

Cần hạ tầng dùng chung (PostgreSQL và Kafka):
    docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait
Consumer C# cần .NET SDK 10 (`dotnet`). Mọi bảng nằm trong schema `lab71` và mọi topic bắt đầu
bằng `lab71.`; chạy xong cả hai bị xóa (trừ --keep). Dữ liệu là dữ liệu giả.
"""

import argparse
import asyncio
import json
import shutil
import subprocess
import sys
import uuid
import warnings
from collections import Counter
from collections.abc import Callable
from datetime import UTC, datetime, timedelta
from pathlib import Path
from typing import Any

from consumers.scored_consumer import (
    SUPPORTED_BY_NEW_CONSUMER,
    SUPPORTED_BY_OLD_CONSUMER,
    EnvelopeError,
    decode,
)
from fastapi.testclient import TestClient
from sqlalchemy import create_engine, event, text
from sqlalchemy.engine import Engine
from sqlalchemy.exc import OperationalError

from kyc_service.api.app import create_app
from kyc_service.api.container import Container, FakeAdapterWarning, build_container
from kyc_service.application.commands import ResumeScreening, SubmitDriverApplication
from kyc_service.application.errors import UnitOfWorkNotStarted
from kyc_service.application.integration_events import UnmappedDomainEvent, scored_envelope
from kyc_service.domain.events import DomainEvent, KycApplicationScored
from kyc_service.domain.kyc import KycApplication, RiskScore
from kyc_service.domain.review import ReviewReason as R
from kyc_service.infrastructure.clocks import FixedClock, ManualClock
from kyc_service.infrastructure.fake_services import FakeOcr
from kyc_service.infrastructure.ids import SequentialIds
from kyc_service.infrastructure.kafka.admin import (
    check_kafka,
    create_topic,
    delete_lab_topics,
    make_producer,
    read_all,
)
from kyc_service.infrastructure.kafka.dispatcher import (
    OutboxDispatcher,
    SimulatedCrash,
    claim_batch,
)
from kyc_service.infrastructure.kafka.settings import COMPOSE_HINT as KAFKA_HINT
from kyc_service.infrastructure.kafka.settings import KAFKA_BOOTSTRAP, topic_name
from kyc_service.infrastructure.sql.outbox import OutboxEventPublisher
from kyc_service.infrastructure.sql.schema import create_schema, drop_schema, reset_data
from kyc_service.infrastructure.sql.settings import COMPOSE_HINT, PG_URL, sqlalchemy_url
from kyc_service.infrastructure.sql.unit_of_work import make_session_factory

LAB = Path(__file__).resolve().parent
GOLDEN = LAB / "tests" / "golden"
OUT = LAB / "out"
NOW = datetime(2026, 11, 6, 3, 0, tzinfo=UTC)


def make_engine() -> Engine:
    engine = create_engine(sqlalchemy_url(), pool_pre_ping=True)
    try:
        with engine.connect() as conn:
            conn.execute(text("SELECT 1"))
    except OperationalError:
        print(f"FAIL: không nối được PostgreSQL tại {PG_URL} ({COMPOSE_HINT})")
        sys.exit(1)
    if not asyncio.run(check_kafka()):
        print(f"FAIL: không nối được Kafka tại {KAFKA_BOOTSTRAP} ({KAFKA_HINT})")
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


def outbox_rows(engine: Engine) -> list[dict[str, Any]]:
    with engine.connect() as conn:
        found = conn.execute(
            text(
                "SELECT seq, event_id, topic, message_key, envelope, published_at, attempts"
                " FROM lab71.outbox ORDER BY seq"
            )
        )
        return [dict(r._mapping) for r in found]


def container(
    engine: Engine,
    *,
    topic: str | None = None,
    version: int = 2,
    ids: SequentialIds | None = None,
    clock: ManualClock | FixedClock | None = None,
    **kwargs: Any,
) -> Container:
    return build_container(
        engine,
        clock=clock or ManualClock(NOW),
        ids=ids or SequentialIds(),
        emit_version=version,
        topic=topic,
        log=lambda _: None,
        **kwargs,
    )


def new_topic(label: str) -> str:
    name = topic_name(f"{label}.{uuid.uuid4().hex[:6]}")
    asyncio.run(create_topic(name))
    return name


async def _dispatch(
    engine: Engine, clock: ManualClock, crash_after_publish: int | None
) -> tuple[int, int]:
    producer = await make_producer()
    try:
        d = OutboxDispatcher(make_session_factory(engine), producer, clock)
        report = await d.dispatch_once(crash_after_publish=crash_after_publish)
        return report.claimed, report.published
    finally:
        await producer.stop()


def dispatch(
    engine: Engine, clock: ManualClock, *, crash_after_publish: int | None = None
) -> tuple[int, int]:
    return asyncio.run(_dispatch(engine, clock, crash_after_publish))


def compact(value: Any) -> str:
    return json.dumps(value, ensure_ascii=False, separators=(",", ":"))


def read_event_ids(topic: str) -> list[str]:
    return [json.loads(m.value)["event_id"] for m in asyncio.run(read_all(topic))]


def decide(raw: bytes, supported: frozenset[int]) -> str:
    try:
        return f"đọc được v{decode(raw, supported=supported).version}"
    except EnvelopeError as exc:
        return f"từ chối ({exc.reason})"


def run_csharp(lines: list[str]) -> list[dict[str, Any]]:
    """Ghi các thông điệp ra out/messages.jsonl rồi cho consumer C# đọc; trả các dòng nó in ra."""
    OUT.mkdir(exist_ok=True)
    path = OUT / "messages.jsonl"
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    done = subprocess.run(
        ["dotnet", "run", "--project", str(LAB / "csharp-consumer"), "--", str(path)],
        capture_output=True,
        text=True,
        check=False,
    )
    if done.returncode != 0:
        raise RuntimeError(done.stderr + done.stdout)
    return [json.loads(line) for line in done.stdout.splitlines()]


def broken_messages(engine: Engine) -> list[tuple[str, dict[str, Any]]]:
    """Năm thông điệp hỏng, dựng từ một envelope v2 thật của outbox."""
    reset_data(engine)
    container(engine).mediator.send(submit_for("DR-002"))
    good: dict[str, Any] = outbox_rows(engine)[0]["envelope"]
    no_risk = json.loads(json.dumps(good))
    del no_risk["payload"]["risk_score"]
    null_id = json.loads(json.dumps(good))
    null_id["payload"]["application_id"] = None
    return [
        ("thiếu risk_score", no_risk),
        ("sai event_type", {**good, "event_type": "BookingCreated"}),
        ("thiếu payload", {k: v for k, v in good.items() if k != "payload"}),
        ("payload null", {**good, "payload": None}),
        ("application_id null", null_id),
    ]


def python_reason(message: dict[str, Any]) -> str:
    try:
        decode(compact(message).encode(), supported=SUPPORTED_BY_NEW_CONSUMER)
    except EnvelopeError as exc:
        return exc.reason
    return "accepted"


def commit_fails(engine: Engine) -> Callable[[], None]:
    def hook(conn: object) -> None:
        raise OperationalError("COMMIT", {}, Exception("mất kết nối lúc commit"))

    event.listen(engine, "commit", hook)
    return lambda: event.remove(engine, "commit", hook)


def scored_event() -> KycApplicationScored:
    return KycApplicationScored(
        occurred_at=NOW,
        application_id="KYC-0001",
        driver_id="DR-002",
        risk_score=0.35,
        face_score=0.71,
        ocr_min_confidence=0.58,
        review_reasons=(R.OCR_LOW_CONFIDENCE, R.FACE_LOW_SCORE, R.RISK_HIGH),
    )


class ExplodingRisk:
    def score(self, application: KycApplication) -> RiskScore:
        raise RuntimeError("bug trong fraud-scoring adapter")


def demo(engine: Engine, *, with_csharp: bool) -> None:
    print("Lưu ý: OCR, khuôn mặt, rủi ro là adapter GIẢ (bài 41 đến 61 viết bản thật).")
    print("== 1. Domain event và integration event: hai thứ khác nhau")
    event_ = scored_event()
    print("  domain event KycApplicationScored (kiểu Python, nội bộ service):")
    when = f"{event_.occurred_at:%Y-%m-%d %H:%M} UTC"
    print(f"    occurred_at={when}, application_id={event_.application_id}")
    print(f"    risk_score={event_.risk_score}, face_score={event_.face_score}")
    print(f"    review_reasons={tuple(r.name for r in event_.review_reasons)}  (enum ReviewReason)")
    envelope = scored_envelope(event_, event_id="00000000-0000-4000-8000-000000000001", version=2)
    print("  integration event v2 (JSON trên dây, hợp đồng với service khác):")
    for line in json.dumps(envelope, ensure_ascii=False, indent=2).splitlines():
        print(f"    {line}")

    print("== 2. Outbox: hồ sơ và sự kiện cùng một transaction")
    reset_data(engine)
    c = container(engine)
    c.mediator.send(submit_for("DR-002"))
    (row,) = outbox_rows(engine)
    print(
        f"  sau khi nộp: hồ sơ={scalar(engine, 'SELECT count(*) FROM lab71.kyc_applications')},"
        f" outbox={len(outbox_rows(engine))}, published_at={row['published_at']}"
    )
    reset_data(engine)
    c = container(engine)
    undo = commit_fails(engine)
    try:
        c.mediator.send(submit_for("DR-002"))
    except OperationalError:
        print("  COMMIT thất bại giữa chừng (OperationalError)")
    finally:
        undo()
    apps = scalar(engine, "SELECT count(*) FROM lab71.kyc_applications")
    print(
        f"  sau COMMIT lỗi: hồ sơ={apps}, outbox={len(outbox_rows(engine))}"
        " (bài 70 để lại 1 sự kiện ma)"
    )

    print("== 3. Dispatcher đẩy outbox ra Kafka, ngoài transaction")
    reset_data(engine)
    topic = new_topic("scored")
    c = container(engine, topic=topic)
    for driver in ("DR-001", "DR-002", "DR-004"):
        c.mediator.send(submit_for(driver))
    clock = ManualClock(NOW)
    claimed, published = dispatch(engine, clock)
    print(f"  vòng 1: thuê {claimed}, phát {published}")
    claimed, published = dispatch(engine, clock)
    print(f"  vòng 2: thuê {claimed}, phát {published}")
    messages = asyncio.run(read_all(topic))
    print(
        f"  trên topic: {len(messages)} thông điệp,"
        f" key {sorted(m.key.decode() for m in messages if m.key)}"
    )
    marked = sum(r["published_at"] is not None for r in outbox_rows(engine))
    print(f"  dòng outbox đã đánh dấu: {marked}/3")

    print("== 4. Dispatcher chết giữa phát và đánh dấu: cùng event_id đến hai lần")
    reset_data(engine)
    topic = new_topic("crash")
    c = container(engine, topic=topic)
    c.mediator.send(submit_for("DR-001"))
    c.mediator.send(submit_for("DR-002"))
    clock = ManualClock(NOW)
    try:
        dispatch(engine, clock, crash_after_publish=1)
    except SimulatedCrash as exc:
        print(f"  {exc}")
    print(f"  ngay sau đó (lease còn hiệu lực): thuê/phát = {dispatch(engine, clock)}")
    clock.advance(timedelta(seconds=31))
    print(
        f"  đồng hồ mô phỏng nhích 31 giây (lease hết hạn): thuê/phát = {dispatch(engine, clock)}"
    )
    ids = read_event_ids(topic)
    print(f"  trên topic: {len(ids)} thông điệp, {len(set(ids))} event_id khác nhau")

    print("== 5. Nâng version v1 -> v2: consumer cũ và mới đọc cùng một topic")
    reset_data(engine)
    topic = new_topic("rollout")
    shared = SequentialIds()
    container(engine, topic=topic, version=1, ids=shared).mediator.send(submit_for("DR-001"))
    container(engine, topic=topic, version=2, ids=shared).mediator.send(submit_for("DR-002"))
    dispatch(engine, ManualClock(NOW))
    future = compact({"event_type": "KycApplicationScored", "version": 3, "payload": {}})
    asyncio.run(_publish_raw(topic, future.encode()))
    raws = [m.value for m in asyncio.run(read_all(topic))]
    print(f"  {'thông điệp':<12}{'consumer cũ {1}':<34}consumer mới {{1, 2}}")
    for raw in sorted(raws, key=lambda r: json.loads(r)["version"]):
        version = json.loads(raw)["version"]
        old = decide(raw, SUPPORTED_BY_OLD_CONSUMER)
        new = decide(raw, SUPPORTED_BY_NEW_CONSUMER)
        print(f"  {'version ' + str(version):<12}{old:<34}{new}")

    print("== 6. Consumer C# đọc cùng các thông điệp bằng System.Text.Json")
    if not with_csharp:
        print("  bỏ qua (--no-csharp)")
    elif shutil.which("dotnet") is None:
        print("  bỏ qua: không có `dotnet` (cần .NET SDK 10)")
    else:
        lines = [r.decode() for r in sorted(raws, key=lambda r: json.loads(r)["version"])]
        for out in run_csharp(lines):
            if out["status"] == "accepted":
                print(
                    f"  đọc được v{out['version']}: {out['application_id']}"
                    f" risk_score={out['risk_score']} review_reasons={out['review_reasons']}"
                    f" face_score={out['face_score']}"
                )
            elif out["status"] == "rejected":
                print(f"  từ chối v{out['version']}: {out['reason']}")
            else:
                print(
                    f"  không đặt naming policy: event_id = {out['naive_event_id']},"
                    f" payload null = {out['naive_payload_is_null']}"
                )

        print("== 7. Thông điệp hỏng: C# và Python từ chối cùng lý do")
        broken = broken_messages(engine)
        results = run_csharp([compact(m) for _, m in broken])
        print(f"  {'thông điệp':<22}{'C#':<20}Python")
        for (label, message), got in zip(broken, results, strict=False):
            print(f"  {label:<22}{got.get('reason', got['status']):<20}{python_reason(message)}")


async def _publish_raw(topic: str, value: bytes) -> None:
    producer = await make_producer()
    try:
        await producer.send_and_wait(topic, value, key=b"future")
    finally:
        await producer.stop()


def run_verify(engine: Engine, *, with_csharp: bool) -> int:
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

    def count_apps() -> int:
        return int(scalar(engine, "SELECT count(*) FROM lab71.kyc_applications"))

    # --- outbox trong transaction ---
    @case("một hồ sơ chấm điểm đủ: đúng một dòng outbox, envelope v2 đủ trường, id lưu một lần")
    def _() -> bool:
        reset_data(engine)
        container(engine).mediator.send(submit_for("DR-002"))
        rows = outbox_rows(engine)
        if len(rows) != 1:
            return False
        env = rows[0]["envelope"]
        return (
            env["event_id"] == rows[0]["event_id"] == "00000000-0000-4000-8000-000000000001"
            and rows[0]["message_key"] == "KYC-0001"
            and rows[0]["published_at"] is None
            and (env["event_type"], env["version"]) == ("KycApplicationScored", 2)
            and env["payload"]
            == {
                "application_id": "KYC-0001",
                "driver_id": "DR-002",
                "risk_score": 0.35,
                "review_reasons": ["ocr_low_confidence", "face_low_score", "risk_high"],
                "face_score": 0.71,
                "ocr_min_confidence": 0.58,
            }
        )

    @case("producer ghim version 1: payload chỉ có bốn trường của v1")
    def _() -> bool:
        reset_data(engine)
        container(engine, version=1).mediator.send(submit_for("DR-002"))
        env = outbox_rows(engine)[0]["envelope"]
        return env["version"] == 1 and set(env["payload"]) == {
            "application_id",
            "driver_id",
            "risk_score",
            "review_reasons",
        }

    @case("hồ sơ chưa có điểm (không đồng ý khuôn mặt) không sinh dòng outbox")
    def _() -> bool:
        reset_data(engine)
        container(engine).mediator.send(submit_for("DR-003", consent=False))
        return count_apps() == 1 and outbox_rows(engine) == []

    @case("kyc-ocr tắt rồi làm tiếp: dòng outbox chỉ xuất hiện khi chấm điểm xong, đúng một dòng")
    def _() -> bool:
        reset_data(engine)
        ocr = FakeOcr(available=False)
        c = container(engine, ocr=ocr)
        c.mediator.send(submit_for("DR-001"))
        before = len(outbox_rows(engine))
        ocr.available = True
        c.mediator.send(ResumeScreening("KYC-0001"))
        return before == 0 and len(outbox_rows(engine)) == 1

    @case("COMMIT thất bại: không hồ sơ và không dòng outbox (bài 70 còn để lại sự kiện ma)")
    def _() -> bool:
        reset_data(engine)
        c = container(engine)
        undo = commit_fails(engine)
        try:
            failed = raises(OperationalError, lambda: c.mediator.send(submit_for("DR-001")))
        finally:
            undo()
        return failed and count_apps() == 0 and outbox_rows(engine) == []

    @case("lỗi bất ngờ trước khi chấm điểm xong: không hồ sơ, không outbox")
    def _() -> bool:
        reset_data(engine)
        c = container(engine, risk=ExplodingRisk())
        failed = raises(RuntimeError, lambda: c.mediator.send(submit_for("DR-001")))
        return failed and count_apps() == 0 and outbox_rows(engine) == []

    @case(
        "publisher chỉ chạy trong transaction; domain event chưa có ánh xạ là lỗi, không dòng nào"
    )
    def _() -> bool:
        reset_data(engine)
        c = container(engine)
        publisher = OutboxEventPublisher(
            c.uow, topic="x", emit_version=2, ids=SequentialIds(), clock=FixedClock(NOW)
        )

        class Unknown(DomainEvent):
            pass

        def publish_inside() -> None:
            with c.uow.begin():
                publisher.publish([Unknown(occurred_at=NOW)])

        outside = raises(
            UnitOfWorkNotStarted, lambda: publisher.publish([Unknown(occurred_at=NOW)])
        )
        return outside and raises(UnmappedDomainEvent, publish_inside) and outbox_rows(engine) == []

    @case("envelope không chứa PII: không số CCCD, không họ tên")
    def _() -> bool:
        reset_data(engine)
        container(engine).mediator.send(submit_for("DR-002"))
        dump = compact(outbox_rows(engine)[0]["envelope"])
        return "099190000271" not in dump and "Trần Thị B" not in dump and "099*" not in dump

    # --- hợp đồng ---
    @case("envelope khớp hai golden file v1 và v2 (tests/golden)")
    def _() -> bool:
        event_id = "00000000-0000-4000-8000-000000000001"
        return all(
            scored_envelope(scored_event(), event_id=event_id, version=v)
            == json.loads((GOLDEN / f"scored.v{v}.json").read_text(encoding="utf-8"))
            for v in (1, 2)
        )

    @case("v2 chỉ thêm hai trường tùy chọn vào v1; version chưa định nghĩa bị producer từ chối")
    def _() -> bool:
        v1 = json.loads((GOLDEN / "scored.v1.json").read_text(encoding="utf-8"))
        v2 = json.loads((GOLDEN / "scored.v2.json").read_text(encoding="utf-8"))
        same = all(v2["payload"][k] == v for k, v in v1["payload"].items())
        added = set(v2["payload"]) - set(v1["payload"]) == {"face_score", "ocr_min_confidence"}
        return (
            same
            and added
            and raises(
                ValueError,
                lambda: scored_envelope(scored_event(), event_id="x", version=3),
            )
        )

    @case("consumer: version lạ, True, JSON hỏng, thiếu trường đều bị từ chối với lý do ổn định")
    def _() -> bool:
        good = json.loads((GOLDEN / "scored.v2.json").read_text(encoding="utf-8"))
        samples: list[tuple[bytes, str]] = [
            (compact({**good, "version": 3}).encode(), "unsupported_version"),
            (compact({**good, "version": True}).encode(), "unsupported_version"),
            (b"\xff\xfe", "invalid_json"),
            (
                compact({**good, "payload": {"application_id": "KYC-0001"}}).encode(),
                "invalid_payload",
            ),
        ]
        for raw, reason in samples:
            try:
                decode(raw, supported=SUPPORTED_BY_NEW_CONSUMER)
            except EnvelopeError as exc:
                if exc.reason != reason:
                    return False
            else:
                return False
        return True

    # --- dispatcher và Kafka ---
    @case(
        "dispatcher phát đủ 3 dòng, đánh dấu published_at, vòng sau không phát lại, key là mã hồ sơ"
    )
    def _() -> bool:
        reset_data(engine)
        topic = new_topic("v")
        c = container(engine, topic=topic)
        for driver in ("DR-001", "DR-002", "DR-004"):
            c.mediator.send(submit_for(driver))
        clock = ManualClock(NOW)
        first = dispatch(engine, clock)
        second = dispatch(engine, clock)
        messages = asyncio.run(read_all(topic))
        return (
            first == (3, 3)
            and second == (0, 0)
            and all(r["published_at"] is not None for r in outbox_rows(engine))
            and sorted(m.key.decode() for m in messages if m.key)
            == ["KYC-0001", "KYC-0002", "KYC-0003"]
            and sorted(read_event_ids(topic))
            == [f"00000000-0000-4000-8000-00000000000{n}" for n in (1, 2, 3)]
        )

    @case("thông điệp trên Kafka đúng bằng envelope đã lưu trong outbox")
    def _() -> bool:
        reset_data(engine)
        topic = new_topic("w")
        container(engine, topic=topic).mediator.send(submit_for("DR-002"))
        dispatch(engine, ManualClock(NOW))
        (message,) = asyncio.run(read_all(topic))
        return json.loads(message.value) == outbox_rows(engine)[0]["envelope"]

    @case("chết giữa phát và đánh dấu: lease chặn; hết lease thì phát lại, 3 thông điệp 2 event_id")
    def _() -> bool:
        reset_data(engine)
        topic = new_topic("x")
        c = container(engine, topic=topic)
        c.mediator.send(submit_for("DR-001"))
        c.mediator.send(submit_for("DR-002"))
        clock = ManualClock(NOW)
        crashed = raises(SimulatedCrash, lambda: dispatch(engine, clock, crash_after_publish=1))
        unmarked = [r["published_at"] for r in outbox_rows(engine)] == [None, None]
        blocked = dispatch(engine, clock) == (0, 0)
        clock.advance(timedelta(seconds=31))
        again = dispatch(engine, clock) == (2, 2)
        ids = read_event_ids(topic)
        return crashed and unmarked and blocked and again and len(ids) == 3 and len(set(ids)) == 2

    @case("hai dispatcher cùng lúc thuê hai tập dòng rời nhau (SKIP LOCKED)")
    def _() -> bool:
        reset_data(engine)
        c = container(engine, topic="lab71.unused")
        for driver in ("DR-001", "DR-002", "DR-004", "DR-005"):
            c.mediator.send(submit_for(driver))
        factory = make_session_factory(engine)
        lease = timedelta(seconds=30)
        with factory() as a, a.begin(), factory() as b, b.begin():
            first = claim_batch(a, now=NOW, lease=lease, limit=2)
            second = claim_batch(b, now=NOW, lease=lease, limit=2)
        return len(first) == len(second) == 2 and {x.seq for x in first}.isdisjoint(
            {x.seq for x in second}
        )

    @case("thuê dòng: trong lease không ai thuê lại, hết lease thì thuê lại và attempts tăng")
    def _() -> bool:
        reset_data(engine)
        container(engine, topic="lab71.unused").mediator.send(submit_for("DR-001"))
        factory = make_session_factory(engine)
        lease = timedelta(seconds=30)
        with factory() as s, s.begin():
            first = len(claim_batch(s, now=NOW, lease=lease, limit=10))
        with factory() as s, s.begin():
            held = claim_batch(s, now=NOW + timedelta(seconds=10), lease=lease, limit=10)
        with factory() as s, s.begin():
            later = len(claim_batch(s, now=NOW + timedelta(seconds=31), lease=lease, limit=10))
        return first == 1 and held == [] and later == 1 and outbox_rows(engine)[0]["attempts"] == 2

    @case("thứ tự nâng version: consumer cũ chỉ đọc 2 thông điệp v1 và từ chối 2 v2; mới đọc đủ 4")
    def _() -> bool:
        reset_data(engine)
        topic = new_topic("y")
        shared = SequentialIds()
        container(engine, topic=topic, version=1, ids=shared).mediator.send(submit_for("DR-001"))
        container(engine, topic=topic, version=1, ids=shared).mediator.send(submit_for("DR-002"))
        container(engine, topic=topic, version=2, ids=shared).mediator.send(submit_for("DR-004"))
        container(engine, topic=topic, version=2, ids=shared).mediator.send(submit_for("DR-005"))
        dispatch(engine, ManualClock(NOW))
        raws = [m.value for m in asyncio.run(read_all(topic))]
        old = Counter(decide(r, SUPPORTED_BY_OLD_CONSUMER) for r in raws)
        new = Counter(decide(r, SUPPORTED_BY_NEW_CONSUMER) for r in raws)
        return old == Counter({"đọc được v1": 2, "từ chối (unsupported_version)": 2}) and new == (
            Counter({"đọc được v1": 2, "đọc được v2": 2})
        )

    @case("api: POST 201 và outbox có đúng một dòng cho hồ sơ vừa nộp")
    def _() -> bool:
        reset_data(engine)
        client = TestClient(create_app(container(engine)))
        r = client.post(
            "/applications",
            json={
                "driver_id": "DR-002",
                "cccd_ref": "synthetic/cccd/DR-002.png",
                "gplx_ref": "synthetic/gplx/DR-002.png",
                "selfie_ref": "synthetic/selfie/DR-002.png",
                "face_consent": True,
            },
        )
        return r.status_code == 201 and len(outbox_rows(engine)) == 1

    # --- consumer C# ---
    if with_csharp:
        if shutil.which("dotnet") is None:
            print(
                "FAIL: cần `dotnet` (.NET SDK 10) để kiểm consumer C#; thêm --no-csharp để bỏ qua"
            )
            return 1

        def csharp_lines(versions: list[tuple[int, str]]) -> list[str]:
            reset_data(engine)
            shared = SequentialIds()
            for version, driver in versions:
                container(engine, version=version, ids=shared).mediator.send(submit_for(driver))
            return [compact(r["envelope"]) for r in outbox_rows(engine)]

        @case("C#: đọc v1, v2, v2 đúng nghĩa như consumer Python; version 3 bị từ chối")
        def _() -> bool:
            lines = csharp_lines([(1, "DR-001"), (2, "DR-002"), (2, "DR-004")])
            lines.append(compact({"event_type": "KycApplicationScored", "version": 3}))
            out = run_csharp(lines)
            accepted = [o for o in out if o["status"] == "accepted"]
            if [o["version"] for o in accepted] != [1, 2, 2]:
                return False
            for line, got in zip(lines, accepted, strict=False):
                py = decode(line.encode(), supported=SUPPORTED_BY_NEW_CONSUMER)
                same = (
                    got["event_id"] == py.event_id
                    and got["application_id"] == py.application_id
                    and got["risk_score"] == py.risk_score
                    and got["review_reasons"] == list(py.review_reasons)
                    and got["face_score"] == py.face_score
                    and got["ocr_min_confidence"] == py.ocr_min_confidence
                )
                if not same:
                    return False
            return {"status": "rejected", "version": 3, "reason": "unsupported_version"} in out

        @case(
            "C#: thiếu risk_score, sai event_type, thiếu hoặc null payload bị từ chối, như Python"
        )
        def _() -> bool:
            broken = broken_messages(engine)
            out = run_csharp([compact(m) for _, m in broken])
            reasons = [o["reason"] for o in out if o["status"] == "rejected"]
            python = [python_reason(m) for _, m in broken]
            return (
                not [o for o in out if o["status"] == "accepted"]
                and reasons == ["invalid_payload", "wrong_event_type", *["invalid_payload"] * 3]
                and python == reasons
            )

        @case("C#: không đặt naming policy thì không lỗi mà chỉ nhận giá trị mặc định")
        def _() -> bool:
            naive = run_csharp(csharp_lines([(2, "DR-002")]))[-1]
            return (
                naive["status"] == "naive"
                and naive["naive_event_id"] == "00000000-0000-0000-0000-000000000000"
                and naive["naive_payload_is_null"] is True
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


def cleanup(engine: Engine) -> None:
    drop_schema(engine)
    asyncio.run(delete_lab_topics())


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--verify", action="store_true", help="kiểm các kết quả in trong bài")
    parser.add_argument("--no-csharp", action="store_true", help="bỏ qua consumer C# (cần .NET)")
    parser.add_argument("--keep", action="store_true", help="giữ schema lab71 và topic lab71.*")
    parser.add_argument("--cleanup", action="store_true", help="chỉ xóa schema và topic")
    args = parser.parse_args()
    warnings.simplefilter("ignore", FakeAdapterWarning)  # lab cố ý dùng adapter giả
    engine = make_engine()
    if args.cleanup:
        cleanup(engine)
        print("đã xóa schema lab71 và các topic lab71.*")
        return 0
    create_schema(engine)
    asyncio.run(delete_lab_topics())
    try:
        if args.verify:
            return run_verify(engine, with_csharp=not args.no_csharp)
        demo(engine, with_csharp=not args.no_csharp)
        return 0
    finally:
        if not args.keep:
            cleanup(engine)
        engine.dispose()


if __name__ == "__main__":
    sys.exit(main())
