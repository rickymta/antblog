"""Middleware biên request: request id, đo thời gian, một dòng log mỗi request."""

import logging
import re
import uuid
from collections.abc import Awaitable, Callable
from time import perf_counter

from fastapi import FastAPI, Request, Response

from app.logs import request_id_var

access_log = logging.getLogger("toinoi.access")

# Chỉ nhận request id của client nếu có dạng an toàn; ngược lại tự sinh. Không phản chiếu chuỗi
# tùy ý của client vào log và header (chống chèn dòng log, header quá dài).
_REQUEST_ID = re.compile(r"[A-Za-z0-9._-]{8,64}")


def pick_request_id(incoming: str | None) -> str:
    if incoming is not None and _REQUEST_ID.fullmatch(incoming):
        return incoming
    return uuid.uuid4().hex


def add_request_context(app: FastAPI) -> None:
    slow_ms = app.state.settings.slow_request_ms

    @app.middleware("http")
    async def request_context(
        request: Request, call_next: Callable[[Request], Awaitable[Response]]
    ) -> Response:
        request_id = pick_request_id(request.headers.get("x-request-id"))
        request.state.request_id = request_id  # exception handler (ngoài middleware) đọc từ đây
        token = request_id_var.set(request_id)
        started = perf_counter()
        try:
            try:
                response = await call_next(request)
            except Exception:
                access_log.exception("request.failed", extra=_fields(request, 500, started))
                raise
            fields = _fields(request, response.status_code, started)
            level = logging.ERROR if response.status_code >= 500 else logging.INFO
            if level == logging.INFO and fields["duration_ms"] >= slow_ms:
                level = logging.WARNING
            access_log.log(level, "request.completed", extra=fields)
            response.headers["X-Request-Id"] = request_id
            response.headers["Server-Timing"] = f"app;dur={fields['duration_ms']}"
            return response
        finally:
            request_id_var.reset(token)


def _fields(request: Request, status: int, started: float) -> dict[str, object]:
    # route.path là mẫu "/v1/trips/{trip_id}", không phải đường dẫn thật: số giá trị khác nhau
    # có hạn nên dùng được làm nhãn cho dashboard; route "-" nghĩa là không khớp route nào (404).
    route = getattr(request.scope.get("route"), "path", "-")
    fields: dict[str, object] = {
        "method": request.method,
        "route": route,
        "status": status,
        "duration_ms": round((perf_counter() - started) * 1000, 2),
    }
    customer_id = getattr(request.state, "customer_id", None)
    if customer_id is not None:
        fields["customer_id"] = customer_id
    return fields
