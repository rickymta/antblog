---
slug: postgresql-idempotency-outbox-transaction
title: "Idempotency và transactional outbox: gửi lại mà không tạo trùng"
summary: "Khách Tới Nơi bấm «Đặt chuyến», mạng rớt, app gửi lại: không được thành hai chuyến; tạo chuyến xong phải báo dịch vụ thông báo, không mất tin, không báo trùng. Idempotency key và outbox trên PostgreSQL 18, kiểm bằng lab."
category: postgresql
level: advanced
tags: [postgresql, pg-039, idempotency, outbox, on-conflict, at-least-once, giao-dich, toi-noi]
language: vi
---

Khách Tới Nơi bấm «Đặt chuyến» lúc đang trong thang máy. Request đi tới server, server tạo chuyến, nhưng phản hồi không về được app. Sau vài giây app báo lỗi mạng và **tự gửi lại**. Nếu server coi đó là request mới, khách có hai chuyến, hai tài xế chạy tới, và có thể bị trừ tiền hai lần.

Việc thứ hai cũng nằm ở cùng chỗ: tạo chuyến xong, server phải báo cho dịch vụ thông báo qua message broker (hệ thống trung chuyển tin giữa các service) để gửi «Đã nhận yêu cầu chuyến» cho khách. Gửi broker trước COMMIT thì broker có thể nhận tin của một chuyến bị huỷ sau đó. Gửi sau COMMIT thì server chết giữa chừng là mất tin.

Bài này giải hai việc bằng hai mẫu chạy trên PostgreSQL: **idempotency key** cho request lặp và **transactional outbox** cho sự kiện, rồi kiểm từng chỗ có thể hỏng: request trùng đồng thời, request chết giữa chừng, publisher chết giữa «gửi» và «đánh dấu», consumer nhận tin trùng.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-039 · Phần 4/6 · Nâng cao</span>
<p>Đọc trước: <a href="/articles/postgresql-for-update-skip-locked">PG-036 · FOR UPDATE, NOWAIT và SKIP LOCKED</a> và <a href="/articles/postgresql-upsert-on-conflict">PG-017 · UPSERT và xung đột khoá duy nhất</a>. Lab: <code>examples/toi-noi-postgresql/p04</code> (tệp <code>39-idempotency-outbox.sql</code>, phép thử K039 ở <code>DO-DAC.md</code>) và thí nghiệm <code>p04/thu-nghiem/pg039-idempotency-outbox-bai-tap.sql</code>. Sau bài này, bạn kiểm được request lặp, lỗi giữa chừng và xử lý sự kiện trùng.</p>
</div>

## 1. Ba kết cục mà client nhìn thấy

Từ phía app, một request «đặt chuyến» có ba kết cục:

| Client thấy | Server thực sự | Gửi lại có an toàn? |
|---|---|---|
| Phản hồi thành công | Đã COMMIT | Không cần gửi |
| Lỗi rõ ràng trước COMMIT (422, 23514…) | Giao dịch đã huỷ | An toàn — chưa có gì |
| Timeout, «server closed the connection unexpectedly» | **Không biết** | Chỉ an toàn nếu server nhận ra request lặp |

Dòng thứ ba là vấn đề. [PG-031](/articles/postgresql-transaction-commit-rollback) đã tái hiện một nửa của nó: phiên bị `pg_terminate_backend` **trước** khi COMMIT tới server, client nhận «MẤT KẾT NỐI», và chuyến không được lưu. Nửa còn lại — COMMIT **đã** tới server, server đã ghi commit, nhưng phản hồi bị mất trên đường về — cho client đúng cùng một lỗi mạng.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chưa tái hiện được — lập luận</span>
<p>Lab chưa dựng được tình huống «COMMIT tới server nhưng client không nhận phản hồi»: phải cắt mạng đúng sau khi server ghi commit và trước khi phản hồi rời server, runner hiện không làm tất định được. Phần này là lập luận, không phải kết quả đo.</p>
</div>

