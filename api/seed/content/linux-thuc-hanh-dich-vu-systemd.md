---
slug: linux-thuc-hanh-dich-vu-systemd
title: "Thực hành: bàn giao bao-gia thành dịch vụ systemd hoàn chỉnh, siết bằng drop-in và bộ chấm có khởi động lại server"
summary: "Bài thực hành cuối Phần 4: siết unit của bao-gia bằng một drop-in (ProtectSystem=strict, ProtectHome, PrivateTmp, ReadWritePaths, NoNewPrivileges, MemoryMax), đo bằng systemd-analyze security, kiểm chứng dịch vụ vẫn làm đúng việc, tài khoản deploy của CI khởi động lại được, bảng kiểm tra bàn giao, và bộ chấm 26 mục có giết tiến trình và khởi động lại cả server."
category: devops-linux
level: advanced
tags: [linux, lnx-040, systemd, drop-in, hardening, protectsystem, thuc-hanh, ssh-studio, toi-noi]
language: vi
---

Cuối tuần, Bình duyệt lại `bao-gia` trước khi bàn giao cho nhóm vận hành: dịch vụ chạy bằng systemd, có log trong journal, tự đứng dậy khi chết, có timer dọn báo cáo, log được xoay vòng. Còn một câu Bình luôn hỏi: «Nếu `bao-gia` có lỗ hổng và bị chiếm, kẻ tấn công làm được gì trên server?» Hiện tại câu trả lời là: mọi thứ mà `baogia` làm được, tức đọc được `/home`, ghi được mọi chỗ mà nhóm `toinoi` ghi được. Bài này thu hẹp con số đó, rồi kiểm tra toàn bộ Phần 4 bằng một bộ chấm.

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-040 · Phần 4/6 · Nâng cao · Thực hành</span>
<p>Trạng thái đầu bài: <code>npm run setup -- 40</code> (đã có unit, timer, logrotate, giới hạn journal của các bài trước). Thử tự làm theo yêu cầu trước khi đọc lời giải. Cuối bài, <code>npm run kiem-tra:p04</code> chấm toàn bộ Phần 4; thêm <code>-- --boot</code> để bộ chấm khởi động lại cả server rồi kiểm tra lại.</p>
</div>

## Yêu cầu

1. `bao-gia` chỉ ghi được vào thư mục log của nó; toàn bộ hệ thống file còn lại chỉ đọc; không thấy thư mục home của ai; có `/tmp` riêng; không leo quyền được; dùng tối đa 200 MB RAM. **Không sửa file unit gốc.**
2. Dịch vụ vẫn trả lời, vẫn ghi log yêu cầu, vẫn `reload` được.
3. Tài khoản `deploy` của CI ([LNX-030](/articles/linux-thuc-hanh-khoa-dang-nhap-ssh)) khởi động lại được `bao-gia`, không làm được gì hơn.
4. Sau khi khởi động lại cả server: `bao-gia` tự chạy, timer dọn báo cáo vẫn chờ.

## 1. Siết bằng drop-in

**Drop-in** là file `.conf` trong thư mục `TÊN.service.d/`, ghi đè hoặc thêm vào đúng những dòng cần đổi của unit gốc, giống `journald.conf.d` ở [LNX-037](/articles/linux-journalctl-doc-log). Unit gốc giữ nguyên; muốn bỏ thay đổi thì xoá một file. (`sudo systemctl edit bao-gia` tạo và mở đúng file drop-in đó.)

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-thuc-hanh-dich-vu-systemd/ssh-studio-01.webp" alt="Vim mở /etc/systemd/system/bao-gia.service.d/50-an-toan.conf qua sudoedit trong SSH Studio, mục [Service] với NoNewPrivileges=yes, PrivateTmp=yes, ProtectSystem=strict, ProtectHome=yes, ReadWritePaths=/var/log/toinoi, ProtectKernelTunables=yes, ProtectControlGroups=yes, RestrictSUIDSGID=yes, MemoryMax=200M" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · drop-in siết <code>bao-gia</code></figcaption></figure>

```ini
# Siết bao-gia (LNX-040): hệ thống file chỉ đọc trừ thư mục log, không thấy /home,
# /tmp riêng, không leo quyền, giới hạn bộ nhớ.
[Service]
NoNewPrivileges=yes
PrivateTmp=yes
ProtectSystem=strict
ProtectHome=yes
ReadWritePaths=/var/log/toinoi
ProtectKernelTunables=yes
ProtectControlGroups=yes
RestrictSUIDSGID=yes
MemoryMax=200M
```

