---
slug: postgresql-kieu-du-lieu-tien-thoi-gian-uuid
title: "Chọn kiểu dữ liệu cho tiền, thời gian và định danh"
summary: "Kế toán Tới Nơi hỏi tổng cước ngày 02/11: cước lưu bằng float sẽ lệch, ngày lấy theo UTC sẽ gom nhầm chuyến 00:10 sáng 03/11. Chọn kiểu cho tiền, thời điểm, định danh và lập data dictionary."
category: postgresql
level: intermediate
tags: [postgresql, pg-004, data-type, timestamptz, uuid, money, toi-noi]
language: vi
---

Kế toán Tới Nơi hỏi một câu tưởng đơn giản: «Tổng cước các chuyến đặt trong ngày 02/11 là bao nhiêu?». Có hai cách trả lời sai mà không ai nhận ra. Nếu cước được lưu bằng số thực theo đơn vị nghìn đồng, phép cộng sinh ra những sai lệch cực nhỏ — đủ để đối soát với cổng thanh toán không bao giờ khớp tuyệt đối. Nếu «ngày» được tính theo giờ UTC, chuyến 100008 khách đặt lúc 00:10 sáng 03/11 giờ Việt Nam sẽ bị gom vào ngày 02/11.

Cả hai đều **không phát sinh thông báo lỗi nào**. Chọn kiểu dữ liệu là quyết định database chặn sai sót nào và để lọt sai sót nào. Bài này chọn kiểu cho tiền, thời gian, định danh của bảng `trips`, rồi gom quyết định vào một data dictionary.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-004 · Phần 1/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/du-an-mau-toi-noi">Dự án mẫu Tới Nơi</a>, <a href="/articles/postgresql-bang-dong-cot-primary-foreign-key">PG-003: Bảng, dòng, cột và khoá</a>. Lab: <code>examples/toi-noi-postgresql/p01</code> (<code>01-schema.sql</code>, <code>03-fixture-invalid.sql</code>, <code>05-select.sql</code>). Sau bài này, bạn lập được data dictionary có đơn vị, ý nghĩa của NULL và múi giờ cho từng cột.</p>
</div>

## 1. Chọn kiểu là chọn miền giá trị, đơn vị và nghĩa của NULL

Trước khi gõ tên kiểu cho một cột, trả lời ba câu:

1. **Miền giá trị** (domain) là gì? Số nguyên hay có phần lẻ, lớn tới đâu, có âm không?
2. **Đơn vị** là gì? Đồng hay nghìn đồng, giờ Việt Nam hay UTC, giây hay phút?
3. **NULL nghĩa là gì?** «Chưa xảy ra», «không áp dụng» hay «không biết» — và có được phép không?

Kiểu dữ liệu là lớp kiểm tra đầu tiên: giá trị ngoài miền của kiểu bị từ chối ngay lúc đọc, trước mọi ràng buộc. Năm câu sai kiểu trong `03-fixture-invalid.sql` (kết quả thật):

| Giá trị gửi vào | Cột | Thông báo của PostgreSQL | SQLSTATE |
|---|---|---|---|
| `'45.000đ'` | `fare integer` | `invalid input syntax for type integer: "45.000đ"` | `22P02` |
| `3000000000` | `fare integer` | `integer out of range` | `22003` |
| `'khach-so-1'` | `customer_id uuid` | `invalid input syntax for type uuid: "khach-so-1"` | `22P02` |
| `'2026-02-30 09:00:00+07'` | `requested_at timestamptz` | `date/time field value out of range: …` | `22008` |
| `'hôm qua'` | `requested_at timestamptz` | `invalid input syntax for type timestamp with time zone: "hôm qua"` | `22007` |

Nếu các cột này là `text`, cả «45.000đ» lẫn «hôm qua» đều được nhận. Kiểu càng sát miền giá trị thật, database càng chặn được nhiều.

## 2. Tiền: số nguyên đồng

