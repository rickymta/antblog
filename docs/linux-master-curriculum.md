# Linux qua SSH Studio và dự án Tới Nơi

Khung biên tập v1 · 22/11/2026 · **6 phần × 10 bài = 60 bài**. Mỗi phần có **3 cơ bản / 4 trung bình / 3 nâng cao**, gồm cả bài thực hành cuối phần. Đây là khung viết bài; hoàn thành khung không có nghĩa đã hoàn thành nội dung.

## Phạm vi và cách đọc

Series riêng cho **Linux trên server**, thuộc chuyên mục `devops-linux` ("Linux", con của `devops`). Ví dụ lấy từ hệ thống hư cấu **Tới Nơi** (đặt xe, giao đồ ăn, giao bưu kiện): An, developer của nhóm backend, lần đầu được cấp tài khoản trên server `toinoi-app-01`, nơi chạy dịch vụ báo giá `bao-gia`; Bình thuộc nhóm hạ tầng. Đối tượng: developer và tester chưa từng làm việc trên server Linux, người đã gõ lệnh theo thói quen nhưng chưa hiểu cơ chế, và người phải tự vận hành vài server cho nhóm.

**Mọi thao tác được làm qua một công cụ, SSH Studio, theo hai cách:**

1. **Terminal**: gõ lệnh trong phiên SSH. Đây là cách chính, vì server thật thường chỉ có dòng lệnh.
2. **Giao diện của SSH Studio** khi thao tác có trên giao diện: khung **Files** (SFTP: duyệt, upload, download, đổi tên, xoá), **trình sửa file**, hộp thoại **Đổi quyền**, **Đồng bộ**, tab **Ports** (port forwarding), **SSH Keys**, **Known Hosts**, tìm trong terminal, Launchpad và workspace.

Hai cách cùng thao tác trên một server, nên người đọc thấy ngay nút trên giao diện tương ứng lệnh gì và lệnh làm thay đổi gì trên hệ thống file. Bài không dạy "bấm nút" thay cho hiểu cơ chế: mỗi thao tác đều giải thích nó làm gì với file, quyền, tiến trình hay kết nối mạng.

CB = `beginner`, TB = `intermediate`, NC = `advanced`; toàn khung có 18 CB / 24 TB / 18 NC.

| Phần | Chủ đề | Đầu vào | Sản phẩm cuối phần |
|---|---|---|---|
| P01 | Nền tảng: kết nối, shell, file và quyền | Không cần biết Linux | Thư mục triển khai `/srv/toinoi/bao-gia` có cấu trúc, nhóm và quyền đúng, kiểm tra được bằng script |
| P02 | Xử lý văn bản và shell script | P01 | Script sao lưu và báo cáo log của `bao-gia`, chạy được theo lịch |
| P03 | Người dùng, sudo và SSH an toàn | P01–P02 | Server chỉ đăng nhập bằng SSH key, có tài khoản triển khai riêng, sudo giới hạn theo việc |
| P04 | Tiến trình, dịch vụ và gói phần mềm | P01–P03 | `bao-gia` chạy bằng systemd unit, có log trong journal, tự khởi động lại, có timer dọn log |
| P05 | Mạng, lưu trữ và giám sát | P01–P04 | Tường lửa chỉ mở cổng cần thiết, truy cập dịch vụ nội bộ qua port forwarding, báo cáo đĩa và tài nguyên |
| P06 | Vận hành nhiều server và xử lý sự cố | P01–P05 | Workspace nhiều server có bastion, runbook xử lý sự cố có bằng chứng chạy |

## Môi trường và công cụ

