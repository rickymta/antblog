-- BẢN SAI, chỉ để dựng lỗi: cùng kiểm tra nhưng không khoá dòng kỳ.
create or replace function guard_period() returns trigger
language plpgsql as $$
declare
    p accounting_periods%rowtype;
begin
    select * into p from accounting_periods
     where new.posted_at >= starts_at and new.posted_at < ends_at;
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

create or replace function guard_posting_period() returns trigger
language plpgsql as $$
begin
    return new;
end;
$$;
