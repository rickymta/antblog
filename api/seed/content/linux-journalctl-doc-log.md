---
slug: linux-journalctl-doc-log
title: "journalctl: đọc log của dịch vụ theo unit, thời gian, mức ưu tiên, theo dõi trực tiếp và giới hạn dung lượng journal"
summary: "Đọc log của bao-gia trong journal: lọc theo unit, số dòng, khoảng thời gian, định dạng short-iso và cat, mức ưu tiên và cách một chương trình báo mức bằng tiền tố <N>, lần theo một lỗi thật khi reload, các trường của một bản ghi (json-pretty, _UID, _PID, _SYSTEMD_UNIT), journalctl -f trong SSH Studio, và giới hạn dung lượng bằng drop-in của journald, vacuum."
category: devops-linux
level: intermediate
tags: [linux, lnx-037, journalctl, journald, systemd, log, priority, ssh-studio, toi-noi]
language: vi
---

`bao-gia` đã chạy bằng systemd ([LNX-036](/articles/linux-systemd-unit-bao-gia)). Trước đây, log của nó là một file trong `/var/log/toinoi`; giờ log vòng đời và lỗi đi vào **journal** của systemd, chung chỗ với log của `ssh`, `cron`, `sudo` và mọi dịch vụ khác. Một chỗ cho tất cả thì tiện, với điều kiện biết cách lọc ra đúng thứ mình cần. Đó là việc của `journalctl`.

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-037 · Phần 4/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/linux-systemd-unit-bao-gia">LNX-036</a>. Trạng thái đầu bài: <code>npm run setup -- 37</code>. Cần thuộc nhóm <code>systemd-journal</code> (<a href="/articles/linux-systemd-systemctl">LNX-035</a>); nếu không, dùng <code>sudo journalctl</code>.</p>
</div>

## 1. Log của một unit

Mọi thứ một dịch vụ in ra stdout và stderr, systemd chuyển vào **systemd-journald**. Journald lưu mỗi dòng thành một bản ghi có cấu trúc (thời điểm, unit, PID, người dùng, mức ưu tiên…), ở dạng nhị phân trong `/var/log/journal`. Bạn không đọc file đó bằng `cat`; bạn hỏi bằng `journalctl`.

```text
an@toinoi-app-01:~$ journalctl -u bao-gia --no-pager | head -n 8
Nov 24 07:05:47 toinoi-app-01 systemd[1]: Starting bao-gia.service - bao-gia: dich vu bao gia cua Toi Noi...
Nov 24 07:05:47 toinoi-app-01 kiem-tra-cau-hinh.sh[3788]: bao-gia: bang gia co 4 loai dich vu
Nov 24 07:05:47 toinoi-app-01 kiem-tra-cau-hinh.sh[3788]: bao-gia: cau hinh hop le
Nov 24 07:05:47 toinoi-app-01 systemd[1]: bao-gia.service: Main process exited, code=exited, status=203/EXEC
Nov 24 07:05:47 toinoi-app-01 systemd[1]: bao-gia.service: Failed with result 'exit-code'.
Nov 24 07:05:47 toinoi-app-01 systemd[1]: Failed to start bao-gia.service - bao-gia: dich vu bao gia cua Toi Noi.
Nov 24 07:05:47 toinoi-app-01 systemd[1]: Stopped bao-gia.service - bao-gia: dich vu bao gia cua Toi Noi.
Nov 24 07:05:47 toinoi-app-01 systemd[1]: Starting bao-gia.service - bao-gia: dich vu bao gia cua Toi Noi...
an@toinoi-app-01:~$ journalctl -u bao-gia --no-pager -n 3
Nov 24 07:05:52 toinoi-app-01 systemd[1]: Reloading bao-gia.service - bao-gia: dich vu bao gia cua Toi Noi...
Nov 24 07:05:52 toinoi-app-01 bao-gia[3859]: INFO nhan SIGHUP: mo lai file log, nap lai bang gia (4 loai dich vu)
Nov 24 07:05:52 toinoi-app-01 systemd[1]: Reloaded bao-gia.service - bao-gia: dich vu bao gia cua Toi Noi.
an@toinoi-app-01:~$ journalctl -u bao-gia --no-pager --since '5 min ago' -o short-iso | tail -n 3
2026-11-24T07:05:52+07:00 toinoi-app-01 systemd[1]: Reloading bao-gia.service - bao-gia: dich vu bao gia cua Toi Noi...
2026-11-24T07:05:52+07:00 toinoi-app-01 bao-gia[3859]: INFO nhan SIGHUP: mo lai file log, nap lai bang gia (4 loai dich vu)
2026-11-24T07:05:52+07:00 toinoi-app-01 systemd[1]: Reloaded bao-gia.service - bao-gia: dich vu bao gia cua Toi Noi.
```

