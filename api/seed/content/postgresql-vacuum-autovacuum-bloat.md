---
slug: postgresql-vacuum-autovacuum-bloat
title: "VACUUM, autovacuum và bloat: dữ liệu không tăng mà file vẫn lớn"
summary: "Quan sát 2.500 dead tuples trước và 0 sau VACUUM trong lab; phân biệt không gian tái sử dụng, bloat và chính sách autovacuum."
category: postgresql
level: intermediate
tags: [postgresql, pg-044, vacuum, maintenance, toi-noi]
language: vi
---

Bảng chuyến liên tục đổi trạng thái nhưng tổng số chuyến giữ nguyên. Dung lượng tăng không nhất thiết là dữ liệu nghiệp vụ tăng: MVCC phải giữ phiên bản cũ cho các giao dịch còn đọc được nó. Người vận hành cần phân biệt tuple chết, chỗ trống tái sử dụng và dung lượng trả lại hệ điều hành.

<div class="tkh-callout"><span class="tkh-callout-title">PG-044 · Phần 5/6 · Trung bình</span><p>Đọc trước: <a href="/articles/postgresql-pg-stat-activity-quan-sat">quan sát giao dịch</a>. Lab PostgreSQL 18.1: <code>examples/toi-noi-postgresql/p05</code>; chạy <code>npm run p05</code>. Kết quả lưu trong <code>p05/OUTPUT.txt</code>.</p></div>

## 1. VACUUM làm gì

VACUUM thu hồi không gian của tuple không còn cần thiết để tái sử dụng, hỗ trợ visibility map và phòng ngừa cạn tuổi transaction ID. VACUUM thông thường không hứa thu nhỏ toàn file về kích thước tối thiểu. VACUUM FULL viết lại bảng, có yêu cầu khoá và dung lượng khác; nó không phải việc mặc định mỗi khi thấy bảng lớn. [Routine vacuuming](https://www.postgresql.org/docs/18/routine-vacuuming.html).

Với Tới Nơi, câu hỏi là khả năng dọn có theo kịp tốc độ đổi trạng thái không. Cần xem cả các giao dịch giữ snapshot cũ: vacuum chạy xong vẫn có thể chưa dọn được phiên bản mà giao dịch còn cần.

## 2. Thí nghiệm có điểm bắt đầu

Lab tạo bảng phụ 5.000 dòng. Chỉ bảng đó tạm tắt autovacuum để worker không dọn trước khi quan sát; toàn cluster phần 5 vẫn bật autovacuum. Cuối phép thử, tuỳ chọn bảng được reset.

```sql
CREATE EXTENSION pgstattuple;
CREATE TABLE ops.churn(id int, payload text)
  WITH (autovacuum_enabled=false);
INSERT INTO ops.churn
SELECT n, repeat(md5(n::text),10)
FROM generate_series(1,5000) n;
DELETE FROM ops.churn WHERE id % 2 = 0;
SELECT dead_tuple_count FROM pgstattuple('ops.churn');
VACUUM (ANALYZE) ops.churn;
SELECT dead_tuple_count FROM pgstattuple('ops.churn');
ALTER TABLE ops.churn RESET (autovacuum_enabled);
```

Các bước DELETE và VACUUM chạy riêng, không bọc chung trong transaction. pgstattuple dùng để kiểm dữ liệu nhỏ của lab; không biến phép quét bảng này thành truy vấn dashboard mỗi giây trên bảng lớn.

## 3. Kết quả và điều chưa kết luận

Trước vacuum có **2.500 dead tuples**, sau vacuum là **0**. Điều được chứng minh là phiên bản chết trong fixture được dọn. Không có bằng chứng file đã nhỏ đi một nửa; runner không suy ra phần trăm bloat của hệ thống thật từ phép thử này.

Quan sát vận hành thường xuyên có thể bắt đầu bằng:

```sql
SELECT relname, n_live_tup, n_dead_tup,
       last_autovacuum, last_autoanalyze, autovacuum_count
FROM pg_stat_user_tables
WHERE schemaname = 'ops';
```

Số tuple ở statistics là ước lượng và có độ trễ cập nhật. Ghép nó với tốc độ UPDATE/DELETE, tuổi transaction, kích thước file và tiến độ vacuum. Một số n_dead_tup cao ngay sau đợt xoá lớn chưa đủ kết luận worker hỏng.

## 4. Chính sách theo bảng

Bảng trạng thái chuyến có nhiều update khác bảng sự kiện append-only. Một ngưỡng tỷ lệ chung trên bảng rất lớn có thể để tích tụ nhiều tuple chết trước khi đạt điều kiện kích hoạt. Điều chỉnh theo bảng cần baseline: tốc độ tạo phiên bản chết, thời gian dọn, I/O budget và thời gian phản ứng mong muốn. Sau thay đổi, đo lại chính các chỉ số đó.

Tắt autovacuum toàn hệ thống để giảm tải tức thời sẽ chuyển công việc sang tương lai, không làm nhu cầu bảo trì biến mất. Trước khi tăng worker hay thay ngưỡng, dùng bài trước để tìm giao dịch dài đang cản việc dọn.

## 5. Bài tập hai phiên

Mở Repeatable Read và đọc bảng trước DELETE, giữ snapshot đó rồi chạy vacuum ở phiên khác. Đo trước/sau, kết thúc phiên đọc rồi vacuum lần nữa. Lời giải cần lịch hai phiên và các mốc quan sát, phân biệt “lệnh đã chạy nhưng chưa thể dọn” với “lệnh không thực thi”. Không ghi chung hai tình huống là VACUUM bị lỗi.

Đọc tiếp: [WAL và checkpoint](/articles/postgresql-wal-checkpoint-durability).
