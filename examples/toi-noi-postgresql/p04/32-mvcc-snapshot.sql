-- PG-032 · MVCC và snapshot: mỗi phiên nhìn thấy gì? — `postgresql-mvcc-snapshot`
--
-- Bài toán Tới Nơi: tổng đài viên [B] mở màn chi tiết chuyến 400101 đúng lúc server [A] đang
-- sửa cước chuyến đó. B thấy cước cũ hay mới? B có phải CHỜ A không? PostgreSQL trả lời bằng
-- MVCC (multi-version concurrency control): UPDATE không ghi đè dòng mà tạo PHIÊN BẢN MỚI;
-- mỗi câu lệnh đọc theo một SNAPSHOT — "ảnh chụp" danh sách giao dịch đã/chưa commit — và
-- chỉ thấy phiên bản hợp lệ với snapshot đó. Người đọc không chặn người ghi và ngược lại.
--
-- Mỗi phiên bản dòng mang hai cột hệ thống: xmin = xid của giao dịch TẠO ra phiên bản,
-- xmax = xid của giao dịch XOÁ/THAY nó (hoặc đang khoá nó); 0 = chưa ai đụng tới.
-- xid (transaction id) tăng dần trên cả cluster nên lab in TƯƠNG ĐỐI: lab_dat_moc_xid() lấy
-- xid của chính nó làm mốc X; giao dịch ghi kế tiếp là X+1, X+2…; "<X" = commit từ trước mốc
-- (dữ liệu fixture). Hàm tiện ích: p04/00-mo-rong-schema.sql.

-- @A
SET search_path TO toinoi, public;

-- @B
SET search_path TO toinoi, public;

-- @A
SELECT lab_dat_moc_xid();

-- ===========================================================================
-- §1. Giao dịch chỉ ĐỌC không được cấp xid; xid chỉ cấp ở câu GHI đầu tiên.

-- @A
BEGIN;

-- @A
-- expect-value: 31000
SELECT fare FROM trips WHERE code = 400101;

-- @A
-- expect-value: NULL
SELECT pg_current_xact_id_if_assigned();

-- @A
-- UPDATE tạo phiên bản MỚI (xmin = X+1, xmax = 0). RETURNING đọc chính phiên bản mới.
-- expect-line: 35000 | 2 | X+1 | 0
UPDATE trips SET fare = 35000, version = version + 1, updated_at = now()
WHERE code = 400101
RETURNING fare, version, xid_td(xmin) AS xmin, xid_td(xmax) AS xmax;

-- @A
-- expect-value: X+1
SELECT xid_td(pg_current_xact_id_if_assigned());

-- ===========================================================================
-- §2. B đọc khi A CHƯA commit: thấy phiên bản CŨ, không phải chờ.

-- @B
-- Phiên bản cũ: xmin "<X" (nạp từ fixture), xmax = X+1 — A đã "đánh dấu thay thế" nó
-- nhưng chưa commit, nên với B phiên bản cũ VẪN là bản đúng.
-- expect-line: 31000 | 1 | <X | X+1
SELECT fare, version, xid_td(xmin) AS xmin, xid_td(xmax) AS xmax
FROM trips WHERE code = 400101;

-- @B
-- Snapshot của câu này gồm ba phần: xmax = (xid lớn nhất ĐÃ KẾT THÚC) + 1 — mọi xid ≥ xmax
-- coi như chưa commit; xmin = xid nhỏ nhất còn chạy — mọi xid < xmin đã kết thúc; "đang
-- chạy" = các xid trong [xmin, xmax) chưa kết thúc lúc chụp. Ở đây A (X+1) là giao dịch mới
-- nhất, chưa kết thúc ⇒ X+1 ≥ xmax: vô hình với B mà không cần nằm trong danh sách.
-- expect-value: xmin=X+1 xmax=X+1 đang chạy={}
SELECT snap_td(pg_current_snapshot());

-- @B
-- expect-value: in progress
SELECT pg_xact_status(xid_x(1));

-- Trong page heap lúc này có HAI phiên bản của chuyến 400101 (pageinspect): bản cũ ở ô 31
-- trỏ (t_ctid) sang bản mới ở ô 41. Cột hot/heap_only: bản mới nằm cùng page và không đổi cột có
-- index ⇒ HOT update (P03 PG-024 giải thích kỹ).
-- @B
-- expect-line: 31 | <X | X+1 | (0,41) | t | f
-- expect-line: 41 | X+1 | 0 | (0,41) | f | t
SELECT lp, xid_td(t_xmin) AS t_xmin, xid_td(t_xmax) AS t_xmax, t_ctid,
       (t_infomask2 & 16384) <> 0 AS hot_updated, (t_infomask2 & 32768) <> 0 AS heap_only