| Tuỳ chọn | Tác dụng |
|---|---|
| `-u TÊN` | Chỉ log của một unit |
| `-n N` | N dòng cuối |
| `--since`, `--until` | Khoảng thời gian: `'5 min ago'`, `'2026-11-24 07:00'`, `today`, `yesterday` |
| `-o short-iso` | Thời điểm dạng ISO, có múi giờ, dễ sắp và so |
| `-o cat` | Chỉ nội dung, bỏ thời gian và tên |
| `-r` | Mới nhất lên đầu |
| `--no-pager` | In thẳng ra terminal thay vì mở [pager](/articles/linux-cay-thu-muc-duong-dan) |
| `-b` | Chỉ từ lần boot này (`-b -1`: lần trước) |

## 2. Mức ưu tiên

Mỗi bản ghi có một **mức ưu tiên** theo chuẩn syslog, từ 0 (nghiêm trọng nhất) tới 7:

| Số | Tên | Dùng khi |
|---|---|---|
| 3 | `err` | Lỗi |
| 4 | `warning` | Cảnh báo |
| 6 | `info` | Thông tin bình thường |
| 7 | `debug` | Gỡ lỗi |

(0–2 là `emerg`, `alert`, `crit`, dành cho sự cố cấp hệ thống.)

Một chương trình chạy dưới systemd báo mức cho từng dòng bằng cách bắt đầu dòng bằng **`<N>`**: `bao-gia` in `<4>WARN …` cho yêu cầu không hợp lệ, `<3>LOI …` cho lỗi. Journald đọc tiền tố, ghi mức, rồi bỏ nó khỏi nội dung.

```text
an@toinoi-app-01:~$ curl -s 'http://127.0.0.1:8080/bao-gia?dich_vu=xe-tai&km=5'
{"loi": "dich_vu hoac km khong hop le"}
an@toinoi-app-01:~$ curl -s 'http://127.0.0.1:8080/bao-gia?dich_vu=bike&km=abc'
{"loi": "dich_vu hoac km khong hop le"}
an@toinoi-app-01:~$ journalctl -u bao-gia --no-pager -p warning -o short-iso
2026-11-24T07:05:47+07:00 toinoi-app-01 systemd[1]: bao-gia.service: Failed with result 'exit-code'.
2026-11-24T07:05:47+07:00 toinoi-app-01 systemd[1]: Failed to start bao-gia.service - bao-gia: dich vu bao gia cua Toi Noi.
2026-11-24T07:05:48+07:00 toinoi-app-01 systemd[1]: bao-gia.service: Main process exited, code=killed, status=9/KILL
2026-11-24T07:05:48+07:00 toinoi-app-01 systemd[1]: bao-gia.service: Failed with result 'signal'.
2026-11-24T07:05:53+07:00 toinoi-app-01 bao-gia[3859]: WARN yeu cau khong hop le: dich_vu=xe-tai km=5
2026-11-24T07:05:53+07:00 toinoi-app-01 bao-gia[3859]: WARN yeu cau khong hop le: dich_vu=bike km=abc
```

