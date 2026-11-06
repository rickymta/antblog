"""Client gRPC async: mọi lời gọi có deadline; id yêu cầu gắn bằng interceptor."""

import asyncio

import grpc

import fraud_scoring_pb2 as pb
import fraud_scoring_pb2_grpc as pb_grpc
from interceptors import RequestIdClientInterceptor


class FraudClient:
    def __init__(
        self, target: str, *, default_timeout: float = 0.5, max_in_flight: int = 200
    ) -> None:
        self.ids = RequestIdClientInterceptor()
        self._channel = grpc.aio.insecure_channel(target, interceptors=[self.ids])
        self._stub = pb_grpc.FraudScoringStub(self._channel)
        self._default_timeout = default_timeout
        # Giới hạn số lời gọi của CLIENT NÀY đang bay (bài 23: Semaphore). Hàng chờ của gRPC core
        # trên server (grpc.server.max_pending_requests, mặc định 1000) từ chối lời gọi vượt ngưỡng
        # bằng CANCELLED trước khi tới handler; giới hạn ở client chỉ giữ tổng dưới ngưỡng đó
        # khi chỉ có một client (xem bài).
        self._slots = asyncio.Semaphore(max_in_flight)

    async def score(
        self,
        trip_id: str,
        customer_id: str,
        device_id: str,
        distance_km: str,
        *,
        driver_id: str | None = None,
        timeout: float | None = None,
    ) -> pb.ScoreResponse:
        request = pb.ScoreRequest(
            trip_id=trip_id, customer_id=customer_id, device_id=device_id, distance_km=distance_km
        )
        if driver_id is not None:  # không gán thì field vắng mặt trên dây, khác với chuỗi rỗng
            request.driver_id = driver_id
        # timeout là deadline tính từ lúc gọi: gRPC truyền nó sang server qua header grpc-timeout.
        # Thời gian chờ ô trống của Semaphore nằm TRƯỚC deadline, nên không tính vào nó.
        async with self._slots:
            return await self._stub.Score(request, timeout=timeout or self._default_timeout)

    async def close(self) -> None:
        await self._channel.close()

    async def __aenter__(self) -> "FraudClient":
        return self

    async def __aexit__(self, *exc: object) -> None:
        await self.close()
