"""Interceptor: server ghi nhật ký mỗi lời gọi; client gắn x-request-id và đo thời gian."""

import asyncio
import time
import uuid
from dataclasses import dataclass

import grpc


@dataclass(frozen=True)
class CallRecord:
    method: str
    code: grpc.StatusCode
    millis: float
    request_id: str


class LoggingServerInterceptor(grpc.aio.ServerInterceptor):
    """Bọc handler unary-unary: ghi (method, status, thời gian, x-request-id) vào `records`."""

    def __init__(self) -> None:
        self.records: list[CallRecord] = []

    async def intercept_service(self, continuation, handler_call_details):
        handler = await continuation(handler_call_details)
        if handler is None or handler.unary_unary is None:
            return handler  # chỉ bọc unary-unary; kiểu khác để nguyên
        inner = handler.unary_unary
        metadata = dict(handler_call_details.invocation_metadata or ())
        request_id = metadata.get("x-request-id", "-")

        async def logged(request, context):
            start = time.perf_counter()
            code = grpc.StatusCode.OK
            try:
                return await inner(request, context)
            except grpc.aio.AbortError:
                code = context.code()
                raise
            except asyncio.CancelledError:
                code = grpc.StatusCode.CANCELLED  # nhãn quy ước của lab: handler bị hủy (client bỏ hoặc hết hạn)
                raise
            finally:
                self.records.append(
                    CallRecord(
                        handler_call_details.method,
                        code,
                        (time.perf_counter() - start) * 1000,
                        request_id,
                    )
                )

        return grpc.unary_unary_rpc_method_handler(
            logged,
            request_deserializer=handler.request_deserializer,
            response_serializer=handler.response_serializer,
        )


class RequestIdClientInterceptor(grpc.aio.UnaryUnaryClientInterceptor):
    """Gắn x-request-id vào metadata mỗi lời gọi; giữ id cuối để test đối chiếu với server."""

    def __init__(self) -> None:
        self.last_request_id = ""

    async def intercept_unary_unary(self, continuation, client_call_details, request):
        self.last_request_id = uuid.uuid4().hex[:12]
        metadata = list(client_call_details.metadata or ())
        metadata.append(("x-request-id", self.last_request_id))
        details = client_call_details._replace(metadata=metadata)
        return await continuation(details, request)
