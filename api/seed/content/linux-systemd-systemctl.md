---
slug: linux-systemd-systemctl
title: "systemd và systemctl: đọc trạng thái dịch vụ, start, stop, enable, phụ thuộc giữa các unit"
summary: "systemd là PID 1 và quản lý dịch vụ thế nào, unit là gì, liệt kê dịch vụ đang chạy, đọc từng dòng của systemctl status, hỏi nhanh bằng is-active, is-enabled, show, xem file unit bằng systemctl cat, start/stop khác enable/disable (symlink trong multi-user.target.wants), phụ thuộc và thứ tự khởi động, systemctl --failed, và quyền đọc journal qua nhóm systemd-journal."
category: devops-linux
level: intermediate
tags: [linux, lnx-035, systemd, systemctl, service, unit, enable, journal, ssh-studio, toi-noi]
language: vi
---

[LNX-034](/articles/linux-job-nen-nohup-tmux) kết thúc với một lời khuyên: việc phải chạy mãi thì đừng để trong `nohup` hay `tmux` của ai đó. `bao-gia` là một việc như thế: phải chạy 24/7, tự khởi động khi server bật lại, tự đứng dậy khi chết, và có log ở một chỗ. Trên Ubuntu, người làm việc đó là **systemd**. Trước khi viết dịch vụ cho `bao-gia` ([LNX-036](/articles/linux-systemd-unit-bao-gia)), An học cách đọc những dịch vụ đang có.

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-035 · Phần 4/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/linux-tien-trinh-ps-top">LNX-031</a>, <a href="/articles/linux-tin-hieu-kill">LNX-032</a>. Trạng thái đầu bài: <code>npm run setup -- 35</code>. Server lab chạy systemd làm PID 1 như một server thật; vài dòng (đường dẫn <code>CGroup</code> có chữ <code>/docker/…</code>) là dấu vết của việc lab chạy trong container.</p>
</div>

## 1. systemd và unit

**systemd** là tiến trình đầu tiên kernel chạy (PID 1). Nó khởi động mọi thứ khác theo đúng thứ tự, giữ chúng chạy, ghi log của chúng vào **journal** ([LNX-037](/articles/linux-journalctl-doc-log)), và dừng chúng khi tắt máy. Mỗi thứ systemd quản lý là một **unit**, mô tả bằng một file văn bản:

| Loại unit | Đuôi | Ví dụ |
|---|---|---|
| Dịch vụ | `.service` | `ssh.service`, `cron.service`, `bao-gia.service` |
| Hẹn giờ | `.timer` | `logrotate.timer` ([LNX-038](/articles/linux-cron-systemd-timer)) |
| Nhóm mục tiêu | `.target` | `multi-user.target`: «máy đã lên, sẵn sàng cho người dùng» |
| Khác | `.socket`, `.mount`, `.path` | |

**`systemctl`** là lệnh để nói chuyện với systemd.

```text
an@toinoi-app-01:~$ ps -o pid,cmd -p 1
    PID CMD
      1 /sbin/init
an@toinoi-app-01:~$ systemctl --version | head -n 1
systemd 255 (255.4-1ubuntu8.19)
an@toinoi-app-01:~$ systemctl list-units --type=service --state=running --no-pager
  UNIT                     LOAD   ACTIVE SUB     DESCRIPTION
  cron.service             loaded active running Regular background program processing daemon
  dbus.service             loaded active running D-Bus System Message Bus
  ssh.service              loaded active running OpenBSD Secure Shell server
  systemd-journald.service loaded active running Journal Service

Legend: LOAD   → Reflects whether the unit definition was properly loaded.
        ACTIVE → The high-level unit activation state, i.e. generalization of SUB.
        SUB    → The low-level unit activation state, values depend on unit type.

4 loaded units listed.
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-systemd-systemctl/ssh-studio-04.webp" alt="Terminal của SSH Studio: ps -o pid,cmd -p 1 cho /sbin/init, systemctl --version cho systemd 255, list-units --type=service --state=running liệt kê cron, dbus, ssh, systemd-journald" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · systemd là PID 1 và các dịch vụ đang chạy</figcaption></figure>

Server lab tối giản nên chỉ có bốn dịch vụ đang chạy; một máy ảo Ubuntu thường có vài chục.

## 2. Đọc `systemctl status`

```text
an@toinoi-app-01:~$ systemctl status cron --no-pager
● cron.service - Regular background program processing daemon
     Loaded: loaded (/usr/lib/systemd/system/cron.service; enabled; preset: enabled)
     Active: active (running) since Tue 2026-11-24 07:04:57 +07; 48s ago
       Docs: man:cron(8)
   Main PID: 1891 (cron)
      Tasks: 1 (limit: 38443)
     Memory: 504.0K (peak: 504.0K)
        CPU: 2ms
     CGroup: /docker/c228c6fe829bbbdc35e4a059586331a9c9b70f195c352c923da91dbfae3fb840/system.slice/cron.service
             └─1891 /usr/sbin/cron -f -P