Tiền Việt Nam không có đơn vị lẻ dưới đồng trong nghiệp vụ của Tới Nơi, nên lab lưu cước là **số nguyên đồng**: `fare integer NOT NULL` — 45.000 đ lưu là `45000`. Các lựa chọn khác và lý do không dùng:

| Kiểu | Chính xác? | Vấn đề với cước Tới Nơi |
|---|---|---|
| `integer` (đồng) | có | tối đa 2.147.483.647 đ một giá trị — dư cho một chuyến |
| `bigint` (đồng) | có | 8 byte thay vì 4; cần cho **tổng**, không cần cho một chuyến |
| `numeric` | có | cần khi có phần lẻ thật (tỷ lệ phí, tiền ngoại tệ); tính toán chậm hơn số nguyên |
| `real` / `double precision` | **không** | sai số nhị phân, cộng dồn lệch |
| `money` | có (phần lẻ cố định) | định dạng phụ thuộc `lc_monetary` của server |

Tài liệu PostgreSQL nói thẳng: `real` và `double precision` là kiểu *«inexact, variable-precision»*, và khuyên dùng `numeric` khi cần lưu và tính chính xác, như số tiền. Thử với giả định một hệ cũ lưu cước theo **nghìn đồng** bằng số thực, tức 100 đ là `0.1` (truy vấn chạy trên lab, PGlite 17.5):

```sql
SELECT 0.1::double precision + 0.2::double precision AS float_cong,
       0.1::double precision + 0.2::double precision = 0.3 AS float_bang_0_3,
       0.1::numeric + 0.2::numeric = 0.3 AS numeric_bang_0_3;

SELECT sum(0.1::double precision) AS tong_float,
       sum(0.1::numeric)          AS tong_numeric,
       sum(100)                   AS tong_dong_integer
FROM generate_series(1, 10);
```

```text
     float_cong      | float_bang_0_3 | numeric_bang_0_3
---------------------+----------------+------------------
 0.30000000000000004 | f              | t
(1 row)

     tong_float     | tong_numeric | tong_dong_integer
--------------------+--------------+-------------------
 0.9999999999999999 |          1.0 |              1000
(1 row)
```

Mười khoản 100 đ cộng bằng `double precision` không ra đúng 1 nghìn đồng, và phép so sánh bằng trả `f`. Làm tròn khi hiển thị che được con số, nhưng không cứu được phép so sánh hay đối soát. `numeric` và số nguyên đồng đều chính xác; với VND, số nguyên đồng đơn giản và nhanh hơn — tài liệu ghi nhận phép tính `numeric` chậm hơn hẳn kiểu số nguyên.

**Tổng thì sao?** Với 2 triệu chuyến/ngày (hồ sơ Tới Nơi) và cước trung bình **giả định** 100.000 đ, tổng một ngày là 200 tỷ đồng — vượt xa 2,1 tỷ. PostgreSQL lo việc này: `sum` của `integer` trả về `bigint`. Trên fixture 15 chuyến:

```text
SELECT pg_typeof(sum(fare)) AS kieu_tong, sum(fare) AS tong_cuoc FROM trips;
 kieu_tong | tong_cuoc
-----------+-----------
 bigint    |    840000
(1 row)
```

**Chia tiền phải chọn cách làm tròn.** Chia đôi, chia ba một khoản (chia cước cho nhiều điểm giao, chia hoa hồng) là chỗ số nguyên đòi bạn ra quyết định:

```text
SELECT 100000 / 3 AS chia_integer, 100000 % 3 AS phan_du, 100000::numeric / 3 AS chia_numeric;
 chia_integer | phan_du |    chia_numeric
--------------+---------+--------------------
        33333 |       1 | 33333.333333333333
(1 row)
```

Chia `integer` cho `integer` cắt phần lẻ: ba phần 33.333 đ cộng lại thiếu 1 đ. `numeric` cũng không giải quyết — không ai trả 33.333,33 đ. Quy tắc nghiệp vụ phải nói phần dư thuộc về phần nào.

**Vì sao không dùng `money`?** Tài liệu cho biết phần lẻ của nó do thiết lập `lc_monetary` quyết định, và dữ liệu có thể không nạp được vào database có `lc_monetary` khác. Trên lab:

```text
SELECT current_setting('lc_monetary') AS lc_monetary, 45000::money AS cuoc_money;
 lc_monetary | cuoc_money
-------------+------------
 C           | $45,000.00
(1 row)
```

Cước 45.000 đồng được in thành 45 nghìn **đô la** với hai chữ số lẻ. Định dạng hiển thị là việc của ứng dụng; database nên giữ con số và đơn vị rõ ràng.

## 3. Thời điểm: `timestamptz`, không phải `timestamp`

PostgreSQL có hai kiểu ngày giờ đầy đủ, cùng 8 byte:

- **`timestamp with time zone`** (viết tắt `timestamptz`): một **khoảnh khắc** trên dòng thời gian. Tài liệu mô tả: chuỗi đầu vào có ghi múi giờ được đổi sang UTC; chuỗi không ghi múi giờ được hiểu theo tham số `TimeZone` của phiên; giá trị lưu bên trong là UTC và múi giờ gốc **không được giữ lại**. Khi xuất ra, giá trị được đổi sang múi giờ của phiên.
- **`timestamp without time zone`** (viết `timestamp` trần là kiểu này, theo chuẩn SQL): chỉ là **ngày + giờ trên mặt đồng hồ**, không gắn với nơi nào. Theo tài liệu, nếu chuỗi có ghi múi giờ, PostgreSQL lặng lẽ bỏ qua phần đó.

Đưa cùng một chuỗi — giờ đặt của chuyến 100008 — vào hai kiểu, ở phiên giờ Việt Nam rồi ở phiên UTC:

```sql
SET TIME ZONE 'Asia/Ho_Chi_Minh';
SELECT '2026-11-03 00:10:00+07'::timestamptz AS co_mui_gio,
       '2026-11-03 00:10:00+07'::timestamp   AS khong_mui_gio;
SET TIME ZONE 'UTC';
SELECT '2026-11-03 00:10:00+07'::timestamptz AS co_mui_gio,
       '2026-11-03 00:10:00+07'::timestamp   AS khong_mui_gio;
```

```text
       co_mui_gio       |    khong_mui_gio
------------------------+---------------------
 2026-11-03 00:10:00+07 | 2026-11-03 00:10:00
(1 row)

       co_mui_gio       |    khong_mui_gio
------------------------+---------------------
 2026-11-02 17:10:00+00 | 2026-11-03 00:10:00
(1 row)
```

Cột `timestamptz` hiển thị khác nhau nhưng là **cùng một khoảnh khắc** (00:10 giờ VN = 17:10 UTC hôm trước). Cột `timestamp` đã bỏ `+07` mà không báo gì: «00:10» không còn biết là giờ ở đâu. Hai nguồn ghi theo hai múi giờ vào cùng cột `timestamp` sẽ lệch 7 tiếng mà không cách nào tách lại.

Vì vậy mọi mốc sự kiện của Tới Nơi đều là `timestamptz`. Lab ghi mọi mốc trong fixture kèm `+07` và đặt `SET TIME ZONE 'Asia/Ho_Chi_Minh'` đầu mỗi tệp — **tên múi giờ đầy đủ** mang theo quy tắc lịch sử của vùng, còn chữ viết tắt chỉ là một độ lệch cố định. Chuỗi không ghi offset là chuỗi «đoán» theo phiên đang chạy: cùng một script nạp dữ liệu, chạy ở phiên UTC và phiên giờ VN sẽ ghi hai khoảnh khắc cách nhau 7 tiếng (bài tập 2).

## 4. Ngày nghiệp vụ: `date` và câu hỏi «ngày theo giờ nào?»

`date` (4 byte) lưu một ngày lịch, không có giờ. Nghe hợp với «ngày 02/11» của kế toán, nhưng **ngày của một khoảnh khắc phụ thuộc múi giờ**. Ép `timestamptz` về `date` bằng `::date` dùng múi giờ của **phiên**; muốn ngày theo giờ Việt Nam bất kể phiên đặt gì, đổi trước bằng `AT TIME ZONE` — biểu thức này đổi `timestamptz` thành `timestamp` *như giờ hiện trên đồng hồ ở vùng đó*. Trên lab, ở phiên UTC:

```sql
SELECT code,
       requested_at,
       requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh'         AS gio_vn,
       (requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::date AS ngay_vn,
       requested_at::date                                   AS ngay_theo_phien
FROM trips
WHERE code IN (100007, 100008)
ORDER BY code;
```

```text
  code  |      requested_at      |       gio_vn        |  ngay_vn   | ngay_theo_phien
--------+------------------------+---------------------+------------+-----------------
 100007 | 2026-11-02 16:50:00+00 | 2026-11-02 23:50:00 | 2026-11-02 | 2026-11-02
 100008 | 2026-11-02 17:10:00+00 | 2026-11-03 00:10:00 | 2026-11-03 | 2026-11-02
(2 rows)
```

Chuyến 100008 thuộc ngày 03/11 theo giờ VN nhưng ngày 02/11 theo phiên UTC. Tệp `05-select.sql` đếm cả fixture theo hai cách và ra **9** chuyến «ngày 02/11» ở phiên UTC so với **8** theo giờ Việt Nam. Đó chính là câu trả lời sai thứ hai của kế toán ở đầu bài. Cách viết bộ lọc khoảng thời gian đúng (khoảng nửa mở `[00:00 ngày 02/11, 00:00 ngày 03/11)` theo giờ VN) là bài [PG-007](/articles/postgresql-select-filter-order-limit).

Một bẫy khác cũng không báo lỗi: định dạng ngày khi nhập chuỗi. Kết quả thật ở cuối `03-fixture-invalid.sql`:

```text
 datestyle | chuoi_03_11_2026 | iso_2026_11_03
-----------+------------------+----------------
 ISO, MDY  | 2026-03-11       | 2026-11-03
(1 row)
```

Với `DateStyle` mặc định `ISO, MDY`, chuỗi `'03/11/2026'` được hiểu là **ngày 11 tháng 3**. Luôn viết ngày giờ theo ISO 8601: `'2026-11-03 09:00:00+07'`.

## 5. Định danh: `uuid`, identity và sequence

**`uuid` cho `id`.** Kiểu `uuid` lưu một giá trị 128 bit; dạng chữ chuẩn là 36 ký tự hex có gạch nối. Lưu bằng `uuid` thay vì `text` vừa gọn vừa được kiểm định dạng (`'khach-so-1'` bị chặn ở mục 1). Trên lab:

```text
SELECT pg_typeof(gen_random_uuid()) AS kieu, pg_column_size(gen_random_uuid()) AS byte_uuid,
       length(gen_random_uuid()::text) AS ky_tu_dang_chu, pg_column_size(100004::bigint) AS byte_bigint;
 kieu | byte_uuid | ky_tu_dang_chu | byte_bigint
------+-----------+----------------+-------------
 uuid |        16 |             36 |           8
(1 row)
```

`gen_random_uuid()` có sẵn trong lõi PostgreSQL (từ bản 13, không cần extension) và theo tài liệu trả về UUID **phiên bản 4 — ngẫu nhiên**. Lợi ích: ứng dụng tự sinh id trước khi ghi, id không lộ số lượng chuyến, gộp dữ liệu nhiều nguồn không lo trùng. Cái giá: gấp đôi `bigint`, khó đọc, giá trị ngẫu nhiên rải khắp index (đo ở Phần 3). PostgreSQL 18 bổ sung `uuidv7()` có thành phần thời gian; lab ghim bản 17 nên chưa dùng.

**Identity cho `code`.** `code bigint GENERATED ALWAYS AS IDENTITY (START WITH 100000)` là cột **identity**: theo tài liệu, nó gắn với một sequence ngầm, tự nhận giá trị khi thêm dòng, mặc nhiên `NOT NULL`. Với `ALWAYS`, giá trị ghi tay chỉ được nhận kèm `OVERRIDING SYSTEM VALUE` (với `BY DEFAULT` thì giá trị người dùng được ưu tiên). Lab chọn `ALWAYS` để không ai vô tình tự đặt mã chuyến:

```text
ERROR:  cannot insert a non-DEFAULT value into column "code"
DETAIL:  Column "code" is an identity column defined as GENERATED ALWAYS.
HINT:  Use OVERRIDING SYSTEM VALUE to override.
```

**Sequence không bảo đảm liên tục.** Sau khi chạy `00`→`03`, hỏi sequence của `code`:

```text
SELECT last_value FROM trips_code_seq;
 last_value
------------
     100028
(1 row)
```

Bảng vẫn chỉ có 15 chuyến (100000–100014), nhưng sequence đã cấp tới 100028: một câu hợp lệ bị `ROLLBACK` và 13 câu thêm chuyến bị NOT NULL / FK / CHECK từ chối đều đã lấy số. Các câu sai kiểu (`22P02`, `22003`, `22007`, `22008`) và `428C9` thì không tiêu số, vì bị từ chối trước khi cột identity được cấp số (lúc đọc giá trị hoặc lúc kiểm tra câu lệnh). Kết quả: chuyến tạo ở `04-lifecycle.sql` nhận mã **100029**. Tài liệu (mục kiểu `serial`) cũng cảnh báo cột lấy số từ sequence có thể có «lỗ hổng» dù không xoá dòng nào. Đừng dùng `code` để đếm, đừng viết logic kiểu «mã kế tiếp là max + 1».

**`integer` hay `bigint` cho số tăng dần?** Tính bằng số liệu của hồ sơ Tới Nơi: `trips` thêm khoảng 730 triệu dòng/năm ⇒ một cột `integer` (tối đa 2.147.483.647) cạn sau chưa tới 3 năm; `trip_events` khoảng 16 triệu dòng/ngày ⇒ cạn sau khoảng 134 ngày. Lab dùng `bigint` cho cả hai. Ngược lại, `cities.id` là `smallint` (tối đa 32.767) vì Tới Nơi chỉ có 6 thành phố.

## 6. NULL: «chưa có», không phải 0

NULL không phải 0 hay chuỗi rỗng; nó nói giá trị **không có**. Mỗi cột cho phép NULL phải ghi rõ nghĩa:

- `accepted_at IS NULL` = **chưa có ai nhận**. Đừng thay bằng ngày giả như `1970-01-01`: mọi phép tính thời gian chờ sẽ sai.
- `fare` là `NOT NULL`, và **0 đ là giá trị hợp lệ** — chuyến 100008 miễn phí do khuyến mãi. «Cước 0 đ» và «chưa biết cước» là hai sự thật khác nhau.
- `merchant_id IS NULL` = **không áp dụng** (không phải đơn đồ ăn), khác «chưa biết quán».

NULL còn ảnh hưởng tới ràng buộc: `CHECK (accepted_at >= requested_at)` cho qua chuyến chưa có `accepted_at`, vì biểu thức ra NULL chứ không ra false. Lab vì vậy viết các quy tắc «phải có» bằng `IS NULL` / `IS NOT NULL` — chi tiết ở [PG-005](/articles/postgresql-create-table-constraints); logic ba giá trị của NULL trong truy vấn thuộc Phần 2.

## 7. Data dictionary có đơn vị, NULL và múi giờ

**Data dictionary** (từ điển dữ liệu) là bảng mô tả từng cột: kiểu, đơn vị, miền giá trị, NULL nghĩa là gì, múi giờ. Đây là sản phẩm của bài — bản cho `trips`:

| Cột | Kiểu | Đơn vị / miền | NULL nghĩa là | Ghi chú |
|---|---|---|---|---|
| `id` | `uuid` | UUID v4 | không cho phép | định danh kỹ thuật, khoá chính |
| `code` | `bigint` identity | từ 100000, tăng, có thể nhảy cóc | không cho phép | mã cho khách/CSKH; không dùng để đếm |
| `kind` | `text` | `ride` / `food` / `parcel` | không cho phép | |
| `city_id` | `smallint` | id trong `cities` | không cho phép | |
| `customer_id` | `uuid` | id trong `customers` | không cho phép | |
| `driver_id` | `uuid` | id trong `drivers` | chưa có tài xế nhận | |
| `merchant_id` | `uuid` | id trong `merchants` | không áp dụng (không phải `food`) | |
| `status` | `text` | 8 trạng thái vòng đời | không cho phép | mặc định `requested` |
| `fare` | `integer` | **đồng**, ≥ 0, tối đa 2.147.483.647 | không cho phép | 0 đ = miễn phí, hợp lệ |
| `requested_at` | `timestamptz` | khoảnh khắc khách đặt | không cho phép | ngày nghiệp vụ tính theo `Asia/Ho_Chi_Minh` |
| `accepted_at` | `timestamptz` | khoảnh khắc tài xế nhận | chưa ai nhận | ≥ `requested_at` |
| `completed_at` | `timestamptz` | khoảnh khắc hoàn thành | chưa hoàn thành | chỉ có khi `completed` |
| `updated_at` | `timestamptz` | lần sửa gần nhất | không cho phép | |

Từ điển nên nằm gần dữ liệu: lab ghi phần quan trọng nhất vào catalog bằng `COMMENT ON COLUMN` (psql `\d+ trips` đọc được). Đọc lại bằng SQL:

```sql
SELECT a.attname AS cot, format_type(a.atttypid, a.atttypmod) AS kieu,
       NOT a.attnotnull AS cho_null, col_description(a.attrelid, a.attnum) AS chu_thich
FROM pg_attribute a
WHERE a.attrelid = 'toinoi.trips'::regclass AND a.attnum > 0 AND NOT a.attisdropped
ORDER BY a.attnum;
```

```text
     cot      |           kieu           | cho_null |                         chu_thich
--------------+--------------------------+----------+------------------------------------------------------------
 id           | uuid                     | f        | Định danh kỹ thuật UUID v4; ứng dụng hoặc database sinh
 code         | bigint                   | f        | Mã chuyến tăng dần cho người dùng và CSKH; có thể nhảy cóc
 kind         | text                     | f        |
 …
 fare         | integer                  | f        | Cước tính bằng đồng (VND), số nguyên không âm
 requested_at | timestamp with time zone | f        | Thời điểm khách đặt (timestamptz)
 accepted_at  | timestamp with time zone | t        | Thời điểm tài xế nhận; NULL khi chưa ai nhận
 …
(13 rows)
```

Sáu cột `kind`, `city_id`, `customer_id`, `driver_id`, `merchant_id`, `status` chưa có chú thích — khoảng trống bạn sẽ lấp một phần ở bài tập 1.

## 8. Lỗi thường gặp và đánh đổi

- **Tiền bằng `real`/`double precision`**: sai số không thấy khi hiển thị, chỉ lộ ra lúc đối soát.
- **Không ghi đơn vị**: cột `fare` không rõ đồng hay nghìn đồng thì sớm muộn có người nhân 1.000 hai lần.
- **`timestamp` cho mốc sự kiện, chuỗi thời gian không offset, `::date` không chỉ múi giờ**: kết quả phụ thuộc phiên.
- **`integer` cho số tăng dần của bảng lớn**: đổi kiểu cột trên bảng hàng trăm triệu dòng là migration nặng (PG-009).
- **Coi NULL như 0 hoặc dùng giá trị giả** (`1970-01-01`, `-1`).

Đánh đổi chính: số nguyên đồng gọn và nhanh nhưng buộc quy tắc làm tròn phải viết ra; `numeric` linh hoạt nhưng chậm hơn; `uuid` v4 tiện cho hệ phân tán nhưng tốn chỗ và rải ngẫu nhiên trên index.

## 9. Bài tập