- **Server lab:** Ubuntu **24.04 LTS** chạy trong Docker (ảnh ghim theo digest), **systemd làm PID 1** và `sshd` là một service, để server lab giống server thật: có `systemctl`, `journalctl`, cron, `sudo` hỏi mật khẩu. Kernel là kernel của máy chạy Docker (trên Windows là kernel của WSL 2); bài nói rõ ở chỗ kết quả phụ thuộc kernel. Cổng SSH chỉ mở ở `127.0.0.1:2222`.
- **SSH Studio** (SSH client của tác giả, Tauri 2 + React + xterm.js, lõi Rust), bản **0.3.0**. Giao diện tiếng Việt, nền tối. **Ảnh chụp màn hình trong bài lấy từ SSH Studio chạy trực tiếp từ repository `ssh-studio`** với thư mục dữ liệu riêng (`dev.sshstudio.blog`, không có server nào khác ngoài lab), kết nối tới server lab thật. Khi SSH Studio chưa hỗ trợ một việc, bài làm bằng lệnh trong terminal và nói rõ.
- **Trình soạn thảo trong terminal: `vim`.** Cả series chỉ dùng `vim` (và trình sửa file của SSH Studio); không dùng `nano`. Ảnh lab cài gói `vim`, không cài `nano`.
- **Lab tái lập:** `examples/toi-noi-linux`, runner Node 22 không có dependency npm. Mỗi bài có một **kịch bản lệnh** `pNN/MM-ten.sh`; runner chạy kịch bản trong một phiên đăng nhập của người dùng `an` (có TTY) và in lại từng lệnh kèm dấu nhắc. `pNN/OUTPUT.txt` là bản ghi mẫu. Ngày giờ, kích thước thư mục, số inode, phiên bản kernel và fingerprint host key khác nhau giữa các máy; bài nói rõ chỗ nào phải trùng, chỗ nào không.
- **Trạng thái đầu bài:** `npm run setup -- 05` tạo lại server và chạy im lặng các kịch bản trước, đưa server về đúng trạng thái đầu bài LNX-005 để kết nối bằng SSH Studio và làm theo. Bài đánh số liên tục qua các phần (`setup -- 15` chạy cả Phần 1 rồi LNX-011…014).
- **Script gõ bằng vim:** trong kịch bản, `#file ĐƯỜNG NGUỒN` cho runner ghi nội dung `pNN/files/NGUỒN` (và in `vim ĐƯỜNG` như một lệnh); công cụ chụp ảnh gõ thật nội dung đó bằng vim trong SSH Studio rồi đối chiếu từng byte với file mẫu. `#check` đối chiếu kết quả của một thao tác sửa bằng vim.
- **Tài khoản lab:** `an` và `binh` (mật khẩu `toinoi-lab`, chỉ dùng cho lab), tài khoản dịch vụ `baogia` (không đăng nhập được), nhóm `toinoi`. Không dùng `root` để đăng nhập.

## Tổ chức chuyên mục

Chuyên mục `devops-linux` (icon `devops`), con của `devops`, cùng cấp với `devops-aws` và `devops-azure`. Tiền tố bài `linux-`, mã bài `lnx-0NN`, lộ trình `linux-qua-ssh-studio` (P01) rồi `linux-qua-ssh-studio-phan-02` … `-phan-06`. Chỉ thêm vào `api/seed/learning-paths.json` các bài đã viết và kiểm tra.

Bài có nhắc Linux ở series khác giữ chuyên mục cũ và được liên kết khi cần: máy ảo Linux trên Azure (`azure-virtual-machine-linux-chay-api`), EC2 trong series AWS, xác thực SSH của Git (GIT-024).

## Tiêu chuẩn viết và hoàn tất một phần

Frontmatter: `slug`, `title`, `summary`, `category: devops-linux`, `level`, `tags` (gồm mã bài `lnx-0NN`), `language: vi`. Giọng văn theo [`content-dictionary/style-guide.md`](content-dictionary/style-guide.md) và [`terminology.md`](content-dictionary/terminology.md). Cấu trúc mỗi bài: mở bằng một tình huống của nhóm Tới Nơi; giải thích cơ chế; **làm bằng lệnh trong terminal của SSH Studio** (khối lệnh + kết quả chép từ `OUTPUT.txt` của lab); **làm trên giao diện SSH Studio** khi có (có ảnh); bảng đối chiếu thao tác giao diện ↔ lệnh; lỗi thường gặp; bài tập; đọc tiếp.

Thuật ngữ Linux giữ tiếng Anh khi không có từ Việt quen dùng (shell, kernel, prompt, pipe, symlink, sudo, daemon, mount…); "thư mục", "quyền", "tiến trình", "người dùng", "nhóm" dùng tiếng Việt. Không bịa kết quả lệnh: mọi khối kết quả lấy từ lab đã chạy.

**Ảnh minh hoạ:** chụp SSH Studio thật với server lab, cửa sổ **1600×900 điểm CSS ở tỉ lệ 1,5**, tức ảnh **2400×1350 pixel** (không thấp hơn 1600×900): `apps/web/public/images/articles/<slug>/ssh-studio-NN.webp`, nhúng bằng `<figure class="tkh-fig"><div class="tkh-fig-box"><img … width="1600" height="900" loading="lazy"></div><figcaption>…</figcaption></figure>`, có `alt` mô tả nội dung. Ảnh do `tools/chup-anh` của lab tạo: công cụ gõ **đúng các lệnh của kịch bản bài** vào terminal của SSH Studio và thao tác trên giao diện ở các dòng `#gui`, nên ảnh và bản ghi dạng chữ cùng một quy trình. Ảnh không thay cho khối lệnh/kết quả dạng chữ. Khung local của Files chỉ trỏ vào `examples/toi-noi-linux/may-cua-ban`; không để lọt thư mục cá nhân, server hay tài khoản thật vào ảnh.