```

| Dòng | Đọc thế nào |
|---|---|
| `●` xanh / `○` / `×` đỏ | Đang chạy / không chạy / lỗi |
| `Loaded: … cron.service; enabled; preset: enabled` | File unit ở đâu; **enabled**: tự chạy khi boot |
| `Active: active (running) since …; 41s ago` | Trạng thái và từ lúc nào |
| `Main PID` | Tiến trình chính systemd đang trông |
| `Tasks`, `Memory`, `CPU` | Tài nguyên cả nhóm tiến trình của dịch vụ đã dùng |
| `CGroup` | Cây tiến trình của dịch vụ |
| Mấy dòng cuối | Log gần nhất từ journal (nếu bạn có quyền đọc, xem mục 5) |

**cgroup** là cơ chế của kernel gom các tiến trình của một dịch vụ vào một nhóm: systemd biết chính xác tiến trình nào thuộc dịch vụ nào (kể cả tiến trình con, cháu), giới hạn tài nguyên theo nhóm, và dừng được cả nhóm.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-systemd-systemctl/ssh-studio-01.webp" alt="Terminal của SSH Studio: systemctl status cron --no-pager cho cron.service loaded từ /usr/lib/systemd/system, enabled, Active: active (running), Main PID 1899, Tasks, Memory, CGroup" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · đọc <code>systemctl status</code>: nạp từ đâu, có bật lúc boot không, đang chạy không</figcaption></figure>

## 3. Hỏi nhanh, xem file unit

```text
an@toinoi-app-01:~$ systemctl is-active cron ssh
active
active
an@toinoi-app-01:~$ systemctl is-enabled cron ssh
enabled
enabled
an@toinoi-app-01:~$ systemctl show cron --no-pager -p MainPID -p ActiveState -p SubState -p Restart -p NRestarts
Restart=on-failure
MainPID=1891
NRestarts=0
ActiveState=active
SubState=running
an@toinoi-app-01:~$ systemctl cat cron --no-pager
[Unit]
Description=Regular background program processing daemon
Documentation=man:cron(8)
After=remote-fs.target nss-user-lookup.target

[Service]
EnvironmentFile=-/etc/default/cron
ExecStart=/usr/sbin/cron -f -P $EXTRA_OPTS
IgnoreSIGPIPE=false
KillMode=process
Restart=on-failure
SyslogFacility=cron

[Install]
WantedBy=multi-user.target
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-systemd-systemctl/ssh-studio-05.webp" alt="Terminal của SSH Studio: is-active và is-enabled cho cron và ssh, systemctl show cron với MainPID, ActiveState, SubState, Restart=on-failure, systemctl cat cron in file unit với [Unit], [Service], [Install]" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · hỏi nhanh trạng thái và đọc file unit</figcaption></figure>

- **`is-active`**, **`is-enabled`**: in một từ, mã thoát 0 nếu đúng. Hợp cho script: `systemctl is-active --quiet bao-gia || báo động`.
- **`systemctl show -p THUỘC_TÍNH`**: đọc từng thuộc tính chính xác, không phải đoán từ `status`. `NRestarts` là số lần systemd tự khởi động lại dịch vụ.
- **`systemctl cat`**: nội dung file unit đang được dùng, kèm đường dẫn ở dòng đầu. File của gói nằm ở `/usr/lib/systemd/system/`; file bạn viết đặt ở `/etc/systemd/system/` (ưu tiên hơn).

File unit của `cron` cho thấy cấu trúc mọi unit dịch vụ: `[Unit]` (mô tả, thứ tự), `[Service]` (chạy gì, `Restart=on-failure`), `[Install]` (`WantedBy=multi-user.target`: được bật cùng nhóm mục tiêu nào).

## 4. start/stop khác enable/disable

