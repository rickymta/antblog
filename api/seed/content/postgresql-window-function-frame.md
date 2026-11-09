---
slug: postgresql-window-function-frame
title: "Window function và cửa sổ tính toán: luỹ kế, xếp hạng và frame"
summary: "Sau ngày bão 20/10, biểu đồ «trung bình 7 ngày» của Tới Nơi Đà Nẵng báo 797.143 đ trong khi bảng kế hoạch ghi 677.286 đ, còn «top 3 tài xế» có bốn người. Học PARTITION BY, ORDER BY và frame ROWS/RANGE/GROUPS để luỹ kế, xếp hạng và so sánh chuyến trước có đúng một nghĩa."
category: postgresql
level: intermediate
tags: [postgresql, pg-015, window-function, frame, rank, lag, toi-noi]
language: vi
---

Sáng 31/10, đội vận hành Tới Nơi ở Đà Nẵng xem lại tuần có bão: ngày 20/10 không có chuyến nào. Dashboard ghi «doanh thu trung bình 7 ngày» của 21/10 là **797.143 đ**; bảng kế hoạch của trưởng vùng, dựng tay trên lịch đủ 7 ngày, ghi **677.286 đ**. Cùng buổi họp, «top 3 tài xế» của Đà Nẵng có **bốn** tên, và luỹ kế chuyến của khách doanh nghiệp nhảy từ 3 lên 9 trong một dòng.

Không câu nào lỗi cú pháp. Cả ba dùng **window function** — hàm tính trên một nhóm dòng «xung quanh» dòng hiện tại — và cả ba để PostgreSQL tự chọn **frame**, tức tập dòng thật sự đưa vào phép tính. Bài này đi qua ba phần của `OVER (…)`, frame mặc định khi thứ tự có hoà, rồi viết lại luỹ kế, trung bình trượt, xếp hạng và so sánh với dòng trước sao cho con số không phụ thuộc may rủi.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-015 · Phần 2/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/postgresql-aggregate-group-by-having">PG-012 · GROUP BY và HAVING</a>, <a href="/articles/postgresql-subquery-exists-cte">PG-014 · Subquery, EXISTS và CTE</a>. Lab: <code>examples/toi-noi-postgresql/p02</code> (tệp <code>15-window-function.sql</code>) và thí nghiệm thêm <code>p02/thu-nghiem/pg015-*.sql</code>. Sau bài này, bạn tính được luỹ kế, xếp hạng và so sánh chuyến trước với frame ghi rõ.</p>
</div>

## 1. Dữ liệu dùng trong bài

Fixture Phần 2 là ảnh chụp lúc **01:00 ngày 31/10/2026 giờ VN**: 3.174 chuyến (mã 100000–103173 tăng theo giờ đặt), 2.707 chuyến hoàn thành, tổng cước 182.287.000 đ. Ba điểm cài sẵn cho bài này: Đà Nẵng **0 chuyến** ngày bão 20/10; khách doanh nghiệp có hai lô đặt **cùng giây** (10/10 09:00:00 × 6, 18/10 14:30:00 × 4); tài xế 8101 và 8109 cùng 68 chuyến hoàn thành.

```bash
cd examples/toi-noi-postgresql
node run.mjs p01/00-reset.sql p01/01-schema.sql p02/00-mo-rong-schema.sql \
  p02/01-fixture.sql p02/02-kiem-tra-fixture.sql p02/15-window-function.sql
```

Thí nghiệm thêm chạy trên cùng nền: thay tệp cuối bằng `p02/thu-nghiem/pg015-khung-cua-so.sql` hoặc `p02/thu-nghiem/pg015-bai-tap.sql`. Kết quả trong bài chép từ lần chạy thật trên PGlite 0.3.16 (PostgreSQL 17.5), bỏ dòng `-- [lab]`; chưa chạy trên server thật. Mỗi tệp đặt `search_path` = `toinoi`, `TimeZone` = `Asia/Ho_Chi_Minh`.

