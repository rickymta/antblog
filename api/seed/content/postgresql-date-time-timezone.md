---
slug: postgresql-date-time-timezone
title: "Ngày giờ và múi giờ trong truy vấn: bộ lọc ngày nghiệp vụ có kiểm tra hai biên"
summary: "Ba người đếm «chuyến ngày 15/10» của Tới Nơi ra 105, 106 và 107; báo cáo tháng theo UTC thiếu 12 chuyến; một câu AT TIME ZONE '+07' lùi chuyến về chiều hôm trước. Bài học về timestamptz, AT TIME ZONE, date_trunc có múi giờ và cách kiểm tra hai biên để mỗi ngày, tuần, tháng có đúng một đáp án."
category: postgresql
level: intermediate
tags: [postgresql, pg-016, timestamptz, timezone, date-trunc, interval, toi-noi]
language: vi
---

Cuối tháng 10, ba người ở Tới Nơi cùng trả lời câu «hôm 15/10 có bao nhiêu chuyến?». Kế toán báo **105**, CSKH báo **107**, đội dữ liệu báo **106**. Cùng tuần, báo cáo tháng 10 trên công cụ BI (phiên đặt UTC) thiếu 12 chuyến so với số của vận hành; và một kỹ sư viết `AT TIME ZONE '+07'` để «đổi sang giờ Việt Nam», nhận về chuyến 07:00 sáng 15/10 bị đổi thành **17:00 chiều 14/10**.

PG-007 đã đặt quy tắc «mốc ghi `+07`, nửa mở». Bài này đi tiếp: vì sao từng cách sai lệch đúng một vài chuyến, cách dựng biên từ tham số ngày, gom theo ngày/tuần/tháng/giờ mà không phụ thuộc phiên, ghi múi giờ bằng tên chứ không bằng độ lệch, và thói quen **kiểm tra hai biên** cho mọi bộ lọc thời gian.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-016 · Phần 2/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/postgresql-kieu-du-lieu-tien-thoi-gian-uuid">PG-004 · kiểu tiền, thời gian và định danh</a>, <a href="/articles/postgresql-select-filter-order-limit">PG-007 · SELECT, lọc và sắp xếp</a>. Lab: <code>examples/toi-noi-postgresql/p02</code> (tệp <code>16-date-time-timezone.sql</code>) và thí nghiệm thêm <code>p02/thu-nghiem/pg016-*.sql</code>. Sau bài này, bạn viết được bộ lọc ngày nghiệp vụ kèm kiểm tra hai biên.</p>
</div>

## 1. Dữ liệu dùng trong bài

Fixture Phần 2: 3.174 chuyến đặt từ 01/10 tới 00:58 ngày 31/10/2026 giờ VN. Có **383** chuyến đặt 00:00–06:59 giờ VN — tức vẫn là ngày hôm trước theo UTC. Năm chuyến cài sát biên ngày 15/10 (giờ VN): **101489** 00:00:00 · **101502** 06:59:59 · **101503** 07:00:00 · **101594** 23:59:59.5 · **101595** 00:00:00 ngày 16/10.

```bash
cd examples/toi-noi-postgresql
node run.mjs p01/00-reset.sql p01/01-schema.sql p02/00-mo-rong-schema.sql \
  p02/01-fixture.sql p02/02-kiem-tra-fixture.sql p02/16-date-time-timezone.sql
```

Thí nghiệm thêm chạy trên cùng nền, thay tệp cuối bằng `p02/thu-nghiem/pg016-mui-gio.sql` hoặc `p02/thu-nghiem/pg016-bai-tap.sql`. Kết quả chép từ lần chạy thật trên PGlite 0.3.16 (PostgreSQL 17.5); chưa chạy trên server thật. Tệp lab đặt `TimeZone` = `Asia/Ho_Chi_Minh` ở đầu, đổi giữa chừng khi cần rồi đặt lại.

## 2. Ba kiểu, và AT TIME ZONE đi hai chiều

Nhắc lại PG-004: **`timestamptz`** lưu một **khoảnh khắc** tuyệt đối (bên trong là UTC); `TimeZone` của phiên chỉ quyết định cách **hiển thị** và cách **hiểu chuỗi không ghi múi giờ**. **`timestamp`** (không múi giờ) là «giờ trên đồng hồ treo tường», không biết là ở đâu. **`date`** là một ngày lịch.

