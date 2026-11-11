-- Thí nghiệm PG-039 (lời giải bài tập) · `postgresql-idempotency-outbox-transaction`
--
-- Bài 1: "cùng request" được so bằng md5 của jsonb — thứ tự khoá và khoảng trắng không làm
--        đổi hash, nhưng 42000 và 42000.0 là hai request KHÁC nhau.
-- Bài 2: thiết kế HAI giao dịch ("chèn khoá, COMMIT, rồi mới tạo chuyến") — server chết giữa
--        hai bước ⇒ khoá mồ côi: mọi lần gửi lại đều nhận NULL, không ai tạo chuyến.
-- Bài 3: hai consumer cùng nhận một tin. Consumer thứ hai CHỜ ở khoá chính processed_messages;
--        consumer đầu chết ⇒ consumer sau tự xử lý; consumer đầu commit ⇒ consumer sau bỏ qua.
-- Hàm dat_chuyen() và xu_ly_tin() chép NGUYÊN VĂN từ p04/39-idempotency-outbox.sql §2, §6.

-- @A
SET search_path TO toinoi, public;

-- @B
SET search_path TO toinoi, public;

-- @C
SET search_path TO toinoi, public;

-- @C
CREATE FUNCTION dat_chuyen(p_key text, p_customer uuid, p_req jsonb) RETURNS jsonb
LANGUAGE plpgsql AS $$
DECLARE
  v_hash text := md5(p_req::text);
  v_key  idempotency_keys%ROWTYPE;
  v_trip trips%ROWTYPE;
BEGIN
  INSERT INTO idempotency_keys (key, customer_id, request_hash)
  VALUES (p_key, p_customer, v_hash)
  ON CONFLICT (key) DO NOTHING;
  IF NOT FOUND THEN
    -- Khoá đã có (và đã commit — nếu đang có giao dịch khác giữ khoá, INSERT ở trên đã CHỜ nó).
    SELECT * INTO v_key FROM idempotency_keys WHERE key = p_key;
    IF v_key.request_hash <> v_hash OR v_key.customer_id <> p_customer THEN
      RAISE EXCEPTION 'idempotency key % đã dùng cho một request khác', p_key
        USING HINT = 'Client phải sinh khoá MỚI cho request mới (HTTP 422)';
    END IF;
    RETURN v_key.response || jsonb_build_object('lap_lai', true);
  END IF;
  INSERT INTO trips (kind, city_id, customer_id, fare, pickup_area_id)
  VALUES (p_req->>'kind', 1, p_customer, (p_req->>'fare')::integer, (p_req->>'pickup_area_id')::integer)
  RETURNING * INTO v_trip;
  INSERT INTO trip_events (trip_id, status) VALUES (v_trip.id, 'requested');
  INSERT INTO outbox (aggregate_type, aggregate_id, event_type, payload)
  VALUES ('trip', v_trip.id, 'trip_requested',
          jsonb_build_object('code', v_trip.code, 'customer_id', p_customer));
  UPDATE idempotency_keys
  SET trip_id = v_trip.id, completed_at = now(),
      response = jsonb_build_object('trip_code', v_trip.code, 'status', v_trip.status)
  WHERE key = p_key;
  RETURN jsonb_build_object('trip_code', v_trip.code, 'status', v_trip.status, 'lap_lai', false);
END
$$;

-- ===========================================================================
-- Bài 1. Hash của request.

-- @A
-- Cùng nội dung, khác thứ tự khoá và khoảng trắng: md5 của TEXT khác, md5 của JSONB giống.
-- expect-line: f | t
SELECT md5('{"kind": "ride", "fare": 42000, "pickup_area_id": 111}')
         = md5('{"pickup_area_id":111,"fare":42000,"kind":"ride"}') AS cung_text,
       md5('{"kind": "ride", "fare": 42000, "pickup_area_id": 111}'::jsonb::text)
         = md5('{"pickup_area_id":111,"fare":42000,"kind":"ride"}'::jsonb::text) AS cung_jsonb;

-- @A
-- expect-value: {"status": "requested", "lap_lai": false, "trip_code": 400200}
SELECT dat_chuyen('bt1-0001-aaaa', 'c4000000-0000-4000-8000-000000000001',
                  '{"kind": "ride", "fare": 42000, "pickup_area_id": 111}');

-- @A
-- Gửi lại, thư viện JSON phía client xếp khoá khác thứ tự ⇒ vẫn là "cùng request".
-- expect-value: {"status": "requested", "lap_lai": true, "trip_code": 400200}
SELECT dat_chuyen('bt1-0001-aaaa', 'c4000000-0000-4000-8000-000000000001',
                  '{"pickup_area_id":111,"fare":42000,"kind":"ride"}');

-- @A
-- jsonb giữ nguyên cách viết số (42000.0 ≠ 42000 khi in ra text) ⇒ hash khác.
-- expect-line: {"fare": 42000.0} | f
SELECT '{"fare": 42000.0}'::jsonb::text AS dang_text,
       md5('{"fare": 42000.0}'::jsonb::text) = md5('{"fare": 42000}'::jsonb::text) AS cung_hash;