## 2. Window function khác GROUP BY ở đâu?

`GROUP BY` **gộp** dòng: 2.707 chuyến thành 3 dòng thành phố. **Window function** tính trên một tập dòng liên quan nhưng **giữ nguyên từng dòng**; nhận ra nó qua mệnh đề `OVER (…)`. Tỉ trọng doanh thu cần cả số của dòng lẫn tổng mọi dòng:

```sql
SELECT c.code AS tp,
       sum(t.fare)                                              AS doanh_thu,
       round(100.0 * sum(t.fare) / sum(sum(t.fare)) OVER (), 2) AS ty_trong_pct
FROM trips AS t JOIN cities AS c ON c.id = t.city_id
WHERE t.status = 'completed'
GROUP BY c.code
ORDER BY doanh_thu DESC;
```

Kết quả: SG 92.880.000 (50,95 %), HN 62.991.000 (34,56 %), DN 26.416.000 (14,49 %); câu trong lab in thêm cột `tong` = 182.287.000. `sum(sum(t.fare)) OVER ()` đọc được nếu nhớ **thứ tự logic**:

`FROM`/`JOIN` → `WHERE` → `GROUP BY` → `HAVING` → **window function** → danh sách cột → `ORDER BY` → `LIMIT`.

Cửa sổ chạy **sau** khi gom nhóm: `sum(t.fare)` bên trong là hàm gộp, `sum(…) OVER ()` bên ngoài cộng ba dòng đã gom; `OVER ()` rỗng = toàn bộ kết quả. Cũng vì thế tài liệu PostgreSQL ghi: window function chỉ được dùng ở danh sách cột và `ORDER BY`.

```sql
hàm(...) OVER (
  PARTITION BY ...   -- chia nhóm, mỗi nhóm tính riêng (bỏ trống = một nhóm)
  ORDER BY ...       -- thứ tự trong nhóm
  frame              -- dòng nào trong nhóm được tính cho dòng hiện tại
)
```

**Hàm gộp** (`sum`, `avg`, `count`…) đi với `OVER` thì tính trên **frame**. **Hàm cửa sổ thuần** (`row_number`, `rank`, `lag`, `lead`…) tính trên **partition** theo thứ tự — trừ `first_value`/`last_value`/`nth_value`, vốn tính trên frame (bẫy ở mục 7).

## 3. Luỹ kế: ghi frame ra

Câu 1 của `15-window-function.sql` dựng **lịch đủ 30 ngày** (`DATE '2026-10-01' + n`) rồi `LEFT JOIN` doanh thu theo ngày của Đà Nẵng, để ngày bão có dòng 0 thay vì biến mất. Luỹ kế ghi frame tường minh:

```sql
sum(coalesce(d.doanh_thu, 0)) OVER (ORDER BY l.ngay
                                    ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) AS luy_ke
```

`ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW` = mọi dòng từ đầu partition tới chính dòng này. In 17–24/10 (hai cột cuối dùng ở mục 4):

```text
    ngay    | so_chuyen | doanh_thu |  luy_ke  | tb7_lich_day | tb7_rows_thua
------------+-----------+-----------+----------+--------------+---------------
 2026-10-17 |        13 |    718000 | 15952000 |       927429 |        927429
 2026-10-18 |        16 |    945000 | 16897000 |       859857 |        859857
 2026-10-19 |        11 |    637000 | 17534000 |       828571 |        828571
 2026-10-20 |         0 |         0 | 17534000 |       672571 |
 2026-10-21 |        14 |    872000 | 18406000 |       677286 |        797143
 2026-10-22 |        11 |    785000 | 19191000 |       679000 |        789429
 2026-10-23 |        13 |    941000 | 20132000 |       699714 |        813429
 2026-10-24 |        14 |    787000 | 20919000 |       709571 |        812143
(8 rows)
```

