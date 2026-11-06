-- Schema sổ cái rút gọn của loạt: chỉ 3 bảng bài 23 cần.
-- Không ghi tên schema: bảng nằm ở schema đầu tiên trong search_path của kết nối.

create table accounts (
    id         text primary key,
    kind       text not null check (kind in ('asset', 'liability', 'revenue', 'expense')),
    created_at timestamptz not null default now()
);

create table journal_entries (
    id              bigint generated always as identity primary key,
    idempotency_key text not null unique,
    reversal_of     bigint references journal_entries (id),
    posted_at       timestamptz not null default now()
);

create table postings (
    entry_id    bigint not null references journal_entries (id),
    line_no     int    not null,
    account_id  text   not null references accounts (id),
    amount_dong bigint not null check (amount_dong <> 0),
    primary key (entry_id, line_no)
);
