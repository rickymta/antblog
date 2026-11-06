"""Hợp đồng sự kiện: golden file, ánh xạ domain event -> integration event, consumer tham chiếu."""

import json
from datetime import UTC, datetime
from pathlib import Path

import pytest
from consumers.scored_consumer import (
    SUPPORTED_BY_NEW_CONSUMER,
    SUPPORTED_BY_OLD_CONSUMER,
    EnvelopeError,
    decode,
)

from kyc_service.application.integration_events import (
    UnmappedDomainEvent,
    scored_envelope,
    to_envelope,
)
from kyc_service.domain.events import DomainEvent, KycApplicationScored
from kyc_service.domain.review import ReviewReason as R

GOLDEN = Path(__file__).parent / "golden"
NOW = datetime(2026, 11, 6, 3, 0, tzinfo=UTC)
EVENT_ID = "00000000-0000-4000-8000-000000000001"

EVENT = KycApplicationScored(
    occurred_at=NOW,
    application_id="KYC-0001",
    driver_id="DR-002",
    risk_score=0.35,
    face_score=0.71,
    ocr_min_confidence=0.58,
    review_reasons=(R.OCR_LOW_CONFIDENCE, R.FACE_LOW_SCORE, R.RISK_HIGH),
)


def golden(version: int) -> bytes:
    return (GOLDEN / f"scored.v{version}.json").read_bytes()


@pytest.mark.parametrize("version", [1, 2])
def test_envelope_matches_the_golden_file(version: int) -> None:
    produced = scored_envelope(EVENT, event_id=EVENT_ID, version=version)
    assert produced == json.loads(golden(version))


def test_v2_only_adds_optional_fields_to_v1() -> None:
    v1 = json.loads(golden(1))
    v2 = json.loads(golden(2))
    assert {k: v for k, v in v2["payload"].items() if k in v1["payload"]} == v1["payload"]
    assert set(v2["payload"]) - set(v1["payload"]) == {"face_score", "ocr_min_confidence"}
    assert {k: v for k, v in v2.items() if k not in ("version", "payload")} == {
        k: v for k, v in v1.items() if k not in ("version", "payload")
    }


def test_wire_names_are_snake_case_everywhere() -> None:
    env = json.loads(golden(2))
    names = [*env, *env["payload"]]
    assert all(n == n.lower() and " " not in n and "-" not in n for n in names)
    assert not any(c.isupper() for c in "".join(names))


def test_undefined_version_is_refused_at_the_producer() -> None:
    with pytest.raises(ValueError, match="version 3"):
        scored_envelope(EVENT, event_id=EVENT_ID, version=3)


def test_unmapped_domain_event_is_an_error_not_silence() -> None:
    class Other(DomainEvent):
        pass

    with pytest.raises(UnmappedDomainEvent):
        to_envelope(Other(occurred_at=NOW), event_id=EVENT_ID, version=1)


def test_new_consumer_reads_both_versions_and_fills_missing_fields_with_none() -> None:
    one = decode(golden(1), supported=SUPPORTED_BY_NEW_CONSUMER)
    two = decode(golden(2), supported=SUPPORTED_BY_NEW_CONSUMER)
    assert (one.face_score, one.ocr_min_confidence) == (None, None)
    assert (two.face_score, two.ocr_min_confidence) == (0.71, 0.58)
    assert (
        one.review_reasons
        == two.review_reasons
        == ("ocr_low_confidence", "face_low_score", "risk_high")
    )
    assert one.occurred_at == NOW


def test_old_consumer_rejects_version_2_before_reading_the_payload() -> None:
    assert decode(golden(1), supported=SUPPORTED_BY_OLD_CONSUMER).version == 1
    with pytest.raises(EnvelopeError) as info:
        decode(golden(2), supported=SUPPORTED_BY_OLD_CONSUMER)
    assert info.value.reason == "unsupported_version"


def test_unknown_extra_fields_inside_a_supported_version_are_ignored() -> None:
    env = json.loads(golden(2))
    env["payload"]["coupon"] = "NEWUSER50"
    env["x_trace"] = "abc"
    got = decode(json.dumps(env).encode(), supported=SUPPORTED_BY_NEW_CONSUMER)
    assert got.application_id == "KYC-0001"


@pytest.mark.parametrize(
    ("mutate", "reason"),
    [
        (lambda e: e.update(version=True), "unsupported_version"),  # True == 1 không được lọt
        (lambda e: e.update(version=[2]), "unsupported_version"),
        (lambda e: e.update(version=3), "unsupported_version"),
        (lambda e: e.update(event_type="BookingCreated"), "wrong_event_type"),
        (lambda e: e["payload"].pop("risk_score"), "invalid_payload"),
        (lambda e: e["payload"].update(review_reasons="risk_high"), "invalid_payload"),
        (lambda e: e.update(occurred_at="hôm qua"), "invalid_payload"),
    ],
    ids=["True", "list", "v3", "sai-loại", "thiếu-trường", "reasons-không-phải-list", "ngày-sai"],
)
def test_malformed_messages_are_rejected_with_a_stable_reason(mutate, reason: str) -> None:  # type: ignore[no-untyped-def]
    env = json.loads(golden(2))
    mutate(env)
    with pytest.raises(EnvelopeError) as info:
        decode(json.dumps(env).encode(), supported=SUPPORTED_BY_NEW_CONSUMER)
    assert info.value.reason == reason


def test_not_json_and_not_an_object() -> None:
    for raw, reason in ((b"\xff\xfe", "invalid_json"), (b"[1]", "invalid_envelope")):
        with pytest.raises(EnvelopeError) as info:
            decode(raw, supported=SUPPORTED_BY_NEW_CONSUMER)
        assert info.value.reason == reason
