---
slug: postgresql-statistics-analyze-cardinality
title: "Statistics, ANALYZE và ước lượng cardinality: khi planner đoán sai số dòng"
summary: "Tới Nơi nạp 30.000 chuyến khuyến mãi ngày 01/11, planner vẫn tin ngày đó có 1 chuyến và chọn Nested Loop. Đọc pg_stats, MCV, histogram, thống kê mở rộng để tìm và sửa chỗ ước lượng lệch so với thực tế trên dữ liệu phân bố không đều."
category: postgresql
level: intermediate
tags: [postgresql, pg-027, statistics, analyze, cardinality, pg-stats, explain, toi-noi]
language: vi
---

Sáng 01/11, đội tăng trưởng của Tới Nơi chạy khuyến mãi «đồng giá cuối tuần». Lúc 10:00 đã có 30.000 chuyến của ngày mới. Báo cáo «chuyến hôm nay theo loại xe» vốn chạy tức thì bỗng chậm hẳn. Kế hoạch thực thi cho thấy planner tin rằng ngày 01/11 có **1 chuyến**, nên chọn Nested Loop: mỗi chuyến tra index tài xế một lần, tổng cộng 30.000 vòng lặp. Không ai sửa SQL, không ai xoá index. Thứ đã cũ là **thống kê**: lần ANALYZE gần nhất chạy khi bảng chưa có chuyến nào của 01/11.

Ở [PG-021](/articles/postgresql-explain-ke-hoach-thuc-thi), cùng một câu SQL ra Index Scan + Limit với khách B1 và Bitmap Heap Scan + Sort với khách `0910000500`. Cả hai lựa chọn xuất phát từ **số dòng ước lượng**. Bài này mở hộp chứa con số đó: planner lấy thống kê ở đâu, tính ra sao, sai ở đâu trên dữ liệu lệch như của Tới Nơi, và sửa bằng gì.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-027 · Phần 3/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/postgresql-explain-ke-hoach-thuc-thi">PG-021 · Đọc EXPLAIN từ truy vấn tìm chuyến</a>. Lab: <code>examples/toi-noi-postgresql/p03</code> (tệp <code>27-statistics.sql</code>) và thí nghiệm <code>p03/thu-nghiem/pg027-cong-thuc-uoc-luong.sql</code>. Sau bài này, bạn tìm được chỗ ước lượng lệch so với thực tế trên dữ liệu phân bố không đều, đọc được <code>pg_stats</code> và chọn cách sửa phù hợp: tăng mức thống kê, thống kê mở rộng hay chạy lại ANALYZE.</p>
</div>

## 1. Dữ liệu và cách chạy

Phần 3 chạy trên **PostgreSQL 18.1 thật** (Phần 1–2 dùng PGlite 17.5); runner tự dựng cluster tạm và chạy tệp bài trên bản sao mới của nền:

```bash
cd examples/toi-noi-postgresql
node pg/run-pg.mjs p03 --only 27                                                   # bài lab
node pg/run-pg.mjs --nen p03/lab.json:p03 p03/thu-nghiem/pg027-cong-thuc-uoc-luong.sql   # thí nghiệm
```

Bảng `trips` có **1.000.000 chuyến** (19.441 page), phân bố cố ý lệch: khách doanh nghiệp B1 (`0919990001`) có **51.151** chuyến (5,1 %), 5 khách cá nhân đi trên 1.000 chuyến, 8.151 khách chưa đi chuyến nào; **191.852** khách khác nhau có chuyến. Khu đón (`pickup_area_id`) phụ thuộc hoàn toàn vào thành phố (`city_id`).

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Số ước lượng không cố định</span>
<p>ANALYZE đọc một <b>mẫu ngẫu nhiên</b>, nên <code>n_distinct</code>, tần suất MCV và mọi <code>rows=</code> ước lượng đổi chút ít sau mỗi lần ANALYZE dù dữ liệu không đổi. Số ước lượng dưới đây lấy từ <b>một lần chạy</b>, có dao động; hãy so <b>hệ số lệch</b> và chiều của nó. Số thực tế (<code>actual rows</code>, <code>count(*)</code>) thì tất định.</p>
</div>

