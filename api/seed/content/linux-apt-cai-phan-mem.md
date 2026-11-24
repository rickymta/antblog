---
slug: linux-apt-cai-phan-mem
title: "Cài phần mềm bằng apt: kho gói, apt update, install, remove, purge, dpkg và ghim phiên bản"
summary: "Gói và kho gói trên Ubuntu, apt update khác apt upgrade ở đâu, tìm và xem gói trước khi cài, cài jq, tmux, sysstat, phụ thuộc kéo theo, dpkg -l, -L, -S để biết file nào thuộc gói nào, remove khác purge (trạng thái rc), autoremove, apt-mark hold để ghim phiên bản, và đọc lịch sử cài đặt trong /var/log/apt."
category: devops-linux
level: beginner
tags: [linux, lnx-033, apt, dpkg, package, ubuntu, apt-mark, ssh-studio, toi-noi]
language: vi
---

Bản 1.5.0 của `bao-gia` trả JSON, và An muốn đọc nó cho dễ ngay trên server; Bình thì cần một công cụ giữ việc chạy lâu sống sót khi mạng chập chờn. Hai thứ đó (`jq` và `tmux`) chưa có trên `toinoi-app-01`. Trên Windows, An sẽ tải file cài đặt từ trang web của phần mềm. Trên server Ubuntu, cách làm đúng gần như luôn là **apt**.

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-033 · Phần 4/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/linux-sudo-sudoers">LNX-024</a> (sudo). Trạng thái đầu bài: <code>npm run setup -- 33</code>. Bài tải gói từ kho của Ubuntu nên server lab cần Internet; phiên bản và kích thước gói bạn thấy có thể khác bài, tuỳ thời điểm.</p>
</div>

## 1. Gói và kho gói

Phần mềm trên Ubuntu được phân phối dưới dạng **gói** (*package*, file `.deb`): chương trình, file cấu hình mặc định, trang `man`, và danh sách **phụ thuộc** (những gói khác cần có). Gói nằm trong **kho** (*repository*) do Ubuntu vận hành; mỗi gói trong kho được ký, và apt kiểm tra chữ ký trước khi cài.

| Công cụ | Làm gì |
|---|---|
| **`apt`** | Tìm, tải, cài, gỡ, nâng cấp; tự giải quyết phụ thuộc. Lệnh dùng hằng ngày |
| **`dpkg`** | Lớp bên dưới: cài một file `.deb`, liệt kê gói đã cài, file nào của gói nào. Không tự tải phụ thuộc |
| `apt-get`, `apt-cache` | Bản cũ hơn của `apt`, giao diện ổn định, hay dùng trong script |

```text
an@toinoi-app-01:~$ grep -v '^#' /etc/apt/sources.list.d/ubuntu.sources | grep -E '^(Types|URIs|Suites|Components):'
Types: deb
URIs: http://archive.ubuntu.com/ubuntu/
Suites: noble noble-updates noble-backports
Components: main universe restricted multiverse
Types: deb
URIs: http://security.ubuntu.com/ubuntu/
Suites: noble-security
Components: main universe restricted multiverse
an@toinoi-app-01:~$ sudo apt update 2>&1 | tail -n 3
Building dependency tree...
Reading state information...
All packages are up to date.
```

- Kho được khai báo trong `/etc/apt/sources.list.d/ubuntu.sources`: hai địa chỉ (kho chính và kho bản vá bảo mật), các **suite** của Ubuntu 24.04 «noble» (`noble` lúc phát hành, `noble-updates` bản cập nhật, `noble-security` bản vá bảo mật), và các **component** (`main` do Canonical hỗ trợ, `universe` cộng đồng…).
- **`apt update`** chỉ tải **danh sách gói mới nhất** từ kho về máy, không cài hay nâng cấp gì. Chạy nó trước khi cài, để apt biết phiên bản mới nhất. Dòng cuối cho biết có gói nào nâng cấp được không.
- **`apt upgrade`** mới là bước nâng cấp các gói đã cài. Trên server, nâng cấp là một việc có kế hoạch (đọc danh sách, chọn giờ, theo dõi dịch vụ sau đó), không chạy tiện tay.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-apt-cai-phan-mem/ssh-studio-01.webp" alt="Terminal của SSH Studio: nội dung kho trong ubuntu.sources với URIs archive.ubuntu.com và security.ubuntu.com, Suites noble noble-updates noble-backports noble-security, Components main universe restricted multiverse, sudo apt update kết thúc bằng thông báo về số gói nâng cấp được" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · kho gói và <code>apt update</code></figcaption></figure>

