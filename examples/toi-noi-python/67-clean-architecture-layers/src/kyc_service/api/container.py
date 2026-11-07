"""Composition root: nơi DUY NHẤT biết adapter cụ thể nào được cắm vào port nào."""

import warnings
from dataclasses import dataclass

from kyc_service.application.ports import (
    Clock,
    FaceMatchPort,
    KycRepository,
    OcrPort,
    RiskScorePort,
)
from kyc_service.application.use_cases import (
    GetApplicationUseCase,
    ReviewApplicationUseCase,
    SubmitDriverApplicationUseCase,
)
from kyc_service.infrastructure.clocks import SystemClock
from kyc_service.infrastructure.fake_services import FakeFaceMatch, FakeOcr, FakeRiskScore
from kyc_service.infrastructure.memory_repository import InMemoryKycRepository


@dataclass(frozen=True)
class Container:
    submit: SubmitDriverApplicationUseCase
    review: ReviewApplicationUseCase
    get: GetApplicationUseCase


def build_container(
    *,
    repository: KycRepository | None = None,
    ocr: OcrPort | None = None,
    face: FaceMatchPort | None = None,
    risk: RiskScorePort | None = None,
    clock: Clock | None = None,
) -> Container:
    """Port nào không truyền thì dùng adapter giả và CẢNH BÁO; test truyền adapter riêng.

    Không fallback âm thầm: một bản chạy thật mà quên truyền adapter OCR sẽ đọc giấy tờ bằng
    bảng tra cố định, nên việc dùng adapter giả phải lộ ra.
    """
    fallbacks = [
        name
        for name, given in (
            ("repository", repository),
            ("ocr", ocr),
            ("face", face),
            ("risk", risk),
        )
        if given is None
    ]
    if fallbacks:
        warnings.warn(
            f"build_container() dùng adapter giả cho: {', '.join(fallbacks)}. "
            "Chỉ cho demo và test.",
            UserWarning,
            stacklevel=2,
        )
    repository = repository or InMemoryKycRepository()
    clock = clock or SystemClock()
    return Container(
        submit=SubmitDriverApplicationUseCase(
            repository, ocr or FakeOcr(), face or FakeFaceMatch(), risk or FakeRiskScore(), clock
        ),
        review=ReviewApplicationUseCase(repository, clock),
        get=GetApplicationUseCase(repository),
    )
