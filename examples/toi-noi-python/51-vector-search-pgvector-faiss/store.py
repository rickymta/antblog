"""PostgreSQL + pgvector: bảng, nạp dữ liệu, truy vấn top-k, chỉ mục HNSW/IVFFlat."""

import os
import sys
import time
from collections.abc import Iterator
from contextlib import contextmanager

import numpy as np
import psycopg
from pgvector.psycopg import register_vector

PG_URL = os.environ.get("TOINOI_PG_URL", "postgresql://toinoi:toinoi@localhost:54329/toinoi")
SCHEMA = "lab51"
COMPOSE = "docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait"
TABLE = f"{SCHEMA}.driver_faces"

TOPK_SQL = f"SELECT driver_id, embedding <=> %s AS dist FROM {TABLE} ORDER BY embedding <=> %s LIMIT %s"


def driver_id(n: int) -> str:
    """Mã tài khoản. Lab cần tới vài trăm nghìn dòng nên dùng 6 chữ số (`DR-000001`), dài hơn `DR-001` của bài trước."""
    return f"DR-{n:06d}"


def connect() -> psycopg.Connection:
    """Nối PostgreSQL hạ tầng chung; không nối được thì in FAIL và thoát mã 1."""
    try:
        conn = psycopg.connect(PG_URL, autocommit=True, connect_timeout=5)
    except psycopg.OperationalError:
        print(f"FAIL: không nối được PostgreSQL tại {PG_URL} ({COMPOSE})")
        sys.exit(1)
    conn.execute("CREATE EXTENSION IF NOT EXISTS vector")
    register_vector(conn)
    return conn


def reset_schema(conn: psycopg.Connection, dim: int) -> None:
    """Tạo lại schema lab51 và bảng driver_faces(driver_id, person, embedding vector(dim))."""
    conn.execute(f"DROP SCHEMA IF EXISTS {SCHEMA} CASCADE")
    conn.execute(f"CREATE SCHEMA {SCHEMA}")
    conn.execute(
        f"""CREATE TABLE {TABLE} (
            driver_id text PRIMARY KEY,
            person    int,                  -- nhãn kiểm tra của lab (Olivetti); NULL với tài khoản nền
            status    text NOT NULL,        -- 'active' hoặc 'banned' (tài khoản đã bị khóa)
            embedding vector({dim}) NOT NULL
        )"""
    )


def drop_schema(conn: psycopg.Connection) -> None:
    conn.execute(f"DROP SCHEMA IF EXISTS {SCHEMA} CASCADE")


def load_rows(
    conn: psycopg.Connection, ids: list[str], persons: list[int | None], statuses: list[str], vectors: np.ndarray
) -> float:
    """Nạp bằng COPY nhị phân. Trả về số giây."""
    t0 = time.perf_counter()
    with conn.cursor() as cur, cur.copy(f"COPY {TABLE} (driver_id, person, status, embedding) FROM STDIN WITH (FORMAT BINARY)") as copy:
        copy.set_types(["text", "int4", "text", "vector"])
        for i, p, s, v in zip(ids, persons, statuses, vectors, strict=True):
            copy.write_row((i, p, s, v))
    conn.execute(f"ANALYZE {TABLE}")
    return time.perf_counter() - t0


def schema_exists(conn: psycopg.Connection) -> bool:
    row = conn.execute("SELECT EXISTS (SELECT 1 FROM pg_namespace WHERE nspname = %s)", (SCHEMA,)).fetchone()
    assert row is not None
    return bool(row[0])


def count_rows(conn: psycopg.Connection) -> int:
    row = conn.execute(f"SELECT count(*) FROM {TABLE}").fetchone()
    assert row is not None
    return int(row[0])


@contextmanager
def exact_scan(conn: psycopg.Connection) -> Iterator[None]:
    """Tắt chỉ mục để PostgreSQL quét tuần tự: đáp án chính xác của pgvector."""
    conn.execute("SET enable_indexscan = off")
    try:
        yield
    finally:
        conn.execute("RESET enable_indexscan")


def filter_sql(status: str) -> str:
    """Top-k chỉ trong các tài khoản có `status` cho trước (hằng chuỗi cố định trong code, không nhận đầu vào người dùng)."""
    return (
        f"SELECT driver_id, embedding <=> %s AS dist FROM {TABLE} WHERE status = '{status}' "
        "ORDER BY embedding <=> %s LIMIT %s"
    )


