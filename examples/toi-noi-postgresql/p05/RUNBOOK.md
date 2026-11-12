# Runbook P05 · Tới Nơi

Phạm vi: diễn tập local bằng `npm run p05`. Mọi lệnh phá dữ liệu chỉ thuộc cluster tạm runner tạo. Production cần người phụ trách, quyền thực thi và target được xác nhận riêng.

## Trước khi chạy

1. Xác nhận binary PostgreSQL và OpenSSL theo README; đủ disk cho hai cluster, base backup và WAL archive.
2. Không sử dụng máy nhiều người dùng không tin cậy: control plane lab dùng trust loopback.
3. Lưu output cũ khi cần so sánh; runner chỉ thay OUTPUT.txt sau một lần hoàn toàn thành công.

## Khi kiểm tra thất bại

| Dấu hiệu | Kiểm bằng chứng | Hành động |
|---|---|---|
| Permission không như dự kiến | Role hiện tại, owner, membership, ACL | Sửa GRANT/default privileges; không cấp ALL để bỏ lỗi |
| TLS không lên | OpenSSL exit, cert SAN, HBA order, client verify-full | Sửa danh tính/trust; không hạ sslmode |
| Không thấy blocker | PID A/B và transaction A còn mở | Xác minh lịch gửi, không tăng sleep để che sai thứ tự |
| Dead tuple không như fixture | Phiên đọc cũ, autovacuum table override | Phân biệt chưa thể dọn với chưa chạy vacuum |
| Restore logic lệch | pg_restore exit, count/sum/digest, owner/ACL | Giữ đích riêng để chẩn đoán; chưa nghiệm thu |
| Không có archive | archive_command, pg_stat_archiver, file segment | Sửa nguyên nhân; không đánh dấu có backup chỉ vì có base |
| Recovery không đạt target | recovery signal, target, restore_command, chuỗi WAL | Không resume hoặc đổi target âm thầm |
| Tenant đọc sai | current role/context, policy, FORCE/BYPASSRLS | Sửa mô hình quyền và context; kiểm lại cả INSERT |

## Diễn tập xoá nhầm

1. Base backup hoàn thành; ghi digest sau khi thêm chuyến 101.
2. Tạo restore point `p05_before_delete`, sau đó DELETE trong source lab.
3. Switch WAL; đợi segment được archive. File backup đơn lẻ chưa đủ.
4. Dựng đích khác cổng; restore base và WAL; target action pause.
5. Đợi replay paused, kiểm **101 dòng / 3.045.000 VND / digest đúng**, tenant 1 thấy **51**.
6. Chỉ sau các cổng trên mới resume; đợi hết recovery, thử INSERT chuyến 102.
7. Ghi thời gian/phạm vi; dọn source và đích. Không cắt routing bên ngoài lab.

## Hồ sơ bàn giao production cần bổ sung

Ghi người phụ trách quyết định dừng/mở ghi, mục tiêu RPO/RTO, thời gian phát hiện/copy/replay/đối soát/routing và các giao dịch hợp lệ sau target cần bù. Chuẩn bị phương án dữ liệu khi quay lại sau khi đích đã nhận ghi mới. Thiết kế storage offsite, encryption, retention, role mapping, kiểm extension và phục hồi định kỳ; các mục này chưa được lab local chứng nhận.
