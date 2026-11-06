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
