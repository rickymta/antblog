-- Schema chuẩn của loạt (bốn bảng) cộng bảng của bài 43. Không ghi tên schema:
-- search_path của kết nối đã trỏ vào schema lab_<uuid> của ca chạy.

CREATE TABLE accounts (
    id         text PRIMARY KEY,
    kind       text NOT NULL CHECK (kind IN ('asset', 'liability', 'revenue', 'expense')),
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE journal_entries (
    id              bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    idempotency_key text NOT NULL UNIQUE,
    reversal_of     bigint REFERENCES journal_entries (id),
    posted_at       timestamptz NOT NULL DEFAULT now()
);

-- Mỗi bút toán chỉ được đảo một lần, dù key khác nhau: ràng buộc nằm ở dữ liệu, không ở code gọi.
CREATE UNIQUE INDEX journal_entries_one_reversal
    ON journal_entries (reversal_of) WHERE reversal_of IS NOT NULL;

CREATE TABLE postings (
    entry_id    bigint NOT NULL REFERENCES journal_entries (id),
    line_no     int NOT NULL,
    account_id  text NOT NULL REFERENCES accounts (id),
    amount_dong bigint NOT NULL CHECK (amount_dong <> 0),
    PRIMARY KEY (entry_id, line_no)
);

-- Projection: luôn tính lại được từ postings.
CREATE TABLE account_balances (
    account_id   text PRIMARY KEY REFERENCES accounts (id),
    balance_dong bigint NOT NULL,
    version      bigint NOT NULL DEFAULT 0,
    updated_at   timestamptz NOT NULL DEFAULT now()
);

-- Reason code là dữ liệu tham chiếu: enum `ReasonCode` trong Rust phải khớp đúng bảng này.
CREATE TABLE reason_codes (
    code        text PRIMARY KEY,
    kind        text NOT NULL CHECK (kind IN ('REFUND', 'CHARGEBACK')),
    description text NOT NULL
);

INSERT INTO reason_codes (code, kind, description) VALUES
    ('TRIP_CANCELLED_BY_DRIVER', 'REFUND',     'Tài xế hủy sau khi đã thu cước'),
    ('FARE_DISPUTE',             'REFUND',     'Khách khiếu nại cước, bộ phận hỗ trợ xác nhận'),
    ('DUPLICATE_CHARGE',         'REFUND',     'Thu trùng cho cùng một chuyến'),
    ('SERVICE_NOT_RENDERED',     'REFUND',     'Chuyến không thực hiện được'),
    ('FRAUD',                    'CHARGEBACK', 'Chủ thẻ không nhận giao dịch này'),
    ('NOT_RECEIVED',             'CHARGEBACK', 'Chủ thẻ nói chưa nhận dịch vụ');

-- Một dòng cho mỗi chuyến đã thu. refunded_dong là số đã hoàn; CHECK là chốt chặn cuối cùng.
CREATE TABLE payments (
    trip_id       text PRIMARY KEY,
    customer_id   text NOT NULL REFERENCES accounts (id),
    driver_id     text NOT NULL REFERENCES accounts (id),
    external_ref  text NOT NULL UNIQUE,
    captured_dong bigint NOT NULL CHECK (captured_dong > 0),
    fee_dong      bigint NOT NULL CHECK (fee_dong >= 0 AND fee_dong <= captured_dong),
    refunded_dong bigint NOT NULL DEFAULT 0,
    refund_count  int NOT NULL DEFAULT 0,
    status        text NOT NULL CHECK (status IN
                    ('INITIATED', 'AUTHORIZED', 'CAPTURED', 'FAILED', 'REFUND_PENDING', 'REFUNDED')),
    CONSTRAINT refund_within_captured CHECK (refunded_dong >= 0 AND refunded_dong <= captured_dong)
);

-- Maker-checker: người tạo và người duyệt phải khác nhau, và trạng thái khớp với người duyệt.
CREATE TABLE refund_requests (
    id          bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    trip_id     text NOT NULL REFERENCES payments (trip_id),
    amount_dong bigint NOT NULL CHECK (amount_dong > 0),
    reason_code text NOT NULL REFERENCES reason_codes (code),
    status      text NOT NULL DEFAULT 'PENDING' CHECK (status IN ('PENDING', 'APPROVED', 'REJECTED')),
    created_by  text NOT NULL CHECK (created_by <> ''),
    decided_by  text,
    created_at  timestamptz NOT NULL DEFAULT now(),
    decided_at  timestamptz,
    entry_id    bigint REFERENCES journal_entries (id),
    CONSTRAINT checker_differs_from_maker CHECK (decided_by IS NULL OR decided_by <> created_by),
    CONSTRAINT decision_matches_status CHECK (
        (status = 'PENDING' AND decided_by IS NULL AND decided_at IS NULL AND entry_id IS NULL)
        OR (status = 'APPROVED' AND decided_by IS NOT NULL AND decided_at IS NOT NULL AND entry_id IS NOT NULL)
        OR (status = 'REJECTED' AND decided_by IS NOT NULL AND decided_at IS NOT NULL AND entry_id IS NULL)
    )
);

CREATE TABLE chargebacks (
    id           bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    psp_case_ref text NOT NULL UNIQUE,
    trip_id      text NOT NULL REFERENCES payments (trip_id),
    amount_dong  bigint NOT NULL CHECK (amount_dong > 0),
    reason_code  text NOT NULL REFERENCES reason_codes (code),
    status       text NOT NULL DEFAULT 'OPEN' CHECK (status IN ('OPEN', 'WON', 'LOST')),
    entry_id     bigint NOT NULL REFERENCES journal_entries (id),
    created_at   timestamptz NOT NULL DEFAULT now()
);

-- Audit: ai làm gì, lúc nào, lên đối tượng nào. Chỉ thêm, không sửa.
CREATE TABLE audit_log (
    id      bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    at      timestamptz NOT NULL DEFAULT now(),
    actor   text NOT NULL,
    action  text NOT NULL,
    subject text NOT NULL,
    detail  text NOT NULL DEFAULT ''
);