## 2. Planner lấy số liệu ở đâu?

Planner *ước lượng* **cardinality** (bản số, ở đây là số dòng) mà mỗi nút kế hoạch trả ra, dựa trên hai nguồn:

- `pg_class.reltuples` và `relpages`: số dòng và số page của bảng, cập nhật khi VACUUM, ANALYZE hay CREATE INDEX chạy, không cập nhật theo từng câu ghi.
- `pg_statistic`, đọc qua view `pg_stats`: thống kê từng cột do **ANALYZE** thu thập.

Với bảng lớn, ANALYZE không đọc hết bảng mà lấy mẫu **300 × statistics target** dòng. Target mặc định là `default_statistics_target = 100`, tức 30.000 dòng, và cũng là số phần tử tối đa của danh sách giá trị phổ biến và của histogram. Các cột đáng đọc trong `pg_stats`:

| Cột | Nghĩa |
|---|---|
| `null_frac` | tỉ lệ NULL |
| `n_distinct` | số giá trị khác nhau; **số âm** = tỉ lệ so với số dòng (−1 nghĩa là mọi giá trị đều khác nhau) |
| `most_common_vals`, `most_common_freqs` | **MCV** (most common values, giá trị phổ biến nhất) và tần suất từng giá trị |
| `histogram_bounds` | các mốc chia phần giá trị *ngoài MCV* thành những «xô» có số dòng gần bằng nhau |
| `correlation` | độ khớp giữa thứ tự vật lý trong heap và thứ tự giá trị (PG-023, PG-024) |

Cột `customer_id` của `trips` sau ANALYZE của nền (lab, khối biến động):

```text
 null_frac | n_distinct | so_mcv | tan_suat_mcv_1 | tong_tan_suat_mcv | so_moc_histogram
-----------+------------+--------+----------------+-------------------+------------------
         0 |      82202 |     51 |         0.0500 |             0.081 |              101
```

Hai lần dựng nền khác (khi chạy thí nghiệm của bài) cho `n_distinct` **82.254** với **60** MCV (tổng tần suất 0,0872) và **81.567** với **50** MCV (0,0850). Mẫu khác nhau, bộ số khác nhau. Điều không đổi: `n_distinct` chỉ bằng khoảng **0,43 lần** số khách thật (191.852), và B1 luôn đứng đầu MCV với tần suất ≈ 0,05.

Vì sao thấp? Mẫu 30.000 dòng chỉ chạm một phần nhỏ trong gần 192 nghìn khách; ANALYZE phải *suy* số giá trị chưa gặp, và với phân bố «số ít khách đi rất nhiều» của Tới Nơi phép suy cho kết quả thấp. Tài liệu ANALYZE nói rõ ước lượng này có thể khá sai, kể cả với target lớn nhất.

## 3. Giá trị trong MCV và ngoài MCV

### 3.1. Khách B1: trong MCV

Với `customer_id = B1`, planner lấy thẳng tần suất của B1 trong MCV nhân số dòng:

```text
 Bitmap Heap Scan on trips  (cost=1384.18..21450.59 rows=50033 width=129) (actual rows=51151.00 loops=1)
   Recheck Cond: (customer_id = '239ec4d4-ca1d-4ae9-8459-a2f5227a39c1'::uuid)
   Heap Blocks: exact=18139
   ->  Bitmap Index Scan on ix_trips_customer_requested  (cost=0.00..1371.67 rows=50033 width=0) (actual rows=51151.00 loops=1)
```

Ước khoảng 50.000, thực tế 51.151: lệch 1,0 lần. (PostgreSQL 18 in `actual rows` có hai chữ số thập phân.)

### 3.2. Mọi khách ngoài MCV nhận cùng một ước lượng

Với giá trị *không* có trong MCV, tài liệu PostgreSQL (mục Row Estimation Examples) cho công thức: phần dòng không thuộc MCV được chia **đều** cho các giá trị còn lại.

```text
selectivity = (1 − null_frac − Σ most_common_freqs) / (n_distinct − số MCV)
rows        = selectivity × số dòng của bảng
```

