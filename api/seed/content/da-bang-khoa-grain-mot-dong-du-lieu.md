---
slug: da-bang-khoa-grain-mot-dong-du-lieu
title: "Bảng, khoá và grain: một dòng dữ liệu đại diện cho điều gì?"
summary: "Bắt đầu SQL cho DA bằng bộ dữ liệu Tới Nơi: phân biệt một chuyến, một sự kiện và một lần thử thanh toán; kiểm tra khoá và grain trước khi đếm hoặc nối bảng."
category: data-analysis
level: beginner
tags: [data-analysis, da-011, sql, grain, toi-noi]
language: vi
---

Một lượt T01 có ba mốc nhận–đón–hoàn thành, hai lần thử thanh toán và hai ticket hỗ trợ. Nếu mỗi dòng đều được gọi là “một chuyến”, ba báo cáo sẽ cho ba câu trả lời khác nhau dù cùng nói về T01.

Trước câu SQL đầu tiên, DA cần ghi **grain**: một dòng trong tập dữ liệu đại diện cho điều gì. Đây là nền tảng để dùng đúng COUNT, JOIN và mọi chỉ số phía sau.

<div class="tkh-callout"><span class="tkh-callout-title">DA-011 · Phần 2/8 · Cơ bản</span><p>Đọc trước: <a href="/articles/da-thuc-hanh-ke-hoach-phan-tich-huy-chuyen">bài tổng hợp P01</a>. Đầu ra: data dictionary và bảng quan hệ cho lab P02. Không cần biết SQL trước; chạy bộ thực hành theo README tại <code>examples/toi-noi-da/p02-sql</code>.</p></div>

## 1. Bộ dữ liệu mới của Phần 2

P02-LAB v1 là snapshot giả lập tại **13/10/2026 00:00, Asia/Ho_Chi_Minh**. Bộ này có 14 lượt, gồm các trường hợp biên và ngoại lệ. Cohort chính gồm ride ở city_id=1, nhận lần đầu trong khoảng 05/10 đến trước 12/10. Đây là bộ dữ liệu học tập độc lập, không tái tạo 1.000/1.500 lượt của P01 và không ánh xạ city_id=1 thành khu vực A cũ.

Các đối tượng `da_*` là bảng/view tạm trong một kết nối PostgreSQL. Trường `picked_up_at`, `cancelled_at`, `history_complete` là dữ liệu phân tích bổ sung cho lab, không tuyên bố schema ứng dụng hiện có đã chứa chúng. Chính sách thanh toán/hoàn tiền trong fixture cũng không thay đổi quy tắc nghiệp vụ chung.

`setup.sql` tạo dữ liệu; `queries.sql` có truy vấn theo bài; `verify.sql` kiểm tra kết quả; `verify.py` chạy trên cụm PostgreSQL tạm và kiểm tra cả các khối SQL trong bài. Dữ liệu tự biến mất khi kết nối/cụm tạm đóng, không ghi vào database của blog.

## 2. Đọc một bảng như một hợp đồng

| Bảng | Một dòng | Khoá chính | Quan hệ cần nhớ |
|---|---|---|---|
| da_trips | Một lượt/chuyến trong snapshot | trip_id | Một lượt có thể có nhiều event/payment/ticket |
| da_events | Một mốc trạng thái được ghi nhận | event_id | trip_id nối về lượt |
| da_payments | Một lần thử thanh toán | payment_id | Không phải lần nào cũng thành công |
| da_refunds | Một yêu cầu hoàn tiền | refund_id | Nối về payment_id, không trực tiếp là giá chuyến |
| da_tickets | Một ticket gắn một lượt trong lab | ticket_id | Quy ước mỗi ticket gắn một lượt chỉ áp dụng trong lab |

Trong hệ thống thật, ticket có thể liên quan nhiều lượt. Cần kiểm tra mô hình thực tế trước khi đem truy vấn lab sang nguồn khác. Một cách thiết kế đơn giản trong bài học không trở thành quy luật chung.

## 3. Khoá chính và khoá ngoại giúp gì?

Khoá chính định danh duy nhất một dòng, không nhận NULL. Khoá ngoại kiểm tra liên kết tới dòng tồn tại trong bảng được tham chiếu. Chúng giúp bảo vệ cấu trúc, nhưng không chứng minh ý nghĩa nghiệp vụ đã đúng: hai event có hai ID khác nhau vẫn có thể là cùng một sự kiện gửi lại.

Ví dụ `payment_id` duy nhất không có nghĩa mỗi `trip_id` chỉ có một payment. T01 có P01F thất bại và P01 thành công. Nếu xoá dòng thất bại như “trùng trip_id”, ta mất lịch sử thử thanh toán; nếu cộng cả hai như tiền đã thu, ta tăng sai số tiền.

