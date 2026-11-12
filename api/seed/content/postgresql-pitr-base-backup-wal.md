---
slug: postgresql-pitr-base-backup-wal
title: "Khôi phục theo thời điểm với base backup và WAL: cứu dữ liệu trước DELETE nhầm"
summary: "Diễn tập archive recovery trên cluster riêng: giữ được chuyến phát sinh sau base backup, dừng trước DELETE và kiểm khả năng ghi sau recovery."
category: postgresql
level: advanced
tags: [postgresql, pg-048, pitr, recovery, toi-noi]
language: vi
---

Một lệnh xoá nhầm làm mất toàn bộ chuyến. Backup đêm qua cứu được phần cũ nhưng chưa đủ: chuyến mới sáng nay cũng cần giữ. PITR kết hợp base backup với WAL để chọn mốc phục hồi. Bài này chạy chuỗi đó trên hai cluster tạm riêng biệt.

<div class="tkh-callout"><span class="tkh-callout-title">PG-048 · Phần 5/6 · Nâng cao</span><p>Đọc trước: <a href="/articles/postgresql-pg-dump-restore">dump/restore</a> và <a href="/articles/postgresql-wal-checkpoint-durability">WAL</a>. Lab: <code>examples/toi-noi-postgresql/p05</code>, PostgreSQL 18.1; chạy <code>npm run p05</code>. Chỉ runner mới thực hiện các lệnh phá dữ liệu trong cluster vừa tạo; không chạy DELETE mẫu trên database hiện có.</p></div>

## 1. Chọn mốc có ý nghĩa

```text
base backup → thêm chuyến 101 → restore point → DELETE nhầm
                                  ↑ điểm muốn dừng
```

Lab dùng restore point có tên, một dạng chọn target của PITR, thay vì đoán timestamp. Nếu chỉ lấy trạng thái đầu backup, sẽ thiếu chuyến 101; nếu replay quá xa, sẽ nhận DELETE. Hai lỗi này có thể đều xảy ra dù server khởi động thành công.

Cần base backup hợp lệ và chuỗi WAL đủ tới mục tiêu. Mất segment hoặc archive chưa hoàn tất có thể khiến recovery không đạt target. [Continuous archiving và PITR](https://www.postgresql.org/docs/18/continuous-archiving.html).

## 2. Archive và base backup

Runner bật archive_mode và cấu hình archive_command gọi `wal-copy.mjs`. Nếu file archive đã có, helper chỉ chấp nhận khi nội dung giống; khác nội dung là lỗi, không ghi đè. Đây là kho archive trong thư mục lab, chưa có offsite, retention hay đảm bảo độ bền production.

```text
pg_basebackup -h 127.0.0.1 -p <SOURCE_PORT> -U postgres
              -D <BASE_DIR> -Fp -X stream --checkpoint=fast
```

Base backup plain kèm WAL stream phục vụ tính nhất quán của backup. Thay đổi sau khi backup hoàn tất cần archive để tới restore point. Runner không dùng -R vì không tạo streaming standby.

## 3. Tai nạn có kiểm soát

```sql
INSERT INTO ops.trips VALUES (101, 1, 45000, 'completed');
SELECT pg_create_restore_point('p05_before_delete');
DELETE FROM ops.trips;
SELECT pg_switch_wal();
```

Source có 0 chuyến sau DELETE. Runner đợi segment chứa thay đổi được archive trước khi dựng đích. Có restore point trong nguồn chưa đủ bảo đảm phục hồi nếu source mất mà WAL chưa tới kho backup.

## 4. Dựng đích riêng để kiểm

Runner tạo cluster thứ hai, dừng nó, cất data directory ban đầu trong cùng root tạm rồi copy base backup vào. Đích có cổng khác, archive_mode off và các setting:

```text
restore_command = '<lệnh lấy WAL từ archive về %p>'
recovery_target_name = 'p05_before_delete'
recovery_target_action = 'pause'
```

Tạo `recovery.signal`, khởi động rồi đợi `pg_is_wal_replay_paused()` là true. Lệnh restore thực, quoting đường dẫn và xử lý file thiếu nằm trong run.mjs; đoạn trên chỉ giải thích cấu hình.

Kết quả thật: **101 chuyến, 3.045.000 đ**, digest khớp trạng thái trước DELETE. Nó chứng minh replay đã vượt trạng thái base backup mà chưa áp xoá. Runner cũng kiểm policy của PG-049 còn hiệu lực: tenant 1 thấy 51 chuyến. Trong chương trình, bước dựng policy được làm trước backup để kiểm được tính bảo toàn này.

## 5. Nghiệm thu rồi mới ghi

Sau đối soát, runner gọi `pg_wal_replay_resume()`, đợi recovery kết thúc rồi INSERT chuyến 102. Đây là kiểm đích ghi được, chưa phải tự động cắt traffic production.

Số đo trong output chỉ từ khởi động đích đến kiểm xong trạng thái pause, không gồm phát hiện sự cố, chuẩn bị máy, copy backup và cập nhật routing. Không gọi số đó là RTO đầu cuối. Lab giữ đủ dữ liệu tới mốc đã chọn; nó không chứng minh RPO bằng 0 cho mọi sự cố hoặc giữ dữ liệu hợp lệ phát sinh sau target.

## 6. Bài tập mất WAL

Trên bản chạy riêng, làm thiếu một segment bắt buộc. Quan sát vì sao “server đã lên” khác “đã replay đúng target”. Nộp timeline, backup/target chọn, segment thiếu, cách phục hồi archive và đối soát sau đó. Không sửa target tới mốc sớm hơn chỉ để vượt kiểm tra mà không ghi nhận dữ liệu bị mất.

Đọc tiếp: [RLS và tenant](/articles/postgresql-rls-tenant-security).
