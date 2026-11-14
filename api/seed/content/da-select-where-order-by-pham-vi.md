---
slug: da-select-where-order-by-pham-vi
title: "SELECT, WHERE, ORDER BY: đọc đúng tập chuyến cần phân tích"
summary: "Viết truy vấn đầu tiên cho cohort Tới Nơi: chọn cột, lọc dịch vụ/thành phố/thời gian, kiểm tra ranh giới tuần và sắp xếp có thứ tự xác định trước khi lấy mẫu."
category: data-analysis
level: beginner
tags: [data-analysis, da-012, sql, filtering, toi-noi]
language: vi
---

Một truy vấn chạy thành công có thể trả sai tập dữ liệu. Nếu chọn ngày huỷ thay vì ngày nhận, lượt T08 biến mất khỏi cohort tuần đang phân tích dù nó đã được nhận trong tuần đó.

Ở bài này, ta biến hợp đồng phạm vi thành điều kiện SQL và kiểm tra từng lượt được đưa vào hoặc loại ra.

<div class="tkh-callout"><span class="tkh-callout-title">DA-012 · Phần 2/8 · Cơ bản</span><p>Đọc trước: <a href="/articles/da-bang-khoa-grain-mot-dong-du-lieu">DA-011</a>. Chạy setup.sql trong cùng kết nối trước các truy vấn. Đầu ra: truy vấn cohort và danh sách trường hợp biên; dataset là P02-LAB v1, độc lập với P01.</p></div>

## 1. SELECT chọn thông tin muốn đọc

```sql
SELECT trip_id, kind, city_id, status, accepted_at
FROM da_trips
ORDER BY trip_id;
```

SELECT nêu các cột đầu ra, FROM nêu bảng. Liệt kê cột giúp người đọc biết thông tin nào đang được sử dụng và tránh vô tình lấy thêm dữ liệu không cần thiết khi schema được mở rộng. Câu này trả về 14 dòng, nhưng chưa phải cohort chính.

SQL ở đây là truy vấn đọc. Chọn bớt cột không xoá cột nguồn; điều kiện WHERE không xoá các dòng ngoài phạm vi. Kết quả truy vấn là một tập dòng dùng để đọc hoặc xử lý tiếp.

## 2. WHERE mã hoá phạm vi nghiệp vụ

Hợp đồng P02: ride tại city_id=1, được nhận lần đầu trong khoảng từ đầu ngày 05/10 đến trước đầu ngày 12/10/2026 theo UTC+07. Câu lệnh tương ứng:

```sql
SELECT trip_id, status, accepted_at
FROM da_trips
WHERE kind='ride' AND city_id=1
  AND accepted_at >= TIMESTAMPTZ '2026-10-05 00:00:00+07'
  AND accepted_at <  TIMESTAMPTZ '2026-10-12 00:00:00+07'
ORDER BY accepted_at, trip_id;
```

Kết quả đúng gồm T01–T08, thứ tự thời gian là **T01, T02, T03, T04, T05, T06, T08, T07**. Không có yêu cầu nào buộc trip ID phải tăng theo thứ tự thời gian.

| Lượt | Vì sao giữ/loại? |
|---|---|
| T07 | Nhận 11/10 23:59:59, thuộc kỳ dù hoàn thành sang 12/10 |
| T08 | Nhận 10/10, thuộc kỳ dù huỷ 12/10 |
| T09 | Nhận đúng 12/10 00:00, thuộc kỳ sau |
| T10 | Nhận 04/10 23:59:59, thuộc kỳ trước |
| T11/T12 | Sai dịch vụ / sai thành phố |
| T13/T14 | Chưa từng được nhận trong fixture; không thuộc cohort đã nhận |

Nếu chỉ lọc `status='cancelled'`, bạn đã bỏ mẫu số của tỷ lệ huỷ. Hãy tạo cohort đầy đủ trước, rồi đếm các kết quả trong cohort đó.

## 3. Vì sao không dùng BETWEEN cho hai đầu tuần?

BETWEEN bao gồm cả hai đầu. Nếu kỳ đầu lấy tới 12/10 00:00 và kỳ sau bắt đầu đúng thời điểm đó, T09 có thể xuất hiện ở cả hai kỳ. Khoảng **[đầu kỳ, đầu kỳ kế tiếp)** bằng `>=` và `<` giúp tránh trùng lặp ở ranh giới.

