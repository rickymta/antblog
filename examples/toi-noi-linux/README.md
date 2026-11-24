# Lab: Linux qua SSH Studio và Tới Nơi

Server Linux để làm theo series [Linux qua SSH Studio](../../docs/linux-master-curriculum.md): một container **Ubuntu 24.04 LTS** chạy **systemd** và **sshd** như server thật, kèm dữ liệu mẫu của dịch vụ báo giá `bao-gia` của Tới Nơi.

Cần **Docker Desktop** (engine Linux) và **Node.js 22**. Runner không có dependency npm, không cần `npm install`.

## Dùng nhanh

```sh
cd examples/toi-noi-linux
npm run up        # dựng ảnh, khởi động server, in thông tin kết nối và fingerprint
npm run info      # in lại thông tin kết nối
npm run down      # xoá server
```

| Thông tin | Giá trị |
|---|---|
| Địa chỉ / cổng | `127.0.0.1` / `2222` (chỉ mở cho máy của bạn) |
| Người dùng | `an` (có sudo), `binh` (có sudo, thuộc nhóm `toinoi`) |
| Mật khẩu | `toinoi-lab` — chỉ dùng cho lab này |
| Tài khoản dịch vụ | `baogia` (không đăng nhập được), nhóm `toinoi` |
| Dịch vụ mẫu | `/opt/toinoi/bao-gia` · log `/var/log/toinoi/bao-gia.log` (362 dòng cố định, ngày 21/11) |
| Log thêm từ Phần 2 | `bao-gia.log.1` (20/11, 305 dòng) và `bao-gia.log.2.gz` (19/11, 302 dòng), do `chuan-bi.sh` đưa vào ở đầu LNX-011 |
| Thêm từ Phần 3 | `libpam-google-authenticator`, `oathtool`; key của lab, host key thay thế và bí mật TOTP cố định trong `/usr/share/toinoi-lab/p03` (chỉ root đọc được); key «trên laptop của An» ở `may-cua-ban/khoa-lab` |
| Thêm từ Phần 4 | `python3-minimal`, `logrotate`; `bao-gia` 1.5.0 (Python) và script mẫu trong `/usr/share/toinoi-lab/p04`, do `chuan-bi.sh` đưa vào ở đầu LNX-031. `jq`, `tmux` do LNX-033 cài bằng apt (server cần Internet) |
| Thêm từ Phần 5 | Server thứ hai `toinoi-app-02` (10.20.0.12) chỉ trong mạng nội bộ `10.20.0.0/24` của hai container, không mở cổng ra máy bạn; `netcat-openbsd`, `bind9-dnsutils`, `lsof`, `ufw`, `nftables`, `e2fsprogs`; key đồng bộ và `chuan-bi.sh` (file log 400 MB, ảnh đĩa 512 MB cho LNX-047) trong `/usr/share/toinoi-lab/p05`. nginx do LNX-049 cài bằng apt (server cần Internet) |
| Thêm từ Phần 6 | Server thứ ba `toinoi-bastion` (10.20.0.10), SSH ở `127.0.0.1` cổng `2200`, chỉ nhận key. Đầu LNX-051, `chuan-bi.sh` cho app02 và bastion host key riêng (cố định) và cài key `an-laptop` (bastion nhận thêm key đồng bộ của app01). `unattended-upgrades`, `fail2ban`, `ansible-core` do LNX-054…056 cài bằng apt (server cần Internet) |
| Trình soạn thảo | `vim` (series không dùng `nano`) |

Trong SSH Studio: **Thêm server** → địa chỉ `127.0.0.1`, port `2222`, username `an`, xác thực Password. Bài [LNX-002](../../api/seed/content/linux-dung-lab-docker-ket-noi-ssh-studio.md) hướng dẫn từng bước, kể cả đối chiếu fingerprint.

> Container chạy `privileged` vì systemd cần quản lý cgroup. Đây là lab trên máy cá nhân: đừng mở cổng 2222 ra mạng và đừng dùng mật khẩu này ở nơi khác.

## Làm theo từng bài

Mỗi bài có một **kịch bản lệnh** trong `p01/` (bài 01…10), `p02/` (bài 11…20), `p03/` (bài 21…30), `p04/` (bài 31…40), `p05/` (bài 41…50) hoặc `p06/` (bài 51…60). Kịch bản là file `.sh` thường; bạn có thể đọc để biết bài gõ những lệnh nào.

