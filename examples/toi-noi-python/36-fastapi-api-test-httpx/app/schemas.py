"""Hợp đồng HTTP: báo giá y hệt lab 32 (extra=forbid, decimal_places=3, quote_id), chuyến, và lỗi RFC 9457."""

from datetime import datetime
from decimal import Decimal

from pydantic import BaseModel, ConfigDict, Field

from app.domain import ServiceKind, TripStatus


class QuoteRequest(BaseModel):
    """Body của POST /v1/quotes. Chỉ kiểm hình dạng; miền 0..100 km là việc của miền (400)."""

    model_config = ConfigDict(extra="forbid")

    distance_km: Decimal = Field(
        decimal_places=3,
        allow_inf_nan=False,
        description="Khoảng cách km, tối đa 3 chữ số thập phân. Nên gửi dạng chuỗi.",
        examples=["2.5"],
    )
    service: ServiceKind = ServiceKind.RIDE


class QuoteResponse(BaseModel):
    """Báo giá trả về. Decimal ra JSON là chuỗi ("14250"), không phải số thực."""

    quote_id: str = Field(examples=["QT-0001"])
    distance_km: Decimal
    service: ServiceKind
    fare: Decimal = Field(description="Cước VND, số nguyên dạng chuỗi.", examples=["14250"])
    currency: str = "VND"
    expires_at: datetime = Field(description="UTC, hết hạn sau 5 phút.")


class TripOut(BaseModel):
    trip_id: str
    customer_id: str
    driver_id: str | None
    service: ServiceKind
    distance_km: Decimal
    status: TripStatus


class FieldProblem(BaseModel):
    field: str = Field(examples=["body.distance_km"])
    code: str = Field(examples=["decimal_max_places"])
    message: str


class ProblemDetails(BaseModel):
    """`extra="allow"` vì RFC cho phép thành viên mở rộng (min_km, max_km, request_id)."""

    model_config = ConfigDict(extra="allow")

    type: str = Field(default="about:blank")
    title: str
    status: int
    detail: str | None = None
    instance: str | None = None
    errors: list[FieldProblem] | None = Field(default=None, description="Chỉ có ở lỗi validation.")