## 2. Tìm và xem gói trước khi cài

```text
an@toinoi-app-01:~$ jq --version
-bash: jq: command not found
an@toinoi-app-01:~$ apt search --names-only '^(jq|tmux|sysstat)$' 2>/dev/null | grep -v -E '^$|\.\.\.$'
jq/noble-updates,noble-security 1.7.1-3ubuntu0.24.04.2 amd64
  lightweight and flexible command-line JSON processor
sysstat/noble 12.6.1-2 amd64
  system performance tools for Linux
tmux/noble-updates 3.4-1ubuntu0.2 amd64
  terminal multiplexer
an@toinoi-app-01:~$ apt show jq 2>/dev/null | grep -E '^(Package|Version|Depends|Installed-Size|Section|Description):'
Package: jq
Version: 1.7.1-3ubuntu0.24.04.2
Section: utils
Installed-Size: 116 kB
Depends: libc6 (>= 2.38), libjq1 (= 1.7.1-3ubuntu0.24.04.2)
Description: lightweight and flexible command-line JSON processor
an@toinoi-app-01:~$ apt-cache policy tmux
tmux:
  Installed: (none)
  Candidate: 3.4-1ubuntu0.2
  Version table:
     3.4-1ubuntu0.2 500
        500 http://archive.ubuntu.com/ubuntu noble-updates/main amd64 Packages
     3.4-1build1 500
        500 http://archive.ubuntu.com/ubuntu noble/main amd64 Packages

an@toinoi-app-01:~$ sudo apt install -y jq tmux sysstat 2>&1 | grep -E -A2 '^(The following NEW|[0-9]+ upgraded)'
The following NEW packages will be installed:
  jq libevent-core-2.1-7t64 libjq1 libonig5 libsensors-config libsensors5
  libutempter0 sysstat tmux
0 upgraded, 9 newly installed, 0 to remove and 0 not upgraded.
Need to get 1483 kB of archives.
After this operation, 4226 kB of additional disk space will be used.
```

- **`apt search`** tìm theo tên và mô tả; `--names-only` với biểu thức chính quy cho kết quả gọn. Mỗi dòng ghi tên, suite chứa bản mới nhất, phiên bản, kiến trúc.
- **`apt show`** in thông tin một gói: phiên bản, phụ thuộc (`Depends`), dung lượng sau khi cài, mô tả.
- **`apt-cache policy`** cho biết bản nào đang cài (`Installed`), bản nào sẽ được cài (`Candidate`), và mỗi bản đến từ kho nào. Hữu ích khi có nhiều bản.
- **`apt install`** cài cả `jq`, `tmux` và công cụ thống kê `sysstat`, kéo theo 6 thư viện phụ thuộc. `-y` trả lời «có» sẵn; khi gõ tay, bỏ `-y` để đọc danh sách trước khi đồng ý.

Bài lọc bớt output của `apt install`; khi bạn tự chạy, apt in thêm từng bước tải (`Get:`), giải nén (`Unpacking`) và cài đặt (`Setting up`).

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-apt-cai-phan-mem/ssh-studio-02.webp" alt="Terminal của SSH Studio: cuối kết quả apt search với sysstat và tmux, apt show jq với Version, Depends, Installed-Size, apt-cache policy tmux với Installed none và Candidate, sudo apt install -y jq tmux sysstat liệt kê 9 gói mới, jq --version và tmux -V" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · tìm, xem và cài gói</figcaption></figure>

## 3. Gói đã cài: dpkg

```text
an@toinoi-app-01:~$ dpkg -l jq tmux sysstat | tail -n 3
ii  jq             1.7.1-3ubuntu0.24.04.2 amd64        lightweight and flexible command-line JSON processor
ii  sysstat        12.6.1-2               amd64        system performance tools for Linux
ii  tmux           3.4-1ubuntu0.2         amd64        terminal multiplexer
an@toinoi-app-01:~$ dpkg -L jq | grep bin/
/usr/bin/jq
an@toinoi-app-01:~$ dpkg -S /usr/bin/pstree
psmisc: /usr/bin/pstree
an@toinoi-app-01:~$ dpkg -S "$(command -v crontab)"
cron: /usr/bin/crontab
an@toinoi-app-01:~$ echo '{"dich_vu":"car4","gia":69000}' | jq .gia
69000
```

