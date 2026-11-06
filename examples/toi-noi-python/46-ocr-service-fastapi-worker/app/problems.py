"""Lỗi theo RFC 9457 (application/problem+json), cùng dạng với lab 32 (`app/problems.py`):
type + title + status + detail + instance, lỗi 422 có `errors[{field, code, message}]`."""

import logging
from http import HTTPStatus
from typing import Any

from fastapi import FastAPI, Request
from fastapi.encoders import jsonable_encoder
from fastapi.exceptions import RequestValidationError
from fastapi.responses import JSONResponse
from starlette.exceptions import HTTPException as StarletteHTTPException

log = logging.getLogger("toinoi.problems")

PROBLEM_JSON = "application/problem+json"
TYPE_BASE = "https://toinoi.example/problems/"  # tên miền .example: dành riêng, không bao giờ phân giải thật

MESSAGES_VI = {
    "missing": "thiếu trường bắt buộc",
    "string_pattern_mismatch": "không đúng định dạng",
    "enum": "giá trị không thuộc danh sách cho phép",
    "json_invalid": "body không phải JSON hợp lệ",
}


class ProblemError(Exception):
    """Lỗi nghiệp vụ có cấu trúc; handler biến nó thành problem+json."""

    def __init__(self, status: int, slug: str, title: str, detail: str, **extensions: Any) -> None:
        super().__init__(detail)
        self.status, self.slug, self.title, self.detail = status, slug, title, detail
        self.extensions = extensions


def problem_response(
    request: Request, *, status: int, type_: str, title: str, detail: str | None = None,
    headers: dict[str, str] | None = None, **extensions: Any,
) -> JSONResponse:
    body = {"type": type_, "title": title, "status": status, "detail": detail,
            "instance": request.url.path, **extensions}
    return JSONResponse(jsonable_encoder({k: v for k, v in body.items() if v is not None}),
                        status_code=status, media_type=PROBLEM_JSON, headers=headers)


def field_problems(exc: RequestValidationError) -> list[dict[str, str]]:
    """Danh sách phẳng {field, code, message}. KHÔNG chép `input`: có thể là dữ liệu cá nhân."""
    out = []
    for err in exc.errors():
        code = err["type"]
        field = "body" if code == "json_invalid" else ".".join(str(part) for part in err["loc"])
        out.append({"field": field, "code": code, "message": MESSAGES_VI.get(code, err["msg"])})
    return out


def install(app: FastAPI) -> None:
    @app.exception_handler(ProblemError)
    async def on_problem(request: Request, exc: ProblemError) -> JSONResponse:
        return problem_response(request, status=exc.status, type_=TYPE_BASE + exc.slug,
                                title=exc.title, detail=exc.detail, **exc.extensions)

    @app.exception_handler(RequestValidationError)
    async def on_validation(request: Request, exc: RequestValidationError) -> JSONResponse:
        errors = field_problems(exc)
        return problem_response(request, status=422, type_=TYPE_BASE + "validation-error",
                                title="Yêu cầu không hợp lệ", detail=f"{len(errors)} lỗi trong yêu cầu",
                                errors=errors)

    @app.exception_handler(StarletteHTTPException)
    async def on_http(request: Request, exc: StarletteHTTPException) -> JSONResponse:
        phrase = HTTPStatus(exc.status_code).phrase
        detail = exc.detail if isinstance(exc.detail, str) and exc.detail != phrase else None
        return problem_response(request, status=exc.status_code, type_="about:blank", title=phrase,
                                detail=detail, headers=dict(exc.headers) if exc.headers else None)

    @app.exception_handler(Exception)
    async def on_unhandled(request: Request, exc: Exception) -> JSONResponse:
        log.error("lỗi không lường trước ở %s %s", request.method, request.url.path, exc_info=exc)
        return problem_response(request, status=500, type_=TYPE_BASE + "internal-error",
                                title="Lỗi nội bộ", detail="Có lỗi ở phía máy chủ; hãy thử lại sau.")