```text
an@toinoi-app-01:~$ sudo systemctl stop cron
an@toinoi-app-01:~$ systemctl is-active cron
inactive
an@toinoi-app-01:~$ systemctl status cron --no-pager | head -n 3
○ cron.service - Regular background program processing daemon
     Loaded: loaded (/usr/lib/systemd/system/cron.service; enabled; preset: enabled)
     Active: inactive (dead) since Tue 2026-11-24 07:05:46 +07; 11ms ago
an@toinoi-app-01:~$ sudo systemctl start cron
an@toinoi-app-01:~$ sudo systemctl disable cron
Synchronizing state of cron.service with SysV service script with /usr/lib/systemd/systemd-sysv-install.
Executing: /usr/lib/systemd/systemd-sysv-install disable cron
Removed "/etc/systemd/system/multi-user.target.wants/cron.service".
an@toinoi-app-01:~$ ls /etc/systemd/system/multi-user.target.wants/
e2scrub_reap.service  remote-fs.target  ssh.service
an@toinoi-app-01:~$ sudo systemctl enable --now cron
Synchronizing state of cron.service with SysV service script with /usr/lib/systemd/systemd-sysv-install.
Executing: /usr/lib/systemd/systemd-sysv-install enable cron
Created symlink /etc/systemd/system/multi-user.target.wants/cron.service → /usr/lib/systemd/system/cron.service.
an@toinoi-app-01:~$ systemctl is-enabled cron
enabled
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-systemd-systemctl/ssh-studio-06.webp" alt="Terminal của SSH Studio: sudo systemctl stop cron, is-active in inactive, status cho Active: inactive (dead) nhưng vẫn enabled" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>stop</code> dừng dịch vụ, không đổi trạng thái lúc boot</figcaption></figure>

| Lệnh | Ảnh hưởng | Khi nào |
|---|---|---|
| `start` / `stop` / `restart` | **Bây giờ** | Chạy, dừng, khởi động lại ngay |
| `enable` / `disable` | **Lần boot sau** | Có tự chạy khi server khởi động không |
| `enable --now` | Cả hai | Bật lúc boot và chạy ngay |

`enable` không có gì bí ẩn: nó tạo một **symlink** trong `/etc/systemd/system/multi-user.target.wants/` trỏ tới file unit. Khi máy lên tới `multi-user.target`, systemd khởi động mọi unit trong thư mục `.wants` đó. `disable` xoá symlink. Dòng `Synchronizing state … SysV` là vì `cron` còn có script khởi động kiểu cũ trong `/etc/init.d`; systemd giữ hai bên khớp nhau.

`stop` gửi `SIGTERM` cho tiến trình chính, đợi, rồi mới `SIGKILL` ([LNX-032](/articles/linux-tin-hieu-kill)). Vì `cron` có `Restart=on-failure`, nếu bạn `kill -9` nó thay vì `systemctl stop`, systemd sẽ khởi động lại ngay: dịch vụ systemd thì dừng bằng `systemctl`.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-systemd-systemctl/ssh-studio-02.webp" alt="Terminal của SSH Studio: sudo systemctl start cron, disable xoá symlink trong multi-user.target.wants, ls thư mục wants không còn cron, enable --now tạo lại symlink, is-enabled in enabled" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>enable</code>/<code>disable</code> chỉ là tạo hoặc xoá symlink</figcaption></figure>

## 5. Phụ thuộc, lỗi, và quyền đọc journal

```text
an@toinoi-app-01:~$ systemctl list-dependencies ssh --no-pager | head -n 8
ssh.service
● ├─system.slice
● └─sysinit.target
●   ├─dev-hugepages.mount
●   ├─dev-mqueue.mount
○   ├─kmod-static-nodes.service
●   ├─ldconfig.service
○   ├─proc-sys-fs-binfmt_misc.automount
an@toinoi-app-01:~$ systemctl show ssh --no-pager -p After -p Wants | cut -c1-100
Wants=
After=basic.target systemd-journald.socket auditd.service sysinit.target network.target -.mount syst
an@toinoi-app-01:~$ systemctl --failed --no-pager
  UNIT LOAD ACTIVE SUB DESCRIPTION

0 loaded units listed.
an@toinoi-app-01:~$ systemctl list-unit-files --type=timer --no-pager | head -n 6
UNIT FILE                      STATE    PRESET
apt-daily-upgrade.timer        enabled  enabled
apt-daily.timer                enabled  enabled
dpkg-db-backup.timer           enabled  enabled
e2scrub_all.timer              enabled  enabled
fstrim.timer                   enabled  enabled

an@toinoi-app-01:~$ journalctl -u cron -n 2 --no-pager
Hint: You are currently not seeing messages from other users and the system.
      Users in groups 'adm', 'systemd-journal' can see all messages.
      Pass -q to turn off this notice.
