---
slug: postgresql-wal-checkpoint-durability
title: "WAL, checkpoint và độ bền dữ liệu: COMMIT đã hứa điều gì?"
summary: "Kiểm cấu hình độ bền và đo WAL cùng checkpoint trên fixture nhỏ; ghi rõ phạm vi số đo trước khi quyết định tuning."
category: postgresql
level: intermediate
tags: [postgresql, pg-045, wal, checkpoint, toi-noi]
language: vi
---

API đã báo thanh toán thành công rồi máy database phải khởi động lại. Người trực cần hiểu COMMIT đã bảo đảm điều gì, và vì sao một checkpoint không thay thế backup. Bài này nối giao dịch phần 4 với đường ghi xuống storage.

<div class="tkh-callout"><span class="tkh-callout-title">PG-045 · Phần 5/6 · Trung bình</span><p>Đọc trước: <a href="/articles/postgresql-vacuum-autovacuum-bloat">bảo trì phiên bản dữ liệu</a>. Lab: <code>examples/toi-noi-postgresql/p05</code>, PostgreSQL 18.1; chạy <code>npm run p05</code>. Số đo thực ở <code>OUTPUT.txt</code>, phương pháp ở README.</p></div>

## 1. Không tối ưu bằng cách bỏ qua lời hứa

```sql
SELECT current_setting('fsync'),
       current_setting('synchronous_commit'),
       current_setting('full_page_writes');
```

Lab yêu cầu cả ba là on. Synchronous commit off có thể làm mất giao dịch vừa báo thành công khi crash; fsync off có nguy cơ nghiêm trọng hơn với tính nhất quán sau sự cố. Một benchmark không xảy ra crash không thể chứng minh các đánh đổi đó miễn phí. [Cấu hình WAL](https://www.postgresql.org/docs/18/runtime-config-wal.html).

Với thanh toán, xuất phát điểm phải là mức độ bền đã hứa với người dùng. Nếu chấp nhận mất dữ liệu ở một workload khác, quyết định đó cần được ghi rõ theo workload, không áp đại trà từ một mẹo tăng TPS.

## 2. WAL và checkpoint khác backup

WAL chứa thông tin phục vụ phục hồi; thay đổi tương ứng cần được ghi log bền vững trước khi data page được ghi theo cơ chế write-ahead. Sau crash, log giúp replay phần công việc cần thiết. Checkpoint tạo mốc phục hồi và ghi các trang bẩn theo cơ chế của server, không tạo bản sao độc lập để mang sang máy khác. [WAL trong phục hồi](https://www.postgresql.org/docs/18/continuous-archiving.html).

Một chuyến có thể đổi trạng thái nhiều lần trên cùng page. Lượng WAL không chỉ bằng kích thước cột bị sửa: index và full-page images cũng có vai trò. Muốn dự toán storage/archive phải đo workload, không lấy số byte của fare nhân số UPDATE.

## 3. Phép đo nhỏ đã chạy

Mỗi lượt runner lưu LSN, cập nhật 100 chuyến, tính chênh lệch WAL rồi chạy checkpoint:

```sql
SELECT pg_current_wal_insert_lsn(); -- lưu mốc trước
UPDATE ops.trips SET fare = fare + 1;
-- Thay placeholder bằng giá trị đã lưu:
SELECT pg_wal_lsn_diff(pg_current_wal_insert_lsn(), '<LSN_TRUOC>');
CHECKPOINT;
```

Runner thay LSN bằng giá trị thật. Nó lặp ba lần, ghi updateMs, checkpointMs, walBytes rồi đặt fare về 30.000 đ để phần backup có baseline rõ ràng. Thời gian tính ở Node bao gồm khởi động psql và kết nối; không phải latency thuần của SQL.

Fixture nhỏ, cache ấm, không tải đồng thời và không nhóm đối chứng “không checkpoint”. Do đó không dùng kết quả để khẳng định checkpoint làm production chậm bao nhiêu hay chọn checkpoint_timeout tối ưu. Những con số trong OUTPUT là quan sát của một lần thực nghiệm, không phải kỳ vọng cố định để assert.

## 4. Muốn quyết định tuning thì đo thêm gì

Giữ cùng dữ liệu, phân bố cập nhật, số client và storage. Chạy đủ lâu để thấy nhiều chu kỳ checkpoint, thu throughput, percentile latency, WAL rate, I/O và thời gian checkpoint. Đổi cấu hình có giả thuyết cụ thể rồi lặp lại. Chỉ so trung bình của hai lần chạy ngắn rất dễ nhầm ảnh hưởng cache và tải nền thành hiệu quả tuning.

Theo dõi cả khả năng archive/replica tiêu thụ WAL. max_wal_size không phải giới hạn cứng cho mọi tình huống giữ WAL; lỗi archive và replication slot cần được chẩn đoán riêng. Checkpoint thành công không chứng minh các bản WAL đã tới kho backup.

## 5. Bài tập

Dùng bản sao lab lớn hơn, chạy cùng workload trước và sau một checkpoint. Ghi cấu hình, số dòng, cache, số lượt, WAL bytes và thời gian. Nêu giả thuyết cho chênh lệch rồi kiểm nó; nếu chưa có dữ kiện, không quy mọi khác biệt cho checkpoint. Sản phẩm là một báo cáo có giới hạn kết luận rõ ràng.

Đọc tiếp: [Dump và restore](/articles/postgresql-pg-dump-restore).
