// Đặc tả phép ĐO của P03 — chạy bằng `npm run p03:measure` (= node pg/run-measure.mjs p03/do-dac.mjs),
// ghi kết quả ra p03/DO-DAC.md. KHÔNG nằm trong `npm test` (mất vài phút, số phụ thuộc máy).
//
// Mỗi nhóm: một database mới chép từ nền p03; `chuan_bi` chạy một lần; mỗi phép đo:
// thiet_lap → [truoc_moi_lan → ĐO → sau_moi_lan] × (khởi động + lap) → go_bo.
// `do` dùng được biến psql đặt trong chuan_bi (\gset), vd. :'kh_b1'.

const LS_GOC = `PREPARE lich_su_goc(uuid) AS
SELECT t.code, t.completed_at, t.kind, t.fare,
       coalesce(sum(p.amount) FILTER (WHERE p.kind = 'charge' AND p.status = 'succeeded'), 0)
     - coalesce(sum(p.amount) FILTER (WHERE p.kind = 'refund' AND p.status = 'succeeded'), 0) AS da_thu
FROM trips AS t LEFT JOIN payments AS p ON p.trip_id = t.id
WHERE t.customer_id = $1 AND t.status = 'completed'
GROUP BY t.id ORDER BY t.completed_at DESC, t.code DESC LIMIT 20;
PREPARE lich_su_moi(uuid) AS
SELECT t.code, t.completed_at, t.kind, t.fare, tien.da_thu
FROM (SELECT id, code, completed_at, kind, fare FROM trips
      WHERE customer_id = $1 AND status = 'completed'
      ORDER BY completed_at DESC, code DESC LIMIT 20) AS t
CROSS JOIN LATERAL (
  SELECT coalesce(sum(p.amount) FILTER (WHERE p.kind = 'charge' AND p.status = 'succeeded'), 0)
       - coalesce(sum(p.amount) FILTER (WHERE p.kind = 'refund' AND p.status = 'succeeded'), 0) AS da_thu
  FROM payments AS p WHERE p.trip_id = t.id) AS tien
ORDER BY t.completed_at DESC, t.code DESC;`;

const KHACH = `SELECT id AS kh_le FROM customers WHERE phone = '0910000500' \\gset
SELECT id AS kh_b1 FROM customers WHERE phone = '0919990001' \\gset`;

const CHEP_50K = `INSERT INTO ghi_thu SELECT * FROM trips WHERE code BETWEEN 600000 AND 649999`;
const NGAY_MOI = `INSERT INTO trips (id, kind, city_id, customer_id, driver_id, merchant_id, status, fare,
                   requested_at, accepted_at, completed_at, updated_at, pickup_area_id)
SELECT p03_uuid('ngay-moi', code), kind, city_id, customer_id, driver_id, merchant_id, status, fare,
       requested_at + interval '9 days', accepted_at + interval '9 days', completed_at + interval '9 days',
       updated_at + interval '9 days', pickup_area_id
FROM trips WHERE code BETWEEN 1060000 AND 1064999 AND status = 'completed'`;

