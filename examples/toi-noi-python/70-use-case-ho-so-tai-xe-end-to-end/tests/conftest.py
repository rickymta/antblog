"""Fixture dùng chung. Test chạm cơ sở dữ liệu cần PostgreSQL của hạ tầng chung đang chạy."""

from collections.abc import Iterator
from datetime import UTC, datetime

import pytest
from sqlalchemy import create_engine, event, text
from sqlalchemy.engine import Engine
from sqlalchemy.exc import OperationalError

from kyc_service.api.container import Container, build_container
from kyc_service.application.commands import SubmitDriverApplication
from kyc_service.infrastructure.clocks import FixedClock
from kyc_service.infrastructure.sql.schema import create_schema, drop_schema, reset_data
from kyc_service.infrastructure.sql.settings import COMPOSE_HINT, PG_URL, sqlalchemy_url

NOW = datetime(2026, 11, 6, 3, 0, tzinfo=UTC)


@pytest.fixture(scope="session")
def engine() -> Iterator[Engine]:
    eng = create_engine(sqlalchemy_url(), pool_pre_ping=True)
    try:
        with eng.connect() as conn:
            conn.execute(text("SELECT 1"))
    except OperationalError:
        pytest.fail(f"không nối được PostgreSQL tại {PG_URL} ({COMPOSE_HINT})", pytrace=False)
    create_schema(eng)
    yield eng
    drop_schema(eng)
    eng.dispose()


@pytest.fixture
def clean(engine: Engine) -> None:
    """Xóa dữ liệu và đưa sequence về đầu: mỗi test bắt đầu từ KYC-0001."""
    reset_data(engine)


@pytest.fixture
def tx_events(engine: Engine) -> Iterator[list[str]]:
    """Ghi lại begin/commit/rollback THẬT của kết nối PostgreSQL do SQLAlchemy báo."""
    seen: list[str] = []
    listeners = {
        "begin": lambda conn: seen.append("begin"),
        "commit": lambda conn: seen.append("commit"),
        "rollback": lambda conn: seen.append("rollback"),
    }
    for name, fn in listeners.items():
        event.listen(engine, name, fn)
    yield seen
    for name, fn in listeners.items():
        event.remove(engine, name, fn)


@pytest.fixture
def log_lines() -> list[str]:
    return []


@pytest.fixture
def container(engine: Engine, clean: None, log_lines: list[str]) -> Container:
    return build_container(engine, clock=FixedClock(NOW), log=log_lines.append)


def submit_for(driver: str, *, consent: bool = True) -> SubmitDriverApplication:
    return SubmitDriverApplication(
        driver_id=driver,
        cccd_ref=f"synthetic/cccd/{driver}.png",
        gplx_ref=f"synthetic/gplx/{driver}.png",
        selfie_ref=f"synthetic/selfie/{driver}.png",
        face_consent=consent,
    )
