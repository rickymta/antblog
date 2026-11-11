---
slug: postgresql-thuc-hanh-tranh-chap-nhan-chuyen
title: "Thực hành: hai tài xế cùng nhận một chuyến"
summary: "Chuyến 400101 được gửi cho các tài xế gần nhất và hai người cùng được báo «bạn đã nhận chuyến». Bài thực hành cuối Phần 4: bàn giao lịch chạy, bất biến «đúng một người thắng» và bằng chứng từ lab lẫn phép thử 8 phiên."
category: postgresql
level: advanced
tags: [postgresql, pg-040, thuc-hanh, concurrency, invariant, khoa-dong, toi-noi]
language: vi
---

Chuyến 400101 ở khu 112 vừa được đặt. Hệ thống gửi nó cho vài tài xế gần nhất cùng lúc; ai bấm «Nhận» trước thì được. Tài xế 9101 bấm, app báo «Bạn đã nhận chuyến», anh quay đầu xe. Tài xế 9102 bấm sau nửa giây, app **cũng** báo «Bạn đã nhận chuyến». Trong database chỉ còn tên 9102. Một người chạy tới điểm đón vô ích, và khách thấy tên tài xế đổi giữa chừng.

Hồ sơ Tới Nơi đã ghi bài toán này từ đầu: nhiều tài xế có thể cùng bấm nhận một chuyến trong cùng một giây, và chỉ một người được nhận. Test bằng tay gần như không bao giờ thấy lỗi, vì hai người bấm cách nhau đủ xa. Thứ cần bàn giao là **lịch chạy** chỉ ra cơ chế, **bất biến** viết thành SQL kiểm được, và **bằng chứng** khi các phiên chạy đua thật.

Đây là bài thực hành cuối Phần 4. Bạn nhận đề, tự làm, rồi mới mở lời giải mẫu và tự chấm theo mục 4.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-040 · Phần 4/6 · Nâng cao</span>
<p>Đọc trước: toàn bộ PG-031 … PG-039 (danh sách ở mục 6), đặc biệt <a href="/articles/postgresql-optimistic-version-conditional-update">PG-035 · Cập nhật có điều kiện</a>, <a href="/articles/postgresql-for-update-skip-locked">PG-036 · FOR UPDATE, NOWAIT và SKIP LOCKED</a> và <a href="/articles/postgresql-serializable-retry">PG-038 · Serializable và retry</a>. Lab: <code>examples/toi-noi-postgresql/p04</code> (tệp <code>40-thuc-hanh-nhan-chuyen.sql</code>, phép thử K040 ở <code>DO-DAC.md</code>) và thí nghiệm <code>p04/thu-nghiem/pg040-nhan-chuyen-bai-tap.sql</code>. Sau bài này, bạn bàn giao được lịch chạy, invariant và bằng chứng chỉ một người thắng.</p>
</div>

## 1. Đề bài

### Bối cảnh

Nền P04 (PostgreSQL 18.1 thật): 10 chuyến đang chờ `400101`…`400110`, `status = 'requested'`, `version = 1`; tài xế `9101` [A], `9102` [B], `9103` [C], đều online. Ba bảng liên quan (`p04/00-mo-rong-schema.sql`):

| Bảng | Vai trò |
|---|---|
| `trips` | `status`, `driver_id`, `accepted_at`, `version` — nơi «ai đang giữ chuyến» được ghi |
| `trip_claims` | Một dòng = app đã báo một tài xế «bạn đã nhận chuyến». **Cố ý không UNIQUE** để đếm được vi phạm |
| `trip_assignments` | Phân công, `PRIMARY KEY (trip_id)` — một chuyến một tài xế |

Lab rút gọn vòng đời: chuyến đi thẳng `requested` → `accepted` (hồ sơ có thêm `searching`; không đổi bản chất bài toán).

### Sản phẩm bàn giao

| Mã | Sản phẩm | Yêu cầu tối thiểu |
|---|---|---|
| D1 | Đặc tả bất biến | Viết bằng lời trước khi viết SQL: «thắng» là gì, kiểm ở đâu |
| D2 | Lịch chạy | Ít nhất bốn cách nhận chuyến, mỗi cách một lịch hai–ba phiên **tất định**, đặt kết quả của bước bị chặn đúng chỗ nó thực sự xong |
| D3 | Kiểm bất biến | Một câu SQL cho mọi cách, có expected output |
| D4 | Phép thử đồng thời | Nhiều phiên chạy đua thật, lặp nhiều lần, đếm vi phạm; ghi máy và giới hạn |
| D5 | Lựa chọn | Cách nào dùng, vì sao, cái giá |
| D6 | Bằng chứng | Chạy từ database mới, kỳ vọng do máy kiểm, điều **chưa** kiểm |

