"""API nhận ảnh CCCD, xếp job OCR vào hàng đợi arq và cho hỏi trạng thái.

API không chạy OCR và không giữ trạng thái job: mọi thứ nằm ở Redis, nên chạy bao nhiêu tiến trình API
cũng đúng. Chạy: uv run --project <lab> uvicorn app.main:app --app-dir <lab> --port 8046 [--workers 2]
"""

import asyncio
import logging
import os
import uuid
from contextlib import asynccontextmanager
from enum import StrEnum
from typing import Annotated, Any

from arq import create_pool
from arq.connections import ArqRedis, RedisSettings
from arq.constants import in_progress_key_prefix, job_key_prefix, result_key_prefix
from arq.jobs import deserialize_job, deserialize_result
from fastapi import Depends, FastAPI, Form, Request, Response, UploadFile
from pydantic import BaseModel

from . import problems
from .images import CorruptImage, ImageTooBig, NotAnImage, inspect_image
from .jobs import deserialize, serialize
from .problems import ProblemError
from .settings import PREFIX, Settings
from .storage import LocalObjectStore, ObjectStore, object_key, sha256_hex

UPLOAD_PATH = "/v1/kyc/ocr-jobs"
MULTIPART_OVERHEAD = 64 * 1024  # boundary, tiêu đề các phần, trường form: phần «thừa» so với riêng ảnh
# Cố định, tùy ý: cùng (kyc_id, loại giấy tờ, hash ảnh) luôn ra cùng job_id (UUID v5), không cần bộ đếm hay kho nào.
# Mã suy ra được từ ba giá trị ấy, nên `GET` job phải kiểm quyền người gọi với hồ sơ (bài 34); lab chưa có xác thực.
RETRYABLE = {"ocr_timeout"}  # lỗi có thể do worker quá tải: gửi lại đúng ảnh được chạy lại. Các lỗi khác được nhớ.
JOB_NAMESPACE = uuid.UUID("5d8c3a52-61f4-4b7e-9b1d-2f0c7a9e4d11")

log = logging.getLogger("toinoi.api")


class DocumentType(StrEnum):
    CCCD = "cccd"  # GPLX cần bộ tách trường riêng (bài 44), nên API chưa nhận


class JobState(StrEnum):
    QUEUED = "queued"
    RUNNING = "running"
    DONE = "done"
    FAILED = "failed"


class JobError(BaseModel):
    code: str
    message: str


class JobOut(BaseModel):
    job_id: str
    kyc_id: str
    status: JobState
    result: dict[str, Any] | None = None  # chỉ khi done; không có raw_text
    error: JobError | None = None  # chỉ khi failed
    queue_ms: int | None = None  # từ lúc xếp hàng tới lúc worker bắt đầu (chỉ khi job đã xong)
    run_ms: int | None = None  # thời gian worker chạy job


def job_id_for(kyc_id: str, document_type: str, digest: str) -> str:
    return str(uuid.uuid5(JOB_NAMESPACE, f"{kyc_id}:{document_type}:{digest}"))


def arq_id(job_id: str) -> str:
    return PREFIX + job_id  # arq không cho đổi tiền tố key nội bộ, nên job_id mang tiền tố của lab


