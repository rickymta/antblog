-- Schema chuẩn của loạt (bài 25), rút gọn, cộng bốn bảng riêng của bài 47.
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
-- Đối soát lọc bút toán theo cửa sổ thời gian nên cần chỉ mục này.
CREATE INDEX journal_entries_posted_at ON journal_entries (posted_at);

CREATE TABLE postings (
    entry_id    bigint NOT NULL REFERENCES journal_entries (id),
    line_no     int    NOT NULL,
    account_id  text   NOT NULL REFERENCES accounts (id),
    amount_dong bigint NOT NULL CHECK (amount_dong <> 0),
    PRIMARY KEY (entry_id, line_no)
);

-- Bút toán thu tiền nào ứng với mã giao dịch VíXanh nào. Một external_ref có thể có nhiều
-- bút toán (ta ghi tách), nên external_ref KHÔNG unique ở đây.
CREATE TABLE psp_refs (
    entry_id     bigint PRIMARY KEY REFERENCES journal_entries (id),
    external_ref text   NOT NULL
);
CREATE INDEX psp_refs_external_ref ON psp_refs (external_ref);

-- Một lần đối soát = một file sao kê cho một kỳ. Cùng file, cùng kỳ chạy lại không tạo run mới;
-- cùng file đối soát kỳ khác là run khác (hash một mình sẽ trả nhầm kết quả của kỳ kia).
CREATE TABLE reconciliation_runs (
    id                 bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    provider           text        NOT NULL,
    file_name          text        NOT NULL,
    file_sha256        text        NOT NULL CHECK (file_sha256 ~ '^[0-9a-f]{64}$'),
    period_start       timestamptz NOT NULL,
    period_end         timestamptz NOT NULL CHECK (period_end > period_start),
    statement_rows     bigint      NOT NULL DEFAULT 0,
    bad_lines          bigint      NOT NULL DEFAULT 0,
    ledger_rows        bigint      NOT NULL DEFAULT 0,
    refs_matched       bigint      NOT NULL DEFAULT 0,
    refs_matched_split bigint      NOT NULL DEFAULT 0,
    refs_out_of_period bigint      NOT NULL DEFAULT 0,
    run_at             timestamptz NOT NULL,
    UNIQUE (file_sha256, period_start, period_end)
);

-- Chỉ lưu NGOẠI LỆ. Mỗi dòng là một việc mở cho Finance (status OPEN); ledger không tự sửa gì.
CREATE TABLE reconciliation_items (
    id           bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    run_id       bigint NOT NULL REFERENCES reconciliation_runs (id),
    kind         text   NOT NULL CHECK (kind IN (
                     'MISSING_IN_LEDGER', 'MISSING_IN_PSP', 'AMOUNT_MISMATCH',
                     'DUPLICATE', 'DATE_SHIFT', 'BAD_LINE')),
    external_ref text,
    line_no      bigint,
    ours_dong    bigint,
    psp_dong     bigint,
    diff_dong    bigint,
    ours_rows    int,
    psp_rows     int,
    ours_at      timestamptz,
    psp_at       timestamptz,
    detail       text   NOT NULL,
    status       text   NOT NULL DEFAULT 'OPEN' CHECK (status IN ('OPEN', 'ACKNOWLEDGED', 'RESOLVED')),
    CHECK ((kind = 'BAD_LINE' AND line_no IS NOT NULL) OR (kind <> 'BAD_LINE' AND external_ref IS NOT NULL))
);
CREATE INDEX reconciliation_items_run ON reconciliation_items (run_id, kind);

INSERT INTO accounts (id, kind) VALUES
    ('driver:DR-001', 'liability'),
    ('platform:revenue', 'revenue'),
    ('psp:vixanh:clearing', 'asset');
INSERT INTO accounts (id, kind)
    SELECT 'customer:CU-' || lpad(n::text, 3, '0'), 'liability' FROM generate_series(1, 12) AS n;
