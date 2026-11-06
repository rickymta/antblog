"""Bài 32: validation, Problem Details và hợp đồng OpenAPI cho API báo giá.

Chạy server (cổng 8032):
  uv run --project examples/toi-noi-python/32-fastapi-validation-problem-details \\
      uvicorn app.main:app --app-dir examples/toi-noi-python/32-fastapi-validation-problem-details --port 8032
"""

import itertools
from datetime import UTC, datetime
from decimal import Decimal

from fastapi import APIRouter, FastAPI, Request, Response

from app import problems
from app.domain import QUOTE_TTL, Money, Quote, QuoteNotFound, calculate_fare
from app.schemas import ProblemDetails, QuoteRequest, QuoteResponse

TAGS = [{"name": "quotes", "description": "Báo giá cước theo khoảng cách. Chưa tạo chuyến, chưa giữ tiền."}]

# Lỗi chung cho mọi route của router. FastAPI tự thêm 422 mặc định (HTTPValidationError) vào route có tham số,
# nên phải khai báo lại để tài liệu khớp handler; `quote_id: str` hiện chưa thể gây 422, khai báo để tài liệu
# không đổi nghĩa khi kiểu tham số thay đổi.
COMMON_ERRORS = {
    422: {"model": ProblemDetails, "description": "Yêu cầu sai hình dạng (thiếu trường, sai kiểu, trường lạ)."},
    500: {"model": ProblemDetails, "description": "Lỗi nội bộ; chi tiết chỉ có trong log."},
}

v1 = APIRouter(prefix="/v1", tags=["quotes"], responses=COMMON_ERRORS)


@v1.post(
    "/quotes",
    operation_id="createQuote",
    summary="Tạo báo giá",
    status_code=201,
    response_model=QuoteResponse,
    responses={400: {"model": ProblemDetails, "description": "Khoảng cách ngoài miền 0..100 km."}},
)
async def create_quote(body: QuoteRequest, request: Request, response: Response) -> QuoteResponse:
    fare = calculate_fare(body.distance_km, service=body.service)  # DistanceOutOfRange -> handler -> 400
    quote = Quote(body.distance_km, Money(fare), datetime.now(UTC) + QUOTE_TTL)
    quote_id = f"QT-{next(request.app.state.counter):04d}"
    out = QuoteResponse(
        quote_id=quote_id,
        distance_km=Decimal(format(quote.trip_distance_km, "f")),  # "1e2" vào thì ra "100", không ra "1E+2"
        service=body.service,
        fare=quote.fare.amount,
        currency=quote.fare.currency,
        expires_at=quote.expires_at,
    )
    request.app.state.quotes[quote_id] = out
    response.headers["Location"] = f"/v1/quotes/{quote_id}"
    return out


@v1.get(
    "/quotes/{quote_id}",
    operation_id="getQuote",
    summary="Đọc báo giá",
    response_model=QuoteResponse,
    responses={404: {"model": ProblemDetails, "description": "Không có báo giá với id này."}},
)
async def get_quote(quote_id: str, request: Request) -> QuoteResponse:
    found = request.app.state.quotes.get(quote_id)
    if found is None:
        raise QuoteNotFound(quote_id)
    return found


def create_app(*, problem_details: bool = True) -> FastAPI:
    """`problem_details=False` dựng app y hệt nhưng giữ nguyên lỗi mặc định của FastAPI (để so sánh)."""
    app = FastAPI(
        title="Tới Nơi - Báo giá",
        # info.version là phiên bản hợp đồng (semver); /v1 trong URL chỉ là số major
        version="1.0.0",
        description="Bài 32: lỗi theo RFC 9457. Mô hình cước là giả định học tập.",
        openapi_tags=TAGS,
    )
    app.state.quotes = {}
    app.state.counter = itertools.count(1)
    if problem_details:
        problems.install(app)
    app.include_router(v1)
    if problem_details:
        problems.problem_media_type_in_openapi(app)
    return app


app = create_app()
