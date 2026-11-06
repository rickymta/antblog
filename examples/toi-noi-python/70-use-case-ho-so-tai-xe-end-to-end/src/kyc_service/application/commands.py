"""Command: ý định thay đổi trạng thái. Kiểu chú thích mô tả hình dạng hợp lệ; behavior
validation đọc chính các chú thích này (pydantic), nên khai báo một lần, dùng cho mọi nơi gửi."""

from dataclasses import dataclass
from typing import Annotated, Literal

from pydantic import StrictBool, StringConstraints

from kyc_service.application.messages import Command
from kyc_service.application.views import ApplicationView

ApplicationId = Annotated[str, StringConstraints(pattern=r"^KYC-[0-9]{4,}$")]
DriverId = Annotated[str, StringConstraints(pattern=r"^DR-[0-9]{3}$")]
ReviewerId = Annotated[str, StringConstraints(pattern=r"^RV-[0-9]{3}$")]
ImageRef = Annotated[str, StringConstraints(strip_whitespace=True, min_length=1)]
Reason = Annotated[str, StringConstraints(strip_whitespace=True, min_length=1)]


@dataclass(frozen=True)
class SubmitDriverApplication(Command[ApplicationView]):
    driver_id: DriverId
    cccd_ref: ImageRef
    gplx_ref: ImageRef
    selfie_ref: ImageRef
    face_consent: StrictBool  # đồng ý là quyết định pháp lý: "false" dạng chuỗi không được đoán


@dataclass(frozen=True)
class RecordReviewDecision(Command[ApplicationView]):
    application_id: ApplicationId
    reviewer_id: ReviewerId
    decision: Literal["approved", "rejected"]
    reason: Reason


@dataclass(frozen=True)
class ResumeScreening(Command[ApplicationView]):
    """Làm tiếp các bước kiểm tra còn dở của một hồ sơ (service ngoài đã sống lại)."""

    application_id: ApplicationId
