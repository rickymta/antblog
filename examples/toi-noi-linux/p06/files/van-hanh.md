# Runbook vận hành toinoi-app-01

Chủ: nhóm vận hành Tới Nơi. Cập nhật lần cuối: 24/11/2026 (An).

## Vào server
- SSH Studio, workspace "Cá nhân", thư mục Production: toinoi-app-01 (key an-laptop + OTP).
- toinoi-app-02 chỉ vào qua jump host toinoi-bastion.
- Không vào được: xem mục "Không SSH được".

## Kiểm tra hằng ngày
- Chạy `kiem-tra-van-hanh`. Mã thoát 0 là ổn; mỗi dòng LOI là một việc phải xử lý.

## bao-gia không trả lời
1. `systemctl status bao-gia` và `journalctl -u bao-gia -n 20`.
2. Đang dừng: `sudo systemctl start bao-gia`, chạy lại `kiem-tra-van-hanh`.
3. Vừa triển khai: `trien-khai <bản trước>` (xem `ls /srv/toinoi/bao-gia/releases`).

## Đĩa hoặc inode trên 80%
1. `df -h` và `df -i` để biết ổ nào, chỗ hay inode.
2. `sudo du -xh --max-depth=1 <ổ> | sort -h`; inode: đếm file theo thư mục.
3. Không xoá file đang mở (`sudo lsof +L1`); bản sao lưu giữ 3 bản mới nhất.

## Không SSH được
1. Đọc thông báo của SSH Studio: từ chối kết nối, hết thời gian, host key đổi hay xác thực thất bại.
2. Từ bastion: `nc -zv -w 3 <máy> 22` để tách lỗi mạng/tường lửa với lỗi sshd.
3. Console của nhà cung cấp: `systemctl status ssh`, `journalctl -u ssh`, `ufw status`.
4. Host key đổi: chỉ tin khi fingerprint khớp với console.

## Cập nhật và khởi động lại
- Bản vá bảo mật tự cài hằng ngày (unattended-upgrades), trừ nginx.
- `/var/run/reboot-required` có mặt: khởi động lại trong khung 22:00–23:00, báo trước kênh vận hành.
