-- Chuẩn bị database cho Debezium (chạy bằng writer endpoint của Aurora, role có quyền rds_superuser).
-- Điều kiện: cluster parameter group có rds.logical_replication = 1 và instance đã reboot (bài 31).
-- Dùng: psql "host=<writer> dbname=tripsdb user=toinoi_admin sslmode=require" -v dbz_password="$DBZ_PASSWORD" -f sql/outbox.sql

-- Bảng outbox theo thiết kế ở tài liệu kiến trúc: ghi cùng transaction với thay đổi nghiệp vụ.
CREATE TABLE IF NOT EXISTS outbox (
    event_id          uuid        PRIMARY KEY,
    aggregate_type    text        NOT NULL,           -- booking, dispatch, trip... quyết định topic đích
    aggregate_id      text        NOT NULL,           -- tripId: khoá partition trong Kafka
    event_type        text        NOT NULL,
    aggregate_version integer     NOT NULL,
    occurred_at       timestamptz NOT NULL DEFAULT now(),
    payload           jsonb       NOT NULL
);

-- User riêng cho Debezium: chỉ đọc bảng outbox và có quyền sao chép logical.
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'debezium') THEN
        CREATE ROLE debezium WITH LOGIN;
    END IF;
END
$$;
ALTER ROLE debezium WITH PASSWORD :'dbz_password';
GRANT rds_replication TO debezium;
GRANT USAGE ON SCHEMA public TO debezium;
GRANT SELECT ON outbox TO debezium;

-- Publication tạo sẵn (publication.autocreate.mode = disabled trong connector).
-- Nếu publication đã có thì lệnh này báo lỗi; bỏ qua.
CREATE PUBLICATION toinoi_outbox_pub FOR TABLE outbox;

-- Ví dụ ghi một sự kiện cùng transaction với thay đổi chuyến (rút gọn: bảng trips bỏ qua ở đây).
BEGIN;
INSERT INTO outbox (event_id, aggregate_type, aggregate_id, event_type, aggregate_version, payload)
VALUES (gen_random_uuid(), 'booking', 't9001', 'BookingCreated', 1,
        '{"tripId":"t9001","city":"hcm","service":"ride"}');
COMMIT;

-- Theo dõi slot: slot không có consumer giữ WAL lại và làm đầy đĩa.
SELECT slot_name, active, pg_size_pretty(pg_wal_lsn_diff(pg_current_wal_lsn(), restart_lsn)) AS giu_lai
FROM pg_replication_slots;
