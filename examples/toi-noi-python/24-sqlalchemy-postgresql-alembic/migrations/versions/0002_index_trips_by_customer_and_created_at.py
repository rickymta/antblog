"""index trips by customer and created_at

Revision ID: 0002
Revises: 0001
Create Date: 2026-11-06

Sinh bằng `alembic revision --autogenerate` sau khi thêm `Index(...)` vào models.py.
"""

from collections.abc import Sequence

from alembic import op

revision: str = "0002"
down_revision: str | Sequence[str] | None = "0001"
branch_labels: str | Sequence[str] | None = None
depends_on: str | Sequence[str] | None = None


def upgrade() -> None:
    op.create_index(
        "ix_trips_customer_created",
        "trips",
        ["customer_id", "created_at"],
        unique=False,
        schema="lab24",
    )


def downgrade() -> None:
    op.drop_index("ix_trips_customer_created", table_name="trips", schema="lab24")