**`-p warning`** lấy mức `warning` **và nghiêm trọng hơn**. Có cả dòng của chính systemd (dịch vụ không lên, bị giết bằng `SIGKILL` ở bài trước) lẫn hai yêu cầu sai của `bao-gia`. Khi sự cố xảy ra, `-p err` hay `-p warning` cắt bỏ hàng trăm dòng `info` để còn lại thứ đáng đọc. Trên terminal (như trong ảnh), `journalctl` còn tô màu theo mức: cảnh báo màu vàng, lỗi màu đỏ.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-journalctl-doc-log/ssh-studio-01.webp" alt="Terminal của SSH Studio: journalctl -u bao-gia với các dòng khởi động, -n 3 với Reloading, nhan SIGHUP, Reloaded, --since 5 min ago -o short-iso, hai lệnh curl trả JSON lỗi, journalctl -p warning liệt kê dòng Failed, status=9/KILL và hai dòng WARN yeu cau khong hop le" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · lọc log theo unit, thời gian và mức ưu tiên</figcaption></figure>

## 3. Lần theo một lỗi thật

Lúc cập nhật bảng giá, có người xoá nhầm file rồi `reload`:

```text
an@toinoi-app-01:~$ mv /srv/toinoi/bao-gia/releases/1.5.0/bang-gia.csv /tmp/bang-gia.csv
an@toinoi-app-01:~$ sudo systemctl reload bao-gia
an@toinoi-app-01:~$ journalctl -u bao-gia --no-pager -p err -n 1 -o short-iso
2026-11-24T07:05:53+07:00 toinoi-app-01 bao-gia[3859]: LOI  nhan SIGHUP: khong doc duoc bang gia /srv/toinoi/bao-gia/current/bang-gia.csv: No such file or directory; giu bang gia cu
an@toinoi-app-01:~$ curl -s http://127.0.0.1:8080/suc-khoe
{"trang_thai": "ok", "phien_ban": "1.5.0", "pid": 3859, "moi_truong": "production", "so_dich_vu": 4}
an@toinoi-app-01:~$ mv /tmp/bang-gia.csv /srv/toinoi/bao-gia/releases/1.5.0/bang-gia.csv
an@toinoi-app-01:~$ sudo systemctl reload bao-gia
an@toinoi-app-01:~$ journalctl -u bao-gia --no-pager -n 2 -o cat
INFO nhan SIGHUP: mo lai file log, nap lai bang gia (4 loai dich vu)
Reloaded bao-gia.service - bao-gia: dich vu bao gia cua Toi Noi.
```

- `journalctl -p err -n 1` chỉ ra ngay: `bao-gia` không đọc được bảng giá khi nhận `SIGHUP`, và **giữ bảng giá cũ** trong bộ nhớ.
- `/suc-khoe` vẫn trả `ok` với 4 loại dịch vụ: dịch vụ không sập. Một chương trình reload tốt thì nạp cấu hình mới thất bại không làm hỏng cấu hình đang chạy, nhưng lỗi phải hiện trong log, nếu không sẽ không ai biết bảng giá mới chưa được áp dụng.
- Trả file về, `reload` lại: nạp thành công.

## 4. Một bản ghi có những trường gì

```text
an@toinoi-app-01:~$ journalctl --no-pager -n 1 -o json-pretty SYSLOG_IDENTIFIER=bao-gia | grep -E '"(MESSAGE|PRIORITY|_PID|_UID|_SYSTEMD_UNIT|_COMM|SYSLOG_IDENTIFIER)"'
        "PRIORITY" : "6",
        "MESSAGE" : "INFO nhan SIGHUP: mo lai file log, nap lai bang gia (4 loai dich vu)",
        "_SYSTEMD_UNIT" : "bao-gia.service",
        "SYSLOG_IDENTIFIER" : "bao-gia",
        "_PID" : "3859",
        "_UID" : "990",
        "_COMM" : "bao-gia",
an@toinoi-app-01:~$ journalctl --no-pager _UID=990 -n 2 -o cat
LOI  nhan SIGHUP: khong doc duoc bang gia /srv/toinoi/bao-gia/current/bang-gia.csv: No such file or directory; giu bang gia cu
INFO nhan SIGHUP: mo lai file log, nap lai bang gia (4 loai dich vu)
```

