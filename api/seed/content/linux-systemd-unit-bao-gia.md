---
slug: linux-systemd-unit-bao-gia
title: "Viết unit systemd cho bao-gia: User=, EnvironmentFile=, ExecStartPre=, Restart= và đọc lỗi 203/EXEC"
summary: "Đưa dịch vụ báo giá 1.5.0 vào systemd: file cấu hình môi trường, viết /etc/systemd/system/bao-gia.service bằng sudoedit, daemon-reload, lần khởi động đầu thất bại với status=203/EXEC và cách đọc ra nguyên nhân, sửa bằng symlink current, chạy bằng tài khoản dịch vụ baogia, enable, giết tiến trình bằng SIGKILL để thấy Restart=on-failure làm việc, và reload bằng SIGHUP."
category: devops-linux
level: intermediate
tags: [linux, lnx-036, systemd, unit-file, service, restart, environmentfile, ssh-studio, toi-noi]
language: vi
---

Bản 1.5.0 của `bao-gia` đã nằm trên server từ đầu Phần 4, chưa ai chạy nó. Bình đặt yêu cầu: chạy bằng tài khoản dịch vụ `baogia` (không phải `an`, không phải `root`), cấu hình và bí mật đọc từ file trong `shared/config`, kiểm tra cấu hình trước khi chạy, tự khởi động cùng server, tự đứng dậy nếu chết. Tất cả những điều đó là vài dòng trong một **file unit**.

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-036 · Phần 4/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/linux-systemd-systemctl">LNX-035</a>, <a href="/articles/linux-thuc-hanh-thu-muc-trien-khai">LNX-010</a> (cấu trúc <code>releases</code>, <code>shared</code>, <code>current</code>). Trạng thái đầu bài: <code>npm run setup -- 36</code>. Đăng nhập lại bằng phiên mới để nhóm <code>systemd-journal</code> thêm ở bài trước có hiệu lực.</p>
</div>

## 1. Bản 1.5.0 và cấu hình

```text
an@toinoi-app-01:~$ ls -l /srv/toinoi/bao-gia/releases/1.5.0
total 16
-rw-r----- 1 an toinoi  188 Nov 23 08:40 bang-gia.csv
-rwxr-x--- 1 an toinoi 5505 Nov 23 08:40 bao-gia
-rwxr-x--- 1 an toinoi  403 Nov 23 08:40 kiem-tra-cau-hinh.sh
an@toinoi-app-01:~$ head -n 3 /srv/toinoi/bao-gia/releases/1.5.0/bao-gia
#!/usr/bin/python3
#
an@toinoi-app-01:~$ readlink /srv/toinoi/bao-gia/current
releases/1.4.2
an@toinoi-app-01:~$ vim /srv/toinoi/bao-gia/shared/config/bao-gia.env
an@toinoi-app-01:~$ ls -l /srv/toinoi/bao-gia/shared/config
total 12
-rw-r----- 1 an toinoi 275 Nov 24 07:03 appsettings.json
-rw-rw-r-- 1 an an     257 Nov 24 07:05 bao-gia.env
-rw------- 1 an toinoi 126 Nov 24 07:03 bi-mat.env
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-systemd-unit-bao-gia/ssh-studio-05.webp" alt="Terminal của SSH Studio: ls -l releases/1.5.0 có bang-gia.csv, bao-gia, kiem-tra-cau-hinh.sh, head -n 3 cho #!/usr/bin/python3, current vẫn trỏ releases/1.4.2, vim bao-gia.env, ls -l shared/config" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · bản 1.5.0 đã có, <code>current</code> vẫn ở 1.4.2</figcaption></figure>

- `bao-gia` 1.5.0 là một chương trình Python chỉ dùng thư viện chuẩn; dòng shebang `#!/usr/bin/python3` ([LNX-016](/articles/linux-shell-script-dau-tien)) cho phép chạy thẳng file. Nó lắng nghe cổng 8080 ở `127.0.0.1`, đọc cấu hình từ **biến môi trường**, in log vòng đời ra stdout và ghi từng yêu cầu vào file log.
- `current` vẫn trỏ tới `1.4.2`. An để nguyên, có chủ đích, để thấy chuyện gì xảy ra.
- An gõ file cấu hình bằng vim. Không có bí mật nào ở đây; khoá dịch vụ bản đồ nằm riêng trong `bi-mat.env` (quyền `600`) từ [LNX-010](/articles/linux-thuc-hanh-thu-muc-trien-khai).

