# P05 · Bảo mật, bảo trì và phục hồi (PG-041–050)

Lab vận hành **PostgreSQL thật**, đã chạy trên PostgreSQL 18.1 / Windows. Không dùng PGlite. Hai cluster tạm do runner tạo chỉ nghe localhost, không gắn vào service PostgreSQL sẵn có. Mọi backup, archive, key và password của lab đều tạm; output kiểm tra là artifact duy nhất được giữ.

## Chạy

Tại `examples/toi-noi-postgresql`:

```powershell
# Nếu binary không ở vị trí mặc định:
$env:PG_BIN = 'C:\Program Files\PostgreSQL\18\bin'
$env:OPENSSL_BIN = 'C:\Program Files\Git\usr\bin\openssl.exe'
npm run p05
```

Cần Node 22+, PostgreSQL 18 có initdb, pg_ctl, psql, pg_dump, pg_restore, pg_basebackup, extension pgstattuple, và OpenSSL hỗ trợ `req -addext`. Linux/macOS dùng PG_BIN của máy, OPENSSL_BIN mặc định `openssl`; runner sử dụng lại cấu hình locale của lab P03+. Đường Linux/macOS chưa được chạy trong đợt viết này. Version đầu ra là phiên bản thực đã kiểm, không phải khuyến nghị dùng bản vá cũ khi triển khai.

Không truyền URL của database khác cho script. Không có chế độ attach hoặc tham số bỏ qua assertion. Tài khoản postgres/replication dùng trust loopback để điều khiển cluster tạm: chạy trên máy phát triển cá nhân, không trên máy có người dùng không tin cậy. Đây không phải cấu hình xác thực production.

`run.mjs` lấy binary qua `pg/lib/bin.mjs`, tạo cluster bằng `pg/lib/cluster.mjs`, mở phiên đồng thời bằng `PsqlSession`. Khác mặc định P03/P04, P05 bật autovacuum; chỉ bảng churn tạm tắt trong thí nghiệm rồi reset. fsync, synchronous_commit và full_page_writes phải on. Không thay runner của phần trước.

## Dữ liệu và thứ tự

`00-schema.sql` tạo projection ops.trips: 100 chuyến, tenant 1 có 50, tenant 2 có 50, mỗi chuyến 30.000 VND. Đây là mô hình rút gọn vận hành, không thay schema nghiệp vụ P01–P04. Role tn_owner sở hữu; tn_app đọc/thêm/sửa; tn_report chỉ đọc; tn_monitor nhận pg_monitor.

| Bài | Phép kiểm trong run.mjs |
|---|---|
| 041 | Report SELECT được, DELETE bị chặn; app DDL bị chặn; app không CREATE được trong schema public; owner đúng; bảng mới có SELECT cho report nhờ default privileges nhưng **không** cho app |
| 042 | Verifier SCRAM; TLS verify-full (TLSv1.3); từ chối plaintext, sai password, sai hostname; verify-ca vẫn nhận hostname sai; log server cho biết dòng HBA khớp |
| 043 | Phiên psql thực (blocker, waiter, phiên lock_timeout, phiên idle): quan hệ blocker đúng trước khi kiểm Lock wait; pg_stat_activity/pg_locks; tn_monitor thấy query, tn_report thấy `<insufficient privilege>`; lock_timeout 55P03; idle_in_transaction_session_timeout kết thúc phiên |
| 044 | 5.000 tuple, xoá một nửa; pgstattuple thấy 2.500 chết → vacuum → 0, kích thước file giữ nguyên; snapshot REPEATABLE READ đang mở giữ 500 tuple chết đến khi COMMIT; reset tuỳ chọn bảng |
| 045 | Ba lượt UPDATE 100 dòng + CHECKPOINT; WAL bytes và thời gian; pg_stat_checkpointer đếm +3; full-page image: cùng UPDATE ngay sau checkpoint ghi nhiều WAL hơn lần lặp lại; trả fare về baseline |
| 046 | Dump ops sang custom archive; TOC không chứa role, globals qua pg_dumpall; restore database riêng: digest, owner, ACL, quyền ghi bị cấm; restore `--no-owner --no-acl` cùng digest nhưng mất owner/ACL |
| 047 | 6 job / 2 kết nối với service 50 ms; queue wait; statement timeout trả 57014; CONNECTION LIMIT 1 của role từ chối kết nối thứ hai |
| 049 | Thiết lập RLS trước physical backup: thiếu tenant, tenant 1, INSERT/UPDATE sai tenant, UPDATE dòng tenant khác đổi 0 dòng, phạm vi SET LOCAL, report không có policy, owner, FORCE, superuser |
| 048 | Base backup (backup_label) rồi thêm chuyến 101; restore point; DELETE; đợi archive (pg_stat_archiver); copy sang cluster riêng và replay tới pause; đích đang pause từ chối ghi |
| 050 | Khớp 101 chuyến / 3.045.000 VND và tenant 1 có 51; kết thúc recovery sang timeline 2 (file 00000002.history), INSERT chuyến 102 |