Lập luận: giao dịch là nguyên tử ở phía **server**, nhưng việc **client biết** kết quả là một thông điệp mạng riêng đi sau. Không có giao dịch nào bao được cả hai. Vì vậy client chỉ có hai lựa chọn khi gặp lỗi mạng: không gửi lại (có thể mất chuyến) hoặc gửi lại (có thể trùng). Muốn gửi lại an toàn, server phải nhận ra «request này tôi đã làm rồi» — đó là **idempotency** (luỹ đẳng: làm một lần hay nhiều lần cho cùng kết quả, đã gặp ở PG-017 với lô nhập thực đơn).

PostgreSQL có một công cụ hẹp hơn cho đúng tình huống này: `pg_xact_status(xid)`. Tài liệu (mục System Information Functions) nêu ví dụ dùng hàm để biết giao dịch đã commit hay huỷ sau khi app và server mất kết nối giữa lúc COMMIT; [PG-032](/articles/postgresql-mvcc-snapshot) đã dùng nó để đọc `in progress`/`committed`. Nhưng app phải lưu xid trước COMMIT, hàm chỉ trả lời cho giao dịch «đủ gần», và nó không giúp khi request được gửi lại tới một server khác. Idempotency key giải được cả những trường hợp đó.

## 2. Idempotency key: chèn khoá cùng giao dịch với chuyến

App sinh một chuỗi ngẫu nhiên cho **mỗi lần khách bấm** (không phải mỗi lần gửi HTTP) và gửi kèm request; gửi lại thì dùng lại khoá cũ. Server ghi khoá vào bảng có khoá chính (`p04/00-mo-rong-schema.sql`, rút gọn):

```sql
CREATE TABLE idempotency_keys (
  key          text        NOT NULL,
  customer_id  uuid        NOT NULL,
  request_hash text        NOT NULL,   -- md5 nội dung request
  trip_id      uuid,
  response     jsonb,                  -- kết quả trả cho client lần đầu
  completed_at timestamptz,
  CONSTRAINT pk_idempotency_keys PRIMARY KEY (key),
  CONSTRAINT ck_idempotency_keys_done CHECK ((completed_at IS NULL) = (response IS NULL))
);
```

Lần đầu (`39-idempotency-outbox.sql` §1), tất cả trong **một** giao dịch:

```sql
BEGIN;
INSERT INTO idempotency_keys (key, customer_id, request_hash)
VALUES ('dat-0001-aaaa', 'c4000000-0000-4000-8000-000000000001',
        md5('{"fare": 42000, "kind": "ride", "pickup_area_id": 111}'::jsonb::text))
ON CONFLICT (key) DO NOTHING
RETURNING key;                                  -- 1 dòng: dat-0001-aaaa
INSERT INTO trips (…) VALUES (…) RETURNING id AS trip_id, code AS trip_code \gset   -- 400200
INSERT INTO trip_events (trip_id, status) VALUES (:'trip_id', 'requested');
INSERT INTO outbox (…) VALUES ('trip', :'trip_id', 'trip_requested', …) RETURNING id;  -- 1
UPDATE idempotency_keys
SET trip_id = :'trip_id', completed_at = now(),
    response = jsonb_build_object('trip_code', :trip_code, 'status', 'requested')
WHERE key = 'dat-0001-aaaa'
RETURNING response;
COMMIT;
```

```text
                   response
----------------------------------------------
 {"status": "requested", "trip_code": 400200}
(1 row)
```

Phản hồi «mất trên đường về». App gửi lại, cùng khoá:

```text
 key
-----
(0 rows)

INSERT 0 0
                   response                   | cung_noi_dung
----------------------------------------------+---------------
 {"status": "requested", "trip_code": 400200} | t
```

`ON CONFLICT DO NOTHING` không chèn được (0 dòng) ⇒ server đọc và trả lại **đúng** phản hồi đã lưu, không tạo chuyến thứ hai. Cột `cung_noi_dung` so hash của request: khoá giống mà nội dung khác là lỗi phía client, phải báo lỗi chứ không lặng lẽ trả kết quả cũ.

Ba quyết định thiết kế ở đây đều có lý do:

- **Chèn khoá trước, không `SELECT` trước.** «Đọc xem có chưa rồi mới chèn» là đọc–rồi–ghi: hai request đồng thời cùng đọc thấy «chưa có» ([PG-034](/articles/postgresql-isolation-read-committed-repeatable-read)). Khoá chính mới là trọng tài.
- **Khoá, chuyến, sự kiện, outbox và kết quả cùng một giao dịch.** Có khoá thì có chuyến, và ngược lại. Mục 5 cho thấy vì sao điều này gỡ được ca khó nhất.
- **Lưu `response`, không chỉ lưu «đã làm».** Request lặp cần trả **cùng nội dung** (mã chuyến 400200), không phải một thông báo chung chung.

## 3. Gói thành một hàm

Server API gọi một hàm mỗi request (§2, nguyên văn):

```sql
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
```

Ba lần gọi với khoá `dat-0002-bbbb`: lần đầu, gửi lại cùng nội dung, rồi cùng khoá nhưng `fare` 99000:

```text
                           dat_chuyen
----------------------------------------------------------------
 {"status": "requested", "lap_lai": false, "trip_code": 400201}
…
 {"status": "requested", "lap_lai": true, "trip_code": 400201}
…
ERROR:  idempotency key dat-0002-bbbb đã dùng cho một request khác
HINT:  Client phải sinh khoá MỚI cho request mới (HTTP 422)
CONTEXT:  PL/pgSQL function dat_chuyen(text,uuid,jsonb) line 14 at RAISE
-- [lab] SQLSTATE P0001
```

`P0001` là mã mặc định của `RAISE EXCEPTION` khi không ghi mã riêng (tài liệu PL/pgSQL, mục Errors and Messages). Hash tính trên `jsonb` chứ không trên chuỗi gốc: `jsonb` không giữ khoảng trắng và thứ tự khoá, nên thư viện JSON phía client xếp khoá khác đi vẫn là «cùng request» (bài tập 1). Thứ tự khoá trong kết quả in ra (`status` trước `trip_code`) cũng do `jsonb` tự sắp, không phải do hàm.

## 4. Hai request cùng khoá chạy đồng thời

Khách bấm hai lần thật nhanh, hai server khác nhau nhận hai request (§3). Lịch chạy theo thứ tự xảy ra thật:

| Bước | A (request thứ nhất) | B (request thứ hai, cùng khoá) |
|---|---|---|
| T1 | `BEGIN` | |
| T2 | `dat_chuyen('dat-0003-cccc', …)` → `lap_lai: false`, `trip_code: 400202` (chưa commit) | |
| T3 | | `dat_chuyen('dat-0003-cccc', …)` ⏳ **Lock: transactionid — bị chặn bởi [A]** |
| T4 | `COMMIT` | |
| T5 | | ✔ `{"status": "requested", "lap_lai": true, "trip_code": 400202}` |

B **chờ**, không lỗi, không tạo chuyến thứ hai. Cơ chế nằm ở index duy nhất: chèn một khoá trùng với dòng do giao dịch **chưa commit** chèn thì phải chờ giao dịch đó kết thúc rồi kiểm lại (tài liệu, mục Index Uniqueness Checks). A commit ⇒ có xung đột ⇒ `DO NOTHING` ⇒ `NOT FOUND`. Câu `SELECT` kế tiếp trong hàm thấy được dòng của A vì hàm `VOLATILE` lấy snapshot mới cho mỗi câu (mục Function Volatility Categories), và ở Read Committed snapshot mới gồm giao dịch vừa commit.

Đây là bằng chứng hai phiên mà [PG-017](/articles/postgresql-upsert-on-conflict) phải hẹn sang Phần 4. Tài liệu về Read Committed ghi rõ `INSERT … ON CONFLICT DO NOTHING` có thể không chèn vì kết quả của một giao dịch khác mà snapshot của câu không thấy, và nói đó là hành vi của Read Committed. Lab chỉ chạy ở Read Committed; đừng giả định hàm này cư xử y hệt ở Repeatable Read hay Serializable khi chưa thử.

## 5. Request đầu chết giữa chừng

Ca khó nhất (§4): request đầu tạo xong mọi thứ nhưng server sập **trước COMMIT**, trong lúc request lặp đang chờ.

