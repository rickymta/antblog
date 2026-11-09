---
slug: da-join-cardinality-dem-trung
title: "JOIN và lỗi nhân bản dữ liệu khi nối chuyến với thanh toán"
summary: "Một lượt có hai payment và hai ticket có thể thành bốn dòng. Thực hành JOIN, EXISTS và tổng hợp trước khi nối để giữ đúng số lượt và số tiền trong báo cáo Tới Nơi."
category: data-analysis
level: intermediate
tags: [data-analysis, da-014, sql, join, cardinality, toi-noi]
language: vi
---

Báo cáo T01 bỗng có bốn dòng và tổng payment 400.000, dù bảng chuyến chỉ có một T01. Database không tính nhầm: truy vấn đã yêu cầu mọi cặp payment–ticket phù hợp.

JOIN cần được thiết kế theo **cardinality**, số dòng mỗi phía có thể ghép với nhau, và grain mong muốn ở đầu ra.

<div class="tkh-callout"><span class="tkh-callout-title">DA-014 · Phần 2/8 · Trung bình</span><p>Đọc trước: <a href="/articles/da-count-sum-avg-group-by">DA-013</a>. Đầu ra: truy vấn một dòng/lượt có tổng thu và cờ hỗ trợ, kèm đối soát trước/sau join. Dùng setup.sql của P02-LAB.</p></div>

## 1. Quan sát phép nhân trên T01

```sql
SELECT c.trip_id, p.payment_id, p.status, t.ticket_id, p.amount_vnd
FROM da_cohort c
JOIN da_payments p USING (trip_id)
JOIN da_tickets t USING (trip_id)
WHERE c.trip_id='T01'
ORDER BY p.payment_id,t.ticket_id;
```

P01/P01F mỗi dòng ghép với S01/S02, tạo **2 × 2 = 4 dòng**. Cả hai attempt đều có amount 100.000, nên cộng trực tiếp thành 400.000. Trong khi khoản thu thành công chỉ là P01, 100.000.

Lọc payment thành công trước vẫn còn hai dòng do hai ticket, nên SUM vẫn sai thành 200.000. Hai vấn đề độc lập: chọn đúng trạng thái tiền và giữ đúng grain khi nối.

## 2. DISTINCT không chữa mọi phép tổng hợp

COUNT(DISTINCT trip_id) có thể trả lại một lượt, nhưng SUM(amount_vnd) vẫn bị nhân. `SUM(DISTINCT amount_vnd)` lại loại theo **giá trị tiền**, không theo khoản thu: hai lượt khác nhau đều thu 100.000 sẽ bị gộp thành một.

Trong cohort, T01 và T06 đều thu thành công 100.000. Đó là hai khoản hợp lệ. Hãy bảo vệ định danh giao dịch và grain, không dùng giá trị tiền như khoá dedup.

Nếu nguồn gửi bản sao cùng giao dịch, việc loại trùng cần một quy tắc theo hợp đồng nguồn trước khi tổng hợp; không tuỳ tiện lấy một dòng mà chưa biết các dòng có mâu thuẫn không.

## 3. Chỉ cần biết có ticket thì dùng EXISTS

```sql
SELECT c.trip_id,
       EXISTS (SELECT 1 FROM da_tickets t WHERE t.trip_id=c.trip_id) AS has_ticket
FROM da_cohort c
ORDER BY c.trip_id;
```

Tám dòng được giữ nguyên; T01, T02, T04 có ticket. EXISTS trả lời câu hỏi có hay không, không mang các dòng ticket vào đầu ra. Có **3/8 lượt có ticket**, không phải năm lượt vì có năm ticket.

Đây là chỉ số trong fixture chưa có cửa sổ ticket, không tự là guardrail khiếu nại của P01. Muốn đo khiếu nại cần định nghĩa loại ticket, thời gian theo dõi, độ phủ và mapping nguồn riêng.

## 4. Tổng hợp từng phía trước khi nối

