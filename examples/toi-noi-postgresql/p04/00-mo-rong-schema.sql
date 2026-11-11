-- P04 · 00-mo-rong-schema.sql — mở rộng lược đồ P01/P02 cho phần "Giao dịch, MVCC và đồng thời".
--
-- Nền P04 (p04/lab.json): p01/00-reset.sql → p01/01-schema.sql → p02/00-mo-rong-schema.sql
-- (chỉ LƯỢC ĐỒ, không fixture P01/P02) → tệp này → 01-fixture.sql → 02-kiem-tra-fixture.sql.
-- P04 không cần bảng lớn: mọi hiện tượng của phần này (khoá, snapshot, xung đột) xảy ra trên
-- VÀI dòng; phép thử đồng thời (p04/dong-thoi.mjs) tự sinh dữ liệu của nó.
--
-- Tệp này chỉ THÊM (không sửa cột P01/P02):
--   * trips.version           — số phiên bản dòng cho optimistic concurrency (PG-035)
--   * wallets                 — ví VíXanh của khách và tài xế (lost update PG-034, deadlock PG-037)
--   * trip_offers             — lời mời nhận chuyến gửi tới từng tài xế (PG-040)
--   * trip_assignments        — phân công: MỘT chuyến MỘT tài xế bằng khoá chính (PG-040)
--   * trip_claims             — nhật ký "ứng dụng đã báo tài xế: bạn nhận được chuyến" (PG-040)
--   * driver_duty             — tài xế trực khu: quy tắc "mỗi khu luôn còn ≥ 1 người trực" (PG-038)
--   * jobs, job_runs          — hàng đợi việc cho worker + nhật ký mỗi lần xử lý (PG-036)
--   * idempotency_keys        — khoá chống lặp request (PG-039)
--   * outbox, broker_messages, processed_messages, notifications — transactional outbox (PG-039)
--   * lab_moc_xid + hàm xid_td()/snap_td() — in xid TƯƠNG ĐỐI (X+1, X+2…) cho đầu ra tất định
--   * lab_khoa(), lab_cho()   — đọc pg_locks / pg_stat_activity theo NHÃN PHIÊN (A, B…) thay PID

SET search_path TO toinoi, public;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

DO $$
BEGIN
  IF to_regclass('toinoi.trips') IS NULL OR to_regclass('toinoi.areas') IS NULL THEN
    RAISE EXCEPTION 'Chưa có lược đồ P01 + P02 trong schema toinoi'
      USING HINT = 'Nền P04 = p01/00-reset.sql, p01/01-schema.sql, p02/00-mo-rong-schema.sql rồi p04/00…02 (npm run p04 làm sẵn).';
  END IF;
  IF current_setting('server_version_num')::int < 180000 THEN
    RAISE EXCEPTION 'Lab P04 đã kiểm trên PostgreSQL 18; server này là %', current_setting('server_version');
  END IF;
END
$$;

-- pageinspect: xem các phiên bản dòng trong page (PG-032); pgstattuple: đếm dòng chết.
CREATE EXTENSION IF NOT EXISTS pageinspect SCHEMA public;
CREATE EXTENSION IF NOT EXISTS pgstattuple SCHEMA public;

-- ---------------------------------------------------------------------------
-- Phiên bản dòng cho optimistic concurrency: mỗi lần SỬA chuyến, ứng dụng tăng version
-- và chỉ sửa khi version còn đúng như lúc đọc (UPDATE … WHERE id = … AND version = …).
ALTER TABLE trips ADD COLUMN version integer NOT NULL DEFAULT 1;
ALTER TABLE trips ADD CONSTRAINT ck_trips_version_positive CHECK (version > 0);
COMMENT ON COLUMN trips.version IS 'Phiên bản dòng (optimistic concurrency): +1 mỗi lần sửa';