```text
# Cấu hình của bao-gia, đọc qua EnvironmentFile= của unit (LNX-036). Không có bí mật ở đây.
BAO_GIA_CONG=8080
BAO_GIA_MOI_TRUONG=production
BAO_GIA_BANG_GIA=/srv/toinoi/bao-gia/current/bang-gia.csv
BAO_GIA_LOG=/var/log/toinoi/bao-gia.log
```

## 2. File unit

File unit do người quản trị viết đặt ở **`/etc/systemd/system/`**. An soạn bằng `sudoedit` ([LNX-025](/articles/linux-sshd-config-an-toan)):

```bash
sudoedit /etc/systemd/system/bao-gia.service
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-systemd-unit-bao-gia/ssh-studio-01.webp" alt="Vim mở /etc/systemd/system/bao-gia.service qua sudoedit trong SSH Studio, gồm các mục [Unit] với Description và After=network.target, [Service] với Type=exec, User=baogia, Group=toinoi, WorkingDirectory, hai dòng EnvironmentFile, ExecStartPre, ExecStart, ExecReload, Restart=on-failure, RestartSec=2, và [Install] với WantedBy=multi-user.target" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>sudoedit</code> file unit của bao-gia</figcaption></figure>

```ini
[Unit]
Description=bao-gia: dich vu bao gia cua Toi Noi
After=network.target

[Service]
Type=exec
User=baogia
Group=toinoi
WorkingDirectory=/srv/toinoi/bao-gia/current
EnvironmentFile=/srv/toinoi/bao-gia/shared/config/bao-gia.env
EnvironmentFile=/srv/toinoi/bao-gia/shared/config/bi-mat.env
ExecStartPre=/srv/toinoi/bao-gia/current/kiem-tra-cau-hinh.sh
ExecStart=/srv/toinoi/bao-gia/current/bao-gia
ExecReload=/bin/kill -HUP $MAINPID
Restart=on-failure
RestartSec=2

[Install]
WantedBy=multi-user.target
```

| Dòng | Ý nghĩa |
|---|---|
| `After=network.target` | Khởi động sau khi mạng đã lên (chỉ thứ tự) |
| `Type=exec` | Coi là đã khởi động khi chương trình được chạy thành công; chạy không được thì `start` báo lỗi ngay |
| `User=`, `Group=` | Chạy bằng tài khoản dịch vụ ([LNX-021](/articles/linux-quan-ly-nguoi-dung-nhom)) và nhóm `toinoi` (đọc được bảng giá `640`) |
| `WorkingDirectory=` | Thư mục hiện tại của tiến trình |
| `EnvironmentFile=` | Nạp biến môi trường từ file. systemd đọc bằng quyền root, nên `bi-mat.env` vẫn giữ `600` thuộc `an`; `baogia` không đọc được file, chỉ nhận giá trị |
| `ExecStartPre=` | Chạy trước; lỗi thì không chạy `ExecStart` |
| `ExecStart=` | Chương trình chính, đường dẫn tuyệt đối |
| `ExecReload=` | `systemctl reload` làm gì: gửi `SIGHUP` cho tiến trình chính |
| `Restart=on-failure`, `RestartSec=2` | Tự chạy lại sau 2 giây nếu thoát với lỗi hoặc bị giết bằng tín hiệu |
| `WantedBy=multi-user.target` | `enable` sẽ gắn dịch vụ vào lúc boot |

```text
an@toinoi-app-01:~$ sudo systemctl daemon-reload
an@toinoi-app-01:~$ systemctl cat bao-gia --no-pager
[Unit]
Description=bao-gia: dich vu bao gia cua Toi Noi
After=network.target

[Service]
Type=exec
User=baogia
Group=toinoi
WorkingDirectory=/srv/toinoi/bao-gia/current
EnvironmentFile=/srv/toinoi/bao-gia/shared/config/bao-gia.env
EnvironmentFile=/srv/toinoi/bao-gia/shared/config/bi-mat.env
ExecStartPre=/srv/toinoi/bao-gia/current/kiem-tra-cau-hinh.sh
ExecStart=/srv/toinoi/bao-gia/current/bao-gia
ExecReload=/bin/kill -HUP $MAINPID
Restart=on-failure
RestartSec=2

[Install]
WantedBy=multi-user.target
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-systemd-unit-bao-gia/ssh-studio-06.webp" alt="Terminal của SSH Studio: sudoedit bao-gia.service, sudo systemctl daemon-reload, systemctl cat bao-gia in file unit trong /etc/systemd/system với User=baogia, EnvironmentFile, ExecStartPre, ExecStart, Restart=on-failure" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>daemon-reload</code> rồi kiểm tra systemd đã đọc đúng file unit</figcaption></figure>