Chỉ dùng kiến thức P01–P04. Không sửa `trip_claims` thành UNIQUE trong lời giải chính (đó là bài tập 3).

## 2. Gợi ý trước khi làm

- Cách sai phổ biến nhất là **đọc rồi ghi** ở hai câu: đọc `status`, thấy `requested`, rồi `UPDATE` không điều kiện. Viết nó trước, để có đối chứng.
- Read Committed đọc lại dòng khi UPDATE gặp khoá ([PG-034](/articles/postgresql-isolation-read-committed-repeatable-read)) — dùng nó thế nào cho đúng? ([PG-035](/articles/postgresql-optimistic-version-conditional-update))
- Khoá dòng trước khi đọc ([PG-033](/articles/postgresql-lock-row-table), [PG-036](/articles/postgresql-for-update-skip-locked)), và ràng buộc duy nhất làm trọng tài ([PG-039](/articles/postgresql-idempotency-outbox-transaction)).
- Lịch chạy là thứ tự **gửi**; bước có ⏳ chưa xong. Kết quả của nó thuộc về sau bước nhả khoá.
- «Không vi phạm trong N lần» không chứng minh cách đó đúng; chỉ cần **một** lần vi phạm để bác bỏ.

```bash
cd examples/toi-noi-postgresql
node pg/run-pg.mjs p04 --only 40                                                  # ~1 s nền + vài giây
node pg/run-dong-thoi.mjs p04/dong-thoi.mjs --nho --only K040                     # phép thử cỡ nhỏ, không ghi tệp
node pg/run-pg.mjs --nen p04/lab.json:p04 p04/thu-nghiem/pg040-nhan-chuyen-bai-tap.sql
```

## 3. Lời giải mẫu

Lời giải là `40-thuc-hanh-nhan-chuyen.sql`; số đo là `DO-DAC.md`. Mở từng phần **sau khi** đã có bản của bạn.

### 3.1 D1 — Bất biến

<details><summary><b>Lời giải</b></summary>
<p>Với mỗi chuyến đã có người nhận: <b>đúng một</b> tài xế được báo «bạn đã nhận chuyến» (đúng một dòng <code>trip_claims</code>), và người đó <b>chính là</b> <code>trips.driver_id</code>. Thêm điều kiện ngược cho phép thử: chuyến được tranh không bao giờ rơi vào cảnh <b>không ai</b> thắng.</p>
<p>Bất biến đặt trên <code>trip_claims</code>, không chỉ trên <code>trips</code>, vì lỗi người dùng thấy là <i>app báo nhầm</i>. Ở cách sai, <code>trips</code> luôn trông «hợp lệ» — một chuyến, một <code>driver_id</code> — trong khi hai người đã được báo nhận. Kiểm mỗi <code>trips</code> sẽ không bao giờ thấy lỗi.</p>
</details>

### 3.2 D2 — Lịch (a): đọc rồi ghi, không khoá

<details><summary><b>Lời giải</b></summary>
<table>
<thead><tr><th>Bước</th><th>A (9101)</th><th>B (9102)</th></tr></thead>
<tbody>
<tr><td>T1</td><td><code>BEGIN; SELECT status FROM trips WHERE code = 400101</code> → <code>requested</code></td><td></td></tr>
<tr><td>T2</td><td></td><td><code>BEGIN;</code> cùng câu → <code>requested</code></td></tr>
<tr><td>T3</td><td><code>UPDATE trips SET status = 'accepted', driver_id = …9101 … WHERE code = 400101</code> → <code>UPDATE 1</code>; chèn claim 9101</td><td></td></tr>
<tr><td>T4</td><td><code>COMMIT</code></td><td></td></tr>
<tr><td>T5</td><td></td><td>Cùng UPDATE với 9102 → <code>UPDATE 1</code> — không chờ, không lỗi; chèn claim 9102</td></tr>
<tr><td>T6</td><td></td><td><code>COMMIT</code></td></tr>
</tbody>
</table>
<pre><code class="language-text"> duoc_bao_nhan | driver_trong_db
---------------+-----------------
          9101 |            9102
          9102 |            9102</code></pre>
