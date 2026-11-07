-- Schema của lab 54: bốn bảng chuẩn của loạt (bài 25), trigger append-only (bài 24), outbox (bài 37)
-- và hai bảng của use case: trip_payments (aggregate có version, bài 53) và psp_refs (đối soát, bài 47).
-- Trigger kiểm cân nằm ở 02_balance_check.sql, chép nguyên văn từ lab 25.
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
    line_no     int    NOT NULL,
    account_id  text   NOT NULL REFERENCES accounts (id),
    amount_dong bigint NOT NULL CHECK (amount_dong <> 0),
    PRIMARY KEY (entry_id, line_no)
);

-- Append-only: mọi UPDATE/DELETE theo dòng và TRUNCATE đều bị từ chối.
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

-- Projection: luôn tính lại được từ postings, cập nhật trong CÙNG transaction với bút toán.
CREATE TABLE account_balances (
    account_id   text PRIMARY KEY REFERENCES accounts (id),
    balance_dong bigint NOT NULL,
    version      bigint NOT NULL DEFAULT 0,
    updated_at   timestamptz NOT NULL DEFAULT now()
);

-- Outbox: sự kiện cần phát, ghi cùng transaction với bút toán; relay đọc rồi đánh dấu published_at.
CREATE TABLE outbox (
    id           bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    topic        text NOT NULL,
    key          text NOT NULL,
    payload      jsonb NOT NULL,
    created_at   timestamptz NOT NULL DEFAULT now(),
    published_at timestamptz
);
CREATE INDEX outbox_unpublished ON outbox (id) WHERE published_at IS NULL;

-- Aggregate TripPayment: một dòng mỗi chuyến, `version` là khóa lạc quan. Không có bảng inbox:
-- aggregate nhớ dữ kiện nào đã nhận nên tự khử trùng.
CREATE TABLE trip_payments (
    trip_id    text PRIMARY KEY,
    version    bigint NOT NULL CHECK (version >= 1),
    state      jsonb NOT NULL,
    updated_at timestamptz NOT NULL DEFAULT now()
);

-- Mã giao dịch VíXanh của từng bút toán capture: chỉ dòng clearing của bút toán này có mặt ở sao kê.
CREATE TABLE psp_refs (
    entry_id     bigint PRIMARY KEY REFERENCES journal_entries (id),
    external_ref text NOT NULL
);
