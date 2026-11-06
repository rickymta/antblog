"""Hợp đồng HTTP: model request/response của báo giá (như bài 31) và model của lỗi (RFC 9457)."""

from datetime import datetime
from decimal import Decimal

from pydantic import BaseModel, ConfigDict, Field

from app.domain import ServiceKind


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


class FieldProblem(BaseModel):
    """Một lỗi theo trường. Client rẽ nhánh theo `code`, hiển thị `message`."""

    field: str = Field(examples=["body.distance_km"])
    code: str = Field(examples=["decimal_max_places"])
    message: str


class ProblemDetails(BaseModel):
    """Đối tượng lỗi theo RFC 9457 (media type application/problem+json).

    `extra="allow"` vì RFC cho phép thành viên mở rộng (ví dụ min_km, max_km).
    """

    model_config = ConfigDict(extra="allow")

    type: str = Field(default="about:blank", examples=["https://toinoi.example/problems/distance-out-of-range"])
    title: str = Field(description="Tóm tắt ngắn, cố định theo `type`.")
    status: int = Field(description="Mã HTTP, lặp lại cho tiện đọc từ body.")
    detail: str | None = Field(default=None, description="Giải thích riêng cho lần lỗi này.")
    instance: str | None = Field(default=None, description="Đường dẫn của request gây lỗi.")
    errors: list[FieldProblem] | None = Field(default=None, description="Chỉ có ở lỗi validation.")