**Bẫy lọc cùng cấp.** Câu trên tính cửa sổ trên cả tháng ở truy vấn trong, truy vấn ngoài mới lọc 17–24/10. Đặt `WHERE ngay BETWEEN '2026-10-17' AND '2026-10-24'` ngay trong câu có cửa sổ (`pg015-khung-cua-so.sql`, mục A) thì `WHERE` chạy trước: «luỹ kế» bắt đầu lại từ **718.000** (17/10), 1.663.000, 2.300.000… và ngày 20/10 biến mất. Quy tắc: lọc **phạm vi tính** ở truy vấn trong, lọc **phạm vi hiển thị** ở ngoài.

## 4. Trung bình 7 ngày trên dữ liệu thưa

Hai cột cuối trả lời câu hỏi mở bài:

- `tb7_lich_day` = `avg(…) OVER (ORDER BY l.ngay ROWS BETWEEN 6 PRECEDING AND CURRENT ROW)` trên **lịch đủ**: 7 dòng = 7 ngày. Ngày 21/10: (773 + 796 + 718 + 945 + 637 + 0 + 872) nghìn ÷ 7 = **677.286 đ** — số của bảng kế hoạch.
- `tb7_rows_thua`: **cùng frame** nhưng trên dữ liệu gốc, nơi 20/10 không có dòng. «7 dòng» của 21/10 trải từ 14/10 tới 21/10 — **8 ngày lịch** — và ngày bão không được tính là 0: (839 + 773 + 796 + 718 + 945 + 637 + 872) nghìn ÷ 7 = **797.143 đ** — số của dashboard.

`ROWS` đếm **dòng vật lý**; nó chỉ trùng «ngày» khi mỗi ngày có đúng một dòng.

### RANGE theo khoảng thời gian

Không dựng lịch thì đổi chế độ frame:

```sql
sum(doanh_thu) OVER (ORDER BY ngay RANGE BETWEEN interval '6 days' PRECEDING AND CURRENT ROW)
```

`RANGE … interval '6 days' PRECEDING` chọn mọi dòng có `ngay` trong [ngày hiện tại − 6 ngày, ngày hiện tại] — ranh giới theo **giá trị**, không theo số dòng. Câu 2 của tệp lab lấy tổng đó **chia 7** và so với `tb7_lich_day` ở mọi ngày có dữ liệu: **0** ngày lệch.

Nhưng để nguyên `avg()` trên cùng frame (thí nghiệm B, trích 4/8 dòng):

```text
    ngay    | ngay_dau_khung | so_dong_trong_khung | avg_range_sai | tb7_dung
------------+----------------+---------------------+---------------+----------
 2026-10-19 | 2026-10-13     |                   7 |        828571 |   828571
 2026-10-21 | 2026-10-15     |                   6 |        790167 |   677286
 2026-10-26 | 2026-10-21     |                   6 |        925333 |   793143
 2026-10-27 | 2026-10-21     |                   7 |        918143 |   918143
```

Ranh giới đúng, nhưng suốt 21–26/10 khung chỉ có **6 dòng** và `avg` chia 6: ra 790.167 đ — đáp án thứ ba, cũng sai. `RANGE` sửa **ranh giới**, không biến ngày không có dòng thành 0.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Một câu hỏi, ba đáp án</span>
<p>Ngày 21/10 Đà Nẵng: <code>ROWS</code> trên dữ liệu thưa 797.143 đ · <code>avg</code> trên <code>RANGE '6 days'</code> 790.167 đ · lịch đủ (hoặc tổng <code>RANGE</code> ÷ 7) 677.286 đ. Chỉ số cuối đúng nghĩa «trung bình một ngày trong 7 ngày gần nhất». Trước khi viết trung bình trượt, hỏi: dữ liệu có thể thiếu ngày không?</p>
</div>

`RANGE` có khoảng cách, chế độ `GROUPS` và mệnh đề loại trừ `EXCLUDE` có từ **PostgreSQL 11** (release notes: mọi tuỳ chọn frame của SQL:2011). `RANGE` có khoảng cách cần **đúng một** cột `ORDER BY`; với cột `date`/`timestamp`, khoảng cách là một `interval`.

