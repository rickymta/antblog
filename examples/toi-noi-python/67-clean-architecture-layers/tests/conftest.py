from datetime import UTC, datetime

import pytest

from kyc_service.api.container import Container, build_container
from kyc_service.application.use_cases import SubmitDriverApplicationRequest
from kyc_service.infrastructure.clocks import FixedClock
from kyc_service.infrastructure.memory_repository import InMemoryKycRepository

NOW = datetime(2026, 11, 6, 3, 0, tzinfo=UTC)


@pytest.fixture
def repo() -> InMemoryKycRepository:
    return InMemoryKycRepository()


@pytest.fixture
def container(repo: InMemoryKycRepository) -> Container:
    return build_container(repository=repo, clock=FixedClock(NOW))


def request_for(driver: str, *, consent: bool = True) -> SubmitDriverApplicationRequest:
    return SubmitDriverApplicationRequest(
        driver_id=driver,
        cccd_ref=f"synthetic/cccd/{driver}.png",
        gplx_ref=f"synthetic/gplx/{driver}.png",
        selfie_ref=f"synthetic/selfie/{driver}.png",
        face_consent=consent,
    )