<p>B quyết định dựa trên một lần đọc đã cũ, và câu ghi của B không mang điều kiện nào để database biết điều đó. Không có gì «hỏng» theo nghĩa kỹ thuật: mọi câu đều thành công. Đó là lý do lỗi này lọt qua mọi kiểm tra không có hai phiên.</p>
</details>

### 3.3 D2 — Lịch (b), (c), (d): ba cách đúng

<details><summary><b>Lời giải</b></summary>
<p><b>(b) Một câu UPDATE có điều kiện</b>, ghi claim trong cùng câu bằng CTE, ba tài xế, chuyến 400102. B và C là câu tự commit:</p>
<pre><code class="language-sql">WITH nhan AS (
  UPDATE trips SET status = 'accepted', driver_id = 'd4000000-0000-4000-8000-000000009101',
                   accepted_at = now(), updated_at = now(), version = version + 1
  WHERE code = 400102 AND status = 'requested'
  RETURNING id, driver_id)
INSERT INTO trip_claims (trip_id, driver_id, method)
SELECT id, driver_id, 'b_co_dieu_kien' FROM nhan
RETURNING (SELECT code FROM drivers WHERE id = driver_id);</code></pre>
<table>
<thead><tr><th>Bước</th><th>A (9101)</th><th>B (9102)</th><th>C (9103)</th></tr></thead>
<tbody>
<tr><td>T1</td><td><code>BEGIN;</code> câu trên → <code>9101</code></td><td></td><td></td></tr>
<tr><td>T2</td><td></td><td>câu trên ⏳ <b>Lock: transactionid — bị chặn bởi [A]</b></td><td></td></tr>
<tr><td>T3</td><td></td><td></td><td>câu trên ⏳ <b>Lock: tuple — bị chặn bởi [B]</b></td></tr>
<tr><td>T4</td><td><code>COMMIT</code></td><td></td><td></td></tr>
<tr><td>T5</td><td></td><td>✔ <code>(0 rows)</code>, <code>INSERT 0 0</code></td><td></td></tr>
<tr><td>T6</td><td></td><td></td><td>✔ <code>(0 rows)</code>, <code>INSERT 0 0</code></td></tr>
</tbody>
</table>
<p>B gặp dòng A đang sửa nên chờ; A commit thì B <b>đánh giá lại điều kiện <code>WHERE</code> trên phiên bản mới</b> của dòng (tài liệu Transaction Isolation, Read Committed), thấy <code>accepted</code> ⇒ 0 dòng ⇒ không claim. C không chờ A mà xếp hàng sau B, người đang giữ khoá <code>tuple</code> — đầu hàng chờ của dòng (PG-033).</p>
<p><b>(c) Khoá trước, kiểm trong ứng dụng</b>, chuyến 400103: A <code>BEGIN; SELECT status … FOR UPDATE</code> → <code>requested</code> · B <code>BEGIN;</code> cùng câu ⏳ <b>bị chặn bởi [A]</b> · A <code>UPDATE</code> → <code>UPDATE 1</code>, chèn claim, <code>COMMIT</code> · B ✔ <code>accepted</code> ⇒ app không ghi · B <code>ROLLBACK</code>. <code>FOR UPDATE</code> trả về phiên bản mới của dòng sau khi chờ, nên B đọc đúng.</p>
<p><b>(d) Ràng buộc làm trọng tài</b>, chuyến 400104: A <code>BEGIN; INSERT INTO trip_assignments … 9101</code> → <code>INSERT 0 1</code> · B <code>BEGIN; INSERT … 9102</code> ⏳ <b>bị chặn bởi [A]</b> · C <code>INSERT … 9103 ON CONFLICT (trip_id) DO NOTHING</code> ⏳ <b>bị chặn bởi [A]</b> · A cập nhật <code>trips</code>, chèn claim, <code>COMMIT</code> · B ✔ lỗi · C ✔ <code>INSERT 0 0</code> · B <code>ROLLBACK</code>.</p>
<pre><code class="language-text">ERROR:  duplicate key value violates unique constraint "pk_trip_assignments"
DETAIL:  Key (trip_id)=(a4000000-0000-4000-8000-000000400104) already exists.
-- [lab] SQLSTATE 23505 · constraint pk_trip_assignments · table trip_assignments</code></pre>
<p>Ở (d), B và C cùng chờ <b>A</b>: đây là chờ ở index duy nhất (chèn trùng khoá của một giao dịch chưa commit), không phải hàng chờ khoá dòng như (b). Cách này đúng kể cả khi code ứng dụng quên điều kiện: chỉ một dòng phân công tồn tại được.</p>
</details>

