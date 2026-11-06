"""Hai khác biệt cụ thể giữa SQLite và PostgreSQL, chạy cùng một ý định trên cả hai.

Đây không phải chê SQLite (nó là database tốt cho việc của nó); nó chỉ cho thấy test
xanh trên SQLite không chứng minh gì về hành vi của PostgreSQL ở production.
"""

import sqlite3

import psycopg
import pytest
from psycopg import errors as pg_errors


def test_sqlite_default_table_accepts_text_in_a_numeric_column_but_postgresql_rejects(conn: psycopg.Connection) -> None:
    lite = sqlite3.connect(":memory:")
    lite.execute("CREATE TABLE t (distance_km NUMERIC)")
    lite.execute("INSERT INTO t VALUES ('abc')")             # bảng thường của SQLite: kiểu chỉ là «affinity»
    assert lite.execute("SELECT distance_km, typeof(distance_km) FROM t").fetchone() == ("abc", "text")

    conn.execute("CREATE TEMP TABLE t (distance_km numeric)")
    with pytest.raises(pg_errors.InvalidTextRepresentation):  # PostgreSQL: kiểu là ràng buộc thật
        conn.execute("INSERT INTO t VALUES ('abc')")


def test_sqlite_strict_tables_have_no_numeric_type() -> None:
    # STRICT (SQLite 3.37+) chặn text trong cột số, nhưng chỉ cho INT/INTEGER/REAL/TEXT/BLOB/ANY:
    # không có numeric(12,0) hay timestamptz như bảng trips của PostgreSQL.
    lite = sqlite3.connect(":memory:")
    lite.execute("CREATE TABLE ok (x REAL) STRICT")
    with pytest.raises(sqlite3.IntegrityError):
        lite.execute("INSERT INTO ok VALUES ('abc')")
    with pytest.raises(sqlite3.OperationalError, match="unknown datatype"):
        lite.execute("CREATE TABLE bad (fare NUMERIC) STRICT")


def test_sqlite_has_no_skip_locked_but_postgresql_does(conn: psycopg.Connection) -> None:
    sql = "SELECT 1 FROM t WHERE x = 1 FOR UPDATE SKIP LOCKED"

    lite = sqlite3.connect(":memory:")
    lite.execute("CREATE TABLE t (x INTEGER)")
    with pytest.raises(sqlite3.OperationalError, match="syntax error"):
        lite.execute(sql)

    conn.execute("CREATE TEMP TABLE t (x integer)")
    conn.execute("BEGIN")
    assert conn.execute(sql).fetchall() == []                # PostgreSQL chạy được
    conn.execute("ROLLBACK")
