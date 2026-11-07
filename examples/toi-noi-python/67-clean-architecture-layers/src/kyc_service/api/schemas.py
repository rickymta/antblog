"""Hình dạng JSON của API. Pydantic chỉ sống ở lớp này."""

from typing import Literal

from pydantic import BaseModel, ConfigDict, Field, StrictBool


class SubmitBody(BaseModel):
    # forbid: client không được gửi kèm `status` hay trường lạ để tự đặt trạng thái hồ sơ.
    model_config = ConfigDict(extra="forbid")

    driver_id: str = Field(pattern=r"^DR-[0-9]{3}$", examples=["DR-001"])
    cccd_ref: str = Field(min_length=1)
    gplx_ref: str = Field(min_length=1)
    selfie_ref: str = Field(min_length=1)
    face_consent: StrictBool  # chỉ true/false của JSON; chuỗi "false" bị từ chối, không đổi ngầm


class ReviewBody(BaseModel):
    model_config = ConfigDict(extra="forbid")

    reviewer_id: str = Field(pattern=r"^RV-[0-9]{3}$", examples=["RV-001"])
    decision: Literal["approved", "rejected"]
    reason: str = Field(min_length=1)


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
    note: str | None
    reviewed_by: str | None
    pending_check: str | None