Thí nghiệm `pg027-cong-thuc-uoc-luong.sql` tự tính công thức này từ `pg_stats` rồi đặt cạnh số của planner:

```text
 uoc_tu_tinh | planner_khach_0_chuyen | planner_khach_76_chuyen
-------------+------------------------+-------------------------
        11.1 |                     11 |                      11
```

(1 − 0 − 0,0872) / (82.254 − 60) × 1.000.000 ≈ 11,1. Khách `0910000066` đi **76** chuyến và khách chưa đi chuyến nào đều được ước **11** dòng. Lab ghi hệ số lệch của khách 76 chuyến khoảng **6,9 lần** ở một lần chạy (kiểm trong khoảng 3–15 lần). Với khách 0 chuyến, planner vẫn ước như khách «trung bình», vì không có thống kê nào nói giá trị đó vắng mặt.

Đây là chỗ lệch điển hình trên dữ liệu không đều: giá trị **nặng nhưng chưa đủ nặng để vào MCV** bị ước thấp. Ở PG-021, khách 40 chuyến được ước khoảng 11 nên planner chọn Bitmap + Sort; ở PG-030 cùng loại ước lượng dẫn tới Nested Loop.

## 4. Sửa ước lượng của một cột

### 4.1. Tăng statistics target riêng cho cột

```sql
ALTER TABLE trips ALTER COLUMN customer_id SET STATISTICS 1000;
ANALYZE trips (customer_id);
```

Target 1000 nghĩa là mẫu 300.000 dòng và MCV tối đa 1.000 giá trị. Kết quả một lần chạy:

```text
 n_distinct | so_mcv
------------+--------
  -0.157148 |   1000
```

`n_distinct` chuyển sang dạng âm: ≈ 0,157 × 1.000.000 = 157 nghìn khách, sát hơn nhiều so với 82 nghìn. Khách 76 chuyến lọt vào MCV, nên hệ số lệch về khoảng **1,0**. Cái giá: ANALYZE đọc nhiều dòng hơn, `pg_statistic` to hơn, planner tốn thêm chút thời gian với danh sách dài hơn.

### 4.2. Ghi đè n_distinct: sửa khách trung bình, không sửa khách nặng

Cách thứ hai là khai số giá trị khác nhau cho cột: `ALTER TABLE … ALTER COLUMN … SET (n_distinct = …)`. Giá trị dương là số tuyệt đối, giá trị trong khoảng −1 đến 0 là tỉ lệ theo số dòng. Lệnh chỉ có hiệu lực từ lần ANALYZE kế tiếp. Thí nghiệm ghi đè bằng số đếm thật:

```sql
ALTER TABLE trips ALTER COLUMN customer_id SET (n_distinct = 191852);
ANALYZE trips (customer_id);
```

| Khách | Trước | Sau khi ghi đè n_distinct |
|---|---|---|
| 0 chuyến (thực tế 0) | ước ≈ 11 | ước ≈ **5** |
| 76 chuyến | lệch ≈ 7 lần | lệch ≈ **15 lần** (một lần chạy: 15,2) |

Ước lượng cho khách «trung bình» sát hơn (thực tế trung bình ≈ 5,2 chuyến mỗi khách có chuyến), nhưng khách nặng ngoài MCV còn bị ước thấp **hơn** trước. Mẫu số của công thức tăng, tử số giữ nguyên. Ghi đè `n_distinct` chỉ giải quyết đúng vấn đề «đếm sai số giá trị khác nhau», không giải quyết «phân bố lệch».

## 5. Hai cột tương quan: thống kê mở rộng

Planner mặc định coi các điều kiện trong `WHERE` là **độc lập** và nhân xác suất của chúng. Với Tới Nơi điều đó sai: biết khu đón 311 là biết thành phố Đà Nẵng.

```text
 Bitmap Heap Scan on trips  (cost=2972.62..24356.62 rows=2798 width=129) (actual rows=22222.00 loops=1)
   Recheck Cond: (city_id = 3)
   Filter: (pickup_area_id = 311)
   Rows Removed by Filter: 107573
   …
```

