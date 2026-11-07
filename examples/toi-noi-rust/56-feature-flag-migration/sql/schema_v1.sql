-- Schema ở phiên bản 1: bốn bảng chuẩn của loạt (bài 25) cộng ba bảng của bài 56.
-- Không ghi tên schema: search_path của kết nối đã trỏ vào schema lab_<uuid>.
-- Trigger cân bút toán nạp riêng từ 02_balance_check.sql (đúng bản của bài 25).

CREATE TABLE accounts (
    id         text PRIMARY KEY,
    kind       text NOT NULL CHECK (kind IN ('asset', 'liability', 'revenue', 'expense')),
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE journal_entries (
    id              bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    idempotency_key text NOT NULL UNIQUE,
    reversal_of     bigint REFERENCES journal_entries (id),
    posted_at       timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE postings (
    entry_id    bigint NOT NULL REFERENCES journal_entries (id),
    line_no     int NOT NULL,
    account_id  text NOT NULL REFERENCES accounts (id),
    amount_dong bigint NOT NULL CHECK (amount_dong <> 0),
    PRIMARY KEY (entry_id, line_no)
);

CREATE TABLE account_balances (
    account_id   text PRIMARY KEY REFERENCES accounts (id),
    balance_dong bigint NOT NULL,
    version      bigint NOT NULL DEFAULT 0,
    updated_at   timestamptz NOT NULL DEFAULT now()
);

-- Append-only (bài 24): posting đã ghi không được sửa hay xoá.
CREATE FUNCTION postings_guard() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'sổ cái append-only: % trên postings bị cấm', TG_OP
        USING ERRCODE = 'TN001';
END;
$$;

CREATE TRIGGER postings_append_only
    BEFORE UPDATE OR DELETE ON postings
    FOR EACH ROW EXECUTE FUNCTION postings_guard();

-- Sự kiện quyết định phí của từng chuyến. Phiên bản 1 chỉ lưu tỷ lệ phí đã áp dụng.
CREATE TABLE fee_events (
    seq         bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    trip_id     text NOT NULL UNIQUE,
    fare_dong   bigint NOT NULL CHECK (fare_dong > 0),
    fee_percent int NOT NULL CHECK (fee_percent BETWEEN 0 AND 100),
    decided_at  timestamptz NOT NULL DEFAULT now()
);

-- Cờ tính năng nằm trong DB: một nguồn cho mọi instance, đổi không cần triển khai.
CREATE TABLE feature_flags (
    name            text PRIMARY KEY,
    enabled         boolean NOT NULL DEFAULT false,
    rollout_percent int NOT NULL DEFAULT 0 CHECK (rollout_percent BETWEEN 0 AND 100),
    updated_at      timestamptz NOT NULL DEFAULT now()
);

-- Checkpoint của các lần backfill: chạy lại tiếp từ chỗ dừng.
CREATE TABLE migration_checkpoints (
    name       text PRIMARY KEY,
    last_key   bigint NOT NULL DEFAULT 0,
    updated_at timestamptz NOT NULL DEFAULT now()
);

INSERT INTO feature_flags (name) VALUES ('fee-rule-v2');

INSERT INTO accounts (id, kind) VALUES
    ('customer:CU-001', 'liability'),
    ('customer:CU-002', 'liability'),
    ('customer:CU-003', 'liability'),
    ('driver:DR-001', 'liability'),
    ('driver:DR-002', 'liability'),
    ('driver:DR-003', 'liability'),
    ('platform:revenue', 'revenue'),
    ('psp:vixanh:clearing', 'asset');
