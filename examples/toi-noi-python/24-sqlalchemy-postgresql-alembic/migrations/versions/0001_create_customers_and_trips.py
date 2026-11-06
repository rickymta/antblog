"""create customers and trips

Revision ID: 0001
Revises:
Create Date: 2026-11-06

Sinh bằng `alembic revision --autogenerate`, sau đó đọc lại và gọn hóa bằng tay:
autogenerate chỉ là bản nháp, người review migration chịu trách nhiệm về nó.
"""

from collections.abc import Sequence

import sqlalchemy as sa
from alembic import op

# Định danh revision do Alembic dùng để xếp thứ tự.
revision: str = "0001"
down_revision: str | Sequence[str] | None = None
branch_labels: str | Sequence[str] | None = None
depends_on: str | Sequence[str] | None = None


def upgrade() -> None:
    op.create_table(
        "customers",
        sa.Column("customer_id", sa.String(length=16), nullable=False),
        sa.Column("full_name", sa.String(length=100), nullable=False),
        sa.PrimaryKeyConstraint("customer_id"),
        schema="lab24",
    )
    op.create_table(
        "trips",
        sa.Column("trip_id", sa.String(length=16), nullable=False),
        sa.Column("customer_id", sa.String(length=16), nullable=False),
        sa.Column("driver_id", sa.String(length=16), nullable=True),
        sa.Column(
            "service",
            sa.Enum("ride", "food", "parcel", name="servicekind", native_enum=False, length=16),
            nullable=False,
        ),
        sa.Column("distance_km", sa.Numeric(precision=7, scale=3), nullable=False),
        sa.Column("fare", sa.Numeric(precision=12, scale=0), nullable=False),
        sa.Column(
            "status",
            sa.Enum(
                "requested", "searching", "accepted", "arriving",
                "picked_up", "in_transit", "completed", "cancelled",
                name="tripstatus", native_enum=False, length=16,
            ),
            nullable=False,
        ),
        sa.Column(
            "created_at", sa.DateTime(timezone=True), server_default=sa.text("now()"), nullable=False
        ),
        sa.ForeignKeyConstraint(["customer_id"], ["lab24.customers.customer_id"]),
        sa.PrimaryKeyConstraint("trip_id"),
        schema="lab24",
    )


def downgrade() -> None:
    op.drop_table("trips", schema="lab24")
    op.drop_table("customers", schema="lab24")
