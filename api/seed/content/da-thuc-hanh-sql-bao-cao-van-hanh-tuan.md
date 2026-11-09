---
slug: da-thuc-hanh-sql-bao-cao-van-hanh-tuan
title: "Thực hành: bộ SQL báo cáo vận hành một tuần"
summary: "Tổng hợp Phần 2 bằng bộ SQL có thể chạy: cohort tám lượt, tỷ lệ huỷ tạm tính, ngày chờ kết quả, tiền thu/hoàn và checklist kiểm tra trước khi bàn giao báo cáo Tới Nơi."
category: data-analysis
level: advanced
tags: [data-analysis, da-020, sql, capstone, reconciliation, toi-noi]
language: vi
---

Người vận hành cần báo cáo tuần 05–11/10: nhận bao nhiêu lượt, huỷ bao nhiêu, còn bao nhiêu chưa có kết quả và số tiền liên quan. Bạn đã học từng phép SQL; bài cuối phần yêu cầu ghép chúng thành một sản phẩm có thể chạy lại và giải thích được.

Báo cáo đúng không chỉ là một SELECT không báo lỗi. Nó phải giữ đúng cohort, giữ các trạng thái chưa biết, đối soát tiền và ghi rõ đâu là dữ liệu chốt, đâu là số tạm tính.

<div class="tkh-callout"><span class="tkh-callout-title">DA-020 · Phần 2/8 · Nâng cao</span><p>Đọc DA-011–DA-019, gần nhất <a href="/articles/da-tien-hoan-tien-doi-soat">bài đối soát tiền</a>. Đầu ra: bộ truy vấn, bảng kết quả, kiểm tra và bản bàn giao. Lab: <code>examples/toi-noi-da/p02-sql</code>; không cần dữ liệu hoặc tài khoản production.</p></div>

## 1. Đọc hợp đồng báo cáo trước khi viết SQL

Dataset P02-LAB v1 gồm 14 lượt giả lập. Cohort chính chọn ride/city_id=1, accepted_at từ 05/10/2026 00:00 +07 tới trước 12/10/2026 00:00 +07. Snapshot chốt 13/10 00:00 +07. Có tám lượt T01–T08.

Một kết quả T05 vẫn chưa biết; báo cáo phải ghi tỷ lệ huỷ xác nhận trên cohort, không tự đổi thành tỷ lệ cuối. Tiền của cohort tính các khoản thành công đến snapshot, không giới hạn ở tuần settled_at trừ khi đang làm báo cáo dòng tiền theo sự kiện riêng.

Dataset này độc lập P01, không dùng để suy luận chất lượng thực tế của Tới Nơi. Các bảng tạm có cột phân tích bổ sung và thanh toán giả lập, không phải migration cho backend.

## 2. Chạy lab và đọc sản phẩm kiểm tra

Trong thư mục lab, README hướng dẫn hai cách: chạy `verify.py` để tạo cụm PostgreSQL tạm độc lập, hoặc dùng psql trên một database lab với setup/queries/verify trong **cùng một kết nối**. TEMP table không tồn tại qua các lần kết nối riêng.

Bộ tự động có các việc: nạp fixture; chạy truy vấn tham chiếu; kiểm tra số mong đợi; đổi timezone để phát hiện phụ thuộc ngầm; thử thiếu lịch sử và cohort rỗng; chạy các khối SQL trong mười bài. Lỗi SQL hoặc assertion khiến tiến trình thất bại.

Các kiểm tra không chứng minh mọi truy vấn production đúng. Chúng bảo vệ các tình huống và hợp đồng của lab, giúp người học phát hiện khi sửa truy vấn làm thay đổi ý nghĩa.

## 3. Báo cáo ngày: tổng hợp trước, giữ đủ lịch

