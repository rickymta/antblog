"""Quy tắc chấm điểm và servicer gRPC bất đồng bộ (grpc.aio) của fraud-scoring."""

import asyncio
from dataclasses import dataclass, field
from decimal import Decimal, InvalidOperation

import grpc

import fraud_scoring_pb2 as pb
import fraud_scoring_pb2_grpc as pb_grpc
from interceptors import LoggingServerInterceptor

MODEL_VERSION = "rules-2026.11.1"
MAX_KM = Decimal("100")

# Trọng số của bộ luật mẫu. Đây là luật viết tay để minh họa hợp đồng, không phải mô hình thật.
W_DEVICE_SHARED = 0.45
W_PAIR_REPEAT = 0.35
W_LONG_TRIP = 0.15


@dataclass
class FeatureStore:
    """Đặc trưng online giả. Thật sự đến từ Redis/PostgreSQL (bài 24, 27); ở đây là dict."""

    device_customers: dict[str, set[str]] = field(default_factory=dict)
    pair_trips_24h: dict[tuple[str, str], int] = field(default_factory=dict)
    # Độ trễ mô phỏng của lần tra cứu đặc trưng (giây); dùng để thấy deadline hoạt động.
    lookup_delay: float = 0.0
    # Bộ đếm để kiểm: handler chạy xong hay bị hủy giữa chừng khi client hết hạn.
    completed: int = 0
    cancelled: int = 0

    @classmethod
    def seeded(cls) -> "FeatureStore":
        return cls(
            device_customers={"DEV-0099": {"CU-101", "CU-102", "CU-103"}},
            pair_trips_24h={("DR-007", "CU-050"): 4},
        )


def parse_distance(raw: str) -> Decimal:
    """Chuỗi thập phân -> Decimal trong miền 0..100; sai thì ValueError."""
    try:
        value = Decimal(raw)
    except InvalidOperation:
        raise ValueError(f"khoảng cách không phải số: {raw!r}") from None
    if not value.is_finite() or not (Decimal("0") <= value <= MAX_KM):
        raise ValueError(f"khoảng cách ngoài miền 0..100 km: {raw}")
    return value


def score_trip(
    request: pb.ScoreRequest, distance: Decimal, store: FeatureStore
) -> tuple[float, list[str]]:
    """Cộng điểm theo luật; trả (điểm 0..1, mã lý do). Thuần: không I/O, test được riêng."""
    score, reasons = 0.0, []
    others = store.device_customers.get(request.device_id, set()) - {request.customer_id}
    if len(others) >= 3:  # thiết bị này đã dùng cho từ 3 khách khác trở lên
        score += W_DEVICE_SHARED
        reasons.append("device_shared")
    if request.HasField("driver_id"):  # proto3 optional: phân biệt "chưa có" với chuỗi rỗng
        if store.pair_trips_24h.get((request.driver_id, request.customer_id), 0) >= 3:
            score += W_PAIR_REPEAT
            reasons.append("pair_repeat")
    if distance >= 50:
        score += W_LONG_TRIP
        reasons.append("long_trip")
    return round(min(score, 1.0), 2), reasons


class FraudScoringService(pb_grpc.FraudScoringServicer):
    def __init__(self, store: FeatureStore) -> None:
        self._store = store

    async def Score(self, request: pb.ScoreRequest, context: grpc.aio.ServicerContext):
        # Kiểm đầu vào trước, trả INVALID_ARGUMENT: lỗi của người gọi, gọi lại y nguyên vẫn lỗi.
        if not request.trip_id or not request.customer_id:
            await context.abort(grpc.StatusCode.INVALID_ARGUMENT, "thiếu trip_id hoặc customer_id")
        try:
            distance = parse_distance(request.distance_km)
        except ValueError as exc:
            await context.abort(grpc.StatusCode.INVALID_ARGUMENT, str(exc))

        try:
            # Tra cứu đặc trưng là điểm chờ I/O. Client hết hạn thì gRPC hủy coroutine này:
            # CancelledError được ném đúng chỗ `await`, nên ta dọn và đếm được.
            await asyncio.sleep(self._store.lookup_delay)
            score, reasons = score_trip(request, distance, self._store)
            self._store.completed += 1
        except asyncio.CancelledError:
            self._store.cancelled += 1
            raise
        return pb.ScoreResponse(score=score, reasons=reasons, model_version=MODEL_VERSION)


async def start_server(
    store: FeatureStore,
    address: str = "127.0.0.1:0",
    options: tuple[tuple[str, int], ...] = (),
) -> tuple[grpc.aio.Server, int, LoggingServerInterceptor]:
    """Dựng server grpc.aio; cổng 0 = hệ điều hành chọn cổng trống. Trả (server, cổng, nhật ký)."""
    log = LoggingServerInterceptor()
    server = grpc.aio.server(interceptors=[log], options=options)
    pb_grpc.add_FraudScoringServicer_to_server(FraudScoringService(store), server)
    port = server.add_insecure_port(address)  # không TLS: chỉ cho lab cục bộ
    await server.start()
    return server, port, log
