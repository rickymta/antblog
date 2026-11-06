"""Fixture hạ tầng: container dựng một lần, dữ liệu tách riêng mỗi ca test.

Quy tắc: thứ ĐẮT (container, mất vài giây) để scope rộng; thứ RẺ và phải sạch
(schema, database) để scope function.
"""

from __future__ import annotations

import os
import time
import uuid
from collections.abc import Callable, Iterator

import psycopg
import pymongo
import pytest
from testcontainers.community.mongodb import MongoDbContainer
from testcontainers.community.postgres import PostgresContainer

from trips_db import DDL, OcrDocumentStore, TripRepository

PG_IMAGE = "pgvector/pgvector:pg17"  # cùng image với infra/compose.yml
MONGO_IMAGE = "mongo:8"


def _container_scope(fixture_name: str, config: pytest.Config) -> str:
    """Scope động: mặc định `session`; đặt LAB28_SCOPE=function để đo giá của sai lầm kinh điển."""
    return os.environ.get("LAB28_SCOPE", "session")


@pytest.fixture(scope=_container_scope)
def pg_dsn() -> Iterator[str]:
    """DSN của PostgreSQL. Có TOINOI_PG_URL thì dùng compose sẵn; không thì dựng container."""
    if url := os.environ.get("TOINOI_PG_URL"):
        yield url
        return
    started = time.perf_counter()
    # driver=None: URL dạng postgresql://... (psycopg tự hiểu), không phải postgresql+psycopg2://
    with PostgresContainer(PG_IMAGE, driver=None) as pg:
        print(f"\n[lab28] PostgreSQL container sẵn sàng sau {time.perf_counter() - started:.2f} s")
        yield pg.get_connection_url()


@pytest.fixture(scope=_container_scope)
def mongo_url() -> Iterator[str]:
    """URL của MongoDB. Có TOINOI_MONGO_URL thì dùng compose sẵn; không thì dựng container."""
    if url := os.environ.get("TOINOI_MONGO_URL"):
        yield url
        return
    started = time.perf_counter()
    with MongoDbContainer(MONGO_IMAGE) as mongo:
        print(f"\n[lab28] MongoDB container sẵn sàng sau {time.perf_counter() - started:.2f} s")
        yield mongo.get_connection_url()


@pytest.fixture
def pg_schema(pg_dsn: str) -> Iterator[str]:
    """Một schema mới cho mỗi ca test, có bảng từ DDL; xóa cả schema khi ca xong."""
    name = f"t_{uuid.uuid4().hex[:10]}"
    with psycopg.connect(pg_dsn, autocommit=True) as admin:
        admin.execute(f'CREATE SCHEMA "{name}"')
        admin.execute(f'SET search_path TO "{name}"')
        admin.execute(DDL)
    try:
        yield name
    finally:
        with psycopg.connect(pg_dsn, autocommit=True) as admin:
            admin.execute(f'DROP SCHEMA "{name}" CASCADE')


@pytest.fixture
def connect(pg_dsn: str, pg_schema: str) -> Iterator[Callable[[], psycopg.Connection]]:
    """Nhà máy kết nối: mọi kết nối đều trỏ search_path vào schema của ca test hiện tại."""
    opened: list[psycopg.Connection] = []

    def _connect() -> psycopg.Connection:
        conn = psycopg.connect(pg_dsn, autocommit=True, options=f"-c search_path={pg_schema}")
        opened.append(conn)
        return conn

    yield _connect
    for conn in opened:
        conn.close()


@pytest.fixture
def conn(connect: Callable[[], psycopg.Connection]) -> psycopg.Connection:
    return connect()


@pytest.fixture
def trips(conn: psycopg.Connection) -> TripRepository:
    return TripRepository(conn)


@pytest.fixture
def ocr_store(mongo_url: str) -> Iterator[OcrDocumentStore]:
    """Một database MongoDB riêng cho mỗi ca test; xóa cả database khi xong."""
    db_name = f"t_{uuid.uuid4().hex[:10]}"
    # tz_aware=True: datetime đọc về là aware UTC (mặc định PyMongo trả datetime naive).
    client = pymongo.MongoClient(mongo_url, serverSelectionTimeoutMS=5000, tz_aware=True)
    try:
        yield OcrDocumentStore(client[db_name]["ocr_documents"])
    finally:
        client.drop_database(db_name)
        client.close()
