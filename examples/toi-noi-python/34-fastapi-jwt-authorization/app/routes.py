"""Route của lab 34: cấp token, rồi mỗi route khai báo scope nó cần bằng `require_scope`."""

import asyncio
import logging
from datetime import UTC, datetime
from decimal import Decimal
from typing import Annotated

from fastapi import APIRouter, Depends, HTTPException, Request, Response
from fastapi.security import OAuth2PasswordRequestForm

from app.domain import QUOTE_TTL, Money, Quote, QuoteNotFound, ServiceKind, Trip, TripStatus, calculate_fare
from app.problems import OAuthError
from app.schemas import ProblemDetails, QuoteRequest, QuoteResponse, TokenResponse, TripResponse
from app.security import Authenticated, Principal, TokenService, require_scope
from app.users import UserRecord

log = logging.getLogger("toinoi.auth")

COMMON_ERRORS = {
    401: {"model": ProblemDetails, "description": "Thiếu token, hoặc token không hợp lệ hay đã hết hạn."},
    403: {"model": ProblemDetails, "description": "Token hợp lệ nhưng thiếu scope của route."},
    500: {"model": ProblemDetails, "description": "Lỗi nội bộ; chi tiết chỉ có trong log, tra bằng request_id."},
}
router = APIRouter(prefix="/v1", tags=["api"], responses=COMMON_ERRORS)
auth_router = APIRouter(prefix="/v1", tags=["auth"])


def seed_trips() -> dict[str, Trip]:
    trips = [
        Trip("TN-0001", "CU-001", "DR-001", ServiceKind.RIDE, Decimal("2.5"), TripStatus.IN_TRANSIT),
        Trip("TN-0002", "CU-002", "DR-002", ServiceKind.FOOD, Decimal("5"), TripStatus.COMPLETED),
        Trip("TN-0003", "CU-001", None, ServiceKind.PARCEL, Decimal("12"), TripStatus.SEARCHING),
    ]
    return {t.trip_id: t for t in trips}


# ---------------------------------------------------------------- cấp token (OAuth2 password flow)


@auth_router.post(
    "/auth/token",
    operation_id="issueToken",
    responses={400: {"model": ProblemDetails, "description": "invalid_grant hoặc invalid_scope (RFC 6749 mục 5.2)."}},
)
async def login(
    request: Request, response: Response, form: Annotated[OAuth2PasswordRequestForm, Depends()]
) -> TokenResponse:
    users: dict[str, UserRecord] = request.app.state.users
    hasher = request.app.state.hasher
    tokens: TokenService = request.app.state.tokens

    user = users.get(form.username)
    # Tên không tồn tại vẫn tốn một lần băm, để thời gian trả lời không tiết lộ tài khoản có thật.
    hash_to_check = user.password_hash if user else request.app.state.dummy_hash
    # Băm argon2 tốn CPU cỡ vài chục mili giây: đưa sang thread để không chặn event loop (bài 21).
    password_ok = await asyncio.to_thread(hasher.verify, form.password, hash_to_check)
    if user is None or not password_ok:
        # Không log tên và mật khẩu đã gõ. RFC 6749 mục 5.2: sai thông tin đăng nhập là invalid_grant, HTTP 400.
        log.warning("auth.login.failed known_user=%s request_id=%s", user is not None, request.state.request_id)
        raise OAuthError("invalid_grant", "sai tên đăng nhập hoặc mật khẩu")

    # Scope xin cấp phải là tập con của scope người dùng được phép; không xin thì cấp đủ.
    requested = frozenset(form.scopes) or user.scopes
    if not requested <= user.scopes:
        raise OAuthError("invalid_scope", "xin scope ngoài quyền của tài khoản")

    token = tokens.issue(user.user_id, requested)
    log.info("auth.login.succeeded user=%s scope=%s", user.user_id, " ".join(sorted(requested)))
    # RFC 6749 mục 5.1: response chứa token phải có Cache-Control: no-store và Pragma: no-cache.
    response.headers["Cache-Control"] = "no-store"
    response.headers["Pragma"] = "no-cache"
    return TokenResponse(access_token=token, expires_in=tokens.ttl_seconds, scope=" ".join(sorted(requested)))


