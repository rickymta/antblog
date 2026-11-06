"""Fixture dùng chung: app mới cho mỗi test, client async có lifespan, token tự ký, đồng hồ cố định."""

from collections.abc import AsyncIterator, Callable
from datetime import UTC, datetime, timedelta

import httpx
import jwt
import pytest
from fastapi import FastAPI

from app.main import create_app, get_clock
from app.settings import Settings

# Khóa ký của TEST, độc lập với giá trị lab: test tự ký token bằng jwt.encode chứ không gọi code phát token của app.
TEST_SECRET = "lab36-test-secret-" + "0123456789abcdef" * 3  # 66 byte: đủ dài cho cả HS256 lẫn HS512
ISSUER, AUDIENCE = "toinoi-lab36", "toinoi-api"
FIXED_NOW = datetime(2026, 11, 6, 3, 0, tzinfo=UTC)


def make_settings() -> Settings:
    return Settings(environment="dev", jwt_secret=TEST_SECRET, jwt_issuer=ISSUER, jwt_audience=AUDIENCE)


def pytest_addoption(parser: pytest.Parser) -> None:
    parser.addoption("--update-snapshots", action="store_true", help="ghi lại tests/snapshots/*.json")


@pytest.fixture
def update_snapshots(request: pytest.FixtureRequest) -> bool:
    return request.config.getoption("--update-snapshots")


@pytest.fixture
def app() -> FastAPI:
    """App MỚI cho mỗi test: dependency_overrides và app.state không rò rỉ giữa các test."""
    application = create_app(make_settings())
    application.dependency_overrides[get_clock] = lambda: lambda: FIXED_NOW
    return application


@pytest.fixture
async def client(app: FastAPI) -> AsyncIterator[httpx.AsyncClient]:
    # ASGITransport KHÔNG chạy lifespan; tự mở bằng router.lifespan_context để app.state.repo tồn tại.
    async with app.router.lifespan_context(app):
        transport = httpx.ASGITransport(app=app)
        async with httpx.AsyncClient(transport=transport, base_url="http://test") as http:
            yield http


MakeToken = Callable[..., dict[str, str]]


@pytest.fixture
def auth() -> MakeToken:
    """auth(sub="CU-001", scope="customer") -> {"Authorization": "Bearer ..."}; tham số khác để làm hỏng token."""

    def make(sub: str = "CU-001", scope: str | list[str] = "customer", *,
             expires_in: timedelta = timedelta(minutes=15), secret: str = TEST_SECRET,
             algorithm: str = "HS256", issuer: str = ISSUER, audience: str = AUDIENCE,
             omit: tuple[str, ...] = ()) -> dict[str, str]:
        now = datetime.now(UTC)
        claims = {"iss": issuer, "aud": audience, "sub": sub, "scope": scope, "iat": now,
                  "exp": now + expires_in}
        for name in omit:
            del claims[name]
        token = jwt.encode(claims, secret, algorithm=algorithm)
        return {"Authorization": f"Bearer {token}"}

    return make
