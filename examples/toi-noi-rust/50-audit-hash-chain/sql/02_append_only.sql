-- Lớp 2: trigger từ chối UPDATE, DELETE và TRUNCATE trên các bảng chỉ-thêm.
create function reject_mutation() returns trigger
language plpgsql as $$
begin
    raise exception 'append-only: % trên %.% bị cấm', tg_op, tg_table_schema, tg_table_name
        using errcode = 'restrict_violation';
end;
$$;

create trigger journal_entries_append_only before update or delete on journal_entries
    for each row execute function reject_mutation();
create trigger postings_append_only before update or delete on postings
    for each row execute function reject_mutation();
create trigger entry_chain_append_only before update or delete on entry_chain
    for each row execute function reject_mutation();
create trigger audit_log_append_only before update or delete on audit_log
    for each row execute function reject_mutation();

create trigger journal_entries_no_truncate before truncate on journal_entries
    for each statement execute function reject_mutation();
create trigger postings_no_truncate before truncate on postings
    for each statement execute function reject_mutation();
create trigger entry_chain_no_truncate before truncate on entry_chain
    for each statement execute function reject_mutation();
create trigger audit_log_no_truncate before truncate on audit_log
    for each statement execute function reject_mutation();