`OUTPUT.txt` có hai loại dòng: `PASS`/`MEASURE` là cổng kiểm và số quan sát; khối `--- 0xx …` là output psql hoặc trích log server thật mà các bài dẫn lại. Port, PID, thời điểm và mật khẩu được che; LSN và tên file WAL là của lần chạy đó, lần chạy khác có thể khác.

049 chạy trước 048 để physical backup bao gồm policy. Mỗi assertion thất bại làm exit khác 0 và không ghi đè OUTPUT.txt. Dữ liệu và ACL của logical restore được kiểm trước RLS để không lẫn hai loại quyền. Roles có sẵn vì đích logic cùng cluster; bài không tuyên bố đã thử logical restore sang cluster mới có role mapping khác.

## Phương pháp đo

`OUTPUT.txt` là output thực lần thành công, gồm phiên bản Node/PostgreSQL, hệ điều hành và các giá trị quan sát. Bảng ops.trips 100 dòng nằm trong cache ấm, shared_buffers 128 MB, work_mem 4 MB; các cấu hình khác kế thừa LAB_SETTINGS trừ override trong runner. Không tải ứng dụng ngoài lab, không cố xoá cache OS.

- PG-045: 3 lượt update/checkpoint, mỗi lời gọi có chi phí mở psql/kết nối. WAL diff ở cấp cluster có thể chứa công việc nền. Không có nhóm đối chứng để định lượng tác động checkpoint, không kết luận tuning hay throughput production.
- PG-047: mô hình queue tối giản, hai kết nối đã mở; 6 job cùng enqueue. Thời gian là lúc worker nhận job trừ enqueue, không bao gồm query. Không chạy PgBouncer; chưa có queue cancellation/health check production.
- PG-048: chỉ từ start đích tới pause đã đối soát, không gồm copy base backup, phát hiện sự cố hoặc routing. Đây không phải RTO đầu cuối. Target có tên xác định trước DELETE, không phải phép thử recovery_target_time.

Không assert số millisecond. Các bài tập mở rộng ở cuối từng bài là công việc của người đọc, chưa được gọi là phép đo đã chạy. Phép thử này không mô phỏng crash điện, disk corruption, failover, offsite retention hoặc tải lớn.

## Phục hồi và dọn

Archive helper từ chối ghi đè file khác nội dung, trả nonzero khi source WAL thiếu. Đích lấy bản sao base backup và WAL archive của source trong cùng máy. Runner không overwrite source; data directory khởi tạo của đích được đổi tên trong chính root tạm rồi thay bằng bản sao backup. Dọn bằng finally và lifecycle của cluster helper, gồm cả trường hợp assertion thất bại.

Không commit file dump, WAL, chứng chỉ hoặc mật khẩu. Không dùng chuỗi archive của lab như cơ chế backup production. Xem [RUNBOOK.md](RUNBOOK.md) để biết các cổng nghiệm thu và cách xử lý lỗi.
