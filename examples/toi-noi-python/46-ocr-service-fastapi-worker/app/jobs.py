"""Worker arq: hàm job `ocr_document` và cấu hình `WorkerSettings`.

Chạy worker riêng (từ thư mục gốc repository):
    uv run --directory examples/toi-noi-python/46-ocr-service-fastapi-worker arq app.jobs.WorkerSettings
"""

import asyncio
import json
import logging
import os
import time
from typing import Any

from arq.connections import RedisSettings
from arq.worker import func
from PIL import Image

from .images import decode_fully
from .ocr import (LANGS, PSM, OcrFailed, OcrTimeout, find_tesseract, parse_card, run_tesseract,
                  tesseract_version, tsv_lines)
from .settings import Settings
from .storage import LocalObjectStore, sha256_hex

log = logging.getLogger("toinoi.ocr")

REVIEW_THRESHOLD = 0.8  # dưới ngưỡng hoặc thiếu trường: cần người duyệt (cùng ngưỡng bài 26)


# arq mặc định serialize bằng pickle; giải pickle dữ liệu đọc từ Redis là chạy mã của người ghi vào Redis.
# Như bài 40, lab dùng JSON: job chỉ mang chuỗi và số, xem được bằng redis-cli.
def serialize(data: Any) -> bytes:
    return json.dumps(data, default=repr).encode()


def deserialize(raw: bytes) -> Any:
    return json.loads(raw)


# Thông báo gửi ra client là câu cố định; chi tiết (stderr của Tesseract, thông báo của Pillow) chỉ vào log.
ERROR_MESSAGES = {
    "image_unreadable": "không giải nén được ảnh",
    "ocr_timeout": "OCR quá hạn",
    "ocr_failed": "Tesseract không đọc được ảnh",
}


def failure(code: str, **log_detail: object) -> dict[str, Any]:
    """Thất bại có phân loại. Trả về bình thường (không raise) để arq coi job là «chạy xong» và không tự thử lại.
    `image_unreadable` chạy lại vẫn vậy nên được nhớ trong cửa sổ idempotency; `ocr_timeout` có thể do worker quá
    tải nên API cho phép gửi lại (xem RETRYABLE trong app/main.py)."""
    if log_detail:
        log.warning("job thất bại %s: %s", code, log_detail)
    return {"ok": False, "error": {"code": code, "message": ERROR_MESSAGES[code]}}


async def startup(ctx: dict[str, Any]) -> None:
    settings = ctx.setdefault("settings", Settings())  # kiểm thử có thể đưa Settings riêng qua ctx
    ctx["store"] = LocalObjectStore(settings.upload_dir)
    ctx["tesseract"] = find_tesseract()  # thiếu binary: worker không khởi động, thay vì lỗi ở mỗi job
    ctx["model_version"] = f"tesseract-{await tesseract_version(ctx['tesseract'])}+{LANGS}/psm{PSM}"


async def ocr_document(ctx: dict[str, Any], kyc_id: str, object_key: str, document_type: str) -> dict[str, Any]:
    settings: Settings = ctx["settings"]
    data = await asyncio.to_thread(ctx["store"].get, object_key)
    try:
        # Giải nén thật trong thread: ảnh cắt cụt qua được cửa của API nhưng dừng ở đây.
        await asyncio.to_thread(decode_fully, data)
    except (OSError, Image.DecompressionBombError) as exc:
        return failure("image_unreadable", error=str(exc))

    if settings.debug_delay_s:
        await asyncio.sleep(settings.debug_delay_s)
    started = time.perf_counter()
    try:
        tsv = await run_tesseract(ctx["tesseract"], data, timeout=settings.ocr_timeout_s)
    except OcrTimeout as exc:
        return failure("ocr_timeout", kyc_id=kyc_id, detail=str(exc))
    except OcrFailed as exc:
        return failure("ocr_failed", kyc_id=kyc_id, stderr=str(exc))
    ocr_ms = round((time.perf_counter() - started) * 1000)

    fields, field_confidence = parse_card(tsv)
    confidence = min(field_confidence.values())
    return {
        "ok": True,
        "kyc_id": kyc_id,
        "document_type": document_type,
        "image_sha256": sha256_hex(data),  # để bài 47 đặt khóa document theo ảnh
        "fields": fields,
        "field_confidence": field_confidence,
        "confidence": confidence,
        # Kết quả OCR là ĐẦU VÀO cho người duyệt, không phải quyết định xác minh.
        "needs_review": confidence < REVIEW_THRESHOLD,
        "model_version": ctx["model_version"],
        "ocr_ms": ocr_ms,
        "worker_pid": os.getpid(),  # để lab thấy job chạy ở tiến trình worker nào
        "raw_text": "\n".join(" ".join(t.text for t in line) for line in tsv_lines(tsv)),
    }


_settings = Settings()


class WorkerSettings:
    functions = [func(ocr_document, max_tries=3)]  # lần thử thêm chỉ dùng khi worker chết giữa chừng
    on_startup = startup
    redis_settings = RedisSettings.from_dsn(_settings.redis_url)
    queue_name = _settings.queue_name
    max_jobs = _settings.max_jobs
    job_timeout = _settings.job_timeout_s
    keep_result = _settings.result_ttl_s
    log_results = False  # arq mặc định in kết quả job ra log: ở đây kết quả có số giấy tờ rõ
    poll_delay = 0.1
    job_serializer = serialize
    job_deserializer = deserialize
