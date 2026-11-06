"""Cấu hình kết nối của lab 24: URL lấy từ TOINOI_PG_URL, schema riêng `lab24`."""

import os

# URL mặc định của hạ tầng dùng chung (examples/toi-noi-python/infra/compose.yml).
DEFAULT_PG_URL = "postgresql://toinoi:toinoi@localhost:54329/toinoi"
PG_URL = os.environ.get("TOINOI_PG_URL", DEFAULT_PG_URL)
SCHEMA = "lab24"


def sqlalchemy_url(url: str = PG_URL) -> str:
    """SQLAlchemy chọn driver qua phần sau dấu `+`: `postgresql+psycopg://` là psycopg 3.

    Ta ghi rõ `+psycopg` thay vì dựa vào driver mặc định của `postgresql://`: SQLAlchemy 2.0
    mặc định là psycopg2, còn từ 2.1 mặc định đã đổi sang psycopg 3.
    """
    for prefix in ("postgresql://", "postgres://"):
        if url.startswith(prefix):
            return "postgresql+psycopg://" + url[len(prefix):]
    return url
