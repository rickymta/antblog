"""Lớp api qua TestClient, với PostgreSQL thật phía sau."""

from typing import Any

import pytest
from fastapi.testclient import TestClient
from sqlalchemy import text
from sqlalchemy.engine import Engine

from kyc_service.api.app import create_app
from kyc_service.api.container import build_container
from kyc_service.infrastructure.clocks import FixedClock
from kyc_service.infrastructure.fake_services import FakeOcr

from .conftest import NOW

pytestmark = pytest.mark.usefixtures("clean")

BODY = {
    "driver_id": "DR-001",
    "cccd_ref": "synthetic/cccd/DR-001.png",
    "gplx_ref": "synthetic/gplx/DR-001.png",
    "selfie_ref": "synthetic/selfie/DR-001.png",
    "face_consent": True,
}


def client(engine: Engine, **kwargs: Any) -> TestClient:
    container = build_container(
        engine, emit_version=2, clock=FixedClock(NOW), log=lambda _: None, **kwargs
    )
    return TestClient(create_app(container))


def test_submit_then_read_through_a_fresh_container(engine: Engine) -> None:
    r = client(engine).post("/applications", json=BODY)
    assert r.status_code == 201
    assert r.headers["location"] == "/applications/KYC-0001"
    assert "099090000183" not in r.text
    again = client(engine)  # container mới, cùng cơ sở dữ liệu: dữ liệu còn nguyên
    assert again.get("/applications/KYC-0001").json()["status"] == "needs_review"
    assert [a["application_id"] for a in again.get("/review-queue").json()] == ["KYC-0001"]


def test_ocr_down_is_202(engine: Engine) -> None:
    r = client(engine, ocr=FakeOcr(available=False)).post("/applications", json=BODY)
    assert (r.status_code, r.json()["pending_check"]) == (202, "kyc-ocr")


def test_extra_status_field_is_422_and_nothing_is_stored(engine: Engine) -> None:
    c = client(engine)
    r = c.post("/applications", json={**BODY, "status": "approved"})
    assert (r.status_code, r.json()["errors"][0]["code"]) == (422, "extra_forbidden")
    assert c.get("/review-queue").json() == []


def test_review_of_submitted_is_409_and_unknown_is_404(engine: Engine) -> None:
    c = client(engine, ocr=FakeOcr(available=False))
    c.post("/applications", json=BODY)
    review = {"reviewer_id": "RV-001", "decision": "approved", "reason": "ép"}
    conflict = c.post("/applications/KYC-0001/review", json=review)
    missing = c.get("/applications/KYC-9999")
    assert conflict.status_code == 409
    assert conflict.json()["type"].endswith("invalid-status-transition")
    assert missing.status_code == 404
    assert missing.headers["content-type"].startswith("application/problem+json")


def test_two_sequential_decisions_are_refused_by_the_domain_not_by_the_version(
    engine: Engine,
) -> None:
    c = client(engine)
    c.post("/applications", json=BODY)
    first = c.post(
        "/applications/KYC-0001/review",
        json={"reviewer_id": "RV-001", "decision": "approved", "reason": "khớp"},
    )
    second = c.post(
        "/applications/KYC-0001/review",
        json={"reviewer_id": "RV-002", "decision": "rejected", "reason": "mờ"},
    )
    assert first.status_code == 200
    assert second.status_code == 409
    assert second.json()["type"].endswith("invalid-status-transition")


def test_pending_check_survives_a_later_get(engine: Engine) -> None:
    c = client(engine, ocr=FakeOcr(available=False))
    posted = c.post("/applications", json=BODY)
    got = c.get(posted.headers["location"])
    assert (posted.status_code, got.json()["pending_check"]) == (202, "kyc-ocr")


def test_application_ids_go_past_9999(engine: Engine) -> None:
    with engine.begin() as conn:
        conn.execute(text("ALTER SEQUENCE lab71.kyc_application_seq RESTART WITH 10000"))
    c = client(engine)
    created = c.post("/applications", json=BODY)
    assert created.headers["location"] == "/applications/KYC-10000"
    assert c.get("/applications/KYC-10000").status_code == 200
