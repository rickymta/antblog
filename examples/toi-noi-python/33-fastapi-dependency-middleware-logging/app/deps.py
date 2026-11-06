"""Các dependency của API: cấu hình, phiên kho chuyến, khách hiện tại, đồng hồ."""

import logging
import re
from collections.abc import AsyncIterator
from datetime import UTC, datetime
from typing import Annotated

from fastapi import Depends, Header, HTTPException, Request

from app.repo import Database, TripSession
from app.settings import Settings

log = logging.getLogger("toinoi.deps")


def get_settings(request: Request) -> Settings:
    # Cấu hình tạo MỘT lần trong create_app; dependency chỉ đưa nó tới handler (và test thay được).
    return request.app.state.settings


def get_db(request: Request) -> Database:
    return request.app.state.db


async def get_session(request: Request) -> AsyncIterator[TripSession]:
    """Dependency có yield: phần trước yield chạy trước handler, phần sau chạy để dọn dẹp.
    `try/finally` bắt buộc: không có nó, handler ném lỗi thì phiên không bao giờ được đóng."""
    session = request.app.state.db.open_session()
    log.debug("db.session.opened", extra={"session_no": session.session_no})
    try:
        yield session
    finally:
        session.close()
        log.debug("db.session.closed", extra={"session_no": session.session_no})


_CUSTOMER_ID = re.compile(r"CU-\d{3}")


def current_customer(request: Request, x_customer_id: Annotated[str | None, Header()] = None) -> str:
    """CHỖ TẠM: tin header do client gửi. Bài 34 thay phần nguồn của dependency này bằng token đã
    ký; handler vẫn nhận `CustomerDep`, chỉ bổ sung kiểm theo vai trò. Không dùng header kiểu này
    ngoài bài học."""
    if x_customer_id is None or not _CUSTOMER_ID.fullmatch(x_customer_id):
        # 401 luôn kèm WWW-Authenticate (RFC 9110), kể cả ở chỗ tạm này.
        raise HTTPException(
            status_code=401,
            detail="thiếu hoặc sai X-Customer-Id",
            headers={"WWW-Authenticate": "Bearer"},
        )
    # request.state là nơi middleware đọc lại sau khi handler chạy xong (xem app/middleware.py).
    request.state.customer_id = x_customer_id
    return x_customer_id


def utc_now() -> datetime:
    return datetime.now(UTC)


async def get_session_with_commit(request: Request) -> AsyncIterator[TripSession]:
    """Như get_session nhưng commit ở cuối và commit luôn thất bại: để so scope request với function."""
    session = request.app.state.db.open_session(fail_commit=True)
    try:
        yield session
        session.commit()
    finally:
        session.close()


SettingsDep = Annotated[Settings, Depends(get_settings)]
DbDep = Annotated[Database, Depends(get_db)]
# scope="function": phần sau yield chạy ngay khi handler trả về, TRƯỚC khi response được gửi.
SessionDep = Annotated[TripSession, Depends(get_session, scope="function")]
CustomerDep = Annotated[str, Depends(current_customer)]
NowDep = Annotated[datetime, Depends(utc_now)]