| Bước | A (request đầu) | B (gửi lại) | C (quan sát) |
|---|---|---|---|
| T1 | `BEGIN` | | |
| T2 | `dat_chuyen('dat-0004-dddd', …)` → `trip_code: 400203` (chưa commit) | | |
| T3 | | `dat_chuyen('dat-0004-dddd', …)` ⏳ bị chặn bởi [A] | |
| T4 | | | `pg_terminate_backend(A)` → `t` |
| T5 | | ✔ `{"status": "requested", "lap_lai": false, "trip_code": 400204}` | |
| T6 | `COMMIT` → ✖ **MẤT KẾT NỐI** | | |

A bị huỷ ⇒ khoá của A biến mất cùng chuyến, sự kiện và tin outbox ⇒ B chèn được khoá và **tự làm**. Không có khoá mồ côi để dọn, không có chuyến nửa vời:

```text
      key      |  code  | su_kien | outbox
---------------+--------+---------+--------
 dat-0001-aaaa | 400200 |       1 |      1
 dat-0002-bbbb | 400201 |       1 |      1
 dat-0003-cccc | 400202 |       1 |      1
 dat-0004-dddd | 400204 |       1 |      1
```

`SELECT count(*) FROM trips WHERE code = 400203` → `0`. Mã 400203 bị «đốt» vì sequence không quay lui khi ROLLBACK (PG-031).

So với thiết kế **hai giao dịch** — «chèn khoá, COMMIT, rồi mới tạo chuyến»: server chết giữa hai bước để lại khoá không có kết quả; với đúng hàm `dat_chuyen` ở trên, mọi lần gửi lại đều nhận `NULL` mãi mãi (bài tập 2). Thiết kế đó cần thêm trạng thái «đang xử lý» và thời hạn để người khác tiếp quản — lab không làm, và bạn chỉ nên chọn nó khi phần xử lý không nằm gọn trong một giao dịch database.

## 6. Transactional outbox: sự kiện cùng giao dịch, gửi sau

Gửi broker trực tiếp từ server là **ghi hai nơi** (dual write) không có giao dịch chung: broker đã nhận thì không rollback được. Outbox tách việc làm hai bước:

1. Ghi sự kiện vào bảng `outbox` **cùng giao dịch** với chuyến (đã thấy ở `dat_chuyen`). Có chuyến thì có sự kiện, chuyến bị huỷ thì sự kiện cũng mất.
2. Một **publisher** đọc các dòng `published_at IS NULL`, gửi broker, rồi đánh dấu `published_at`.

Bước 2 vẫn là hai nơi. Lab §5 cho publisher chết đúng giữa «gửi» và «đánh dấu». [D] là kết nối riêng tới «broker» (bảng `broker_messages` giả lập hệ thống khác, tự commit):

| Bước | A (publisher 1) | B (publisher 2) | D (broker) | C |
|---|---|---|---|---|
| T1 | `BEGIN`; `SELECT id FROM outbox WHERE published_at IS NULL ORDER BY id LIMIT 2 FOR UPDATE SKIP LOCKED` → 1, 2 | | | |
| T2 | | | nhận tin 1, 2 từ `publisher-A` (`INSERT 0 2`) | |
| T3 | | | | `pg_terminate_backend(A)` → `t` |
| T4 | | `BEGIN`; cùng câu, `LIMIT 10` → **1, 2, 3, 5** | | |
| T5 | | | nhận tin 1, 2, 3, 5 từ `publisher-B` (`INSERT 0 4`) | |
| T6 | | `UPDATE outbox SET published_at = now() … WHERE id IN (1, 2, 3, 5)` → `UPDATE 4`; `COMMIT` | | |

Dòng outbox id 4 không có: nó thuộc giao dịch đã huỷ ở mục 5 (identity cũng không quay lui). Broker cuối cùng:

```text
 outbox_id | count |       string_agg
-----------+-------+-------------------------
         1 |     2 | publisher-A,publisher-B
         2 |     2 | publisher-A,publisher-B
         3 |     1 | publisher-B
         5 |     1 | publisher-B
```