**`daemon-reload`** bảo systemd đọc lại file unit sau mỗi lần thêm hoặc sửa.

## 3. Lần khởi động đầu tiên thất bại

```text
an@toinoi-app-01:~$ id -nG
an sudo systemd-journal toinoi ssh-users
an@toinoi-app-01:~$ sudo systemctl start bao-gia
Job for bao-gia.service failed because the control process exited with error code.
See "systemctl status bao-gia.service" and "journalctl -xeu bao-gia.service" for details.
an@toinoi-app-01:~$ systemctl status bao-gia --no-pager | head -n 7
● bao-gia.service - bao-gia: dich vu bao gia cua Toi Noi
     Loaded: loaded (/etc/systemd/system/bao-gia.service; disabled; preset: enabled)
     Active: activating (auto-restart) (Result: exit-code) since Tue 2026-11-24 07:05:47 +07; 6ms ago
    Process: 3788 ExecStartPre=/srv/toinoi/bao-gia/current/kiem-tra-cau-hinh.sh (code=exited, status=0/SUCCESS)
    Process: 3794 ExecStart=/srv/toinoi/bao-gia/current/bao-gia (code=exited, status=203/EXEC)
   Main PID: 3794 (code=exited, status=203/EXEC)
        CPU: 9ms
an@toinoi-app-01:~$ sudo systemctl stop bao-gia
an@toinoi-app-01:~$ journalctl -u bao-gia --no-pager -n 4 -o cat
bao-gia.service: Main process exited, code=exited, status=203/EXEC
bao-gia.service: Failed with result 'exit-code'.
Failed to start bao-gia.service - bao-gia: dich vu bao gia cua Toi Noi.
Stopped bao-gia.service - bao-gia: dich vu bao gia cua Toi Noi.
```

Cách đọc một dịch vụ không lên:

1. **`systemctl status`**: `ExecStartPre` qua (`status=0/SUCCESS`), nhưng `ExecStart` thoát với **`status=203/EXEC`**. Mã từ 200 trở lên là của **systemd**, không phải của chương trình: 203 nghĩa là systemd không chạy được file đó (không có, không có quyền chạy, hoặc dòng shebang sai). Trạng thái `activating (auto-restart)`: `Restart=on-failure` đang thử lại.
2. **`journalctl -u`**: log đầy đủ (An đã thuộc nhóm `systemd-journal`, xem `id -nG`).
3. An **`stop`** để chặn vòng thử lại trong lúc sửa.

Nguyên nhân: `current` trỏ tới `1.4.2`, bản đó không có file `bao-gia`.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-systemd-unit-bao-gia/ssh-studio-02.webp" alt="Terminal của SSH Studio: id -nG có systemd-journal, sudo systemctl start bao-gia báo Job for bao-gia.service failed, status hiện ExecStartPre status=0/SUCCESS, ExecStart status=203/EXEC và activating auto-restart, stop rồi journalctl -o cat với Main process exited status=203/EXEC, Failed to start" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · lần đầu thất bại: <code>status=203/EXEC</code></figcaption></figure>

## 4. Sửa và chạy

