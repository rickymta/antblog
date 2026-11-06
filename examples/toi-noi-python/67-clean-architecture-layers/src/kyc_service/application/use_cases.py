"""Use case: điều phối domain và port, không chứa quy tắc nghiệp vụ và không biết HTTP.

Hai việc ghi (nộp hồ sơ, người duyệt quyết định) và một việc đọc. Bài 68 đổi mỗi use case thành
command/query handler; bài 70 thêm ngưỡng, domain event và bảng ca.
"""

from dataclasses import dataclass
from datetime import datetime

from kyc_service.application.errors import ApplicationNotFound, PortFailure
from kyc_service.application.ports import (
    Clock,
    FaceMatchPort,
    KycRepository,
    OcrPort,
    RiskScorePort,
)
from kyc_service.application.views import ApplicationView
from kyc_service.domain.kyc import DocumentImage, DocumentKind, KycApplication, KycStatus


@dataclass(frozen=True)
class SubmitDriverApplicationRequest:
    driver_id: str
    cccd_ref: str
    gplx_ref: str
    selfie_ref: str
    face_consent: bool


class SubmitDriverApplicationUseCase:
    """Nhận hồ sơ, gọi OCR -> khuôn mặt -> điểm rủi ro qua port, rồi đưa vào hàng chờ duyệt.

    Mọi hồ sơ đi hết đường đều dừng ở NEEDS_REVIEW: máy không tự duyệt hay từ chối ai.
    """

    def __init__(
        self,
        repository: KycRepository,
        ocr: OcrPort,
        face: FaceMatchPort,
        risk: RiskScorePort,
        clock: Clock,
    ) -> None:
        self._repository = repository
        self._ocr = ocr
        self._face = face
        self._risk = risk
        self._clock = clock

    def execute(self, request: SubmitDriverApplicationRequest) -> ApplicationView:
        application = KycApplication.submit(
            application_id=self._repository.next_id(),
            driver_id=request.driver_id,
            documents=(
                DocumentImage(DocumentKind.CCCD, request.cccd_ref),
                DocumentImage(DocumentKind.GPLX, request.gplx_ref),
            ),
            selfie_ref=request.selfie_ref,
            face_consent=request.face_consent,
            now=self._clock.now(),
        )
        self._repository.add(application)  # nhận hồ sơ trước: service ngoài hỏng cũng không mất

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
                # Bảng chuyển cho phép OCR_DONE -> NEEDS_REVIEW, không cho nhảy sang điểm rủi ro.
                application.send_to_review("chưa có đồng ý so khớp khuôn mặt")
        except PortFailure as failure:
            pending = failure.service  # hồ sơ dừng ở trạng thái đã đạt, chờ kiểm lại sau
        self._repository.save(application)
        return ApplicationView.of(application, pending_check=pending)


class ReviewApplicationUseCase:
    def __init__(self, repository: KycRepository, clock: Clock) -> None:
        self._repository = repository
        self._clock = clock

    def execute(
        self, application_id: str, *, reviewer_id: str, decision: KycStatus, reason: str
    ) -> ApplicationView:
        application = self._repository.get(application_id)
        if application is None:
            raise ApplicationNotFound(application_id)
        now: datetime = self._clock.now()
        application.decide(reviewer_id=reviewer_id, decision=decision, reason=reason, now=now)
        self._repository.save(application)
        return ApplicationView.of(application)


class GetApplicationUseCase:
    def __init__(self, repository: KycRepository) -> None:
        self._repository = repository

    def execute(self, application_id: str) -> ApplicationView:
        application = self._repository.get(application_id)
        if application is None:
            raise ApplicationNotFound(application_id)
        return ApplicationView.of(application)