```sh
npm run setup -- 05     # tạo lại server, đưa về trạng thái ĐẦU bài LNX-005
npm run setup -- 15     # như trên với LNX-015 (chạy cả Phần 1 rồi bài 11…14)
npm run p01             # tạo lại server, chạy cả Phần 1, in bản ghi (giống p01/OUTPUT.txt)
node run.mjs p01 07     # như trên nhưng chỉ in bản ghi của bài 07
npm run p02             # tạo lại server, chạy Phần 1 im lặng rồi Phần 2, in bản ghi (p02/OUTPUT.txt)
npm run kiem-tra        # chấm bài thực hành LNX-010 trên server đang chạy
npm run kiem-tra:p02    # chấm bài thực hành LNX-020 (20 yêu cầu)
npm run p03             # tạo lại server, chạy Phần 1–2 im lặng rồi Phần 3 (p03/OUTPUT.txt)
npm run kiem-tra:p03    # chấm bài thực hành LNX-030 (22 yêu cầu, gồm đăng nhập SSH thật)
npm run p04             # tạo lại server, chạy Phần 1–3 im lặng rồi Phần 4 (p04/OUTPUT.txt)
npm run kiem-tra:p04    # chấm bài thực hành LNX-040 (24 yêu cầu; thêm "-- --boot" để khởi động lại server rồi kiểm tra)
npm run p05             # tạo lại cả hai server, chạy Phần 1–4 im lặng rồi Phần 5 (p05/OUTPUT.txt)
npm run kiem-tra:p05    # chấm bài thực hành LNX-050 (20 yêu cầu, gồm kết nối thật từ toinoi-app-02)
npm run p06             # tạo lại cả ba server, chạy Phần 1–5 im lặng rồi Phần 6 (p06/OUTPUT.txt)
npm run kiem-tra:p06    # chấm Phần 6 (24 yêu cầu: bastion, tường lửa app02, vá, fail2ban, Ansible, triển khai, runbook)
npm test                # test của runner (không cần Docker)
```

`p01/OUTPUT.txt` là bản ghi mẫu. Khi so với máy của bạn, những thứ sau **sẽ khác**: ngày giờ, phiên bản kernel (kernel là của máy chạy Docker), số inode, mã máy trong `/var/log/journal`, fingerprint host key (sinh lúc build ảnh). Tên file, quyền, chủ sở hữu, số dòng và nội dung log **phải trùng**.

Quy ước trong file kịch bản:

| Dòng | Ý nghĩa |
|---|---|
| `## Tiêu đề` | In thành `# Tiêu đề` trong bản ghi |
| `#root lệnh` | Runner chạy với quyền root, không in: dữ liệu của lab (ví dụ log của Phần 2) |
| `#file ĐƯỜNG NGUỒN [NN]` | In `vim ĐƯỜNG` như một lệnh rồi ghi nội dung `pNN/files/NGUỒN` vào `ĐƯỜNG`: thay cho việc gõ file đó bằng vim |
| `#check ĐƯỜNG NGUỒN` | Ghi `pNN/files/NGUỒN` vào `ĐƯỜNG` mà không in: kết quả của một thao tác sửa bằng vim ngay trước đó |
| `#sudofile visudo|sudoedit ĐƯỜNG NGUỒN [NN]` | In `sudo visudo -f ĐƯỜNG` hoặc `sudoedit ĐƯỜNG` rồi ghi `pNN/files/NGUỒN` vào file hệ thống đó (Phần 3) |
| `#login NGƯỜI KEY [otp]` | Đăng nhập SSH thật vào chính server bằng key của lab (kèm mã TOTP nếu có `otp`), không in: để log của `sshd` có những lần đăng nhập mà bài đọc |
| `#ngat` | Mô phỏng mất kết nối SSH rồi đăng nhập lại (Phần 4): như bash khi nhận SIGHUP, gửi SIGHUP cho mọi job của shell, rồi coi như phiên mới. Công cụ chụp ảnh đóng tab phiên thật trong SSH Studio ở dòng `#gui` đi kèm |
| `#may appNN` | Các dòng sau chạy trên server khác (Phần 5: `app02` là `toinoi-app-02`), trong một phiên đăng nhập riêng của `an` trên máy đó, tới dòng `#may app01`. Bản ghi in dấu nhắc của máy đó. Công cụ chụp ảnh chạy những đoạn này ngầm (không có trong ảnh) |
| `#phien TÊN` | Như `#may`, cho Phần 6 (`app01`, `app02`, `bastion`): runner chạy các dòng sau trên máy đó trong một phiên mới. Công cụ chụp ảnh gõ chúng trong tab SSH Studio của máy đó (mở kết nối nếu chưa có). Các lệnh về phiên (`who`, `w`, `ss` của sshd) chỉ có nội dung khi chạy trong SSH Studio, vì runner dùng `su`, không phải phiên SSH |
| `#wait N` | Chờ N giây (chờ cron chạy ở LNX-020) |
| `#@ lệnh` | Runner chạy nhưng không in: thay cho một thao tác trên giao diện SSH Studio (upload, sửa file bằng trình sửa file, sửa bằng vim) |
| `#clear`, `#shot NN`, `#gui tên` | Chỉ dẫn cho công cụ chụp ảnh; runner bỏ qua |
| `# …` | Ghi chú |