-- ---------------------------------------------------------------------------
-- Ví VíXanh. id là số nhỏ, đặt tay (khách 1001…, tài xế 2001…) để "khoá theo thứ tự id"
-- ở PG-037 dễ đọc. Số dư là số nguyên đồng, KHÔNG âm.
CREATE TABLE wallets (
  id          integer     NOT NULL,
  owner_kind  text        NOT NULL,
  customer_id uuid,
  driver_id   uuid,
  balance     integer     NOT NULL,
  updated_at  timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT pk_wallets PRIMARY KEY (id),
  CONSTRAINT uq_wallets_customer UNIQUE (customer_id),
  CONSTRAINT uq_wallets_driver UNIQUE (driver_id),
  CONSTRAINT fk_wallets_customer FOREIGN KEY (customer_id) REFERENCES customers (id),
  CONSTRAINT fk_wallets_driver FOREIGN KEY (driver_id) REFERENCES drivers (id),
  CONSTRAINT ck_wallets_owner_kind CHECK (owner_kind IN ('customer', 'driver')),
  CONSTRAINT ck_wallets_one_owner CHECK (
    (owner_kind = 'customer' AND customer_id IS NOT NULL AND driver_id IS NULL)
    OR (owner_kind = 'driver' AND driver_id IS NOT NULL AND customer_id IS NULL)),
  CONSTRAINT ck_wallets_balance_non_negative CHECK (balance >= 0)
);
COMMENT ON TABLE wallets IS 'Ví VíXanh của khách/tài xế (đồng, không âm) — dữ liệu hư cấu';

-- ---------------------------------------------------------------------------
-- Lời mời nhận chuyến: hệ thống gửi chuyến cho vài tài xế gần nhất cùng lúc.
CREATE TABLE trip_offers (
  trip_id      uuid        NOT NULL,
  driver_id    uuid        NOT NULL,
  offered_at   timestamptz NOT NULL DEFAULT now(),
  status       text        NOT NULL DEFAULT 'offered',
  responded_at timestamptz,
  CONSTRAINT pk_trip_offers PRIMARY KEY (trip_id, driver_id),
  CONSTRAINT fk_trip_offers_trip FOREIGN KEY (trip_id) REFERENCES trips (id),
  CONSTRAINT fk_trip_offers_driver FOREIGN KEY (driver_id) REFERENCES drivers (id),
  CONSTRAINT ck_trip_offers_status CHECK (status IN ('offered', 'accepted', 'lost', 'rejected', 'expired')),
  CONSTRAINT ck_trip_offers_responded CHECK ((status = 'offered') = (responded_at IS NULL))
);

-- Phân công: KHOÁ CHÍNH trip_id ⇒ một chuyến tối đa MỘT tài xế, do database bảo đảm
-- (cách (d) của PG-040). Bảng riêng để so sánh với cách cập nhật trips có điều kiện.
CREATE TABLE trip_assignments (
  trip_id     uuid        NOT NULL,
  driver_id   uuid        NOT NULL,
  assigned_at timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT pk_trip_assignments PRIMARY KEY (trip_id),
  CONSTRAINT fk_trip_assignments_trip FOREIGN KEY (trip_id) REFERENCES trips (id),
  CONSTRAINT fk_trip_assignments_driver FOREIGN KEY (driver_id) REFERENCES drivers (id)
);

-- Nhật ký "ứng dụng đã báo tài xế: BẠN NHẬN ĐƯỢC CHUYẾN" — ghi trong CÙNG giao dịch với lệnh
-- nhận. Đây là thước đo của bất biến PG-040: mỗi chuyến đúng MỘT dòng. KHÔNG có UNIQUE
-- (trip_id) — cố ý, để cách làm sai lộ ra thành hai dòng thay vì bị chặn.
CREATE TABLE trip_claims (
  id         bigint      GENERATED ALWAYS AS IDENTITY,
  trip_id    uuid        NOT NULL,
  driver_id  uuid        NOT NULL,
  method     text        NOT NULL,
  claimed_at timestamptz NOT NULL DEFAULT clock_timestamp(),
  CONSTRAINT pk_trip_claims PRIMARY KEY (id),
  CONSTRAINT fk_trip_claims_trip FOREIGN KEY (trip_id) REFERENCES trips (id),
  CONSTRAINT fk_trip_claims_driver FOREIGN KEY (driver_id) REFERENCES drivers (id)
);
CREATE INDEX ix_trip_claims_trip ON trip_claims (trip_id);

