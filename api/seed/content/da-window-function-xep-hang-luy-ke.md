---
slug: da-window-function-xep-hang-luy-ke
title: "Window function: xếp hạng, luỹ kế và so sánh kỳ trước"
summary: "Giữ từng dòng nhưng tính theo cửa sổ: ROW_NUMBER, RANK, LAG và SUM OVER trong báo cáo Tới Nơi; xử lý ngày trống, frame và rò rỉ dữ liệu tương lai."
category: data-analysis
level: advanced
tags: [data-analysis, da-018, sql, window-function, toi-noi]
language: vi
---

Vận hành cần xem mỗi ngày nhận bao nhiêu lượt, luỹ kế từ đầu tuần và so với ngày trước. GROUP BY cho ra số lượt mỗi ngày, nhưng chưa tự tạo được luỹ kế hoặc lấy được giá trị của dòng trước. Window function giải quyết bước tính trên các dòng liên quan mà vẫn giữ từng dòng đầu ra.

<div class="tkh-callout"><span class="tkh-callout-title">DA-018 · Phần 2/8 · Nâng cao</span><p>Đọc trước: <a href="/articles/da-cte-subquery-buoc-kiem-tra">DA-017</a>. Đầu ra: báo cáo bảy ngày có luỹ kế và so sánh với ngày trước đó, cùng bảng xếp hạng nêu rõ chính sách xử lý hoà. Dùng P02-LAB, không suy rộng độ lớn của các chỉ số từ tám lượt giả lập.</p></div>

## 1. OVER giữ dòng, GROUP BY gộp dòng

```sql
SELECT trip_id,status,fare_vnd,
       AVG(fare_vnd) OVER (PARTITION BY status) AS status_average
FROM da_cohort ORDER BY trip_id;
```

Vẫn tám dòng. Mỗi dòng mang trung bình của nhóm status tương ứng: completed là 107.500, cancelled khoảng 76.666,67, accepted 70.000. PARTITION BY chia tập tính toán thành các nhóm; PARTITION BY không lọc bỏ nhóm khác khỏi đầu ra.

Nếu dùng GROUP BY status, chỉ còn ba dòng. Chọn cách nào phụ thuộc vào grain cần bàn giao, chứ không phải vì OVER trông nâng cao hơn.

## 2. Xếp hạng và chính sách hoà

```sql
SELECT trip_id,fare_vnd,
       ROW_NUMBER() OVER (ORDER BY fare_vnd DESC,trip_id) AS position,
       RANK() OVER (ORDER BY fare_vnd DESC) AS fare_rank,
       DENSE_RANK() OVER (ORDER BY fare_vnd DESC) AS dense_fare_rank
FROM da_cohort ORDER BY position;
```

T03 đứng đầu, T07 tiếp theo; T01/T06 cùng giá 100.000. ROW_NUMBER dùng trip_id để tạo thứ tự duy nhất; RANK và DENSE_RANK xem hai giá bằng nhau là hoà. RANK để lại khoảng trống thứ hạng sau nhóm hoà, DENSE_RANK thì không.

Nếu thêm trip_id vào ORDER BY của RANK, bạn đã phá vỡ thế hoà theo giá. Đừng sao chép cùng một ORDER BY cho mọi hàm mà không xem lại tiêu chí xếp hạng.

Muốn lọc top theo kết quả window, thường đặt phép tính trong CTE/subquery rồi lọc bên ngoài. WHERE trong cùng truy vấn được xử lý trước hàm window, nên ở đó không dùng được alias position như một cột nguồn.

## 3. Ngày không có dữ liệu vẫn là một ngày

Trong cohort, 05/10 có hai lượt; 06–11/10 mỗi ngày một lượt. Đối chiếu với fixture: ngày 06 có T03, ngày 07 có T04, ngày 08 có T05, ngày 09 có T06, ngày 10 có T08, ngày 11 có T07. **Cả bảy ngày đều có lượt** trong baseline này. Tuy vậy báo cáo nên có date spine — tập ngày đầy đủ — để báo cáo vẫn đúng khi một ngày không có lượt nào trong biến thể kiểm tra.