| Dòng | Tác dụng |
|---|---|
| `ProtectSystem=strict` | Toàn bộ hệ thống file chỉ đọc với dịch vụ… |
| `ReadWritePaths=/var/log/toinoi` | …trừ đúng thư mục này |
| `ProtectHome=yes` | `/home`, `/root` như không tồn tại |
| `PrivateTmp=yes` | `/tmp` riêng, không thấy file tạm của người khác |
| `NoNewPrivileges=yes` | Không leo quyền qua chương trình setuid ([LNX-026](/articles/linux-setuid-setgid-sticky)) |
| `ProtectKernelTunables`, `ProtectControlGroups` | Không sửa tham số kernel, cgroup |
| `RestrictSUIDSGID=yes` | Không tạo được file setuid/setgid |
| `MemoryMax=200M` | Vượt thì kernel giết dịch vụ (rồi `Restart=` đưa nó dậy) |

systemd thực hiện những điều này bằng **namespace** của kernel: dịch vụ nhìn thấy một cây thư mục riêng, đã gắn chỉ-đọc, mà không cần sửa một dòng code nào của ứng dụng.

```text
an@toinoi-app-01:~$ systemd-analyze security bao-gia --no-pager | tail -n 1
→ Overall exposure level for bao-gia.service: 9.2 UNSAFE 😨
an@toinoi-app-01:~$ sudo mkdir -p /etc/systemd/system/bao-gia.service.d
an@toinoi-app-01:~$ sudoedit /etc/systemd/system/bao-gia.service.d/50-an-toan.conf
an@toinoi-app-01:~$ sudo systemctl daemon-reload
an@toinoi-app-01:~$ sudo systemctl restart bao-gia
an@toinoi-app-01:~$ systemctl status bao-gia --no-pager | head -n 5
● bao-gia.service - bao-gia: dich vu bao gia cua Toi Noi
     Loaded: loaded (/etc/systemd/system/bao-gia.service; enabled; preset: enabled)
    Drop-In: /etc/systemd/system/bao-gia.service.d
             └─50-an-toan.conf
     Active: active (running) since Tue 2026-11-24 07:06:01 +07; 1s ago
an@toinoi-app-01:~$ systemd-analyze security bao-gia --no-pager | tail -n 1
→ Overall exposure level for bao-gia.service: 7.8 EXPOSED 🙁
an@toinoi-app-01:~$ systemctl show bao-gia --no-pager -p ProtectSystem -p ProtectHome -p PrivateTmp -p MemoryMax -p ReadWritePaths
MemoryMax=209715200
ReadWritePaths=/var/log/toinoi
PrivateTmp=yes
ProtectHome=yes
ProtectSystem=strict
```

- `status` có thêm dòng **`Drop-In:`**: dấu hiệu luôn cần nhìn khi đọc một dịch vụ lạ, vì cấu hình thật là unit gốc **cộng** các drop-in.
- **`systemd-analyze security`** chấm mức phơi nhiễm từ 0 (chặt) tới 10 (mở): từ 9.2 xuống 7.8. Đừng chạy theo con số: mỗi tuỳ chọn có thể làm hỏng một ứng dụng cụ thể, nên thêm dần, thử từng bước. Lệnh không có `tail` liệt kê từng mục còn hở, là danh sách việc tiếp theo.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-thuc-hanh-dich-vu-systemd/ssh-studio-02.webp" alt="Terminal của SSH Studio: systemd-analyze security 9.2 UNSAFE, mkdir bao-gia.service.d, sudoedit 50-an-toan.conf, daemon-reload, restart, status với dòng Drop-In 50-an-toan.conf, security 7.8 EXPOSED, systemctl show ProtectSystem=strict, ProtectHome=yes, PrivateTmp=yes, MemoryMax=209715200, ReadWritePaths=/var/log/toinoi" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · drop-in và điểm của <code>systemd-analyze security</code></figcaption></figure>

## 2. Dịch vụ vẫn làm đúng việc của nó

Siết mà làm hỏng dịch vụ thì vô ích. Kiểm tra cả ba đường: trả lời, ghi log (thư mục duy nhất được ghi), reload:

```text
an@toinoi-app-01:~$ curl -s http://127.0.0.1:8080/suc-khoe | jq -c .
{"trang_thai":"ok","phien_ban":"1.5.0","pid":4224,"moi_truong":"production","so_dich_vu":4}
an@toinoi-app-01:~$ curl -s 'http://127.0.0.1:8080/bao-gia?dich_vu=delivery&km=7'
{"dich_vu": "delivery", "km": 7.0, "gia": 31000}
an@toinoi-app-01:~$ tail -n 1 /var/log/toinoi/bao-gia.log
2026-11-24 07:06:02 INFO  q-100001 dich_vu=delivery km=7 gia=31000 0 ms
an@toinoi-app-01:~$ sudo systemctl reload bao-gia
an@toinoi-app-01:~$ journalctl -u bao-gia --no-pager -n 2 -o cat
INFO nhan SIGHUP: mo lai file log, nap lai bang gia (4 loai dich vu)
Reloaded bao-gia.service - bao-gia: dich vu bao gia cua Toi Noi.

an@toinoi-app-01:~$ sudo -u deploy sudo -n /usr/bin/systemctl restart bao-gia && echo "deploy: restart duoc"
deploy: restart duoc
an@toinoi-app-01:~$ sudo -u deploy sudo -n /usr/bin/systemctl stop bao-gia || echo "deploy: stop bi tu choi"
sudo: a password is required
deploy: stop bi tu choi
```

- Báo giá giao hàng 7 km ra 31 000 đồng, dòng log mới được ghi. Mã yêu cầu bắt đầu lại từ `q-100001` vì bộ đếm nằm trong bộ nhớ của tiến trình, và dịch vụ vừa được khởi động lại.
- **`deploy`** khởi động lại được (đúng quy tắc sudoers của [LNX-030](/articles/linux-thuc-hanh-khoa-dang-nhap-ssh)), nhưng **dừng** thì không: CI không làm sập được dịch vụ, cả khi bị chiếm.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-thuc-hanh-dich-vu-systemd/ssh-studio-03.webp" alt="Terminal của SSH Studio: curl suc-khoe qua jq -c với trang_thai ok, phien_ban 1.5.0, báo giá delivery 7 km 31000, tail log q-100001, reload và journal nhan SIGHUP, deploy restart duoc, deploy stop bị từ chối a password is required, is-active active" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · dịch vụ vẫn chạy đúng, <code>deploy</code> chỉ khởi động lại được</figcaption></figure>

## 3. Bảng kiểm tra bàn giao

```text
an@toinoi-app-01:~$ systemctl is-enabled bao-gia bao-gia-don-bao-cao.timer logrotate.timer
enabled
enabled
enabled
an@toinoi-app-01:~$ systemctl is-active bao-gia bao-gia-don-bao-cao.timer logrotate.timer
active
active
active
an@toinoi-app-01:~$ systemctl show bao-gia --no-pager -p User -p Restart -p NRestarts
Restart=on-failure
NRestarts=0
User=baogia
an@toinoi-app-01:~$ sudo logrotate -d /etc/logrotate.d/bao-gia 2>&1 | grep -ci error
0
an@toinoi-app-01:~$ journalctl --disk-usage
Archived and active journals take up 32.0M in the file system.
an@toinoi-app-01:~$ systemctl --failed --no-pager
  UNIT LOAD ACTIVE SUB DESCRIPTION

0 loaded units listed.
```

`NRestarts=0` ở đây vì `restart` bằng tay đặt lại bộ đếm; bộ chấm sẽ tự giết tiến trình để thử `Restart=`.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-thuc-hanh-dich-vu-systemd/ssh-studio-04.webp" alt="Terminal của SSH Studio: is-enabled và is-active của bao-gia, bao-gia-don-bao-cao.timer, logrotate.timer đều enabled và active, show User=baogia Restart=on-failure NRestarts=0, logrotate -d đếm 0 lỗi, journalctl --disk-usage, systemctl --failed không có unit lỗi" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · bảng kiểm tra trước khi bàn giao</figcaption></figure>

## 4. Chấm cả Phần 4

Trên máy của bạn, trong `examples/toi-noi-linux`:

```bash
npm run kiem-tra:p04
npm run kiem-tra:p04 -- --boot    # thêm bước khởi động lại cả server
```

| Nhóm | Bộ chấm kiểm tra |
|---|---|
| Unit | `active`, `enabled`, `User=baogia`, `Group=toinoi`, `Restart=on-failure`, hai `EnvironmentFile=`, `ExecReload` gửi SIGHUP, `current` → `1.5.0` |
| Drop-in | `ProtectSystem=strict`, `ProtectHome`, `PrivateTmp`, ghi được `/var/log/toinoi`, có `MemoryMax` |
| Chạy thật | `/suc-khoe` trả `ok` 1.5.0; báo giá ô tô 4 chỗ 12,5 km ra 69 000; yêu cầu được ghi log |
| Tự hồi phục | Giết tiến trình chính bằng `SIGKILL`, sau 4 giây có tiến trình mới |
| reload, logrotate | Ứng dụng nhận SIGHUP; `logrotate -d` không lỗi; cấu hình có `daily`, `rotate 14`, `compress`, `reload` |
| Timer, journal | Timer bật và chờ, 02:30, `Persistent=yes`; service dọn dẹp `oneshot` bằng `baogia`; `SystemMaxUse`; `an` thuộc `systemd-journal` |
| CI | `deploy` khởi động lại được bằng `sudo`, không cần mật khẩu |
| `--boot` | Sau `docker restart`, `bao-gia` tự chạy và trả lời, timer vẫn chờ |