Với `-o json-pretty`, mỗi bản ghi là một tập trường. Trường bắt đầu bằng `_` do **journald tự ghi**, chương trình không giả mạo được: `_PID`, `_UID` (990 là `baogia`), `_SYSTEMD_UNIT`, `_COMM` (tên tiến trình). Lọc theo bất kỳ trường nào bằng `TRƯỜNG=giá trị`: `_UID=990` lấy mọi log do `baogia` in ra, ở bất kỳ unit nào.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-journalctl-doc-log/ssh-studio-02.webp" alt="Terminal của SSH Studio: mv bảng giá sang /tmp, reload, journalctl -p err với dòng LOI nhan SIGHUP khong doc duoc bang gia, suc-khoe vẫn ok với so_dich_vu 4, trả file về và reload thành công, json-pretty với PRIORITY 6, MESSAGE, _SYSTEMD_UNIT bao-gia.service, _UID 990, _COMM bao-gia, journalctl _UID=990" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · một lỗi thật khi reload, và các trường của bản ghi</figcaption></figure>

## 5. Theo dõi trực tiếp

**`journalctl -f`** (*follow*) in các dòng cuối rồi đứng chờ, dòng mới hiện ra ngay khi được ghi, giống `tail -f` ([LNX-006](/articles/linux-xem-sua-file-van-ban)). Bấm Ctrl+C để thoát.

```text
an@toinoi-app-01:~$ (sleep 2; curl -s -o /dev/null 'http://127.0.0.1:8080/bao-gia?dich_vu=tau-thuy&km=1') &
an@toinoi-app-01:~$ timeout 4 journalctl -u bao-gia -f -n 0 -o short-iso
2026-11-24T07:05:57+07:00 toinoi-app-01 bao-gia[3859]: WARN yeu cau khong hop le: dich_vu=tau-thuy km=1
```

Bản ghi của lab dùng `timeout 4` để lệnh tự dừng. Trong SSH Studio, An để `journalctl -u bao-gia -f` chạy trong một tab và gửi yêu cầu từ nơi khác; mỗi yêu cầu sai hiện thành một dòng `WARN` ngay lập tức:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-journalctl-doc-log/ssh-studio-03.webp" alt="journalctl -u bao-gia -f đang chạy trong terminal của SSH Studio: ba dòng cuối của log, rồi các dòng WARN yeu cau khong hop le mới hiện ra màu vàng cho dich_vu=tau-thuy, xe-tai và km=-1" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>journalctl -f</code> theo dõi trực tiếp</figcaption></figure>

Mẹo trong SSH Studio: chế độ **Chia đôi** cho bạn một nửa màn hình chạy `journalctl -f`, nửa kia gõ lệnh, cùng một server.

## 6. Dung lượng và giới hạn

```text
an@toinoi-app-01:~$ journalctl --disk-usage
Archived and active journals take up 32.0M in the file system.
an@toinoi-app-01:~$ ls /var/log/journal/
a1c04d0cce0c46a29f655a7866f7be17
an@toinoi-app-01:~$ journalctl --list-boots --no-pager | tail -n 2
IDX BOOT ID                          FIRST ENTRY                 LAST ENTRY
  0 e977c82074ef493ea1e57f419d0d7857 Tue 2026-11-24 07:03:34 +07 Tue 2026-11-24 07:05:57 +07
an@toinoi-app-01:~$ sudo mkdir -p /etc/systemd/journald.conf.d
an@toinoi-app-01:~$ sudoedit /etc/systemd/journald.conf.d/50-toinoi.conf
an@toinoi-app-01:~$ sudo systemctl restart systemd-journald
an@toinoi-app-01:~$ systemd-analyze cat-config systemd/journald.conf | grep -v '^#' | grep -v '^$'
[Journal]
[Journal]
SystemMaxUse=200M
MaxRetentionSec=30day
[Journal]
ForwardToSyslog=yes
an@toinoi-app-01:~$ sudo journalctl --vacuum-time=30d
Vacuuming done, freed 0B of archived journals from /var/log/journal/a1c04d0cce0c46a29f655a7866f7be17.
Vacuuming done, freed 0B of archived journals from /var/log/journal.
Vacuuming done, freed 0B of archived journals from /run/log/journal.
```

