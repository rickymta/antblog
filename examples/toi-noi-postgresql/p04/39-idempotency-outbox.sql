-- PG-039 · Idempotency và transactional outbox — `postgresql-idempotency-outbox-transaction`
--
-- Bài toán Tới Nơi (1): app khách bấm "Đặt chuyến", mạng chập chờn ⇒ app GỬI LẠI request.
-- Không được tạo hai chuyến. App gửi kèm một khoá ngẫu nhiên (idempotency key) — gửi lại thì
-- dùng LẠI khoá cũ; server ghi khoá vào bảng có khoá chính, cùng giao dịch với chuyến:
--   * lần đầu: INSERT khoá thành công ⇒ tạo chuyến, lưu kết quả vào khoá, COMMIT;
--   * lần lặp: INSERT … ON CONFLICT DO NOTHING không chèn được ⇒ trả lại kết quả đã lưu;
--   * hai request cùng khoá ĐỒNG THỜI: request sau chờ ở index duy nhất tới khi request trước
--     commit (⇒ trả kết quả đã lưu) hoặc huỷ (⇒ tự làm).
-- Bài toán (2): tạo chuyến xong phải báo cho dịch vụ thông báo qua message broker. Gửi broker
-- trong giao dịch thì broker nhận tin của giao dịch sau đó bị huỷ; gửi sau COMMIT thì app chết
-- giữa chừng là mất tin. Outbox: ghi sự kiện vào bảng `outbox` CÙNG giao dịch với chuyến; một
-- publisher đọc outbox, gửi broker rồi đánh dấu đã gửi. Publisher chết giữa "gửi" và "đánh
-- dấu" ⇒ gửi lại ⇒ broker có TIN TRÙNG (ít nhất một lần) ⇒ consumer phải chống trùng.
--
-- Phiên: [A], [B] = hai request / hai publisher; [C] = quan sát, dọn dẹp, consumer;
-- [D] = kết nối RIÊNG tới "broker" (bảng broker_messages, giả lập hệ thống khác).

-- @A
SET search_path TO toinoi, public;

-- @B
SET search_path TO toinoi, public;

-- @C
SET search_path TO toinoi, public;

-- @D
SET search_path TO toinoi, public;

-- ===========================================================================
-- §1. Từng bước, một phiên: lần đầu và lần lặp của cùng một khoá.

-- @A
BEGIN;

-- @A
-- Lần đầu: khoá chưa có ⇒ chèn được (1 dòng trả về).
-- expect-value: dat-0001-aaaa
INSERT INTO idempotency_keys (key, customer_id, request_hash)
VALUES ('dat-0001-aaaa', 'c4000000-0000-4000-8000-000000000001',
        md5('{"fare": 42000, "kind": "ride", "pickup_area_id": 111}'::jsonb::text))
ON CONFLICT (key) DO NOTHING
RETURNING key;

-- @A
INSERT INTO trips (kind, city_id, customer_id, fare, pickup_area_id)
VALUES ('ride', 1, 'c4000000-0000-4000-8000-000000000001', 42000, 111)
RETURNING id AS trip_id, code AS trip_code \gset

-- @A
INSERT INTO trip_events (trip_id, status) VALUES (:'trip_id', 'requested');

-- @A
-- Sự kiện cho hệ thống khác: CÙNG giao dịch ⇒ có chuyến thì có sự kiện, và ngược lại.
-- expect-value: 1
INSERT INTO outbox (aggregate_type, aggregate_id, event_type, payload)
VALUES ('trip', :'trip_id', 'trip_requested',
        jsonb_build_object('code', :trip_code, 'customer_id', 'c4000000-0000-4000-8000-000000000001'))
RETURNING id;

-- @A
-- Lưu KẾT QUẢ trả cho client vào khoá — lần lặp sẽ trả lại đúng nội dung này.
-- expect-line: {"status": "requested", "trip_code": 400200}
UPDATE idempotency_keys
SET trip_id = :'trip_id', completed_at = now(),
    response = jsonb_build_object('trip_code', :trip_code, 'status', 'requested')
WHERE key = 'dat-0001-aaaa'
RETURNING response;

-- @A
COMMIT;

-- App không nhận được phản hồi (mạng rớt) ⇒ gửi LẠI, cùng khoá.
-- @A
BEGIN;

-- @A
-- Khoá đã có ⇒ không chèn ⇒ 0 dòng.
-- expect-rows: 0
INSERT INTO idempotency_keys (key, customer_id, request_hash)
VALUES ('dat-0001-aaaa', 'c4000000-0000-4000-8000-000000000001',
        md5('{"fare": 42000, "kind": "ride", "pickup_area_id": 111}'::jsonb::text))
