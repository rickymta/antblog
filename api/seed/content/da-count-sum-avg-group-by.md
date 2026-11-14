---
slug: da-count-sum-avg-group-by
title: "COUNT, SUM, AVG và GROUP BY: tổng hợp số liệu đầu tiên"
summary: "Tổng hợp cohort Tới Nơi bằng SQL, kiểm tra COUNT cột và COUNT dòng, tránh chia số nguyên và hiểu vì sao tổng giá chuyến không phải tiền đã thu."
category: data-analysis
level: beginner
tags: [data-analysis, da-013, sql, aggregation, toi-noi]
language: vi
---

Cohort đã có đúng tám lượt. Bước tiếp là trả lời bao nhiêu lượt hoàn thành, bao nhiêu lượt huỷ, tổng giá đang ghi nhận là bao nhiêu. Các hàm tổng hợp biến nhiều dòng thành con số, nhưng cũng dễ làm mất thông tin về mẫu số và giá trị thiếu.

<div class="tkh-callout"><span class="tkh-callout-title">DA-013 · Phần 2/8 · Cơ bản</span><p>Đọc trước: <a href="/articles/da-select-where-order-by-pham-vi">DA-012</a>. Đầu ra: bảng tổng hợp theo trạng thái và phép đối soát. Mọi SQL dùng bảng/view tạm từ setup.sql của P02-LAB.</p></div>

## 1. COUNT dòng khác COUNT giá trị

```sql
SELECT COUNT(*) AS cohort_rows,
       COUNT(picked_up_at) AS rows_with_pickup,
       COUNT(cancel_reason) AS rows_with_reason
FROM da_cohort;
```

Kỳ vọng **8, 4, 2**. COUNT(*) đếm dòng; COUNT(cột) bỏ qua NULL. Chỉ có hai nhãn lý do dù có ba lượt huỷ: T04 thiếu nhãn. Lấy số nhãn lý do làm số lượt huỷ sẽ khiến tử số bị thấp.

COUNT(DISTINCT trip_id) đếm giá trị khoá khác nhau và bỏ NULL. Trong cohort hiện tại nó bằng COUNT(*) vì grain đã được bảo đảm. Sau khi nối bảng, hàm này có thể giúp đếm lượt nhưng không tự sửa các phép SUM bị nhân.

## 2. GROUP BY đổi grain thành một dòng/nhóm

```sql
SELECT status, COUNT(*) AS trips, SUM(fare_vnd) AS quoted_vnd
FROM da_cohort
GROUP BY status
ORDER BY status;
```

| status | trips | quoted_vnd |
|---|---:|---:|
| accepted | 1 | 70.000 |
| cancelled | 3 | 230.000 |
| completed | 4 | 430.000 |

Tổng cộng có tám lượt, với tổng giá ghi trên các lượt là **730.000 VND**. Giá của lượt huỷ hoặc đang chờ không tự trở thành khoản thu. Tên `quoted_vnd` nhắc người đọc rằng đây là con số giá trong fixture; đến DA-019 mới đối chiếu dòng tiền.

Sau GROUP BY, một dòng không còn là một lượt. Nếu đếm số dòng của bảng kết quả, bạn thu được ba trạng thái chứ không phải ba chuyến.

## 3. WHERE lọc dòng, HAVING lọc nhóm

```sql
SELECT status, COUNT(*) AS trips
FROM da_cohort
GROUP BY status
HAVING COUNT(*) >= 2
ORDER BY status;
```

Chỉ còn cancelled và completed. HAVING lọc dựa trên kết quả tổng hợp của từng nhóm, còn WHERE chọn các dòng đầu vào. Nếu bỏ nhóm nhỏ khỏi biểu đồ, phải ghi rõ tổng hiển thị không còn bao phủ toàn cohort.

Không đặt điều kiện “ít nhất hai lượt” lên từng dòng nguồn. Một dòng không biết kích thước nhóm cho tới bước tổng hợp. Việc tách hai tầng giúp truy vấn dễ đọc và tránh vô tình làm thay đổi mẫu số.

## 4. Tính tỷ lệ mà không mất phần thập phân