async def describe_job(pool: ArqRedis, settings: Settings, job_id: str) -> JobOut | None:
    """Đổi trạng thái trong Redis (khóa job, khóa đang chạy, khóa kết quả) sang bốn trạng thái của ta.

    Đọc cả ba khóa trong MỘT transaction để có ảnh chụp nhất quán. Bản đầu gọi `Job.status()` rồi `Job.info()` của arq
    (hai lượt đọc): job xong đúng giữa hai lượt thì lượt sau không thấy gì và API trả 404 cho một job có thật.
    """
    aid = arq_id(job_id)
    async with pool.pipeline(transaction=True) as tr:
        tr.get(result_key_prefix + aid)
        tr.get(job_key_prefix + aid)
        tr.exists(in_progress_key_prefix + aid)
        result_raw, job_raw, in_progress = await tr.execute()
    if result_raw:
        info = deserialize_result(result_raw, deserializer=deserialize)
        kyc_id = info.args[0]
        timing = {"queue_ms": round((info.start_time - info.enqueue_time).total_seconds() * 1000),
                  "run_ms": round((info.finish_time - info.start_time).total_seconds() * 1000)}
        if not info.success:  # hàm job ném ngoại lệ không lường trước: không đưa nội dung lỗi ra ngoài
            return JobOut(job_id=job_id, kyc_id=kyc_id, status=JobState.FAILED, **timing,
                          error=JobError(code="internal_error", message="worker gặp lỗi không lường trước"))
        if info.result["ok"]:
            return JobOut(job_id=job_id, kyc_id=kyc_id, status=JobState.DONE, **timing,
                          result={k: v for k, v in info.result.items() if k not in ("ok", "kyc_id", "raw_text")})
        return JobOut(job_id=job_id, kyc_id=kyc_id, status=JobState.FAILED, **timing, error=JobError(**info.result["error"]))
    if job_raw is None:
        return None  # chưa từng có, hoặc đã quá thời gian lưu kết quả
    kyc_id = deserialize_job(job_raw, deserializer=deserialize).args[0]
    return JobOut(job_id=job_id, kyc_id=kyc_id, status=JobState.RUNNING if in_progress else JobState.QUEUED)


def too_large(limit: int) -> ProblemError:
    return ProblemError(413, "upload-too-large", "Ảnh quá lớn", f"Tối đa {limit} byte mỗi ảnh.")


async def read_limited(file: UploadFile, limit: int) -> bytes:
    """Đọc tối đa limit+1 byte: thừa một byte là đủ để biết vượt, không nạp cả file vào RAM."""
    if file.size is not None and file.size > limit:
        raise too_large(limit)
    data = await file.read(limit + 1)
    if len(data) > limit:
        raise too_large(limit)
    return data


# ---- Dependency: lấy thứ do create_app dựng, để kiểm thử thay được (bài 36) ------------------


def get_settings(request: Request) -> Settings:
    return request.app.state.settings


def get_store(request: Request) -> ObjectStore:
    return request.app.state.store


def get_pool(request: Request) -> ArqRedis:
    return request.app.state.arq