### 3.4 D3 — Kiểm bất biến

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">SELECT t.code, c.method, count(*) AS so_nguoi_thang,
       count(*) FILTER (WHERE c.driver_id = t.driver_id) AS khop_driver_id,
       CASE WHEN count(*) = 1 AND count(*) FILTER (WHERE c.driver_id = t.driver_id) = 1
            THEN 'đạt' ELSE 'VI PHẠM' END AS bat_bien
FROM trip_claims AS c JOIN trips AS t ON t.id = c.trip_id
GROUP BY t.code, c.method ORDER BY t.code;</code></pre>
<pre><code class="language-text">  code  |     method     | so_nguoi_thang | khop_driver_id | bat_bien
--------+----------------+----------------+----------------+----------
 400101 | a_khong_khoa   |              2 |              1 | VI PHẠM
 400102 | b_co_dieu_kien |              1 |              1 | đạt
 400103 | c_for_update   |              1 |              1 | đạt
 400104 | d_unique       |              1 |              1 | đạt</code></pre>
<p>Thêm cho (d): <code>trip_assignments</code> khớp <code>trips</code> — <code>400104 | 9101 | 9101</code>. Mỗi dòng có chỉ thị <code>expect-line</code> trong tệp lab, nên đổi lịch mà kết quả đổi là runner báo <code>FAIL</code>.</p>
</details>

### 3.5 D4 — Phép thử đồng thời

<details><summary><b>Lời giải</b></summary>
<p>Kịch bản K040 (<code>p04/dong-thoi.mjs</code>): <b>1.000 chuyến × 8 tài xế</b>, mỗi chuyến cả 8 phiên psql thật cùng bấm; các phiên chờ nhau xong chuyến này mới sang chuyến sau («rào»), 3 lần, mỗi lần một database mới. Cách (e) là <b>đúng code sai của (a)</b> nhưng chạy Repeatable Read, thử lại toàn giao dịch khi <code>40001</code> (tối đa 10 lần). Máy: Intel Core i7-10700 (16 luồng), 64 GB RAM, NVMe Kingston SA2000, Windows 11 Pro, Node 22.18, PostgreSQL 18.1, tải CPU nền ~6 %.</p>
<table>
<thead><tr><th>Cách</th><th>Giây/lần (min–max)</th><th>Lượt thắng / thua</th><th>Chuyến &gt; 1 người thắng</th><th>Ghi chú</th></tr></thead>
<tbody>
<tr><td>(a) không khoá</td><td>7,26–7,33</td><td>8.000 / 0</td><td><b>1.000 / 1.000</b> mỗi lần</td><td>7.000 claim ≠ <code>trips.driver_id</code></td></tr>
<tr><td>(b) UPDATE có điều kiện</td><td>1,32–1,48</td><td>1.000 / 7.000</td><td>0</td><td>một câu, tự commit</td></tr>
<tr><td>(c) FOR UPDATE</td><td>3,92–4,00</td><td>1.000 / 7.000</td><td>0</td><td></td></tr>
<tr><td>(d) PK <code>trip_assignments</code></td><td>3,34–3,86</td><td>1.000 / 7.000</td><td>0</td><td></td></tr>
<tr><td>(e) code (a) + RR + thử lại</td><td>15,48–17,04</td><td>1.000 / 7.000</td><td>0</td><td><b>7.000 lần <code>40001</code></b> + thử lại mỗi lần</td></tr>
</tbody>
</table>
<p>Ở cả năm cách, «chuyến không ai thắng» = 0 qua 3 lần; với (b)–(e), «chuyến &gt; 1 người thắng» và «claim ≠ <code>driver_id</code>» = 0 qua 3 lần. (e) trả đúng 7 lần <code>40001</code> mỗi chuyến — bằng số người thua.</p>
<p><b>Đừng trích 1.000/1.000 của (a) như tỉ lệ production.</b> Rào ép cả 8 phiên đọc <code>requested</code> trước khi ai kịp commit, để tranh chấp xảy ra trên từng chuyến; ngoài đời các lần bấm không đồng thời đến vậy, nên tỉ lệ thấp hơn nhiều. Con số chứng minh một điều: khi tranh chấp xảy ra, (a) sai, và (b)–(e) không sai lần nào trong 3.000 chuyến bị tranh (24.000 lượt bấm) mỗi cách. Thời gian chỉ so tương đối trên một máy, ống psql ↔ Node ~0,3 ms/lượt.</p>
</details>

