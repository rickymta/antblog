-- Sổ cái append-only: cấm UPDATE, DELETE và TRUNCATE trên bút toán và posting.
-- Sửa sai bằng bút toán đảo, không sửa dòng đã ghi. account_balances là projection nên vẫn được cập nhật.
create function reject_ledger_mutation() returns trigger
language plpgsql as $$
begin
    raise exception 'sổ cái append-only: % trên %.% bị cấm', tg_op, tg_table_schema, tg_table_name
        using errcode = 'restrict_violation';
end;
$$;

create trigger journal_entries_append_only
    before update or delete on journal_entries
    for each row execute function reject_ledger_mutation();

create trigger postings_append_only
    before update or delete on postings
    for each row execute function reject_ledger_mutation();

create trigger postings_no_truncate
    before truncate on postings
    for each statement execute function reject_ledger_mutation();