ON CONFLICT (key) DO NOTHING
RETURNING key;

-- @A
-- ⇒ trả lại kết quả lần đầu. Kiểm cùng nội dung request (hash) để phát hiện khoá bị dùng lại
-- cho một request KHÁC (lỗi phía client ⇒ trả 422, không âm thầm trả kết quả cũ).
-- expect-line: {"status": "requested", "trip_code": 400200} | t
SELECT response, request_hash = md5('{"fare": 42000, "kind": "ride", "pickup_area_id": 111}'::jsonb::text) AS cung_noi_dung
FROM idempotency_keys WHERE key = 'dat-0001-aaaa';

-- @A
COMMIT;

-- ===========================================================================
-- §2. Gói thành một hàm (server API gọi một lần mỗi request).

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

-- @A
-- expect-value: {"status": "requested", "lap_lai": false, "trip_code": 400201}
SELECT dat_chuyen('dat-0002-bbbb', 'c4000000-0000-4000-8000-000000000002',
                  '{"kind": "parcel", "fare": 35000, "pickup_area_id": 112}');

-- @A
-- Gửi lại: cùng kết quả, đánh dấu lap_lai.
-- expect-value: {"status": "requested", "lap_lai": true, "trip_code": 400201}
SELECT dat_chuyen('dat-0002-bbbb', 'c4000000-0000-4000-8000-000000000002',
                  '{"kind": "parcel", "fare": 35000, "pickup_area_id": 112}');

-- @A
-- Cùng khoá, KHÁC nội dung ⇒ lỗi (P0001 = RAISE EXCEPTION của PL/pgSQL).
-- expect-error: P0001
SELECT dat_chuyen('dat-0002-bbbb', 'c4000000-0000-4000-8000-000000000002',
                  '{"kind": "parcel", "fare": 99000, "pickup_area_id": 112}');

-- ===========================================================================
-- §3. Hai request CÙNG KHOÁ đồng thời (bấm hai lần thật nhanh, hai server khác nhau xử lý).

-- @A
BEGIN;

-- @A
-- expect-value: {"status": "requested", "lap_lai": false, "trip_code": 400202}
SELECT dat_chuyen('dat-0003-cccc', 'c4000000-0000-4000-8000-000000000003',
                  '{"kind": "ride", "fare": 51000, "pickup_area_id": 113}');

-- @B
-- B chèn cùng khoá ⇒ CHỜ ở index duy nhất pk_idempotency_keys tới khi A kết thúc.
-- expect-blocked
-- expect-value: {"status": "requested", "lap_lai": true, "trip_code": 400202}
SELECT dat_chuyen('dat-0003-cccc', 'c4000000-0000-4000-8000-000000000003',
                  '{"kind": "ride", "fare": 51000, "pickup_area_id": 113}');

-- @A
COMMIT;

-- (A commit ⇒ B thấy khoá đã có ⇒ trả kết quả của A, lap_lai = true — dòng "✔" ở trên.)

-- ===========================================================================
-- §4. Request đầu CHẾT giữa chừng (huỷ phiên trước COMMIT) trong lúc request lặp đang chờ.

-- @A
BEGIN;

-- @A
-- A đã tạo chuyến (mã 400203), sự kiện, outbox — tất cả CHƯA commit.
-- expect-value: {"status": "requested", "lap_lai": false, "trip_code": 400203}
SELECT dat_chuyen('dat-0004-dddd', 'c4000000-0000-4000-8000-000000000004',
                  '{"kind": "ride", "fare": 47000, "pickup_area_id": 111}');

-- @B
-- expect-blocked
-- expect-value: {"status": "requested", "lap_lai": false, "trip_code": 400204}
SELECT dat_chuyen('dat-0004-dddd', 'c4000000-0000-4000-8000-000000000004',
                  '{"kind": "ride", "fare": 47000, "pickup_area_id": 111}');

-- @C
-- Server của A sập (ở đây: huỷ backend của A).
-- expect-value: t
SELECT pg_terminate_backend(pid, 5000) FROM pg_stat_activity WHERE application_name = 'lab-A';

-- (Giao dịch của A bị huỷ ⇒ khoá idempotency của A biến mất ⇒ B chèn được và TỰ tạo chuyến
--  400204 — dòng "✔" ở trên. Mã 400203 bị đốt; chuyến/sự kiện/outbox của A không còn gì.)

-- @A
-- expect-mat-ket-noi
COMMIT;

