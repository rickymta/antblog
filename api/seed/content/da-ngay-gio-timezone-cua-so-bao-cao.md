---
slug: da-ngay-gio-timezone-cua-so-bao-cao
title: "Ngày giờ, múi giờ và cửa sổ báo cáo"
summary: "Lượt nhận cuối tuần nhưng huỷ hoặc hoàn tiền sang tuần sau thuộc báo cáo nào? Dùng timestamptz, AT TIME ZONE và biên nửa mở để giữ đúng cohort và kỳ dòng tiền."
category: data-analysis
level: intermediate
tags: [data-analysis, da-016, sql, timezone, cohort, toi-noi]
language: vi
---

T07 được nhận lúc 23:59:59 ngày 11/10, hoàn thành ngày 12/10. T08 nhận ngày 10/10 nhưng huỷ ngày 12/10. Chúng đều thuộc cohort nhận tuần 05–11/10, dù các sự kiện kết quả nằm ngoài tuần.

Không có một “ngày của chuyến” duy nhất phù hợp mọi báo cáo. DA phải chọn mốc thời gian theo câu hỏi và giữ nó xuyên suốt tử số, mẫu số và cách đối soát.

<div class="tkh-callout"><span class="tkh-callout-title">DA-016 · Phần 2/8 · Trung bình</span><p>Đọc trước: <a href="/articles/da-null-case-trang-thai-nghiep-vu">DA-015</a>. Đầu ra: quy tắc thời gian và truy vấn không phụ thuộc múi giờ của phiên. Bộ dữ liệu chốt tại 13/10/2026 00:00 +07; không dùng now() để thay ngày chốt của fixture.</p></div>

## 1. Thời điểm và ngày địa phương khác nhau

`timestamptz` biểu diễn một thời điểm; cách hiển thị chịu ảnh hưởng của múi giờ trong phiên. Kiểu này không giữ nguyên tên múi giờ mà người nhập đã dùng. `timestamp without time zone` biểu diễn ngày giờ không kèm quy tắc chuyển thành thời điểm toàn cục.

Để lấy ngày vận hành, chuyển thời điểm sang múi giờ cần dùng rồi mới ép về date:

```sql
SELECT trip_id,
       accepted_at,
       (accepted_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::date AS accepted_day,
       (cancelled_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::date AS cancelled_day
FROM da_cohort WHERE trip_id IN ('T07','T08') ORDER BY trip_id;
```

T07 có accepted_day=11/10; T08 có accepted_day=10/10, cancelled_day=12/10. Việc ép `accepted_at::date` trực tiếp phụ thuộc múi giờ của phiên và có thể đổi ngày khi chạy ở môi trường khác.

## 2. Cố định ranh giới kỳ báo cáo bằng thời điểm rõ ràng

```sql
SELECT trip_id
FROM da_trips
WHERE kind='ride' AND city_id=1
  AND accepted_at >= TIMESTAMPTZ '2026-10-05 00:00+07'
  AND accepted_at <  TIMESTAMPTZ '2026-10-12 00:00+07'
ORDER BY trip_id;
```

Truy vấn vẫn trả T01–T08 ngay cả khi múi giờ của phiên là UTC. Độ lệch múi giờ (offset) ở hai literal đã xác định thời điểm, còn cách hiển thị có thể khác. Bộ kiểm tra lab đổi múi giờ của phiên để xác nhận tính ổn định này.

Ở các vùng có giờ mùa hè, một ngày địa phương không phải lúc nào cũng dài 24 giờ. Khi mở rộng bài toán, hãy xác định biên từ lịch địa phương và tên múi giờ phù hợp thay vì cộng cứng số giây. P02 chỉ dùng Asia/Ho_Chi_Minh với các mốc đã cho.

## 3. Báo cáo cohort và báo cáo sự kiện

```sql
SELECT COUNT(*) AS cancellations_happened_in_week
FROM da_trips
WHERE kind='ride' AND city_id=1
  AND cancelled_at >= TIMESTAMPTZ '2026-10-05 00:00+07'
  AND cancelled_at <  TIMESTAMPTZ '2026-10-12 00:00+07';
```

Câu này đếm T02, T04, T14: **3 sự kiện huỷ trong tuần**. Cohort nhận cũng có **3 lượt huỷ** nhưng gồm T02, T04, T08. Hai tổng bằng nhau là do tình cờ, còn các đối tượng thì khác nhau.

