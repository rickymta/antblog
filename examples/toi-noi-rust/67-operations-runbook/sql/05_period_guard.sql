-- Chặn ghi vào kỳ đã đóng (bài 48, rút gọn): bút toán có posted_at rơi vào kỳ CLOSED bị từ chối.
-- Bút toán sửa sai của kỳ đã đóng phải mang posted_at của kỳ đang mở.
CREATE FUNCTION guard_period() RETURNS trigger
LANGUAGE plpgsql AS $$
DECLARE
    p accounting_periods%ROWTYPE;
BEGIN
    SELECT * INTO p FROM accounting_periods
     WHERE NEW.posted_at >= starts_at AND NEW.posted_at < ends_at;
    IF FOUND AND p.status = 'CLOSED' THEN
        RAISE EXCEPTION 'kỳ % đã đóng: không ghi posted_at = % vào kỳ này', p.label, NEW.posted_at
            USING ERRCODE = 'restrict_violation';
    END IF;
    RETURN NEW;
END;
$$;
CREATE TRIGGER journal_entries_guard_period BEFORE INSERT ON journal_entries
    FOR EACH ROW EXECUTE FUNCTION guard_period();