### 3.6 D5 — Chọn cách nào

<details><summary><b>Lời giải</b></summary>
<ul>
<li><b>Mặc định: (b).</b> Một câu, không có khoảng hở giữa đọc và ghi ở ứng dụng, đúng ở Read Committed nhờ đánh giá lại điều kiện. Nhanh nhất trong phép thử. Nhớ đọc số dòng: 0 dòng là «đã có người nhận» — hoặc chính mình đã nhận (bài tập 2).</li>
<li><b>Lưới an toàn: (d).</b> Đặt bất biến vào schema: code sau này có quên điều kiện, PK vẫn chặn. Kết hợp (b) trong code với một ràng buộc duy nhất là cách phòng thủ hai lớp.</li>
<li><b>Khi phải quyết định trong ứng dụng: (c).</b> Ví dụ kiểm thêm điều kiện nghiệp vụ trước khi nhận. Giữ khoá trong lúc ứng dụng nghĩ ⇒ giữ giao dịch ngắn, không gọi mạng ở giữa; không xoá/đổi khoá chính thì dùng <code>FOR NO KEY UPDATE</code> để không chặn INSERT dòng con (PG-033).</li>
<li><b>Khi không sửa được đoạn đọc–ghi: (e).</b> Hợp lệ, nhưng mọi người thua đều phải chạy lại cả giao dịch, và code thử lại phải có trần + backoff (PG-038).</li>
<li><b>Không dùng SKIP LOCKED ở đây.</b> Nó hợp cho hàng đợi «lấy việc nào cũng được» (PG-036); nhận <i>đúng chuyến này</i> mà bỏ qua dòng đang khoá là trả lời «còn trống» trong khi người khác sắp nhận.</li>
</ul>
</details>

### 3.7 D6 — Bằng chứng và giới hạn

<details><summary><b>Lời giải</b></summary>
<p>Mỗi tệp bài chạy trên database mới chép từ nền; <code>npm run p04</code> (cả mười bài): <b>613 bước, 0 FAIL</b>, 25 bước <code>expect-blocked</code>, hai lần chạy độc lập cho đầu ra giống hệt nhau sau chuẩn hoá. Phép thử K040: bất biến «phải đúng» ĐẠT cả 3 lần cho (b)–(e). Thí nghiệm của bài ĐẠT trên PostgreSQL 18.1.</p>
<p><b>Chưa kiểm:</b> Linux/macOS; mất kết nối do mạng thật (lab chỉ mô phỏng bằng <code>pg_terminate_backend</code>); ứng dụng thật có độ trễ mạng giữa đọc và ghi (khoảng hở lớn hơn ống psql — tỉ lệ vi phạm của (a) có thể cao hoặc thấp hơn); tải nền của production (560 UPDATE/giây vào <code>trips</code> theo hồ sơ) cùng lúc với tranh chấp. Ba lần không vi phạm là bằng chứng bổ sung; lập luận ở 3.3 mới là chứng minh.</p>
</details>

## 4. Tiêu chí tự chấm

Mỗi tiêu chí 10 điểm; từ 80 điểm là đạt để sang Phần 5; tiêu chí 1, 3 và 6 bắt buộc.

| # | Tiêu chí | Đạt khi |
|---|---|---|
| 1 | Bất biến đúng chỗ | Kiểm trên «ai được báo thắng», không chỉ trên `trips` |
| 2 | Có đối chứng sai | Lịch (a) tái hiện hai người thắng, tất định |
| 3 | Lịch đúng thứ tự xảy ra | Kết quả bước ⏳ đặt sau bước nhả khoá; ghi rõ ai chặn ai |
| 4 | Giải thích cơ chế | Đánh giá lại điều kiện (b), đọc bản mới sau khi chờ (c), chờ ở index duy nhất (d) |
| 5 | Xếp hàng | Nói được vì sao C chờ B ở (b) nhưng chờ A ở (d) |
| 6 | Kiểm bằng máy | SQL bất biến có expected output, runner báo ĐẠT |
| 7 | Phép thử | Nhiều phiên, nhiều lần, có máy, min–max |
| 8 | Không trích sai số | Không coi 1.000/1.000 của (a) là tỉ lệ production |
| 9 | Lựa chọn có giá | Nêu cái giá của (c) và (e), vai trò lưới an toàn của (d) |
| 10 | Trung thực | Liệt kê điều chưa kiểm |