def create_app(settings: Settings | None = None) -> FastAPI:
    settings = settings or Settings()

    @asynccontextmanager
    async def lifespan(app: FastAPI):
        app.state.arq = await create_pool(
            RedisSettings.from_dsn(settings.redis_url), default_queue_name=settings.queue_name,
            job_serializer=serialize, job_deserializer=deserialize)
        yield
        await app.state.arq.aclose()

    app = FastAPI(title="Tới Nơi · kyc-ocr", version="1.0.0", lifespan=lifespan)
    app.state.settings = settings
    app.state.store = LocalObjectStore(settings.upload_dir)
    problems.install(app)

    @app.middleware("http")
    async def edge(request: Request, call_next):
        """Lớp 1: từ chối sớm theo Content-Length. Ngoài ra ghi PID tiến trình phục vụ để thấy nhiều tiến trình."""
        if request.method == "POST" and request.url.path == UPLOAD_PATH:
            declared = request.headers.get("content-length", "")
            if declared.isdigit() and int(declared) > settings.max_upload_bytes + MULTIPART_OVERHEAD:
                return problems.problem_response(
                    request, status=413, type_=problems.TYPE_BASE + "upload-too-large", title="Ảnh quá lớn",
                    detail=f"Body khai báo {declared} byte, vượt giới hạn.")
        response = await call_next(request)
        response.headers["X-Served-By"] = str(os.getpid())
        return response

    @app.get("/healthz", tags=["ops"])
    async def healthz(pool: Annotated[ArqRedis, Depends(get_pool)]) -> dict[str, Any]:
        return {"status": "ok", "pid": os.getpid(), "queue_depth": await pool.zcard(settings.queue_name)}

    @app.post(UPLOAD_PATH, status_code=202, response_model=JobOut, operation_id="submitOcrJob", tags=["kyc"],
              responses={200: {"description": "Đã có job cho cùng hồ sơ và cùng ảnh: trả lại job đó"},
                         413: {"description": "Ảnh quá lớn"}, 415: {"description": "Không phải JPEG/PNG"}})
    async def submit(
        response: Response,
        kyc_id: Annotated[str, Form(pattern=r"^KYC-\d{4}$")],
        document_type: Annotated[DocumentType, Form()],
        file: UploadFile,
        settings: Annotated[Settings, Depends(get_settings)],
        store: Annotated[ObjectStore, Depends(get_store)],
        pool: Annotated[ArqRedis, Depends(get_pool)],
    ) -> JobOut:
        data = await read_limited(file, settings.max_upload_bytes)  # lớp 2: byte thật của phần file
        try:
            info = inspect_image(data, settings.max_pixels)  # lớp 3: nội dung thật, không tin Content-Type
        except NotAnImage as exc:
            raise ProblemError(415, "unsupported-image", "Không phải ảnh JPEG hoặc PNG", str(exc)) from exc
        except ImageTooBig as exc:
            raise ProblemError(400, "image-too-many-pixels", "Ảnh có quá nhiều điểm ảnh", str(exc)) from exc
        except CorruptImage as exc:
            raise ProblemError(400, "image-corrupt", "Ảnh bị hỏng",
                               "Không đọc trọn vẹn được file ảnh; hãy chụp hoặc gửi lại.") from exc

        digest = sha256_hex(data)
        job_id = job_id_for(kyc_id, document_type.value, digest)
        key = object_key(kyc_id, digest, info.ext)
        # Ghi trước, xếp hàng sau: job không bao giờ trỏ vào object chưa có. Ghi lại cùng ảnh là vô hại.
        await asyncio.to_thread(store.put, key, data)
        enqueued = await pool.enqueue_job(
            "ocr_document", kyc_id, key, document_type.value,
            _job_id=arq_id(job_id), _queue_name=settings.queue_name)
        out = await describe_job(pool, settings, job_id)
        assert out is not None
        if enqueued is None and out.status is JobState.FAILED and out.error and out.error.code in RETRYABLE:
            # Job trước hỏng vì quá hạn OCR: xóa khóa kết quả rồi xếp lại. Nếu hai yêu cầu cùng thử lại thì chỉ một
            # enqueue_job thành công, yêu cầu còn lại thấy job mới và rơi vào nhánh «đã có» bên dưới.
            await pool.delete(result_key_prefix + arq_id(job_id))
            enqueued = await pool.enqueue_job(
                "ocr_document", kyc_id, key, document_type.value,
                _job_id=arq_id(job_id), _queue_name=settings.queue_name)
            out = await describe_job(pool, settings, job_id)
            assert out is not None
        if enqueued is None:  # arq từ chối vì job_id đã có (đang chờ, đang chạy, hoặc đã xong chưa hết hạn)
            response.status_code = 200
            response.headers["Idempotent-Replayed"] = "true"
        else:
            response.headers["Location"] = f"{UPLOAD_PATH}/{job_id}"
        return out

    @app.get(UPLOAD_PATH + "/{job_id}", response_model=JobOut, operation_id="getOcrJob", tags=["kyc"])
    async def get_job(
        job_id: uuid.UUID, response: Response,
        settings: Annotated[Settings, Depends(get_settings)],
        pool: Annotated[ArqRedis, Depends(get_pool)],
    ) -> JobOut:
        out = await describe_job(pool, settings, str(job_id))
        if out is None:
            raise ProblemError(404, "job-not-found", "Không có job này",
                               f"{job_id} không tồn tại hoặc đã quá thời gian lưu kết quả.")
        if out.status in (JobState.QUEUED, JobState.RUNNING):
            response.headers["Retry-After"] = "1"
        return out

    return app


app = create_app()
