"""Micro-batching: gom các yêu cầu đến gần nhau thành một lần suy luận.

Mỗi yêu cầu đặt vào `asyncio.Queue` kèm một Future. MỘT worker lấy yêu cầu đầu tiên, chờ tối đa
`max_wait_ms` cho đủ `max_batch` yêu cầu, loại những yêu cầu không còn ai chờ, rồi chạy một lần
suy luận trên thread riêng (event loop vẫn rảnh để nhận lời gọi và trả lời health check).
"""

import asyncio
from collections import Counter
from collections.abc import Callable
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass, field

import numpy as np


class QueueFull(Exception):
    """Hàng chờ suy luận không còn chỗ: bên gọi nên nhận RESOURCE_EXHAUSTED rồi lùi dần."""


class DeadlineExpired(Exception):
    """Yêu cầu hết hạn trong lúc xếp hàng: chưa tốn một giây suy luận nào."""


class BatcherStopped(Exception):
    """Batcher đã dừng (server đang tắt)."""


@dataclass
class Stats:
    """Bộ đếm để lab chứng minh điều bài viết nói; không phải metric production."""

    submitted: int = 0
    rejected_full: int = 0
    batches: int = 0
    inferred: int = 0           # số ảnh đã đi qua mô hình
    delivered: int = 0          # số kết quả đến được người chờ
    skipped_cancelled: int = 0  # người chờ đã bỏ (deadline của client) trước khi suy luận
    skipped_expired: int = 0    # hạn đã qua khi lập batch, hủy chưa kịp tới
    wasted: int = 0             # người chờ bỏ GIỮA lúc thread đang suy luận: không dừng được
    batch_sizes: Counter[int] = field(default_factory=Counter)


@dataclass
class _Item:
    x: np.ndarray
    deadline: float | None  # mốc tuyệt đối theo loop.time(); None = không có hạn
    future: asyncio.Future[np.ndarray]


class MicroBatcher:
    def __init__(
        self,
        run_batch: Callable[[np.ndarray], np.ndarray],
        *,
        max_batch: int = 8,
        max_wait_ms: float = 5.0,
        queue_size: int = 64,
        in_thread: bool = True,
    ) -> None:
        self._run_batch = run_batch
        self.max_batch = max_batch
        self.max_wait = max_wait_ms / 1000
        self._queue: asyncio.Queue[_Item] = asyncio.Queue(maxsize=queue_size)
        self._in_thread = in_thread
        # Một thread suy luận: ONNX Runtime đã tự song song hóa bên trong một lần chạy.
        self._pool = ThreadPoolExecutor(max_workers=1, thread_name_prefix="infer")
        self._worker: asyncio.Task[None] | None = None
        self._stopped = False
        self.stats = Stats()

    def start(self) -> None:
        self._worker = asyncio.create_task(self._loop(), name="micro-batcher")

    async def stop(self) -> None:
        """Dừng nhận việc, hủy worker, báo lỗi cho mọi yêu cầu còn nằm trong hàng."""
        self._stopped = True
        if self._worker:
            self._worker.cancel()
            await asyncio.gather(self._worker, return_exceptions=True)
        while not self._queue.empty():
            item = self._queue.get_nowait()
            if not item.future.done():
                item.future.set_exception(BatcherStopped("server đang tắt"))
        await asyncio.to_thread(self._pool.shutdown)  # chờ thread suy luận xong mà không chặn event loop

    def submit_many(
        self, xs: list[np.ndarray], deadline: float | None
    ) -> list[asyncio.Future[np.ndarray]]:
        """Đặt TẤT CẢ hoặc không cái nào (một lời gọi Compare cần hai chỗ). Không có `await` giữa
        lúc kiểm chỗ và lúc đặt, nên không coroutine nào chen vào được."""
        if self._stopped:
            raise BatcherStopped("server đang tắt")
        if self._queue.maxsize - self._queue.qsize() < len(xs):
            self.stats.rejected_full += len(xs)
            raise QueueFull(f"hàng chờ đầy ({self._queue.qsize()}/{self._queue.maxsize})")
        loop = asyncio.get_running_loop()
        futures = []
        for x in xs:
            future: asyncio.Future[np.ndarray] = loop.create_future()
            self._queue.put_nowait(_Item(x, deadline, future))
            futures.append(future)
        self.stats.submitted += len(xs)
        return futures

    async def _collect(self) -> list[_Item]:
        """Lấy một batch: yêu cầu đầu tiên chờ vô hạn, các yêu cầu sau chỉ chờ tới hết `max_wait`."""
        loop = asyncio.get_running_loop()
        batch = [await self._queue.get()]
        cutoff = loop.time() + self.max_wait
        while len(batch) < self.max_batch:
            try:
                async with asyncio.timeout_at(cutoff):
                    batch.append(await self._queue.get())
            except TimeoutError:
                break
        return batch

    async def _loop(self) -> None:
        loop = asyncio.get_running_loop()
        while True:
            batch = await self._collect()
            live: list[_Item] = []
            try:
                now = loop.time()
                for item in batch:
                    if item.future.cancelled():
                        self.stats.skipped_cancelled += 1
                    elif item.deadline is not None and now >= item.deadline:
                        self.stats.skipped_expired += 1
                        item.future.set_exception(DeadlineExpired("hết hạn trong hàng chờ"))
                    else:
                        live.append(item)
                if not live:
                    continue
                stacked = np.stack([item.x for item in live])
                try:
                    if self._in_thread:
                        result = await loop.run_in_executor(self._pool, self._run_batch, stacked)
                    else:
                        result = self._run_batch(stacked)  # chặn event loop: chỉ để so sánh trong bài
                except Exception as exc:  # noqa: BLE001 - lỗi suy luận phải tới mọi người đang chờ
                    for item in live:
                        if not item.future.done():
                            item.future.set_exception(exc)
                    continue
                self.stats.batches += 1
                self.stats.inferred += len(live)
                self.stats.batch_sizes[len(live)] += 1
                for item, vector in zip(live, result, strict=True):
                    if item.future.cancelled():
                        self.stats.wasted += 1  # suy luận xong rồi mới thấy người chờ đã bỏ
                    else:
                        item.future.set_result(vector)
                        self.stats.delivered += 1
            except asyncio.CancelledError:
                # Worker bị hủy giữa lúc đang giữ một batch: không để Future nào treo mãi.
                for item in batch:
                    if not item.future.done():
                        item.future.set_exception(BatcherStopped("server đang tắt"))
                raise
