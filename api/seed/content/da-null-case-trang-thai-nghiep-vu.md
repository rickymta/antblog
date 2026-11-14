---
slug: da-null-case-trang-thai-nghiep-vu
title: "NULL, CASE và trạng thái nghiệp vụ trong báo cáo"
summary: "T04 thiếu lý do nhưng vẫn là lượt huỷ; T05 chưa có kết quả không thể coi là thành công. Thực hành NULL, CASE, COALESCE và phân loại kết quả đúng hợp đồng."
category: data-analysis
level: intermediate
tags: [data-analysis, da-015, sql, null, data-quality, toi-noi]
language: vi
---

Trong cohort, cả T04 và T05 đều không có picked_up_at. Nhưng T04 đã huỷ, còn T05 vẫn accepted ở snapshot. T01 không có cancel_reason vì không huỷ; T04 không có lý do vì nhãn bị thiếu. Cùng là NULL, ý nghĩa nghiệp vụ không giống nhau.

DA cần giữ khác biệt ấy trong truy vấn và báo cáo, thay vì biến mọi chỗ trống thành 0 hoặc “không”.

<div class="tkh-callout"><span class="tkh-callout-title">DA-015 · Phần 2/8 · Trung bình</span><p>Đọc trước: <a href="/articles/da-join-cardinality-dem-trung">DA-014</a>. Đầu ra: bảng phân loại kết quả và báo cáo số lượt chưa biết kết quả. Dùng snapshot P02-LAB; history_complete là cờ do fixture cung cấp, không suy ra tự động từ việc không thấy event.</p></div>

## 1. NULL không so bằng dấu bằng

```sql
SELECT trip_id,status,cancel_reason
FROM da_cohort
WHERE cancel_reason IS NULL
ORDER BY trip_id;
```

Kết quả có T01, T03, T04, T05, T06, T07. Nếu dùng `cancel_reason = NULL`, điều kiện không trở thành true như kỳ vọng. Với phép so sánh thông thường, NULL tạo giá trị unknown; WHERE chỉ giữ những dòng có điều kiện cho kết quả true.

Tương tự, `cancel_reason <> 'changed_plan'` không tự giữ các NULL. Nếu muốn cả thiếu nhãn, phải nói rõ bằng `OR cancel_reason IS NULL`, hoặc chọn toán tử phù hợp với nghĩa cần đo.

## 2. Phân biệt chưa biết và không áp dụng

```sql
SELECT trip_id,
       CASE
         WHEN status<>'cancelled' THEN 'not_applicable'
         WHEN cancel_reason IS NULL THEN 'missing_reason'
         ELSE cancel_reason
       END AS reason_label
FROM da_cohort ORDER BY trip_id;
```

T02/T08 có changed_plan, T04 missing_reason, các lượt còn lại not_applicable ở snapshot. Nhãn `not_applicable` không có nghĩa T05 sẽ không huỷ sau này; nhãn này chỉ cho biết ở trạng thái hiện tại không có lý do huỷ nào để phân loại.

Cột này được tạo có chủ đích để hiển thị. Dữ liệu gốc vẫn giữ NULL để không đánh mất khả năng phân biệt nghĩa khi câu hỏi thay đổi.

## 3. CASE phải theo bằng chứng và thứ tự ưu tiên

```sql
SELECT trip_id,
  CASE
    WHEN NOT history_complete THEN 'unknown'
    WHEN status='cancelled' AND cancelled_at IS NOT NULL
         AND picked_up_at IS NULL THEN 'cancelled_before_pickup'
    WHEN picked_up_at IS NOT NULL THEN 'picked_up'
    ELSE 'unknown'
  END AS outcome
FROM da_cohort ORDER BY trip_id;
```

CASE chọn nhánh true đầu tiên. Phép kiểm tra độ đầy đủ lịch sử được đặt ở nhánh đầu, để không gọi một lượt là huỷ trước khi đón chỉ vì lịch sử đón bị thiếu. Trong fixture, kết quả là ba cancelled_before_pickup, bốn picked_up và một unknown (T05).

Một cờ history_complete không tự động đáng tin: nguồn phải có cách chứng minh và duy trì ý nghĩa của cờ này. Ở lab ta cấp cờ để luyện SQL; ở hệ thống thật cần hợp đồng nguồn, kiểm tra trạng thái và quy trình xử lý mâu thuẫn. Các trạng thái ngoài fixture phải được thiết kế riêng, không cứ rơi vào ELSE là xong công việc dữ liệu.