**Từ điển keyword:** thuật ngữ quan trọng của mỗi phần được đề xuất ở `content-dictionary/glossary-candidates/devops-linux.json` (cùng cấu trúc với các file khác: `terms`, `conflicts`, `deferred`, đếm `occurrences` trên bài thật). Từ quá chung dễ bắt nhầm ("shell", "quyền", "nhóm", "pipe") gắn `categories: ["devops-linux"]` hoặc `autoLink: false`.

Hoàn thành **nội dung** một phần: cập nhật trạng thái ở đây, seed lộ trình, README lab, chạy `npm test` và `npm run kiem-tra` của lab, kiểm lint thuật ngữ, rồi commit riêng `content(linux): complete part N`. Seed/deploy là bước riêng.

## P01 · Nền tảng: kết nối, shell, file và quyền

Đầu vào: không cần biết Linux. Đầu ra: kết nối được tới server bằng SSH Studio, đi lại trong cây thư mục, tạo/sửa/tìm file, đọc và đặt quyền, nối lệnh bằng pipe; bàn giao thư mục triển khai `/srv/toinoi/bao-gia`.

Lộ trình: `linux-qua-ssh-studio`. Lab: `examples/toi-noi-linux/p01`.

| ID | Bài / slug | Mức | Làm được sau bài / sản phẩm thực hành | Trạng thái |
|---|---|---|---|---|
| LNX-001 | Linux là gì và SSH Studio giúp gì cho nhóm Tới Nơi<br>`linux-la-gi-va-ssh-studio` | CB | Phân biệt kernel, bản phân phối, shell; biết server khác máy cá nhân ở đâu; hai cách làm việc trong SSH Studio | Đã viết |
| LNX-002 | Dựng server Linux bằng Docker và kết nối bằng SSH Studio<br>`linux-dung-lab-docker-ket-noi-ssh-studio` | CB | Dựng lab, thêm server, đối chiếu fingerprint host key, đăng nhập bằng mật khẩu, Known Hosts | Đã viết |
| LNX-003 | Shell và dòng lệnh: lệnh, tuỳ chọn, đối số và cách tra trợ giúp<br>`linux-shell-dong-lenh-tro-giup` | CB | Đọc dấu nhắc, tách lệnh/tuỳ chọn/đối số, `type`, `--help`, `man`, mã thoát `$?`, phím tắt terminal | Đã viết |
| LNX-004 | Cây thư mục Linux: đường dẫn, `cd`, `ls` và khung Files<br>`linux-cay-thu-muc-duong-dan` | TB | Đường dẫn tuyệt đối/tương đối, ý nghĩa `/etc`, `/var`, `/opt`, `/srv`, `/home`; `ls -l`, `tree`, `file`, `stat` | Đã viết |
| LNX-005 | Tạo, sao chép, di chuyển và xoá file: lệnh và SFTP<br>`linux-tao-sao-chep-di-chuyen-xoa-file` | TB | `mkdir -p`, `cp -r`, `mv`, `rm -r`, wildcard, brace expansion; upload/download bằng khung Files | Đã viết |
| LNX-006 | Xem và sửa file văn bản: `cat`, `less`, `tail`, `vim` và trình sửa file của SSH Studio<br>`linux-xem-sua-file-van-ban` | TB | Xem file dài, tìm trong `less`, sửa bằng `vim` (ba chế độ), sửa bằng trình sửa file, `diff` | Đã viết |
| LNX-007 | Người dùng, nhóm và quyền file: `chmod`, `chown`, `sudo`<br>`linux-nguoi-dung-nhom-quyen-file` | TB | Đọc `rwx` của file và thư mục, `chmod` dạng chữ/số, `umask`, `sudo`, vì sao đổi nhóm phải đăng nhập lại | Đã viết |
| LNX-008 | Chuyển hướng và pipe: stdin, stdout, stderr<br>`linux-chuyen-huong-pipe` | NC | `>`, `>>`, `2>`, `2>&1`, `<`, pipe, `sort \| uniq -c`, `tee`, `&&`/`\|\|` | Đã viết |
| LNX-009 | Tìm kiếm: `find`, `grep` và tìm trong terminal<br>`linux-tim-kiem-find-grep` | NC | Tìm file theo tên/loại/kích thước/thời gian/quyền, `-exec`; `grep` có ngữ cảnh, regex, đệ quy | Đã viết |
| LNX-010 | Thực hành: chuẩn bị thư mục triển khai dịch vụ báo giá<br>`linux-thuc-hanh-thu-muc-trien-khai` | NC | Bàn giao `/srv/toinoi/bao-gia` có `releases`, `shared`, symlink `current`, nhóm và quyền đúng; `kiem-tra` 15 PASS | Đã viết |