## 5. Frame mặc định khi thứ tự có hoà

Luỹ kế 18 chuyến của khách doanh nghiệp (câu 2 của tệp lab), trích quanh lô 10/10:

```sql
SELECT code, requested_at, fare,
       count(*)  OVER (ORDER BY requested_at)                                AS dem_mac_dinh,
       sum(fare) OVER (ORDER BY requested_at)                                AS luy_ke_mac_dinh,
       count(*)  OVER (ORDER BY requested_at, code ROWS UNBOUNDED PRECEDING) AS dem_rows,
       sum(fare) OVER (ORDER BY requested_at, code ROWS UNBOUNDED PRECEDING) AS luy_ke_rows
FROM trips
WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
ORDER BY requested_at, code;
```

```text
  code  |      requested_at      | fare  | dem_mac_dinh | luy_ke_mac_dinh | dem_rows | luy_ke_rows
--------+------------------------+-------+--------------+-----------------+----------+-------------
 100669 | 2026-10-07 09:20:00+07 | 57000 |            3 |          177000 |        3 |      177000
 100971 | 2026-10-10 09:00:00+07 | 24000 |            9 |          462000 |        4 |      201000
 100972 | 2026-10-10 09:00:00+07 | 53000 |            9 |          462000 |        5 |      254000
 …
 100976 | 2026-10-10 09:00:00+07 | 61000 |            9 |          462000 |        9 |      462000
 101224 | 2026-10-12 11:05:00+07 | 27000 |           10 |          489000 |       10 |      489000
 …
 102897 | 2026-10-28 10:30:00+07 | 27000 |           18 |          809000 |       18 |      809000
(18 rows)
```

`dem_mac_dinh` nhảy 3 → **9**: cả sáu chuyến của lô đều là «chuyến thứ 9», cùng luỹ kế 462.000. Đó là **frame mặc định**: theo tài liệu PostgreSQL 17, nó là `RANGE BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW` và khi có `ORDER BY` thì kéo tới **dòng ngang hàng cuối cùng** của dòng hiện tại. Dòng **ngang hàng** (peer) bằng nhau trên mọi khoá `ORDER BY` — ở đây sáu chuyến cùng giây. Không có `ORDER BY` thì mọi dòng ngang hàng, nên `OVER ()` ra tổng.

Hai cột `_rows` thêm khoá phụ `code` và ghi `ROWS UNBOUNDED PRECEDING`: tăng từng dòng, thứ tự xác định. Dòng cuối của hai cách luôn khớp (18 chuyến, 809.000 đ), nên lỗi hay lọt qua kiểm «tổng cuối kỳ». Cần **cả hai** thay đổi: chỉ đổi sang `ROWS` mà thiếu `code` thì luỹ kế tăng từng dòng nhưng thứ tự trong lô tuỳ PostgreSQL (cùng lý do tie-breaker ở PG-007).

### ROWS, RANGE và GROUPS cạnh nhau

Thí nghiệm D dùng cùng «1 PRECEDING» trên 18 chuyến đó. Ở chuyến 100971, đầu lô 10/10:

| Chế độ | «1 PRECEDING» nghĩa là | `sum(fare)` |
|---|---|---|
| `ROWS` (khoá `requested_at, code`) | 1 **dòng** trước + dòng hiện tại | 57.000 + 24.000 = 81.000 |
| `GROUPS` | 1 **nhóm ngang hàng** trước + cả nhóm hiện tại | 57.000 + cả lô 285.000 = 342.000 |
| `RANGE` với `interval '1 hour'` | mọi dòng có **giá trị** khoá cách ≤ 1 giờ | chỉ cả lô = 285.000 |

Với `GROUPS` và `RANGE`, sáu chuyến trong lô luôn chung một khung nên chung một số; `ROWS` thì khác từng dòng. `RANGE` có khoảng cách chỉ nhận một cột sắp xếp — thêm `code` là lỗi `42P20` `RANGE with offset PRECEDING/FOLLOWING requires exactly one ORDER BY column`; `GROUPS` thiếu `ORDER BY` cũng `42P20` (thí nghiệm E).

