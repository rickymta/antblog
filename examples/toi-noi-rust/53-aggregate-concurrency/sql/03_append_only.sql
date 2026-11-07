-- Sổ append-only (bút toán của bài 24): mọi UPDATE/DELETE theo dòng đều bị từ chối.
-- Vì không ai sửa bút toán nên không có «hai người sửa chồng nhau», và journal_entries không cần version.
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
