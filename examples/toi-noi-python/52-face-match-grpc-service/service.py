"""Servicer FaceMatch (grpc.aio) và vòng đời của service: nạp mô hình một lần, warm-up, health.

`serving()` đóng vai `lifespan` của FastAPI (bài 31, bài 33): mở tài nguyên đắt tiền một lần khi
khởi động, đóng khi tắt. Khác ở thứ tự: server gRPC mở cổng TRƯỚC khi nạp mô hình để health check
đã trả lời được `NOT_SERVING`; chỉ sau warm-up mới chuyển `SERVING`.
"""

import asyncio
from collections.abc import AsyncIterator, Callable, Sequence
from contextlib import asynccontextmanager
from dataclasses import dataclass, field
from typing import Protocol

import grpc
import numpy as np
from grpc_health.v1 import health, health_pb2, health_pb2_grpc

import face_match_pb2 as pb
import face_match_pb2_grpc as pb_grpc
from batcher import BatcherStopped, DeadlineExpired, MicroBatcher, QueueFull
from model import (
    MAX_IMAGE_BYTES, MODEL_VERSION, BadImage, Embedder, ensure_batch_model, preprocess, warm_decoder,
)

SERVICE_NAME = "toinoi.face.v1.FaceMatch"
# Tin nhắn lớn nhất server nhận: Compare mang hai ảnh. Lớn hơn thì lõi gRPC từ chối trước handler.
MAX_MESSAGE_BYTES = 2 * MAX_IMAGE_BYTES + 64 * 1024


@dataclass(frozen=True)
class ServiceConfig:
    max_batch: int = 8
    max_wait_ms: float = 5.0
    queue_size: int = 64
    providers: tuple[str, ...] = ("CPUExecutionProvider",)
    intra_op_threads: int = 4
    # Đệm batch cho đủ cỡ cố định; chỉ cần với provider phải biên dịch lại khi hình dạng đổi.
    pad_to: int | None = None
    in_thread: bool = True
    # Ngân sách còn lại ít hơn mức này thì không đủ thời gian suy luận: từ chối ngay, khỏi xếp hàng.
    min_budget_ms: float = 5.0
    # Thời gian tối đa cho các lời gọi đang chạy hoàn tất khi tắt, trước khi gRPC hủy chúng.
    shutdown_grace: float = 5.0
    server_options: tuple[tuple[str, int], ...] = ()

    @property
    def warm_sizes(self) -> tuple[int, ...]:
        """Cỡ batch chạy thử khi warm-up: chỉ cỡ 1 và cỡ lớn nhất (hoặc đúng cỡ đệm nếu có `pad_to`),
        không phải mọi cỡ ở giữa: provider biên dịch lại theo hình dạng thì phải dùng `pad_to`."""
        if self.pad_to:
            return (self.pad_to,)
        return tuple(sorted({1, self.max_batch}))


class EmbedderLike(Protocol):
    """Giao diện tối thiểu service cần; test thay bằng bản chậm để dựng tình huống."""

    providers: list[str]

    def embed_batch(self, batch: np.ndarray, pad_to: int | None = None) -> np.ndarray: ...
    def warm_up(
        self, sizes: Sequence[int], pad_to: int | None = None
    ) -> list[tuple[int, float]]: ...


@dataclass
class Readiness:
    """Trạng thái dùng chung giữa servicer và vòng đời."""

    batcher: MicroBatcher | None = None
    config: ServiceConfig = field(default_factory=ServiceConfig)


class FaceMatchService(pb_grpc.FaceMatchServicer):
    def __init__(self, state: Readiness) -> None:
        self._state = state

    async def _vectors(
        self, images: list[tuple[str, bytes]], context: grpc.aio.ServicerContext
    ) -> list[np.ndarray]:
        batcher = self._state.batcher
        if batcher is None:  # mô hình chưa nạp xong: lỗi hạ tầng, gọi lại sau là được
            await context.abort(grpc.StatusCode.UNAVAILABLE, "mô hình đang nạp")
        # Kiểm đầu vào trước, trả INVALID_ARGUMENT: gọi lại y nguyên vẫn lỗi.
        arrays = []
        for label, raw in images:
            try:
                arrays.append(preprocess(raw))
            except BadImage as exc:
                await context.abort(grpc.StatusCode.INVALID_ARGUMENT, f"{label}: {exc}")

        # Deadline của client đến đây dưới dạng thời gian còn lại; đổi sang mốc tuyệt đối để batcher
        # so với đồng hồ của chính nó. Còn quá ít thì khỏi xếp hàng.
        remaining = context.time_remaining()
        deadline = None
        if remaining is not None:
            if remaining * 1000 < self._state.config.min_budget_ms:
                await context.abort(grpc.StatusCode.DEADLINE_EXCEEDED, "ngân sách còn lại quá ngắn")
            deadline = asyncio.get_running_loop().time() + remaining
        try:
            futures = batcher.submit_many(arrays, deadline)
        except QueueFull as exc:
            await context.abort(grpc.StatusCode.RESOURCE_EXHAUSTED, str(exc))
        except BatcherStopped as exc:
            await context.abort(grpc.StatusCode.UNAVAILABLE, str(exc))

        # Client hết hạn thì gRPC hủy handler ngay tại dòng này; gather chuyển lệnh hủy xuống các
        # Future, và batcher bỏ qua chúng khi lập batch (nếu chưa tới lượt suy luận).
        results = await asyncio.gather(*futures, return_exceptions=True)
        for result in results:
            if isinstance(result, DeadlineExpired):
                await context.abort(grpc.StatusCode.DEADLINE_EXCEEDED, str(result))
            if isinstance(result, BatcherStopped):
                await context.abort(grpc.StatusCode.UNAVAILABLE, str(result))
            if isinstance(result, BaseException):
                await context.abort(grpc.StatusCode.INTERNAL, "lỗi suy luận")  # không lộ chi tiết
        return [r for r in results if isinstance(r, np.ndarray)]

    async def Embed(self, request: pb.EmbedRequest, context: grpc.aio.ServicerContext):
        (vector,) = await self._vectors([("image", request.image)], context)
        return pb.EmbedResponse(embedding=vector.tolist(), model_version=MODEL_VERSION)

    async def Compare(self, request: pb.CompareRequest, context: grpc.aio.ServicerContext):
        a, b = await self._vectors(
            [("image_a", request.image_a), ("image_b", request.image_b)], context
        )
        # Hai vector đã chuẩn hóa L2: tích vô hướng là cosine.
        score = float(np.clip(np.dot(a, b), -1.0, 1.0))
        return pb.CompareResponse(score=score, model_version=MODEL_VERSION)


