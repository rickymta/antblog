---
slug: postgresql-pg-stat-activity-quan-sat
title: "Quan sát phiên làm việc và truy vấn chậm: tìm đúng người giữ khoá"
summary: "Chẩn đoán request chậm bằng pg_stat_activity và pg_blocking_pids; tái hiện backend bị khoá rồi theo dõi khi được giải phóng."
category: postgresql
level: beginner
tags: [postgresql, pg-043, monitoring, locks, toi-noi]
language: vi
---

Request hoàn tiền mất vài giây nhưng CPU database thấp. Tăng CPU có thể không giúp gì: request đang chờ giao dịch khác trả khoá. Bài này đưa thí nghiệm concurrency của phần 4 vào cách chẩn đoán khi trực vận hành Tới Nơi.

<div class="tkh-callout"><span class="tkh-callout-title">PG-043 · Phần 5/6 · Cơ bản</span><p>Đọc trước: <a href="/articles/postgresql-ket-noi-auth-tls">bảo vệ kết nối</a> và <a href="/articles/postgresql-lock-row-table">khoá dòng</a>. Lab: <code>examples/toi-noi-postgresql/p05</code>, chạy <code>npm run p05</code> trên PostgreSQL 18.1 thật.</p></div>

## 1. Trạng thái và sự kiện chờ

`pg_stat_activity` cho biết backend đang làm gì. `state` và `wait_event_type` trả lời hai câu khác nhau: có đang chạy câu lệnh không và đang chờ tài nguyên gì. Active vẫn có thể chờ Lock; với idle, query có thể là câu trước đó chứ không phải câu đang chạy. [Tài liệu thống kê hoạt động](https://www.postgresql.org/docs/18/monitoring-stats.html).

```sql
SELECT pid, application_name, state,
       clock_timestamp() - xact_start AS transaction_age,
       clock_timestamp() - query_start AS query_age,
       wait_event_type, wait_event,
       pg_blocking_pids(pid) AS blockers,
       left(query, 160) AS query_sample
FROM pg_stat_activity
WHERE datname = current_database()
  AND pid <> pg_backend_pid()
ORDER BY xact_start NULLS LAST;
```

Lab tạo tn_monitor và cấp pg_monitor. Triển khai cần xét ai được xem query text vì có thể chứa dữ liệu nhạy cảm. Không đưa SQL nguyên văn lên dashboard công khai.

## 2. Lịch chạy có bằng chứng

| Bước | Phiên A | Phiên B | Quan sát |
|---|---|---|---|
| 1 | BEGIN; UPDATE id=1 | Chưa chạy | A giữ khoá |
| 2 | Chưa kết thúc | UPDATE id=1 | B chờ |
| 3 | Vẫn giữ giao dịch | Vẫn chờ | blocker của B chứa PID A |
| 4 | ROLLBACK | UPDATE hoàn thành | Hết chặn |

```sql
BEGIN;
UPDATE ops.trips SET fare = fare WHERE id = 1;
-- Giữ giao dịch mở cho tới khi người quan sát kiểm xong.
```

Giá không đổi nhưng UPDATE vẫn lấy khoá. Runner mở hai tiến trình psql, gửi UPDATE của B bất đồng bộ và đợi catalog xác nhận quan hệ blocker trước khi đọc wait_event_type. Nó không sleep một khoảng cố định rồi đoán rằng B đã bị chặn. Assertion ghi `043 identifies lock wait: Lock`, sau đó A rollback, B hoàn thành và hai phiên đóng.

## 3. Từ PID tới nguyên nhân

Nối PID với application_name, request/trace ID và người sở hữu luồng nghiệp vụ. Nếu A giữ transaction trong lúc gọi HTTP ra ngoài, cần rút ngắn transaction; tăng timeout B chỉ che triệu chứng. Nếu A chạy migration cần thiết, có thể phải điều phối cửa sổ triển khai.

Cancel một câu lệnh và terminate một phiên có tác động khác nhau. Ứng dụng phải xử lý lỗi, rollback và retry theo tính chất nghiệp vụ. Không tự động kill mọi phiên query_age lớn: một phiên báo cáo lâu có thể hợp lệ, còn transaction rảnh nhưng giữ khoá mới là vấn đề.

Khi không có Lock wait, kiểm loại chờ và dữ liệu khác trước khi kết luận CPU hay I/O. Một snapshot không biểu diễn phân bố độ trễ cả ngày. Ghép activity với log, metric theo khoảng thời gian và execution plan ở phần 3; tránh dùng vài request đang active để đại diện toàn workload.

## 4. Bài tập runbook

Cho A giữ transaction sau UPDATE nhưng không gửi thêm SQL. Thu trạng thái A/B, blocker và tuổi giao dịch; kết thúc A rồi ghi lúc B chạy tiếp. Nộp runbook có dấu hiệu, truy vấn xác nhận, người liên hệ, biện pháp tạm thời và sửa lâu dài. Kết luận phải dựa trên quan hệ blocker, không chỉ chọn query_age lớn nhất.

Đọc tiếp: [VACUUM và autovacuum](/articles/postgresql-vacuum-autovacuum-bloat).