Ước 2.798, thực tế 22.222: thấp khoảng **7,9 lần** ở lần chạy này. Planner nhân «13 % chuyến ở Đà Nẵng» với «2,2 % chuyến ở khu 311», trong khi điều kiện thứ hai đã bao hàm điều kiện thứ nhất. GROUP BY hai cột lệch theo cách tương tự: planner nhân số giá trị khác nhau (3 × 73 = 219 nhóm), còn thực tế chỉ có **75** nhóm (72 khu + 3 nhóm mất GPS), lệch ≈ 2,9 lần.

**Thống kê mở rộng** (`CREATE STATISTICS`) báo cho ANALYZE thu thập thống kê trên *tổ hợp* cột:

```sql
CREATE STATISTICS st_trips_tp_khu (dependencies, ndistinct, mcv) ON city_id, pickup_area_id FROM trips;
ANALYZE trips;
SELECT statistics_name, n_distinct, dependencies FROM pg_stats_ext WHERE statistics_name = 'st_trips_tp_khu';
```

```text
 statistics_name |  n_distinct   |     dependencies
-----------------+---------------+-----------------------
 st_trips_tp_khu | {"4, 14": 75} | {"14 => 4": 0.989670}
```

Cột được gọi bằng số thứ tự (attnum): 4 = `city_id`, 14 = `pickup_area_id`. `ndistinct` ghi 75 tổ hợp; `dependencies` ghi «khu ⇒ thành phố» với độ phụ thuộc ≈ 0,99. Con số không đạt 1,0 vì khoảng 1 % chuyến mất GPS có khu NULL ở cả ba thành phố. Chiều «thành phố ⇒ khu» yếu tới mức không được ghi. Sau đó ước lượng của câu lọc là **21.887** (lệch 1,0), GROUP BY ước sát 75 nhóm.

| Loại | Thống kê gì | Giúp loại câu nào |
|---|---|---|
| `dependencies` | phụ thuộc hàm giữa các cột | `col = hằng` hoặc `col IN (…)` trên nhiều cột |
| `ndistinct` | số tổ hợp giá trị khác nhau | GROUP BY, DISTINCT trên nhiều cột |
| `mcv` | danh sách tổ hợp phổ biến và tần suất | điều kiện kết hợp, kể cả loại `dependencies` không hỗ trợ |

Giới hạn theo tài liệu: phụ thuộc hàm không áp dụng cho so sánh khoảng, `LIKE` hay so sánh hai cột; thống kê mở rộng **không** dùng cho điều kiện JOIN. Chỉ tạo trên tổ hợp cột thật sự đi cùng nhau trong truy vấn.

## 6. Điều kiện trên biểu thức: không có thống kê, planner đoán

Báo cáo «chuyến đặt trong khung 12 giờ» viết `extract(hour FROM requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh') = 12`. ANALYZE thu thập thống kê theo *cột*, không theo biểu thức, nên planner không có gì để tra (thí nghiệm, phần 2):

| Điều kiện | Ước lượng | Thực tế |
|---|---:|---:|
| giờ = 12 | 5.000 | 89.965 |
| giờ = 3 | 5.000 | (thí nghiệm không đếm) |

5.000 = 0,5 % số dòng, độ chọn lọc mặc định của phép `=` mà planner dùng khi không biết gì. Giờ cao điểm 12 giờ hay 3 giờ sáng đều nhận cùng con số, nên ước lượng thấp khoảng 18 lần ở khung trưa. Có hai cách cho planner thống kê trên biểu thức:

- Index trên biểu thức (PG-025): ANALYZE thu thập thống kê cho biểu thức, đổi lại mọi câu ghi phải cập nhật thêm index.
- **Thống kê trên biểu thức**, có từ PostgreSQL 14 (release notes 14: «Allow extended statistics on expressions»), không có chi phí ghi:

```sql
CREATE STATISTICS st_trips_gio_dat ON (extract(hour FROM requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh')) FROM trips;
ANALYZE trips;
```

Sau ANALYZE, hệ số lệch về khoảng 1,0, và `pg_stats_ext_exprs` ghi `n_distinct = 24` (24 giờ trong ngày).