-- @C
-- Mỗi khoá đúng một chuyến; mọi chuyến của bài có đúng một sự kiện và một tin outbox.
-- expect-line: dat-0001-aaaa | 400200 | 1 | 1
-- expect-line: dat-0002-bbbb | 400201 | 1 | 1
-- expect-line: dat-0003-cccc | 400202 | 1 | 1
-- expect-line: dat-0004-dddd | 400204 | 1 | 1
SELECT k.key, t.code,
       (SELECT count(*) FROM trip_events AS e WHERE e.trip_id = t.id) AS su_kien,
       (SELECT count(*) FROM outbox AS o WHERE o.aggregate_id = t.id) AS outbox
FROM idempotency_keys AS k JOIN trips AS t ON t.id = k.trip_id
ORDER BY k.key;

-- @C
-- Không có chuyến "nửa vời" của A (mã 400203).
-- expect-value: 0
SELECT count(*) FROM trips WHERE code = 400203;

-- ===========================================================================
-- §5. Publisher đọc outbox (SKIP LOCKED — nhiều publisher song song không lấy trùng), gửi
-- broker bằng kết nối riêng [D], rồi đánh dấu đã gửi. Giao hàng "ít nhất một lần".

-- @A
SET search_path TO toinoi, public;

-- @A
BEGIN;

-- @A
-- Publisher A lấy 2 tin đầu (id outbox 1, 2, 3, 5 — id 4 là của giao dịch đã huỷ ở §4).
-- expect-line: 1
-- expect-line: 2
SELECT id FROM outbox WHERE published_at IS NULL ORDER BY id LIMIT 2 FOR UPDATE SKIP LOCKED;

-- @D
-- "Gửi" 2 tin sang broker — broker là hệ thống khác, đã nhận là nhận (không rollback được).
-- expect-rows: 2
INSERT INTO broker_messages (outbox_id, event_type, payload, sent_by)
SELECT id, event_type, payload, 'publisher-A' FROM outbox WHERE id IN (1, 2) ORDER BY id;

-- @C
-- Publisher A chết TRƯỚC khi kịp đánh dấu published_at.
-- expect-value: t
SELECT pg_terminate_backend(pid, 5000) FROM pg_stat_activity WHERE application_name = 'lab-A';

-- @B
BEGIN;

-- @B
-- Publisher B: tin 1, 2 vẫn "chưa gửi" (giao dịch đánh dấu của A chưa từng commit).
-- expect-line: 1
-- expect-line: 2
-- expect-line: 3
-- expect-line: 5
SELECT id FROM outbox WHERE published_at IS NULL ORDER BY id LIMIT 10 FOR UPDATE SKIP LOCKED;

-- @D
-- expect-rows: 4
INSERT INTO broker_messages (outbox_id, event_type, payload, sent_by)
SELECT id, event_type, payload, 'publisher-B' FROM outbox WHERE id IN (1, 2, 3, 5) ORDER BY id;

-- @B
-- expect-rows: 4
UPDATE outbox SET published_at = now(), publish_attempts = publish_attempts + 1
WHERE id IN (1, 2, 3, 5);

-- @B
COMMIT;

-- @C
-- Broker nhận 6 tin cho 4 sự kiện: tin 1 và 2 bị gửi HAI lần.
-- expect-line: 1 | 2 | publisher-A,publisher-B
-- expect-line: 2 | 2 | publisher-A,publisher-B
-- expect-line: 3 | 1 | publisher-B
-- expect-line: 5 | 1 | publisher-B
SELECT outbox_id, count(*), string_agg(sent_by, ',' ORDER BY seq)
FROM broker_messages GROUP BY outbox_id ORDER BY outbox_id;

-- ===========================================================================
-- §6. Consumer "gửi thông báo cho khách" chống trùng: ghi (consumer, outbox_id) vào bảng
-- processed_messages (khoá chính) CÙNG giao dịch với tác dụng; chèn không được ⇒ đã xử lý
-- ⇒ bỏ qua. (Ở đây xử lý 6 tin trong một câu cho gọn; consumer thật commit từng tin.)

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

-- @C
-- expect-line: 1 | 1 | đã gửi thông báo
-- expect-line: 2 | 2 | đã gửi thông báo
-- expect-line: 3 | 1 | bỏ qua (đã xử lý)
-- expect-line: 4 | 2 | bỏ qua (đã xử lý)
-- expect-line: 5 | 3 | đã gửi thông báo
-- expect-line: 6 | 5 | đã gửi thông báo
SELECT seq, outbox_id, xu_ly_tin(seq) FROM broker_messages ORDER BY seq;

-- @C
-- Bất biến: mỗi sự kiện đúng MỘT thông báo dù broker giao trùng.
-- expect-line: 1 | 1
-- expect-line: 2 | 1
-- expect-line: 3 | 1
-- expect-line: 5 | 1
SELECT outbox_id, count(*) FROM notifications GROUP BY outbox_id ORDER BY outbox_id;
