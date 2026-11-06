"""Adapter: bọc «SDK bản đồ» có interface khác để phần còn lại của Tới Nơi chỉ thấy RoutingService."""

from decimal import Decimal
from typing import Protocol

from domain import STANDARD, FarePolicy, calculate_fare

Point = tuple[float, float]  # (lat, lng)


# ---------- Phía có sẵn, không sửa được: SDK của nhà cung cấp (giả lập) ----------
class MapVietError(Exception):
    """Lỗi của SDK: mang mã số riêng của nhà cung cấp."""

    def __init__(self, code: int) -> None:
        super().__init__(f"MapViet lỗi {code}")
        self.code = code


class MapVietClient:
    """SDK giả: trả JSON dạng dict, quãng đường là float tính bằng MÉT, thứ tự (lng, lat)."""

    def __init__(self, routes: dict[tuple[Point, Point], float]) -> None:
        self._routes = routes

    def get_route(self, from_lnglat: Point, to_lnglat: Point) -> dict[str, list[dict[str, float]]]:
        origin = (from_lnglat[1], from_lnglat[0])
        destination = (to_lnglat[1], to_lnglat[0])
        if (origin, destination) not in self._routes:
            raise MapVietError(404)
        return {"routes": [{"distance": self._routes[(origin, destination)], "duration": 600.0}]}


# ---------- Phía Tới Nơi cần: interface của chính ta ----------
class RoutingUnavailable(Exception):
    """Lỗi của Tới Nơi: không phụ thuộc nhà cung cấp nào."""


class RoutingService(Protocol):
    def distance_km(self, origin: Point, destination: Point) -> Decimal: ...


def meters_to_km(meters: float) -> Decimal:
    """Phần lõi thuần: float mét -> Decimal km. Dùng Decimal(str(...)), không Decimal(float)."""
    return Decimal(str(meters)) / Decimal(1000)


class MapVietAdapter:
    """Dịch ba thứ: thứ tự tọa độ, đơn vị (mét -> km Decimal) và lỗi (mã số -> RoutingUnavailable)."""

    def __init__(self, client: MapVietClient) -> None:
        self._client = client

    def distance_km(self, origin: Point, destination: Point) -> Decimal:
        try:
            data = self._client.get_route(
                (origin[1], origin[0]), (destination[1], destination[0])
            )
        except MapVietError as exc:
            raise RoutingUnavailable(f"không có tuyến (mã nhà cung cấp {exc.code})") from exc
        return meters_to_km(data["routes"][0]["distance"])


def quote_trip(
    routing: RoutingService,
    origin: Point,
    destination: Point,
    policy: FarePolicy = STANDARD,
) -> Decimal:
    """Use case chỉ biết RoutingService; không biết MapViet, mét hay mã lỗi số."""
    return calculate_fare(routing.distance_km(origin, destination), policy)