`AT TIME ZONE` chuyển giữa hai kiểu đầu, và **đổi chiều theo kiểu đầu vào**:

| Biểu thức | Kết quả | Nghĩa |
|---|---|---|
| `timestamptz AT TIME ZONE 'Asia/Ho_Chi_Minh'` | `timestamp` | Khoảnh khắc này, đồng hồ ở Việt Nam chỉ mấy giờ? |
| `timestamp AT TIME ZONE 'Asia/Ho_Chi_Minh'` | `timestamptz` | Giờ đồng hồ này, nếu đọc ở Việt Nam, là khoảnh khắc nào? |

```sql
SELECT code,
       requested_at,                                   -- theo TimeZone của phiên
       requested_at AT TIME ZONE 'UTC'              AS gio_utc,
       requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh' AS gio_vn
FROM trips WHERE code = 101503;
```

```text
  code  |      requested_at      |       gio_utc       |       gio_vn
--------+------------------------+---------------------+---------------------
 101503 | 2026-10-15 07:00:00+07 | 2026-10-15 00:00:00 | 2026-10-15 07:00:00
(1 row)
```

Cột 2 có `+07` (một `timestamptz`), cột 3–4 không có (hai `timestamp`). Áp `AT TIME ZONE 'Asia/Ho_Chi_Minh'` hai lần liên tiếp lại ra đúng `requested_at` ban đầu — đi rồi về.

## 3. Sáu cách đếm «ngày 15/10», hai phiên

Câu 2 của `16-date-time-timezone.sql` đếm bằng sáu cách trong một câu (`count(*) FILTER (WHERE …)`, PG-012), chạy ở phiên giờ VN rồi phiên UTC:

| | Cách viết | Phiên VN | Phiên UTC |
|---|---|---|---|
| a | `>= '2026-10-15 00:00:00+07' AND < '2026-10-16 00:00:00+07'` | **106** | **106** |
| b | `(requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::date = DATE '2026-10-15'` | 106 | 106 |
| c | `date_trunc('day', requested_at, 'Asia/Ho_Chi_Minh') = '2026-10-15 00:00:00+07'` | 106 | 106 |
| d | `requested_at::date = DATE '2026-10-15'` | 106 | **105** |
| e | `BETWEEN '2026-10-15 00:00:00' AND '2026-10-15 23:59:59'` | **105** | 105 |
| f | `>= '2026-10-15' AND <= '2026-10-16'` | **107** | 105 |

Đáp án đúng là **106**. Ba người ở đầu bài là ba dòng của bảng:

- **e (105)** bỏ sót 101594 lúc 23:59:59.5 — `timestamptz` lưu tới micro giây, «23:59:59» không phải giây cuối của ngày.
- **f (107)** lấy cả 101595 lúc đúng 00:00:00 ngày 16 — `<=` gồm mốc cuối, nên chuyến đó bị đếm vào hai ngày.
- **d** đúng ở phiên VN và sai ở phiên UTC: `::date` cắt ngày theo `TimeZone` của **phiên**. Tương tự, mốc không ghi múi giờ ở e, f được hiểu theo phiên (tài liệu: chuỗi không nêu múi giờ thì «assumed to be in the time zone indicated by the system's TimeZone parameter»). Ở phiên UTC cả ba dồn về 105 nhưng vì lý do khác: ngày 15/10 UTC là 07:00 ngày 15 tới 06:59 ngày 16 giờ VN.

Chỉ a, b, c không đổi theo phiên — cả ba **ghi rõ múi giờ** trong câu. Thí nghiệm B của `pg016-mui-gio.sql` thêm ba cách hay viết nữa, cũng phụ thuộc phiên (106 ở phiên VN, 105 ở phiên UTC): `date_trunc('day', requested_at)` **hai tham số**, so sánh với hằng `DATE '2026-10-15'` (PostgreSQL ép `date` thành nửa đêm **theo phiên**), và ép `requested_at::timestamp`. Dạng `date_trunc` ba tham số (có múi giờ) có từ **PostgreSQL 12** (release notes 12: «additional argument to control the time zone»).

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Phép thử một dòng</span>
<p>Chạy câu lọc ngày hai lần: một lần sau <code>SET TIME ZONE 'Asia/Ho_Chi_Minh'</code>, một lần sau <code>SET TIME ZONE 'UTC'</code>. Kết quả đổi ⇒ câu đang dựa vào phiên, và sẽ sai trên công cụ BI, trên job chạy ở container, hoặc trên máy đồng nghiệp ở múi giờ khác.</p>
</div>

Giữa a, b và c nên chọn a cho bộ lọc: so sánh trực tiếp trên cột, không biến đổi cột ở mỗi dòng — có lợi khi có index (Phần 3). b và c hợp cho **gom nhóm** (mục 5).

## 4. Kiểm tra hai biên

Một bộ lọc ngày đúng phải **giữ** chuyến đúng 00:00:00 ngày 15 và chuyến 23:59:59.5, **loại** chuyến đúng 00:00:00 ngày 16. In năm chuyến biên với kết quả của bộ lọc nửa mở (a) và hai bộ lọc sai (cả hai đã ghi `+07` để loại yếu tố phiên):

```sql
SELECT code, requested_at,
       requested_at >= '2026-10-15 00:00:00+07' AND requested_at < '2026-10-16 00:00:00+07'  AS a_nua_mo,
       requested_at BETWEEN '2026-10-15 00:00:00+07' AND '2026-10-15 23:59:59+07'            AS e_between,
       requested_at >= '2026-10-15 00:00:00+07' AND requested_at <= '2026-10-16 00:00:00+07' AS f_nho_hon_bang
FROM trips
WHERE code IN (101489, 101502, 101503, 101594, 101595)
ORDER BY code;
```

```text
  code  |       requested_at       | a_nua_mo | e_between | f_nho_hon_bang
--------+--------------------------+----------+-----------+----------------
 101489 | 2026-10-15 00:00:00+07   | t        | t         | t
 101502 | 2026-10-15 06:59:59+07   | t        | t         | t
 101503 | 2026-10-15 07:00:00+07   | t        | t         | t
 101594 | 2026-10-15 23:59:59.5+07 | t        | f         | t
 101595 | 2026-10-16 00:00:00+07   | f        | f         | t
(5 rows)
```

Hai chuyến 101502/101503 nằm ở biên **ngày UTC** (07:00 giờ VN = 00:00 UTC) — biên mà một bộ lọc dựa vào phiên UTC sẽ cắt nhầm. Bộ dữ liệu kiểm thử của mọi báo cáo theo ngày nên có đủ bốn loại điểm: đầu ngày, ngay trước cuối ngày (có phần lẻ giây), đúng đầu ngày sau, và hai phía của nửa đêm UTC.

### Dựng biên từ tham số ngày

Ứng dụng thường truyền vào một **ngày** (`date`), không phải hai mốc. Dựng biên trong SQL mà không phụ thuộc phiên:

```sql
SELECT make_timestamptz(2026, 10, 15, 0, 0, 0, 'Asia/Ho_Chi_Minh')          AS tu,
       (DATE '2026-10-15')::timestamp AT TIME ZONE 'Asia/Ho_Chi_Minh'      AS tu_cach_2,
       (DATE '2026-10-15' + 1)::timestamp AT TIME ZONE 'Asia/Ho_Chi_Minh'  AS den_truoc;
```

Cả hai cách đều cho ra `2026-10-15 00:00:00+07`, biên trên `2026-10-16 00:00:00+07`, và lọc nửa mở bằng chúng đếm đúng **106**. Cách 2 là chiều thứ hai của bảng mục 2: `date` → `timestamp` nửa đêm → «nửa đêm đó ở Việt Nam là khoảnh khắc nào».

## 5. Gom theo ngày, tuần, tháng

Gom theo **ngày VN** và theo **ngày UTC** lệch nhau ở mọi chuyến đặt 00:00–06:59 giờ VN:

```text
    ngay    | theo_ngay_vn | theo_ngay_utc
------------+--------------+---------------
 2026-10-14 |          100 |           102
 2026-10-15 |          106 |           105
 2026-10-16 |          103 |           104
(3 rows)
```

Tổng ba ngày gần bằng nhau nhưng từng ngày đều sai — loại lỗi khó thấy nhất trên biểu đồ.

**Tháng.** Kỳ «01/10 → hết 30/10» theo ngày VN có **3.168** chuyến; gom theo các ngày UTC tương ứng (30/09 → 30/10) ra **3.174** — thừa 6 chuyến đang chạy đặt 00:30–00:58 ngày 31/10 giờ VN, vốn là ngày 30/10 theo UTC. Ở đầu tháng thì ngược lại: tháng 10 theo UTC bắt đầu lúc 07:00 sáng 01/10 giờ VN, nên đếm cả tháng theo UTC được **3.162** — thiếu đúng **12** chuyến đặt trước 07:00 ngày 01/10 (bài tập 3). `date_trunc('month', TIMESTAMPTZ '2026-11-01 00:30:00+07', 'UTC')` trả `2026-10-01 07:00:00+07`: chuyến nửa đêm 01/11 giờ VN vẫn thuộc tháng 10 UTC (thí nghiệm D).

**Tuần.** `date_trunc('week', …)` đưa mốc về **00:00 thứ hai**. Thí nghiệm C gom theo tuần giờ VN:

```text
        dau_tuan        | so_chuyen
------------------------+-----------
 2026-09-28 00:00:00+07 |       440
 2026-10-05 00:00:00+07 |       751
 2026-10-12 00:00:00+07 |       748
 2026-10-19 00:00:00+07 |       728
 2026-10-26 00:00:00+07 |       507
(5 rows)
```

01/10/2026 là thứ năm (`extract(isodow …)` = 4), nên «tuần bắt đầu 28/09» chỉ có bốn ngày tháng 10 — so tuần đầu với tuần đủ là so khập khiễng. Tuần cuối gồm cả 6 chuyến đang chạy sáng 31/10. Hai cách đánh số thứ trong tuần: `isodow` thứ hai = 1 … chủ nhật = 7; `dow` chủ nhật = 0. Mã tuần ISO: `to_char(DATE '2026-10-12', 'IYYY-"W"IW')` = `2026-W42`.

## 6. Giờ trong ngày

`extract(hour FROM requested_at)` dùng múi giờ **phiên**. Viết `extract(hour FROM requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh')`: giờ 0–6 lần lượt ra 68 · 28 · 39 · 39 · 25 · 61 · 123 chuyến, cộng lại **383** — đúng số chuyến lệch ngày UTC ở mục 5. Giờ cao điểm (thí nghiệm E): theo giờ VN là **18 giờ** (289 chuyến), 12 giờ (280), 11 giờ (255); gom theo giờ UTC thì «giờ cao điểm» trở thành 11, 5 và 4 giờ — đúng số chuyến, sai nhãn, và ai đọc biểu đồ sẽ xếp ca tài xế lệch 7 tiếng.

## 7. Lịch ngày cho báo cáo

Báo cáo theo ngày cần một **lịch ngày đầy đủ** để ngày không có chuyến vẫn có dòng (PG-015 dùng nó cho trung bình 7 ngày). Hãy chú ý kiểu dữ liệu:

```sql
SELECT pg_typeof(g), pg_typeof(g::date), pg_typeof(DATE '2026-10-01' + 1)
FROM generate_series(DATE '2026-10-01', DATE '2026-10-02', interval '1 day') AS g
LIMIT 1;
```

```text
   kieu_generate_series   | sau_khi_ep | date_cong_so_nguyen
--------------------------+------------+---------------------
 timestamp with time zone | date       | date
(1 row)
```

`generate_series(date, date, interval)` trả **`timestamptz`** — nửa đêm theo phiên — không phải `date`. Đem kết quả so với cột `date` thì PostgreSQL ép kiểu ngầm: đúng ở phiên VN, lệch ở phiên khác. Dùng `DATE '2026-10-01' + n` với `generate_series(0, 29)`, hoặc ép `::date` ngay khi sinh. Lịch + `LEFT JOIN` với biên dựng như mục 4 cho ra, với Đà Nẵng, 19/10: 13 chuyến · 20/10 (bão): **0** · 21/10: 17 — ngày bão vẫn có dòng.

## 8. Ghi múi giờ bằng tên, không bằng độ lệch

Câu `AT TIME ZONE '+07'` ở đầu bài (thí nghiệm A):

```sql
SELECT code,
       requested_at AT TIME ZONE 'UTC+7'            AS chuoi_utc_cong_7,
       requested_at AT TIME ZONE '+07'              AS chuoi_cong_07,
       requested_at AT TIME ZONE INTERVAL '+07:00'  AS interval_cong_07,
       requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh' AS ten_mui_gio
FROM trips WHERE code = 101503;
```

```text
  code  |  chuoi_utc_cong_7   |    chuoi_cong_07    |  interval_cong_07   |     ten_mui_gio
--------+---------------------+---------------------+---------------------+---------------------
 101503 | 2026-10-14 17:00:00 | 2026-10-14 17:00:00 | 2026-10-15 07:00:00 | 2026-10-15 07:00:00
(1 row)
```

Chuỗi `'UTC+7'` và `'+07'` được hiểu theo **quy ước POSIX**; tài liệu PostgreSQL 17 (mục B.5) ghi: dấu dương dùng cho múi giờ **phía tây** Greenwich, «ngược với quy ước ISO-8601 dùng ở những chỗ khác trong PostgreSQL». Vậy `'+07'` là UTC−7, và 07:00 sáng 15/10 giờ VN bị đổi thành 17:00 chiều 14/10. `SET TIME ZONE 'UTC+7'` cũng vậy: chuyến 101503 hiện `2026-10-14 17:00:00-07`. Nhưng `SET TIME ZONE '+07'` — một **số** — lại là phía đông (tài liệu `SET`: «Positive values are east from UTC») và hiện đúng `+07`. Cùng ký tự, hai nghĩa ngược nhau giữa `SET` và `AT TIME ZONE`.

Lý do thứ hai để dùng tên là **giờ mùa hè**. Việt Nam không có, nhưng New York lùi đồng hồ ngày 01/11/2026:

```sql
SET TIME ZONE 'America/New_York';
SELECT TIMESTAMPTZ '2026-11-01 00:00:00 America/New_York'                       AS bat_dau,
       TIMESTAMPTZ '2026-11-01 00:00:00 America/New_York' + interval '1 day'    AS cong_1_ngay,
       TIMESTAMPTZ '2026-11-01 00:00:00 America/New_York' + interval '24 hours' AS cong_24_gio;
```

```text
        bat_dau         |      cong_1_ngay       |      cong_24_gio
------------------------+------------------------+------------------------
 2026-11-01 00:00:00-04 | 2026-11-02 00:00:00-05 | 2026-11-01 23:00:00-05
(1 row)
```

Ngày đó dài 25 giờ: «+1 ngày» tới nửa đêm hôm sau, «+24 giờ» dừng ở 23:00. Tên múi giờ mang theo lịch sử đổi giờ; một độ lệch cố định thì không. Ở giờ VN hai phép cộng ra cùng `2026-10-16 00:00:00+07` — nhưng mã viết hôm nay có thể chạy cho thị trường khác ngày mai.

## 9. Interval và phép cộng tháng

Hiệu hai `timestamptz` là một **`interval`**. Thời gian trung bình từ lúc đặt tới lúc hoàn thành (`avg(completed_at - requested_at)`): food `00:29:00`, parcel `00:28:00`, ride `00:28:24` — đổi ra phút bằng `extract(epoch FROM …) / 60`: 29,0 · 28,0 · 28,4. Khoảng `1 day 06:33:00` giữa hai chuyến ở PG-015 bằng **30,55 giờ** (thí nghiệm G); khi **so sánh** interval, PostgreSQL coi 1 ngày = 24 giờ, nhưng khi **cộng** vào `timestamptz` thì không (New York ở trên).

Cộng **tháng** có luật riêng (thí nghiệm F): `DATE '2026-01-31' + interval '1 month'` = **28/02** (ngày bị kẹp về cuối tháng đích, đúng như tài liệu mô tả); cộng thêm một tháng nữa ra **28/03**, còn `+ interval '2 months'` từ 31/01 ra **31/03**. Kỳ thanh toán hằng tháng phải tính từ mốc gốc (`gốc + n tháng`), không cộng dồn từ kỳ trước.

**`now()`** trả thời điểm **bắt đầu giao dịch** (tài liệu: `now()` tương đương `transaction_timestamp()`; muốn giờ thật đang chạy thì `clock_timestamp()`). Báo cáo nên nhận mốc «tính đến» làm tham số — như fixture chốt ở 01:00 ngày 31/10 — thay vì `now()`, để chạy lại cho cùng kết quả.

## 10. Lỗi thường gặp

| Triệu chứng | Nguyên nhân | Cách sửa |
|---|---|---|
| Hai người đếm cùng ngày ra hai số | `::date`, `date_trunc` 2 tham số, mốc không ghi múi giờ ⇒ theo phiên | Mốc ghi `+07` hoặc dựng bằng `AT TIME ZONE 'Asia/Ho_Chi_Minh'` |
| Thiếu chuyến cuối ngày | `BETWEEN … 23:59:59` | Nửa mở `>= từ AND < đến` |
| Một chuyến thuộc hai ngày | `<=` mốc đầu ngày sau | Nửa mở |
| Tổng tháng lệch vài chuyến | Gom theo ngày/tháng UTC | `date_trunc('month', …, 'Asia/Ho_Chi_Minh')` |
| Giờ lệch 14 tiếng | `AT TIME ZONE '+07'` / `'UTC+7'` (POSIX, phía tây) | Tên `'Asia/Ho_Chi_Minh'` hoặc `INTERVAL '+07:00'` |
| Lịch ngày lệch ở phiên khác | `generate_series(date, …)` trả `timestamptz` | `DATE + n` hoặc `::date` |
| Kỳ tháng trôi về 28 | Cộng dồn `+ interval '1 month'` | Tính từ mốc gốc |

## 11. Đánh đổi

- **Lọc bằng mốc hay bằng biểu thức trên cột:** mốc nửa mở (a) để nguyên cột, hợp với index (Phần 3); `(… AT TIME ZONE …)::date = …` (b) dễ đọc, hợp để gom nhóm, nhưng biến đổi mọi dòng.
- **Lưu thêm cột «ngày nghiệp vụ» `date`:** gom nhanh và không còn lo chuyện phụ thuộc phiên, nhưng phải giữ đồng bộ với `requested_at` và với định nghĩa ngày (ngày vận hành bắt đầu 04:00 thì sao — bài tập 2?). Cột sinh tự động là một lựa chọn; đánh giá ở Phần 3 khi có số đo.
- **`AT TIME ZONE` hay `date_trunc` 3 tham số:** hai cách cho cùng ngày VN; `date_trunc` giữ kiểu `timestamptz` (so được với mốc `+07`), `AT TIME ZONE … ::date` ra `date` (so được với tham số ngày).

## 12. Bài tập

**Bài 1.** Đếm chuyến đặt trong **tuần 12–18/10 giờ VN** (thứ hai → chủ nhật) bằng bộ lọc nửa mở, đối chiếu bằng `date_trunc('week', …)` ba tham số, rồi chạy lại ở phiên UTC.

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">SELECT count(*) FILTER (WHERE requested_at &gt;= '2026-10-12 00:00:00+07'
                          AND requested_at &lt;  '2026-10-19 00:00:00+07')                      AS nua_mo,
       count(*) FILTER (WHERE date_trunc('week', requested_at, 'Asia/Ho_Chi_Minh')
                              = '2026-10-12 00:00:00+07')                                    AS date_trunc_tuan