- **`dpkg -l`**: hai chữ đầu là trạng thái, `ii` = đã cài đầy đủ.
- **`dpkg -L GÓI`**: các file gói đã cài lên máy.
- **`dpkg -S ĐƯỜNG_DẪN`**: file này thuộc gói nào. Câu hỏi hay gặp: «lệnh `pstree` ở đâu ra?» → gói `psmisc`. Muốn tìm gói chứa một file **chưa cài**, dùng `apt-file` (một gói riêng).
- `jq` đọc JSON từ stdin: `jq .gia` lấy trường `gia`. Từ [LNX-036](/articles/linux-systemd-unit-bao-gia), An dùng nó với kết quả của `bao-gia`.

## 4. Gỡ gói: remove, purge, autoremove

```text
an@toinoi-app-01:~$ sudo apt remove -y sysstat 2>&1 | grep -E '^(Removing|The following packages will be REMOVED)' -A1
The following packages will be REMOVED:
  sysstat
--
Removing sysstat (12.6.1-2) ...
Processing triggers for man-db (2.12.0-4build2) ...
an@toinoi-app-01:~$ dpkg -l sysstat | tail -n 1
rc  sysstat        12.6.1-2     amd64        system performance tools for Linux
an@toinoi-app-01:~$ ls /etc/sysstat
sysstat  sysstat.ioconf
an@toinoi-app-01:~$ sudo apt purge -y sysstat 2>&1 | grep -E '^Purging'
Purging configuration files for sysstat (12.6.1-2) ...
an@toinoi-app-01:~$ dpkg -l sysstat 2>&1 | tail -n 1
dpkg-query: no packages found matching sysstat
an@toinoi-app-01:~$ ls /etc/sysstat
ls: cannot access '/etc/sysstat': No such file or directory
an@toinoi-app-01:~$ sudo apt autoremove -y 2>&1 | grep -E '^Removing'
Removing libsensors5:amd64 (1:3.6.0-9build1) ...
Removing libsensors-config (1:3.6.0-9build1) ...
```

| Lệnh | Gỡ gì | Trạng thái sau |
|---|---|---|
| `apt remove GÓI` | Chương trình, **giữ** file cấu hình trong `/etc` | `rc` (removed, config còn) |
| `apt purge GÓI` | Cả file cấu hình | không còn trong `dpkg -l` |
| `apt autoremove` | Các gói phụ thuộc không còn ai cần | |

`remove` giữ cấu hình để lần cài lại không mất những gì bạn đã chỉnh; `/etc/sysstat` vẫn còn. Gỡ hẳn thì `purge`. Sau đó `autoremove` dọn hai thư viện `libsensors` mà chỉ `sysstat` cần. Trước khi `autoremove` trên server thật, đọc danh sách: đôi khi một gói bạn cài tay bị đánh dấu nhầm là «cài tự động».

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-apt-cai-phan-mem/ssh-studio-03.webp" alt="Terminal của SSH Studio: dpkg -l với trạng thái ii, dpkg -L jq, dpkg -S /usr/bin/pstree thuộc psmisc, jq .gia in 69000, apt remove sysstat, dpkg -l sysstat trạng thái rc và /etc/sysstat còn, apt purge xoá cấu hình, autoremove gỡ libsensors" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>dpkg</code>, <code>remove</code>, <code>purge</code></figcaption></figure>

## 5. Ghim phiên bản

Đôi khi bạn **không** muốn một gói được nâng cấp: phiên bản mới chưa được kiểm thử với dịch vụ, hoặc đang chờ một bản sửa lỗi.

```text
an@toinoi-app-01:~$ sudo apt-mark hold tmux
tmux set on hold.
an@toinoi-app-01:~$ apt-mark showhold
tmux
an@toinoi-app-01:~$ apt list --upgradable 2>/dev/null | wc -l
1
an@toinoi-app-01:~$ sudo apt-mark unhold tmux
Canceled hold on tmux.
an@toinoi-app-01:~$ ls /var/log/apt/
eipp.log.xz  history.log  term.log
an@toinoi-app-01:~$ grep -E '^(Start-Date|Commandline)' /var/log/apt/history.log | tail -n 6
Start-Date: 2026-11-24  07:05:26
Commandline: apt remove -y sysstat
Start-Date: 2026-11-24  07:05:28
Commandline: apt purge -y sysstat
Start-Date: 2026-11-24  07:05:30
Commandline: apt autoremove -y
```