T14 chưa từng được nhận nên không nằm trong cohort; T08 huỷ sau biên tuần nhưng vẫn nằm trong cohort. Nếu chỉ đối soát tổng 3 với 3 thì có thể bỏ sót lỗi phạm vi. Cần kiểm tra danh sách khoá và ý nghĩa mốc.

## 4. Ngày kết quả khác thời điểm biết kết quả

Một sự kiện có thể xảy ra trước ngày chốt nhưng tới kho báo cáo muộn. Để tái tạo “chúng ta biết gì tại thời điểm đó”, cần có thời điểm ghi nhận (ingest) hoặc snapshot đã lưu. Chỉ lọc occurred_at không đủ để tái dựng những gì ta đã biết vào thời điểm đó khi dữ liệu tới trễ hay bị sửa.

Lab là một snapshot cố định đã cung cấp, không mô phỏng đầy đủ thời điểm xảy ra sự kiện (event time) và thời điểm nạp dữ liệu (ingestion time). Không dùng snapshot này để khẳng định pipeline vận hành đã xử lý đúng mọi ca trễ. Khi P03 bổ sung tracking, vấn đề này cần metadata và kiểm thử riêng.

## 5. Chốt cohort không phải chốt số tiền theo tuần

T07 thuộc cohort tuần nhưng được thu lúc 12/10 00:35. Hoàn tiền R03 của T03 cũng xảy ra ngày 12/10. Nếu báo cáo tiền của cohort đến snapshot, cả hai được tính; nếu báo cáo dòng tiền phát sinh trong tuần 05–11/10, cả hai nằm ngoài cửa sổ đó.

Tên cột cần nói rõ: `cohort_net_cash_vnd` và `cash_settled_in_week` trả lời hai câu hỏi. Không yêu cầu chúng bằng nhau; đối soát bằng danh sách giao dịch đi qua ranh giới kỳ báo cáo.

## 6. Khoảng chờ phải nêu đơn vị và tập dữ liệu được đo

```sql
SELECT trip_id,
       ROUND(EXTRACT(EPOCH FROM (picked_up_at-accepted_at))/60.0,2) AS wait_minutes
FROM da_cohort WHERE picked_up_at IS NOT NULL
ORDER BY trip_id;
```

T01 5 phút, T03 10, T06 20, T07 khoảng 15,02 phút vì thời điểm nhận rơi vào giây thứ 59. Câu này chỉ mô tả nhóm đã đón, không đo trải nghiệm chờ của T02/T04/T08/T05. Không thay NULL thành 0 phút rồi lấy trung bình toàn cohort.

Khoảng thời gian âm là dấu hiệu cần kiểm tra đồng hồ hoặc thứ tự sự kiện, không phải một giá trị để tự đổi dấu. Nên kiểm tra và ghi log trước khi dùng trong báo cáo.

## 7. Bài tập và gợi ý lời giải

Giải thích vì sao tổng huỷ theo sự kiện và theo cohort cùng là 3 mà vẫn không thể dùng thay nhau. Viết danh sách lượt của mỗi tập dựa trên dữ kiện ở trên.

<details><summary>Gợi ý trả lời</summary><p>Sự kiện trong tuần gồm T02/T04/T14; cohort nhận gồm T02/T04/T08. T14 chưa nhận, T08 huỷ tuần sau. Mẫu số M-01 là cohort đã nhận, nên tử số phải là các lượt của chính cohort ấy.</p></details>

**Câu hỏi:** Chạy lại báo cáo vào tháng sau với now() có tái tạo được báo cáo của fixture không?

<details><summary>Gợi ý trả lời</summary><p>Không nếu now() thay cửa sổ hoặc ngày chốt. Cần tham số kỳ báo cáo và snapshot hoặc phiên bản rõ ràng. Trong lab, các mốc cố định giúp mọi lần chạy trả cùng dữ kiện; hệ thống thật cần cơ chế lưu trữ để bảo đảm khả năng tái lập phù hợp.</p></details>

Tham khảo: [PostgreSQL — Date/Time Functions](https://www.postgresql.org/docs/18/functions-datetime.html). Bài tiếp: [DA-017: CTE và subquery](/articles/da-cte-subquery-buoc-kiem-tra).
