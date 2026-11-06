-- Schema chuẩn của loạt (bài 25 giải thích thiết kế); bài này chỉ cần ba bảng đầu và trigger.
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

-- Append-only: mọi UPDATE/DELETE theo dòng và TRUNCATE đều bị từ chối.
-- SQLSTATE 'TN001' là mã tự đặt của loạt, để Rust phân biệt với lỗi khác.
CREATE FUNCTION forbid_mutation() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'sổ cái append-only: % trên % bị cấm', TG_OP, TG_TABLE_NAME
        USING ERRCODE = 'TN001';
END;
$$;

CREATE TRIGGER journal_entries_append_only
    BEFORE UPDATE OR DELETE ON journal_entries
    FOR EACH ROW EXECUTE FUNCTION forbid_mutation();
CREATE TRIGGER postings_append_only
    BEFORE UPDATE OR DELETE ON postings
    FOR EACH ROW EXECUTE FUNCTION forbid_mutation();
CREATE TRIGGER journal_entries_no_truncate
    BEFORE TRUNCATE ON journal_entries
    FOR EACH STATEMENT EXECUTE FUNCTION forbid_mutation();
CREATE TRIGGER postings_no_truncate
    BEFORE TRUNCATE ON postings
    FOR EACH STATEMENT EXECUTE FUNCTION forbid_mutation();

-- Tài khoản mẫu của Tới Nơi (DR-002/DR-003 dành cho hai thí nghiệm hạn mức).
INSERT INTO accounts (id, kind) VALUES
    ('customer:CU-001', 'liability'),
    ('driver:DR-001', 'liability'),
    ('driver:DR-002', 'liability'),
    ('driver:DR-003', 'liability'),
    ('platform:revenue', 'revenue'),
    ('psp:vixanh:clearing', 'asset');
