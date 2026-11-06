"""Route của API báo giá. Handler mỏng: nhận dependency đã dựng sẵn, gọi miền, trả model."""

import asyncio
import logging
from collections.abc import AsyncIterator, Callable
from datetime import timedelta
from typing import Annotated

from fastapi import APIRouter, Depends, HTTPException, Request, Response

from app.deps import (
    CustomerDep,
    DbDep,
    NowDep,
    SessionDep,
    SettingsDep,
    get_session,
    get_session_with_commit,
)
from app.domain import Money, Quote, QuoteNotFound, calculate_fare
from app.logs import request_id_var
from app.repo import TripSession
from app.schemas import ProblemDetails, QuoteRequest, QuoteResponse, TripResponse

log = logging.getLogger("toinoi.api")

COMMON_ERRORS = {
    422: {"model": ProblemDetails, "description": "Yêu cầu sai hình dạng (thiếu trường, sai kiểu, trường lạ)."},
    500: {"model": ProblemDetails, "description": "Lỗi nội bộ; chi tiết chỉ có trong log, tra bằng request_id."},
}
router = APIRouter(prefix="/v1", tags=["quotes"], responses=COMMON_ERRORS)
diagnostics = APIRouter(prefix="/v1/diagnostics", tags=["diagnostics"])


@router.post(
    "/quotes",
    operation_id="createQuote",
    status_code=201,
    response_model=QuoteResponse,
    responses={400: {"model": ProblemDetails, "description": "Khoảng cách ngoài miền 0..100 km."}},
)
async def create_quote(
    body: QuoteRequest, request: Request, response: Response, settings: SettingsDep, now: NowDep
) -> QuoteResponse:
    fare = calculate_fare(body.distance_km, service=body.service)  # DistanceOutOfRange -> handler -> 400
    quote = Quote(body.distance_km, Money(fare), now + timedelta(seconds=settings.quote_ttl_seconds))
    quote_id = f"QT-{next(request.app.state.counter):04d}"
    out = QuoteResponse(
        quote_id=quote_id,
        distance_km=quote.trip_distance_km,
        service=body.service,
        fare=quote.fare.amount,
        currency=quote.fare.currency,
        expires_at=quote.expires_at,
    )
    request.app.state.quotes[quote_id] = out
    response.headers["Location"] = f"/v1/quotes/{quote_id}"
    return out


@router.get(
    "/quotes/{quote_id}",
    operation_id="getQuote",
    response_model=QuoteResponse,
    responses={404: {"model": ProblemDetails, "description": "Không có báo giá với id này."}},
)
async def get_quote(quote_id: str, request: Request) -> QuoteResponse:
    found = request.app.state.quotes.get(quote_id)
    if found is None:
        raise QuoteNotFound(quote_id)
    return found


@router.get(
    "/trips/{trip_id}",
    operation_id="getTrip",
    response_model=TripResponse,
    responses={
        401: {"model": ProblemDetails, "description": "Thiếu hoặc sai danh tính khách."},
        404: {"model": ProblemDetails, "description": "Không có chuyến, hoặc chuyến của khách khác."},
    },
)
async def get_trip(trip_id: str, customer: CustomerDep, session: SessionDep) -> TripResponse:
    trip = session.get_trip(trip_id)
    # Chuyến của khách khác trả 404 giống chuyến không tồn tại: không lộ chuyến nào có thật.
    if trip is None or trip.customer_id != customer:
        raise HTTPException(status_code=404, detail="không tìm thấy chuyến")
    log.info("trip.read", extra={"trip_id": trip.trip_id})
    return TripResponse(**{k: getattr(trip, k) for k in TripResponse.model_fields})


# ---------------------------------------------------------------- route quan sát (chỉ môi trường dev)


def _report(db_opened_total: int, *sessions: TripSession) -> dict[str, object]:
    numbers = [s.session_no for s in sessions]
    return {"session_nos": numbers, "distinct": len(set(numbers)), "opened_total": db_opened_total}


@diagnostics.get("/dependency-cache")
async def dependency_cache(
    a: SessionDep,
    b: SessionDep,
    c: Annotated[TripSession, Depends(get_session, scope="function", use_cache=False)],
    db: DbDep,
) -> dict[str, object]:
    """a, b cùng cache; c có use_cache=False nên là phiên thứ hai."""
    return _report(db.opened_total, a, b, c)


@diagnostics.get("/dependency-cache-control")
async def dependency_cache_control(a: SessionDep, b: SessionDep, c: SessionDep, db: DbDep) -> dict[str, object]:
    """Đối chứng: không use_cache=False thì ba tham số là một phiên."""
    return _report(db.opened_total, a, b, c)


@diagnostics.get("/dependency-scope-mismatch")
async def dependency_scope_mismatch(
    a: SessionDep, b: Annotated[TripSession, Depends(get_session)], db: DbDep
) -> dict[str, object]:
    """Cùng hàm nhưng khác scope (function và request mặc định): cache KHÔNG dùng chung, mở hai phiên."""
    return _report(db.opened_total, a, b)


def probe(name: str) -> Callable[[], AsyncIterator[None]]:
    """Dependency có yield chỉ để ghi log lúc vào và lúc ra."""

    async def dependency() -> AsyncIterator[None]:
        log.info("dep.enter", extra={"dep": name})
        try:
            yield
        finally:
            log.info("dep.exit", extra={"dep": name})

    return dependency


@diagnostics.get("/scope-order")
async def scope_order(
    _a: Annotated[None, Depends(probe("request-scope"))],
    _b: Annotated[None, Depends(probe("function-scope"), scope="function")],
) -> dict[str, bool]:
    log.info("handler.body")
    return {"ok": True}


FunctionCommit = Annotated[TripSession, Depends(get_session_with_commit, scope="function")]
RequestCommit = Annotated[TripSession, Depends(get_session_with_commit)]


@diagnostics.post("/commit-function", status_code=201)
async def commit_function(session: FunctionCommit) -> dict[str, bool]:
    """Commit nằm trong phần dọn của dependency scope function: commit lỗi thì client thấy 500."""
    return {"ok": True}


@diagnostics.post("/commit-request", status_code=201)
async def commit_request(session: RequestCommit) -> dict[str, bool]:
    """Cùng commit lỗi nhưng scope request: response 201 đã đi trước khi commit chạy."""
    return {"ok": True}


def _sync_dependency() -> str:
    # `def` thường: FastAPI chạy trong threadpool, nhưng contextvars vẫn được sao chép sang.
    return request_id_var.get()


async def _read_in_task() -> str:
    return request_id_var.get()


@diagnostics.get("/context")
async def context(in_sync_dependency: Annotated[str, Depends(_sync_dependency)]) -> dict[str, str]:
    return {
        "in_handler": request_id_var.get(),
        "in_sync_dependency": in_sync_dependency,
        "in_thread": await asyncio.to_thread(request_id_var.get),
        "in_task": await asyncio.create_task(_read_in_task()),
    }


@diagnostics.get("/boom")
async def boom(session: SessionDep) -> None:
    raise RuntimeError(f"lỗi giả lập khi phiên {session.session_no} đang mở")


@diagnostics.get("/slow")
async def slow() -> dict[str, bool]:
    await asyncio.sleep(0.05)  # vượt ngưỡng slow_request_ms đặt thấp trong lab để thấy mức WARNING
    return {"ok": True}