Không mất tin nào, nhưng tin 1 và 2 tới **hai lần**. Đây là bản chất của outbox: giao hàng **ít nhất một lần** (at-least-once). `FOR UPDATE SKIP LOCKED` cho nhiều publisher chạy song song mà không lấy trùng dòng **đang bị khoá** ([PG-036](/articles/postgresql-for-update-skip-locked)); nó không chống được trùng sau khi publisher chết, vì khoá dòng chết theo giao dịch.

## 7. Consumer chống trùng

Vì broker có thể giao trùng, phía nhận phải idempotent. Consumer «gửi thông báo cho khách» ghi `(consumer, outbox_id)` vào bảng có khoá chính `processed_messages`, **cùng giao dịch** với tác dụng (§6):

```sql
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
```

```text
 seq | outbox_id |     xu_ly_tin
-----+-----------+-------------------
   1 |         1 | đã gửi thông báo
   2 |         2 | đã gửi thông báo
   3 |         1 | bỏ qua (đã xử lý)
   4 |         2 | bỏ qua (đã xử lý)
   5 |         3 | đã gửi thông báo
   6 |         5 | đã gửi thông báo
```

Sáu tin cho bốn sự kiện ⇒ mỗi `outbox_id` đúng **1** dòng `notifications`. Cùng mẫu với idempotency key ở mục 2, chỉ đổi khoá: request dùng khoá do client sinh, consumer dùng id của sự kiện. Hai consumer nhận cùng một tin **đồng thời** cũng chờ nhau ở khoá chính như mục 4 — thí nghiệm ở bài tập 3.

## 8. Ở quy mô: phép thử K039

Lịch tất định chỉ ra cơ chế; phép thử K039 cho các phiên chạy đua thật (`p04/DO-DAC.md`). Máy: Intel Core i7-10700 (16 luồng), 64 GB RAM, NVMe Kingston SA2000, Windows 11 Pro, Node 22.18, PostgreSQL 18.1, tải CPU nền ~6 %; 8 phiên psql thật, 3 lần, mỗi lần một database mới. Tải: **400 khoá**, mỗi khoá **hai request đồng thời**; 10 % request mở giao dịch, gọi `dat_chuyen()` rồi bị giết tiến trình trước COMMIT và client gửi lại; 4 publisher lấy lô 5 bằng SKIP LOCKED, 10 % chết sau khi gửi trước khi đánh dấu; **hai consumer cùng nhận mọi tin**.

| Đại lượng (3 lần) | Min–max |
|---|---|
| Thời gian mỗi lần | 7,95–8,99 s |
| Request chết trước COMMIT | 78–80 |
| Tạo chuyến mới / trả lại kết quả cũ | 400 / 400 (cả 3 lần) |
| Publisher chết giữa gửi và đánh dấu | 9–15 |
| Tin gửi broker (tin trùng) | 445–475 (45–75) |
| Consumer xử lý / bỏ qua | 400 / 490–550 |

| Bất biến | Kết quả 3 lần |
|---|---|
| Khoá không có đúng 1 chuyến = 0 | 0 · 0 · 0 |
| Chuyến tạo ra = số khoá (400) | 400 · 400 · 400 |
| Chuyến thiếu/thừa sự kiện hoặc outbox = 0 | 0 · 0 · 0 |
| Outbox chưa gửi = 0 | 0 · 0 · 0 |
| Sự kiện không có đúng 1 thông báo = 0 | 0 · 0 · 0 |

Đọc đúng giới hạn: số tin trùng phụ thuộc tỉ lệ chết 10 % đặt trong đặc tả phép thử, không phải đặc tính của PostgreSQL. Ba lần không vi phạm cũng không **chứng minh** thiết kế đúng — lập luận ở mục 4–7 mới là chứng minh; phép thử là bằng chứng bổ sung, và chỉ một lần vi phạm là đủ bác bỏ. Thời gian chỉ so tương đối trên máy này.

## 9. Lỗi thường gặp và đánh đổi

