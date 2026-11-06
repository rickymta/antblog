"""Hợp đồng HTTP của endpoint báo giá: hai model pydantic v2, tách khỏi model miền."""

from datetime import datetime
from decimal import Decimal

from pydantic import BaseModel, ConfigDict, Field

from app.domain import ServiceKind


class QuoteRequest(BaseModel):
    """Body của POST /v1/quotes.

    Chỉ kiểm HÌNH DẠNG: có đủ trường, đúng kiểu, tối đa 3 chữ số thập phân, hữu hạn.
    Miền 0..100 km là quy tắc nghiệp vụ, nằm ở calculate_fare (khác biệt 422 và 400).
    """

    model_config = ConfigDict(extra="forbid")

    distance_km: Decimal = Field(decimal_places=3, allow_inf_nan=False, examples=["2.5"])
    service: ServiceKind = ServiceKind.RIDE


class QuoteResponse(BaseModel):
    """Báo giá trả về. Decimal ra JSON là chuỗi ("14250"), không phải số thực."""

    quote_id: str = Field(examples=["QT-0001"])
    distance_km: Decimal
    service: ServiceKind
    fare: Decimal = Field(description="Cước VND, số nguyên dạng chuỗi.", examples=["14250"])
    currency: str = "VND"
    expires_at: datetime = Field(description="UTC, hết hạn sau 5 phút.")