**Bài 1.** Tới Nơi thêm bảng `payments`: mỗi lần khách thanh toán một chuyến có số tiền khách trả, tiền tip (có thể không có), phí cổng thanh toán tính theo tỷ lệ 1,5% và thời điểm cổng xác nhận (có thể chưa có). Lập data dictionary: kiểu, đơn vị, NULL nghĩa là gì. Viết thêm `COMMENT ON COLUMN` cho `trips.status`.

<details><summary><b>Lời giải</b></summary><p>Gợi ý một phương án: <code>amount integer NOT NULL</code> (đồng, ≥ 0); <code>tip integer NOT NULL DEFAULT 0</code> nếu «không tip» chỉ là 0 đ — hoặc cho NULL nếu cần phân biệt «khách chưa chọn tip»; ghi rõ lựa chọn. Tỷ lệ phí là phần lẻ thật: <code>numeric(5,4)</code> lưu <code>0.0150</code>, hoặc số nguyên «phần vạn» lưu <code>150</code>. Tiền phí tính ra phải làm tròn về đồng theo quy tắc viết ra. <code>confirmed_at timestamptz</code>, NULL = cổng chưa xác nhận. Chú thích: <code>COMMENT ON COLUMN trips.status IS 'Trạng thái hiện tại; 8 giá trị của vòng đời, mặc định requested';</code></p></details>

**Bài 2.** Một script nạp dữ liệu chạy ở phiên `TimeZone = 'UTC'` và ghi `requested_at` bằng chuỗi `'2026-11-03 08:00:00'` (không có offset) cho một chuyến khách đặt lúc 8 giờ sáng giờ Việt Nam. Ở phiên `Asia/Ho_Chi_Minh`, giá trị hiển thị là gì?

<details><summary><b>Lời giải</b></summary><p>Chuỗi được hiểu theo phiên UTC nên lưu khoảnh khắc 08:00 UTC; ở phiên giờ VN hiển thị <code>2026-11-03 15:00:00+07</code> — lệch 7 tiếng. (Đã kiểm trên lab: tạo bảng tạm ở phiên UTC rồi đọc ở phiên giờ VN.) Sửa bằng cách ghi <code>'2026-11-03 08:00:00+07'</code>.</p></details>

**Bài 3.** Hồ sơ Tới Nơi lưu toạ độ tài xế (`driver_locations.lat`, `lng`) bằng `double precision`. Có mâu thuẫn với lời khuyên «không dùng số thực cho tiền» không?

<details><summary><b>Lời giải</b></summary><p>Không. Toạ độ GPS là phép đo vốn đã có sai số vài mét; sai số làm tròn nhị phân nhỏ hơn rất nhiều bậc và không ai so bằng tuyệt đối hay cộng dồn toạ độ để đối soát. Tiền thì khác: phải chính xác tới từng đồng và phải cộng, so khớp với hệ khác. Chọn kiểu theo miền giá trị và cách dùng, không theo một luật chung «cấm float».</p></details>

## 10. Đọc tiếp

- [PG-005: CREATE TABLE và ràng buộc dữ liệu](/articles/postgresql-create-table-constraints) — biến data dictionary thành DDL với NOT NULL, UNIQUE, CHECK.
- [PG-007: SELECT đầu tiên](/articles/postgresql-select-filter-order-limit) — lọc theo khoảng thời gian giờ Việt Nam.
- Tài liệu PostgreSQL 17: [Numeric Types](https://www.postgresql.org/docs/17/datatype-numeric.html), [Monetary Types](https://www.postgresql.org/docs/17/datatype-money.html), [Date/Time Types](https://www.postgresql.org/docs/17/datatype-datetime.html), [AT TIME ZONE](https://www.postgresql.org/docs/17/functions-datetime.html), [UUID Type](https://www.postgresql.org/docs/17/datatype-uuid.html), [UUID Functions](https://www.postgresql.org/docs/17/functions-uuid.html), [CREATE TABLE — identity](https://www.postgresql.org/docs/17/sql-createtable.html).

Bài tiếp theo: [PG-005: CREATE TABLE và ràng buộc dữ liệu](/articles/postgresql-create-table-constraints)