## 7. Histogram và thống kê cũ

Với cột liên tục như `requested_at`, phần không thuộc MCV được mô tả bằng histogram: 101 mốc chia một triệu chuyến thành 100 xô gần bằng nhau. Điều kiện khoảng được ước bằng cách đếm số xô bị phủ và nội suy tuyến tính trong xô dở dang. Khoảng 7 ngày (01–08/08) ước lệch khoảng 1,0 lần.

Histogram chỉ đúng tới **lần ANALYZE gần nhất**. Lab nạp 30.000 chuyến ngày 01/11 rồi chưa ANALYZE:

```text
 thuc_te                     -- uoc_luong(… requested_at ngày 01/11 …)
---------
   30000
  lech
---------
 30000.0
```

Mốc cuối của histogram vẫn ở 31/10, nên planner tin ngày 01/11 gần như trống: ước **1**, lệch 30.000 lần. Hệ quả đối với câu báo cáo:

```text
 GroupAggregate
   …
         ->  Nested Loop
               ->  Index Scan using ix_trips_city_requested on trips t
                     Index Cond: ((requested_at >= '2026-11-01 00:00:00+07'::timestamp with time zone) AND …)
               ->  Index Scan using pk_drivers on drivers d
```

Index Scan trên `ix_trips_city_requested (city_id, requested_at)` với điều kiện chỉ ở cột thứ hai là **skip scan** của PostgreSQL 18 (PG-025). Sau `ANALYZE trips`, ước lượng lệch khoảng 1,3 lần, kế hoạch đổi thành **Hash Join** + HashAggregate: đọc 6.000 tài xế một lần vào bảng băm thay vì tra index 30.000 lần.

Có một điều thú vị: planner **không** mù hoàn toàn với dòng mới. Thí nghiệm (phần 4) cho thấy `reltuples` vẫn là 1.000.000 và `relpages` vẫn là 19.441, nhưng bảng thật đã có 20.023 page. Planner nhân mật độ cũ (dòng/page) với số page hiện tại, nên `SELECT id FROM trips` vẫn được ước sát 1.030.000 dòng (lệch 1,0). Thứ cũ là **phân bố**: planner biết bảng to thêm, nhưng không biết phần to thêm nằm ở giá trị nào.

### 7.1. Autovacuum có tự cứu không?

Cluster lab tắt autovacuum để tái lập. Trên server thật, autovacuum chạy ANALYZE cho một bảng khi số dòng thêm/sửa/xoá kể từ lần ANALYZE trước vượt ngưỡng:

```text
ngưỡng = autovacuum_analyze_threshold + autovacuum_analyze_scale_factor × reltuples
       = 50 + 0,1 × 1.000.000 ≈ 100.050
```

```text
 base | scale | nguong_lam_tron | n_mod_since_analyze | se_tu_analyze
------+-------+-----------------+---------------------+---------------
 50   | 0.1   |          100000 |               30000 | f
```

30.000 dòng mới chưa tới ngưỡng: dù autovacuum bật, bảng vẫn chưa được ANALYZE lại (lab chỉ áp công thức vào số đếm thật, không chạy autovacuum). Với `trips` thật của Tới Nơi (730 triệu dòng sau một năm, theo hồ sơ dự án), 10 % là khoảng 73 triệu dòng thay đổi. Cách xử lý: hạ ngưỡng riêng cho bảng (`ALTER TABLE trips SET (autovacuum_analyze_scale_factor = 0.01)`), hoặc `ANALYZE trips` ngay sau mỗi lô nạp lớn.

## 8. Quy trình tìm chỗ ước lượng lệch

1. `EXPLAIN (ANALYZE)` (câu được thực thi thật, PG-021): đặt `rows=` cạnh `actual rows × loops` ở **từng nút**, từ nút lá lên.
2. Tìm nút **thấp nhất** có độ lệch từ khoảng 10 lần; nút cha thường chỉ kế thừa sai số của con.
3. Kiểm tra xem thống kê còn mới không: `last_analyze`, `last_autoanalyze`, `n_mod_since_analyze` trong `pg_stat_user_tables`.
4. Đọc `pg_stats` của cột: giá trị có trong MCV không, `n_distinct` so với `count(DISTINCT …)`.
5. Sửa theo nguyên nhân:

| Nguyên nhân | Dấu hiệu | Cách sửa |
|---|---|---|
| Thống kê cũ | `n_mod_since_analyze` lớn, giá trị nằm ngoài histogram | `ANALYZE`; hạ scale factor cho bảng lớn |
| Phân bố lệch, giá trị nặng ngoài MCV | mọi giá trị ngoài MCV nhận cùng một ước lượng | `SET STATISTICS` cao hơn cho đúng cột đó |
| Cột tương quan | lệch khi lọc/gom nhiều cột, từng cột riêng thì đúng | `CREATE STATISTICS (dependencies, ndistinct, mcv)` |
| Biểu thức | ước lượng tròn 0,5 % số dòng | `CREATE STATISTICS ON (biểu thức)` hoặc index biểu thức |

Hàm `uoc_luong(sql)` của nền lab làm sẵn bước 1 cho nút gốc.

## 9. Lỗi thường gặp

- **Trích con số ước lượng như hằng số.** `n_distinct = 82202` của máy này là 82.254 ở máy khác. Ghi hệ số lệch và nguồn gốc, đừng ghi chữ số cuối.
- **Tưởng «đã có autovacuum» là thống kê luôn mới.** Ngưỡng tỉ lệ theo kích thước bảng; bảng càng lớn càng lâu được ANALYZE lại. Lô nạp lớn nên kết thúc bằng `ANALYZE`.
- **Tăng `default_statistics_target` cho cả server** trong khi chỉ một cột có vấn đề. Cái giá (ANALYZE lâu hơn, `pg_statistic` to hơn) áp lên mọi cột của mọi bảng. Đặt target theo cột.
- **Tạo `CREATE STATISTICS` rồi quên ANALYZE**, hoặc quên rằng pg_upgrade lên bản 18 giữ thống kê thường nhưng **không** giữ thống kê mở rộng (release notes 18). Cả hai trường hợp: đối tượng thống kê rỗng tới lần ANALYZE kế tiếp.
- **Dùng thống kê mở rộng để chữa ước lượng JOIN.** Planner không dùng chúng cho điều kiện nối.

## 10. Đánh đổi

Thống kê chi tiết hơn tốn thời gian ANALYZE, dung lượng `pg_statistic` và chút thời gian lập kế hoạch. Thống kê lấy mẫu còn khiến hai lần ANALYZE trên cùng dữ liệu có thể cho hai kế hoạch khác nhau khi chi phí hai phương án sát nhau; lab gặp thật ở PG-030 (kế hoạch chung lúc Index Scan + Limit, lúc Bitmap + Sort). Target cao giảm dao động nhưng không xoá hẳn. Truy vấn quan trọng mà chỉ có kế hoạch tốt nhờ «may mắn của mẫu» thì nên sửa cấu trúc (index, viết lại câu).

## 11. Bài tập

**Bài 1.** Từ `pg_stats` và `pg_class`, tự tính số dòng planner sẽ ước cho `customer_id = <một khách không có trong MCV>`, rồi đối chiếu với `EXPLAIN`.

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">SELECT round(((1 - s.null_frac - (SELECT sum(f) FROM unnest(s.most_common_freqs) AS f))
             / ((CASE WHEN s.n_distinct &lt; 0 THEN -s.n_distinct * c.reltuples ELSE s.n_distinct END)
                - cardinality(s.most_common_vals::text::text[]))
             * c.reltuples)::numeric, 1) AS uoc_tu_tinh
FROM pg_stats AS s JOIN pg_class AS c ON c.oid = 'trips'::regclass
WHERE s.schemaname = 'toinoi' AND s.tablename = 'trips' AND s.attname = 'customer_id';</code></pre>
<p>Một lần chạy: 11,1 so với <code>rows=11</code> của planner (thí nghiệm kiểm tỉ số trong khoảng 0,9–1,1). <code>n_distinct</code> âm phải đổi về số tuyệt đối trước khi trừ số MCV. Kết quả không phụ thuộc khách nào, miễn khách đó ngoài MCV: khách 0 chuyến và khách 76 chuyến nhận cùng ước lượng.</p>
</details>

