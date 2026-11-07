"""Composition root: ghép adapter vào port, handler vào mediator, behavior vào pipeline."""

import logging
import warnings
from collections.abc import Callable
from dataclasses import dataclass

from sqlalchemy.engine import Engine

from kyc_service.application.behaviors import (
    logging_behavior,
    transaction_behavior,
    validation_behavior,
)
from kyc_service.application.commands import (
    RecordReviewDecision,
    ResumeScreening,
    SubmitDriverApplication,
)
from kyc_service.application.handlers import (
    GetApplicationHandler,
    ListReviewQueueHandler,
    RecordReviewDecisionHandler,
    ResumeScreeningHandler,
    Screening,
    SubmitDriverApplicationHandler,
)
from kyc_service.application.integration_events import TOPIC
from kyc_service.application.mediator import Mediator
from kyc_service.application.ports import (
    Clock,
    FaceMatchPort,
    IdGenerator,
    OcrPort,
    RiskScorePort,
)
from kyc_service.application.queries import GetApplication, ListReviewQueue
from kyc_service.domain.policy import ReviewPolicy
from kyc_service.infrastructure.clocks import SystemClock
from kyc_service.infrastructure.fake_services import FakeFaceMatch, FakeOcr, FakeRiskScore
from kyc_service.infrastructure.ids import UuidGenerator
from kyc_service.infrastructure.kafka.settings import topic_name
from kyc_service.infrastructure.sql.outbox import OutboxEventPublisher
from kyc_service.infrastructure.sql.read_model import SqlAlchemyReadModel
from kyc_service.infrastructure.sql.unit_of_work import SqlAlchemyUnitOfWork, make_session_factory


class FakeAdapterWarning(UserWarning):
    """`build_container` đang dùng adapter GIẢ cho một hoặc nhiều service."""


@dataclass(frozen=True)
class Container:
    mediator: Mediator
    uow: SqlAlchemyUnitOfWork  # test cần mở thêm Unit of Work thứ hai để tái hiện xung đột


def build_container(
    engine: Engine,
    *,
    ocr: OcrPort | None = None,
    face: FaceMatchPort | None = None,
    risk: RiskScorePort | None = None,
    clock: Clock | None = None,
    policy: ReviewPolicy | None = None,
    emit_version: int,  # phiên bản hợp đồng producer đang phát: cấu hình triển khai, không mặc định
    topic: str | None = None,
    ids: IdGenerator | None = None,
    log: Callable[[str], None] | None = None,
) -> Container:
    ports = (("ocr", ocr), ("face", face), ("risk", risk))
    fake = [name for name, adapter in ports if adapter is None]
    if fake:  # không fallback âm thầm: port nào không truyền thì dùng adapter giả và CẢNH BÁO
        warnings.warn(
            FakeAdapterWarning(
                f"build_container() dùng adapter giả cho: {', '.join(fake)}. Chỉ cho demo và test."
            ),
            stacklevel=2,
        )
    clock = clock or SystemClock()
    screening = Screening(
        ocr or FakeOcr(),
        face or FakeFaceMatch(),
        risk or FakeRiskScore(),
        clock,
        policy or ReviewPolicy(),
    )
    session_factory = make_session_factory(engine)
    uow = SqlAlchemyUnitOfWork(session_factory)
    read_model = SqlAlchemyReadModel(session_factory)
    # Domain event đi vào bảng outbox bằng chính Session của transaction đang mở.
    events = OutboxEventPublisher(
        uow,
        topic=topic or topic_name(TOPIC),
        emit_version=emit_version,
        ids=ids or UuidGenerator(),
        clock=clock,
    )
    log = log or logging.getLogger("kyc.pipeline").info

    # Thứ tự là hành vi: log bọc ngoài cùng (thấy cả lỗi validate), validate chạy TRƯỚC khi
    # mở transaction (thông điệp sai không tốn transaction), transaction sát handler nhất.
    mediator = Mediator([logging_behavior(log), validation_behavior(), transaction_behavior(uow)])
    mediator.register(
        SubmitDriverApplication, SubmitDriverApplicationHandler(uow, screening, clock, events)
    )
    mediator.register(ResumeScreening, ResumeScreeningHandler(uow, screening, events))
    mediator.register(RecordReviewDecision, RecordReviewDecisionHandler(uow, clock))
    mediator.register(GetApplication, GetApplicationHandler(read_model))
    mediator.register(ListReviewQueue, ListReviewQueueHandler(read_model))
    return Container(mediator, uow)
