---
slug: da-tien-hoan-tien-doi-soat
title: "Tiền, hoàn tiền và đối soát: vì sao tổng thu khác tổng giá chuyến?"
summary: "Đối chiếu giá chuyến, khoản thu thành công, hoàn tiền và tiền ròng trong cohort Tới Nơi. Giải thích chênh lệch theo trạng thái và biên kỳ, không gắn nhãn doanh thu cho dữ liệu chưa đủ."
category: data-analysis
level: advanced
tags: [data-analysis, da-019, sql, reconciliation, payments, toi-noi]
language: vi
---

Báo cáo ghi tổng giá 730.000, người phụ trách thanh toán thấy thu 450.000, còn số sau hoàn là 400.000. Cả ba có thể đúng với ba định nghĩa khác nhau. Nhiệm vụ DA là giải thích từng đại lượng và lập cầu nối chênh lệch, không chọn một số rồi gọi hai số còn lại là lỗi.

<div class="tkh-callout"><span class="tkh-callout-title">DA-019 · Phần 2/8 · Nâng cao</span><p>Đọc trước: <a href="/articles/da-window-function-xep-hang-luy-ke">DA-018</a>, đặc biệt JOIN ở DA-014 và thời gian ở DA-016. Đầu ra: bảng đối soát tiền theo cohort và kỳ phát sinh. Đây là bài dữ liệu giả lập, không phải chính sách kế toán hay phí huỷ của Tới Nơi.</p></div>

## 1. Chốt ý nghĩa của tiền trong fixture

`fare_vnd` là giá ghi trên lượt. Payment succeeded cùng settled_at là khoản thu đã được xác nhận theo hợp đồng lab; failed/pending chưa được cộng vào tiền thu. Refund succeeded là khoản hoàn xác nhận, pending chưa trừ khỏi tiền ròng đã thực hiện.

T02 có khoản thu giả lập 20.000 rồi hoàn đủ. Đây là tình huống học tập, không khẳng định đó là phí huỷ hoặc một quy tắc thu cọc đang có trong ứng dụng. T03 hoàn 30.000; T07 có yêu cầu hoàn pending 10.000.

Mọi amount là số nguyên VND, dùng bigint cho dữ liệu và phép tổng hợp phù hợp. Không dùng kiểu số thực nhị phân để lưu đơn vị tiền chính xác. Nếu có nhiều tiền tệ, không cộng thẳng trước khi định nghĩa đồng tiền, tỷ giá và thời điểm quy đổi.

## 2. Bảng đối soát một dòng/lượt

```sql
SELECT trip_id,fare_vnd,captured_vnd,refunded_vnd,net_cash_vnd
FROM da_cash ORDER BY trip_id;
```

| Lượt | Giá | Thu xác nhận | Hoàn xác nhận | Ròng |
|---|---:|---:|---:|---:|
| T01 | 100.000 | 100.000 | 0 | 100.000 |
| T02 | 80.000 | 20.000 | 20.000 | 0 |
| T03 | 120.000 | 120.000 | 30.000 | 90.000 |
| T04 | 90.000 | 0 | 0 | 0 |
| T05 | 70.000 | 0 | 0 | 0 |
| T06 | 100.000 | 100.000 | 0 | 100.000 |
| T07 | 110.000 | 110.000 | 0 | 110.000 |
| T08 | 60.000 | 0 | 0 | 0 |
| Tổng | 730.000 | 450.000 | 50.000 | 400.000 |

View tổng hợp payment và refund riêng rồi nối về trip_id. T01 có attempt thất bại vẫn chỉ thu 100.000; R07 pending chưa trừ. Không SUM giá chuyến sau khi nối nhiều payment/refund trực tiếp.

## 3. Lập cầu nối chênh lệch

Tổng giá của **bốn lượt hoàn thành** là 430.000. Thêm khoản thu giả lập của T02 là 20.000, được gross captured 450.000. Trừ 20.000 hoàn của T02 và 30.000 của T03, còn net 400.000.

```sql
SELECT SUM(fare_vnd) AS all_quoted,
       SUM(fare_vnd) FILTER (WHERE status='completed') AS completed_quoted,
       SUM(captured_vnd) AS captured,
       SUM(refunded_vnd) AS refunded,
       SUM(net_cash_vnd) AS net_cash
FROM da_cash;
```

Cầu nối này dựa trên fixture cụ thể: mỗi lượt hoàn thành ở đây thu đúng giá. Trong hệ thống thật có khuyến mãi, trợ giá, nhiều phương thức và nhiều bên, không thể mặc định đẳng thức đó. Cần nguồn và quy tắc để giải thích từng nhóm chênh lệch.