-- No entries --
an@toinoi-app-01:~$ id -nG
an sudo toinoi ssh-users
an@toinoi-app-01:~$ sudo usermod -aG systemd-journal an
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-systemd-systemctl/ssh-studio-07.webp" alt="Terminal của SSH Studio: list-dependencies ssh cho cây phụ thuộc system.slice và sysinit.target, show ssh -p After -p Wants, systemctl --failed báo 0 loaded units listed" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · phụ thuộc giữa các unit và unit đang lỗi</figcaption></figure>

- **`list-dependencies`** vẽ cây những gì một unit cần (`Wants=`, `Requires=`). **`After=`** chỉ là thứ tự (khởi động sau), không kéo theo; muốn «cần và sau», unit ghi cả hai.
- **`systemctl --failed`**: các unit đang lỗi. Câu lệnh đầu tiên nên chạy khi đăng nhập vào một server có vấn đề.
- **`list-unit-files --type=timer`**: các timer cài sẵn của Ubuntu, sẽ gặp lại ở [LNX-038](/articles/linux-cron-systemd-timer).
- **`journalctl` trả về rỗng** với dòng `Hint`: người dùng thường chỉ đọc được log của chính mình. Log hệ thống dành cho nhóm **`systemd-journal`** (hoặc `adm`). An thêm mình vào nhóm; như mọi thay đổi nhóm ([LNX-007](/articles/linux-nguoi-dung-nhom-quyen-file)), nó có hiệu lực từ lần đăng nhập sau. Bài tiếp theo bắt đầu bằng một phiên mới.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-systemd-systemctl/ssh-studio-03.webp" alt="Terminal của SSH Studio: list-unit-files --type=timer liệt kê các timer enabled, journalctl -u cron báo không thấy log của người khác và hệ thống vì an chưa thuộc nhóm adm hoặc systemd-journal, id -nG, usermod -aG systemd-journal an" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · timer có sẵn và quyền đọc journal</figcaption></figure>

## 6. Đối chiếu thao tác

| Việc | Lệnh |
|---|---|
| Dịch vụ đang chạy | `systemctl list-units --type=service --state=running` |
| Trạng thái chi tiết | `systemctl status TÊN` |
| Câu trả lời một từ (cho script) | `systemctl is-active`, `is-enabled` |
| Một thuộc tính | `systemctl show TÊN -p MainPID` |
| Nội dung file unit | `systemctl cat TÊN` |
| Chạy / dừng / khởi động lại | `sudo systemctl start`, `stop`, `restart` |
| Bật / tắt lúc boot | `sudo systemctl enable`, `disable` (`--now` để làm luôn) |
| Unit lỗi | `systemctl --failed` |

## 7. Lỗi thường gặp

**`enable` rồi tưởng dịch vụ đã chạy.** `enable` chỉ là cho lần boot sau. Dùng `enable --now`, hoặc `start` thêm.

**Sửa file unit mà không `daemon-reload`.** systemd vẫn dùng bản cũ đã nạp; `status` sẽ cảnh báo «changed on disk».

**Dừng dịch vụ bằng `kill`.** Với `Restart=`, nó chạy lại ngay; trạng thái và log rối.

**Đọc `Active: active (exited)` thành «đang chạy».** Với dịch vụ chạy một lần (`Type=oneshot`), nghĩa là đã chạy xong và thành công; không có tiến trình nào đang sống.

**`journalctl` không thấy gì.** Thiếu nhóm `systemd-journal`/`adm`, hoặc dùng `sudo journalctl`.

## Bài tập

1. `systemctl list-units --type=target --no-pager`. `multi-user.target` và `graphical.target` khác nhau gì? Server của bạn mặc định đi tới đâu (`systemctl get-default`)?
2. `systemctl show ssh -p Restart -p KillMode -p TimeoutStopUSec`. Dừng `ssh` thì các phiên SSH đang mở có bị cắt không? (Gợi ý: `KillMode`.)
3. `systemctl list-dependencies --reverse cron`: ai cần `cron`?
4. Đặt `systemctl mask cron` rồi thử `start`. Thông báo gì? `unmask` lại. `mask` khác `disable` thế nào?

## Đọc tiếp

[LNX-036 · Viết unit systemd cho bao-gia](/articles/linux-systemd-unit-bao-gia): đưa bản 1.5.0 vào systemd, chạy bằng tài khoản dịch vụ, tự khởi động lại khi chết.