```sql
WITH days AS (
  SELECT DATE '2026-10-05'+i AS day FROM generate_series(0,6) AS g(i)
), daily AS (
  SELECT (c.accepted_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::date AS day,
    COUNT(*) AS accepted,
    COUNT(*) FILTER (WHERE c.outcome='cancelled_before_pickup') AS cancelled,
    COUNT(*) FILTER (WHERE c.outcome='unknown') AS unknown,
    SUM(m.net_cash_vnd) AS net_cash_vnd
  FROM da_classified c JOIN da_cash m USING(trip_id)
  GROUP BY 1
)
SELECT d.day,COALESCE(x.accepted,0) AS accepted,
       COALESCE(x.cancelled,0) AS cancelled,COALESCE(x.unknown,0) AS unknown,
       ROUND(100.0*x.cancelled/NULLIF(x.accepted,0),2) AS confirmed_cancel_pct,
       COALESCE(x.net_cash_vnd,0) AS cohort_net_cash_vnd
FROM days d LEFT JOIN daily x USING(day)
ORDER BY d.day;
```

JOIN hai view ở đây hợp lệ vì mỗi view có đúng một dòng/trip_id trong cohort. Nếu thay `da_cash` bằng payment thô, bạn có thể nhân mẫu số. Date spine giữ bảy ngày nếu dữ liệu một ngày rỗng; tỷ lệ ngày rỗng phải NULL, không phải 0%.

| Ngày nhận | Được nhận | Huỷ xác nhận | Chưa biết | Tỷ lệ xác nhận | Tiền ròng cohort đến snapshot |
|---|---:|---:|---:|---:|---:|
| 05/10 | 2 | 1 | 0 | 50% | 100.000 |
| 06/10 | 1 | 0 | 0 | 0% | 90.000 |
| 07/10 | 1 | 1 | 0 | 100% | 0 |
| 08/10 | 1 | 0 | 1 | 0% tạm tính | 0 |
| 09/10 | 1 | 0 | 0 | 0% | 100.000 |
| 10/10 | 1 | 1 | 0 | 100% | 0 |
| 11/10 | 1 | 0 | 0 | 0% | 110.000 |

Giá trị 0% ngày 08 chỉ nói chưa có huỷ được xác nhận; vì còn một lượt chưa biết, không được diễn giải “ngày này không có huỷ cuối cùng”. Người đọc cần nhìn cột unknown và nhãn tạm tính cùng tỷ lệ.

## 4. Tổng tuần phải tính lại từ tử và mẫu

```sql
SELECT COUNT(*) AS accepted,
       COUNT(*) FILTER (WHERE outcome='cancelled_before_pickup') AS cancelled,
       COUNT(*) FILTER (WHERE outcome='unknown') AS unknown,
       ROUND(100.0*COUNT(*) FILTER (WHERE outcome='cancelled_before_pickup')
          /NULLIF(COUNT(*),0),2) AS confirmed_cancel_pct
FROM da_classified;
```

Kết quả **8 / 3 / 1 / 37,50%**. Không lấy trung bình bảy tỷ lệ ngày. Các ngày có mẫu số khác nhau, và một ngày còn unknown. Giới hạn theo kết quả chưa biết là 37,5–50% trong giả định fixture.

Tiền phải đối soát riêng: tổng giá 730.000; giá bốn lượt hoàn thành 430.000; thu xác nhận 450.000; hoàn xác nhận 50.000; ròng 400.000. Ròng của chính cohort nhưng chỉ settled trong tuần là 320.000. Những số khác nhau này cần nhãn rõ, không ép bằng nhau.

## 5. Checklist trước bàn giao

| Nhóm kiểm tra | Kỳ vọng |
|---|---|
| Phạm vi | Đúng T01–T08; loại T09–T14 đúng lý do |
| Grain | Sau nối tiền vẫn tám dòng/tám trip_id |
| Trạng thái | 3 huỷ trước đón, 4 đã đón, 1 unknown |
| Lịch | Bảy ngày; timezone rõ; mốc cohort không đổi thành mốc huỷ |
| Thiếu dữ liệu | T04 thiếu lý do vẫn đếm huỷ; T05 không bị gắn thành công |
| Tiền | Không tính failed/pending thành thu; không trừ refund pending |
| Tái lập | Ghi snapshot, version, file SQL và cách chạy |
| Giới hạn | Chưa phải doanh thu hoặc kết quả tác động sản phẩm |

