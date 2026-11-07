"""FastAPI: dịch HTTP sang use case và lỗi sang Problem Details (một dạng như lab 32)."""

from dataclasses import asdict
from typing import Any

from fastapi import FastAPI, Request, Response
from fastapi.exceptions import RequestValidationError
from fastapi.responses import JSONResponse

from kyc_service.api.container import Container
from kyc_service.api.schemas import ApplicationResponse, ReviewBody, SubmitBody
from kyc_service.application.commands import (
    RecordReviewDecision,
    ResumeScreening,
    SubmitDriverApplication,
)
from kyc_service.application.errors import (
    ApplicationNotFound,
    ConcurrencyConflict,
    ValidationFailed,
)
from kyc_service.application.mediator import HandlerNotFound
from kyc_service.application.queries import GetApplication, ListReviewQueue
from kyc_service.application.views import ApplicationView
from kyc_service.domain.errors import DomainError, InvalidStatusTransition

TYPE_BASE = "https://toinoi.example/problems/"


def problem(
    request: Request, status: int, slug: str, title: str, detail: str, **extra: Any
) -> JSONResponse:
    body = {
        "type": TYPE_BASE + slug,
        "title": title,
        "status": status,
        "detail": detail,
        "instance": request.url.path,
        **extra,
    }
    return JSONResponse(body, status_code=status, media_type="application/problem+json")


def _response(view: ApplicationView) -> ApplicationResponse:
    return ApplicationResponse.model_validate(asdict(view))


def create_app(container: Container) -> FastAPI:
    mediator = container.mediator
    app = FastAPI(title="Tới Nơi · KYC tài xế (bài 71)")

    def validation_problem(request: Request, errors: list[dict[str, str]]) -> JSONResponse:
        return problem(
            request,
            422,
            "validation-error",
            "Yêu cầu không hợp lệ",
            f"{len(errors)} lỗi trong yêu cầu",
            errors=errors,
        )

    @app.exception_handler(RequestValidationError)
    async def on_request_validation(request: Request, exc: RequestValidationError) -> JSONResponse:
        # Không chép `input` vào phản hồi: có thể là dữ liệu nhạy cảm.
        return validation_problem(
            request,
            [
                {
                    "field": ".".join(str(p) for p in e["loc"]),
                    "code": e["type"],
                    "message": e["msg"],
                }
                for e in exc.errors()
            ],
        )

    @app.exception_handler(ValidationFailed)
    async def on_message_validation(request: Request, exc: ValidationFailed) -> JSONResponse:
        # Lỗi do behavior validation: cùng dạng với lỗi của FastAPI, field gắn tiền tố `body.`.
        return validation_problem(
            request,
            [
                {"field": f"body.{e.field}", "code": e.code, "message": e.message}
                for e in exc.errors
            ],
        )

    @app.exception_handler(ApplicationNotFound)
    async def on_not_found(request: Request, exc: ApplicationNotFound) -> JSONResponse:
        return problem(request, 404, "application-not-found", "Không có hồ sơ", str(exc))

    @app.exception_handler(InvalidStatusTransition)
    async def on_transition(request: Request, exc: InvalidStatusTransition) -> JSONResponse:
        return problem(
            request, 409, "invalid-status-transition", "Trạng thái hồ sơ không cho phép", str(exc)
        )

    @app.exception_handler(ConcurrencyConflict)
    async def on_conflict(request: Request, exc: ConcurrencyConflict) -> JSONResponse:
        return problem(
            request, 409, "concurrent-update", "Hồ sơ vừa bị người khác cập nhật", str(exc)
        )

    @app.exception_handler(DomainError)
    async def on_domain(request: Request, exc: DomainError) -> JSONResponse:
        return problem(request, 422, "domain-rule", "Vi phạm quy tắc nghiệp vụ", str(exc))

    @app.exception_handler(HandlerNotFound)
    async def on_no_handler(request: Request, exc: HandlerNotFound) -> JSONResponse:
        return problem(request, 500, "wiring-error", "Lỗi cấu hình service", "thiếu handler")

    @app.post("/applications", status_code=201, response_model=ApplicationResponse)
    def submit(body: SubmitBody, response: Response) -> ApplicationResponse:
        view = mediator.send(SubmitDriverApplication(**body.model_dump()))
        response.headers["Location"] = f"/applications/{view.application_id}"
        if view.pending_check:
            response.status_code = 202  # nhận hồ sơ nhưng một service chưa kiểm được
        return _response(view)

    @app.get("/applications/{application_id}", response_model=ApplicationResponse)
    def get(application_id: str) -> ApplicationResponse:
        return _response(mediator.send(GetApplication(application_id)))

    @app.get("/review-queue", response_model=list[ApplicationResponse])
    def review_queue() -> list[ApplicationResponse]:
        return [_response(v) for v in mediator.send(ListReviewQueue())]

    @app.post("/applications/{application_id}/resume", response_model=ApplicationResponse)
    def resume(application_id: str, response: Response) -> ApplicationResponse:
        view = mediator.send(ResumeScreening(application_id))
        if view.pending_check:
            response.status_code = 202  # vẫn còn một service chưa kiểm được
        return _response(view)

    @app.post("/applications/{application_id}/review", response_model=ApplicationResponse)
    def review(application_id: str, body: ReviewBody) -> ApplicationResponse:
        command = RecordReviewDecision(application_id=application_id, **body.model_dump())
        return _response(mediator.send(command))

    return app
