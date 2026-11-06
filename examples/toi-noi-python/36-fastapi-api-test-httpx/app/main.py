"""API báo giá và chuyến của Tới Nơi, đủ phức tạp để có thứ đáng kiểm chứng ở bài 36.

Chạy (cần TOINOI_ENVIRONMENT và TOINOI_JWT_SECRET, xem README):
  uv run --project <lab> uvicorn --factory app.main:create_app --app-dir <lab> --port 8036
"""

import itertools
from collections.abc import AsyncIterator, Callable
from contextlib import asynccontextmanager
from datetime import UTC, datetime
from typing import Annotated

from fastapi import APIRouter, Depends, FastAPI, Request, Response

from app import problems, request_id
from app.domain import QUOTE_TTL, QuoteNotFound, Trip, TripNotFound, calculate_fare
from app.repo import InMemoryTripRepository, TripRepository, seed_trips
from app.schemas import ProblemDetails, QuoteRequest, QuoteResponse, TripOut
from app.security import Authenticated, Principal, TokenService, require_scope
from app.settings import Settings

Clock = Callable[[], datetime]


def err(description: str) -> dict:
    return {"model": ProblemDetails, "description": description}


# ---- Dependency: mỗi cái là một «khớp nối» để test thay thế --------------------------------


def get_clock() -> Clock:
    return lambda: datetime.now(UTC)


def get_repo(request: Request) -> TripRepository:
    return request.app.state.repo  # do lifespan dựng


@asynccontextmanager
async def lifespan(app: FastAPI) -> AsyncIterator[None]:
    app.state.repo = InMemoryTripRepository(seed_trips())
    yield


router = APIRouter(prefix="/v1")
UNAUTHORIZED = err("Thiếu hoặc sai token (kèm WWW-Authenticate: Bearer).")
FORBIDDEN = err("Token đúng nhưng thiếu scope.")
INVALID_BODY = err("Yêu cầu sai hình dạng (thiếu trường, sai kiểu, trường lạ).")


@router.post("/quotes", operation_id="createQuote", status_code=201, response_model=QuoteResponse,
             responses={400: err("Khoảng cách ngoài miền 0..100 km."), 401: UNAUTHORIZED, 403: FORBIDDEN,
                        422: INVALID_BODY})
async def create_quote(
    body: QuoteRequest, request: Request, response: Response,
    principal: Annotated[Principal, require_scope("customer")],
    clock: Annotated[Clock, Depends(get_clock)],
) -> QuoteResponse:
    fare = calculate_fare(body.distance_km, service=body.service)  # DistanceOutOfRange -> handler -> 400
    quote_id = f"QT-{next(request.app.state.counter):04d}"
    out = QuoteResponse(quote_id=quote_id, distance_km=body.distance_km, service=body.service, fare=fare,
                        expires_at=clock() + QUOTE_TTL)
    request.app.state.quotes[quote_id] = (principal.user_id, out)  # lưu cả chủ báo giá
    response.headers["Location"] = f"/v1/quotes/{quote_id}"
    return out


@router.get("/quotes/{quote_id}", operation_id="getQuote", response_model=QuoteResponse,
            responses={401: UNAUTHORIZED, 403: FORBIDDEN, 404: err("Không có báo giá này, hoặc của khách khác.")})
async def get_quote(quote_id: str, request: Request,
                    principal: Annotated[Principal, require_scope("customer")]) -> QuoteResponse:
    owner, found = request.app.state.quotes.get(quote_id, (None, None))
    if found is None or owner != principal.user_id:
        raise QuoteNotFound(quote_id)  # cùng một câu trả lời cho «không có» và «của người khác»
    return found


def visible_to(principal: Principal, trip: Trip) -> bool:
    return ("dispatcher" in principal.scopes
            or ("customer" in principal.scopes and trip.customer_id == principal.user_id)
            or ("driver" in principal.scopes and trip.driver_id == principal.user_id))


@router.get("/trips/{trip_id}", operation_id="getTrip", response_model=TripOut,
            responses={401: UNAUTHORIZED, 404: err("Không có chuyến, hoặc không phải chuyến của bạn."),
                       503: err("Kho dữ liệu tạm thời không dùng được.")})
async def get_trip(trip_id: str, principal: Authenticated,
                   repo: Annotated[TripRepository, Depends(get_repo)]) -> Trip:
    """`Authenticated` chỉ đòi token hợp lệ, không đòi scope (không có 403 ở route này): quyền được quyết
    định theo quan hệ giữa người gọi và chuyến, và mọi từ chối đều là 404."""
    trip = await repo.get(trip_id)
    if trip is None or not visible_to(principal, trip):
        raise TripNotFound(trip_id)
    return trip


@router.post("/trips/{trip_id}/cancel", operation_id="cancelTrip", response_model=TripOut,
             responses={401: UNAUTHORIZED, 403: FORBIDDEN, 404: err("Không có chuyến này."),
                        409: err("Trạng thái hiện tại không cho phép hủy."),
                        503: err("Kho dữ liệu tạm thời không dùng được.")})
async def cancel_trip(trip_id: str, _: Annotated[Principal, require_scope("dispatcher")],
                      repo: Annotated[TripRepository, Depends(get_repo)]) -> Trip:
    trip = await repo.get(trip_id)
    if trip is None:
        raise TripNotFound(trip_id)
    trip.cancel()  # InvalidTransition -> handler -> 409
    await repo.save(trip)
    return trip


def create_app(settings: Settings | None = None) -> FastAPI:
    # Settings() ném ValidationError nếu thiếu biến hoặc khóa yếu: service không lên.
    settings = settings or Settings()  # type: ignore[call-arg]
    app = FastAPI(title="Tới Nơi · Quote & Trip API", version="1.0.0", lifespan=lifespan)
    app.state.settings = settings
    app.state.tokens = TokenService(settings)
    app.state.quotes = {}
    app.state.counter = itertools.count(1)
    problems.install(app)
    request_id.install(app)
    app.include_router(router)
    problems.problem_media_type_in_openapi(app)
    return app
