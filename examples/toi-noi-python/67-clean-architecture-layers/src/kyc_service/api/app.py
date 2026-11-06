"""FastAPI: dịch HTTP sang use case và lỗi sang Problem Details (một dạng như lab 32)."""

from dataclasses import asdict
from typing import Any

from fastapi import FastAPI, Request, Response
from fastapi.exceptions import RequestValidationError
from fastapi.responses import JSONResponse

from kyc_service.api.container import Container, build_container
from kyc_service.api.schemas import ApplicationResponse, ReviewBody, SubmitBody
from kyc_service.application.errors import ApplicationNotFound
from kyc_service.application.use_cases import SubmitDriverApplicationRequest
from kyc_service.application.views import ApplicationView
from kyc_service.domain.errors import DomainError, InvalidStatusTransition
from kyc_service.domain.kyc import KycStatus

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


def create_app(container: Container | None = None) -> FastAPI:
    container = container or build_container()
    app = FastAPI(title="Tới Nơi · KYC tài xế (bài 67)")

    @app.exception_handler(RequestValidationError)
    async def on_validation(request: Request, exc: RequestValidationError) -> JSONResponse:
        # Không chép `input` vào phản hồi: có thể là dữ liệu nhạy cảm.
        errors = [
            {"field": ".".join(str(p) for p in e["loc"]), "code": e["type"], "message": e["msg"]}
            for e in exc.errors()
        ]
        return problem(
            request,
            422,
            "validation-error",
            "Yêu cầu không hợp lệ",
            f"{len(errors)} lỗi trong yêu cầu",
            errors=errors,
        )

    @app.exception_handler(ApplicationNotFound)
    async def on_not_found(request: Request, exc: ApplicationNotFound) -> JSONResponse:
        return problem(request, 404, "application-not-found", "Không có hồ sơ", str(exc))

    @app.exception_handler(InvalidStatusTransition)
    async def on_transition(request: Request, exc: InvalidStatusTransition) -> JSONResponse:
        return problem(
            request, 409, "invalid-status-transition", "Trạng thái hồ sơ không cho phép", str(exc)
        )

    @app.exception_handler(DomainError)
    async def on_domain(request: Request, exc: DomainError) -> JSONResponse:
        return problem(request, 422, "domain-rule", "Vi phạm quy tắc nghiệp vụ", str(exc))

    @app.post("/applications", status_code=201, response_model=ApplicationResponse)
    def submit(body: SubmitBody, response: Response) -> ApplicationResponse:
        view = container.submit.execute(SubmitDriverApplicationRequest(**body.model_dump()))
        response.headers["Location"] = f"/applications/{view.application_id}"
        if view.pending_check:
            response.status_code = 202  # nhận hồ sơ nhưng một service chưa kiểm được
        return _response(view)

    @app.get("/applications/{application_id}", response_model=ApplicationResponse)
    def get(application_id: str) -> ApplicationResponse:
        return _response(container.get.execute(application_id))

    @app.post("/applications/{application_id}/review", response_model=ApplicationResponse)
    def review(application_id: str, body: ReviewBody) -> ApplicationResponse:
        view = container.review.execute(
            application_id,
            reviewer_id=body.reviewer_id,
            decision=KycStatus(body.decision),
            reason=body.reason,
        )
        return _response(view)

    return app