-- ---------------------------------------------------------------------------
-- Tài xế trực khu (PG-038): quy tắc nghiệp vụ "mỗi khu đang mở luôn còn ít nhất MỘT tài
-- xế trực" — quy tắc trên NHIỀU dòng, không CHECK/UNIQUE nào diễn đạt được.
CREATE TABLE driver_duty (
  driver_id  uuid        NOT NULL,
  area_id    integer     NOT NULL,
  on_duty    boolean     NOT NULL,
  changed_at timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT pk_driver_duty PRIMARY KEY (driver_id),
  CONSTRAINT fk_driver_duty_driver FOREIGN KEY (driver_id) REFERENCES drivers (id),
  CONSTRAINT fk_driver_duty_area FOREIGN KEY (area_id) REFERENCES areas (id)
);
CREATE INDEX ix_driver_duty_area ON driver_duty (area_id);

-- ---------------------------------------------------------------------------
-- Hàng đợi việc nền (PG-036): gửi thông báo, tính cước, đối soát…
CREATE TABLE jobs (
  id         bigint      GENERATED ALWAYS AS IDENTITY,
  kind       text        NOT NULL,
  trip_id    uuid,
  priority   smallint    NOT NULL DEFAULT 5,       -- 1 = gấp nhất
  status     text        NOT NULL DEFAULT 'queued',
  attempts   integer     NOT NULL DEFAULT 0,
  run_after  timestamptz NOT NULL DEFAULT now(),
  locked_by  text,
  locked_at  timestamptz,
  done_at    timestamptz,
  created_at timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT pk_jobs PRIMARY KEY (id),
  CONSTRAINT fk_jobs_trip FOREIGN KEY (trip_id) REFERENCES trips (id),
  CONSTRAINT ck_jobs_kind CHECK (kind IN ('gui_thong_bao', 'tinh_cuoc', 'doi_soat')),
  CONSTRAINT ck_jobs_status CHECK (status IN ('queued', 'running', 'done', 'failed')),
  CONSTRAINT ck_jobs_priority CHECK (priority BETWEEN 1 AND 9),
  CONSTRAINT ck_jobs_running_locked CHECK ((status = 'running') = (locked_by IS NOT NULL)),
  CONSTRAINT ck_jobs_done_at CHECK ((status = 'done') = (done_at IS NOT NULL))
);
-- Index phục vụ câu "lấy việc kế tiếp": chỉ việc đang chờ, theo thứ tự ưu tiên.
CREATE INDEX ix_jobs_queued ON jobs (priority, id) WHERE status = 'queued';

-- Mỗi lần một worker THỰC SỰ xử lý một việc (tác dụng phụ) ghi một dòng. Bất biến của
-- PG-036: mỗi việc đúng một dòng. Không UNIQUE (job_id) — cố ý, để lộ việc bị xử lý hai lần.
CREATE TABLE job_runs (
  id      bigint      GENERATED ALWAYS AS IDENTITY,
  job_id  bigint      NOT NULL,
  worker  text        NOT NULL,
  ran_at  timestamptz NOT NULL DEFAULT clock_timestamp(),
  CONSTRAINT pk_job_runs PRIMARY KEY (id),
  CONSTRAINT fk_job_runs_job FOREIGN KEY (job_id) REFERENCES jobs (id)
);
CREATE INDEX ix_job_runs_job ON job_runs (job_id);