Dạng đầy đủ: `{ROWS | RANGE | GROUPS} BETWEEN điểm_đầu AND điểm_cuối [EXCLUDE …]`, mỗi điểm là `UNBOUNDED PRECEDING`, `n PRECEDING`, `CURRENT ROW`, `n FOLLOWING` hoặc `UNBOUNDED FOLLOWING` (điểm đầu không được là `UNBOUNDED FOLLOWING`). `EXCLUDE CURRENT ROW | GROUP | TIES` bỏ dòng hiện tại và/hoặc dòng ngang hàng khỏi frame.

## 6. Xếp hạng: row_number, rank, dense_rank

Lần thử đầu của «top 3 tài xế mỗi thành phố» hay là lọc hạng trong `WHERE`:

```sql
SELECT d.code, rank() OVER (ORDER BY count(*) DESC) AS hang
FROM trips AS t JOIN drivers AS d ON d.id = t.driver_id
WHERE t.status = 'completed' AND rank() OVER (ORDER BY count(*) DESC) <= 3
GROUP BY d.code;
```

```text
ERROR:  window functions are not allowed in WHERE
```

SQLSTATE `42P20`: `WHERE` chạy trước khi có hạng. Tính hạng ở truy vấn con (hoặc CTE, PG-014) rồi lọc ngoài:

```sql
SELECT tp, tai_xe, hoan_thanh, so_thu_tu, hang, hang_lien_tuc
FROM (
  SELECT c.code AS tp, d.code AS tai_xe, count(*) AS hoan_thanh,
         row_number() OVER (PARTITION BY c.code ORDER BY count(*) DESC, d.code) AS so_thu_tu,
         rank()       OVER w AS hang,
         dense_rank() OVER w AS hang_lien_tuc
  FROM trips AS t
  JOIN drivers AS d ON d.id = t.driver_id
  JOIN cities  AS c ON c.id = d.city_id
  WHERE t.status = 'completed'
  GROUP BY c.code, d.code
  WINDOW w AS (PARTITION BY c.code ORDER BY count(*) DESC)
) AS x
WHERE hang_lien_tuc <= 3
ORDER BY tp COLLATE "C", so_thu_tu;
```

```text
 tp | tai_xe | hoan_thanh | so_thu_tu | hang | hang_lien_tuc
----+--------+------------+-----------+------+---------------
 DN |   8305 |         44 |         1 |    1 |             1
 DN |   8304 |         41 |         2 |    2 |             2
 DN |   8302 |         40 |         3 |    3 |             3
 DN |   8310 |         40 |         4 |    3 |             3
 HN |   8210 |         63 |         1 |    1 |             1
 …
 SG |   8101 |         68 |         1 |    1 |             1
 SG |   8109 |         68 |         2 |    1 |             1
 SG |   8120 |         66 |         3 |    3 |             2
 SG |   8108 |         64 |         4 |    4 |             3
 SG |   8124 |         64 |         5 |    4 |             3
(12 rows)
```

8101 và 8109 cùng 68 chuyến: `row_number` cho 1 và 2 (khoá phụ `d.code` phân xử), `rank` và `dense_rank` cùng cho hạng 1. Dòng kế tiếp (8120) có `rank` **3** — nhảy cóc — còn `dense_rank` **2** — liên tục. Vì vậy «top 3» Đà Nẵng có bốn người (8302, 8310 cùng 40 chuyến), Sài Gòn có năm. Đây là câu hỏi **nghiệp vụ**: «ba người» (`row_number`, cần quy tắc phân xử hoà thành văn bản), «ba hạng có người» (`dense_rank`) hay «không quá hai người hơn mình» (`rank`) — ghi quy tắc cạnh báo cáo. `row_number` **cần** khoá phụ; `rank`/`dense_rank` thì **không được** có, thêm `d.code` là hết hoà.