| Lỗi | Hậu quả | Cách đúng |
|---|---|---|
| `SELECT` khoá trước rồi `INSERT` | Hai request đồng thời cùng thấy «chưa có»: hai chuyến, hoặc 23505 trả về khách như lỗi 500 | `INSERT … ON CONFLICT DO NOTHING` làm trọng tài |
| Khoá chèn trong giao dịch riêng | Server chết giữa chừng ⇒ khoá mồ côi, gửi lại nhận `NULL` (bài tập 2) | Cùng giao dịch, hoặc thêm trạng thái + thời hạn |
| Sinh khoá cho mỗi lần gửi HTTP | Lần gửi lại có khoá mới ⇒ vẫn trùng | Khoá theo hành động của người dùng |
| Không so nội dung request | Lỗi client dùng lại khoá ⇒ lặng lẽ trả kết quả của request khác | Lưu hash, khác ⇒ 422 |
| Gửi broker trong/sau giao dịch | Tin của chuyến đã huỷ, hoặc mất tin | Outbox |
| Consumer giả định «đúng một lần» | Khách nhận hai thông báo | Chống trùng bằng khoá chính, cùng giao dịch với tác dụng |

Đánh đổi cần ghi vào thiết kế:

- **Bảng khoá và outbox lớn dần.** Mỗi request thêm một dòng khoá; mỗi lần đánh dấu outbox là một UPDATE để lại phiên bản cũ. Cần chính sách giữ khoá bao lâu và dọn dòng đã gửi — thuộc Phần 5 (VACUUM, bảo trì). Lab không đặt con số thời hạn nào.
- **Publisher giữ khoá dòng trong lúc gọi mạng.** Ở lab, giao dịch của publisher mở suốt thời gian gửi broker. Lô nhỏ thì giao dịch ngắn nhưng nhiều vòng hơn.
- **Thứ tự.** Nhiều publisher song song với SKIP LOCKED không còn gửi theo đúng thứ tự `id`; consumer không được dựa vào thứ tự tin nếu chưa thiết kế riêng cho việc đó.
- **Tác dụng ngoài database.** `processed_messages` chống trùng được vì `notifications` nằm cùng database. Nếu tác dụng là gọi nhà cung cấp SMS thật, không có giao dịch chung: cần khoá idempotency phía nhà cung cấp hoặc chấp nhận một khoảng trùng nhỏ.

## 10. Bài tập

Lời giải chạy được ở `p04/thu-nghiem/pg039-idempotency-outbox-bai-tap.sql` (không có trong lab chính):

```bash
cd examples/toi-noi-postgresql
node pg/run-pg.mjs p04 --only 39
node pg/run-pg.mjs --nen p04/lab.json:p04 p04/thu-nghiem/pg039-idempotency-outbox-bai-tap.sql
```

**Bài 1.** App iOS gửi lại request với các khoá JSON theo thứ tự khác, không có khoảng trắng. `dat_chuyen` coi là cùng request hay báo 422? Còn `"fare": 42000.0` thay cho `42000`?

<details><summary><b>Lời giải</b></summary>
<p>So hash: <code>md5</code> của hai chuỗi text khác nhau (<code>f</code>), của hai giá trị sau khi ép <code>jsonb</code> thì bằng nhau (<code>t</code>). Gọi lại với <code>'{"pickup_area_id":111,"fare":42000,"kind":"ride"}'</code> trả <code>{"status": "requested", "lap_lai": true, "trip_code": 400200}</code> — cùng request. Nhưng <code>'{"fare": 42000.0}'::jsonb::text</code> vẫn là <code>{"fare": 42000.0}</code>: <code>jsonb</code> chuẩn hoá thứ tự khoá và khoảng trắng, không chuẩn hoá cách viết số. Hash khác ⇒ <code>P0001</code>. Hệ quả: client phải tuần tự hoá số nhất quán, hoặc server tính hash trên các trường đã ép kiểu (<code>fare::integer</code>) thay vì trên cả JSON.</p>
</details>