- **`--disk-usage`**: journal đang chiếm bao nhiêu.
- `/var/log/journal/` có một thư mục theo mã máy: journal được lưu **bền** qua các lần khởi động lại (`--list-boots` liệt kê các lần boot). Nếu thư mục này không có, journal chỉ ở `/run/log/journal` và mất khi tắt máy.
- Mặc định journald tự giới hạn ở 10% dung lượng hệ thống file (tối đa 4 GB). An đặt giới hạn rõ ràng bằng một **drop-in**, file trong `journald.conf.d/` ghi đè đúng những dòng cần đổi, không sửa file gốc:

```ini
# Giới hạn journal của toinoi-app-01 (LNX-037): tối đa 200 MB trên đĩa, giữ không quá 30 ngày.
[Journal]
SystemMaxUse=200M
MaxRetentionSec=30day
```

- **`systemd-analyze cat-config`** in cấu hình gộp từ mọi file, theo đúng thứ tự áp dụng.
- **`journalctl --vacuum-time=30d`** (hoặc `--vacuum-size=100M`) dọn ngay các file journal cũ đã đóng, không đợi giới hạn.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-journalctl-doc-log/ssh-studio-04.webp" alt="Terminal của SSH Studio: journalctl --disk-usage, ls /var/log/journal, list-boots, mkdir journald.conf.d, sudoedit 50-toinoi.conf, restart systemd-journald, systemd-analyze cat-config với SystemMaxUse=200M và MaxRetentionSec=30day, vacuum-time=30d" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · giới hạn dung lượng journal</figcaption></figure>

## 7. Đối chiếu thao tác

| Việc | Lệnh |
|---|---|
| Log của một dịch vụ | `journalctl -u TÊN` (`-n 50`, `--no-pager`) |
| Trong khoảng thời gian | `journalctl --since '1 hour ago'` |
| Chỉ lỗi và cảnh báo | `journalctl -p warning` |
| Theo dõi | `journalctl -u TÊN -f` |
| Từ lần boot này / trước | `journalctl -b`, `journalctl -b -1` |
| Theo người dùng, PID | `journalctl _UID=990`, `_PID=1234` |
| Dung lượng, dọn | `journalctl --disk-usage`, `sudo journalctl --vacuum-time=30d` |
| Ghi một dòng vào journal từ script | `logger -p user.warning "nội dung"` |

## 8. Lỗi thường gặp

**`-- No entries --` dù dịch vụ có log.** Thiếu quyền: thêm vào nhóm `systemd-journal`, đăng nhập lại.

**Đọc log trong pager rồi tưởng bị treo.** `journalctl` mở `less` khi output dài; `q` để thoát, hoặc `--no-pager`.

**Ghi log ra file riêng rồi quên xoay vòng.** Log trong journal có giới hạn sẵn; file log của bạn thì không, cho tới [LNX-039](/articles/linux-logrotate).

**In mọi thứ ở mức `info`.** Lỗi chìm giữa hàng nghìn dòng. Cho chương trình báo đúng mức (`<3>`, `<4>`), hoặc ghi bằng thư viện log có mức.

**Tưởng journal là kho log lâu dài.** Nó có giới hạn và nằm trên chính server. Log cần giữ lâu hoặc phân tích chung nhiều server thì chuyển đi nơi khác (Phần 6).

## Bài tập

1. `journalctl -u ssh --since today -p notice`. Bạn thấy những sự kiện đăng nhập nào?
2. `logger -t thu-nghiem -p user.err "loi gia lap"`, rồi tìm nó bằng `journalctl -t thu-nghiem` và `-p err`.
3. `journalctl -u bao-gia -o json | jq -r '.MESSAGE' | tail -n 5`. Khi nào bạn cần JSON thay vì văn bản?
4. Đổi `MaxRetentionSec=30day` thành `7day`, `restart systemd-journald`, `--vacuum-time=7d`. `--disk-usage` thay đổi thế nào? Đặt lại 30 ngày.

## Đọc tiếp

[LNX-038 · Lên lịch: cron và systemd timer](/articles/linux-cron-systemd-timer): việc chạy theo giờ, và chuyện gì xảy ra với việc bị lỡ khi server tắt.
