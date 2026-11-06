"""Service tiếp nhận ảnh giấy tờ tài xế: upload, xử lý nền, theo dõi tiến độ bằng SSE.

Chạy: uv run --project <lab> uvicorn app.main:app --app-dir <lab> --port 8035
"""

import asyncio
import logging
from collections.abc import AsyncIterator
from enum import StrEnum
from typing import Annotated

from fastapi import BackgroundTasks, Depends, FastAPI, Form, Header, Request, Response, UploadFile
from fastapi.responses import StreamingResponse
from fastapi.sse import EventSourceResponse, ServerSentEvent
from pydantic import BaseModel

from . import problems
from .errors import JobNotFound, UploadTooLarge
from .images import inspect_image
from .jobs import TERMINAL, Job, JobRegistry, JobStatus, event_stream, run_job
from .problems import ProblemDetails, problem_response
from .settings import Settings
from .storage import LocalObjectStore, ObjectStore, object_key

UPLOAD_PATH = "/v1/kyc/documents"
# Phần «thừa» của multipart (boundary, tiêu đề các phần, trường form) so với riêng ảnh.
MULTIPART_OVERHEAD = 64 * 1024

log = logging.getLogger("toinoi.upload")


class DocumentKind(StrEnum):
    ID_CARD = "id_card"
    LICENSE = "license"


class DocumentOut(BaseModel):
    key: str
    kind: DocumentKind
    size: int
    format: str
    width: int
    height: int
    original_filename: str | None


class UploadAccepted(BaseModel):
    job_id: str
    status: JobStatus
    document: DocumentOut


class JobOut(BaseModel):
    job_id: str
    driver_id: str
    status: JobStatus
    percent: int
    result: dict | None


# ---- Dependency: lấy thứ do create_app dựng, để test thay được (bài 36) --------------------


def get_settings(request: Request) -> Settings:
    return request.app.state.settings


def get_store(request: Request) -> ObjectStore:
    return request.app.state.store


def get_jobs(request: Request) -> JobRegistry:
    return request.app.state.jobs


def job_or_404(job_id: str, jobs: Annotated[JobRegistry, Depends(get_jobs)]) -> Job:
    job = jobs.get(job_id)
    if job is None:
        raise JobNotFound(job_id)
    return job


async def read_limited(file: UploadFile, limit: int) -> bytes:
    """Lớp 2. Starlette luôn đặt `file.size` (số byte đã nhận của phần file), nên nhánh đầu là nhánh chạy
    thật. Nhánh đọc limit+1 byte là chốt dự phòng khi `size` là None (UploadFile dựng tay, bản cũ)."""
    if file.size is not None and file.size > limit:
        raise UploadTooLarge(limit)
    data = await file.read(limit + 1)
    if len(data) > limit:
        raise UploadTooLarge(limit)
    return data