## 4. Giữ unknown trong mẫu số và giải thích kết quả

```sql
SELECT outcome, COUNT(*) AS trips
FROM da_classified GROUP BY outcome ORDER BY outcome;
```

T05 vẫn thuộc tám lượt đã nhận. Tỷ lệ huỷ xác nhận là 3/8 = 37,5%. Nếu T05 cuối cùng huỷ trước khi đón, tỷ lệ thành 4/8 = 50%; nếu không thì vẫn là 37,5%. Khoảng này chỉ xét một kết quả chưa biết và các giả định của fixture, không phải khoảng tin cậy.

Không loại T05 để báo 3/7 như tỷ lệ cohort, không dùng `COALESCE(outcome,'success')`. Một bản ghi chờ lâu ở snapshot có thể là vấn đề cần kiểm tra, không phải lý do để bỏ dòng mà không nói rõ.

## 5. COALESCE hữu ích khi đã rõ nghĩa số 0

View tiền dùng COALESCE để biến “không có khoản nào đã thu trong bản export đầy đủ” thành 0. Nhưng nếu không biết bản export có đầy đủ hay không, điền 0 sẽ tạo một kết luận nghiệp vụ không có căn cứ.

```sql
SELECT trip_id,captured_vnd,refunded_vnd,net_cash_vnd
FROM da_cash WHERE trip_id IN ('T02','T04','T05')
ORDER BY trip_id;
```

T02 đã thu 20.000 và hoàn 20.000, net=0. T04 có một lần thử (attempt) ở trạng thái pending nhưng chưa có tiền thu thành công, net=0. T05 chưa có lần thử nào, net=0. Cùng net 0 nhưng lịch sử khác nhau; nếu cần phân tích vận hành thanh toán, phải giữ cả trạng thái và số lần thử, chứ không chỉ net.

## 6. Kiểm tra giá trị thiếu đúng mẫu số

Trong ba lượt cancelled, hai lượt có lý do huỷ. Tỷ lệ thiếu nhãn trong nhóm huỷ là **1/3 ≈ 33,33%**, không phải 1/8 nếu câu hỏi là chất lượng nhãn huỷ.

```sql
SELECT COUNT(*) AS cancelled,
       COUNT(*) FILTER (WHERE cancel_reason IS NULL) AS missing_reason,
       ROUND(100.0*COUNT(*) FILTER (WHERE cancel_reason IS NULL)
         /NULLIF(COUNT(*),0),2) AS missing_reason_pct
FROM da_cohort WHERE status='cancelled';
```

Thiếu lý do không làm mất bằng chứng rằng T04 đã huỷ trong fixture. Việc thiếu nhãn giới hạn phân tích lý do huỷ, nhưng không nhất thiết phải loại lượt đó khỏi chỉ số huỷ.

## 7. Bài tập và tự đánh giá

Rà soát quy tắc “nếu picked_up_at NULL thì huỷ, còn lại thành công”. Nêu ít nhất hai trường hợp sai và cách sửa.

<details><summary>Gợi ý trả lời</summary><p>T05 chưa có kết quả nhưng không có mốc đón (pickup); một nguồn thiếu lịch sử cũng có thể không có mốc đón dù thực tế từng đón. Cần kiểm tra độ đầy đủ, trạng thái và mốc huỷ; giữ unknown khi chưa đủ. Có mốc đón cũng không đồng nghĩa thanh toán thành công hoặc toàn hành trình hoàn thành.</p></details>

**Câu hỏi:** Tại sao không điền cancel_reason='other' cho T04?

<details><summary>Gợi ý trả lời</summary><p>Nhãn other có thể mang nghĩa là người dùng đã chọn một lý do khác. NULL hiện tại chỉ cho biết thiếu giá trị. Gộp hai nhóm này sẽ làm mất khả năng đo chất lượng ghi nhận và dễ diễn giải sai cơ cấu lý do.</p></details>

Tham khảo: [PostgreSQL — Comparison Functions](https://www.postgresql.org/docs/18/functions-comparison.html), [Conditional Expressions](https://www.postgresql.org/docs/18/functions-conditional.html). Bài tiếp: [DA-016: thời gian và cửa sổ báo cáo](/articles/da-ngay-gio-timezone-cua-so-bao-cao).