Runner chạy kịch bản trong một phiên đăng nhập của `an` có TTY rộng 100 cột, nên `ls` chia cột như trên terminal. `~/.bashrc` thoát ngay khi shell không tương tác, nên runner nạp phần còn lại của nó (alias `ls`, `grep`, `ll` và những dòng bạn tự thêm) để kết quả giống phiên SSH; lệnh `source ~/.bashrc` trong kịch bản cũng được chạy theo cách đó. Hai điểm khác phiên SSH thật: bản ghi không có dòng `[sudo] password for an:` (runner xác thực sẵn), và mỗi bài là một phiên đăng nhập mới.

## Thư mục

| Đường dẫn | Nội dung |
|---|---|
| `docker/Dockerfile`, `docker-compose.yml` | Ảnh và container của server lab |
| `docker/fixtures/` | Dữ liệu mẫu chép vào ảnh; `bao-gia.log` sinh bằng `tools/tao-log.mjs` |
| `lib/lab.mjs`, `run.mjs` | Runner |
| `p01/NN-*.sh`, `p01/OUTPUT.txt`, `p01/kiem-tra.mjs` | Kịch bản, bản ghi mẫu và bộ chấm của Phần 1 |
| `p02/NN-*.sh`, `p02/files/`, `p02/OUTPUT.txt`, `p02/kiem-tra.mjs` | Như trên cho Phần 2; `files/` là các script và cấu hình mà bài gõ bằng vim |
| `docker/fixtures/p02/` | Log của 19 và 20/11 và `chuan-bi.sh` |
| `p03/NN-*.sh`, `p03/files/`, `p03/OUTPUT.txt`, `p03/kiem-tra.mjs` | Như trên cho Phần 3; `files/` có sudoers, cấu hình sshd và PAM |
| `docker/fixtures/p03/` | Key của lab (`an-laptop`, `deploy-ci`), host key thay thế cho LNX-023, bí mật TOTP và hai script askpass; chỉ dùng cho lab |
| `p04/NN-*.sh`, `p04/files/`, `p04/OUTPUT.txt`, `p04/kiem-tra.mjs` | Như trên cho Phần 4; `files/` có unit systemd, timer, drop-in, cấu hình logrotate và journald |
| `docker/fixtures/p04/` | `bao-gia` 1.5.0, script mẫu cho bài tiến trình (`tinh-lai-bao-cao`, `xuat-du-lieu`, `xuat-chuyen`) và `chuan-bi.sh` |
| `p05/NN-*.sh`, `p05/files/`, `p05/OUTPUT.txt`, `p05/kiem-tra.mjs` | Như trên cho Phần 5; `files/` có `~/.ssh/config` cho app02, site nginx, script, service và timer báo cáo tài nguyên. `p05/tu-may-ban.txt` là kết quả gọi `bao-gia` qua đường hầm từ Windows, do công cụ chụp ảnh ghi ở LNX-044 |
| `docker/fixtures/p05/` | Key đồng bộ `dong-bo` (chỉ dùng cho lab) và `chuan-bi.sh` |
| `p06/NN-*.sh`, `p06/files/`, `p06/OUTPUT.txt`, `p06/kiem-tra.mjs` | Như trên cho Phần 6; `files/` có cấu hình unattended-upgrades, jail fail2ban, inventory và playbook Ansible, script `trien-khai`, `kiem-tra-van-hanh` và runbook. `p06/tu-may-ban.txt` là phần đầu `workspace.json` và file sao lưu của SSH Studio, do công cụ chụp ghi ở LNX-053 |
| `docker/fixtures/p06/` | Host key cố định của app02 và bastion (và host key «sau khi dựng lại» của app02 cho LNX-059), bản build `bao-gia` 1.6.0, `chuan-bi.sh`; chỉ dùng cho lab |
| `may-cua-ban/` | Thư mục "máy của bạn" dùng cho khung local của Files (file để upload; `bang-gia-moi/` cho Đồng bộ ở LNX-046) |
| `tools/chup-anh/` | Công cụ chụp ảnh SSH Studio cho bài viết, xem [README](tools/chup-anh/README.md) |