## P02 · Xử lý văn bản và shell script

Đầu vào: P01. Đầu ra: thư mục `/srv/toinoi/bao-gia/scripts` với `bao-cao-log`, `sao-luu-log`, `tom-tat.awk` và `hang-ngay` chạy lúc 01:17 mỗi đêm bằng cron; báo cáo và bản sao lưu log của `bao-gia`.

Lộ trình: `linux-qua-ssh-studio-phan-02`. Lab: `examples/toi-noi-linux/p02` (bài đánh số tiếp 11…20). Đầu phần, lab thêm log đã xoay vòng của 19/11 (`bao-gia.log.2.gz`) và 20/11 (`bao-gia.log.1`, có lần nâng cấp 1.4.1 → 1.4.2 và sự cố khoá bảng giá lúc 09:05–09:15); ảnh lab cài thêm `zstd`, `xz`, `shellcheck`. Bộ chấm: `npm run kiem-tra:p02` (20 yêu cầu).

| ID | Bài / slug | Mức | Làm được sau bài / sản phẩm thực hành | Trạng thái |
|---|---|---|---|---|
| LNX-011 | Biến môi trường, `PATH` và file khởi động của shell<br>`linux-bien-moi-truong-path-bashrc` | CB | `export`, `PATH`, `.bashrc` vs `.profile`, alias, vì sao lệnh "không tìm thấy" | Đã viết |
| LNX-012 | vim cho công việc hằng ngày trên server<br>`linux-vim-hang-ngay` | CB | Di chuyển, tìm/thay thế, sao chép/dán, undo, sửa nhiều file, `.vimrc` tối thiểu | Đã viết |
| LNX-013 | Trích cột và dòng: `cut`, `sort`, `uniq`, `tr`, `wc`<br>`linux-cut-sort-uniq-tr` | CB | Báo cáo số lỗi theo dịch vụ và theo giờ từ log | Đã viết |
| LNX-014 | `sed`: sửa văn bản theo dòng<br>`linux-sed-sua-van-ban` | TB | Thay thế, xoá dòng, sửa tại chỗ có bản sao lưu, biểu thức chính quy cơ bản | Đã viết |
| LNX-015 | `awk`: xử lý log có cột<br>`linux-awk-xu-ly-log` | TB | Lọc theo điều kiện, tính tổng/trung bình thời gian báo giá, nhóm theo khoá | Đã viết |
| LNX-016 | Shell script đầu tiên: shebang, biến, tham số, mã thoát<br>`linux-shell-script-dau-tien` | TB | Script có `set -euo pipefail`, đọc tham số, trả mã thoát đúng | Đã viết |
| LNX-017 | Điều kiện, vòng lặp và hàm trong bash<br>`linux-bash-dieu-kien-vong-lap-ham` | TB | `if`/`case`/`for`/`while`, `test`, hàm, xử lý tên file có dấu cách | Đã viết |
| LNX-018 | Nén và lưu trữ: `tar`, `gzip`, `zstd`<br>`linux-tar-gzip-nen-luu-tru` | NC | Đóng gói log theo ngày, kiểm tra và giải nén, tải về bằng khung Files | Đã viết |
| LNX-019 | Script an toàn: quoting, `trap`, file tạm, khoá chạy đơn<br>`linux-shell-script-an-toan` | NC | Script không hỏng khi lỗi giữa chừng, không chạy chồng nhau | Đã viết |
| LNX-020 | Thực hành: script sao lưu và báo cáo log của bao-gia<br>`linux-thuc-hanh-script-sao-luu-log` | NC | Bàn giao script có kiểm tra tự động, chạy bằng cron | Đã viết |

## P03 · Người dùng, sudo và SSH an toàn

