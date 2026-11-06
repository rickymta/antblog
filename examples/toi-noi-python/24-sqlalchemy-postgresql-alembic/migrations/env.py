"""Môi trường chạy migration: nối Alembic với metadata của models.py và PostgreSQL của lab."""

from logging.config import fileConfig

from alembic import context
from sqlalchemy import create_engine, pool, text

from models import Base
from settings import PG_URL, SCHEMA, sqlalchemy_url

config = context.config
# main.py gọi Alembic bằng API và tắt cấu hình logging của file ini để output gọn.
if config.config_file_name is not None and config.attributes.get("configure_logger", True):
    fileConfig(config.config_file_name, disable_existing_loggers=False)

# Metadata là thứ `--autogenerate` so với database thật để đề xuất migration.
target_metadata = Base.metadata


def include_name(name: str | None, type_: str, parent_names: dict) -> bool:
    """Chỉ xét schema lab24; bỏ qua mọi schema khác trong database dùng chung."""
    if type_ == "schema":
        return name == SCHEMA
    return True


def _configure(**kwargs) -> None:
    context.configure(
        target_metadata=target_metadata,
        include_schemas=True,
        include_name=include_name,
        version_table_schema=SCHEMA,  # bảng alembic_version cũng nằm trong lab24
        **kwargs,
    )


def run_migrations_offline() -> None:
    """Chế độ offline: chỉ in SQL ra stdout (`alembic upgrade head --sql`), không kết nối."""
    _configure(url=sqlalchemy_url(PG_URL), literal_binds=True)
    with context.begin_transaction():
        context.run_migrations()


def run_migrations_online() -> None:
    engine = create_engine(sqlalchemy_url(PG_URL), poolclass=pool.NullPool)
    with engine.connect() as connection:
        # Schema phải có trước khi tạo alembic_version trong đó.
        connection.execute(text(f'CREATE SCHEMA IF NOT EXISTS "{SCHEMA}"'))
        connection.commit()
        _configure(connection=connection)
        with context.begin_transaction():
            context.run_migrations()


if context.is_offline_mode():
    run_migrations_offline()
else:
    run_migrations_online()