Tất cả `PASS` (26 mục với `--boot`) là xong Phần 4. Mục `FAIL` có kèm giá trị thật đọc được trên server.

## 5. Danh sách kiểm tra cho mọi dịch vụ mới

- [ ] Chạy bằng tài khoản dịch vụ riêng, không `root` ([LNX-021](/articles/linux-quan-ly-nguoi-dung-nhom), [LNX-036](/articles/linux-systemd-unit-bao-gia)).
- [ ] Cấu hình và bí mật trong `EnvironmentFile=` quyền chặt, không nằm trong unit.
- [ ] `ExecStartPre=` kiểm tra cấu hình; `Type=exec` để lỗi khởi động báo ngay.
- [ ] `Restart=on-failure`; thử bằng `kill -9` một lần.
- [ ] `reload` có nghĩa (SIGHUP) nếu ứng dụng hỗ trợ.
- [ ] Log báo đúng mức (`<3>`, `<4>`); đọc được bằng `journalctl -u` ([LNX-037](/articles/linux-journalctl-doc-log)).
- [ ] File log riêng có logrotate và `postrotate` ([LNX-039](/articles/linux-logrotate)).
- [ ] Việc định kỳ là timer có `Persistent=true` ([LNX-038](/articles/linux-cron-systemd-timer)).
- [ ] Drop-in siết quyền, kiểm bằng `systemd-analyze security`, thử dịch vụ sau mỗi bước.
- [ ] Khởi động lại cả server ít nhất một lần trước khi bàn giao.

## 6. Lỗi thường gặp

**Siết một lúc mười tuỳ chọn rồi dịch vụ không lên.** Thêm từng nhóm, `restart`, thử. Lỗi thường gặp nhất: `ProtectSystem=strict` mà quên `ReadWritePaths=` cho thư mục dịch vụ cần ghi (log, cache, socket).

**Sửa unit gốc của gói phần mềm.** Lần cập nhật gói sau sẽ ghi đè. Luôn dùng drop-in.

**Quên `daemon-reload` sau khi thêm drop-in.** `status` sẽ cảnh báo, và dịch vụ chạy theo cấu hình cũ.

**Chưa bao giờ khởi động lại server.** Dịch vụ quên `enable`, timer quên `[Install]`, mount quên `/etc/fstab`: tất cả chỉ lộ ra lúc 3 giờ sáng khi nhà cung cấp khởi động lại máy.

## Bài tập

1. Thêm `ReadOnlyPaths=/srv/toinoi/bao-gia` (thừa với `strict`, nhưng ghi rõ ý định) và `InaccessiblePaths=/srv/toinoi/bao-gia/scripts`. `restart`, kiểm tra dịch vụ còn chạy. Bằng cách nào chứng minh `baogia` không đọc được `scripts` nữa? (Gợi ý: `sudo systemd-run -p User=baogia -p InaccessiblePaths=… --wait ls …`.)
2. Đọc danh sách đầy đủ của `systemd-analyze security bao-gia`. Chọn ba mục hở, thêm vào drop-in, thử lại. Điểm còn bao nhiêu?
3. Đặt `MemoryMax=20M`, `restart`, gửi vài yêu cầu. Journal ghi gì? (`oom-kill`?) Đặt lại 200M.
4. Chạy `npm run kiem-tra:p04 -- --boot`. Trong lúc server khởi động lại, SSH Studio của bạn báo gì? Kết nối lại cần những gì (gợi ý: mã OTP)?

## Đọc tiếp

Phần 4 khép lại với một dịch vụ chạy đúng cách trên một server. **Phần 5** mở rộng ra ngoài máy: mạng và cổng, tường lửa, truy cập dịch vụ nội bộ bằng port forwarding của SSH Studio, đĩa và giám sát tài nguyên. Bắt đầu với [LNX-041 · Địa chỉ IP, cổng và ss: dịch vụ đang nghe ở đâu](/articles/linux-ip-cong-ss).
