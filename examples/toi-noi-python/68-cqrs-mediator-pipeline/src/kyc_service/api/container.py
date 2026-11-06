"""Composition root: ghép adapter vào port, handler vào mediator, behavior vào pipeline."""

import logging
import warnings
from collections.abc import Callable
from dataclasses import dataclass

from kyc_service.application.behaviors import (
    logging_behavior,
    transaction_behavior,
    validation_behavior,
)
from kyc_service.application.commands import RecordReviewDecision, SubmitDriverApplication
from kyc_service.application.handlers import (
    GetApplicationHandler,
    ListReviewQueueHandler,
    RecordReviewDecisionHandler,
    SubmitDriverApplicationHandler,
)
from kyc_service.application.mediator import Mediator
from kyc_service.application.ports import Clock, FaceMatchPort, OcrPort, RiskScorePort
from kyc_service.application.queries import GetApplication, ListReviewQueue
from kyc_service.infrastructure.clocks import SystemClock
from kyc_service.infrastructure.fake_services import FakeFaceMatch, FakeOcr, FakeRiskScore
from kyc_service.infrastructure.memory_read_model import InMemoryReadModel
from kyc_service.infrastructure.memory_repository import InMemoryKycRepository
from kyc_service.infrastructure.memory_transactions import InMemoryTransactions


@dataclass(frozen=True)
class Container:
    mediator: Mediator
    repository: InMemoryKycRepository  # test và demo cần soi trạng thái lưu
    transactions: InMemoryTransactions


def build_container(
    *,
    repository: InMemoryKycRepository | None = None,
    ocr: OcrPort | None = None,
    face: FaceMatchPort | None = None,
    risk: RiskScorePort | None = None,
    clock: Clock | None = None,
    log: Callable[[str], None] | None = None,
) -> Container:
    # Không fallback âm thầm: port nào không truyền thì dùng adapter giả và CẢNH BÁO.
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
    transactions = InMemoryTransactions(repository)
    read_model = InMemoryReadModel(repository)
    log = log or logging.getLogger("kyc.pipeline").info

    # Thứ tự là hành vi: log bọc ngoài cùng (thấy cả lỗi validate), validate chạy TRƯỚC khi
    # mở transaction (thông điệp sai không tốn transaction), transaction sát handler nhất.
    mediator = Mediator(
        [logging_behavior(log), validation_behavior(), transaction_behavior(transactions)]
    )
    mediator.register(
        SubmitDriverApplication,
        SubmitDriverApplicationHandler(
            repository, ocr or FakeOcr(), face or FakeFaceMatch(), risk or FakeRiskScore(), clock
        ),
    )
    mediator.register(RecordReviewDecision, RecordReviewDecisionHandler(repository, clock))
    mediator.register(GetApplication, GetApplicationHandler(read_model))
    mediator.register(ListReviewQueue, ListReviewQueueHandler(read_model))
    return Container(mediator, repository, transactions)