Đầu vào: P01–P02. Đầu ra: `toinoi-app-01` chỉ nhận SSH key, không mật khẩu, không root, chỉ nhóm `ssh-users`; người cần thêm mã TOTP, tài khoản `deploy` của CI chỉ dùng key qua `Match User`; sudo theo việc cho `cuong` và `deploy`; thư mục chung `3770`.

Lộ trình: `linux-qua-ssh-studio-phan-03`. Lab: `examples/toi-noi-linux/p03` (bài 21…30). Ảnh lab cài thêm `libpam-google-authenticator`, `oathtool` và bộ key, host key thay thế, bí mật TOTP cố định trong `/usr/share/toinoi-lab/p03`; key «trên laptop của An» ở `may-cua-ban/khoa-lab`. Runner có thêm chỉ dẫn `#sudofile` (sửa file bằng `visudo`/`sudoedit`) và `#login` (đăng nhập SSH thật, có OTP). Bộ chấm: `npm run kiem-tra:p03` (22 yêu cầu, gồm đăng nhập thật).

| ID | Bài / slug | Mức | Làm được sau bài / sản phẩm thực hành | Trạng thái |
|---|---|---|---|---|
| LNX-021 | Quản lý người dùng và nhóm<br>`linux-quan-ly-nguoi-dung-nhom` | CB | `useradd`, `usermod -aG`, `passwd -l`, `chage`, tài khoản dịch vụ `nologin`, `userdel`; đổi mật khẩu bắt buộc lần đầu trong SSH Studio | Đã viết |
| LNX-022 | SSH key: tạo trong SSH Studio và cài lên server<br>`linux-ssh-key-ssh-studio` | CB | Nhập key vào thư viện SSH Studio, `authorized_keys` với quyền đúng, chuyển server sang SSH Key, kiểm chứng trong log, lỗi `StrictModes` | Đã viết |
| LNX-023 | Host key, Known Hosts và cảnh báo host key thay đổi<br>`linux-host-key-known-hosts` | CB | Fingerprint host key, `known_hosts` băm, cảnh báo host key đổi ở OpenSSH và SSH Studio, xác minh rồi quên host, giữ host key khi dựng lại | Đã viết |
| LNX-024 | `sudo` và `sudoers`: cấp quyền theo việc<br>`linux-sudo-sudoers` | TB | `visudo -f /etc/sudoers.d/…`, `Cmnd_Alias`, `NOPASSWD`, `sudo -l -U`, lệnh thoát ra shell, `visudo -c`, nhật ký sudo | Đã viết |
| LNX-025 | Cấu hình `sshd`: tắt mật khẩu, tắt root, giới hạn người dùng<br>`linux-sshd-config-an-toan` | TB | `sshd -T`, `sudoedit`, «giá trị đầu tiên thắng» trong `sshd_config.d`, `AllowGroups`, `reload` và phiên dự phòng | Đã viết |
| LNX-026 | Quyền đặc biệt: setuid, setgid, sticky bit và thư mục dùng chung<br>`linux-setuid-setgid-sticky` | TB | Đọc `s`/`S`/`t`/`T`, setuid của `passwd`, setgid cho thư mục log, sticky `/tmp`, thư mục chung `3770` | Đã viết |
| LNX-027 | ACL: quyền chi tiết hơn chủ sở hữu–nhóm–người khác<br>`linux-acl-setfacl` | TB | `setfacl -m`/`-d`/`-x`/`-k`/`-b`, `getfacl`, dấu `+`, mask và `chmod` | Đã viết |
| LNX-028 | SSH agent, passphrase và nhiều key<br>`linux-ssh-agent-passphrase` | NC | Key có passphrase, `ssh-agent`, `ssh-add`, `~/.ssh/config` với `IdentitiesOnly`, `ssh -G`, agent forwarding; phương thức Agent của SSH Studio | Đã viết |
| LNX-029 | Xác thực hai bước cho SSH (TOTP)<br>`linux-ssh-mfa-totp` | NC | `~/.google_authenticator`, PAM `pam_google_authenticator`, `AuthenticationMethods publickey,keyboard-interactive`, đăng nhập hai bước trong SSH Studio | Đã viết |
| LNX-030 | Thực hành: khoá chặt đăng nhập của toinoi-app-01<br>`linux-thuc-hanh-khoa-dang-nhap-ssh` | NC | Bàn giao tài khoản `deploy` chỉ dùng key, sudo hai lệnh, `Match User deploy`, `sshd -T -C`; `kiem-tra` 22 PASS | Đã viết |

## P04 · Tiến trình, dịch vụ và gói phần mềm