```sql
SELECT COUNT(*) AS accepted,
       COUNT(*) FILTER (WHERE outcome='cancelled_before_pickup') AS cancelled,
       ROUND(100.0 * COUNT(*) FILTER (WHERE outcome='cancelled_before_pickup')
             / NULLIF(COUNT(*),0),2) AS confirmed_cancel_pct
FROM da_classified;
```

Kết quả: **8, 3, 37,50%**. `100.0` giúp phép chia dùng số thập phân; chia hai số nguyên trước có thể mất phần lẻ. `NULLIF` biến mẫu số 0 thành NULL để không chia cho 0. FILTER chọn những dòng được tính vào riêng phép đếm đó, không loại các dòng ấy khỏi cohort chung.

Trong fixture còn T05 chưa biết kết quả trước khi đón. Vì vậy 37,50% là tỷ lệ huỷ **đã xác nhận trên cohort**, chưa phải tỷ lệ cuối. Giới hạn kịch bản là 37,50–50% nếu chỉ còn một kết quả này chưa biết. Bài DA-015 sẽ trình bày trạng thái ấy rõ hơn.

## 5. AVG trả lời câu hỏi nào?

```sql
SELECT ROUND(AVG(fare_vnd),2) AS average_quoted_vnd
FROM da_cohort WHERE status='completed';
```

Bốn lượt hoàn thành có tổng giá 430.000, trung bình **107.500 VND**. Đây là trung bình trên lượt hoàn thành, không đại diện cho mọi yêu cầu hay mọi lượt đã nhận.

AVG bỏ giá trị NULL. Nếu một nhóm có nhiều dữ liệu thiếu, trung bình có thể chỉ phản ánh nhóm được ghi nhận. Khi báo cáo, kèm số có giá trị, số thiếu và điều kiện lọc thay vì chỉ đưa một giá trị AVG.

Đừng lấy trung bình đơn giản các tỷ lệ nhóm khi mẫu số khác nhau. Tỷ lệ chung phải dựa trên tổng tử số và tổng mẫu số phù hợp; ví dụ chung đã học ở P01 vẫn áp dụng khi viết SQL.

## 6. Tập rỗng và số 0 không giống nhau

```sql
SELECT COUNT(*) AS rows, SUM(fare_vnd) AS total, AVG(fare_vnd) AS average
FROM da_cohort WHERE city_id=999;
```

Kết quả là **0, NULL, NULL**. COUNT có thể trả 0 khi không có dòng; SUM/AVG ở đây không trả 0. Có thể dùng COALESCE để hiển thị tổng 0 khi hợp đồng đã xác nhận không có dữ liệu phát sinh, nhưng không được dùng nó để che nguồn bị thiếu hoặc chưa cập nhật.

Với tỷ lệ của cohort rỗng, giữ NULL hoặc “không xác định”; 0% nghĩa là có mẫu số nhưng không có trường hợp nào xảy ra, khác với trường hợp không có lượt nào.

## 7. Bài tập và gợi ý lời giải

Tạo bảng trạng thái chỉ gồm số lượt và tỷ trọng trên tám lượt. Tự kiểm tra tỷ trọng cộng thành 100%, nhưng ghi riêng rằng accepted là kết quả chưa hoàn tất.

<details><summary>Gợi ý kết quả</summary><p>Accepted 1/8 = 12,5%; cancelled 3/8 = 37,5%; completed 4/8 = 50%. Đây là cơ cấu trạng thái snapshot. Không tự coi completed là phần bù duy nhất của cancelled vì còn accepted.</p></details>

**Câu hỏi:** Có thể tính tỷ lệ huỷ là 3/(3+4) rồi bỏ T05 không?

<details><summary>Gợi ý trả lời</summary><p>Đó là 42,86% trong nhóm có hai trạng thái cuối đã quan sát, khác với hợp đồng cohort tám lượt. Muốn dùng, phải đặt tên và giải thích tập dữ liệu khác này; không được đổi M-01 mà không nói rõ, cũng không coi T05 là không huỷ.</p></details>

Tham khảo: [PostgreSQL — Aggregate Functions](https://www.postgresql.org/docs/18/functions-aggregate.html). Bài tiếp: [DA-014: JOIN và nhân bản số liệu](/articles/da-join-cardinality-dem-trung).
