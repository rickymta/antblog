"""Xác thực và phân quyền như lab 34: token thật (ký bằng jwt.encode), mọi token xấu cùng một câu 401, thay bộ kiểm token."""

import logging
from datetime import timedelta
from typing import Annotated

import pytest

from app.security import Principal, TokenService, current_principal, get_token_service, require_scope
from tests.conftest import TEST_SECRET

QUOTE = {"distance_km": "2"}
INVALID_TOKEN = 'Bearer error="invalid_token"'


async def test_missing_token_is_401_with_a_plain_challenge(client):
    response = await client.post("/v1/quotes", json=QUOTE)

    assert response.status_code == 401
    assert response.headers["www-authenticate"] == "Bearer"  # chưa gửi credential: chưa có mã lỗi (RFC 6750)
    assert response.json()["type"] == "about:blank"
    assert response.json()["title"] == "Unauthorized"


BAD_TOKENS = {
    "expired": {"expires_in": timedelta(seconds=-1)},
    "wrong-signature": {"secret": "khoa-khac-" + "0123456789abcdef" * 3},
    "wrong-algorithm": {"algorithm": "HS512"},  # đúng khóa nhưng sai thuật toán đã cố định
    "wrong-audience": {"audience": "api-khac"},
    "wrong-issuer": {"issuer": "ai-do"},
    "missing-iat": {"omit": ("iat",)},
    "scope-as-list": {"scope": ["customer"]},  # RFC 6749: scope là chuỗi, không phải list
}


async def test_every_bad_token_gets_the_same_401_and_the_reason_goes_to_the_log(client, auth, caplog):
    bodies, reasons = {}, {}
    for name, kwargs in BAD_TOKENS.items():
        caplog.clear()
        with caplog.at_level(logging.WARNING, logger="toinoi.auth"):
            response = await client.post("/v1/quotes", json=QUOTE, headers=auth(**kwargs))
        assert response.status_code == 401, name
        assert response.headers["www-authenticate"] == INVALID_TOKEN, name
        bodies[name] = response.json()
        reasons[name] = caplog.messages[-1]

    # Client không phân biệt được lý do: mọi body giống nhau (kể cả trường `request_id` khác nhau)
    shapes = {tuple(sorted((k, v) for k, v in body.items() if k != "request_id")) for body in bodies.values()}
    assert len(shapes) == 1
    # Còn log thì phân biệt được từng lý do
    assert "reason=expired " in reasons["expired"]
    assert "reason=bad_algorithm " in reasons["wrong-algorithm"]
    assert "reason=bad_audience " in reasons["wrong-audience"]
    assert "reason=missing_claim:iat " in reasons["missing-iat"]
    assert "reason=malformed " in reasons["scope-as-list"]


async def test_garbage_in_the_bearer_slot_is_401(client):
    response = await client.post("/v1/quotes", json=QUOTE, headers={"Authorization": "Bearer khong.phai.jwt"})

    assert response.status_code == 401
    assert response.headers["www-authenticate"] == INVALID_TOKEN


async def test_driver_token_cannot_create_a_quote(client, auth):
    response = await client.post("/v1/quotes", json=QUOTE, headers=auth("DR-001", "driver"))

    assert response.status_code == 403
    assert response.headers["www-authenticate"] == 'Bearer error="insufficient_scope", scope="customer"'
    assert response.json()["title"] == "Forbidden"


async def test_scope_claim_is_a_space_separated_list(client, auth):
    response = await client.post("/v1/quotes", json=QUOTE, headers=auth("CU-001", "driver customer"))

    assert response.status_code == 201


async def test_require_scope_needs_every_listed_scope(app, client, auth):
    @app.get("/_test/both")
    async def both(_: Annotated[Principal, require_scope("customer", "driver")]) -> dict[str, bool]:
        return {"ok": True}

    only_one = await client.get("/_test/both", headers=auth("CU-001", "customer"))
    both_scopes = await client.get("/_test/both", headers=auth("CU-001", "customer driver"))

    assert (only_one.status_code, both_scopes.status_code) == (403, 200)
    assert 'scope="customer driver"' in only_one.headers["www-authenticate"]


class FakeTokens(TokenService):
    """Bộ kiểm token giả: không cần ký token thật, nhưng vẫn trả Principal để kiểm scope chạy tiếp."""

    def __init__(self, principal: Principal) -> None:
        self.principal = principal

    def decode(self, token: str) -> Principal:
        return self.principal


async def test_fake_token_service_skips_signing_but_scope_rules_still_run(app, client):
    bearer = {"Authorization": "Bearer bat-ky-chuoi-nao"}  # oauth2_scheme vẫn cần một chuỗi để đọc
    app.dependency_overrides[get_token_service] = lambda: FakeTokens(Principal("DR-001", frozenset({"driver"})))
    forbidden = await client.post("/v1/quotes", json=QUOTE, headers=bearer)
    app.dependency_overrides[get_token_service] = lambda: FakeTokens(Principal("CU-001", frozenset({"customer"})))
    allowed = await client.post("/v1/quotes", json=QUOTE, headers=bearer)

    assert (forbidden.status_code, allowed.status_code) == (403, 201)


async def test_overriding_current_principal_removes_the_scope_check_too(app, client):
    # Bẫy: kiểm scope nằm TRONG current_principal. Ghi đè chính nó là tháo luôn hàng rào phân quyền.
    app.dependency_overrides[current_principal] = lambda: Principal("DR-001", frozenset({"driver"}))

    response = await client.post("/v1/quotes", json=QUOTE)  # không header, scope driver, vẫn qua

    assert response.status_code == 201


def test_test_secret_is_not_the_lab_secret():
    from app.settings import LAB_DEV_SECRET

    assert TEST_SECRET != LAB_DEV_SECRET
