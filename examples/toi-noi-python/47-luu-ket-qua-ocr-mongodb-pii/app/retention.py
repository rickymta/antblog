"""Hỗ trợ quan sát retention: chạy nhanh bộ quét TTL của mongod và chờ document biến mất."""

import asyncio
import contextlib
import time
from collections.abc import AsyncIterator
from typing import Any

from pymongo import AsyncMongoClient
from pymongo.asynchronous.collection import AsyncCollection
from pymongo.errors import OperationFailure

DEFAULT_TTL_MONITOR_SECS = 60  # mặc định của mongod


@contextlib.asynccontextmanager
async def fast_ttl_monitor(client: AsyncMongoClient[dict[str, Any]], seconds: int = 1) -> AsyncIterator[bool]:
    """Đặt tạm `ttlMonitorSleepSecs` rồi trả lại khi thoát. CHỈ DÀNH CHO LAB.

    Tham số này áp cho CẢ mongod, không riêng collection của lab: mọi lab khác dùng chung container cũng bị quét nhanh trong
    lúc đó. Nó không có trong trang tham số chính thức của MongoDB; lab dùng vì `setParameter` đổi được lúc chạy (đã thử trên
    MongoDB 8.3.13). Máy chủ thật quét theo chu kỳ riêng, nên «xóa đúng giờ» không bao giờ là bảo đảm tới từng giây.

    Trả lại giá trị cũ bằng `finally`; main.py bắt thêm SIGTERM để `finally` kịp chạy. Nếu giá trị đang đọc được đã <= `seconds`
    thì có khả năng một lần chạy trước bị ngắt (SIGKILL) khi chưa trả lại: coi là nhiễm bẩn, cảnh báo và trả về 60 thay vì
    «trả lại» 1. Yield True nếu đổi được; False nếu không có quyền (khi đó phải chờ tới 60 giây một lượt quét).
    """
    admin = client.admin
    old = (await admin.command("getParameter", 1, ttlMonitorSleepSecs=1))["ttlMonitorSleepSecs"]
    restore = old
    if old <= seconds:
        print(f"CẢNH BÁO: ttlMonitorSleepSecs đang là {old} (có thể một lần chạy trước bị ngắt); sẽ trả về "
              f"{DEFAULT_TTL_MONITOR_SECS}.")
        restore = DEFAULT_TTL_MONITOR_SECS
    changed = False
    try:
        await admin.command("setParameter", 1, ttlMonitorSleepSecs=seconds)
        changed = True
    except OperationFailure:
        pass
    try:
        yield changed
    finally:
        if changed:
            await admin.command("setParameter", 1, ttlMonitorSleepSecs=restore)


async def wait_until_gone(col: AsyncCollection[dict[str, Any]], query: dict[str, Any], timeout: float,
                          *, since: float | None = None) -> float | None:
    """Số giây (tính từ `since`, mặc định từ lúc gọi) cho tới khi không còn document khớp `query`;
    None nếu hết `timeout` mà vẫn còn. `since` và kết quả đều theo time.monotonic()."""
    started = time.monotonic()
    while time.monotonic() - started < timeout:
        if await col.count_documents(query) == 0:
            return time.monotonic() - (started if since is None else since)
        await asyncio.sleep(0.25)
    return None
