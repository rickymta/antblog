"""Problem Details (RFC 9457) cùng dạng với lab 32 và 34: 404/405 `about:blank`, 422 `errors[{field, code, message}]`
không chép `input`, 500 `internal-error` không lộ chi tiết, `request_id` ở mọi lỗi (bài 33).
Thêm các lỗi riêng của lab: chuyến không có, chuyển trạng thái sai, kho không dùng được."""

from http import HTTPStatus
from typing import Any

from fastapi import FastAPI, Request
from fastapi.encoders import jsonable_encoder
from fastapi.exceptions import RequestValidationError
from fastapi.responses import JSONResponse
from starlette.exceptions import HTTPException as StarletteHTTPException

from app.domain import MAX_KM, DistanceOutOfRange, InvalidTransition, QuoteNotFound, TripNotFound
from app.repo import RepositoryUnavailable
from app.schemas import FieldProblem, ProblemDetails

PROBLEM_JSON = "application/problem+json"
TYPE_BASE = "https://toinoi.example/problems/"  # .example: tên miền dành riêng, không bao giờ phân giải thật

MESSAGES_VI = {
    "missing": "thiếu trường bắt buộc",
    "decimal_parsing": "không phải số thập phân hợp lệ",
    "decimal_max_places": "tối đa 3 chữ số thập phân",
    "finite_number": "phải là số hữu hạn",
    "enum": "giá trị không thuộc danh sách cho phép",
    "extra_forbidden": "trường không có trong hợp đồng",
    "json_invalid": "body không phải JSON hợp lệ",
}


def problem_response(request: Request, *, status: int, type_: str, title: str, detail: str | None = None,
                     headers: dict[str, str] | None = None, **extensions: Any) -> JSONResponse:
    request_id = getattr(request.state, "request_id", None)
    if request_id is not None:
        extensions["request_id"] = request_id
    problem = ProblemDetails(type=type_, title=title, status=status, detail=detail,
                             instance=request.url.path, **extensions)
    return JSONResponse(jsonable_encoder(problem.model_dump(exclude_none=True)), status_code=status,
                        media_type=PROBLEM_JSON, headers=headers)


def field_problems(exc: RequestValidationError) -> list[FieldProblem]:
    """Đổi lỗi pydantic thành danh sách phẳng. KHÔNG chép `input` (có thể là dữ liệu nhạy cảm)."""
    out = []
    for err in exc.errors():
        code = err["type"]
        field = "body" if code == "json_invalid" else ".".join(str(part) for part in err["loc"])
        out.append(FieldProblem(field=field, code=code, message=MESSAGES_VI.get(code, err["msg"])))
    return out


def install(app: FastAPI) -> None:
    @app.exception_handler(RequestValidationError)
    async def on_validation(request: Request, exc: RequestValidationError) -> JSONResponse:
        errors = field_problems(exc)
        return problem_response(request, status=422, type_=TYPE_BASE + "validation-error",
                                title="Yêu cầu không hợp lệ", detail=f"{len(errors)} lỗi trong yêu cầu",
                                errors=[e.model_dump() for e in errors])

    @app.exception_handler(DistanceOutOfRange)
    async def on_distance(request: Request, exc: DistanceOutOfRange) -> JSONResponse:
        return problem_response(request, status=400, type_=TYPE_BASE + "distance-out-of-range",
                                title="Khoảng cách ngoài miền cho phép",
                                detail=f"khoảng cách {exc.distance_km} km nằm ngoài miền 0..{MAX_KM} km",
                                min_km="0", max_km=str(MAX_KM))

    @app.exception_handler(QuoteNotFound)
    async def on_quote_missing(request: Request, exc: QuoteNotFound) -> JSONResponse:
        return problem_response(request, status=404, type_=TYPE_BASE + "quote-not-found",
                                title="Không tìm thấy báo giá", detail=str(exc))

    @app.exception_handler(TripNotFound)
    async def on_trip_missing(request: Request, exc: TripNotFound) -> JSONResponse:
        return problem_response(request, status=404, type_=TYPE_BASE + "trip-not-found",
                                title="Không tìm thấy chuyến", detail=str(exc))

    @app.exception_handler(InvalidTransition)
    async def on_transition(request: Request, exc: InvalidTransition) -> JSONResponse:
        return problem_response(request, status=409, type_=TYPE_BASE + "invalid-transition",
                                title="Trạng thái hiện tại không cho phép thao tác", detail=str(exc))

    @app.exception_handler(RepositoryUnavailable)
    async def on_repo_down(request: Request, exc: RepositoryUnavailable) -> JSONResponse:
        return problem_response(request, status=503, type_=TYPE_BASE + "repository-unavailable",
                                title="Dịch vụ tạm thời không sẵn sàng", detail="Thử lại sau ít giây.",
                                headers={"Retry-After": "5"})

    @app.exception_handler(StarletteHTTPException)
    async def on_http(request: Request, exc: StarletteHTTPException) -> JSONResponse:
        # 401, 403, 404 route không có, 405 sai phương thức...: type "about:blank", title là cụm từ chuẩn của mã HTTP
        phrase = HTTPStatus(exc.status_code).phrase
        detail = exc.detail if isinstance(exc.detail, str) and exc.detail != phrase else None
        return problem_response(request, status=exc.status_code, type_="about:blank", title=phrase,
                                detail=detail, headers=dict(exc.headers) if exc.headers else None)

    @app.exception_handler(Exception)
    async def on_unhandled(request: Request, exc: Exception) -> JSONResponse:
        # Chi tiết chỉ vào log (Starlette ghi traceback); client chỉ thấy câu chung.
        request_id = getattr(request.state, "request_id", "-")
        return problem_response(request, status=500, type_=TYPE_BASE + "internal-error", title="Lỗi nội bộ",
                                detail="Có lỗi ở phía máy chủ; hãy thử lại sau, kèm mã yêu cầu khi liên hệ hỗ trợ.",
                                headers={"X-Request-Id": request_id})


def problem_media_type_in_openapi(app: FastAPI) -> None:
    """FastAPI gắn schema của `responses={...: {"model": ...}}` vào application/json; lỗi của ta là
    application/problem+json, nên đổi tên khóa trong OpenAPI sau khi sinh (như lab 32)."""
    generate = app.openapi

    def openapi() -> dict[str, Any]:
        if app.openapi_schema:
            return app.openapi_schema
        spec = generate()
        ref = {"$ref": "#/components/schemas/ProblemDetails"}
        default_422 = {"$ref": "#/components/schemas/HTTPValidationError"}
        for path_item in spec["paths"].values():
            for operation in path_item.values():
                responses = operation["responses"]
                for code in [c for c, r in responses.items()
                             if r.get("content", {}).get("application/json", {}).get("schema") == default_422]:
                    del responses[code]  # 422 tự thêm của FastAPI cho route không có gì để validate (chỉ path str)
                for response in responses.values():
                    content = response.get("content", {})
                    if content.get("application/json", {}).get("schema") == ref:
                        content[PROBLEM_JSON] = content.pop("application/json")
        for unused in ("HTTPValidationError", "ValidationError"):
            spec["components"]["schemas"].pop(unused, None)
        return spec

    app.openapi = openapi  # type: ignore[method-assign]