Đầu vào: P01–P03. Đầu ra: `bao-gia` 1.5.0 chạy bằng unit systemd (tài khoản `baogia`, `EnvironmentFile=`, `ExecStartPre=`, `Restart=on-failure`, reload bằng SIGHUP) có drop-in siết quyền, log trong journal (giới hạn bằng drop-in của journald), timer dọn báo cáo `Persistent=true`, logrotate cho log yêu cầu.

Lộ trình: `linux-qua-ssh-studio-phan-04`. Lab: `examples/toi-noi-linux/p04` (bài 31…40). Ảnh lab cài thêm `python3-minimal`, `logrotate`; `/usr/share/toinoi-lab/p04` có `bao-gia` 1.5.0 (Python, chỉ thư viện chuẩn) và các script mẫu cho bài tiến trình. LNX-033 cài `jq`, `tmux` từ kho của Ubuntu (cần Internet). Runner có thêm chỉ dẫn `#ngat` (mô phỏng mất kết nối SSH: shell gửi SIGHUP cho các job); công cụ chụp ảnh đóng tab thật trong SSH Studio. Bộ chấm: `npm run kiem-tra:p04` (24 yêu cầu; `-- --boot` thêm 2 mục sau khi khởi động lại server).

| ID | Bài / slug | Mức | Làm được sau bài / sản phẩm thực hành | Trạng thái |
|---|---|---|---|---|
| LNX-031 | Tiến trình: `ps`, `top`, PID và cây tiến trình<br>`linux-tien-trinh-ps-top` | CB | PID/PPID, `/proc`, `ps aux`, `ps -ef --forest`, `pgrep -a -u -f`, `top` tương tác và `-b`, load average, `free` trong container | Đã viết |
| LNX-032 | Tín hiệu và dừng tiến trình: `kill`, SIGTERM, SIGKILL<br>`linux-tin-hieu-kill` | CB | `kill -l`, SIGTERM có dọn dẹp, `Operation not permitted`, `SigIgn`/`SigCgt`, SIGKILL để lại file khoá, `pkill -e`, `killall -v` | Đã viết |
| LNX-033 | Cài phần mềm bằng `apt`<br>`linux-apt-cai-phan-mem` | CB | Kho gói, `apt update`/`upgrade`, `search`/`show`/`policy`, cài `jq tmux sysstat`, `dpkg -l -L -S`, remove/purge (`rc`), autoremove, `apt-mark hold`, `history.log` | Đã viết |
| LNX-034 | Job nền, `nohup` và `tmux`: việc chạy lâu không chết theo phiên SSH<br>`linux-job-nen-nohup-tmux` | TB | `&`, `jobs`, SIGHUP khi đóng tab SSH Studio (job thường chết, `nohup` sống), `tmux new -d`, `capture-pane`, gắn lại sau khi đứt kết nối | Đã viết |
| LNX-035 | systemd: `systemctl` và trạng thái dịch vụ<br>`linux-systemd-systemctl` | TB | PID 1, unit, `list-units`, đọc `status`, `is-active`/`show`/`cat`, start/stop với enable/disable (symlink `.wants`), `list-dependencies`, `--failed`, nhóm `systemd-journal` | Đã viết |
| LNX-036 | Viết unit systemd cho bao-gia<br>`linux-systemd-unit-bao-gia` | TB | Unit `bao-gia.service` (`Type=exec`, `User=`, hai `EnvironmentFile=`, `ExecStartPre=`, `ExecReload=`, `Restart=`), lỗi `203/EXEC`, `current` → 1.5.0, `kill -9` thấy tự khởi động lại | Đã viết |
| LNX-037 | `journalctl`: đọc log của dịch vụ<br>`linux-journalctl-doc-log` | TB | `journalctl -u -n --since -o`, mức ưu tiên và tiền tố `<N>`, `-p err` lần theo lỗi reload, trường `_UID`/`_PID`, `-f` trong SSH Studio, drop-in `journald.conf.d`, vacuum | Đã viết |
| LNX-038 | Lên lịch: cron và systemd timer<br>`linux-cron-systemd-timer` | NC | cron (crontab, `/etc/crontab`, `cron.daily`) và `list-timers`, cặp service `oneshot` + timer dọn báo cáo, `systemd-analyze calendar`/`verify`, `Persistent=true`, `RandomizedDelaySec`, so sánh cron với timer | Đã viết |
| LNX-039 | logrotate và dọn log<br>`linux-logrotate` | NC | `logrotate.timer`, `logrotate.conf`, `/etc/logrotate.d/bao-gia` (daily, rotate 14, compress, delaycompress, su, create, postrotate reload), `-d`, `-f -v`, kiểm chứng file mới nhờ SIGHUP | Đã viết |
| LNX-040 | Thực hành: đưa bao-gia thành dịch vụ systemd<br>`linux-thuc-hanh-dich-vu-systemd` | NC | Drop-in siết quyền (ProtectSystem=strict, ReadWritePaths, ProtectHome, PrivateTmp, NoNewPrivileges, MemoryMax), `systemd-analyze security`, `deploy` chỉ restart; `kiem-tra` 24 PASS (26 với `--boot`) | Đã viết |