```sql
WITH days AS (
  SELECT DATE '2026-10-05'+i AS day FROM generate_series(0,6) AS g(i)
), daily AS (
  SELECT (accepted_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::date AS day,
         COUNT(*) AS n FROM da_cohort GROUP BY 1
), filled AS (
  SELECT d.day,COALESCE(x.n,0) AS n
  FROM days d LEFT JOIN daily x USING(day)
)
SELECT day,n,
  SUM(n) OVER (ORDER BY day ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) AS cumulative,
  LAG(n) OVER (ORDER BY day) AS previous_day
FROM filled ORDER BY day;
```

Số lượt là **2,1,1,1,1,1,1**; luỹ kế **2,3,4,5,6,7,8**; previous_day của dòng đầu là NULL, sau đó là 2,1,1,1,1,1. NULL ở đầu kỳ nghĩa là không có ngày trước trong tập, chứ không phải đã biết ngày trước có 0 lượt.

Không có date spine, LAG trả về dòng có dữ liệu trước đó, có thể cách nhiều ngày. Nếu cần so sánh đúng ngày lịch trước, phải đưa đủ ngày hoặc nối theo ngày một cách rõ ràng.

## 4. Frame xác định những dòng nào được cộng

`ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW` cộng từ đầu partition tới dòng hiện tại theo thứ tự đã chọn. Khi ORDER BY có giá trị trùng, frame mặc định của một số cách viết có thể gồm cả nhóm peer, khiến tổng nhảy cùng lúc ở tất cả các dòng trùng giá trị.

Với dữ liệu theo lượt, muốn luỹ kế từng dòng phải có thứ tự xác định và frame phù hợp. Với bảng daily đã có một dòng cho mỗi ngày, ngày là khoá sắp xếp duy nhất nên phép tính dễ rà soát hơn.

Không lấy trung bình của các tỷ lệ ngày để thay cho tỷ lệ tuần nếu số lượt mỗi ngày khác nhau. Luỹ kế số huỷ và số nhận rồi chia mới giữ đúng trọng số; đồng thời vẫn phải nêu rõ phần unknown.

## 5. Cửa sổ có thể đưa tương lai vào kết quả

`AVG(n) OVER ()` tính trên cả tập, nên dòng đầu tuần cũng “biết” số cuối tuần. Điều này hợp lệ nếu đang mô tả toàn tuần đã chốt, nhưng không hợp lệ nếu dùng làm tín hiệu ra quyết định ở đầu tuần, khi phần sau của tuần chưa xảy ra.

Tương tự, LEAD lấy dòng sau, frame có FOLLOWING nhìn sang các dòng phía sau. Các hàm ấy không sai; chúng chỉ cần phù hợp với thời điểm sử dụng. Ghi rõ đây là báo cáo lịch sử hay feature tại thời điểm quyết định, rồi chọn cửa sổ tương ứng.

Snapshot P02 có kết quả huỷ của T08 vào ngày 12/10, gắn với cohort ngày 10/10. Nếu phân tích “biết gì vào tối 10/10”, không được dùng kết quả ấy như đã biết; cần snapshot theo thời điểm hoặc metadata đủ để tái dựng.

## 7. Bài tập và tự kiểm tra

Giả sử không có lượt nào được nhận ngày 08/10, nhưng vẫn báo cáo bảy ngày. Luỹ kế sau ngày 08/10 và previous_day của ngày 09/10 phải có ý nghĩa gì?

<details><summary>Gợi ý trả lời</summary><p>Ngày 08 có n=0; luỹ kế giữ nguyên mức cuối ngày 07/10. LAG ở 09/10 phải trả về 0 của ngày 08/10, không nhảy về số của 07/10. Đây là lý do tạo date spine. Không chỉnh dataset chính, để tránh nhầm với đáp án baseline.</p></details>

**Câu hỏi:** Hai lượt cùng giá 100.000 có cần cùng position không?

<details><summary>Gợi ý trả lời</summary><p>ROW_NUMBER cho position duy nhất với khoá phụ ổn định; RANK theo giá xếp hai lượt vào cùng hạng. Chọn theo yêu cầu nghiệp vụ và ghi chính sách hoà, không coi hai kết quả khác nhau là lỗi database.</p></details>

Tham khảo: [PostgreSQL — Window Functions](https://www.postgresql.org/docs/18/tutorial-window.html). Bài tiếp: [DA-019: tiền và đối soát](/articles/da-tien-hoan-tien-doi-soat).