**Bài 2.** Câu `SELECT * FROM trips WHERE extract(hour FROM requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh') = 12` được ước bao nhiêu dòng? Sửa ước lượng mà không tạo index.

<details><summary><b>Lời giải</b></summary>
<p>Ước 5.000 dòng (0,5 % của 1.000.000), thực tế 89.965: thấp khoảng 18 lần. Giờ 3 sáng cũng được ước 5.000. Tạo thống kê trên biểu thức rồi ANALYZE:</p>
<pre><code class="language-sql">CREATE STATISTICS st_trips_gio_dat
  ON (extract(hour FROM requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh')) FROM trips;
ANALYZE trips;
SELECT n_distinct FROM pg_stats_ext_exprs WHERE statistics_name = 'st_trips_gio_dat';  -- 24</code></pre>
<p>Hệ số lệch về khoảng 1,0. Biểu thức trong truy vấn phải viết giống biểu thức trong thống kê. Lời giải đầy đủ: <code>p03/thu-nghiem/pg027-cong-thuc-uoc-luong.sql</code>, phần 2.</p>
</details>

**Bài 3.** Đồng nghiệp đề xuất `ALTER TABLE trips ALTER COLUMN customer_id SET (n_distinct = 191852)` thay cho `SET STATISTICS 1000`, vì «191.852 là số thật». Dự đoán ước lượng cho khách 0 chuyến và khách 76 chuyến sau ANALYZE. Nên chọn cách nào?

<details><summary><b>Lời giải</b></summary>
<p>Khách 0 chuyến: ước khoảng 5 (trước là 11). Khách 76 chuyến: vẫn ngoài MCV, ước khoảng 5, nên lệch khoảng 15 lần (trước khoảng 7). Một lần chạy của thí nghiệm, phần 3: ước 5 và lệch 15,2. Ghi đè <code>n_distinct</code> chỉ sửa mẫu số của công thức. Vấn đề của Tới Nơi là khách nặng không có trong MCV, nên <code>SET STATISTICS 1000</code> (MCV tới 1.000 giá trị) mới đưa khách 76 chuyến về lệch khoảng 1,0. Nếu dùng ghi đè, dạng tỉ lệ (số âm) bám theo bảng khi bảng lớn dần tốt hơn số tuyệt đối.</p>
</details>

## 12. Đọc tiếp

- [PostgreSQL 18 · Statistics Used by the Planner](https://www.postgresql.org/docs/18/planner-stats.html): `reltuples`, target, thống kê mở rộng và giới hạn của chúng.
- [PostgreSQL 18 · Row Estimation Examples](https://www.postgresql.org/docs/18/row-estimation-examples.html): công thức MCV, ngoài MCV, histogram từng bước.
- [PostgreSQL 18 · pg_stats](https://www.postgresql.org/docs/18/view-pg-stats.html), [CREATE STATISTICS](https://www.postgresql.org/docs/18/sql-createstatistics.html), [ANALYZE](https://www.postgresql.org/docs/18/sql-analyze.html), [ALTER TABLE](https://www.postgresql.org/docs/18/sql-altertable.html) (`SET STATISTICS`, `n_distinct`).
- [Routine Vacuuming](https://www.postgresql.org/docs/18/routine-vacuuming.html) (ngưỡng autovacuum ANALYZE); release notes [14](https://www.postgresql.org/docs/release/14.0/) và [18](https://www.postgresql.org/docs/release/18.0/).
- Lab: `examples/toi-noi-postgresql/p03/27-statistics.sql`, kết quả ở `p03/OUTPUT.txt`; thí nghiệm `p03/thu-nghiem/pg027-cong-thuc-uoc-luong.sql`.

Bài tiếp theo: [PG-028 · Khi nào cân nhắc GIN, GiST và BRIN?](/articles/postgresql-gin-gist-brin-lua-chon)