-- @A
-- expect-error: P0001
SELECT dat_chuyen('bt1-0001-aaaa', 'c4000000-0000-4000-8000-000000000001',
                  '{"kind": "ride", "fare": 42000.0, "pickup_area_id": 111}');

-- ===========================================================================
-- Bài 2. Thiết kế HAI giao dịch: bước 1 chèn khoá rồi COMMIT ngay (autocommit), bước 2 tạo
-- chuyến. Server chết sau bước 1 ⇒ khoá còn, chuyến không có.

-- @A
-- expect-rows: 1
INSERT INTO idempotency_keys (key, customer_id, request_hash)
VALUES ('bt2-0002-bbbb', 'c4000000-0000-4000-8000-000000000002',
        md5('{"kind": "parcel", "fare": 35000, "pickup_area_id": 112}'::jsonb::text));

-- (… server chết ở đây, trước bước 2. Client gửi lại cùng khoá:)

-- @A
-- Khoá đã có, chưa có kết quả ⇒ response NULL ⇒ NULL || {...} = NULL. Gửi lại bao nhiêu lần cũng vậy.
-- expect-value: NULL
SELECT dat_chuyen('bt2-0002-bbbb', 'c4000000-0000-4000-8000-000000000002',
                  '{"kind": "parcel", "fare": 35000, "pickup_area_id": 112}');

-- @A
-- Khoá mồ côi: không chuyến, không kết quả. Chỉ dọn tay hoặc có trạng thái "đang xử lý" + thời hạn mới gỡ được.
-- expect-line: bt2-0002-bbbb | NULL | NULL | NULL
SELECT key, trip_id, completed_at, response FROM idempotency_keys WHERE key = 'bt2-0002-bbbb';

-- ===========================================================================
-- Bài 3. Hai consumer cùng nhận một tin (broker giao trùng). Tin seq 1, 2 cùng outbox_id 1;
-- tin seq 3, 4 cùng outbox_id 2.

-- @C
-- expect-rows: 4
INSERT INTO broker_messages (outbox_id, event_type, payload, sent_by)
SELECT o, 'trip_requested',
       jsonb_build_object('code', 400199 + o, 'customer_id', 'c4000000-0000-4000-8000-000000000003'),
       'publisher-' || p
FROM (VALUES (1, 'A'), (1, 'B'), (2, 'A'), (2, 'B')) AS v(o, p);

-- @C
CREATE FUNCTION xu_ly_tin(p_seq bigint) RETURNS text
LANGUAGE plpgsql AS $$
DECLARE
  m broker_messages%ROWTYPE;
BEGIN
  SELECT * INTO m FROM broker_messages WHERE seq = p_seq;
  INSERT INTO processed_messages (consumer, outbox_id) VALUES ('thong-bao', m.outbox_id)
  ON CONFLICT DO NOTHING;
  IF NOT FOUND THEN
    RETURN 'bỏ qua (đã xử lý)';
  END IF;
  INSERT INTO notifications (outbox_id, customer_id, body)
  VALUES (m.outbox_id, (m.payload->>'customer_id')::uuid, 'Đã nhận yêu cầu chuyến ' || (m.payload->>'code'));
  RETURN 'đã gửi thông báo';
END
$$;

-- (3a) Consumer A xử lý tin 1 (chưa commit) rồi CHẾT; consumer B nhận bản trùng (tin 2).

-- @A
BEGIN;

-- @A
-- expect-value: đã gửi thông báo
SELECT xu_ly_tin(1);

-- @B
-- B chèn cùng (consumer, outbox_id) ⇒ chờ giao dịch của A.
-- expect-blocked
-- expect-value: đã gửi thông báo
SELECT xu_ly_tin(2);

-- @C
-- expect-value: t
SELECT pg_terminate_backend(pid, 5000) FROM pg_stat_activity WHERE application_name = 'lab-A';

-- (A chết ⇒ dấu "đã xử lý" và thông báo của A cùng biến mất ⇒ B tự xử lý — dòng "✔" ở trên.)

-- @A
-- expect-mat-ket-noi
COMMIT;

-- (3b) Consumer A xử lý tin 3 và COMMIT; consumer B nhận bản trùng (tin 4) trong lúc A chưa commit.

-- @A
SET search_path TO toinoi, public;

-- @A
BEGIN;

-- @A
-- expect-value: đã gửi thông báo
SELECT xu_ly_tin(3);

-- @B
-- expect-blocked
-- expect-value: bỏ qua (đã xử lý)
SELECT xu_ly_tin(4);

-- @A
COMMIT;

-- @C
-- Mỗi sự kiện đúng một thông báo trong cả hai trường hợp.
-- expect-line: 1 | 1
-- expect-line: 2 | 1
SELECT outbox_id, count(*) FROM notifications GROUP BY outbox_id ORDER BY outbox_id;