Mệnh đề `WINDOW w AS (…)` đặt tên cửa sổ để dùng lại. Gọi `OVER (w ROWS UNBOUNDED PRECEDING)` để **thêm** frame là hợp lệ; nhưng không được đè `ORDER BY` mà `w` đã có (`cannot override ORDER BY clause of window "w"`), cũng không chép một `w` đã có frame (`cannot copy window "w" because it has a frame clause`) — đều `42P20`.

## 7. So sánh với dòng trước: lag

**`lag(biểu_thức [, k [, mặc_định]])`** trả giá trị của dòng đứng trước k dòng (mặc định 1) trong partition; không có thì trả `mặc_định` (mặc định NULL). `lead` là chiều ngược lại. Khách 0910000016 — người đi nhiều nhất, 36 chuyến:

```sql
SELECT t.code, t.requested_at, t.kind,
       lag(t.kind) OVER w                          AS dich_vu_truoc,
       t.requested_at - lag(t.requested_at) OVER w AS cach_chuyen_truoc,
       row_number() OVER w                         AS chuyen_thu
FROM trips AS t JOIN customers AS c ON c.id = t.customer_id
WHERE c.phone = '0910000016'
WINDOW w AS (ORDER BY t.requested_at, t.code)
ORDER BY t.requested_at, t.code
LIMIT 6;
```

```text
  code  |      requested_at      | kind | dich_vu_truoc | cach_chuyen_truoc | chuyen_thu
--------+------------------------+------+---------------+-------------------+------------
 100008 | 2026-10-01 06:12:00+07 | ride |               |                   |          1
 100149 | 2026-10-02 12:45:00+07 | ride | ride          | 1 day 06:33:00    |          2
 100215 | 2026-10-03 07:04:00+07 | ride | ride          | 18:19:00          |          3
 100259 | 2026-10-03 13:02:00+07 | ride | ride          | 05:58:00          |          4
 100277 | 2026-10-03 17:01:00+07 | ride | ride          | 03:59:00          |          5
 100369 | 2026-10-04 12:07:00+07 | ride | ride          | 19:06:00          |          6
(6 rows)
```

Chuyến đầu không có dòng trước nên `lag` là NULL — «không áp dụng» theo nghĩa của PG-011, không phải khoảng cách 0. Hiệu hai `timestamptz` là một `interval` (PG-016). Báo cáo cho mọi khách thì thêm `PARTITION BY t.customer_id`. `LIMIT 6` an toàn vì `LIMIT` cắt **sau** khi cửa sổ đã tính trên cả 36 chuyến — `WHERE` thì không (mục 3).

### «So với hôm qua» trên dữ liệu thưa

`lag` dính cùng bẫy với `ROWS`: «dòng trước» không phải «ngày trước». Thí nghiệm C: trên dữ liệu thưa, dòng trước của 21/10 là 19/10 (637.000 đ), nên báo cáo ghi 872.000 đ là tăng hơn một phần ba, trong khi hôm qua thật bằng 0; còn 20/10 không có dòng nào để so. Trên lịch đủ, «hôm qua» của 21/10 là 0 và phần trăm thay đổi thành chia cho 0 — `NULLIF(hom_qua, 0)` (PG-011) cho NULL thay vì lỗi `22012`; ngày 20/10 ra −100,0 %, 22/10 ra −10,0 %.

### Bẫy last_value

`first_value`/`last_value` tính trên **frame**. Với frame mặc định («tới dòng hiện tại»), `last_value(t.code) OVER (ORDER BY t.requested_at, t.code)` chính là dòng hiện tại: 100008 → 100008, 100149 → 100149… Muốn chuyến cuối tháng (103123) phải mở frame:

```sql
last_value(t.code) OVER (ORDER BY t.requested_at, t.code
                         ROWS BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING) AS chuyen_cuoi
```

`first_value` đúng «tình cờ» vì đầu frame mặc định là đầu partition.