Truy vấn đầu tiên kiểm tra các grain:

```sql
SELECT (SELECT COUNT(*) FROM da_trips) AS trips,
       (SELECT COUNT(*) FROM da_events) AS events,
       (SELECT COUNT(*) FROM da_payments) AS payment_attempts;
```

Kỳ vọng: **14 lượt, 32 sự kiện, 10 lần thử thanh toán**. Không có lý do các số phải bằng nhau.

## 4. Nhìn một lượt qua nhiều bảng

```sql
SELECT trip_id, status, accepted_at, picked_up_at
FROM da_trips WHERE trip_id='T01';

SELECT payment_id, status, amount_vnd
FROM da_payments WHERE trip_id='T01'
ORDER BY payment_id;
```

Truy vấn đầu tiên trả về một dòng. Truy vấn thứ hai trả về hai dòng, có cùng số tiền yêu cầu 100.000 VND nhưng khác kết quả. `amount_vnd` là thuộc tính của từng lần thử (attempt); chỉ khi có trạng thái thành công và hợp đồng nguồn thì mới được dùng giá trị này làm khoản thu trong lab.

Tên cột nên giữ đơn vị: `fare_vnd` và `amount_vnd` đều là số nguyên đồng. `accepted_at` là một thời điểm có kiểu timestamptz. Việc đặt tên rõ giúp đọc câu lệnh dễ hiểu hơn nhưng không thay thế data dictionary về đơn vị, trạng thái và tính đầy đủ.

## 5. View có thể giữ hoặc đổi grain

`da_cohort` chỉ lọc từ `da_trips`, nên vẫn một dòng/lượt. `da_classified` thêm phân loại kết quả nhưng không nhân dòng. `da_cash` tổng hợp các nguồn nhiều dòng về một dòng/lượt rồi mới nối bảng, nên có thể dùng cùng grain với cohort.

Ngược lại, một truy vấn GROUP BY ngày tạo một dòng/ngày; không thể gọi COUNT(*) trên kết quả ấy là số lượt nữa. Phép JOIN với event có thể tạo nhiều dòng cho một lượt. Sau mỗi bước, nên viết một câu: “Đầu ra hiện tại là một dòng cho…”.

Kiểm tra nhanh tính duy nhất của grain cohort:

```sql
SELECT COUNT(*) AS rows, COUNT(DISTINCT trip_id) AS trips
FROM da_cohort;
```

Cả hai bằng **8**. Hai số bằng nhau là kiểm tra cần thiết trong fixture, chưa đủ để chứng minh cohort đúng phạm vi; điều đó còn phụ thuộc điều kiện lọc ở DA-012.

## 6. Sai lầm thường gặp

- Đếm event thành lượt: cần tổng hợp hoặc đếm theo khoá lượt, tuỳ câu hỏi.
- Dùng DISTINCT cho mọi lỗi: có thể che một phép nối sai mà tiền vẫn bị cộng sai.
- Cho rằng ID tăng theo thời gian nghiệp vụ: sequence có thể phản ánh thời điểm ghi, không phải thời điểm xảy ra.
- Coi khoá ngoại là bằng chứng dữ liệu đủ: nguồn có thể thiếu cả một nhóm lượt hợp lệ dù mọi liên kết còn lại đúng.

## 7. Bài tập và tự kiểm tra

Viết data dictionary cho năm bảng, gồm grain, khoá, đơn vị tiền/thời gian và một giới hạn. Sau đó trả lời: vì sao T01 có hai payment mà chỉ có một khoản thu thành công?

<details><summary>Gợi ý trả lời</summary><p>Mỗi payment là một lần thử (attempt). P01F thất bại, P01 thành công; chỉ P01 ghi nhận dòng tiền thu 100.000 trong hợp đồng lab. Giữ cả hai để phân tích tỷ lệ lỗi thanh toán, nhưng không cộng cả hai khi tính thu.</p></details>

**Câu hỏi:** Nếu một event bị gửi lại với ID mới, khoá chính (primary key) có tự phát hiện trùng nghiệp vụ không?

<details><summary>Gợi ý trả lời</summary><p>Không. Cần định danh sự kiện nghiệp vụ hoặc quy tắc khử trùng lặp theo hợp đồng nguồn. Khoá chính chỉ bảo đảm không trùng chính giá trị khoá ấy.</p></details>

Tham khảo: [PostgreSQL — Constraints](https://www.postgresql.org/docs/18/ddl-constraints.html). Bài tiếp: [DA-012: lọc và sắp xếp dữ liệu](/articles/da-select-where-order-by-pham-vi).
