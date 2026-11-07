"""Hình dạng JSON của API. Pydantic ở đây chỉ kiểm kiểu và trường lạ; định dạng mã do behavior
validation của application kiểm, nên worker hay CLI gửi command cũng được kiểm như API."""

from pydantic import BaseModel, ConfigDict, StrictBool


class SubmitBody(BaseModel):
    # forbid: client không được gửi kèm `status` hay trường lạ để tự đặt trạng thái hồ sơ.
    model_config = ConfigDict(extra="forbid")

    driver_id: str
    cccd_ref: str
    gplx_ref: str
    selfie_ref: str
    face_consent: StrictBool  # chỉ true/false của JSON; chuỗi "false" bị từ chối, không đổi ngầm


class ReviewBody(BaseModel):
    model_config = ConfigDict(extra="forbid")

    reviewer_id: str
    decision: str
    reason: str


class OcrResponse(BaseModel):
    kind: str
    id_number_masked: str | None
    confidence: float


class ApplicationResponse(BaseModel):
    application_id: str
    driver_id: str
    status: str
    ocr: list[OcrResponse]
    face_score: float | None
    risk_score: float | None
    risk_reasons: list[str]
    queue_reasons: list[str]
    note: str | None
    reviewed_by: str | None
    pending_check: str | None
