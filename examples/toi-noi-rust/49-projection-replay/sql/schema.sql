-- Schema chuẩn của loạt (bài 25), thêm hai thứ cho bài 49:
--   journal_entries.tx_id        : xid của transaction đã INSERT dòng này;
--   projection_checkpoints       : vị trí đã xử lý của từng projection.
CREATE TABLE accounts (
    id         text PRIMARY KEY,
    kind       text NOT NULL CHECK (kind IN ('asset', 'liability', 'revenue', 'expense')),
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE journal_entries (
    id              bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    idempotency_key text NOT NULL UNIQUE,
    reversal_of     bigint REFERENCES journal_entries (id),
    posted_at       timestamptz NOT NULL DEFAULT now(),
    -- xid8 của transaction ghi dòng này, lưu dạng bigint (xid8 -> text -> bigint).
    tx_id           bigint NOT NULL DEFAULT (pg_current_xact_id()::text::bigint)
);

-- Con trỏ đọc log theo thứ tự (tx_id, id).
CREATE INDEX journal_entries_tx_id_id ON journal_entries (tx_id, id);

CREATE TABLE postings (
    entry_id    bigint NOT NULL REFERENCES journal_entries (id),
    line_no     int    NOT NULL,
    account_id  text   NOT NULL REFERENCES accounts (id),
    amount_dong bigint NOT NULL CHECK (amount_dong <> 0),
    PRIMARY KEY (entry_id, line_no)
);

-- Append-only (bài 24): log không bao giờ bị sửa, nên replay luôn ra cùng kết quả.
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

-- Mỗi projection có một dòng cho mỗi phiên bản: (name, version).
-- `active` = phiên bản đang phục vụ đọc; phiên bản đang dựng lại có active = false.
CREATE TABLE projection_checkpoints (
    name          text    NOT NULL,
    version       int     NOT NULL,
    last_tx_id    bigint  NOT NULL DEFAULT 0,
    last_entry_id bigint  NOT NULL DEFAULT 0,
    active        boolean NOT NULL DEFAULT false,
    PRIMARY KEY (name, version)
);
-- Mỗi projection chỉ có tối đa một phiên bản active.
CREATE UNIQUE INDEX projection_checkpoints_one_active
    ON projection_checkpoints (name) WHERE active;

-- Tài khoản mẫu của Tới Nơi.
INSERT INTO accounts (id, kind) VALUES
    ('customer:CU-001', 'liability'),
    ('customer:CU-002', 'liability'),
    ('customer:CU-003', 'liability'),
    ('driver:DR-001', 'liability'),
    ('driver:DR-002', 'liability'),
    ('platform:revenue', 'revenue'),
    ('psp:vixanh:clearing', 'asset');
