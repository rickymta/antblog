-- Schema của ledger-api: bốn bảng chuẩn của loạt (bài 25), trigger append-only (bài 24),
-- kiểm cân lúc commit (bài 25) và hai bảng của P04: outbox, inbox (bài 37).
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

-- Inbox: sự kiện đã xử lý. Khoá chính là rào chặn xử lý hai lần.
CREATE TABLE inbox (
    event_id     text PRIMARY KEY,
    processed_at timestamptz NOT NULL DEFAULT now()
);

-- Kiểm "tổng posting của một bút toán bằng 0" ngay trong DB.
-- Constraint trigger DEFERRABLE INITIALLY DEFERRED chạy LÚC COMMIT, nên giữa chừng
-- transaction được phép chưa cân (đang chèn từng dòng); chỉ trạng thái cuối mới bị kiểm.
CREATE FUNCTION check_entry_balanced() RETURNS trigger
LANGUAGE plpgsql AS $$
DECLARE
    v_entries bigint[];
    v_entry   bigint;
    v_count   bigint;
    v_sum     numeric;
BEGIN
    IF TG_TABLE_NAME = 'journal_entries' THEN
        v_entries := ARRAY[NEW.id];
    ELSIF TG_OP = 'INSERT' THEN
        v_entries := ARRAY[NEW.entry_id];
    ELSIF TG_OP = 'DELETE' THEN
        v_entries := ARRAY[OLD.entry_id];
    ELSE
        -- UPDATE có thể chuyển posting sang bút toán khác: kiểm cả bút toán cũ lẫn mới.
        v_entries := ARRAY[OLD.entry_id, NEW.entry_id];
    END IF;

    FOREACH v_entry IN ARRAY v_entries LOOP
        -- Bút toán đã bị xoá trong cùng transaction thì không còn gì để kiểm.
        CONTINUE WHEN NOT EXISTS (SELECT 1 FROM journal_entries WHERE id = v_entry);

        SELECT count(*), COALESCE(sum(amount_dong), 0)
          INTO v_count, v_sum
          FROM postings
         WHERE entry_id = v_entry;

        IF v_count < 2 THEN
            RAISE EXCEPTION 'bút toán % có % posting, cần ít nhất 2', v_entry, v_count
                USING ERRCODE = 'check_violation';
        END IF;
        IF v_sum <> 0 THEN
            RAISE EXCEPTION 'bút toán % lệch: tổng posting = %', v_entry, v_sum
                USING ERRCODE = 'check_violation';
        END IF;
    END LOOP;
    RETURN NULL;
END;
$$;

CREATE CONSTRAINT TRIGGER postings_balanced
    AFTER INSERT OR UPDATE OR DELETE ON postings
    DEFERRABLE INITIALLY DEFERRED
    FOR EACH ROW EXECUTE FUNCTION check_entry_balanced();

-- Bút toán không có posting nào không kích hoạt trigger của postings, nên thêm một trigger nữa.
CREATE CONSTRAINT TRIGGER entries_have_postings
    AFTER INSERT ON journal_entries
    DEFERRABLE INITIALLY DEFERRED
    FOR EACH ROW EXECUTE FUNCTION check_entry_balanced();
