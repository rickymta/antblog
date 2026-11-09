---
slug: da-cte-subquery-buoc-kiem-tra
title: "CTE và subquery: chia truy vấn thành bước kiểm tra được"
summary: "Tách cohort, phân loại và tổng hợp thành các bước SQL có grain rõ ràng. Dùng CTE, EXISTS và NOT EXISTS để giữ mẫu số, kiểm tra thiếu dữ liệu và tránh bẫy NULL."
category: data-analysis
level: intermediate
tags: [data-analysis, da-017, sql, cte, subquery, toi-noi]
language: vi
---

Một câu SQL dài có thể vừa lọc cohort, nối payment, chia tỷ lệ và gắn nhãn. Khi số sai, không ai biết bước nào làm đổi mẫu số. Tách truy vấn thành các bước có tên giúp tìm lỗi và review ý nghĩa trước khi tối ưu.

<div class="tkh-callout"><span class="tkh-callout-title">DA-017 · Phần 2/8 · Trung bình</span><p>Đọc trước: <a href="/articles/da-ngay-gio-timezone-cua-so-bao-cao">DA-016</a>. Đầu ra: truy vấn có các bước cohort → phân loại → tổng hợp, kèm kiểm tra đầu ra từng bước. Chạy trên setup.sql của P02-LAB.</p></div>

## 1. CTE đặt tên cho kết quả trung gian

CTE viết bằng WITH và tồn tại trong phạm vi câu lệnh. Nó không tự tạo bảng lâu dài. Tên nên mô tả công việc hoặc grain: scoped, classified, counts dễ review hơn a, b, c.

```sql
WITH scoped AS (
  SELECT * FROM da_cohort
), counts AS (
  SELECT COUNT(*) AS n,
         COUNT(*) FILTER (WHERE status='completed') AS completed
  FROM scoped
)
SELECT n,completed,ROUND(100.0*completed/NULLIF(n,0),2) AS completed_pct
FROM counts;
```

Kết quả là 8, 4, 50%. `scoped` vẫn một dòng/lượt; `counts` là một dòng/tập. CTE thứ hai đọc CTE trước trong cùng câu lệnh.

Trong review, hãy hỏi scoped có đúng tám ID không trước khi nhìn completed_pct. Tỷ lệ đúng định dạng không bảo đảm đầu vào đúng.

## 2. Một pipeline chỉ số có thể kiểm tra từng bước

```sql
WITH scoped AS (
  SELECT * FROM da_classified
), counts AS (
  SELECT COUNT(*) AS n,
    COUNT(*) FILTER (WHERE outcome='cancelled_before_pickup') AS cancelled,
    COUNT(*) FILTER (WHERE outcome='unknown') AS unresolved
  FROM scoped
)
SELECT n,cancelled,unresolved,
       ROUND(100.0*cancelled/NULLIF(n,0),2) AS lower_pct,
       ROUND(100.0*(cancelled+unresolved)/NULLIF(n,0),2) AS upper_pct
FROM counts;
```

Đầu ra **8, 3, 1, 37,50, 50,00**. Hai giới hạn dùng cùng mẫu số, không bỏ T05. Nếu cần điều tra, thay SELECT cuối bằng `SELECT trip_id,outcome FROM scoped ORDER BY trip_id` để nhìn phân loại trong cùng câu WITH.

Không chạy một SELECT rời tham chiếu scoped sau khi câu lệnh đã kết thúc: tên CTE không còn tồn tại. View tạm trong setup khác CTE ở chỗ có thể dùng qua nhiều câu lệnh trong cùng kết nối.

## 3. Subquery khi cần một tập hoặc một giá trị

```sql
SELECT trip_id,fare_vnd
FROM da_cohort
WHERE fare_vnd > (SELECT AVG(fare_vnd) FROM da_cohort)
ORDER BY trip_id;
```

Trung bình giá toàn cohort là 91.250; các lượt vượt mức này là T01/T03/T06/T07. Subquery ở đây trả một giá trị tổng hợp. Nếu một subquery dùng như giá trị lại trả nhiều dòng, PostgreSQL sẽ báo lỗi; đừng dùng LIMIT 1 chỉ để ém một quan hệ vốn chưa được xác định.