def create_app(settings: Settings | None = None) -> FastAPI:
    settings = settings or Settings()
    app = FastAPI(title="Tới Nơi · KYC upload", version="1.0.0")
    app.state.settings = settings
    app.state.store = LocalObjectStore(settings.upload_dir)
    app.state.jobs = JobRegistry()
    problems.install(app)

    @app.middleware("http")
    async def reject_oversized_body(request: Request, call_next):
        """Lớp 1: nhìn Content-Length ĐỂ TỪ CHỐI SỚM, trước khi Starlette đọc cả body vào ổ đĩa."""
        if request.method == "POST" and request.url.path == UPLOAD_PATH:
            declared = request.headers.get("content-length", "")
            if declared.isdigit() and int(declared) > settings.max_upload_bytes + MULTIPART_OVERHEAD:
                exc = UploadTooLarge(settings.max_upload_bytes, declared)
                return problem_response(request, status=413, type_=problems.TYPE_BASE + "upload-too-large",
                                        title="Ảnh quá lớn", detail=f"Body khai báo {declared} byte, vượt giới hạn "
                                        f"{exc.limit} byte mỗi ảnh.", max_bytes=exc.limit)
        return await call_next(request)

    @app.post(UPLOAD_PATH, status_code=202, response_model=UploadAccepted,
              operation_id="uploadKycDocument", tags=["kyc"],
              responses={400: {"model": ProblemDetails, "description": "Ảnh hỏng hoặc quá nhiều điểm ảnh"},
                         413: {"model": ProblemDetails, "description": "Ảnh quá lớn"},
                         415: {"model": ProblemDetails, "description": "Không phải JPEG/PNG"},
                         422: {"model": ProblemDetails, "description": "Sai mã tài xế hoặc loại giấy tờ"}})
    async def upload_document(
        response: Response,
        background: BackgroundTasks,
        driver_id: Annotated[str, Form(pattern=r"^DR-\d{3}$")],
        kind: Annotated[DocumentKind, Form()],
        file: UploadFile,
        settings: Annotated[Settings, Depends(get_settings)],
        store: Annotated[ObjectStore, Depends(get_store)],
        jobs: Annotated[JobRegistry, Depends(get_jobs)],
    ) -> UploadAccepted:
        # Lớp 2: kích thước thật của phần file (đúng cả khi client không gửi Content-Length).
        data = await read_limited(file, settings.max_upload_bytes)
        info = inspect_image(data, settings.max_pixels)  # lớp 3: nội dung thật, không tin Content-Type

        key = object_key(driver_id, data, info.ext)  # khóa từ sha256, không từ file.filename
        await asyncio.to_thread(store.put, key, data)

        job = jobs.create(driver_id, key)
        await job.publish("stored", JobStatus.QUEUED, 0, "đã lưu ảnh, xếp hàng xử lý")
        # Chạy SAU KHI response 202 được gửi; không retry, mất nếu tiến trình chết.
        background.add_task(run_job, job, store, settings.stage_delay)

        response.headers["Location"] = f"/v1/kyc/jobs/{job.job_id}"
        filename = file.filename.replace("\\", "/").rsplit("/", 1)[-1] if file.filename else None  # chỉ để hiển thị lại
        return UploadAccepted(job_id=job.job_id, status=job.status, document=DocumentOut(
            key=key, kind=kind, size=len(data), format=info.format, width=info.width,
            height=info.height, original_filename=filename))

    @app.get("/v1/kyc/jobs/{job_id}", response_model=JobOut, operation_id="getKycJob", tags=["kyc"],
             responses={404: {"model": ProblemDetails, "description": "Không có job này trong tiến trình"}})
    async def get_job(job: Annotated[Job, Depends(job_or_404)]) -> JobOut:
        return JobOut(job_id=job.job_id, driver_id=job.driver_id, status=job.status,
                      percent=job.events[-1].percent if job.events else 0, result=job.result)

    def resume_point(last_event_id: int | None) -> int:
        """Trình duyệt tự gửi Last-Event-ID khi nối lại; phát tiếp từ sau id đó."""
        return -1 if last_event_id is None else last_event_id

    LastEventId = Annotated[int | None, Header(ge=0)]  # id âm bị từ chối bằng 422

    @app.get("/v1/kyc/jobs/{job_id}/events", operation_id="streamKycJobEvents", tags=["kyc"],
             response_class=StreamingResponse)
    async def job_events(
        job: Annotated[Job, Depends(job_or_404)],
        settings: Annotated[Settings, Depends(get_settings)],
        last_event_id: LastEventId = None,
    ) -> Response:
        """SSE viết tay bằng StreamingResponse: thấy rõ định dạng trên dây."""
        last_id = resume_point(last_event_id)
        if job.finished and last_id >= len(job.events) - 1:
            # Client nối lại sau khi đã nhận hết: 204 báo EventSource ngừng nối lại (theo đặc tả).
            return Response(status_code=204)
        return StreamingResponse(
            event_stream(job, last_id, settings.sse_heartbeat),
            media_type="text/event-stream",
            headers={"Cache-Control": "no-cache", "X-Accel-Buffering": "no"})

    @app.get("/v1/kyc/jobs/{job_id}/events-native", operation_id="streamKycJobEventsNative",
             tags=["kyc"], response_class=EventSourceResponse)
    async def job_events_native(
        job: Annotated[Job, Depends(job_or_404)],
        last_event_id: LastEventId = None,
    ) -> AsyncIterator[ServerSentEvent]:
        """Cùng nội dung bằng EventSourceResponse có sẵn của FastAPI (tự thêm ping). Hàm sinh không trả
        được 204: nối lại sau khi xong thì luồng rỗng rồi đóng, client nên tự close() khi nhận `done`."""
        last_id = resume_point(last_event_id)
        while True:
            events = await job.events_after(last_id, timeout=3600)
            for event in events:
                yield ServerSentEvent(data=event, event=event.status.value if event.status in TERMINAL
                                      else "progress", id=str(event.id))
                last_id = event.id
            if not events or events[-1].status in TERMINAL:
                return  # rỗng nghĩa là hết giờ chờ hoặc job đã xong mà không còn gì mới

    problems.problem_media_type_in_openapi(app)
    return app


app = create_app()
