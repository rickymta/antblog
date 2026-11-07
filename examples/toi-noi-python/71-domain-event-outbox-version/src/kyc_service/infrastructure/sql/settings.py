"""Cấu hình kết nối PostgreSQL của lab 71: URL từ TOINOI_PG_URL, schema riêng `lab71`."""

import os

# URL mặc định của hạ tầng dùng chung (examples/toi-noi-python/infra/compose.yml).
DEFAULT_PG_URL = "postgresql://toinoi:toinoi@localhost:54329/toinoi"
PG_URL = os.environ.get("TOINOI_PG_URL", DEFAULT_PG_URL)
SCHEMA = "lab71"
COMPOSE_HINT = "docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait"


def sqlalchemy_url(url: str = PG_URL) -> str:
    """`postgresql+psycopg://` chọn psycopg 3 làm driver một cách tường minh (xem lab 24)."""
    for prefix in ("postgresql://", "postgres://"):
        if url.startswith(prefix):
            return "postgresql+psycopg://" + url[len(prefix) :]
    return url
