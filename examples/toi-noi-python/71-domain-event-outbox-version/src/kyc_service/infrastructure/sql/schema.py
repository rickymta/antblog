"""Tạo và xóa schema của lab. Production dùng migration (Alembic, bài 24), không `create_all`."""

from sqlalchemy import text
from sqlalchemy.engine import Engine

from kyc_service.infrastructure.sql.settings import SCHEMA
from kyc_service.infrastructure.sql.tables import Base


def drop_schema(engine: Engine) -> None:
    with engine.begin() as conn:
        conn.execute(text(f'DROP SCHEMA IF EXISTS "{SCHEMA}" CASCADE'))


def create_schema(engine: Engine) -> None:
    drop_schema(engine)
    with engine.begin() as conn:
        conn.execute(text(f'CREATE SCHEMA "{SCHEMA}"'))
    Base.metadata.create_all(engine)


def reset_data(engine: Engine) -> None:
    """Xóa dữ liệu giữa các test và đưa sequence về đầu; giữ nguyên cấu trúc bảng."""
    with engine.begin() as conn:
        conn.execute(
            text(
                f'TRUNCATE "{SCHEMA}".kyc_applications, "{SCHEMA}".outbox RESTART IDENTITY CASCADE'
            )
        )
        conn.execute(text(f'ALTER SEQUENCE "{SCHEMA}".kyc_application_seq RESTART WITH 1'))
