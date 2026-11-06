"""Problem Details (RFC 9457, application/problem+json): cùng dạng với bài 32.

Rút gọn so với lab 32 ở chỗ chỉ có các lỗi của service upload; dạng của 404/405 (about:blank),
422 (`errors[{field, code, message}]`, không chép `input`) và 500 giữ nguyên.
Bài 33 thêm `request_id`; lab này không có middleware đó nên bỏ qua.
"""

import logging
from http import HTTPStatus
from typing import Any

from fastapi import FastAPI, Request
from fastapi.encoders import jsonable_encoder
from fastapi.exceptions import RequestValidationError
from fastapi.responses import JSONResponse
from pydantic import BaseModel, ConfigDict, Field
from starlette.exceptions import HTTPException as StarletteHTTPException

from .errors import CorruptImage, ImageTooBig, JobNotFound, NotAnImage, UploadTooLarge

log = logging.getLogger("toinoi.problems")
PROBLEM_JSON = "application/problem+json"
TYPE_BASE = "https://toinoi.example/problems/"  # .example: tên miền dành riêng, không phân giải thật

MESSAGES_VI = {
    "missing": "thiếu trường bắt buộc",
    "string_pattern_mismatch": "không đúng mẫu quy định",
    "enum": "giá trị không thuộc danh sách cho phép",
    "greater_than_equal": "phải là số không âm",
    "int_parsing": "không phải số nguyên",
}


class FieldProblem(BaseModel):
    field: str
    code: str
    message: str


class ProblemDetails(BaseModel):
    model_config = ConfigDict(extra="allow")

    type: str = "about:blank"
    title: str
    status: int
    detail: str | None = None
    instance: str | None = None
    errors: list[FieldProblem] | None = Field(default=None, description="Chỉ có ở lỗi validation.")


def problem_response(request: Request, *, status: int, type_: str, title: str, detail: str | None = None,
                     headers: dict[str, str] | None = None, **extensions: Any) -> JSONResponse:
    problem = ProblemDetails(type=type_, title=title, status=status, detail=detail,
                             instance=request.url.path, **extensions)
    return JSONResponse(jsonable_encoder(problem.model_dump(exclude_none=True)), status_code=status,
                        media_type=PROBLEM_JSON, headers=headers)


def install(app: FastAPI) -> None:
    @app.exception_handler(UploadTooLarge)
    async def on_too_large(request: Request, exc: UploadTooLarge) -> JSONResponse:
        detail = (f"Body khai báo {exc.declared} byte, vượt giới hạn {exc.limit} byte mỗi ảnh."
                  if exc.declared else f"Ảnh vượt giới hạn {exc.limit} byte.")
        return problem_response(request, status=413, type_=TYPE_BASE + "upload-too-large",
                                title="Ảnh quá lớn", detail=detail, max_bytes=exc.limit)

    @app.exception_handler(NotAnImage)
    async def on_not_image(request: Request, exc: NotAnImage) -> JSONResponse:
        return problem_response(request, status=415, type_=TYPE_BASE + "unsupported-image",
                                title="Không phải ảnh JPEG hoặc PNG", detail=str(exc))

    @app.exception_handler(CorruptImage)
    async def on_corrupt(request: Request, exc: CorruptImage) -> JSONResponse:
        return problem_response(request, status=400, type_=TYPE_BASE + "image-corrupt",
                                title="Ảnh bị hỏng", detail="Không đọc trọn vẹn được file ảnh; hãy chụp hoặc gửi lại.")

    @app.exception_handler(ImageTooBig)
    async def on_too_big(request: Request, exc: ImageTooBig) -> JSONResponse:
        return problem_response(request, status=400, type_=TYPE_BASE + "image-too-many-pixels",
                                title="Ảnh có quá nhiều điểm ảnh", detail=str(exc))

    @app.exception_handler(JobNotFound)
    async def on_job_missing(request: Request, exc: JobNotFound) -> JSONResponse:
        return problem_response(request, status=404, type_=TYPE_BASE + "job-not-found",
                                title="Không tìm thấy job",
                                detail=f"{exc.job_id} không có trong tiến trình đang phục vụ.")

    @app.exception_handler(RequestValidationError)
    async def on_validation(request: Request, exc: RequestValidationError) -> JSONResponse:
        errors = [FieldProblem(field=".".join(str(p) for p in e["loc"]), code=e["type"],
                               message=MESSAGES_VI.get(e["type"], e["msg"])) for e in exc.errors()]
        return problem_response(request, status=422, type_=TYPE_BASE + "validation-error",
                                title="Yêu cầu không hợp lệ", detail=f"{len(errors)} lỗi trong yêu cầu",
                                errors=[e.model_dump() for e in errors])

    @app.exception_handler(StarletteHTTPException)
    async def on_http(request: Request, exc: StarletteHTTPException) -> JSONResponse:
        phrase = HTTPStatus(exc.status_code).phrase
        detail = exc.detail if isinstance(exc.detail, str) and exc.detail != phrase else None
        return problem_response(request, status=exc.status_code, type_="about:blank", title=phrase,
                                detail=detail, headers=dict(exc.headers) if exc.headers else None)

    @app.exception_handler(Exception)
    async def on_unhandled(request: Request, exc: Exception) -> JSONResponse:
        log.error("lỗi không lường trước ở %s %s", request.method, request.url.path, exc_info=exc)
        return problem_response(request, status=500, type_=TYPE_BASE + "internal-error", title="Lỗi nội bộ",
                                detail="Có lỗi ở phía máy chủ; hãy thử lại sau.")


def problem_media_type_in_openapi(app: FastAPI) -> None:
    """FastAPI gắn schema của `responses={...: {"model": ...}}` vào application/json; lỗi của ta là
    application/problem+json, nên đổi tên khóa trong OpenAPI sau khi sinh (như lab 32)."""
    generate = app.openapi

    def openapi() -> dict[str, Any]:
        if app.openapi_schema:
            return app.openapi_schema
        spec = generate()
        ref = {"$ref": "#/components/schemas/ProblemDetails"}
        for path_item in spec["paths"].values():
            for operation in path_item.values():
                for response in operation["responses"].values():
                    content = response.get("content", {})
                    if content.get("application/json", {}).get("schema") == ref:
                        content[PROBLEM_JSON] = content.pop("application/json")
        return spec

    app.openapi = openapi  # type: ignore[method-assign]
