-- Schema chuẩn của loạt (bài 25), rút gọn: ba bảng sổ cái + hai bảng riêng của bài 38.
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
    line_no     int    NOT NULL,
    account_id  text   NOT NULL REFERENCES accounts (id),
    amount_dong bigint NOT NULL CHECK (amount_dong <> 0),
    PRIMARY KEY (entry_id, line_no)
);

-- Cước phải thu của một chuyến, tạo khi nhận TripCompleted (event nội bộ).
CREATE TABLE payments (
    trip_id     text   PRIMARY KEY,
    customer_id text   NOT NULL,
    driver_id   text   NOT NULL,
    fare_dong   bigint NOT NULL CHECK (fare_dong > 0),
    created_at  timestamptz NOT NULL DEFAULT now()
);

-- Mỗi webhook của VíXanh chỉ một dòng: external_ref là khóa chính nên trùng bị chặn ở đây.
CREATE TABLE psp_webhooks (
    external_ref text   PRIMARY KEY,
    trip_id      text   NOT NULL,
    amount_dong  bigint NOT NULL,
    status       text   NOT NULL
                 CHECK (status IN ('pending', 'applied', 'mismatch', 'double_capture')),
    entry_id     bigint REFERENCES journal_entries (id),
    received_at  timestamptz NOT NULL DEFAULT now(),
    applied_at   timestamptz
);
CREATE INDEX psp_webhooks_pending ON psp_webhooks (trip_id) WHERE status = 'pending';

INSERT INTO accounts (id, kind) VALUES
    ('customer:CU-001', 'liability'),
    ('driver:DR-001', 'liability'),
    ('platform:revenue', 'revenue'),
    ('psp:vixanh:clearing', 'asset');