- **`apt-mark hold`**: `apt upgrade` sẽ bỏ qua gói này. `showhold` liệt kê các gói đang ghim; `unhold` gỡ ghim.
- Ghim là quyết định tạm thời có lý do: ghi lại vì sao và khi nào gỡ, nếu không bạn sẽ quên một gói không bao giờ nhận bản vá bảo mật.
- **`/var/log/apt/history.log`** ghi mọi lần cài, gỡ, nâng cấp: thời điểm, lệnh, gói. Khi dịch vụ bỗng hỏng, câu hỏi đầu tiên là «hôm qua có ai cài gì không?».

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-apt-cai-phan-mem/ssh-studio-04.webp" alt="Terminal của SSH Studio: sudo apt-mark hold tmux, apt-mark showhold in tmux, apt list --upgradable, sudo apt-mark unhold tmux, ls /var/log/apt, history.log với các dòng Start-Date và Commandline của apt remove, purge, autoremove" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · ghim phiên bản và lịch sử của apt</figcaption></figure>

Ubuntu còn có **bản vá bảo mật tự động** (gói `unattended-upgrades`, timer `apt-daily-upgrade` mà bạn sẽ thấy ở [LNX-038](/articles/linux-cron-systemd-timer)): mặc định chỉ cài bản vá từ `noble-security`.

## 6. Đối chiếu thao tác

| Việc | Lệnh |
|---|---|
| Cập nhật danh sách gói | `sudo apt update` |
| Nâng cấp các gói đã cài | `sudo apt upgrade` (đọc danh sách trước) |
| Tìm / xem gói | `apt search`, `apt show`, `apt-cache policy` |
| Cài / gỡ / gỡ cả cấu hình | `sudo apt install`, `remove`, `purge` |
| Dọn phụ thuộc thừa | `sudo apt autoremove` |
| Gói đã cài, file của gói | `dpkg -l`, `dpkg -L` |
| File này của gói nào | `dpkg -S /đường/dẫn` |
| Ghim / gỡ ghim | `sudo apt-mark hold`, `unhold` |
| Lịch sử | `/var/log/apt/history.log` |

## 7. Lỗi thường gặp

**`apt install` báo không tìm thấy gói.** Danh sách gói cũ hoặc chưa có: chạy `sudo apt update` trước.

**Cài phần mềm bằng script tải từ Internet (`curl … | sudo bash`).** Bạn chạy bằng quyền root một thứ chưa đọc. Ưu tiên gói trong kho của Ubuntu; nếu phải dùng kho của bên thứ ba, thêm kho đó đúng cách, có khoá ký.

**`apt upgrade` lúc đang có tải.** Nâng cấp có thể khởi động lại dịch vụ. Chọn giờ, theo dõi sau đó.

**Gỡ bằng `remove` rồi tưởng đã sạch.** Cấu hình cũ vẫn còn và được dùng lại khi cài lại. Muốn sạch: `purge`.

**`dpkg -i file.deb` thiếu phụ thuộc.** Dùng `sudo apt install ./file.deb` để apt tự kéo phụ thuộc.

## Bài tập

1. `apt show tmux` và `apt-cache depends tmux`. Gói nào là phụ thuộc bắt buộc, gói nào chỉ «khuyến nghị»?
2. Tìm gói chứa `/usr/bin/top` và `/usr/sbin/sshd` bằng `dpkg -S`.
3. Cài lại `sysstat`, `apt remove` nó, rồi `dpkg -l | grep '^rc'`. Dọn tất cả gói ở trạng thái `rc` bằng một lệnh.
4. `zcat /var/log/apt/history.log.*.gz | grep Commandline` (nếu có file cũ). Server lab đã được cài những gì lúc dựng ảnh?

## Đọc tiếp

[LNX-034 · Job nền, nohup và tmux](/articles/linux-job-nen-nohup-tmux): giữ việc chạy lâu sống sót khi phiên SSH đứt.
