-- Schema chuẩn của loạt (bài 25) cộng bảng `holds`. Không ghi tên schema: search_path của kết nối
-- đã trỏ vào schema lab_<uuid> của ca chạy.

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

-- Projection: tính lại được từ postings. Hàng này cũng là ổ khóa của từng tài khoản.
CREATE TABLE account_balances (
    account_id   text PRIMARY KEY REFERENCES accounts (id),
    balance_dong bigint NOT NULL,
    version      bigint NOT NULL DEFAULT 0,
    updated_at   timestamptz NOT NULL DEFAULT now()
);

-- Hold KHÔNG phải bút toán: không có posting, không đổi balance_dong. Nó chỉ làm giảm số dư khả dụng:
--   available = balance_dong - SUM(amount_dong - captured_dong) của các hold ACTIVE.
CREATE TABLE holds (
    id              bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    account_id      text NOT NULL REFERENCES accounts (id),
    amount_dong     bigint NOT NULL CHECK (amount_dong > 0),
    captured_dong   bigint NOT NULL DEFAULT 0,
    status          text NOT NULL CHECK (status IN ('ACTIVE', 'CAPTURED', 'VOIDED', 'EXPIRED')),
    idempotency_key text NOT NULL UNIQUE,
    expires_at      timestamptz NOT NULL,
    created_at      timestamptz NOT NULL DEFAULT now(),
    CHECK (captured_dong >= 0 AND captured_dong <= amount_dong)
);

-- Tính khả dụng chỉ đọc hold ACTIVE của một tài khoản; job hết hạn chỉ quét hold ACTIVE theo hạn.
CREATE INDEX holds_active_by_account ON holds (account_id) WHERE status = 'ACTIVE';
CREATE INDEX holds_active_by_expiry ON holds (expires_at) WHERE status = 'ACTIVE';

INSERT INTO accounts (id, kind) VALUES
    ('customer:CU-001', 'liability'),
    ('customer:CU-002', 'liability'),
    ('customer:CU-003', 'liability'),
    ('driver:DR-001', 'liability'),
    ('platform:revenue', 'revenue'),
    ('psp:vixanh:clearing', 'asset');

-- Mở tài khoản thì tạo luôn hàng số dư 0: `SELECT ... FOR UPDATE` trên hàng chưa có không khóa gì.
INSERT INTO account_balances (account_id, balance_dong) SELECT id, 0 FROM accounts;