-- ---------------------------------------------------------------------------
-- Idempotency key (PG-039): app khách gửi kèm mỗi request "đặt chuyến" một khoá ngẫu nhiên;
-- gửi lại (mạng chập chờn, bấm hai lần) dùng LẠI khoá cũ ⇒ server trả lại kết quả lần đầu.
CREATE TABLE idempotency_keys (
  key          text        NOT NULL,
  customer_id  uuid        NOT NULL,
  request_hash text        NOT NULL,           -- md5 nội dung request, để phát hiện khoá dùng lại cho request KHÁC
  trip_id      uuid,
  response     jsonb,
  created_at   timestamptz NOT NULL DEFAULT now(),
  completed_at timestamptz,
  CONSTRAINT pk_idempotency_keys PRIMARY KEY (key),
  CONSTRAINT fk_idempotency_keys_customer FOREIGN KEY (customer_id) REFERENCES customers (id),
  CONSTRAINT fk_idempotency_keys_trip FOREIGN KEY (trip_id) REFERENCES trips (id),
  CONSTRAINT ck_idempotency_keys_format CHECK (key ~ '^[A-Za-z0-9-]{8,64}$'),
  CONSTRAINT ck_idempotency_keys_done CHECK ((completed_at IS NULL) = (response IS NULL))
);

-- Transactional outbox: sự kiện ghi CÙNG giao dịch với dữ liệu nghiệp vụ.
CREATE TABLE outbox (
  id               bigint      GENERATED ALWAYS AS IDENTITY,
  aggregate_type   text        NOT NULL,
  aggregate_id     uuid        NOT NULL,
  event_type       text        NOT NULL,
  payload          jsonb       NOT NULL,
  created_at       timestamptz NOT NULL DEFAULT now(),
  published_at     timestamptz,
  publish_attempts integer     NOT NULL DEFAULT 0,
  CONSTRAINT pk_outbox PRIMARY KEY (id),
  CONSTRAINT ck_outbox_payload_object CHECK (jsonb_typeof(payload) = 'object')
);
CREATE INDEX ix_outbox_unpublished ON outbox (id) WHERE published_at IS NULL;

-- GIẢ LẬP message broker (một hệ thống KHÁC — Kafka, RabbitMQ…): "gửi" = chèn một dòng.
-- Trong lab broker nằm cùng database nhưng publisher ghi vào nó bằng KẾT NỐI RIÊNG, ngoài
-- giao dịch đánh dấu outbox — đúng như gửi qua mạng: không có commit chung hai hệ.
-- KHÔNG UNIQUE (outbox_id): broker nhận trùng được (giao hàng ít nhất một lần).
CREATE TABLE broker_messages (
  seq        bigint      GENERATED ALWAYS AS IDENTITY,
  outbox_id  bigint      NOT NULL,
  event_type text        NOT NULL,
  payload    jsonb       NOT NULL,
  sent_by    text        NOT NULL,
  sent_at    timestamptz NOT NULL DEFAULT clock_timestamp(),
  CONSTRAINT pk_broker_messages PRIMARY KEY (seq)
);

-- Consumer chống trùng: đã xử lý (consumer, outbox_id) thì bỏ qua lần nhận sau.
CREATE TABLE processed_messages (
  consumer     text        NOT NULL,
  outbox_id    bigint      NOT NULL,
  processed_at timestamptz NOT NULL DEFAULT clock_timestamp(),
  CONSTRAINT pk_processed_messages PRIMARY KEY (consumer, outbox_id)
);

-- Tác dụng của consumer "gửi thông báo cho khách": mỗi sự kiện ⇒ đúng một thông báo.
CREATE TABLE notifications (
  id          bigint      GENERATED ALWAYS AS IDENTITY,
  outbox_id   bigint      NOT NULL,
  customer_id uuid        NOT NULL,
  body        text        NOT NULL,
  created_at  timestamptz NOT NULL DEFAULT clock_timestamp(),
  CONSTRAINT pk_notifications PRIMARY KEY (id),
  CONSTRAINT fk_notifications_customer FOREIGN KEY (customer_id) REFERENCES customers (id)
);
CREATE INDEX ix_notifications_outbox ON notifications (outbox_id);

