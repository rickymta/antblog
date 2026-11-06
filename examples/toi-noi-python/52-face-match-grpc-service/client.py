"""Client gRPC async cho kyc-face: mọi lời gọi có deadline; có hỏi health trước khi gọi."""

import grpc
import numpy as np
from grpc_health.v1 import health_pb2, health_pb2_grpc

import face_match_pb2 as pb
import face_match_pb2_grpc as pb_grpc
from service import SERVICE_NAME


class FaceMatchClient:
    def __init__(self, target: str, *, default_timeout: float = 2.0) -> None:
        self._channel = grpc.aio.insecure_channel(target)  # gửi không giới hạn; server tự chặn ở phía nhận
        self._stub = pb_grpc.FaceMatchStub(self._channel)
        self._health = health_pb2_grpc.HealthStub(self._channel)
        self._default_timeout = default_timeout

    async def embed(self, image: bytes, *, timeout: float | None = None) -> np.ndarray:
        # timeout là deadline tính từ lúc gọi; gRPC truyền sang server qua header grpc-timeout.
        response = await self._stub.Embed(
            pb.EmbedRequest(image=image), timeout=timeout or self._default_timeout
        )
        return np.asarray(response.embedding, dtype=np.float32)

    async def compare(self, a: bytes, b: bytes, *, timeout: float | None = None) -> float:
        response = await self._stub.Compare(
            pb.CompareRequest(image_a=a, image_b=b), timeout=timeout or self._default_timeout
        )
        return response.score

    async def health(self, service: str = SERVICE_NAME, *, timeout: float = 1.0) -> str:
        """Tên trạng thái: SERVING, NOT_SERVING..."""
        response = await self._health.Check(
            health_pb2.HealthCheckRequest(service=service), timeout=timeout
        )
        return health_pb2.HealthCheckResponse.ServingStatus.Name(response.status)

    async def close(self) -> None:
        await self._channel.close()

    async def __aenter__(self) -> "FaceMatchClient":
        return self

    async def __aexit__(self, *exc: object) -> None:
        await self.close()