## 8. Lỗi thường gặp

| Triệu chứng | Nguyên nhân | Cách sửa |
|---|---|---|
| Luỹ kế nhảy cóc, nhiều dòng cùng giá trị | Frame mặc định `RANGE` gộp dòng ngang hàng | `ROWS …` + khoá phụ duy nhất |
| Luỹ kế «bắt đầu lại» giữa kỳ | `WHERE` lọc trước cửa sổ | Tính ở truy vấn trong, lọc ở ngoài |
| Trung bình 7 ngày lệch quanh ngày trống | `ROWS`/`lag` trên dữ liệu thiếu ngày | Lịch đủ + `LEFT JOIN` |
| `avg` trên `RANGE` vẫn lệch | `avg` chia số dòng trong khung | Chia số ngày lịch |
| `42P20` window functions not allowed in WHERE | Lọc theo hạng cùng cấp | Truy vấn con / CTE |
| `42803` aggregate … cannot contain window function | `max(lag(…) OVER …)` | Cửa sổ ở truy vấn con, gộp ở ngoài |
| «Top 3» trả 4–5 dòng | `rank`/`dense_rank` có hoà | Chọn hàm theo quy tắc nghiệp vụ |
| `last_value` = dòng hiện tại | Frame mặc định dừng ở dòng hiện tại | Mở frame tới `UNBOUNDED FOLLOWING` |

## 9. Đánh đổi

- **`ROWS` hay `RANGE`:** `ROWS` xác định từng dòng (khi thứ tự xác định), hợp cho luỹ kế; `RANGE` diễn đạt «trong khoảng giá trị» nhưng chỉ một cột sắp xếp và gộp dòng ngang hàng. Trong code báo cáo, luôn ghi frame ra, kể cả khi trùng mặc định.
- **Lịch đủ hay `RANGE` interval:** lịch đủ cho mỗi ngày một dòng (biểu đồ không hở, `lag` đúng «hôm qua») nhưng thêm một `LEFT JOIN`; `RANGE` không cần lịch nhưng mọi phép chia phải tự đếm ngày.
- **Nhiều cửa sổ trong một câu:** tài liệu `SELECT` bảo đảm các hàm có `PARTITION BY`/`ORDER BY` tương đương được tính trong một lượt; đặt tên bằng `WINDOW` vừa gọn vừa nói rõ ý định. Chi phí sắp xếp đo ở Phần 3 (`EXPLAIN`), không kết luận ở đây.

## 10. Bài tập

**Bài 1.** Khách 0910000016: tìm **ba khoảng nghỉ dài nhất** giữa hai chuyến liên tiếp, kèm mã chuyến trước. Thử `max(… lag(…) OVER …)` trước để xem lỗi.

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">SELECT code, chuyen_truoc, khoang_nghi
FROM (
  SELECT t.code,
         lag(t.code) OVER w                          AS chuyen_truoc,
         t.requested_at - lag(t.requested_at) OVER w AS khoang_nghi
  FROM trips AS t JOIN customers AS c ON c.id = t.customer_id
  WHERE c.phone = '0910000016'
  WINDOW w AS (ORDER BY t.requested_at, t.code)
) AS x
ORDER BY khoang_nghi DESC NULLS LAST, code
LIMIT 3;</code></pre>
<p>Kết quả: 102664 sau 102372 (<code>2 days 09:57:00</code>), 101792 sau 101634 (<code>1 day 07:15:00</code>), 103123 sau 102984 (<code>1 day 06:40:00</code>). Viết thẳng <code>max(…)</code> gặp <code>aggregate function calls cannot contain window function calls</code> (<code>42803</code>). <code>NULLS LAST</code> đẩy chuyến đầu (khoảng nghỉ NULL) xuống cuối khi sắp <code>DESC</code>.</p>
</details>

