-- Chuẩn bị PostgreSQL NGUỒN (tự vận hành trên EC2) cho DMS full load + CDC. Chạy bằng superuser trên nguồn.
-- Đọc kỹ: đổi wal_level cần KHỞI ĐỘNG LẠI PostgreSQL (một lần gián đoạn ngắn, lên lịch trước).

-- 1) Kiểm tra cấu hình hiện tại.
SHOW wal_level;                -- cần 'logical'
SHOW max_replication_slots;    -- cần >= số slot DMS dùng, cộng dự phòng
SHOW max_wal_senders;          -- cần >= số kết nối replication, cộng dự phòng

-- 2) Nếu chưa đúng, sửa postgresql.conf rồi khởi động lại:
--    wal_level = logical
--    max_replication_slots = 10
--    max_wal_senders = 10
--    (tuỳ chọn) wal_sender_timeout = 0   -- tránh ngắt kết nối khi load lớn

-- 3) User cho DMS: quyền REPLICATION và SELECT. Đặt mật khẩu bằng \password hoặc secret, KHÔNG viết vào script.
CREATE ROLE dms_user WITH LOGIN REPLICATION;
-- \password dms_user
GRANT USAGE ON SCHEMA public TO dms_user;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO dms_user;

-- 4) Mọi bảng cần khoá chính, hoặc REPLICA IDENTITY FULL, để UPDATE và DELETE được nhân bản đúng.
SELECT c.relname
FROM pg_class c
JOIN pg_namespace n ON n.oid = c.relnamespace
WHERE n.nspname = 'public' AND c.relkind = 'r'
  AND NOT EXISTS (SELECT 1 FROM pg_index i WHERE i.indrelid = c.oid AND i.indisprimary);

-- 5) Theo dõi slot trong lúc migration: slot không được đọc sẽ giữ WAL và có thể làm đầy đĩa nguồn.
SELECT slot_name, plugin, active,
       pg_size_pretty(pg_wal_lsn_diff(pg_current_wal_lsn(), restart_lsn)) AS retained_wal
FROM pg_replication_slots;
