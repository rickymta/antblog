---
slug: postgresql-connection-pool-timeout
title: "Connection pool, giới hạn kết nối và timeout: request đang chờ ở đâu?"
summary: "Sáu công việc dùng hai kết nối thật: đo hàng chờ riêng, kiểm statement_timeout và phân biệt application pool với transaction pooling."
category: postgresql
level: intermediate
tags: [postgresql, pg-047, pooling, timeout, toi-noi]
language: vi
---

Khuyến mãi tăng số request nhưng ngân sách kết nối database vẫn hữu hạn. Mỗi request mở connection mới sẽ đẩy chi phí và hàng chờ xuống PostgreSQL. Pool giới hạn số kết nối dùng lại; nó không biến server thành nơi xử lý vô hạn công việc.

<div class="tkh-callout"><span class="tkh-callout-title">PG-047 · Phần 5/6 · Trung bình</span><p>Đọc trước: <a href="/articles/postgresql-pg-dump-restore">backup logic</a> và <a href="/articles/postgresql-pg-stat-activity-quan-sat">quan sát hoạt động</a>. Lab PostgreSQL 18.1: <code>examples/toi-noi-postgresql/p05</code>; chạy <code>npm run p05</code>.</p></div>

## 1. Tách thời gian hàng chờ khỏi SQL

Request chờ lấy connection rồi mới chạy SQL. Statement timeout trên server không bao gồm thời gian request còn nằm trong queue ứng dụng. Vì vậy “database timeout” cần được phân loại theo vị trí, nếu không ta sẽ chỉnh sai tham số.

Ví dụ ngân sách tự đặt: 100 connection, giữ 20 cho quản trị và job, còn 80 cho API. Bốn replica dùng tối đa 20 mỗi replica là vừa ngân sách; autoscale thành tám replica mà giữ nguyên pool size sẽ thành 160. Đây là bài tính, không phải đề xuất max_connections của hệ thống bạn nên bằng 100.

## 2. Sáu công việc, hai worker

Runner mở hai PsqlSession thật và đưa sáu job vào hàng đợi. Mỗi job chạy `SELECT pg_sleep(0.05)`. Thời gian ghi ở lúc worker lấy job là queue wait:

```javascript
const enqueued = performance.now();
let next = 0;
await Promise.all(workers.map(async session => {
  while (next < 6) {
    const job = next++;
    waits[job] = performance.now() - enqueued;
    await session.run('SELECT pg_sleep(0.05);');
  }
}));
```

Đây là mô hình hàng đợi tối giản, không phải thư viện pool production: chưa có huỷ job, giới hạn queue hay thay connection hỏng. Đầu ra thường có ba nhóm: gần 0 ms, khoảng một lượt service và khoảng hai lượt service. Số thật được lưu trong OUTPUT, không assert millisecond cố định vì còn scheduling và tải máy.

## 3. Timeout kiểm bằng SQLSTATE

```sql
SET statement_timeout = '20ms';
SELECT pg_sleep(0.2);
```

Runner yêu cầu lỗi `57014`. Khi lỗi ở trong transaction, ứng dụng phải rollback rồi mới trả connection về pool. Mọi nhánh exception và cancellation cần release đúng một lần; rò connection sẽ làm pool cạn dù query không còn chạy.

Cần acquisition timeout, giới hạn queue và deadline tổng request bên cạnh statement timeout. Quá tải kéo dài nên từ chối có kiểm soát thay vì giữ request vô hạn. Retry vẫn phải nằm trong ngân sách và tuân theo idempotency ở phần 4.

## 4. Đừng đánh đồng các loại pool

Application pool giữ các kết nối lâu dài; PgBouncer có thể gán backend theo session hoặc transaction. Với transaction pooling, không giả định transaction kế tiếp vẫn ở backend cũ. Tính năng phụ thuộc session state phải đối chiếu chế độ và cấu hình, đặc biệt prepared statement không nên bị kết luận “luôn được” hay “luôn hỏng”. [Ma trận PgBouncer](https://www.pgbouncer.org/features.html).

Lab không chạy PgBouncer nên không gọi số đo trên là benchmark của sản phẩm đó. Khi triển khai thật, kiểm driver cùng SET/RESET, temp table, advisory lock, prepared statement và rotation credential.

Tenant context đặt ở session có thể sót cho request kế tiếp. PG-049 giới hạn nó bằng SET LOCAL trong transaction, đồng thời nhấn mạnh ứng dụng phải xác thực tenant trước khi đặt context.

## 5. Bài tập overload

Thêm queue timeout ngắn hơn lượt thứ ba. Chứng minh job hết hạn không chạy về sau, số connection vẫn là hai và connection được trả sau lỗi SQL. Nộp riêng queue wait, query duration và số job bị từ chối; một latency trung bình gộp tất cả không đủ giải thích nút thắt.

Đọc tiếp: [PITR bằng base backup và WAL](/articles/postgresql-pitr-base-backup-wal).
