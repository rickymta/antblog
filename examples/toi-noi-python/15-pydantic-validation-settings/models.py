"""Mô hình dữ liệu ở biên hệ thống: hồ sơ tài xế và yêu cầu báo giá (pydantic v2).

Dữ liệu trong lab là dữ liệu giả: CCCD bắt đầu bằng 099 (mã tỉnh không tồn tại),
GPLX bắt đầu bằng 99, số điện thoại dạng 0900 000 001.
"""

import re
from dataclasses import dataclass
from decimal import Decimal
from enum import StrEnum
from typing import Any, Literal, Self

from pydantic import (
    AliasChoices,
    BaseModel,
    ConfigDict,
    Field,
    ValidationError,
    ValidationInfo,
    field_validator,
    model_validator,
)
from pydantic_core import PydanticCustomError

# Sáu thành phố của Tới Nơi. Literal giữ danh sách đóng; sai chính tả là lỗi `literal_error`.
City = Literal["Hà Nội", "TP.HCM", "Đà Nẵng", "Hải Phòng", "Cần Thơ", "Nha Trang"]


class ServiceKind(StrEnum):
    RIDE = "ride"
    FOOD = "food"
    PARCEL = "parcel"


# --- 1. dataclass thường: gõ gì nhận nấy, không kiểm tra lúc chạy -----------------------
@dataclass
class DriverRecord:
    driver_id: str
    full_name: str
    phone: str


# --- 2. BaseModel: kiểm tra, chuẩn hóa và báo lỗi ngay khi dựng -------------------------
class DriverProfile(BaseModel):
    model_config = ConfigDict(
        str_strip_whitespace=True,  # cắt khoảng trắng đầu/cuối mọi chuỗi trước khi kiểm
        extra="forbid",  # khóa lạ trong payload là lỗi, không bị bỏ qua lặng lẽ
        validate_assignment=True,  # gán lại thuộc tính cũng chạy lại validator (xem giới hạn trong bài)
        hide_input_in_errors=True,  # str(ValidationError) không in input_value: số giấy tờ không vào log
    )

    driver_id: str = Field(pattern=r"^DR-[0-9]{3}$")  # [0-9], không dùng \d: \d khớp cả chữ số Ả Rập
    # alias: payload cũ của app di động dùng "fullName"; payload mới dùng "full_name"
    full_name: str = Field(
        min_length=2, max_length=80, validation_alias=AliasChoices("full_name", "fullName")
    )
    # repr=False: số giấy tờ không xuất hiện trong repr()/log vô tình
    id_number: str = Field(repr=False)
    license_number: str = Field(repr=False)
    phone: str
    city: City

    @field_validator("id_number", "license_number")
    @classmethod
    def twelve_digits(cls, value: str, info: ValidationInfo) -> str:
        """Chỉ kiểm hình dạng (12 chữ số), không kiểm số có thật hay không."""
        if not re.fullmatch(r"[0-9]{12}", value):
            raise PydanticCustomError(
                "digits_12",
                "{field} phải gồm đúng 12 chữ số",
                {"field": info.field_name},
            )
        return value

    @field_validator("full_name")
    @classmethod
    def collapse_spaces(cls, value: str) -> str:
        return " ".join(value.split())  # "Nguyễn   Văn A" -> "Nguyễn Văn A"

    @field_validator("phone")
    @classmethod
    def normalize_phone(cls, value: str) -> str:
        """'0900 000 001' hay '0900.000.001' đều lưu thành '0900000001'."""
        digits = re.sub(r"[ .\-]", "", value)
        if not re.fullmatch(r"0[0-9]{9}", digits):
            raise PydanticCustomError(
                "phone_format", "số điện thoại phải gồm 10 chữ số bắt đầu bằng 0"
            )
        return digits

    @model_validator(mode="after")
    def documents_are_distinct(self) -> Self:
        """Quy tắc chéo trường của lab: hai giấy tờ không trùng số (bắt lỗi dán nhầm cột)."""
        if self.id_number == self.license_number:
            raise PydanticCustomError(
                "documents_same", "CCCD và GPLX không được cùng một số"
            )
        return self


class FrozenDriverProfile(DriverProfile):
    """Cùng quy tắc, nhưng bất biến: gán lại thuộc tính là lỗi `frozen_instance`."""

    model_config = ConfigDict(frozen=True)


def updated(profile: DriverProfile, **changes: Any) -> DriverProfile:
    """Dựng hồ sơ MỚI từ hồ sơ cũ cộng thay đổi; kiểm lại toàn bộ, hồ sơ cũ giữ nguyên."""
    return type(profile).model_validate({**profile.model_dump(), **changes})


class QuoteBody(BaseModel):
    """Hai trường của yêu cầu báo giá, chưa có quy tắc nào khác."""

    distance_km: Decimal = Field(ge=0, le=100, decimal_places=3)
    service: ServiceKind = ServiceKind.RIDE


class QuoteRequest(QuoteBody):
    """Chuỗi "2.5" và số nguyên đều thành Decimal; số thực (float) bị từ chối."""

    @field_validator("distance_km", mode="before")
    @classmethod
    def reject_float(cls, value: Any) -> Any:
        """Chạy TRƯỚC khi pydantic ép kiểu. float đã mất độ chính xác (bài 02): đòi chuỗi hoặc int."""
        if isinstance(value, float):
            raise PydanticCustomError(
                "float_not_allowed",
                "khoảng cách không được là số thực; gửi chuỗi như \"2.5\" hoặc số nguyên",
            )
        return value


class StrictQuoteBody(QuoteBody):
    """Thêm strict=True: python mode đòi đúng Decimal. Đường JSON vẫn nhận chuỗi (xem bài)."""

    model_config = ConfigDict(strict=True)


class StrictQuoteRequest(QuoteRequest):
    """strict=True cộng validator before: quan sát ở bản đã chạy, JSON chuỗi cũng bị từ chối."""

    model_config = ConfigDict(strict=True)


def error_report(exc: ValidationError) -> list[dict[str, str]]:
    """Đổi ValidationError thành danh sách phẳng, ổn định cho client (field, code, message)."""
    return [
        {
            "field": ".".join(str(part) for part in err["loc"]),
            "code": err["type"],
            "message": err["msg"],
        }
        for err in exc.errors(include_url=False, include_context=False)
    ]


def sample_payload(**overrides: Any) -> dict[str, Any]:
    """Payload hợp lệ của DR-001; ghi đè từng khóa để dựng ca lỗi."""
    base: dict[str, Any] = {
        "driver_id": "DR-001",
        "full_name": "Nguyễn Văn A",
        "id_number": "099000000001",
        "license_number": "990000000001",
        "phone": "0900 000 001",
        "city": "Hà Nội",
    }
    return {**base, **overrides}