# ---------------------------------------------------------------- route được bảo vệ


@router.get("/me", operation_id="getMe")
async def me(principal: Authenticated) -> dict[str, object]:
    return {"user_id": principal.user_id, "scopes": sorted(principal.scopes)}


@router.post(
    "/quotes",
    operation_id="createQuote",
    status_code=201,
    response_model=QuoteResponse,
    responses={
        400: {"model": ProblemDetails, "description": "Khoảng cách ngoài miền 0..100 km."},
        422: {"model": ProblemDetails, "description": "Yêu cầu sai hình dạng (thiếu trường, sai kiểu, trường lạ)."},
    },
)
async def create_quote(
    body: QuoteRequest,
    request: Request,
    response: Response,
    principal: Annotated[Principal, require_scope("customer")],
) -> QuoteResponse:
    fare = calculate_fare(body.distance_km, service=body.service)  # DistanceOutOfRange -> handler -> 400
    quote = Quote(body.distance_km, Money(fare), datetime.now(UTC) + QUOTE_TTL)
    quote_id = f"QT-{next(request.app.state.counter):04d}"
    out = QuoteResponse(
        quote_id=quote_id,
        distance_km=quote.trip_distance_km,
        service=body.service,
        fare=quote.fare.amount,
        currency=quote.fare.currency,
        expires_at=quote.expires_at,
    )
    request.app.state.quotes[quote_id] = (principal.user_id, out)  # lưu cả chủ báo giá
    response.headers["Location"] = f"/v1/quotes/{quote_id}"
    return out


@router.get(
    "/quotes/{quote_id}",
    operation_id="getQuote",
    response_model=QuoteResponse,
    responses={404: {"model": ProblemDetails, "description": "Không có báo giá này, hoặc báo giá của khách khác."}},
)
async def get_quote(
    quote_id: str, request: Request, principal: Annotated[Principal, require_scope("customer")]
) -> QuoteResponse:
    owner, found = request.app.state.quotes.get(quote_id, (None, None))
    if found is None or owner != principal.user_id:
        raise QuoteNotFound(quote_id)
    return found


@router.get(
    "/trips/{trip_id}",
    operation_id="getTrip",
    response_model=TripResponse,
    responses={404: {"model": ProblemDetails, "description": "Không có chuyến, hoặc không phải chuyến của bạn."}},
)
async def get_trip(trip_id: str, request: Request, principal: Authenticated) -> TripResponse:
    """`Authenticated` chỉ đòi token hợp lệ, KHÔNG đòi scope nào (không có cổng 3 ở route này):
    quyền được quyết định ngay dưới đây theo quan hệ giữa người gọi và chuyến (cổng 4)."""
    trip = request.app.state.trips.get(trip_id)
    allowed = trip is not None and (
        "dispatcher" in principal.scopes
        or ("customer" in principal.scopes and trip.customer_id == principal.user_id)
        or ("driver" in principal.scopes and trip.driver_id == principal.user_id)
    )
    if not allowed:
        if trip is not None:
            # Từ chối vì quyền mà vẫn trả 404: ghi log để kẻ dò chuyến của người khác để lại dấu vết.
            log.warning("auth.trip.denied user=%s trip=%s request_id=%s", principal.user_id, trip_id, request.state.request_id)
        raise HTTPException(status_code=404, detail="không tìm thấy chuyến")
    return TripResponse(
        trip_id=trip.trip_id,
        customer_id=trip.customer_id,
        driver_id=trip.driver_id,
        service=trip.service,
        distance_km=trip.distance_km,
        status=trip.status,
    )


@router.get("/driver/availability", operation_id="getAvailability")
async def availability(principal: Annotated[Principal, require_scope("driver")]) -> dict[str, object]:
    return {"driver_id": principal.user_id, "available": True}


@router.get("/dispatch/queue", operation_id="getQueue")
async def queue(
    request: Request, principal: Annotated[Principal, require_scope("dispatcher")]
) -> dict[str, list[str]]:
    waiting = {TripStatus.REQUESTED, TripStatus.SEARCHING}
    trips = request.app.state.trips.values()
    return {"waiting": sorted(t.trip_id for t in trips if t.status in waiting)}