## 5. Bẫy hay gặp khi tự làm

- **Kiểm bất biến chỉ trên `trips`.** Cách (a) để lại một dòng `trips` hoàn toàn hợp lệ.
- **Tưởng `BEGIN … COMMIT` là đủ.** Lịch (a) có giao dịch đầy đủ; giao dịch không làm hai lần đọc–ghi của hai người thành tuần tự.
- **Đọc lịch theo thứ tự gửi.** Ở (b), «0 dòng» của B xảy ra sau COMMIT của A, không phải lúc B gửi câu.
- **UPDATE có điều kiện nhưng không đọc số dòng.** Database đã từ chối đúng, app vẫn báo «Bạn đã nhận chuyến».
- **Coi 23505 ở (d) là lỗi hệ thống.** Đó là «người khác đã nhận»; dùng `ON CONFLICT DO NOTHING` nếu không muốn giao dịch hỏng.
- **Test bằng hai cửa sổ psql gõ tay.** Không bao giờ gõ đủ nhanh để thấy (a) sai; cần lịch tất định hoặc phép thử có rào.

## 6. Tổng kết Phần 4

| Bài | Bạn đã có gì cho sản phẩm cuối phần |
|---|---|
| [PG-031 · Transaction, COMMIT và ROLLBACK](/articles/postgresql-transaction-commit-rollback) | Chuyến + claim cùng thành công hoặc cùng huỷ; COMMIT của giao dịch hỏng trả `ROLLBACK` |
| [PG-032 · MVCC và snapshot](/articles/postgresql-mvcc-snapshot) | Vì sao B đọc `requested` trong khi A đã sửa — snapshot |
| [PG-033 · Khoá dòng, khoá bảng](/articles/postgresql-lock-row-table) | `lab_cho()`/`lab_khoa()`, khoá `tuple` và hàng chờ |
| [PG-034 · Read Committed và Repeatable Read](/articles/postgresql-isolation-read-committed-repeatable-read) | Đọc lại dòng khi UPDATE gặp khoá; RR báo `40001` |
| [PG-035 · Cập nhật có điều kiện](/articles/postgresql-optimistic-version-conditional-update) | Cách (b) và đọc số dòng |
| [PG-036 · FOR UPDATE, NOWAIT, SKIP LOCKED](/articles/postgresql-for-update-skip-locked) | Cách (c); khi nào không dùng SKIP LOCKED |
| [PG-037 · Deadlock và thứ tự khoá](/articles/postgresql-deadlock-lock-order-timeout) | Khoá nhiều dòng theo thứ tự, `lock_timeout` làm lưới an toàn |
| [PG-038 · Serializable và retry](/articles/postgresql-serializable-retry) | Cách (e); thử lại toàn giao dịch có trần và backoff |
| [PG-039 · Idempotency và outbox](/articles/postgresql-idempotency-outbox-transaction) | Cách (d) — ràng buộc duy nhất làm trọng tài; bấm lại không trùng |

Sản phẩm của phần đúng như khung: kịch bản tranh chấp nhận chuyến với kiểm tra bất biến và retry. **Chưa** có: VACUUM dọn phiên bản cũ mà 560 UPDATE/giây để lại, quyền truy cập, sao lưu và khôi phục (Phần 5); mất mạng thật, nhiều máy chủ, broker thật (Phần 6).

## 7. Bài tập mở rộng

Lời giải chạy được ở `p04/thu-nghiem/pg040-nhan-chuyen-bai-tap.sql` (không có trong lab chính).

**Bài 1.** Không được sửa code (a). Chạy nó ở Repeatable Read với lịch: A đọc, B đọc, A UPDATE, B UPDATE, A claim + COMMIT. B nhận gì? Thử lại thì sao?