-- ---------------------------------------------------------------------------
-- Tiện ích cho đầu ra TẤT ĐỊNH của lab (không phải mẫu cho production).
--
-- Số giao dịch (xid) tăng dần trên CẢ cluster, nên số tuyệt đối đổi giữa các lần chạy.
-- Mỗi tệp bài gọi lab_dat_moc_xid() ở đầu: lưu xid của chính lệnh đó làm mốc X; sau đó
-- xid_td(x) in "X+1", "X+2"… — giao dịch ghi đầu tiên sau mốc là X+1. Giao dịch CHỈ ĐỌC
-- không được cấp xid nên không làm lệch thứ tự.
CREATE TABLE lab_moc_xid (
  id boolean NOT NULL DEFAULT true,
  x0 xid8    NOT NULL,
  CONSTRAINT pk_lab_moc_xid PRIMARY KEY (id),
  CONSTRAINT ck_lab_moc_xid_one CHECK (id)
);

CREATE FUNCTION lab_dat_moc_xid() RETURNS text
LANGUAGE sql VOLATILE
AS $$
  INSERT INTO toinoi.lab_moc_xid (id, x0) VALUES (true, pg_current_xact_id())
  ON CONFLICT (id) DO UPDATE SET x0 = excluded.x0
  RETURNING 'mốc X = xid của lệnh này'
$$;

-- xid8 (64 bit, có epoch) → "X+n". Cluster tạm mới ⇒ epoch 0. xid NHỎ HƠN mốc (dữ liệu nạp
-- từ fixture, giao dịch của tệp khác) in là "<X" — số tuyệt đối của chúng phụ thuộc thứ tự
-- chạy các tệp nên không in.
CREATE FUNCTION xid_td(x xid8) RETURNS text
LANGUAGE sql STABLE
AS $$
  SELECT CASE
    WHEN x IS NULL THEN NULL
    WHEN x::text::bigint = 0 THEN '0'
    ELSE (SELECT CASE WHEN d > 0 THEN 'X+' || d WHEN d = 0 THEN 'X' ELSE '<X' END
          FROM (SELECT x::text::bigint - (SELECT x0::text::bigint FROM toinoi.lab_moc_xid) AS d) AS t)
  END
$$;

-- Ngược lại: xid8 của "X+n" (để hỏi pg_xact_status của giao dịch phiên khác).
CREATE FUNCTION xid_x(n integer) RETURNS xid8
LANGUAGE sql STABLE
AS $$
  SELECT ((SELECT x0::text::bigint FROM toinoi.lab_moc_xid) + n)::text::xid8
$$;

-- xid (32 bit, cột hệ thống xmin/xmax) → "X+n". 0 = "không có" (vd. xmax = 0: chưa ai xoá/sửa).
CREATE FUNCTION xid_td(x xid) RETURNS text
LANGUAGE sql STABLE
AS $$
  SELECT CASE
    WHEN x IS NULL THEN NULL
    WHEN x::text::bigint = 0 THEN '0'
    ELSE (SELECT CASE WHEN d > 0 THEN 'X+' || d WHEN d = 0 THEN 'X' ELSE '<X' END
          FROM (SELECT x::text::bigint - (SELECT x0::text::bigint % 4294967296 FROM toinoi.lab_moc_xid) AS d) AS t)
  END
$$;

-- Snapshot dạng tương đối: xmin (mọi xid < xmin đã kết thúc), xmax (= xid lớn nhất đã kết thúc + 1;
-- mọi xid ≥ xmax coi như CHƯA kết thúc khi chụp), danh sách đang chạy lúc chụp (xip).
CREATE FUNCTION snap_td(s pg_snapshot) RETURNS text
LANGUAGE sql STABLE
AS $$
  SELECT 'xmin=' || toinoi.xid_td(pg_snapshot_xmin(s))
      || ' xmax=' || toinoi.xid_td(pg_snapshot_xmax(s))
      || ' đang chạy={' || coalesce((SELECT string_agg(toinoi.xid_td(x), ',' ORDER BY x::text::bigint)
                                       FROM pg_snapshot_xip(s) AS x), '') || '}'
$$;

-- Nhãn phiên của một backend: runner đặt application_name = 'lab-<NHÃN>' cho mỗi phiên.
CREATE FUNCTION lab_nhan(p integer) RETURNS text
LANGUAGE sql STABLE
AS $$
  SELECT coalesce((SELECT CASE WHEN application_name LIKE 'lab-%' THEN substr(application_name, 5)
                               ELSE '?' END
                   FROM pg_stat_activity WHERE pid = p), '?')
