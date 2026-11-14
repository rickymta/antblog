---
slug: ba-sql-kiem-chung-gia-thuyet
title: "SQL cho BA: kiểm chứng giả thuyết bằng truy vấn"
summary: "Tính tỷ lệ trên dữ liệu tổng hợp, nhận diện join nhân dòng và giữ đúng đơn vị đếm trước khi dùng kết quả SQL để quyết định."
category: business-analysis
level: intermediate
tags: [business-analysis, ba-036, sql, metrics, toi-noi]
language: vi
---

Khi join ba chuyến với ba ticket, kết quả có bốn dòng vì một chuyến có hai ticket. COUNT(*) lúc này không còn đếm chuyến. SQL chạy không lỗi vẫn có thể trả một chỉ số sai nghĩa.

<div class="tkh-callout"><span class="tkh-callout-title">BA-036 · Phần 5/8 · Trung bình</span><p>Đầu ra: truy vấn và giải thích mẫu số. Dataset tổng hợp độc lập P01, không chứa dữ liệu thật. Câu SQL dùng PostgreSQL và không thay đổi bảng.</p></div>

## 1. Chốt câu hỏi

Ba lượt đã được nhận trong cùng cohort: T1 huỷ trước đón; T2/T3 không. Hai ticket thuộc T1, một thuộc T2. Cần tính tỷ lệ lượt huỷ và tỷ lệ lượt có ticket, không tính tỷ lệ ticket.

## 2. Đọc dữ liệu trước khi join

Đếm bằng tay: 1/3 huỷ, 2/3 có ticket. Đây là oracle nhỏ để kiểm truy vấn. Nếu kết quả khác, đừng sửa dữ liệu cho khớp SQL; xem join, filter và aggregate.

## 3. Truy vấn tránh nhân dòng

```sql
WITH trips(id, cancelled_before_pickup) AS (
  VALUES ('T1', true), ('T2', false), ('T3', false)
), tickets(id, trip_id) AS (
  VALUES ('S1', 'T1'), ('S2', 'T1'), ('S3', 'T2')
)
SELECT count(*) AS trips,
       count(*) FILTER (WHERE cancelled_before_pickup) AS cancelled,
       round(100.0 * count(*) FILTER (WHERE cancelled_before_pickup)
             / NULLIF(count(*), 0), 2) AS cancel_pct,
       count(*) FILTER (WHERE EXISTS (
         SELECT 1 FROM tickets s WHERE s.trip_id = t.id
       )) AS trips_with_ticket
FROM trips t;
```

Kỳ vọng: trips = 3, cancelled = 1, cancel_pct = 33.33, trips_with_ticket = 2. EXISTS chỉ kiểm tra có ít nhất một ticket hay không, nên không thêm dòng cho từng ticket.

## 4. Tại sao join trực tiếp gây nhầm?

LEFT JOIN tạo T1/S1, T1/S2, T2/S3, T3/null. COUNT(*) = 4; đếm cancelled trên đó = 2. Tỷ lệ 2/4 = 50% là sai so với câu hỏi tính theo chuyến. COUNT(DISTINCT) hoặc aggregate ticket trước có thể đúng, nhưng phải giữ đúng điều kiện và miền dữ liệu.

## 5. Dataset thật cần thêm hợp đồng đo

Truy vấn nhỏ đã giả định cohort và outcome hoàn tất. Khi áp dụng thật cần lọc thời gian/múi giờ, ID duy nhất, lượt chưa có kết quả và event trùng. Không dùng status cancelled hiện tại thay “huỷ trước đón” nếu mô hình khác cho phép huỷ sau đón.

NULLIF bảo vệ mẫu số 0; kết quả null nghĩa là không có tỷ lệ để báo cáo, không phải 0% tốt đẹp.

## 6. Kiểm thêm biến thể

Thêm ticket thứ ba cho T1: số lượt có ticket vẫn 2. Xoá hết ticket: vẫn 3 lượt và tỷ lệ huỷ không đổi. Dataset không có lượt: count 0, tỷ lệ null. Những biến thể này kiểm ý nghĩa, không chỉ cú pháp.

## 7. Thực hành

Viết phiên bản pre-aggregate ticket rồi join. Giải thích mỗi hàng trước/sau join đại diện gì. Chỉ dùng dữ liệu tổng hợp, không cần quyền truy cập production.

<details><summary>COUNT(*) trên LEFT JOIN có đếm hàng không có ticket?</summary><p>Có. COUNT(*) đếm dòng kết quả; COUNT(s.id) bỏ qua null nhưng vẫn đếm từng ticket, nên không cho ra số chuyến có ticket.</p></details>

## 8. Nguồn và bài tiếp

- [PostgreSQL — SELECT](https://www.postgresql.org/docs/current/sql-select.html).

Tiếp theo: [Mapping và đối soát](/articles/ba-mapping-du-lieu-doi-soat).
