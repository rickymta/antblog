"""Hợp đồng HTTP: model của báo giá và của lỗi, copy từ bài 32 (thêm `request_id` như bài 33); thêm model của chuyến và của token."""

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
    request_id: str | None = Field(default=None, description="Mã yêu cầu (bài 33), trùng header X-Request-Id.")


class TokenResponse(BaseModel):
    """Thân response của POST /v1/auth/token theo RFC 6749 mục 5.1."""

    access_token: str
    token_type: str = "bearer"
    expires_in: int = Field(description="Số giây token còn hiệu lực.")
    scope: str = Field(description="Các scope đã cấp, cách nhau bằng dấu cách.")


class TripResponse(BaseModel):
    trip_id: str
    customer_id: str
    driver_id: str | None
    service: ServiceKind
    distance_km: Decimal
    status: TripStatus