$$;

-- Khoá đang giữ / đang chờ của các phiên lab (trừ phiên đang hỏi), dạng đọc được:
-- relation (chỉ BẢNG trong schema toinoi — bỏ khoá trên index để bảng kết quả gọn), tuple,
-- transactionid (kèm "xid của phiên nào"), advisory. Bỏ virtualxid (mỗi phiên luôn giữ khoá
-- trên chính mã giao dịch ảo của mình) và khoá trên catalog hệ thống. Sắp cố định.
CREATE FUNCTION lab_khoa()
RETURNS TABLE (phien text, loai text, doi_tuong text, che_do text, da_cap boolean)
LANGUAGE sql STABLE
AS $$
  SELECT toinoi.lab_nhan(l.pid),
         l.locktype,
         CASE l.locktype
           WHEN 'relation' THEN c.relname
           WHEN 'tuple' THEN c.relname || ' (' || l.page || ',' || l.tuple || ')'
           WHEN 'transactionid' THEN 'xid của ' || coalesce(
             (SELECT string_agg(toinoi.lab_nhan(a.pid), ',') FROM pg_stat_activity AS a
              WHERE a.backend_xid = l.transactionid), '?')
           WHEN 'advisory' THEN 'advisory ' || l.objid
           ELSE l.locktype
         END,
         l.mode,
         l.granted
  FROM pg_locks AS l
  LEFT JOIN pg_class AS c ON c.oid = l.relation
  LEFT JOIN pg_namespace AS n ON n.oid = c.relnamespace
  WHERE l.pid <> pg_backend_pid()
    AND l.locktype IN ('relation', 'tuple', 'transactionid', 'advisory')
    AND (l.locktype NOT IN ('relation', 'tuple') OR (n.nspname = 'toinoi' AND c.relkind IN ('r', 'p')))
    AND toinoi.lab_nhan(l.pid) NOT IN ('?', 'quan-sat')
  ORDER BY 1, 2, 3, 4
$$;

-- Ai đang chờ ai: trạng thái từng phiên lab và các phiên đang chặn nó (pg_blocking_pids).
CREATE FUNCTION lab_cho()
RETURNS TABLE (phien text, trang_thai text, cho text, bi_chan_boi text, cau_lenh text)
LANGUAGE sql STABLE
AS $$
  SELECT toinoi.lab_nhan(a.pid),
         a.state,
         coalesce(a.wait_event_type || ': ' || a.wait_event, ''),
         coalesce((SELECT string_agg(toinoi.lab_nhan(b), ',' ORDER BY toinoi.lab_nhan(b))
                   FROM unnest(pg_blocking_pids(a.pid)) AS b), ''),
         left(regexp_replace(a.query, '\s+', ' ', 'g'), 60)
  FROM pg_stat_activity AS a
  WHERE a.application_name LIKE 'lab-%'
    AND a.application_name NOT IN ('lab-quan-sat', 'lab-admin')
    AND a.pid <> pg_backend_pid()
    AND a.datname = current_database()
  ORDER BY 1
$$;

-- Danh sách bảng thêm ở P04.
-- expect-line: broker_messages
-- expect-line: driver_duty
-- expect-line: idempotency_keys
-- expect-line: job_runs
-- expect-line: jobs
-- expect-line: lab_moc_xid
-- expect-line: notifications
-- expect-line: outbox
-- expect-line: processed_messages
-- expect-line: trip_assignments
-- expect-line: trip_claims
-- expect-line: trip_offers
-- expect-line: wallets
SELECT table_name
FROM information_schema.tables
WHERE table_schema = 'toinoi' AND table_type = 'BASE TABLE'
  AND table_name NOT IN ('areas', 'cities', 'customers', 'drivers', 'merchants', 'payments', 'trip_events', 'trip_ratings', 'trips')
ORDER BY table_name COLLATE "C";
