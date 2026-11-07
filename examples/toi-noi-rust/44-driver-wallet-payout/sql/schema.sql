-- Schema chuẩn của loạt (bài 25) cộng hai bảng của bài 44: payout_batches và payouts.
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

-- Số dư cập nhật cùng transaction với posting. Hàng này vừa là số dư vừa là ổ khóa của ví:
-- lô payout khóa hàng của tài xế bằng FOR UPDATE SKIP LOCKED.
CREATE TABLE account_balances (
    account_id   text PRIMARY KEY REFERENCES accounts (id),
    balance_dong bigint NOT NULL,
    version      bigint NOT NULL DEFAULT 0,
    updated_at   timestamptz NOT NULL DEFAULT now()
);

-- Một lô chi. batch_code UNIQUE: chạy lại cùng mã lô là cùng một lô.
CREATE TABLE payout_batches (
    id         bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    batch_code text NOT NULL UNIQUE,
    period     text NOT NULL,
    status     text NOT NULL CHECK (status IN ('CREATED', 'SENT', 'SETTLED', 'FAILED')),
    created_at timestamptz NOT NULL
);

-- Một khoản chi cho một tài xế. payout_key vừa là idempotency_key của bút toán
-- vừa là mã tham chiếu gửi sang ngân hàng.
CREATE TABLE payouts (
    id                bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    batch_id          bigint NOT NULL REFERENCES payout_batches (id),
    period            text NOT NULL,
    driver_id         text NOT NULL,
    amount_dong       bigint NOT NULL CHECK (amount_dong > 0),
    status            text NOT NULL CHECK (status IN ('CREATED', 'SENT', 'SETTLED', 'FAILED')),
    payout_key        text NOT NULL UNIQUE,
    entry_id          bigint NOT NULL REFERENCES journal_entries (id),
    reversal_entry_id bigint REFERENCES journal_entries (id),
    bank_ref          text,
    attempts          int NOT NULL DEFAULT 0,
    last_error        text,
    created_at        timestamptz NOT NULL,
    updated_at        timestamptz NOT NULL
);

-- Mỗi tài xế nhiều nhất một khoản chi còn hiệu lực trong một kỳ. Khoản FAILED đã được đảo
-- và trả tiền về ví nên không chiếm chỗ: kỳ sau (hoặc lô sau) được chi lại.
CREATE UNIQUE INDEX payouts_period_driver_uq ON payouts (period, driver_id) WHERE status <> 'FAILED';
-- Tổng đã rút trong ngày của một tài xế (giới hạn rút) quét theo chỉ mục này.
CREATE INDEX payouts_driver_created ON payouts (driver_id, created_at);

INSERT INTO accounts (id, kind) VALUES
    ('customer:CU-001', 'liability'),
    ('platform:revenue', 'revenue'),
    ('psp:vixanh:clearing', 'asset'),
    ('bank:payout:clearing', 'asset');