@dataclass
class Running:
    """Những gì `serving()` đưa ra cho người gọi."""

    server: grpc.aio.Server
    port: int
    batcher: MicroBatcher
    health: health.aio.HealthServicer
    embedder: EmbedderLike
    load_ms: float
    warm_up_ms: list[tuple[int, float]]


@asynccontextmanager
async def serving(
    config: ServiceConfig | None = None,
    *,
    address: str = "127.0.0.1:0",
    embedder_factory: Callable[[], EmbedderLike] | None = None,
    hold_ready: asyncio.Event | None = None,
) -> AsyncIterator[Running]:
    """Vòng đời service. `hold_ready` chỉ để test: giữ trạng thái NOT_SERVING tới khi được set."""
    config = config or ServiceConfig()
    loop = asyncio.get_running_loop()
    state = Readiness(config=config)
    health_servicer = health.aio.HealthServicer()
    # HealthServicer mặc định báo SERVING cho "" ngay từ đầu: chặn lại, chưa có mô hình.
    for name in ("", SERVICE_NAME):
        await health_servicer.set(name, health_pb2.HealthCheckResponse.NOT_SERVING)

    options = (
        ("grpc.max_receive_message_length", MAX_MESSAGE_BYTES),
        *config.server_options,
    )
    server = grpc.aio.server(options=options)
    pb_grpc.add_FaceMatchServicer_to_server(FaceMatchService(state), server)
    health_pb2_grpc.add_HealthServicer_to_server(health_servicer, server)
    port = server.add_insecure_port(address)  # không TLS: chỉ cho lab cục bộ
    await server.start()  # mở cổng trước: health check đã trả lời được

    batcher: MicroBatcher | None = None
    try:
        # Nạp mô hình và warm-up trên thread riêng để event loop vẫn trả lời health check.
        started = loop.time()
        factory = embedder_factory or (lambda: Embedder(
            ensure_batch_model(), config.providers, config.intra_op_threads))
        embedder = await asyncio.to_thread(factory)
        load_ms = (loop.time() - started) * 1000
        warm_up_ms = await asyncio.to_thread(embedder.warm_up, config.warm_sizes, config.pad_to)
        await asyncio.to_thread(warm_decoder)

        def run_batch(batch: np.ndarray) -> np.ndarray:
            return embedder.embed_batch(batch, config.pad_to)

        if hold_ready is not None:
            await hold_ready.wait()
        batcher = MicroBatcher(
            run_batch, max_batch=config.max_batch, max_wait_ms=config.max_wait_ms,
            queue_size=config.queue_size, in_thread=config.in_thread,
        )
        batcher.start()
        state.batcher = batcher
        for name in ("", SERVICE_NAME):
            await health_servicer.set(name, health_pb2.HealthCheckResponse.SERVING)
        yield Running(server, port, batcher, health_servicer, embedder, load_ms, warm_up_ms)
    finally:
        # Tắt có thứ tự: báo NOT_SERVING để bộ cân bằng tải ngừng gửi; gRPC ngừng nhận lời gọi mới và đợi các
        # lời gọi đang chạy xong (tối đa `shutdown_grace`) khi worker VẪN CÒN chạy; chỉ sau đó mới dừng
        # batcher. Dừng batcher trước thì việc đang suy luận bị hủy và client nhận UNAVAILABLE.
        await health_servicer.enter_graceful_shutdown()
        await server.stop(grace=config.shutdown_grace)
        if batcher is not None:
            state.batcher = None
            await batcher.stop()
