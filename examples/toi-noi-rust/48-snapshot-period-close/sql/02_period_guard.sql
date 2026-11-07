-- Chặn ghi vào kỳ đã đóng. FOR SHARE khoá dòng kỳ cho tới khi transaction ghi kết thúc,
-- nên việc đóng kỳ (FOR UPDATE) phải đợi các lần ghi đang dở, và lần ghi sau đó thấy CLOSED.
create or replace function guard_period() returns trigger
language plpgsql as $$
declare
    p accounting_periods%rowtype;
begin
    select * into p from accounting_periods
     where new.posted_at >= starts_at and new.posted_at < ends_at
       for share;
    if not found then
        raise exception 'chưa có kỳ kế toán chứa posted_at = %', new.posted_at
            using errcode = 'restrict_violation';
    end if;
    if p.status = 'CLOSED' then
        raise exception 'kỳ % đã đóng: không ghi posted_at = % vào kỳ này', p.label, new.posted_at
            using errcode = 'restrict_violation';
    end if;
    return new;
end;
$$;
drop trigger if exists journal_entries_guard_period on journal_entries;
create trigger journal_entries_guard_period before insert on journal_entries
    for each row execute function guard_period();

-- Bút toán đã có thì không thêm được posting vào kỳ đã đóng: nếu không, chèn thẳng vào postings
-- của một bút toán cũ sẽ đổi số dư kỳ đã công bố mà trigger trên journal_entries không thấy.
create or replace function guard_posting_period() returns trigger
language plpgsql as $$
declare
    p accounting_periods%rowtype;
    at_time timestamptz;
begin
    select e.posted_at into at_time from journal_entries e where e.id = new.entry_id;
    select * into p from accounting_periods
     where at_time >= starts_at and at_time < ends_at
       for share;
    if found and p.status = 'CLOSED' then
        raise exception 'kỳ % đã đóng: không thêm posting vào bút toán %', p.label, new.entry_id
            using errcode = 'restrict_violation';
    end if;
    return new;
end;
$$;
drop trigger if exists postings_guard_period on postings;
create trigger postings_guard_period before insert on postings
    for each row execute function guard_posting_period();