**Bài 2.** Mỗi thành phố, ngày nào doanh thu **luỹ kế** tháng 10 lần đầu đạt **một nửa** doanh thu cả tháng? Dùng hai cửa sổ trong một truy vấn.

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">SELECT tp, min(ngay) AS ngay_vuot_nua, max(tong_thang) AS tong_thang
FROM (
  SELECT c.code AS tp, s.ngay,
         sum(s.doanh_thu) OVER (PARTITION BY c.code ORDER BY s.ngay ROWS UNBOUNDED PRECEDING) AS luy_ke,
         sum(s.doanh_thu) OVER (PARTITION BY c.code)                                         AS tong_thang
  FROM (SELECT city_id, (requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::date AS ngay, sum(fare) AS doanh_thu
        FROM trips WHERE status = 'completed' GROUP BY 1, 2) AS s
  JOIN cities AS c ON c.id = s.city_id
) AS x
WHERE luy_ke * 2 &gt;= tong_thang
GROUP BY tp
ORDER BY tp COLLATE "C";</code></pre>
<p>Kết quả: DN 14/10 (26.416.000 đ), HN 15/10 (62.991.000 đ), SG 16/10 (92.880.000 đ). Cửa sổ thứ hai không có <code>ORDER BY</code> nên frame là cả partition. So <code>luy_ke * 2 &gt;= tong_thang</code> để khỏi lo chia nguyên.</p>
</details>

**Bài 3.** Tài xế 8101: ở mỗi chuyến hoàn thành, tính tổng cước của **3 chuyến hoàn thành ngay trước** (không gồm chuyến hiện tại), theo thời điểm hoàn thành. In 5 chuyến đầu. Chuyến đầu tiên hiện gì?

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">SELECT t.code, t.fare,
       sum(t.fare) OVER w AS ba_chuyen_truoc,
       count(*)    OVER w AS so_chuyen_trong_khung
FROM trips AS t JOIN drivers AS d ON d.id = t.driver_id
WHERE d.code = 8101 AND t.status = 'completed'
WINDOW w AS (ORDER BY t.completed_at, t.code ROWS BETWEEN 3 PRECEDING AND 1 PRECEDING)
ORDER BY t.completed_at, t.code
LIMIT 5;</code></pre>
<p>Kết quả: 100275 → NULL (0 chuyến); 100307 → 81.000 (1); 100467 → 178.000 (2); 100486 → 295.000 (3); 100501 → 251.000 (3, khung đã trượt bỏ 100275). Khung rỗng thì <code>sum</code> là NULL, <code>count</code> là 0 — đúng nghĩa «chưa có chuyến trước». Cách khác: <code>ROWS BETWEEN 3 PRECEDING AND CURRENT ROW EXCLUDE CURRENT ROW</code>; lab so hai cách trên cả 68 chuyến của 8101: 0 dòng khác.</p>
</details>

## 11. Đọc tiếp

- [PostgreSQL 17 · Tutorial: Window Functions](https://www.postgresql.org/docs/17/tutorial-window.html).
- [PostgreSQL 17 · Window Function Calls](https://www.postgresql.org/docs/17/sql-expressions.html#SYNTAX-WINDOW-FUNCTIONS) — frame, frame mặc định, `EXCLUDE`.
- [PostgreSQL 17 · Window Functions](https://www.postgresql.org/docs/17/functions-window.html).
- [PostgreSQL 11 · Release Notes](https://www.postgresql.org/docs/11/release-11.html) — `RANGE` có khoảng cách, `GROUPS`, loại trừ frame.
- Góc nhìn phân tích: [Window function: xếp hạng, luỹ kế và so sánh kỳ trước](/articles/da-window-function-xep-hang-luy-ke) (series DA).
- Lab: `p02/15-window-function.sql`, `p02/thu-nghiem/pg015-khung-cua-so.sql`, `p02/thu-nghiem/pg015-bai-tap.sql`; kết quả đầy đủ ở `p02/OUTPUT.txt`.

Bài tiếp theo: [PG-016 · Ngày giờ và múi giờ trong truy vấn](/articles/postgresql-date-time-timezone).