export default {
  tieu_de: 'P03 · Kết quả đo (PG-021 … PG-030)',
  lenh: 'p03:measure',
  tep: 'p03/do-dac.mjs',
  ra: 'p03/DO-DAC.md',
  nen: { manifest: 'p03/lab.json', khoa: 'p03' },
  lap: 7,
  khoi_dong: 1,
  phuong_phap: `
**Cách đo.** \`explain\` = \`EXPLAIN (ANALYZE, TIMING OFF, BUFFERS, FORMAT JSON)\`, lấy *Execution Time*
(thời gian thực thi phía server — không gồm lập kế hoạch, không gồm gửi kết quả về client; TIMING OFF để bớt
phụ phí đo từng nút). \`client\` = \`\\timing\` của psql: từ lúc gửi tới lúc nhận xong, GỒM commit (fsync WAL) —
dùng cho câu ghi. "Buffers" = shared hit + read ở nút gốc. **Cache:** "nóng" = đã chạy một lần khởi động,
page nằm trong shared_buffers (hoặc, với bảng lớn hơn 1/4 shared_buffers quét tuần tự, trong page cache của
hệ điều hành); "lạnh SB" = trước MỖI lần đo đẩy page của bảng/index liên quan ra khỏi shared_buffers bằng
\`pg_buffercache_evict_relation()\` — page cache của hệ điều hành VẪN nóng (lab không xoá được cache OS), nên
"lạnh SB" chưa phải đọc đĩa thật. Mỗi nhóm chạy trên một database mới chép từ nền p03, một phiên, không tải nền.`,
  gioi_han: `
- Một máy, một lần chạy lệnh đo, dữ liệu tổng hợp (1 triệu chuyến) — số tuyệt đối KHÔNG mang sang máy khác
  hay production được; dùng để so sánh TƯƠNG ĐỐI giữa các phương án trên cùng máy, cùng lúc.
- Ổ đĩa và page cache của hệ điều hành ảnh hưởng mạnh tới các phép đo "lạnh"/Seq Scan; lab không xoá được cache OS.
- \`explain\` không tính thời gian lập kế hoạch và truyền kết quả; câu trả nhiều dòng sẽ chậm hơn thế khi chạy thật.
- Máy đo có thể có tiến trình khác chạy nền (trình duyệt, IDE, service PostgreSQL khác) — xem độ chênh min/max.
- pgbench: tải tổng hợp một loại câu; bộ nhớ đo bằng *private bytes* của các backend pgbench (Windows,
  lấy mẫu ~300 ms một lần) — đỉnh thật có thể cao hơn giữa hai lần lấy mẫu.`,
  nhom: [
    {
      id: 'G1', bai: 'PG-021 / PG-023', ten: 'Lịch sử chuyến của khách — có/không index, cache nóng/lạnh',
      mo_ta: 'Khách cá nhân 0910000500 (40 chuyến) và khách doanh nghiệp B1 0919990001 (51.151 chuyến).',
      chuan_bi: `CREATE EXTENSION pg_buffercache WITH SCHEMA public;\n${KHACH}`,
      do: [
        { id: 'G1-a', ten: 'Khách cá nhân, có index (customer_id, requested_at)', do: `SELECT code, requested_at, fare FROM trips WHERE customer_id = :'kh_le'` },
        { id: 'G1-b', ten: 'Như G1-a, lạnh SB (đẩy trips + index ra khỏi shared_buffers trước mỗi lần)', cache: 'lạnh SB',
          truoc_moi_lan: `SELECT pg_buffercache_evict_relation('trips'), pg_buffercache_evict_relation('ix_trips_customer_requested');`,
          do: `SELECT code, requested_at, fare FROM trips WHERE customer_id = :'kh_le'` },
        { id: 'G1-c', ten: 'Khách cá nhân, KHÔNG index (Seq Scan 19.441 page)', lap: 5,
          thiet_lap: 'BEGIN;\nDROP INDEX ix_trips_customer_requested;', go_bo: 'ROLLBACK;',
          do: `SELECT code, requested_at, fare FROM trips WHERE customer_id = :'kh_le'` },
        { id: 'G1-d', ten: '20 chuyến gần nhất của B1 (Index Scan + Limit)', do: `SELECT code, requested_at, status, fare FROM trips WHERE customer_id = :'kh_b1' ORDER BY requested_at DESC LIMIT 20` },
        { id: 'G1-e', ten: '20 chuyến gần nhất khách cá nhân (Bitmap + Sort)', do: `SELECT code, requested_at, status, fare FROM trips WHERE customer_id = :'kh_le' ORDER BY requested_at DESC LIMIT 20` },
      ],
    },
    {
      id: 'G2', bai: 'PG-023', ten: 'Độ chọn lọc: index hay Seq Scan (cột fare ngẫu nhiên, cột requested_at thẳng hàng)',
      mo_ta: 'Cùng câu SELECT * với hai mức dòng khớp; so kế hoạch planner chọn với kế hoạch bị ép (enable_* = off).',
      chuan_bi: 'CREATE INDEX ix_trips_fare ON trips (fare);\nCREATE INDEX ix_trips_requested ON trips (requested_at);',
      do: [
        { id: 'G2-a', ten: 'fare 120k–150k (11 %) — planner chọn Bitmap', lap: 5, do: 'SELECT * FROM trips WHERE fare BETWEEN 120000 AND 150000' },
        { id: 'G2-b', ten: 'fare 120k–150k (11 %) — ép Seq Scan', lap: 5,
          thiet_lap: 'SET enable_bitmapscan = off;\nSET enable_indexscan = off;', go_bo: 'RESET enable_bitmapscan;\nRESET enable_indexscan;',
          do: 'SELECT * FROM trips WHERE fare BETWEEN 120000 AND 150000' },
        { id: 'G2-c', ten: 'fare 60k–150k (55 %) — planner chọn Seq Scan', lap: 5, do: 'SELECT * FROM trips WHERE fare BETWEEN 60000 AND 150000' },
        { id: 'G2-d', ten: 'fare 60k–150k (55 %) — ép Bitmap', lap: 5,
          thiet_lap: 'SET enable_seqscan = off;', go_bo: 'RESET enable_seqscan;',
          do: 'SELECT * FROM trips WHERE fare BETWEEN 60000 AND 150000' },
        { id: 'G2-e', ten: 'requested_at 60 ngày (~36 %) — planner chọn Index Scan', lap: 5,
          do: `SELECT * FROM trips WHERE requested_at >= TIMESTAMPTZ '2026-09-01 00:00+07' AND requested_at < TIMESTAMPTZ '2026-10-31 00:00+07'` },
        { id: 'G2-f', ten: 'requested_at 60 ngày (~36 %) — ép Seq Scan', lap: 5,
          thiet_lap: 'SET enable_indexscan = off;\nSET enable_bitmapscan = off;', go_bo: 'RESET enable_indexscan;\nRESET enable_bitmapscan;',
          do: `SELECT * FROM trips WHERE requested_at >= TIMESTAMPTZ '2026-09-01 00:00+07' AND requested_at < TIMESTAMPTZ '2026-10-31 00:00+07'` },
      ],
    },
    {
      id: 'G3', bai: 'PG-025', ten: 'Chi phí ghi: chép 50.000 chuyến vào bảng có 0 / 5 / 9 index',
      mo_ta: 'Mỗi lần đo: TRUNCATE + CHECKPOINT rồi `INSERT … SELECT` 50.000 dòng (đo phía client, gồm commit). Cùng định nghĩa index như 25-index-nhieu-cot-partial-expression.sql.',
      chuan_bi: 'CREATE TABLE ghi_thu (LIKE trips INCLUDING DEFAULTS);',
      do: [
        { id: 'G3-a', ten: '0 index', cach_do: 'client', lap: 5, truoc_moi_lan: 'TRUNCATE ghi_thu;\nCHECKPOINT;', do: CHEP_50K },
        { id: 'G3-b', ten: '5 index (như trips của nền)', cach_do: 'client', lap: 5,
          thiet_lap: `ALTER TABLE ghi_thu ADD CONSTRAINT pk_ghi_thu PRIMARY KEY (id);
ALTER TABLE ghi_thu ADD CONSTRAINT uq_ghi_thu_code UNIQUE (code);
CREATE INDEX ON ghi_thu (city_id, requested_at);
CREATE INDEX ON ghi_thu (customer_id, requested_at DESC);
CREATE INDEX ON ghi_thu (driver_id);`,
          truoc_moi_lan: 'TRUNCATE ghi_thu;\nCHECKPOINT;', do: CHEP_50K },
        { id: 'G3-c', ten: '9 index (+ đảo cột, partial, status, expression)', cach_do: 'client', lap: 5,
          thiet_lap: `CREATE INDEX ON ghi_thu (requested_at, city_id);
CREATE INDEX ON ghi_thu (requested_at) WHERE status IN ('requested', 'searching', 'accepted', 'arriving', 'picked_up', 'in_transit');
CREATE INDEX ON ghi_thu (status);
CREATE INDEX ON ghi_thu (((requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::date));`,
          truoc_moi_lan: 'TRUNCATE ghi_thu;\nCHECKPOINT;', do: CHEP_50K },
      ],
    },
    {
      id: 'G4', bai: 'PG-026', ten: 'Index Only Scan, heap fetch và VACUUM',
      mo_ta: 'Giờ đặt các chuyến tháng 10 của B1 (9.365 dòng).',
      chuan_bi: KHACH,
      do: [
        { id: 'G4-a', ten: 'Index Only Scan, bảng vừa VACUUM (Heap Fetches 0)',
          do: `SELECT requested_at FROM trips WHERE customer_id = :'kh_b1' AND requested_at >= TIMESTAMPTZ '2026-10-01 00:00+07'` },
        { id: 'G4-b', ten: 'Thêm cột fare (phải đọc heap)',
          do: `SELECT requested_at, fare FROM trips WHERE customer_id = :'kh_b1' AND requested_at >= TIMESTAMPTZ '2026-10-01 00:00+07'` },
        { id: 'G4-c', ten: 'Index Only Scan sau khi sửa 9.365 dòng, CHƯA VACUUM (Heap Fetches 18.536)',
          thiet_lap: `UPDATE trips SET fare = fare + 1000 WHERE customer_id = :'kh_b1' AND requested_at >= TIMESTAMPTZ '2026-10-01 00:00+07';`,
          do: `SELECT requested_at FROM trips WHERE customer_id = :'kh_b1' AND requested_at >= TIMESTAMPTZ '2026-10-01 00:00+07'` },
        { id: 'G4-d', ten: 'Như G4-c, sau VACUUM',
          thiet_lap: 'VACUUM trips;',
          do: `SELECT requested_at FROM trips WHERE customer_id = :'kh_b1' AND requested_at >= TIMESTAMPTZ '2026-10-01 00:00+07'` },
        { id: 'G4-e', ten: 'Covering index INCLUDE (fare, status)',
          thiet_lap: 'CREATE INDEX ix_trips_khach_bao_gom ON trips (customer_id, requested_at DESC) INCLUDE (fare, status);',
          do: `SELECT requested_at, fare, status FROM trips WHERE customer_id = :'kh_b1' AND requested_at >= TIMESTAMPTZ '2026-10-01 00:00+07'` },
      ],
    },
    {
      id: 'G5', bai: 'PG-028', ten: 'GIN trigram và BRIN (dữ liệu theo thứ tự vs xáo trộn)',
      mo_ta: 'Dữ liệu GPS sinh như 28-gin-gist-brin.sql (2.000.000 điểm). Câu BRIN: "5 phút gần nhất" (61.000 điểm). Các phép đo BRIN tắt Index Scan để loại khoá chính (driver_id, recorded_at).',
      chuan_bi: `CREATE EXTENSION pg_trgm WITH SCHEMA public;
ALTER TABLE driver_locations DROP CONSTRAINT fk_driver_locations_driver, DROP CONSTRAINT pk_driver_locations;
SET work_mem = '256MB';
INSERT INTO driver_locations (driver_id, recorded_at, lat, lng)
WITH tx AS MATERIALIZED (
  SELECT d, p03_uuid('tx', d) AS id, 10.70 + p03_u(md5('gps' || d::text), 0) * 0.15 AS lat0, 106.60 + p03_u(md5('gps' || d::text), 1) * 0.15 AS lng0
  FROM generate_series(1, 1000) AS d)
SELECT tx.id, TIMESTAMPTZ '2026-10-31 01:00:00+07' - (2000 - i) * interval '5 seconds', tx.lat0 + i * 0.00001, tx.lng0 - i * 0.00001
FROM generate_series(1, 2000) AS i CROSS JOIN tx ORDER BY i, tx.d;
CREATE TABLE driver_locations_xao AS SELECT * FROM driver_locations ORDER BY md5(driver_id::text || recorded_at::text);
RESET work_mem;
ALTER TABLE driver_locations ADD CONSTRAINT pk_driver_locations PRIMARY KEY (driver_id, recorded_at);
CREATE INDEX ix_dl_brin ON driver_locations USING brin (recorded_at);
CREATE INDEX ix_dl_xao_brin ON driver_locations_xao USING brin (recorded_at);
VACUUM ANALYZE driver_locations;
VACUUM ANALYZE driver_locations_xao;`,
      do: [
        { id: 'G5-a', ten: "Tên quán ILIKE '%tấm bà năm%' — không index (Seq Scan 15.000 quán)", do: `SELECT name FROM merchants WHERE name ILIKE '%tấm bà năm%'` },
        { id: 'G5-b', ten: 'Như G5-a — GIN gin_trgm_ops',
          thiet_lap: 'CREATE INDEX ix_merchants_name_trgm ON merchants USING gin (name gin_trgm_ops);',
          do: `SELECT name FROM merchants WHERE name ILIKE '%tấm bà năm%'` },
        { id: 'G5-c', ten: 'GPS 5 phút — BRIN, bảng THEO THỨ TỰ',
          thiet_lap: 'SET enable_indexscan = off;\nSET enable_indexonlyscan = off;', go_bo: 'RESET enable_indexscan;\nRESET enable_indexonlyscan;',
          do: `SELECT count(*) FROM driver_locations WHERE recorded_at >= TIMESTAMPTZ '2026-10-31 00:55:00+07'` },
        { id: 'G5-d', ten: 'GPS 5 phút — Seq Scan, bảng theo thứ tự', lap: 5,
          thiet_lap: 'SET enable_indexscan = off;\nSET enable_indexonlyscan = off;\nSET enable_bitmapscan = off;',
          go_bo: 'RESET enable_indexscan;\nRESET enable_indexonlyscan;\nRESET enable_bitmapscan;',
          do: `SELECT count(*) FROM driver_locations WHERE recorded_at >= TIMESTAMPTZ '2026-10-31 00:55:00+07'` },
        { id: 'G5-e', ten: 'GPS 5 phút — B-tree trên recorded_at (Index Only Scan)',
          thiet_lap: 'CREATE INDEX ix_dl_btree ON driver_locations (recorded_at);\nVACUUM driver_locations;',
          do: `SELECT count(*) FROM driver_locations WHERE recorded_at >= TIMESTAMPTZ '2026-10-31 00:55:00+07'` },
        { id: 'G5-f', ten: 'GPS 5 phút — bảng XÁO TRỘN, planner chọn (Seq Scan)', lap: 5,
          do: `SELECT count(*) FROM driver_locations_xao WHERE recorded_at >= TIMESTAMPTZ '2026-10-31 00:55:00+07'` },
        { id: 'G5-g', ten: 'GPS 5 phút — bảng XÁO TRỘN, ép BRIN', lap: 5,
          thiet_lap: 'SET enable_seqscan = off;', go_bo: 'RESET enable_seqscan;',
          do: `SELECT count(*) FROM driver_locations_xao WHERE recorded_at >= TIMESTAMPTZ '2026-10-31 00:55:00+07'` },
      ],
    },
    {
      id: 'G6', bai: 'PG-029', ten: 'Sort và hash theo work_mem (một phiên)',
      do: [
        { id: 'G6-a', ten: 'Sắp 1 triệu chuyến, work_mem 64kB (external merge)', lap: 5,
          thiet_lap: "SET work_mem = '64kB';", go_bo: 'RESET work_mem;', do: 'SELECT code, fare FROM trips ORDER BY fare DESC, code' },
        { id: 'G6-b', ten: 'Sắp 1 triệu chuyến, work_mem 4MB (external merge)', lap: 5,
          do: 'SELECT code, fare FROM trips ORDER BY fare DESC, code' },
        { id: 'G6-c', ten: 'Sắp 1 triệu chuyến, work_mem 256MB (quicksort)', lap: 5,
          thiet_lap: "SET work_mem = '256MB';", go_bo: 'RESET work_mem;', do: 'SELECT code, fare FROM trips ORDER BY fare DESC, code' },
        { id: 'G6-d', ten: 'Đếm chuyến theo khách (191.852 nhóm), work_mem 4MB', lap: 5,
          do: 'SELECT customer_id, count(*) FROM trips GROUP BY customer_id' },
        { id: 'G6-e', ten: 'Như G6-d, work_mem 64MB', lap: 5,
          thiet_lap: "SET work_mem = '64MB';", go_bo: 'RESET work_mem;', do: 'SELECT customer_id, count(*) FROM trips GROUP BY customer_id' },
        { id: 'G6-f', ten: 'Doanh thu theo loại khách (Hash Join, bảng băm 200.003 khách), hash_mem_multiplier 2', lap: 5,
          do: 'SELECT c.kind, count(*), sum(t.fare) FROM trips AS t JOIN customers AS c ON c.id = t.customer_id GROUP BY c.kind' },
        { id: 'G6-g', ten: 'Như G6-f, hash_mem_multiplier 8 (một batch)', lap: 5,
          thiet_lap: 'SET hash_mem_multiplier = 8;', go_bo: 'RESET hash_mem_multiplier;',
          do: 'SELECT c.kind, count(*), sum(t.fare) FROM trips AS t JOIN customers AS c ON c.id = t.customer_id GROUP BY c.kind' },
      ],
    },
    {
      id: 'G7', bai: 'PG-029', ten: 'Nhiều phiên đồng thời × work_mem (pgbench)',
      mo_ta: 'Mỗi giao dịch pgbench chạy một câu gom nhóm theo khách trên cả bảng (HashAggregate 191.852 nhóm — tràn đĩa ở work_mem 4MB, vừa bộ nhớ ở 64MB). work_mem đặt qua PGOPTIONS cho mọi phiên pgbench. Mỗi cấu hình 10 giây.',
      pgbench: {
        giay: 10,
        script: 'SELECT customer_id, count(*) AS n FROM toinoi.trips GROUP BY customer_id ORDER BY n DESC LIMIT 10;',
        cau_hinh: [
          { nhan: 'work_mem 4MB', clients: 1, pgoptions: '-c work_mem=4MB' },
          { nhan: 'work_mem 4MB', clients: 4, pgoptions: '-c work_mem=4MB' },
          { nhan: 'work_mem 4MB', clients: 8, pgoptions: '-c work_mem=4MB' },
          { nhan: 'work_mem 64MB', clients: 1, pgoptions: '-c work_mem=64MB' },
          { nhan: 'work_mem 64MB', clients: 4, pgoptions: '-c work_mem=64MB' },
          { nhan: 'work_mem 64MB', clients: 8, pgoptions: '-c work_mem=64MB' },
        ],
      },
    },
    {
      id: 'G8', bai: 'PG-030', ten: 'Tối ưu lịch sử chuyến hoàn thành của khách: trước / sau + chi phí ghi',
      mo_ta: 'Câu gốc và câu viết lại như 30-toi-uu-truy-van.sql. Mặc định ép kế hoạch RIÊNG cho từng tham số (plan_cache_mode = force_custom_plan) — để auto thì sau 5 lần EXECUTE PostgreSQL có thể đổi sang kế hoạch CHUNG giữa chừng và trộn hai kế hoạch vào cùng một phép đo; G8-h/G8-i đo riêng kế hoạch chung. "Ghi 5.000 chuyến" đo trong giao dịch rồi ROLLBACK + VACUUM sau mỗi lần (không gồm commit; VACUUM dọn dòng/mục index chết của lần trước để các lần đo như nhau).',
      chuan_bi: `SET plan_cache_mode = force_custom_plan;\n${KHACH}\n${LS_GOC}`,
      do: [
        { id: 'G8-a', ten: 'TRƯỚC — câu gốc, khách B1', do: `EXECUTE lich_su_goc(:'kh_b1')` },
        { id: 'G8-b', ten: 'TRƯỚC — câu gốc, khách cá nhân', do: `EXECUTE lich_su_goc(:'kh_le')` },
        { id: 'G8-c', ten: 'Ghi 5.000 chuyến — KHÔNG có index mới', cach_do: 'client', lap: 5,
          truoc_moi_lan: 'BEGIN;', sau_moi_lan: 'ROLLBACK;\nVACUUM trips;', do: NGAY_MOI },
        { id: 'G8-d', ten: 'SAU — câu viết lại + index partial mới, khách B1',
          thiet_lap: `CREATE INDEX ix_trips_khach_hoan_thanh ON trips (customer_id, completed_at DESC, code DESC) WHERE status = 'completed';`,
          do: `EXECUTE lich_su_moi(:'kh_b1')` },
        { id: 'G8-e', ten: 'SAU — câu viết lại, khách cá nhân', do: `EXECUTE lich_su_moi(:'kh_le')` },
        { id: 'G8-f', ten: 'Câu GỐC (không viết lại) khi đã có index mới, khách B1', do: `EXECUTE lich_su_goc(:'kh_b1')` },
        { id: 'G8-h', ten: 'Kế hoạch CHUNG (generic) của câu viết lại, khách B1', lap: 5,
          thiet_lap: 'SET plan_cache_mode = force_generic_plan;', go_bo: 'SET plan_cache_mode = force_custom_plan;',
          do: `EXECUTE lich_su_moi(:'kh_b1')` },
        { id: 'G8-i', ten: 'Kế hoạch CHUNG của câu gốc, khách B1', lap: 5,
          thiet_lap: 'SET plan_cache_mode = force_generic_plan;', go_bo: 'SET plan_cache_mode = force_custom_plan;',
          do: `EXECUTE lich_su_goc(:'kh_b1')` },
        { id: 'G8-g', ten: 'Ghi 5.000 chuyến — CÓ index mới', cach_do: 'client', lap: 5,
          truoc_moi_lan: 'BEGIN;', sau_moi_lan: 'ROLLBACK;\nVACUUM trips;', do: NGAY_MOI },
      ],
    },
  ],
};