Không chữa bằng cách ghi mốc cuối ngày là 23:59:59: dữ liệu có thể chứa phần giây lẻ và bạn sẽ bỏ sót. Mốc đầu ngày kế tiếp thể hiện hợp đồng rõ hơn.

Kiểu TIMESTAMPTZ kèm độ lệch múi giờ (offset) rõ ràng giúp hai literal xác định đúng một thời điểm. Ở DA-016 ta sẽ phân biệt kỹ hơn giữa thời điểm và ngày theo giờ địa phương; hiện tại đừng bỏ `+07` rồi phụ thuộc vào múi giờ mặc định của phiên làm việc.

## 4. AND, OR và dấu ngoặc

Giả sử muốn xem các lượt ride ở city 1 hoặc city 2:

```sql
SELECT trip_id, city_id
FROM da_trips
WHERE kind='ride' AND city_id IN (1,2)
ORDER BY trip_id;
```

`IN` giúp diễn đạt danh sách giá trị. Nếu viết `kind='ride' AND city_id=1 OR city_id=2`, phần `OR city_id=2` có thể đưa mọi dịch vụ ở city 2 vào do thứ tự ưu tiên toán tử. Dataset hiện chưa có food city 2 nên một truy vấn sai vẫn có thể cho kết quả trông đúng; đọc điều kiện và bổ sung test case mới là cách phát hiện.

Dấu ngoặc không chỉ để câu lệnh dài dễ nhìn hơn. Dấu ngoặc thể hiện đúng nhóm điều kiện: `kind='ride' AND (city_id=1 OR city_id=2)`.

## 5. ORDER BY và LIMIT không tạo mẫu đại diện

```sql
SELECT trip_id, accepted_at
FROM da_cohort
ORDER BY accepted_at, trip_id
LIMIT 3;
```

Ba lượt đầu là T01/T02/T03. `trip_id` là khoá phụ để quyết định thứ tự khi thời điểm trùng nhau. Không có ORDER BY, thứ tự trả về không được đảm bảo nên không thể làm cơ sở cho báo cáo; LIMIT 3 khi ấy không có nghĩa ba lượt sớm nhất.

Ba dòng đầu giúp đọc thử dữ liệu, không phải mẫu ngẫu nhiên đại diện cho cả tuần. Chúng dồn vào đầu tuần và có thể thiếu các lượt cuối kỳ. Khi rà soát dữ liệu, cần chủ động xem cả các trường hợp biên, NULL, huỷ và chưa hoàn tất.

## 6. Kiểm tra truy vấn bằng danh sách loại trừ

Một cách rà soát hữu ích là giải thích từng dòng không thuộc cohort, thay vì chỉ nhìn tổng 8. Tổng đúng có thể che tình huống bỏ nhầm một dòng và thêm nhầm một dòng khác.

```sql
SELECT trip_id FROM da_trips
WHERE trip_id NOT IN (SELECT trip_id FROM da_cohort)
ORDER BY trip_id;
```

Ở đây subquery dùng khoá chính không NULL, nên kết quả là T09–T14. Với nguồn có NULL, NOT IN cần được xem xét kỹ; DA-017 sẽ dùng NOT EXISTS để tránh một kiểu bẫy phổ biến.

## 7. Bài tập và tự kiểm tra

Sửa truy vấn cohort để lấy kỳ kế tiếp, bắt đầu từ 12/10, rồi quay về truy vấn chính. Trong fixture, T09 thuộc kỳ sau; đừng suy ra rằng trong vận hành thực tế kỳ sau cũng chỉ có một lượt, vì dataset chỉ gồm các trường hợp học tập.

**Câu hỏi:** Thêm `cancelled_at < '2026-10-12'` vào truy vấn cohort có đúng không?

<details><summary>Gợi ý trả lời</summary><p>Không đúng với câu hỏi về cohort đã nhận. Điều kiện này bỏ T08 và các lượt không có thời điểm huỷ, làm đổi cả tử/mẫu. Ngày huỷ là mốc cho báo cáo sự kiện huỷ, khác mốc xếp cohort nhận.</p></details>

Checklist: chọn cột cần thiết; đủ dịch vụ/vùng; mốc thời gian đúng; khoảng nửa mở; ORDER BY ổn định; hiểu giới hạn của LIMIT; kiểm từng ca biên.

Tham khảo: [PostgreSQL — Querying a Table](https://www.postgresql.org/docs/18/tutorial-select.html). Bài tiếp: [DA-013: tổng hợp số liệu](/articles/da-count-sum-avg-group-by).
