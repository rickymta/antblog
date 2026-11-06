"""Bài 31: endpoint báo giá của Tới Nơi bằng FastAPI.

Chạy server:
  uv run --project examples/toi-noi-python/31-fastapi-endpoint-bao-gia \\
      uvicorn app.main:app --app-dir examples/toi-noi-python/31-fastapi-endpoint-bao-gia --port 8031
"""

import itertools
from datetime import UTC, datetime
from decimal import Decimal

from fastapi import FastAPI, HTTPException, Request, Response

from app.domain import QUOTE_TTL, Money, Quote, calculate_fare
from app.schemas import QuoteRequest, QuoteResponse


def create_app() -> FastAPI:
    app = FastAPI(
        title="Tới Nơi - Báo giá",
        version="0.1.0",
        description="Bài 31: tính cước theo khoảng cách. Mô hình cước là giả định học tập.",
    )
    # «Cơ sở dữ liệu» của lab: dict trong bộ nhớ TIẾN TRÌNH, mất khi restart (xem phần giới hạn).
    app.state.quotes = {}  # quote_id -> (QuoteResponse)
    app.state.counter = itertools.count(1)

    @app.post(
        "/v1/quotes",
        status_code=201,
        response_model=QuoteResponse,
        tags=["quotes"],
        summary="Tạo báo giá",
    )
    async def create_quote(body: QuoteRequest, request: Request, response: Response) -> QuoteResponse:
        try:
            fare = calculate_fare(body.distance_km, service=body.service)
        except ValueError as exc:  # quy tắc nghiệp vụ bị vi phạm -> 400, không phải 422
            raise HTTPException(status_code=400, detail=str(exc)) from exc
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

    @app.get("/v1/quotes/{quote_id}", response_model=QuoteResponse, tags=["quotes"], summary="Đọc báo giá")
    async def get_quote(quote_id: str, request: Request) -> QuoteResponse:
        found = request.app.state.quotes.get(quote_id)
        if found is None:
            raise HTTPException(status_code=404, detail=f"không có báo giá {quote_id}")
        return found

    return app


app = create_app()
