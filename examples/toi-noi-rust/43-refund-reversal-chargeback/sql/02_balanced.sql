-- Kiểm «tổng posting của một bút toán bằng 0» ngay trong DB, lúc COMMIT (xem bài 25).
CREATE FUNCTION check_entry_balanced() RETURNS trigger
LANGUAGE plpgsql AS $$
DECLARE
    v_entry bigint;
    v_count bigint;
    v_sum   numeric;
BEGIN
    IF TG_TABLE_NAME = 'journal_entries' THEN
        v_entry := NEW.id;
    ELSE
        v_entry := NEW.entry_id;
    END IF;
    SELECT count(*), COALESCE(sum(amount_dong), 0) INTO v_count, v_sum
      FROM postings WHERE entry_id = v_entry;
    IF v_count < 2 THEN
        RAISE EXCEPTION 'bút toán % có % posting, cần ít nhất 2', v_entry, v_count
            USING ERRCODE = 'check_violation';
    END IF;
    IF v_sum <> 0 THEN
        RAISE EXCEPTION 'bút toán % lệch: tổng posting = %', v_entry, v_sum
            USING ERRCODE = 'check_violation';
    END IF;
    RETURN NULL;
END;
$$;

CREATE CONSTRAINT TRIGGER postings_balanced
    AFTER INSERT ON postings
    DEFERRABLE INITIALLY DEFERRED
    FOR EACH ROW EXECUTE FUNCTION check_entry_balanced();

CREATE CONSTRAINT TRIGGER entries_have_postings
    AFTER INSERT ON journal_entries
    DEFERRABLE INITIALLY DEFERRED
    FOR EACH ROW EXECUTE FUNCTION check_entry_balanced();