<details><summary><b>Lời giải</b></summary>
<p>Chuyến 400101. B <code>UPDATE</code> ⏳ <b>Lock: transactionid — bị chặn bởi [A]</b>; A chèn claim, <code>COMMIT</code>; B ✔ <code>ERROR: could not serialize access due to concurrent update</code> (<code>40001</code>); B <code>COMMIT</code> trả thẻ <code>ROLLBACK</code>. B thử lại <b>toàn bộ</b> giao dịch: <code>BEGIN ISOLATION LEVEL REPEATABLE READ</code>, đọc → <code>accepted</code> ⇒ thôi. Kết quả <code>400101 | 1 | 9101 | 9101</code>: một claim, khớp <code>driver_id</code>. Đây là bản tất định của K040-e. Ở Read Committed, B sẽ chờ rồi <b>ghi đè</b> như lịch S3 của PG-034 — câu UPDATE của (a) không có điều kiện nào để đánh giá lại.</p>
</details>

**Bài 2.** Mạng chập chờn, app của 9101 gửi «Nhận» hai lần với cách (b). Lần hai trả 0 dòng. App nên báo gì?

<details><summary><b>Lời giải</b></summary>
<p>Chuyến 400105: lần một → <code>9101</code>; lần hai → <code>(0 rows)</code>. Nếu app coi 0 dòng là «đã có người nhận», chính người thắng bị báo thua. Sau 0 dòng, đọc lại người giữ chuyến:</p>
<pre><code class="language-sql">SELECT status, driver_id = 'd4000000-0000-4000-8000-000000009101' AS la_toi FROM trips WHERE code = 400105;</code></pre>
<p>9101 nhận <code>accepted | t</code> ⇒ trả lại «Bạn đã nhận chuyến» mà không ghi thêm claim; 9102 nhận <code>accepted | f</code> ⇒ «Đã có người nhận». Số claim của 400105 vẫn <code>1</code>. Đây là idempotency của PG-039 áp cho tài xế, không cần bảng khoá vì «ai đang giữ chuyến» đã là khoá tự nhiên.</p>
</details>

**Bài 3.** Thêm `CREATE UNIQUE INDEX ux_trip_claims_trip ON trip_claims (trip_id)` rồi chạy lại đúng lịch (a) trên 400106. Bất biến có giữ không? Cái giá là gì?

<details><summary><b>Lời giải</b></summary>
<p>A đọc, B đọc, A ghi + claim + <code>COMMIT</code>. B <code>UPDATE</code> vẫn → <code>UPDATE 1</code> (ghi đè <code>driver_id</code> thành 9102), nhưng chèn claim → <code>23505 ux_trip_claims_trip</code>, <code>DETAIL: Key (trip_id)=(a4000000-0000-4000-8000-000000400106) already exists.</code>; <code>COMMIT</code> trả <code>ROLLBACK</code> nên cả câu ghi đè của B bị huỷ. Kết quả <code>400106 | 1 | 9101 | 9101</code>: bất biến giữ, nhờ claim và UPDATE nằm cùng giao dịch (PG-031). Cái giá: B nhận một lỗi thay vì câu trả lời có nghĩa, và bất biến chỉ được giữ nếu claim luôn được ghi <b>trong cùng giao dịch</b> với UPDATE. Ràng buộc là lưới an toàn, không thay cho cách (b).</p>
</details>

## 8. Đọc tiếp

- [PostgreSQL 18 · Transaction Isolation](https://www.postgresql.org/docs/18/transaction-iso.html): Read Committed đánh giá lại `WHERE` trên phiên bản mới; Repeatable Read báo lỗi tuần tự hoá.
- [PostgreSQL 18 · SELECT — Locking Clause](https://www.postgresql.org/docs/18/sql-select.html): `FOR UPDATE`, `NOWAIT`, `SKIP LOCKED`.
- [PostgreSQL 18 · INSERT](https://www.postgresql.org/docs/18/sql-insert.html) và [Index Uniqueness Checks](https://www.postgresql.org/docs/18/index-unique-checks.html): chờ ở index duy nhất.
- Lab: `p04/40-thuc-hanh-nhan-chuyen.sql`, `p04/OUTPUT.txt`, `p04/DO-DAC.md` (K040-a … K040-e), `p04/dong-thoi.mjs`, `p04/README.md` (ghi chú cho người đọc số đo), `p04/thu-nghiem/pg040-nhan-chuyen-bai-tap.sql`.

Bài tiếp theo: Phần 5 · Bảo mật, bảo trì và phục hồi sắp ra mắt. Đọc lại từ đầu Phần 4: [PG-031: Transaction, COMMIT và ROLLBACK](/articles/postgresql-transaction-commit-rollback)
