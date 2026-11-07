-- Schema chuẩn của loạt (bài 25) cộng bảng `account_limits` của bài này. Không ghi tên schema:
-- search_path của kết nối đã trỏ vào schema lab_<uuid> của ca chạy.

CREATE TABLE accounts (
    id         text PRIMARY KEY,
    kind       text NOT NULL CHECK (kind IN ('asset', 'liability', 'revenue', 'expense')),
    created_at timestamptz NOT NULL DEFAULT now()
);

-- Bút toán là aggregate bất biến: không có cột version vì không có đường UPDATE nào (xem 03_append_only.sql).
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
CREATE INDEX postings_account_entry_idx ON postings (account_id, entry_id);

-- Projection và cũng là ổ khóa của aggregate Account: `version` tăng ở MỌI lần ghi hàng này.
CREATE TABLE account_balances (
    account_id   text PRIMARY KEY REFERENCES accounts (id),
    balance_dong bigint NOT NULL,
    version      bigint NOT NULL DEFAULT 0,
    updated_at   timestamptz NOT NULL DEFAULT now()
);

-- Phần còn lại của aggregate: hạn mức rút trong kỳ và số đã rút. Chỉ được ghi bởi người đã
-- thắng version (hoặc đang giữ khóa) của hàng account_balances tương ứng.
-- CHECK bắt được giá trị vô lý (âm, vượt hạn mức) nhưng KHÔNG bắt được lost update:
-- mỗi bên ghi một con số riêng lẻ hợp lệ.
CREATE TABLE account_limits (
    account_id          text PRIMARY KEY REFERENCES accounts (id),
    withdraw_limit_dong bigint NOT NULL CHECK (withdraw_limit_dong >= 0),
    withdrawn_dong      bigint NOT NULL DEFAULT 0 CHECK (withdrawn_dong >= 0),
    CHECK (withdrawn_dong <= withdraw_limit_dong)
);

INSERT INTO accounts (id, kind) VALUES
    ('platform:revenue', 'revenue'),
    ('psp:vixanh:clearing', 'asset'),
    ('bank:payout:clearing', 'asset');
INSERT INTO account_balances (account_id, balance_dong) SELECT id, 0 FROM accounts;
