-- Schema chuẩn của loạt (bài 25) cộng hai bảng mới của bài 37: outbox và inbox.
-- Không ghi tên schema: search_path của kết nối trỏ vào schema lab_<uuid> của ca chạy.
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

-- Projection số dư: consumer hạ nguồn dựng từ LedgerEntryPosted.
CREATE TABLE account_balances (
    account_id   text PRIMARY KEY REFERENCES accounts (id),
    balance_dong bigint NOT NULL,
    version      bigint NOT NULL DEFAULT 0,
    updated_at   timestamptz NOT NULL DEFAULT now()
);

-- Outbox: ý định gửi, ghi cùng transaction với bút toán. published_at NULL = chưa gửi.
CREATE TABLE outbox (
    id           bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    topic        text NOT NULL,
    key          text NOT NULL,
    payload      jsonb NOT NULL,
    created_at   timestamptz NOT NULL DEFAULT now(),
    published_at timestamptz
);
-- Index riêng cho hàng chưa gửi: relay chỉ quét phần này, không quét cả lịch sử.
CREATE INDEX outbox_unpublished ON outbox (id) WHERE published_at IS NULL;

-- Inbox: event_id đã xử lý. PRIMARY KEY là chốt chặn cuối chống xử lý hai lần.
CREATE TABLE inbox (
    event_id     text PRIMARY KEY,
    processed_at timestamptz NOT NULL DEFAULT now()
);

INSERT INTO accounts (id, kind) VALUES
    ('customer:CU-001', 'liability'),
    ('driver:DR-001', 'liability'),
    ('platform:revenue', 'revenue'),
    ('psp:vixanh:clearing', 'asset');