Một kiểm tra mạnh nên có cả ca thay đổi dữ liệu: thêm ticket không làm tăng tiền; lịch sử không đầy đủ khiến kết quả thành unknown; không có cohort thì tỷ lệ không xác định. Kiểm tra chỉ so tổng ở baseline có thể bỏ sót lỗi vừa mất vừa thêm một dòng.

## 6. Bài tập độc lập trước khi xem đáp án

Tạo bản truy vấn của bạn trong file riêng. Nộp: danh sách cohort, bảng ngày, tổng tuần, cầu nối giá–thu–hoàn, một danh sách ngoại lệ và nhận định tối đa 150 từ. Chạy với setup.sql mới để không phụ thuộc trạng thái thao tác trước đó.

Tiếp theo, giải thích ba lỗi giả lập: (a) tỷ lệ huỷ thành 3/7; (b) thu T01 thành 200.000; (c) báo cáo tiền trong tuần thành 400.000. Mỗi lỗi cần chỉ ra bước có thể gây sai và cách kiểm tra.

<details><summary>Gợi ý chẩn đoán</summary><p>(a) Có thể đã loại T05 unknown khỏi mẫu số. (b) Payment thành công có thể bị nhân bởi hai ticket T01. (c) Có thể đã lấy toàn bộ tiền cohort đến snapshot thay vì lọc settled_at trong tuần. Đây là các giả thuyết lỗi phù hợp fixture; vẫn cần đọc truy vấn để xác nhận.</p></details>

## 7. Bản nhận định mẫu

<details><summary>Mở sau khi tự viết</summary><p>Trong cohort tám lượt ride/city1 nhận tuần 05–11/10, snapshot 13/10 ghi nhận ba huỷ trước đón, bốn đã đón và một chưa rõ kết quả. Tỷ lệ huỷ xác nhận là 37,5%; chưa chốt tỷ lệ cuối vì T05 vẫn unresolved. Thu xác nhận của cohort là 450.000 VND, hoàn 50.000, tiền ròng 400.000 đến snapshot. Nếu chỉ tính settled trong tuần, ròng cohort là 320.000 do khoản thu T07 và hoàn R03 ở kỳ sau. Cần xác minh T05 trước khi chốt kết quả; không coi giá chuyến hay tiền ròng là doanh thu khi chưa có định nghĩa nghiệp vụ.</p></details>

Không đề xuất rollout chat hoặc chọn warehouse từ tám lượt học tập. Mục tiêu phần này là chuyển hợp đồng dữ liệu thành truy vấn đúng và có thể đối soát.

## 8. Rubric tự đánh giá

| Tiêu chí | Điểm |
|---|---:|
| Cohort và thời gian đúng, kiểm tra danh sách ID | 20 |
| Grain/join đúng, không nhân số tiền hoặc lượt | 20 |
| NULL, unknown, mẫu số 0 được xử lý đúng nghĩa | 20 |
| Đối soát tiền và giải thích chênh lệch kỳ | 20 |
| Tái lập, kiểm tra, nhận định và giới hạn | 20 |

Mục tiêu học tập từ 80/100, nhưng phải sửa mọi lỗi trọng yếu: đổi mẫu số âm thầm, tính tiền failed/pending, dùng sai cohort hoặc gọi net cash là doanh thu. Điểm tổng không bù cho một chỉ số cốt lõi sai nghĩa.

## 9. Kết thúc Phần 2

Bạn đã có bộ SQL từ lọc tới báo cáo, cùng các ca kiểm tra và số kỳ vọng. Phần 3 sẽ đào sâu thu thập, chất lượng và khám phá dữ liệu; các giả định “nguồn đầy đủ” trong fixture sẽ trở thành câu hỏi phải đo và kiểm chứng.

Hồ sơ thực hành: `examples/toi-noi-da/p02-sql/README.md`. Tham khảo lại [DA-014: JOIN](/articles/da-join-cardinality-dem-trung), [DA-016: thời gian](/articles/da-ngay-gio-timezone-cua-so-bao-cao), [DA-019: đối soát](/articles/da-tien-hoan-tien-doi-soat). Cú pháp tổng hợp tham khảo [PostgreSQL — Aggregate Functions](https://www.postgresql.org/docs/18/functions-aggregate.html).