FROM heap_page_items(get_raw_page('toinoi.trips', 0))
WHERE lp IN (31, 41)
ORDER BY lp;

-- @A
COMMIT;

-- @B
-- Read Committed (mặc định): MỖI CÂU chụp snapshot mới ⇒ câu sau COMMIT thấy bản mới.
-- expect-line: 35000 | 2 | X+1 | 0
SELECT fare, version, xid_td(xmin) AS xmin, xid_td(xmax) AS xmax
FROM trips WHERE code = 400101;

-- @B
-- expect-value: committed
SELECT pg_xact_status(xid_x(1));

-- @B
-- Không còn giao dịch nào chạy: xmin = xmax = xid kế tiếp.
-- expect-value: xmin=X+2 xmax=X+2 đang chạy={}
SELECT snap_td(pg_current_snapshot());

-- ===========================================================================
-- §3. Snapshot của CẢ GIAO DỊCH (REPEATABLE READ): chụp ở câu đầu tiên, giữ tới hết.

-- @B
BEGIN ISOLATION LEVEL REPEATABLE READ;

-- @B
-- expect-value: 32000
SELECT fare FROM trips WHERE code = 400102;

-- @A
-- Câu lẻ (autocommit) = một giao dịch X+2, commit ngay.
-- expect-line: 36000 | X+2
UPDATE trips SET fare = 36000, version = version + 1, updated_at = now()
WHERE code = 400102
RETURNING fare, xid_td(xmin);

-- @B
-- B vẫn thấy 32000: X+2 commit SAU khi B chụp snapshot (snapshot của B có xmax = X+2).
-- expect-line: 32000 | <X | X+2
SELECT fare, xid_td(xmin) AS xmin, xid_td(xmax) AS xmax FROM trips WHERE code = 400102;

-- @B
-- expect-value: xmin=X+2 xmax=X+2 đang chạy={}
SELECT snap_td(pg_current_snapshot());

-- @B
COMMIT;

-- @B
-- expect-value: 36000
SELECT fare FROM trips WHERE code = 400102;

-- ===========================================================================
-- §4. ROLLBACK cũng để lại dấu vết: phiên bản do giao dịch bị huỷ tạo ra là dòng CHẾT,
-- và bản cũ còn sống vẫn mang xmax của giao dịch đã huỷ.

-- @A
BEGIN;

-- @A
-- expect-value: X+3
UPDATE trips SET fare = 99000 WHERE code = 400103 RETURNING xid_td(xmin);

-- @A
ROLLBACK;

-- @B
-- xmax = X+3 nhưng dòng VẪN SỐNG: X+3 đã abort ⇒ "xmax khác 0" không có nghĩa "đã bị xoá".
-- Người đọc phải tra trạng thái giao dịch (pg_xact) — hoặc đọc hint bit đã ghi lên dòng.
-- expect-line: 33000 | <X | X+3 | aborted
SELECT fare, xid_td(xmin) AS xmin, xid_td(xmax) AS xmax, pg_xact_status(xid_x(3)) AS trang_thai_x3
FROM trips WHERE code = 400103;

-- Dòng chết của bảng trips lúc này: bản cũ của 400101 (thay bởi X+1), bản cũ của 400102
-- (thay bởi X+2) và bản mới của 400103 do X+3 tạo rồi huỷ — 3 phiên bản không ai còn thấy.
-- @B
-- expect-line: 40 | 3
SELECT tuple_count, dead_tuple_count FROM pgstattuple('toinoi.trips');

-- VACUUM dọn phiên bản chết khi KHÔNG CÒN snapshot nào có thể cần chúng (giao dịch mở lâu
-- giữ VACUUM lại — thí nghiệm p03/thu-nghiem/pg026-snapshot-cu-chan-vacuum.sql).
-- @B
VACUUM toinoi.trips;

-- @B
-- expect-line: 40 | 0
SELECT tuple_count, dead_tuple_count FROM pgstattuple('toinoi.trips');

-- ===========================================================================
-- §5. xid 32 bit (cột xmin/xmax, kiểu xid) và xid8 64 bit (pg_current_xact_id(), có
-- "epoch" — không bao giờ quay vòng). txid_current() là tên cũ, trả bigint.

-- @A
-- expect-line: xid8 | bigint | xid | t
SELECT pg_typeof(pg_current_xact_id()) AS kieu_moi,
       pg_typeof(txid_current()) AS kieu_cu,
       pg_typeof(xmin) AS kieu_cot_he_thong,
       pg_current_xact_id()::text::bigint = txid_current() AS cung_mot_so
FROM trips WHERE code = 400101;
