-- Thí nghiệm PG-024 (ba phiên): "CLUSTER giữ khoá ACCESS EXCLUSIVE — không ai đọc
-- được bảng, kể cả SELECT". Chứng minh bằng ba phiên trên bảng bai_cu.trips:
--   A mở giao dịch và đọc bảng (giữ ACCESS SHARE tới khi COMMIT);
--   B chạy CLUSTER ⇒ phải chờ A (cần ACCESS EXCLUSIVE);
--   C chỉ SELECT ⇒ CŨNG bị chặn — xếp hàng SAU yêu cầu khoá của B dù tương thích với A.
-- Đó là bẫy "hàng đợi khoá": một lệnh DDL đang chờ chặn luôn mọi truy vấn đến sau.
-- Mỗi phiên là một kết nối riêng; lệnh dùng tên đầy đủ schema.bảng vì search_path chỉ
-- sống trong phiên đặt nó. Định dạng nhiều phiên: README gốc của lab (mục P03+).

-- @A
BEGIN;
-- expect-value: 20000
SELECT count(*) FROM bai_cu.trips;

-- @B
-- expect-blocked
CLUSTER bai_cu.trips USING trips_pkey;

-- @C
-- expect-blocked
SELECT count(*) FROM bai_cu.trips;

-- Phiên quan sát D: ai giữ khoá gì trên bảng, ai đang chờ.
-- @D
-- expect-line: lab-A | AccessShareLock     | t
-- expect-line: lab-B | AccessExclusiveLock | f
-- expect-line: lab-C | AccessShareLock     | f
SELECT a.application_name, l.mode, l.granted
FROM pg_locks AS l JOIN pg_stat_activity AS a ON a.pid = l.pid
WHERE l.relation = 'bai_cu.trips'::regclass AND a.application_name IN ('lab-A', 'lab-B', 'lab-C')
ORDER BY a.application_name;

-- A kết thúc giao dịch ⇒ B được chạy CLUSTER, xong thì C mới đọc được (runner in kết
-- quả của B và C ngay sau bước này, theo thứ tự chúng chạy xong).
-- @A
COMMIT;

-- Sau CLUSTER: heap xếp theo id.
-- @D
-- expect-line: (0,1) | t
SELECT ctid, id = (SELECT id FROM bai_cu.trips ORDER BY id LIMIT 1) AS la_id_nho_nhat
FROM bai_cu.trips LIMIT 1;