Tổng giá 730.000 còn gồm lượt huỷ/chờ, nên không dùng như khoản phải thu hoặc thiệt hại. Net cash 400.000 cũng chưa phải doanh thu: còn thiếu hợp đồng về phân chia, phí, thuế, thời điểm ghi nhận và các thành phần liên quan. Bài chỉ đo luồng tiền theo định nghĩa lab.

## 4. Tiền của cohort khác tiền phát sinh trong tuần

```sql
WITH flows AS (
  SELECT p.settled_at,p.amount_vnd AS signed_vnd
  FROM da_payments p JOIN da_cohort c USING(trip_id)
  WHERE p.status='succeeded'
  UNION ALL
  SELECT r.settled_at,-r.amount_vnd
  FROM da_refunds r JOIN da_payments p USING(payment_id)
  JOIN da_cohort c USING(trip_id)
  WHERE r.status='succeeded'
)
SELECT SUM(signed_vnd) AS cohort_cash_settled_in_week
FROM flows
WHERE settled_at >= TIMESTAMPTZ '2026-10-05 00:00+07'
  AND settled_at < TIMESTAMPTZ '2026-10-12 00:00+07';
```

Kết quả **320.000**: gross trong tuần 340.000, hoàn trong tuần 20.000. T07 thu 110.000 tuần sau, R03 hoàn 30.000 tuần sau; cộng 320.000 + 110.000 − 30.000 = 400.000 đến snapshot.

Nếu hỏi mọi dòng tiền ride/city1 phát sinh trong tuần, phải thêm T10 thu 40.000 dù T10 thuộc cohort trước: kết quả là 360.000 trong fixture. Không tự cộng T10 vào cohort chính để làm hai báo cáo khớp.

## 5. Refund phải đối chiếu với khoản thu gốc

```sql
SELECT p.payment_id,p.amount_vnd AS captured,
       SUM(r.amount_vnd) AS refunded
FROM da_payments p JOIN da_refunds r USING(payment_id)
WHERE p.status='succeeded' AND r.status='succeeded'
GROUP BY p.payment_id,p.amount_vnd
HAVING SUM(r.amount_vnd)>p.amount_vnd;
```

Fixture trả 0 dòng. Kiểm tra này phát hiện tổng hoàn xác nhận vượt khoản thu, nhưng chưa phát hiện mọi vấn đề: còn cần đảm bảo refund không tham chiếu payment thất bại, không trùng giao dịch, đúng currency và đúng snapshot.

Một pending refund có ý nghĩa vận hành dù chưa là dòng tiền đã thực hiện. Báo cáo nên có thể tách tiền đã hoàn và nghĩa vụ/yêu cầu đang xử lý theo định nghĩa riêng, không trộn hai nhóm.

## 6. Khi số không khớp

Đi theo thứ tự: phạm vi đối tượng → grain → trạng thái → đơn vị tiền → mốc thời gian → độ đầy đủ → trùng/thiếu → quy tắc nghiệp vụ. Mỗi chênh lệch cần danh sách ID và số tiền, không chỉ một tổng “unmatched”.

Ví dụ, nếu net bị giảm thêm 10.000, kiểm tra có trừ R07 pending không. Nếu captured của T01 bị gấp đôi, kiểm tra join ticket. Nếu net tuần bằng 400.000, kiểm tra đã lấy toàn snapshot thay vì lọc settled_at trong tuần chưa.

Không sửa trực tiếp số nguồn để khớp dashboard. Lưu phiên bản truy vấn, bằng chứng đối chiếu và cách điều chỉnh nếu phát hiện lỗi.

## 7. Bài tập và tự kiểm tra

Giả sử R07 chuyển succeeded 10.000 sau snapshot, nhưng muốn giữ báo cáo chốt 13/10 như cũ. Có được chạy lại trên bảng trạng thái hiện tại rồi gọi là báo cáo cùng snapshot không?

<details><summary>Gợi ý trả lời</summary><p>Không nếu không có snapshot hoặc lịch sử đủ tái dựng. Cần nguồn có phiên bản/ngày biết kết quả phù hợp. Với snapshot mới gồm khoản hoàn, net cohort thành 390.000; đó là phiên bản khác, không được sửa nhãn cũ âm thầm.</p></details>

**Câu hỏi:** Vì sao SUM(DISTINCT amount_vnd) làm sai gross cohort?

<details><summary>Gợi ý trả lời</summary><p>T01 và T06 có hai khoản thu hợp lệ cùng 100.000. DISTINCT theo số tiền sẽ bỏ mất một khoản. Cần bảo vệ khoá giao dịch, tổng hợp đúng grain và trạng thái trước khi nối.</p></details>

Tham khảo kỹ thuật: [PostgreSQL — Numeric Types](https://www.postgresql.org/docs/18/datatype-numeric.html). Bài tiếp: [DA-020: báo cáo vận hành tuần](/articles/da-thuc-hanh-sql-bao-cao-van-hanh-tuan).
