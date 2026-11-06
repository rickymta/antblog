-- Schema chuẩn của loạt (bài 25) cộng hai bảng của bài này: entry_chain và audit_log.
-- Không ghi tên schema: bảng nằm ở schema đầu tiên trong search_path, tức lab_<uuid>.
-- Người chạy file này là role chủ bảng (owner), không phải role ứng dụng.
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
    line_no     int not null,
    account_id  text not null references accounts (id),
    amount_dong bigint not null check (amount_dong <> 0),
    primary key (entry_id, line_no)
);

-- Hash chain: mỗi bút toán đúng một dòng. seq tăng đều từ 1; prev_hash là entry_hash của dòng trước
-- (32 byte 0 cho dòng đầu). UNIQUE (prev_hash) là lớp chặn cuối: hai dòng không thể cùng nối vào một đầu.
create table entry_chain (
    seq        bigint primary key check (seq > 0),
    entry_id   bigint not null unique references journal_entries (id),
    prev_hash  bytea not null unique check (length(prev_hash) = 32),
    entry_hash bytea not null check (length(entry_hash) = 32)
);

-- Ai làm gì, vì sao, ai duyệt. Sổ ghi chỉ giữ mã (staff:FIN-007), không giữ tên hay số điện thoại.
create table audit_log (
    id          bigint generated always as identity primary key,
    at          timestamptz not null default now(),
    actor_id    text not null,
    actor_role  text not null,
    action      text not null,
    target      text not null,
    reason_code text not null,
    approved_by text,
    request_id  text not null,
    -- maker-checker: người tạo và người duyệt phải khác nhau
    check (approved_by is null or approved_by <> actor_id)
);