## P05 · Mạng, lưu trữ và giám sát

Đầu vào: P01–P04. Đầu ra: `toinoi-app-01` chỉ mở SSH (`LIMIT`) và HTTPS cho mạng nội bộ, `bao-gia` chỉ nghe loopback sau nginx có TLS (chứng chỉ tự ký có SAN, HTTP chuyển 301), ổ dữ liệu `/du-lieu` gắn theo UUID với `nofail`, bản sao lưu release trên `toinoi-app-02` bằng rsync qua SSH, báo cáo tài nguyên 15 phút một lần bằng timer.

Lộ trình: `linux-qua-ssh-studio-phan-05`. Lab: `examples/toi-noi-linux/p05` (bài 41…50). Lab có thêm server thứ hai `toinoi-app-02` (10.20.0.12, chỉ trong mạng nội bộ `10.20.0.0/24`); runner có chỉ dẫn `#may app02` để chạy một đoạn trên máy đó. Ảnh lab cài thêm `netcat-openbsd`, `bind9-dnsutils`, `lsof`, `ufw`, `nftables`, `e2fsprogs`; `/usr/share/toinoi-lab/p05` có key đồng bộ và script chuẩn bị (file log 400 MB, ảnh đĩa 512 MB cho loop device). LNX-049 cài nginx từ kho của Ubuntu (cần Internet). Bộ chấm: `npm run kiem-tra:p05` (20 yêu cầu, có kết nối thật từ `toinoi-app-02`).

| ID | Bài / slug | Mức | Làm được sau bài / sản phẩm thực hành | Trạng thái |
|---|---|---|---|---|
| LNX-041 | Địa chỉ IP, cổng và `ss`: dịch vụ đang nghe ở đâu<br>`linux-ip-cong-ss` | CB | `ip -br addr`, `ip route`, `/etc/hosts`, `ss -tln`/`-tlnp`/`-tn`, `127.0.0.1` khác `0.0.0.0`, thử bằng `curl` qua loopback và IP nội bộ | Đã viết |
| LNX-042 | DNS và kiểm tra kết nối: `ping`, `dig`, `curl`<br>`linux-dns-ping-curl` | CB | `resolv.conf`, `nsswitch.conf`, `getent` so với `dig`, thêm tên vào `/etc/hosts`, `ping` (và khi nó im lặng), mã lỗi 6/7/28 của `curl`, `-sv`, `-w` đo thời gian | Đã viết |
| LNX-043 | Dung lượng đĩa: `df`, `du` và tìm thứ chiếm chỗ<br>`linux-df-du-dung-luong` | CB | `df -hT`, `du -sh`/`--max-depth`, `find -size +100M`, hết inode (`df -i`) trên tmpfs nhỏ, file đã xoá còn mở (`lsof +L1`), dọn log 400 MB đúng cách | Đã viết |
| LNX-044 | Port forwarding trong SSH Studio: Local, Remote, Dynamic<br>`linux-ssh-port-forwarding` | TB | Tab Ports: Local 18080 → 127.0.0.1:8080 gọi `bao-gia` từ máy mình, `ss` thấy kết nối của `sshd`, Dynamic (SOCKS), `ssh -L/-R/-D` tương ứng | Đã viết |
| LNX-045 | Tường lửa với `ufw` và nftables<br>`linux-tuong-lua-ufw-nftables` | TB | `ufw default deny incoming`, `allow OpenSSH` trước khi bật, `limit`, 443 chỉ cho `10.20.0.0/24`, `nft list ruleset`, kiểm chứng bằng `nc -zv` từ `toinoi-app-02` | Đã viết |
| LNX-046 | Đồng bộ thư mục: Đồng bộ của SSH Studio và `rsync`<br>`linux-dong-bo-rsync-ssh-studio` | TB | Đồng bộ thư mục `bang-gia-moi` lên server (so sánh, chiều, Mirror), `rsync -ai`, `--dry-run --delete`, dấu `/` cuối nguồn, key đồng bộ riêng và `rsync` qua SSH sang app02 | Đã viết |
| LNX-047 | Đĩa, phân vùng, hệ thống file và `mount`<br>`linux-dia-phan-vung-mount` | TB | `lsblk`, `blkid`, `mkfs.ext4 -L`, `mount`, `fstab` theo UUID với `nofail`, `umount` + `mount -a` trước khi khởi động lại, `findmnt` | Đã viết |
| LNX-048 | Bộ nhớ và tải: `free`, `vmstat`, load average<br>`linux-bo-nho-tai-he-thong` | NC | `free` (available, buff/cache), `/proc/meminfo`, swap, `MemoryCurrent` của dịch vụ, đọc từng cột `vmstat`, tải CPU (`r`, `us`, load) và tải đĩa (`bo`, `wa`), `ps --sort=-rss` | Đã viết |
| LNX-049 | TLS và reverse proxy nginx cho bao-gia<br>`linux-nginx-reverse-proxy-tls` | NC | Cài nginx, `bao-gia` về loopback, chứng chỉ tự ký có SAN (`openssl req -addext`), site 443 `proxy_pass` với header `X-Forwarded-*`, 80 chuyển 301, `nginx -t`, `curl --cacert`, log truy cập | Đã viết |
| LNX-050 | Thực hành: mạng và giám sát cho toinoi-app-01<br>`linux-thuc-hanh-mang-giam-sat` | NC | Tường lửa kiểm chứng từ app02, script `bao-cao-tai-nguyen` chạy bằng `baogia` qua timer 15 phút (`StandardOutput=append:`), bảng kiểm tra bàn giao; `kiem-tra` 20 PASS | Đã viết |

