"""Mã yêu cầu tối thiểu (như bài 33): gán `request.state.request_id` và header X-Request-Id."""

import re
import uuid
from collections.abc import Awaitable, Callable

from fastapi import FastAPI, Request, Response

_REQUEST_ID = re.compile(r"[A-Za-z0-9._-]{8,64}")


def pick_request_id(incoming: str | None) -> str:
    # Chỉ nhận mã của client nếu có dạng an toàn; ngược lại tự sinh (lý do ở bài 33).
    return incoming if incoming is not None and _REQUEST_ID.fullmatch(incoming) else uuid.uuid4().hex


def install(app: FastAPI) -> None:
    @app.middleware("http")
    async def request_id(request: Request, call_next: Callable[[Request], Awaitable[Response]]) -> Response:
        rid = pick_request_id(request.headers.get("x-request-id"))
        request.state.request_id = rid
        response = await call_next(request)
        response.headers["X-Request-Id"] = rid
        return response