FROM trips;</code></pre>
<p>Kết quả: 748 | 748 ở cả phiên VN lẫn phiên UTC — cả hai cách đều ghi rõ múi giờ. Mốc cuối là 00:00 thứ hai 19/10, không phải 23:59:59 chủ nhật.</p>
</details>

**Bài 2.** Ca đêm định nghĩa «**ngày vận hành**» bắt đầu lúc **04:00** giờ VN. Đếm chuyến của ngày vận hành 15/10, viết biểu thức gom chuyến theo ngày vận hành, và kiểm hai biên của biểu thức.

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">-- lọc: nửa mở, hai mốc 04:00
SELECT count(*) FROM trips
WHERE requested_at &gt;= '2026-10-15 04:00:00+07' AND requested_at &lt; '2026-10-16 04:00:00+07';
-- gom: lùi 4 giờ trên giờ VN rồi lấy ngày
SELECT ((requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh') - interval '4 hours')::date AS ngay_van_hanh,
       count(*)
FROM trips
WHERE requested_at &gt;= '2026-10-15 00:00:00+07' AND requested_at &lt; '2026-10-17 04:00:00+07'
GROUP BY 1 ORDER BY 1;</code></pre>
<p>Kết quả: <b>107</b> chuyến (khác 106 của ngày lịch); câu gom ra 14/10: 6 · 15/10: 107 · 16/10: 102 — dòng 15/10 khớp câu lọc. Dữ liệu không có chuyến nào đúng 04:00, nên kiểm tra biên bằng giá trị dựng tay (<code>VALUES</code> trong <code>pg016-bai-tap.sql</code>): 15/10 03:59:59.5 → ngày vận hành 14/10; 15/10 04:00:00 → 15/10; 16/10 03:59:59.5 → 15/10; 16/10 04:00:00 → 16/10.</p>
</details>

**Bài 3.** Ứng dụng truyền tham số tháng là ngày đầu tháng (`DATE '2026-10-01'`). Dựng hai biên tháng giờ VN trong SQL, đếm chuyến, rồi so với «tháng 10 theo UTC». Giải thích chênh lệch.

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">WITH tham_so AS (SELECT DATE '2026-10-01' AS thang),
     bien AS (
       SELECT thang::timestamp AT TIME ZONE 'Asia/Ho_Chi_Minh'                         AS tu,
              (thang + interval '1 month')::timestamp AT TIME ZONE 'Asia/Ho_Chi_Minh' AS den_truoc
       FROM tham_so)
SELECT b.tu, b.den_truoc,
       (SELECT count(*) FROM trips WHERE requested_at &gt;= b.tu AND requested_at &lt; b.den_truoc) AS thang_vn,
       (SELECT count(*) FROM trips WHERE requested_at &gt;= '2026-10-01 00:00:00+00'
                                     AND requested_at &lt;  '2026-11-01 00:00:00+00')            AS thang_utc
FROM bien AS b;</code></pre>
<p>Kết quả: biên <code>2026-10-01 00:00:00+07</code> → <code>2026-11-01 00:00:00+07</code>, tháng VN <b>3.174</b> (gồm 6 chuyến đang chạy sáng 31/10), tháng UTC <b>3.162</b>. Tháng UTC bắt đầu 07:00 sáng 01/10 giờ VN, nên thiếu đúng 12 chuyến đặt 00:00–06:59 ngày 01/10 (lab đếm riêng: 12). Ở cuối tháng, chuyến đặt 00:00–06:59 ngày 01/11 giờ VN sẽ bị gán nhầm vào tháng 10 UTC — dữ liệu chụp lúc 31/10 chưa có chuyến nào như vậy.</p>
</details>

## 13. Đọc tiếp

- [PostgreSQL 17 · Date/Time Types](https://www.postgresql.org/docs/17/datatype-datetime.html) — cách hiểu chuỗi không ghi múi giờ, tên và viết tắt múi giờ.
- [PostgreSQL 17 · Date/Time Functions and Operators](https://www.postgresql.org/docs/17/functions-datetime.html) — `AT TIME ZONE`, `date_trunc`, cộng tháng, `now()`.
- [PostgreSQL 17 · POSIX Time Zone Specifications](https://www.postgresql.org/docs/17/datetime-posix-timezone-specs.html) — quy ước dấu ngược ISO.
- [PostgreSQL 17 · SET](https://www.postgresql.org/docs/17/sql-set.html) — `SET TIME ZONE` với số và interval.
- [PostgreSQL 12 · Release Notes](https://www.postgresql.org/docs/12/release-12.html) — `date_trunc` có tham số múi giờ.
- Góc nhìn phân tích: [Ngày giờ, múi giờ và cửa sổ báo cáo](/articles/da-ngay-gio-timezone-cua-so-bao-cao) (series DA).
- Lab: `p02/16-date-time-timezone.sql`, `p02/thu-nghiem/pg016-mui-gio.sql`, `p02/thu-nghiem/pg016-bai-tap.sql`; kết quả đầy đủ ở `p02/OUTPUT.txt`.

Bài tiếp theo: [PG-017 · UPSERT và xung đột khoá duy nhất](/articles/postgresql-upsert-on-conflict).
