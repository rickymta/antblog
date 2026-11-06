"""Cấu hình của lab 39: PostgreSQL (schema `lab39`) và Kafka (topic có tiền tố `lab39.`)."""

import os

DEFAULT_PG_URL = "postgresql://toinoi:toinoi@localhost:54329/toinoi"
PG_URL = os.environ.get("TOINOI_PG_URL", DEFAULT_PG_URL)
KAFKA_BOOTSTRAP = os.environ.get("TOINOI_KAFKA_BOOTSTRAP", "localhost:9094")
SCHEMA = "lab39"

# Topic thật của luồng outbox, và một topic riêng cho phần minh họa dual-write ngây thơ
# để các sự kiện «ma» của phần đó không lẫn vào luồng chính.
TOPIC = "lab39.trip.completed.v1"
NAIVE_TOPIC = "lab39.naive.trip.completed.v1"


def sqlalchemy_url(url: str = PG_URL) -> str:
    """`postgresql+psycopg://` chọn psycopg 3 làm driver một cách tường minh (xem lab 24)."""
    for prefix in ("postgresql://", "postgres://"):
        if url.startswith(prefix):
            return "postgresql+psycopg://" + url[len(prefix):]
    return url
