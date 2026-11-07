-- Schema chuẩn của loạt (bài 25) cộng ba thứ của bài 48: recorded_at, balance_snapshots, accounting_periods.
create table accounts (
    id         text primary key,
    kind       text not null check (kind in ('asset', 'liability', 'revenue', 'expense')),
    created_at timestamptz not null default now()
);

create table journal_entries (
    id              bigint generated always as identity primary key,
    idempotency_key text not null unique,
    reversal_of     bigint references journal_entries (id),
    -- Thời điểm kế toán của bút toán: quyết định nó thuộc kỳ nào. Bài 48 để nghiệp vụ truyền vào.
    posted_at       timestamptz not null default now(),
    -- Thời điểm hệ thống ghi. Chỉ để kiểm toán, không dùng để chia kỳ.
    recorded_at     timestamptz not null default now()
);
-- Truy vấn "các bút toán trong [from, to)" đi theo index này.
create index journal_entries_posted_at_idx on journal_entries (posted_at);

create table postings (
    entry_id     bigint not null references journal_entries (id),
    line_no      int not null,
    account_id   text not null references accounts (id),
    amount_dong  bigint not null check (amount_dong <> 0),
    primary key (entry_id, line_no)
);
create index postings_account_entry_idx on postings (account_id, entry_id);

-- Số dư của account_id ngay TRƯỚC as_of: tổng mọi posting có posted_at < as_of (nửa mở).
create table balance_snapshots (
    account_id   text not null references accounts (id),
    as_of        timestamptz not null,
    balance_dong bigint not null,
    primary key (account_id, as_of)
);

-- Kỳ kế toán nửa mở [starts_at, ends_at). EXCLUDE cấm hai kỳ chồng nhau.
create table accounting_periods (
    id         bigint generated always as identity primary key,
    label      text not null unique,
    starts_at  timestamptz not null,
    ends_at    timestamptz not null,
    status     text not null default 'OPEN' check (status in ('OPEN', 'CLOSED')),
    closed_at  timestamptz,
    check (starts_at < ends_at),
    check ((status = 'CLOSED') = (closed_at is not null)),
    exclude using gist (tstzrange(starts_at, ends_at) with &&)
);

-- Sổ cái append-only (bài 24): sửa sai bằng bút toán mới, không sửa dòng cũ.
create function reject_ledger_mutation() returns trigger
language plpgsql as $$
begin
    raise exception 'sổ cái append-only: % trên % bị cấm', tg_op, tg_table_name
        using errcode = 'restrict_violation';
end;
$$;
create trigger journal_entries_append_only before update or delete on journal_entries
    for each row execute function reject_ledger_mutation();
create trigger postings_append_only before update or delete on postings
    for each row execute function reject_ledger_mutation();

-- Đã đóng thì đóng luôn: không mở lại, không đổi mốc.
create function reject_reopen() returns trigger
language plpgsql as $$
begin
    if old.status = 'CLOSED' then
        raise exception 'kỳ % đã đóng, không sửa được', old.label
            using errcode = 'restrict_violation';
    end if;
    return new;
end;
$$;
create trigger periods_no_reopen before update or delete on accounting_periods
    for each row execute function reject_reopen();