**Bài 2.** Một đồng nghiệp đề xuất: «chèn khoá rồi COMMIT ngay để giữ chỗ, sau đó mới tạo chuyến». Mô phỏng server chết giữa hai bước rồi gửi lại cùng khoá.

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">INSERT INTO idempotency_keys (key, customer_id, request_hash)
VALUES ('bt2-0002-bbbb', 'c4000000-0000-4000-8000-000000000002', md5(…));   -- autocommit
-- server chết; client gửi lại:
SELECT dat_chuyen('bt2-0002-bbbb', 'c4000000-0000-4000-8000-000000000002', '{…}');</code></pre>
<p>Kết quả <code>NULL</code>: khoá đã có nên không chèn được, <code>response</code> là NULL nên <code>NULL || {…}</code> cũng NULL. Dòng khoá còn lại <code>bt2-0002-bbbb | NULL | NULL | NULL</code> — không chuyến, không kết quả, và gửi lại bao nhiêu lần cũng vậy. Muốn dùng thiết kế này phải có trạng thái «đang xử lý» kèm thời điểm hết hạn, để request lặp sau thời hạn được tiếp quản — và khi tiếp quản phải tránh hai server cùng làm (lại là bài toán của PG-036). Đặt khoá cùng giao dịch với chuyến thì không cần gì thêm.</p>
</details>

**Bài 3.** Broker giao một tin cho hai consumer cùng lúc. Viết lịch hai phiên cho hai trường hợp: consumer đầu chết trước COMMIT, và consumer đầu commit.

<details><summary><b>Lời giải</b></summary>
<p>Hai tin <code>seq</code> 1, 2 cùng <code>outbox_id</code> 1; tin 3, 4 cùng <code>outbox_id</code> 2. (a) A: <code>BEGIN; SELECT xu_ly_tin(1)</code> → <code>đã gửi thông báo</code> (chưa commit) · B: <code>xu_ly_tin(2)</code> ⏳ <b>Lock: transactionid — bị chặn bởi [A]</b> · C: <code>pg_terminate_backend(A)</code> → <code>t</code> · B ✔ <code>đã gửi thông báo</code> — dấu «đã xử lý» và thông báo của A cùng biến mất nên B tự làm. (b) A: <code>BEGIN; xu_ly_tin(3)</code> → <code>đã gửi thông báo</code> · B: <code>xu_ly_tin(4)</code> ⏳ · A: <code>COMMIT</code> · B ✔ <code>bỏ qua (đã xử lý)</code>. Cuối cùng <code>1 | 1</code> và <code>2 | 1</code>: mỗi sự kiện đúng một thông báo trong cả hai trường hợp, đúng như mục 4–5 nhưng ở phía nhận.</p>
</details>

## 11. Đọc tiếp

- [PG-040 · Thực hành: hai tài xế cùng nhận một chuyến](/articles/postgresql-thuc-hanh-tranh-chap-nhan-chuyen) — bài tổng hợp Phần 4.
- [PostgreSQL 18 · INSERT](https://www.postgresql.org/docs/18/sql-insert.html): `ON CONFLICT`, arbiter index, chờ khi có index duy nhất.
- [PostgreSQL 18 · Index Uniqueness Checks](https://www.postgresql.org/docs/18/index-unique-checks.html): chờ giao dịch chưa commit rồi kiểm lại.
- [PostgreSQL 18 · Transaction Isolation](https://www.postgresql.org/docs/18/transaction-iso.html): `ON CONFLICT` ở Read Committed.
- [PostgreSQL 18 · Function Volatility Categories](https://www.postgresql.org/docs/18/xfunc-volatility.html): snapshot mới cho mỗi câu trong hàm `VOLATILE`.
- [PostgreSQL 18 · System Information Functions](https://www.postgresql.org/docs/18/functions-info.html): `pg_xact_status`.
- [PostgreSQL 18 · SELECT — Locking Clause](https://www.postgresql.org/docs/18/sql-select.html), [Errors and Messages (PL/pgSQL)](https://www.postgresql.org/docs/18/plpgsql-errors-and-messages.html), [JSON Types](https://www.postgresql.org/docs/18/datatype-json.html).
- Lab: `p04/39-idempotency-outbox.sql`, `p04/OUTPUT.txt`, `p04/DO-DAC.md` (K039), `p04/dong-thoi.mjs`, `p04/README.md`.

Bài tiếp theo: [PG-040: Thực hành: hai tài xế cùng nhận một chuyến](/articles/postgresql-thuc-hanh-tranh-chap-nhan-chuyen)
