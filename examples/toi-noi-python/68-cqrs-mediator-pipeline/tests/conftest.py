from datetime import UTC, datetime

import pytest

from kyc_service.api.container import Container, build_container
from kyc_service.application.commands import SubmitDriverApplication
from kyc_service.infrastructure.clocks import FixedClock

NOW = datetime(2026, 11, 6, 3, 0, tzinfo=UTC)


@pytest.fixture
def events() -> list[str]:
    return []


@pytest.fixture
def container(events: list[str]) -> Container:
    return build_container(clock=FixedClock(NOW), log=events.append)


def submit_for(driver: str, *, consent: bool = True) -> SubmitDriverApplication:
    return SubmitDriverApplication(
        driver_id=driver,
        cccd_ref=f"synthetic/cccd/{driver}.png",
        gplx_ref=f"synthetic/gplx/{driver}.png",
        selfie_ref=f"synthetic/selfie/{driver}.png",
        face_consent=consent,
    )
