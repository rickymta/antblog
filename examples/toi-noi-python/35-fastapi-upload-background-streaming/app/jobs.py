"""Trạng thái job trong bộ nhớ và luồng sự kiện. Mất khi tiến trình khởi động lại: xem phần «BackgroundTasks hay worker»."""

import asyncio
import json
import logging
import uuid
from collections.abc import AsyncIterator
from dataclasses import asdict, dataclass, field
from enum import StrEnum

from .images import process_image
from .storage import ObjectStore

log = logging.getLogger("toinoi.jobs")


class JobStatus(StrEnum):
    QUEUED = "queued"
    RUNNING = "running"
    DONE = "done"
    FAILED = "failed"


TERMINAL = {JobStatus.DONE, JobStatus.FAILED}


@dataclass(frozen=True)
class JobEvent:
    id: int
    stage: str
    status: JobStatus
    percent: int
    message: str

    def to_sse(self) -> str:
        """Định dạng SSE: các dòng `id:`, `event:`, `data:` rồi MỘT dòng trống kết thúc sự kiện."""
        name = self.status.value if self.status in TERMINAL else "progress"
        return f"id: {self.id}\nevent: {name}\ndata: {json.dumps(asdict(self), ensure_ascii=False)}\n\n"


@dataclass
class Job:
    job_id: str
    driver_id: str
    document_key: str
    status: JobStatus = JobStatus.QUEUED
    result: dict | None = None
    events: list[JobEvent] = field(default_factory=list)
    _changed: asyncio.Condition = field(default_factory=asyncio.Condition, repr=False)

    async def publish(self, stage: str, status: JobStatus, percent: int, message: str) -> None:
        async with self._changed:
            self.status = status
            self.events.append(JobEvent(len(self.events), stage, status, percent, message))
            self._changed.notify_all()

    @property
    def finished(self) -> bool:
        return self.status in TERMINAL

    async def events_after(self, last_id: int, timeout: float) -> list[JobEvent]:
        """Chờ tới khi có sự kiện mới hơn last_id hoặc job kết thúc; hết `timeout` thì trả danh sách rỗng.

        Rỗng khi job đã kết thúc nghĩa là không bao giờ có thêm gì: người gọi phải đóng luồng.
        """
        async with self._changed:
            try:
                await asyncio.wait_for(
                    self._changed.wait_for(lambda: len(self.events) > last_id + 1 or self.finished), timeout)
            except TimeoutError:
                return []
            return self.events[last_id + 1:]


class JobRegistry:
    """Từ điển job. Chỉ tiến trình này nhìn thấy nó.

    Mã job ngẫu nhiên, không phải bộ đếm: với bộ đếm riêng mỗi tiến trình, hai worker cùng phát
    JOB-0001 và một lần GET có thể đọc nhầm job của tài xế khác. Mã ngẫu nhiên thì tiến trình
    không giữ job chỉ trả 404.
    """

    def __init__(self) -> None:
        self._jobs: dict[str, Job] = {}

    def create(self, driver_id: str, document_key: str) -> Job:
        job = Job(f"JOB-{uuid.uuid4().hex[:16]}", driver_id, document_key)
        self._jobs[job.job_id] = job
        return job

    def get(self, job_id: str) -> Job | None:
        return self._jobs.get(job_id)

    def __len__(self) -> int:
        return len(self._jobs)


async def run_job(job: Job, store: ObjectStore, stage_delay: float) -> None:
    """Việc chạy SAU KHI client đã nhận 202. Ngoại lệ không có ai bắt hộ: tự ghi trạng thái failed."""
    try:
        await job.publish("read", JobStatus.RUNNING, 10, "đọc ảnh từ kho")
        data = await asyncio.to_thread(store.get, job.document_key)
        await asyncio.sleep(stage_delay)

        await job.publish("thumbnail", JobStatus.RUNNING, 40, "tạo ảnh nhỏ và đo độ nét")
        # Pillow là CPU: đẩy sang thread để vòng lặp sự kiện vẫn phục vụ request khác.
        processed = await asyncio.to_thread(process_image, data)
        thumb_key = job.document_key.rsplit(".", 1)[0] + ".thumb.jpg"
        await asyncio.to_thread(store.put, thumb_key, processed.thumbnail)
        await asyncio.sleep(stage_delay)

        await job.publish("quality", JobStatus.RUNNING, 80, f"độ nét {processed.sharpness}")
        await asyncio.sleep(stage_delay)

        job.result = {"width": processed.width, "height": processed.height,
                      "sharpness": processed.sharpness, "quality": processed.quality,
                      "thumbnail_key": thumb_key}
        await job.publish("done", JobStatus.DONE, 100, "xong; kết quả chờ người duyệt")
    except Exception as exc:  # noqa: BLE001 - ranh giới job: không để ngoại lệ biến mất trong log
        log.exception("job %s lỗi", job.job_id)  # chi tiết (loại lỗi, thông điệp, stack) chỉ vào log
        await job.publish("failed", JobStatus.FAILED, 100, "Không xử lý được ảnh; hãy gửi lại ảnh khác.")


async def event_stream(job: Job, last_id: int, heartbeat: float) -> AsyncIterator[str]:
    """Phát lại sự kiện từ sau `last_id`, rồi chờ sự kiện mới tới khi job kết thúc."""
    while True:
        events = await job.events_after(last_id, heartbeat)
        if not events:
            if job.finished:
                return  # job xong và không còn gì để phát: đóng, không treo
            yield ": ping\n\n"  # dòng chú thích, giữ kết nối khỏi bị proxy cắt vì im lặng
            continue
        for event in events:
            yield event.to_sse()
            last_id = event.id
        if events[-1].status in TERMINAL:
            return
