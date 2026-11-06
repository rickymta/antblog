"""fare.py: tính cước từ dữ liệu thiết bị (mét), có exception tùy biến và logging."""

import logging
from decimal import ROUND_HALF_UP, Decimal, InvalidOperation

# Mỗi module một logger, đặt tên theo __name__ ("fare" khi được import).
logger = logging.getLogger(__name__)

# Mô hình minh họa dùng chung cả loạt: không phải bảng giá vận hành thực tế.
BASE_FARE = Decimal("12000")
INCLUDED_KM = Decimal("2")
PRICE_PER_EXTRA_KM = Decimal("4500")
MAX_KM = Decimal("100")
METRES_PER_KM = Decimal("1000")


class FareError(ValueError):
    """Gốc của mọi lỗi nghiệp vụ tính cước. Kế thừa ValueError để code cũ bắt ValueError vẫn chạy."""


class DistanceOutOfRangeError(FareError):
    """Khoảng cách hợp lệ về kiểu nhưng ngoài miền 0..100 km."""


class DeviceDataError(FareError):
    """Thiết bị gửi chuỗi không đọc được thành số."""


def parse_device_metres(raw: str) -> Decimal:
    """Đọc số mét từ chuỗi thiết bị. Gói lỗi gốc bằng `raise ... from` để giữ nguyên nhân."""
    try:
        return Decimal(raw)
    except InvalidOperation as exc:
        raise DeviceDataError(f"khoảng cách không đọc được: {raw!r}") from exc


def to_km(metres: Decimal, *, buggy: bool = False) -> Decimal:
    """Đổi mét sang km."""
    divisor = Decimal("500") if buggy else METRES_PER_KM
    return metres / divisor


def calculate_fare(distance_km: Decimal) -> Decimal:
    """Tính cước từ khoảng cách km. Validation bằng raise, không dùng assert."""
    if distance_km < 0 or distance_km > MAX_KM:
        raise DistanceOutOfRangeError(f"khoảng cách ngoài miền 0..{MAX_KM} km: {distance_km}")
    extra = max(Decimal(0), distance_km - INCLUDED_KM)
    raw = BASE_FARE + extra * PRICE_PER_EXTRA_KM
    total = raw.quantize(Decimal("1"), rounding=ROUND_HALF_UP)
    assert total >= BASE_FARE  # bất biến nội bộ: lập trình viên tin điều này luôn đúng
    return total


def quote_from_metres(raw_metres: str, *, buggy: bool = False, pause: bool = False) -> Decimal:
    """Báo giá từ chuỗi mét của thiết bị. `buggy=True` bật lỗi cố ý để luyện debug."""
    if pause:
        breakpoint()  # dừng tại đây: n (next), s (step), p biểu_thức, c (continue), q (quit)
    metres = parse_device_metres(raw_metres)
    km = to_km(metres, buggy=buggy)
    total = calculate_fare(km)
    logger.debug("quote_from_metres metres=%s km=%s total=%s", metres, km, total)
    return total


def handle_request(
    raw_metres: str, *, trip_id: str = "TN-0001", buggy: bool = False, pause: bool = False
) -> str:
    """Xử lý một yêu cầu: bắt đúng loại lỗi, trả mã kết quả cho tầng gọi."""
    logger.info("nhận yêu cầu báo giá trip=%s", trip_id)
    try:
        total = quote_from_metres(raw_metres, buggy=buggy, pause=pause)
    except DistanceOutOfRangeError as exc:
        # Lỗi dự kiến của dữ liệu đầu vào: cảnh báo, không cần traceback.
        logger.warning("từ chối trip=%s: %s", trip_id, exc)
        return "REJECT"
    except DeviceDataError:
        # Lỗi dữ liệu thiết bị: ghi cả traceback (logger.exception chỉ dùng trong except).
        logger.exception("dữ liệu thiết bị hỏng trip=%s", trip_id)
        return "DEVICE_ERROR"
    else:
        # Chỉ chạy khi try không có exception.
        logger.info("báo giá thành công trip=%s total=%s", trip_id, total)
        return f"{total} VND"
    finally:
        # Luôn chạy, kể cả khi return ở nhánh trên.
        logger.debug("kết thúc xử lý trip=%s", trip_id)


def mask_phone(phone: str) -> str:
    """Che số điện thoại trước khi đưa vào log: giữ 2 ký tự đầu và 2 ký tự cuối."""
    if len(phone) <= 4:
        return "*" * len(phone)
    return phone[:2] + "*" * (len(phone) - 4) + phone[-2:]


def unsafe_guard(distance_km: Decimal) -> Decimal:
    """VÍ DỤ SAI: dùng assert để validation. Khi chạy `python -O`, dòng assert bị bỏ qua."""
    assert distance_km >= 0, "khoảng cách phải >= 0"
    return distance_km
