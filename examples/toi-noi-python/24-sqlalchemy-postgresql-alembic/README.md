# Lab 24 · SQLAlchemy 2 và psycopg 3 với PostgreSQL: mapping, migration Alembic

Đi kèm bài `python-sqlalchemy-psycopg-postgresql-mapping-migration`.

- `models.py`: mapping ORM 2.0 (`DeclarativeBase`, `Mapped[]`, `mapped_column`) cho `customers` và `trips`, `StrEnum` map thành VARCHAR, schema `lab24`.
- `settings.py`: URL lấy từ `TOINOI_PG_URL` (mặc định `postgresql://toinoi:toinoi@localhost:54329/toinoi`), đổi sang `postgresql+psycopg://` cho SQLAlchemy.
- `alembic.ini`, `migrations/`: Alembic với hai revision (`0001` tạo bảng, `0002` thêm index), sinh ban đầu bằng `--autogenerate` rồi review.
- `main.py`: dựng schema bằng `alembic upgrade head`, thêm dữ liệu giả, đọc bằng `select()`, đếm câu SQL (lazy load và `selectinload`), selectinload chia lô 500 khóa, `native_enum=False` không có CHECK, ràng buộc khóa ngoại, engine async, downgrade/upgrade. Cuối cùng `DROP SCHEMA lab24 CASCADE`.

Cần hạ tầng dùng chung (PostgreSQL), chạy từ thư mục gốc repository:

```bash
docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait
uv run --project examples/toi-noi-python/24-sqlalchemy-postgresql-alembic examples/toi-noi-python/24-sqlalchemy-postgresql-alembic/main.py
uv run --project examples/toi-noi-python/24-sqlalchemy-postgresql-alembic examples/toi-noi-python/24-sqlalchemy-postgresql-alembic/main.py --verify
```

Dùng Alembic bằng tay (schema `lab24` được tạo bởi `migrations/env.py`):

```bash
LAB=examples/toi-noi-python/24-sqlalchemy-postgresql-alembic
uv run --project $LAB alembic -c $LAB/alembic.ini upgrade head
uv run --project $LAB alembic -c $LAB/alembic.ini current
uv run --project $LAB alembic -c $LAB/alembic.ini check
uv run --project $LAB alembic -c $LAB/alembic.ini downgrade -1
uv run --project $LAB $LAB/main.py --cleanup   # DROP SCHEMA lab24 CASCADE
```

Kết quả kỳ vọng: `--verify` kết thúc bằng `OK 27/27` (exit 0). Khi không nối được PostgreSQL, lab in `FAIL: không nối được PostgreSQL tại <url> ...` và exit 1. Lab chỉ chạm schema `lab24`, không động tới schema khác. `--keep` giữ schema sau khi chạy.

Phiên bản đã chạy (xem `uv.lock`): SQLAlchemy 2.1.8, psycopg 3.3.8, Alembic 1.20.1, greenlet 3.5.6 (do `sqlalchemy[asyncio]`), Python 3.14.7. PostgreSQL trong compose là 17.
