-- Bảng của bài 67, ngoài bốn bảng chuẩn. Không ghi tên schema: search_path của kết nối trỏ vào lab_<uuid>.

-- Outbox: ý định phát LedgerEntryPosted, ghi cùng transaction với bút toán. published_at NULL = chưa gửi.
CREATE TABLE outbox (
    id           bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    topic        text NOT NULL,
    key          text NOT NULL,
    payload      jsonb NOT NULL,
    created_at   timestamptz NOT NULL,
    published_at timestamptz
);
CREATE INDEX outbox_unpublished ON outbox (id) WHERE published_at IS NULL;

-- Inbox: sự kiện đã xử lý xong. Khoá chính là chốt chặn xử lý hai lần.
CREATE TABLE inbox (
    event_id     text PRIMARY KEY,
    processed_at timestamptz NOT NULL
);

-- Bút toán thu tiền ứng với mã giao dịch VíXanh nào (như bài 47). Một mã có thể có nhiều bút toán.
CREATE TABLE psp_refs (
    entry_id     bigint PRIMARY KEY REFERENCES journal_entries (id),
    external_ref text NOT NULL
);
CREATE INDEX psp_refs_external_ref ON psp_refs (external_ref);

-- Dòng sao kê VíXanh đã nạp (bản rút gọn của bảng tạm ở bài 47): mã giao dịch và số tiền.
CREATE TABLE psp_statement (
    external_ref text   NOT NULL,
    amount_dong  bigint NOT NULL CHECK (amount_dong > 0)
);

-- Tham chiếu của bút toán điều chỉnh: sửa cái gì, của kỳ nào, vì sao. Bút toán gốc không bị đụng tới.
CREATE TABLE recon_adjustments (
    entry_id        bigint PRIMARY KEY REFERENCES journal_entries (id),
    external_ref    text NOT NULL,
    original_period text NOT NULL,
    reason          text NOT NULL
);

-- Kỳ kế toán nửa mở [starts_at, ends_at) như bài 48 (bản rút gọn: không có snapshot).
CREATE TABLE accounting_periods (
    label     text PRIMARY KEY,
    starts_at timestamptz NOT NULL,
    ends_at   timestamptz NOT NULL,
    status    text NOT NULL DEFAULT 'OPEN' CHECK (status IN ('OPEN', 'CLOSED')),
    CHECK (starts_at < ends_at)
);
