"""Cấu hình kết nối của lab 25: URL lấy từ TOINOI_PG_URL, schema riêng `lab25`."""

import os

DEFAULT_PG_URL = "postgresql://toinoi:toinoi@localhost:54329/toinoi"
PG_URL = os.environ.get("TOINOI_PG_URL", DEFAULT_PG_URL)
SCHEMA = "lab25"


def sqlalchemy_url(url: str = PG_URL) -> str:
    """`postgresql+psycopg://` chọn psycopg 3 làm driver một cách tường minh (xem lab 24)."""
    for prefix in ("postgresql://", "postgres://"):
        if url.startswith(prefix):
            return "postgresql+psycopg://" + url[len(prefix):]
    return url
