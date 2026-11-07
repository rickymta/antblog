"""Consumer C# đọc đúng các thông điệp mà outbox của lab Python tạo ra (cần .NET SDK 10)."""

import json
import shutil
import subprocess
from pathlib import Path

import pytest
from consumers.scored_consumer import SUPPORTED_BY_NEW_CONSUMER, EnvelopeError, decode
from sqlalchemy.engine import Engine

from kyc_service.api.container import build_container
from kyc_service.infrastructure.clocks import FixedClock
from kyc_service.infrastructure.ids import SequentialIds

from .conftest import NOW, outbox_rows, submit_for

pytestmark = [pytest.mark.usefixtures("clean"), pytest.mark.csharp]

PROJECT = Path(__file__).resolve().parent.parent / "csharp-consumer"


def run_csharp(lines: list[str], tmp_path: Path) -> list[dict[str, object]]:
    if shutil.which("dotnet") is None:
        pytest.fail("không có `dotnet` (cần .NET SDK 10); bỏ test này bằng -m 'not csharp'")
    path = tmp_path / "messages.jsonl"
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    done = subprocess.run(
        ["dotnet", "run", "--project", str(PROJECT), "--", str(path)],
        capture_output=True,
        text=True,
        check=False,
    )
    assert done.returncode == 0, done.stderr + done.stdout
    return [json.loads(line) for line in done.stdout.splitlines()]


def test_csharp_reads_v1_v2_and_rejects_v3(engine: Engine, tmp_path: Path) -> None:
    ids = SequentialIds()
    for version, drivers in ((1, ["DR-001"]), (2, ["DR-002", "DR-004"])):
        c = build_container(
            engine,
            clock=FixedClock(NOW),
            ids=ids,
            emit_version=version,
            log=lambda _: None,
        )
        for d in drivers:
            c.mediator.send(submit_for(d))
    lines = [
        json.dumps(r["envelope"], ensure_ascii=False, separators=(",", ":"))
        for r in outbox_rows(engine)
    ]
    lines.append('{"event_type":"KycApplicationScored","version":3}')
    out = run_csharp(lines, tmp_path)
    accepted = [o for o in out if o["status"] == "accepted"]
    assert [o["version"] for o in accepted] == [1, 2, 2]
    for line, got in zip(lines, accepted, strict=False):
        py = decode(line.encode(), supported=SUPPORTED_BY_NEW_CONSUMER)  # cùng nghĩa với Python
        assert got["event_id"] == py.event_id
        assert got["application_id"] == py.application_id
        assert got["risk_score"] == py.risk_score
        assert got["review_reasons"] == list(py.review_reasons)
        assert got["face_score"] == py.face_score
        assert got["ocr_min_confidence"] == py.ocr_min_confidence
    assert {"status": "rejected", "version": 3, "reason": "unsupported_version"} in out


def test_without_the_naming_policy_csharp_silently_gets_defaults(
    engine: Engine, tmp_path: Path
) -> None:
    c = build_container(
        engine, emit_version=2, clock=FixedClock(NOW), ids=SequentialIds(), log=lambda _: None
    )
    c.mediator.send(submit_for("DR-002"))
    line = json.dumps(outbox_rows(engine)[0]["envelope"], separators=(",", ":"))
    out = run_csharp([line], tmp_path)
    naive = out[-1]
    assert naive["status"] == "naive"
    assert naive["naive_event_id"] == "00000000-0000-0000-0000-000000000000"
    assert naive["naive_payload_is_null"] is True


def test_csharp_rejects_messages_with_missing_or_wrong_content(
    engine: Engine, tmp_path: Path
) -> None:
    c = build_container(
        engine, emit_version=2, clock=FixedClock(NOW), ids=SequentialIds(), log=lambda _: None
    )
    c.mediator.send(submit_for("DR-002"))
    good = outbox_rows(engine)[0]["envelope"]
    no_risk = json.loads(json.dumps(good))
    del no_risk["payload"]["risk_score"]
    null_id = json.loads(json.dumps(good))
    null_id["payload"]["application_id"] = None
    broken = [
        no_risk,
        {**good, "event_type": "BookingCreated"},
        {k: v for k, v in good.items() if k != "payload"},
        {**good, "payload": None},
        null_id,
    ]
    lines = [json.dumps(m, separators=(",", ":")) for m in broken]
    out = run_csharp(lines, tmp_path)
    assert [o for o in out if o["status"] == "accepted"] == []
    reasons = [o["reason"] for o in out if o["status"] == "rejected"]
    assert reasons == [
        "invalid_payload",
        "wrong_event_type",
        "invalid_payload",
        "invalid_payload",
        "invalid_payload",
    ]
    python_reasons = []
    for line in lines:
        with pytest.raises(EnvelopeError) as info:
            decode(line.encode(), supported=SUPPORTED_BY_NEW_CONSUMER)
        python_reasons.append(info.value.reason)
    assert python_reasons == reasons  # Python và C# từ chối cùng lý do