BANNED_SQL = filter_sql("banned")


def topk_banned(conn: psycopg.Connection, query: np.ndarray, k: int) -> list[tuple[str, float]]:
    """Top-k chỉ trong các tài khoản đã bị khóa: kiểm tra người đăng ký lại sau khi bị khóa."""
    return [(r[0], float(r[1])) for r in conn.execute(BANNED_SQL, (query, query, k)).fetchall()]


def topk(conn: psycopg.Connection, query: np.ndarray, k: int) -> list[tuple[str, float]]:
    return [(r[0], float(r[1])) for r in conn.execute(TOPK_SQL, (query, query, k)).fetchall()]


def plan_head(conn: psycopg.Connection, query: np.ndarray, k: int, sql: str = TOPK_SQL) -> str:
    """Dòng đầu của kế hoạch thực thi, sau các nút Limit/Sort: cho biết có dùng chỉ mục không."""
    rows = conn.execute("EXPLAIN " + sql, (query, query, k)).fetchall()
    for (line,) in rows:
        stripped = line.strip().lstrip("->").strip()
        if stripped.startswith(("Index Scan", "Seq Scan", "Parallel Seq Scan", "Bitmap Heap Scan")):
            return stripped.split("  (")[0]
    return rows[-1][0]


def prepare_index_build(conn: psycopg.Connection) -> None:
    """Chỉ mục lớn cần bộ nhớ dựng. Đặt workers song song = 0 vì /dev/shm của container Docker mặc định 64 MB:
    khi thử dựng song song với maintenance_work_mem = 512MB, PostgreSQL báo "No space left on device"."""
    conn.execute("SET maintenance_work_mem = '256MB'")
    conn.execute("SET max_parallel_maintenance_workers = 0")


def build_hnsw(conn: psycopg.Connection, m: int = 16, ef_construction: int = 64) -> tuple[float, int]:
    """Dựng HNSW cho khoảng cách cosine. Trả về (giây, byte)."""
    prepare_index_build(conn)
    t0 = time.perf_counter()
    conn.execute(
        f"CREATE INDEX driver_faces_hnsw ON {TABLE} USING hnsw (embedding vector_cosine_ops) "
        f"WITH (m = {m}, ef_construction = {ef_construction})"
    )
    return time.perf_counter() - t0, index_size(conn, "driver_faces_hnsw")


def build_ivfflat(conn: psycopg.Connection, lists: int) -> tuple[float, int]:
    """Dựng IVFFlat (chia vector vào `lists` ô). Phải dựng SAU khi đã có dữ liệu để k-means có tâm hợp lý."""
    prepare_index_build(conn)
    t0 = time.perf_counter()
    conn.execute(f"CREATE INDEX driver_faces_ivf ON {TABLE} USING ivfflat (embedding vector_cosine_ops) WITH (lists = {lists})")
    return time.perf_counter() - t0, index_size(conn, "driver_faces_ivf")


def build_partial_hnsw(conn: psycopg.Connection, status: str = "banned") -> tuple[float, int]:
    """HNSW chỉ trên các dòng có `status` cho trước (chỉ mục một phần): đồ thị nhỏ và chỉ chứa đúng tập cần tìm."""
    prepare_index_build(conn)
    t0 = time.perf_counter()
    conn.execute(
        f"CREATE INDEX driver_faces_hnsw_{status} ON {TABLE} USING hnsw (embedding vector_cosine_ops) WHERE status = '{status}'"
    )
    return time.perf_counter() - t0, index_size(conn, f"driver_faces_hnsw_{status}")


def build_status_btree(conn: psycopg.Connection) -> None:
    conn.execute(f"CREATE INDEX driver_faces_status ON {TABLE} (status)")
    conn.execute(f"ANALYZE {TABLE}")


def drop_index(conn: psycopg.Connection, name: str) -> None:
    conn.execute(f"DROP INDEX IF EXISTS {SCHEMA}.{name}")


def index_size(conn: psycopg.Connection, name: str) -> int:
    row = conn.execute("SELECT pg_relation_size(%s::regclass)", (f"{SCHEMA}.{name}",)).fetchone()
    assert row is not None
    return int(row[0])


def table_size(conn: psycopg.Connection) -> int:
    row = conn.execute("SELECT pg_relation_size(%s::regclass)", (TABLE,)).fetchone()
    assert row is not None
    return int(row[0])