```text
an@toinoi-app-01:~$ ln -sfn releases/1.5.0 /srv/toinoi/bao-gia/current
an@toinoi-app-01:~$ readlink /srv/toinoi/bao-gia/current
releases/1.5.0
an@toinoi-app-01:~$ sudo systemctl start bao-gia
an@toinoi-app-01:~$ systemctl status bao-gia --no-pager
● bao-gia.service - bao-gia: dich vu bao gia cua Toi Noi
     Loaded: loaded (/etc/systemd/system/bao-gia.service; disabled; preset: enabled)
     Active: active (running) since Tue 2026-11-24 07:05:47 +07; 1s ago
    Process: 3807 ExecStartPre=/srv/toinoi/bao-gia/current/kiem-tra-cau-hinh.sh (code=exited, status=0/SUCCESS)
   Main PID: 3812 (bao-gia)
      Tasks: 1 (limit: 38443)
     Memory: 12.4M (peak: 14.3M)
        CPU: 85ms
     CGroup: /docker/c228c6fe829bbbdc35e4a059586331a9c9b70f195c352c923da91dbfae3fb840/system.slice/bao-gia.service
             └─3812 /usr/bin/python3 /srv/toinoi/bao-gia/current/bao-gia

Nov 24 07:05:47 toinoi-app-01 systemd[1]: Starting bao-gia.service - bao-gia: dich vu bao gia…Noi...
Nov 24 07:05:47 toinoi-app-01 kiem-tra-cau-hinh.sh[3807]: bao-gia: bang gia co 4 loai dich vu
Nov 24 07:05:47 toinoi-app-01 kiem-tra-cau-hinh.sh[3807]: bao-gia: cau hinh hop le
Nov 24 07:05:47 toinoi-app-01 systemd[1]: Started bao-gia.service - bao-gia: dich vu bao gia …i Noi.
Nov 24 07:05:47 toinoi-app-01 bao-gia[3812]: INFO bao-gia 1.5.0 khoi dong, pid 3812, cong 8080…ction
Nov 24 07:05:47 toinoi-app-01 bao-gia[3812]: INFO nap bang gia: 4 loai dich vu tu /srv/toinoi/…a.csv
Hint: Some lines were ellipsized, use -l to show in full.
an@toinoi-app-01:~$ ps -o pid,user,group,cmd -p $(systemctl show bao-gia -p MainPID --value)
    PID USER     GROUP    CMD
   3812 baogia   toinoi   /usr/bin/python3 /srv/toinoi/bao-gia/current/bao-gia
an@toinoi-app-01:~$ curl -s 'http://127.0.0.1:8080/bao-gia?dich_vu=car4&km=12.5'
{"dich_vu": "car4", "km": 12.5, "gia": 69000}
```

- **`ln -sfn`** đổi symlink `current` sang bản mới ([LNX-010](/articles/linux-thuc-hanh-thu-muc-trien-khai)): một bước, quay lui cũng một bước.
- `status` giờ là `active (running)`. Các dòng cuối là log từ journal: hai dòng của `kiem-tra-cau-hinh.sh`, rồi chính `bao-gia` báo khởi động và nạp bảng giá. Tên ở mỗi dòng (`bao-gia[PID]`) là chương trình và PID đã in dòng đó. Dòng bị cắt bằng `…` vì terminal hẹp; `systemctl status -l` hoặc `journalctl` in đủ.
- `ps` xác nhận tiến trình chạy bằng `baogia:toinoi`. Lệnh `curl` gửi một yêu cầu báo giá: 12,5 km ô tô 4 chỗ là 69 000 đồng.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-systemd-unit-bao-gia/ssh-studio-03.webp" alt="Terminal của SSH Studio: ln -sfn chuyển current sang releases/1.5.0, start thành công, status active running với Main PID và các dòng log bao-gia khoi dong và nap bang gia, ps cho user baogia group toinoi, curl trả JSON gia 69000" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>bao-gia</code> chạy bằng systemd</figcaption></figure>

## 5. Bật lúc boot, tự đứng dậy khi chết

