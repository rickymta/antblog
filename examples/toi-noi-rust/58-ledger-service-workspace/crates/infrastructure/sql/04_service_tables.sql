-- Bảng riêng của ledger-service, ngoài bốn bảng chuẩn.

-- Hold KHÔNG phải bút toán (bài 42): không có posting, không đổi balance_dong.
--   available = balance_dong - SUM(amount_dong) của hold ACTIVE chưa hết hạn.
CREATE TABLE holds (
    id              bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    account_id      text NOT NULL REFERENCES accounts (id),
    amount_dong     bigint NOT NULL CHECK (amount_dong > 0),
    captured_dong   bigint NOT NULL DEFAULT 0,
    status          text NOT NULL CHECK (status IN ('ACTIVE', 'CAPTURED', 'VOIDED', 'EXPIRED')),
    idempotency_key text NOT NULL UNIQUE,
    expires_at      timestamptz NOT NULL,
    created_at      timestamptz NOT NULL DEFAULT now(),
    CHECK (captured_dong >= 0 AND captured_dong <= amount_dong)
);
CREATE INDEX holds_active_by_account ON holds (account_id) WHERE status = 'ACTIVE';

-- Thanh toán của một chuyến: nguồn để kiểm "tổng hoàn không vượt số đã thu" (bài 43).
-- CHECK là chốt chặn cuối khi khóa dòng bị bỏ sót; trạng thái dùng đúng tên chuẩn của loạt.
CREATE TABLE payments (
    trip_id        text PRIMARY KEY,
    customer_id    text NOT NULL,
    driver_id      text NOT NULL,
    captured_dong  bigint NOT NULL CHECK (captured_dong > 0),
    fee_dong       bigint NOT NULL CHECK (fee_dong >= 0 AND fee_dong <= captured_dong),
    refunded_dong  bigint NOT NULL DEFAULT 0,
    status         text NOT NULL CHECK (status IN
        ('INITIATED', 'AUTHORIZED', 'CAPTURED', 'FAILED', 'REFUND_PENDING', 'REFUNDED')),
    created_at     timestamptz NOT NULL DEFAULT now(),
    CHECK (refunded_dong >= 0 AND refunded_dong <= captured_dong)
);

-- Outbox: sự kiện cần phát, ghi cùng transaction với bút toán (bài 37).
CREATE TABLE outbox (
    id           bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    topic        text NOT NULL,
    key          text NOT NULL,
    payload      jsonb NOT NULL,
    created_at   timestamptz NOT NULL DEFAULT now(),
    published_at timestamptz
);
CREATE INDEX outbox_unpublished ON outbox (id) WHERE published_at IS NULL;

-- Inbox: sự kiện đã xử lý. Khoá chính là rào chặn xử lý hai lần.
CREATE TABLE inbox (
    event_id     text PRIMARY KEY,
    processed_at timestamptz NOT NULL DEFAULT now()
);