```sql
WITH paid AS (
  SELECT trip_id, SUM(amount_vnd) AS captured_vnd
  FROM da_payments WHERE status='succeeded'
  GROUP BY trip_id
)
SELECT c.trip_id, COALESCE(p.captured_vnd,0) AS captured_vnd
FROM da_cohort c LEFT JOIN paid p USING(trip_id)
ORDER BY c.trip_id;
```

`paid` có tối đa một dòng/trip_id. Nối với cohort giữ tám dòng, tổng captured là 450.000. T05 không có payment; LEFT JOIN giữ T05, còn INNER JOIN sẽ loại nó.

COALESCE thành 0 hợp lệ ở đây vì fixture cam kết export khoản thu đầy đủ đến snapshot. Với nguồn không đủ, NULL sau join có thể là thiếu dữ liệu chứ không phải không phát sinh; cần cờ coverage thay vì chỉ điền 0.

## 5. Điều kiện ở WHERE có thể phá mục đích LEFT JOIN

```sql
SELECT c.trip_id, p.payment_id
FROM da_cohort c
LEFT JOIN da_payments p ON p.trip_id=c.trip_id AND p.status='succeeded'
ORDER BY c.trip_id,p.payment_id;
```

Điều kiện trạng thái nằm trong ON nên lượt không có payment thành công vẫn được giữ, các cột p là NULL. Nếu chuyển thành `WHERE p.status='succeeded'`, các dòng NULL bị loại và kết quả không còn giữ toàn bộ cohort.

Câu này vẫn có thể nhân dòng nếu một lượt có nhiều lần thu thành công. Fixture hiện mỗi lượt có tối đa một, nhưng không nên suy ra đó là invariant của mọi hệ thống. Nếu đầu ra cần một dòng/lượt, dùng tổng hợp như bước trước.

## 6. Đối soát sau mỗi bước

Một bảng review nên có: số dòng, số trip_id duy nhất, tổng tiền theo nguồn đã lọc và số lượt không khớp. Nếu số dòng tăng mà số lượt không tăng, hãy tìm quan hệ một–nhiều trước khi tin mọi SUM.

```sql
SELECT COUNT(*) AS rows, COUNT(DISTINCT trip_id) AS trips,
       SUM(captured_vnd) AS captured_vnd
FROM da_cash;
```

Kỳ vọng **8, 8, 450.000**. View `da_cash` còn tổng hợp refunds riêng trước khi nối. Việc có hai nguồn nhiều dòng là lý do phải kiểm từng phía, không chỉ đổi loại JOIN.

## 7. Bài tập và lời giải

Viết kế hoạch tạo báo cáo một dòng/lượt có số ticket và số thu thành công. Chọn grain cho mỗi bước và nêu kiểm tra cần chạy. Nếu thêm một ticket thứ ba cho T01, tiền thu có được đổi không?

<details><summary>Gợi ý trả lời</summary><p>Tổng hợp ticket về trip_id bằng COUNT; tổng hợp payment thành công về trip_id bằng SUM; LEFT JOIN hai kết quả vào cohort. T01 có thêm ticket thì ticket_count tăng, captured vẫn 100.000. Kiểm tra tám trip_id, tám dòng và tổng captured 450.000 nếu dữ liệu tiền không đổi.</p></details>

**Câu hỏi:** LEFT JOIN có tự bảo đảm giữ nguyên số dòng phía trái không?

<details><summary>Gợi ý trả lời</summary><p>Nó giữ sự xuất hiện của dòng trái không có đối tác, nhưng một dòng trái có nhiều đối tác vẫn tạo nhiều dòng. Muốn đúng một dòng/trip, cần phía phải duy nhất theo khoá join hoặc dùng EXISTS/tổng hợp phù hợp.</p></details>

Tham khảo: [PostgreSQL — Joins Between Tables](https://www.postgresql.org/docs/18/tutorial-join.html). Bài tiếp: [DA-015: NULL và phân loại trạng thái](/articles/da-null-case-trang-thai-nghiep-vu).
