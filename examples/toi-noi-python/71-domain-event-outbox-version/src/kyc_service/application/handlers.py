"""Handler: mỗi command/query đúng một handler. Use case nộp hồ sơ nằm ở đây."""

from collections.abc import Callable

from kyc_service.application.commands import (
    RecordReviewDecision,
    ResumeScreening,
    SubmitDriverApplication,
)
from kyc_service.application.errors import (
    ApplicationNotFound,
    InvalidServiceResponse,
    PortFailure,
)
from kyc_service.application.ports import (
    ApplicationReadModel,
    Clock,
    EventPublisher,
    FaceMatchPort,
    OcrPort,
    RiskScorePort,
    UnitOfWork,
)
from kyc_service.application.queries import GetApplication, ListReviewQueue
from kyc_service.application.views import ApplicationView
from kyc_service.domain.errors import DomainError, InvalidStatusTransition
from kyc_service.domain.kyc import DocumentImage, DocumentKind, KycApplication, KycStatus
from kyc_service.domain.policy import ReviewPolicy

# Trạng thái còn bước kiểm tra tự động phía trước.
SCREENING_STATES = frozenset(
    {KycStatus.SUBMITTED, KycStatus.OCR_DONE, KycStatus.FACE_CHECKED, KycStatus.RISK_SCORED}
)


class Screening:
    """Chuỗi kiểm tra tự động: OCR -> khuôn mặt -> điểm rủi ro -> hàng chờ duyệt.

    Mỗi bước chạy khi hồ sơ đang ở đúng trạng thái của nó, nên cùng một chuỗi dùng cho hồ sơ
    mới (từ `submitted`) và hồ sơ làm tiếp (dừng ở giữa vì một service tắt). Chuỗi KHÔNG BAO GIỜ
    đi tới `approved` hay `rejected`: đích cuối của máy là `needs_review`.
    """

    def __init__(
        self,
        ocr: OcrPort,
        face: FaceMatchPort,
        risk: RiskScorePort,
        clock: Clock,
        policy: ReviewPolicy,
    ) -> None:
        self._ocr = ocr
        self._face = face
        self._risk = risk
        self._clock = clock
        self._policy = policy

    def run(self, application: KycApplication) -> str | None:
        """Chạy các bước còn lại. Trả tên service chưa kiểm được, hoặc None nếu đã xong."""
        try:
            if application.status is KycStatus.SUBMITTED:
                self._accept(
                    "kyc-ocr",
                    lambda: application.record_ocr(
                        tuple(self._ocr.read(d) for d in application.documents)
                    ),
                )
            if application.status is KycStatus.OCR_DONE:
                if not application.face_consent:
                    self._queue(application)  # không đồng ý: không gọi kyc-face
                    return None
                self._accept(
                    "kyc-face",
                    lambda: application.record_face(
                        self._face.compare(application.id_document, application.selfie_ref)
                    ),
                )
            if application.status is KycStatus.FACE_CHECKED:
                self._accept(
                    "fraud-scoring", lambda: application.record_risk(self._risk.score(application))
                )
            if application.status is KycStatus.RISK_SCORED:
                self._queue(application)
        except PortFailure as failure:
            # Lỗi đã lường trước: hồ sơ được nhận, dừng ở trạng thái đã đạt. KHÔNG ném lên,
            # nên transaction commit; chỉ lỗi bất ngờ mới làm transaction rollback.
            return failure.service
        return None

    @staticmethod
    def _accept(service: str, step: Callable[[], None]) -> None:
        """Nhận kết quả của một service vào aggregate; kết quả sai quy tắc miền là lỗi của SERVICE.

        Adapter có thể trả đối tượng hợp lệ về kiểu mà vẫn sai nghĩa (OCR trả hai CCCD cho một hồ sơ
        gồm CCCD và GPLX). Domain từ chối bằng `DomainError`; không dịch thì lỗi đó rollback cả
        hồ sơ và thành 422 cho người nộp, dù lỗi nằm ở service.
        """
        try:
            step()
        except DomainError as exc:
            raise InvalidServiceResponse(service, str(exc)) from exc

    def _queue(self, application: KycApplication) -> None:
        application.send_to_review(self._policy.reasons(application), now=self._clock.now())


class SubmitDriverApplicationHandler:
    def __init__(
        self, uow: UnitOfWork, screening: Screening, clock: Clock, events: EventPublisher
    ) -> None:
        self._uow = uow
        self._screening = screening
        self._clock = clock
        self._events = events

    def handle(self, command: SubmitDriverApplication) -> ApplicationView:
        repository = self._uow.applications  # kho gắn với transaction của pipeline
        application = KycApplication.submit(
            application_id=repository.next_id(),
            driver_id=command.driver_id,
            documents=(
                DocumentImage(DocumentKind.CCCD, command.cccd_ref),
                DocumentImage(DocumentKind.GPLX, command.gplx_ref),
            ),
            selfie_ref=command.selfie_ref,
            face_consent=command.face_consent,
            now=self._clock.now(),
        )
        repository.add(application)
        pending = self._screening.run(application)
        repository.save(application)
        self._events.publish(application.pull_events())
        return ApplicationView.of(application, pending_check=pending)


class ResumeScreeningHandler:
    def __init__(self, uow: UnitOfWork, screening: Screening, events: EventPublisher) -> None:
        self._uow = uow
        self._screening = screening
        self._events = events

    def handle(self, command: ResumeScreening) -> ApplicationView:
        repository = self._uow.applications
        application = repository.get(command.application_id)
        if application is None:
            raise ApplicationNotFound(command.application_id)
        if application.status not in SCREENING_STATES:
            raise InvalidStatusTransition(
                f"hồ sơ {application.application_id} ở {application.status.value}: "
                "không còn bước kiểm tra tự động nào để làm tiếp"
            )
        pending = self._screening.run(application)
        repository.save(application)
        self._events.publish(application.pull_events())
        return ApplicationView.of(application, pending_check=pending)


class RecordReviewDecisionHandler:
    def __init__(self, uow: UnitOfWork, clock: Clock) -> None:
        self._uow = uow
        self._clock = clock

    def handle(self, command: RecordReviewDecision) -> ApplicationView:
        repository = self._uow.applications
        application = repository.get(command.application_id)
        if application is None:
            raise ApplicationNotFound(command.application_id)
        application.decide(
            reviewer_id=command.reviewer_id,
            decision=KycStatus(command.decision),
            reason=command.reason,
            now=self._clock.now(),
        )
        repository.save(application)
        return ApplicationView.of(application)


class GetApplicationHandler:
    def __init__(self, read_model: ApplicationReadModel) -> None:
        self._read_model = read_model

    def handle(self, query: GetApplication) -> ApplicationView:
        view = self._read_model.get(query.application_id)
        if view is None:
            raise ApplicationNotFound(query.application_id)
        return view


class ListReviewQueueHandler:
    def __init__(self, read_model: ApplicationReadModel) -> None:
        self._read_model = read_model

    def handle(self, query: ListReviewQueue) -> tuple[ApplicationView, ...]:
        return self._read_model.review_queue()
