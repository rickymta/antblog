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
| Trình soạn thảo | `vim` (series không dùng `nano`) |

Trong SSH Studio: **Thêm server** → địa chỉ `127.0.0.1`, port `2222`, username `an`, xác thực Password. Bài [LNX-002](../../api/seed/content/linux-dung-lab-docker-ket-noi-ssh-studio.md) hướng dẫn từng bước, kể cả đối chiếu fingerprint.

> Container chạy `privileged` vì systemd cần quản lý cgroup. Đây là lab trên máy cá nhân: đừng mở cổng 2222 ra mạng và đừng dùng mật khẩu này ở nơi khác.

## Làm theo từng bài

Mỗi bài có một **kịch bản lệnh** trong `p01/` (bài 01…10) hoặc `p02/` (bài 11…20). Kịch bản là file `.sh` thường; bạn có thể đọc để biết bài gõ những lệnh nào.

```sh
npm run setup -- 05     # tạo lại server, đưa về trạng thái ĐẦU bài LNX-005
npm run setup -- 15     # như trên với LNX-015 (chạy cả Phần 1 rồi bài 11…14)
npm run p01             # tạo lại server, chạy cả Phần 1, in bản ghi (giống p01/OUTPUT.txt)
node run.mjs p01 07     # như trên nhưng chỉ in bản ghi của bài 07
npm run p02             # tạo lại server, chạy Phần 1 im lặng rồi Phần 2, in bản ghi (p02/OUTPUT.txt)
npm run kiem-tra        # chấm bài thực hành LNX-010 trên server đang chạy
npm run kiem-tra:p02    # chấm bài thực hành LNX-020 (20 yêu cầu)
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
| `may-cua-ban/` | Thư mục "máy của bạn" dùng cho khung local của Files (file để upload) |
| `tools/chup-anh/` | Công cụ chụp ảnh SSH Studio cho bài viết, xem [README](tools/chup-anh/README.md) |
