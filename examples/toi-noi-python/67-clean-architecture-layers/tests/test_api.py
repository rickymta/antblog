"""Test lớp api qua TestClient: dịch HTTP và lỗi, không chứa quy tắc nghiệp vụ."""

import pytest
from fastapi.testclient import TestClient

from kyc_service.api.app import create_app
from kyc_service.api.container import build_container
from kyc_service.infrastructure.clocks import FixedClock
from kyc_service.infrastructure.fake_services import FakeOcr
from kyc_service.infrastructure.memory_repository import InMemoryKycRepository

from .conftest import NOW

BODY = {
    "driver_id": "DR-001",
    "cccd_ref": "synthetic/cccd/DR-001.png",
    "gplx_ref": "synthetic/gplx/DR-001.png",
    "selfie_ref": "synthetic/selfie/DR-001.png",
    "face_consent": True,
}


def client(**kwargs: object) -> TestClient:
    return TestClient(create_app(build_container(clock=FixedClock(NOW), **kwargs)))  # type: ignore[arg-type]


def test_submit_created_and_get() -> None:
    c = client()
    r = c.post("/applications", json=BODY)
    assert r.status_code == 201
    assert r.headers["location"] == "/applications/KYC-0001"
    assert "099090000183" not in r.text
    assert c.get("/applications/KYC-0001").json()["status"] == "needs_review"


def test_submit_accepted_when_ocr_down() -> None:
    r = client(ocr=FakeOcr(available=False)).post("/applications", json=BODY)
    assert r.status_code == 202
    assert r.json()["pending_check"] == "kyc-ocr"


@pytest.mark.parametrize(
    ("patch", "field", "code"),
    [
        ({"status": "approved"}, "body.status", "extra_forbidden"),
        ({"driver_id": "driver-1"}, "body.driver_id", "string_pattern_mismatch"),
        ({"face_consent": "có"}, "body.face_consent", "bool_type"),
    ],
    ids=["status-tự-đặt", "mã-sai", "consent-sai-kiểu"],
)
def test_bad_body_is_422_and_stores_nothing(
    patch: dict[str, object], field: str, code: str
) -> None:
    repo = InMemoryKycRepository()
    r = client(repository=repo).post("/applications", json={**BODY, **patch})
    assert r.status_code == 422
    assert {"field": field, "code": code} == {
        k: r.json()["errors"][0][k] for k in ("field", "code")
    }
    assert all(
        set(e) == {"field", "code", "message"} for e in r.json()["errors"]
    )  # không chép input
    assert repo.count() == 0


def test_review_conflict_and_not_found() -> None:
    c = client(ocr=FakeOcr(available=False))
    c.post("/applications", json=BODY)
    review = {"reviewer_id": "RV-001", "decision": "approved", "reason": "ép"}
    assert c.post("/applications/KYC-0001/review", json=review).status_code == 409
    assert c.post("/applications/KYC-9999/review", json=review).status_code == 404


def test_422_does_not_echo_the_rejected_value() -> None:
    r = client().post("/applications", json={**BODY, "driver_id": "BI-MAT-12345"})
    assert r.status_code == 422
    assert "BI-MAT-12345" not in r.text


def test_consent_given_as_string_is_422_not_guessed() -> None:
    repo = InMemoryKycRepository()
    r = client(repository=repo).post("/applications", json={**BODY, "face_consent": "false"})
    assert r.status_code == 422
    assert r.json()["errors"][0]["code"] == "bool_type"
    assert repo.count() == 0