## P06 · Vận hành nhiều server và xử lý sự cố

Đầu vào: P01–P05. Lộ trình dự kiến `-phan-06`.

| ID | Bài / slug | Mức | Làm được sau bài / sản phẩm thực hành | Trạng thái |
|---|---|---|---|---|
| LNX-051 | Workspace, thư mục và Launchpad: nhiều server cùng lúc<br>`linux-workspace-launchpad-nhieu-server` | CB | Gom server của Tới Nơi theo môi trường, kết nối tất cả, theo dõi phiên | Dự kiến |
| LNX-052 | Bastion và jump host<br>`linux-bastion-jump-host` | CB | Vào server nội bộ qua một bước nhảy, chuỗi nhiều bước | Dự kiến |
| LNX-053 | Sao lưu và khôi phục cấu hình SSH Studio, chuyển máy<br>`linux-sao-luu-khoi-phuc-ssh-studio` | CB | Backup mã hoá, khôi phục gộp, vì sao chép `workspace.json` là chưa đủ | Dự kiến |
| LNX-054 | Cập nhật hệ thống an toàn: bản vá, khởi động lại, unattended-upgrades<br>`linux-cap-nhat-he-thong-an-toan` | TB | Lịch vá, kiểm tra cần reboot, quay lui | Dự kiến |
| LNX-055 | Gia cố server: checklist tối thiểu và fail2ban<br>`linux-gia-co-server-fail2ban` | TB | Giảm bề mặt tấn công, chặn dò mật khẩu | Dự kiến |
| LNX-056 | Chạy một lệnh trên nhiều server: vòng lặp SSH và Ansible tối thiểu<br>`linux-nhieu-server-ansible-toi-thieu` | TB | Thu thập thông tin và áp cấu hình đồng loạt | Dự kiến |
| LNX-057 | Triển khai không gián đoạn bằng thư mục release và symlink<br>`linux-trien-khai-release-symlink` | TB | Đổi `current`, quay lui trong một lệnh | Dự kiến |
| LNX-058 | Sự cố: đĩa đầy, hết inode, OOM killer<br>`linux-su-co-dia-day-oom` | NC | Chẩn đoán theo trình tự, có bằng chứng từ log | Dự kiến |
| LNX-059 | Sự cố: không SSH được vào server<br>`linux-su-co-khong-ssh-duoc` | NC | Đọc thông báo lỗi của SSH Studio, khoanh vùng mạng/sshd/xác thực | Dự kiến |
| LNX-060 | Thực hành: runbook vận hành toinoi-app-01<br>`linux-thuc-hanh-runbook-van-hanh` | NC | Bàn giao runbook có bằng chứng chạy trên lab | Dự kiến |
