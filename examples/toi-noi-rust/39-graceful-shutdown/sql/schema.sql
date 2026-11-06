-- Schema chuẩn của loạt (bài 25) cộng hai bảng của phần P04: outbox và inbox (bài 37).
-- Không ghi tên schema: search_path của kết nối đã trỏ vào schema lab_<uuid> của ca chạy.
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

-- Projection: luôn tính lại được từ postings.
CREATE TABLE account_balances (
    account_id   text PRIMARY KEY REFERENCES accounts (id),
    balance_dong bigint NOT NULL,
    version      bigint NOT NULL DEFAULT 0,
    updated_at   timestamptz NOT NULL DEFAULT now()
);

-- Outbox: sự kiện phát ra được ghi cùng transaction với bút toán, relay đọc rồi đẩy lên Kafka.
CREATE TABLE outbox (
    id           bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    topic        text NOT NULL,
    key          text NOT NULL,
    payload      jsonb NOT NULL,
    created_at   timestamptz NOT NULL DEFAULT now(),
    published_at timestamptz
);

-- Inbox: mỗi sự kiện đầu vào chỉ được xử lý một lần.
CREATE TABLE inbox (
    event_id     text PRIMARY KEY,
    processed_at timestamptz NOT NULL DEFAULT now()
);

INSERT INTO accounts (id, kind) VALUES
    ('customer:CU-001', 'liability'),
    ('driver:DR-001', 'liability'),
    ('platform:revenue', 'revenue'),
    ('psp:vixanh:clearing', 'asset');