```text
an@toinoi-app-01:~$ sudo systemctl enable bao-gia
Created symlink /etc/systemd/system/multi-user.target.wants/bao-gia.service → /etc/systemd/system/bao-gia.service.
an@toinoi-app-01:~$ systemctl show bao-gia --no-pager -p MainPID -p NRestarts
MainPID=3812
NRestarts=0
an@toinoi-app-01:~$ sudo kill -KILL $(systemctl show bao-gia -p MainPID --value)
an@toinoi-app-01:~$ systemctl show bao-gia --no-pager -p MainPID -p NRestarts
MainPID=3859
NRestarts=1
an@toinoi-app-01:~$ journalctl -u bao-gia --no-pager -n 6 -o cat
Starting bao-gia.service - bao-gia: dich vu bao gia cua Toi Noi...
bao-gia: bang gia co 4 loai dich vu
bao-gia: cau hinh hop le
Started bao-gia.service - bao-gia: dich vu bao gia cua Toi Noi.
INFO bao-gia 1.5.0 khoi dong, pid 3859, cong 8080, moi truong production
INFO nap bang gia: 4 loai dich vu tu /srv/toinoi/bao-gia/current/bang-gia.csv

an@toinoi-app-01:~$ sudo systemctl reload bao-gia
an@toinoi-app-01:~$ journalctl -u bao-gia --no-pager -n 2 -o cat
INFO nhan SIGHUP: mo lai file log, nap lai bang gia (4 loai dich vu)
Reloaded bao-gia.service - bao-gia: dich vu bao gia cua Toi Noi.
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-systemd-unit-bao-gia/ssh-studio-07.webp" alt="Terminal của SSH Studio: enable bao-gia tạo symlink, MainPID 6839 NRestarts 0, kill -KILL tiến trình chính, sau vài giây MainPID 7020 NRestarts 1, journalctl cho thấy dịch vụ khởi động lại" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>Restart=on-failure</code>: bị giết thì systemd dựng lại</figcaption></figure>

- **`enable`** tạo symlink trong `multi-user.target.wants` ([LNX-035](/articles/linux-systemd-systemctl)).
- An giết tiến trình chính bằng **`SIGKILL`**, mô phỏng một lần chương trình chết bất ngờ. Bốn giây sau: `MainPID` mới, **`NRestarts=1`**, journal ghi lại cả quá trình khởi động. Đó là `Restart=on-failure`.
- **`systemctl reload`** chạy `ExecReload`: `bao-gia` nhận `SIGHUP`, mở lại file log và nạp lại bảng giá mà không ngừng phục vụ. Ứng dụng phải tự hỗ trợ việc đó; không phải chương trình nào cũng có.

`Restart=on-failure` không khởi động lại khi bạn `systemctl stop` (đó là dừng có chủ đích), cũng không khi chương trình thoát với mã 0. Nó cũng có giới hạn: mặc định quá 5 lần trong 10 giây thì systemd bỏ cuộc (`start-limit-hit`), để một dịch vụ hỏng hẳn không quay vòng mãi.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-systemd-unit-bao-gia/ssh-studio-04.webp" alt="Terminal của SSH Studio: sudo systemctl reload bao-gia, journalctl -u bao-gia -n 2 cho INFO nhan SIGHUP: mo lai file log, nap lai bang gia và Reloaded bao-gia.service" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>reload</code> gửi SIGHUP, dịch vụ nạp lại mà không dừng</figcaption></figure>

## 6. Đối chiếu thao tác

| Việc | Lệnh |
|---|---|
| Viết hoặc sửa unit | `sudoedit /etc/systemd/system/TÊN.service`, rồi `sudo systemctl daemon-reload` |
| Hoặc để systemd mở trình soạn thảo | `sudo systemctl edit --full --force TÊN.service` |
| Chạy, xem | `sudo systemctl start TÊN`, `systemctl status TÊN` |
| Log của dịch vụ | `journalctl -u TÊN` |
| Bật lúc boot | `sudo systemctl enable TÊN` |
| Nạp lại cấu hình | `sudo systemctl reload TÊN` |
| Xoá trạng thái lỗi, đếm lại giới hạn khởi động | `sudo systemctl reset-failed TÊN` |

## 7. Lỗi thường gặp

**`status=203/EXEC`.** File trong `ExecStart=` không có, không có quyền `x` với `User=`, hoặc shebang trỏ tới trình thông dịch không có.

**`status=217/USER`.** Tài khoản trong `User=` không tồn tại.

**`status=200/CHDIR`.** `WorkingDirectory=` không có hoặc `User=` không vào được.

**Dùng đường dẫn tương đối hoặc lệnh shell trong `ExecStart=`.** systemd không chạy qua shell: `>`, `|`, `&&`, `~` không có nghĩa. Cần thì `ExecStart=/bin/bash -c '…'`, nhưng tốt hơn là một script.

**Để bí mật thẳng trong file unit.** `systemctl show` và `systemctl cat` cho mọi người dùng đọc được unit. Bí mật nằm trong `EnvironmentFile=` quyền chặt (hoặc `LoadCredential=`).

**Quên `daemon-reload`.** Sửa xong mà dịch vụ vẫn chạy theo cấu hình cũ.

## Bài tập

1. Đổi `BAO_GIA_CONG=8081` trong `bao-gia.env`. `reload` có đủ không, hay cần `restart`? Vì sao? Thử bằng `curl`, rồi đặt lại 8080.
2. Đổi `User=baogia` thành một tài khoản không có (`User=khongco`), `daemon-reload`, `restart`. Mã lỗi là gì? Sửa lại.
3. Đọc `systemctl show bao-gia -p StartLimitBurst -p StartLimitIntervalUSec`. Gây lỗi 203 rồi đếm xem systemd thử bao nhiêu lần trước khi dừng.
4. Viết `ExecStopPost=/usr/bin/logger "bao-gia da dung"` vào unit. Dừng dịch vụ và tìm dòng đó trong journal.

## Đọc tiếp

[LNX-037 · journalctl: đọc log của dịch vụ](/articles/linux-journalctl-doc-log): lọc theo thời gian, mức ưu tiên, theo dõi trực tiếp, và giới hạn dung lượng journal.