Đây là so sánh mô tả trong snapshot, không phải dự báo giá tại lúc đặt. Nếu dùng trung bình cả tuần làm feature dự đoán đầu tuần, bạn có thể đưa thông tin tương lai vào bài toán khác.

## 4. NOT EXISTS tìm thiếu quan hệ một cách rõ ràng

```sql
SELECT c.trip_id
FROM da_cohort c
WHERE NOT EXISTS (
  SELECT 1 FROM da_payments p WHERE p.trip_id=c.trip_id
)
ORDER BY c.trip_id;
```

Chỉ T05 chưa có attempt. Đây khác “chưa có payment thành công”, vốn còn có T04/T08. Câu hỏi khác nhau thì điều kiện trong subquery phải khác.

NOT IN có bẫy khi tập phía phải chứa NULL: các phép so sánh có thể trở thành unknown thay vì true. Với khoá NOT NULL trong lab, ví dụ DA-012 không gặp bẫy đó; với nguồn ngoài không có ràng buộc, nên review hoặc dùng NOT EXISTS diễn đạt quan hệ cần kiểm tra.

## 5. CTE không bảo đảm nhanh hơn hoặc được lưu sẵn

PostgreSQL có thể gộp CTE vào truy vấn cha hoặc xử lý riêng tuỳ đặc điểm và lựa chọn tối ưu. Không xem WITH như một cam kết materialize hay cơ chế cache mặc định. Khi cần tối ưu, đo plan và workload thay vì đổi mọi subquery thành CTE vì tin sẽ nhanh.

Ở P02, mục tiêu là độ đúng và khả năng đọc. Giữ bước trung gian có ý nghĩa; không chia từng phép cộng thành một CTE nếu làm người đọc mất dấu grain.

## 6. Đặt điểm đối soát giữa các bước

| Bước | Grain | Kiểm tra |
|---|---|---|
| Scoped | Một dòng/lượt | Đúng T01–T08, không có T09–T14 |
| Classified | Một dòng/lượt | 3 huỷ, 4 đã đón, 1 unknown; tổng 8 |
| Payment aggregate | Một dòng/lượt có khoản thu | Tổng thành công 450.000 |
| Joined report | Một dòng/lượt | Vẫn 8 dòng, số tiền không bị nhân |
| Counts/daily | Một dòng/tập hoặc ngày | Tổng nhóm khớp mức tuần, tỷ lệ dùng mẫu số tương ứng |

Tách bước giúp kiểm tra nhưng không tự tạo kiểm tra. Bộ `verify.sql` thực hiện assertions để báo lỗi nếu các điều kiện mong muốn không còn đúng.

## 7. Bài tập và lời giải

Viết truy vấn tìm lượt có ticket nhưng không có payment thành công. Không nối trực tiếp ticket và payment nếu chỉ cần điều kiện có/không.

<details><summary>Gợi ý đáp án</summary><p>Dùng EXISTS ticket và NOT EXISTS payment status succeeded theo trip_id. Kết quả T04. T02 có thu thành công rồi hoàn nên không thuộc tập “không có khoản thu thành công”; net 0 không đồng nghĩa chưa thu.</p></details>

**Câu hỏi:** CTE `counts` chỉ còn bảy lượt vì loại unknown; có thể sửa SELECT cuối để khôi phục tỷ lệ cohort tám lượt không?

<details><summary>Gợi ý trả lời</summary><p>Phải quay lại bước đã làm mất dòng hoặc lấy mẫu số từ đúng cohort. Chỉnh định dạng/chia ở SELECT cuối không phục hồi thông tin đã bị bỏ. Tốt hơn giữ cohort và đếm từng lớp bằng FILTER.</p></details>

Tham khảo: [PostgreSQL — WITH Queries](https://www.postgresql.org/docs/18/queries-with.html), [Subquery Expressions](https://www.postgresql.org/docs/18/functions-subquery.html). Bài tiếp: [DA-018: window function](/articles/da-window-function-xep-hang-luy-ke).
