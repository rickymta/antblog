-- Bài 52: bảng outbox (cùng hình với bài 37) và bốn tài khoản của chuyến TN-0001.
-- Schema chuẩn và trigger cân bút toán được nạp từ lab 25 (01_schema.sql, 02_balance_check.sql).
CREATE TABLE outbox (
    id           bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    topic        text NOT NULL,
    key          text NOT NULL,
    payload      jsonb NOT NULL,
    created_at   timestamptz NOT NULL DEFAULT now(),
    published_at timestamptz
);

INSERT INTO accounts (id, kind) VALUES
    ('customer:CU-001', 'liability'),
    ('driver:DR-001', 'liability'),
    ('platform:revenue', 'revenue'),
    ('psp:vixanh:clearing', 'asset');
