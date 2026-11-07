"""Handler: mỗi command/query đúng một handler. Phần thân giữ nguyên logic use case của bài 67."""

from kyc_service.application.commands import RecordReviewDecision, SubmitDriverApplication
from kyc_service.application.errors import ApplicationNotFound, PortFailure
from kyc_service.application.ports import (
    ApplicationReadModel,
    Clock,
    FaceMatchPort,
    OcrPort,
    RiskScorePort,
    UnitOfWork,
)
from kyc_service.application.queries import GetApplication, ListReviewQueue
from kyc_service.application.views import ApplicationView
from kyc_service.domain.kyc import DocumentImage, DocumentKind, KycApplication, KycStatus


class SubmitDriverApplicationHandler:
    def __init__(
        self,
        uow: UnitOfWork,
        ocr: OcrPort,
        face: FaceMatchPort,
        risk: RiskScorePort,
        clock: Clock,
    ) -> None:
        self._uow = uow
        self._ocr = ocr
        self._face = face
        self._risk = risk
        self._clock = clock

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

        pending: str | None = None
        try:
            application.record_ocr(tuple(self._ocr.read(d) for d in application.documents))
            if application.face_consent:
                application.record_face(
                    self._face.compare(application.id_document, application.selfie_ref)
                )
                application.record_risk(self._risk.score(application))
                application.send_to_review("kết quả kiểm tra tự động là đầu vào cho người duyệt")
            else:
                application.send_to_review("chưa có đồng ý so khớp khuôn mặt")
        except PortFailure as failure:
            # Lỗi đã lường trước: hồ sơ được nhận, dừng ở trạng thái đã đạt. KHÔNG ném lên,
            # nên transaction commit; chỉ lỗi bất ngờ mới làm transaction rollback.
            pending = failure.service
        repository.save(application)
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
