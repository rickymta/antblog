---
slug: linux-thuc-hanh-runbook-van-hanh
title: "Thực hành: runbook vận hành toinoi-app-01, một lệnh cho mọi bằng chứng, một tài liệu cho mọi sự cố"
summary: "Bài thực hành cuối series: viết script kiem-tra-van-hanh gom mọi kiểm tra của Phần 4–6 (dịch vụ, HTTPS, đĩa và inode, hạn chứng chỉ, fail2ban, vá tự động, khởi động lại, sao lưu trên app02) thành các dòng OK/CANH BAO/LOI với mã thoát, viết runbook nằm cạnh server, diễn tập một sự cố theo runbook, và bộ chấm 24 mục cho cả Phần 6."
category: devops-linux
level: advanced
tags: [linux, lnx-060, runbook, operations, monitoring, incident, thuc-hanh, ssh-studio, toi-noi]
language: vi
---

Tháng sau, An nghỉ phép hai tuần, và Cường (người mới của nhóm) trực thay. Cường không có năm phần series trong đầu. Thứ Cường cần là hai thứ: **một lệnh** cho biết server có ổn không, và nếu không thì **một tài liệu** nói phải làm gì, theo thứ tự, bằng lệnh cụ thể. Đó là runbook.

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-060 · Phần 6/6 · Nâng cao · Thực hành</span>
<p>Trạng thái đầu bài: <code>npm run setup -- 60</code>. Thử tự làm theo yêu cầu trước khi đọc lời giải. Cuối bài, <code>npm run kiem-tra:p06</code> chấm cả Phần 6: bastion, tường lửa của app02, vá tự động, fail2ban, Ansible, triển khai, trạng thái sau các sự cố, runbook.</p>
</div>

## Yêu cầu

1. Script `/usr/local/bin/kiem-tra-van-hanh`: mỗi kiểm tra một dòng `OK` / `CANH BAO` / `LOI` kèm bằng chứng (giá trị thật), mã thoát bằng số dòng `LOI`.
2. Kiểm tra: dịch vụ `ssh`, `bao-gia`, `nginx`, `ufw`, `fail2ban`; HTTPS `/suc-khoe` và phiên bản; chỗ và inode của `/` và `/du-lieu` (cảnh báo từ 80%, lỗi từ 90%); chứng chỉ TLS còn hạn trên 30 ngày; số IP fail2ban đang chặn; vá tự động; cần khởi động lại; bản sao lưu trên app02.
3. Runbook `/srv/toinoi/runbook/van-hanh.md`: vào server, kiểm tra hằng ngày, `bao-gia` không trả lời, đĩa đầy, không SSH được, cập nhật và khởi động lại.
4. Diễn tập: dừng `bao-gia`, để script phát hiện, xử lý theo runbook, kiểm chứng.

## 1. Một lệnh cho mọi bằng chứng

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-thuc-hanh-runbook-van-hanh/ssh-studio-01.webp" alt="Vim mở /usr/local/bin/kiem-tra-van-hanh qua sudoedit trong SSH Studio: các hàm ok, canh, hong; vòng lặp systemctl is-active cho ssh bao-gia nginx ufw fail2ban; kiểm tra HTTPS; df --output=pcent,ipcent cho / và /du-lieu" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · script <code>kiem-tra-van-hanh</code></figcaption></figure>

```bash
#!/usr/bin/env bash
# Kiểm tra vận hành toinoi-app-01 (LNX-060): mỗi dòng một bằng chứng, OK / CANH BAO / LOI.
# Mã thoát = số mục LOI. Chạy bằng an (cần sudo cho fail2ban, key đồng bộ cho app02).
set -uo pipefail
loi=0
ok()   { printf 'OK        %s\n' "$*"; }
canh() { printf 'CANH BAO  %s\n' "$*"; }
hong() { printf 'LOI       %s\n' "$*"; loi=$((loi + 1)); }

for dv in ssh bao-gia nginx ufw fail2ban; do
  tt=$(systemctl is-active "$dv")
  [ "$tt" = active ] && ok "dich vu $dv: $tt" || hong "dich vu $dv: $tt"
done

kq=$(curl -s --max-time 3 --cacert /etc/ssl/certs/bao-gia.toinoi.lab.crt https://bao-gia.toinoi.lab/suc-khoe)
if grep -q '"trang_thai": "ok"' <<<"$kq"; then
  ok "HTTPS /suc-khoe: phien ban $(sed -E 's/.*"phien_ban": "([^"]+)".*/\1/' <<<"$kq")"
else
  hong "HTTPS /suc-khoe khong tra loi"
fi

for mp in / /du-lieu; do
  read -r dung inode < <(df --output=pcent,ipcent "$mp" | tail -n 1 | tr -d '%')
  if [ "$dung" -ge 90 ] || [ "$inode" -ge 90 ]; then hong "dia $mp: dung ${dung}%, inode ${inode}%"
  elif [ "$dung" -ge 80 ] || [ "$inode" -ge 80 ]; then canh "dia $mp: dung ${dung}%, inode ${inode}%"
  else ok "dia $mp: dung ${dung}%, inode ${inode}%"; fi
done

if openssl x509 -checkend $((30 * 86400)) -noout -in /etc/ssl/certs/bao-gia.toinoi.lab.crt >/dev/null; then
  ok "chung chi TLS con han tren 30 ngay ($(openssl x509 -enddate -noout -in /etc/ssl/certs/bao-gia.toinoi.lab.crt | cut -d= -f2))"
else
  hong "chung chi TLS het han trong 30 ngay"
fi

ban=$(sudo fail2ban-client status sshd | awk -F'\t' '/Currently banned/ { print $2 }')
ok "fail2ban sshd: dang chan ${ban:-0} IP"

[ "$(apt-config dump APT::Periodic::Unattended-Upgrade | cut -d'"' -f2)" = 1 ] \
  && ok "va tu dong: bat" || canh "va tu dong: tat"
[ -e /var/run/reboot-required ] && canh "can khoi dong lai: $(tr '\n' ' ' < /var/run/reboot-required.pkgs 2>/dev/null)" \
  || ok "chua can khoi dong lai"

if sl=$(ssh -o BatchMode=yes -o ConnectTimeout=3 app02 'ls sao-luu/releases' 2>/dev/null); then
  ok "sao luu tren app02: $(tr '\n' ' ' <<<"$sl")"
else
  hong "khong doc duoc sao luu tren app02"
fi

echo "---- $loi muc LOI"
exit "$loi"
```

Mỗi khối là một bài trong series: `systemctl is-active` ([LNX-035](/articles/linux-systemd-systemctl)), `curl --cacert` ([LNX-049](/articles/linux-nginx-reverse-proxy-tls)), `df --output` với `pcent,ipcent` ([LNX-043](/articles/linux-df-du-dung-luong), [LNX-058](/articles/linux-su-co-dia-day-oom)), `openssl x509 -checkend`, `fail2ban-client` ([LNX-055](/articles/linux-gia-co-server-fail2ban)), `apt-config dump` ([LNX-054](/articles/linux-cap-nhat-he-thong-an-toan)), `ssh … app02` ([LNX-046](/articles/linux-dong-bo-rsync-ssh-studio)).

Ba chi tiết thiết kế:

- **`set -uo pipefail` mà không có `-e`**: một kiểm tra thất bại không được dừng cả script; mọi kiểm tra đều phải chạy và in kết quả ([LNX-019](/articles/linux-shell-script-an-toan)).
- **In giá trị thật**, không chỉ OK: «dùng 34%», «phiên bản 1.6.0», «hết hạn 24/11/2027». Người đọc thấy ngay nó gần ngưỡng hay không.
- **Mã thoát = số lỗi**: chạy được trong cron, timer, hay hệ thống giám sát (`kiem-tra-van-hanh || gửi-cảnh-báo`).

```text
an@toinoi-app-01:~$ sudo chmod 755 /usr/local/bin/kiem-tra-van-hanh
an@toinoi-app-01:~$ kiem-tra-van-hanh; echo "ma thoat: $?"
OK        dich vu ssh: active
OK        dich vu bao-gia: active
OK        dich vu nginx: active
OK        dich vu ufw: active
OK        dich vu fail2ban: active
OK        HTTPS /suc-khoe: phien ban 1.6.0
OK        dia /: dung 1%, inode 1%
OK        dia /du-lieu: dung 34%, inode 1%
OK        chung chi TLS con han tren 30 ngay (Nov 24 02:39:42 2027 GMT)
OK        fail2ban sshd: dang chan 0 IP
OK        va tu dong: bat
OK        chua can khoi dong lai
OK        sao luu tren app02: 1.4.2 1.5.0 1.6.0
---- 0 muc LOI
ma thoat: 0
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-thuc-hanh-runbook-van-hanh/ssh-studio-02.webp" alt="Terminal của SSH Studio: kiem-tra-van-hanh in 13 dòng OK cho dịch vụ, HTTPS phiên bản 1.6.0, đĩa / và /du-lieu, chứng chỉ, fail2ban, vá tự động, khởi động lại, sao lưu trên app02 1.4.2 1.5.0 1.6.0, dòng cuối 0 muc LOI và mã thoát 0" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · mọi kiểm tra đều OK</figcaption></figure>

## 2. Runbook nằm cạnh server

Runbook đặt trong `/srv/toinoi/runbook/` (nhóm `toinoi` ghi được, setgid như [LNX-026](/articles/linux-setuid-setgid-sticky)), ngay trên server: lúc sự cố, nó ở đúng chỗ người trực đang đứng. Bản gốc nên nằm trong Git của nhóm, và được đồng bộ ra server.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-thuc-hanh-runbook-van-hanh/ssh-studio-03.webp" alt="Vim mở /srv/toinoi/runbook/van-hanh.md trong SSH Studio: các mục Vào server, Kiểm tra hằng ngày, bao-gia không trả lời với ba bước, Đĩa hoặc inode trên 80%, Không SSH được, Cập nhật và khởi động lại" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · runbook <code>van-hanh.md</code></figcaption></figure>

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-thuc-hanh-runbook-van-hanh/ssh-studio-06.webp" alt="Terminal của SSH Studio: install -d tạo /srv/toinoi/runbook cho nhóm toinoi với setgid, vim van-hanh.md, grep -c '^## ' đếm 6 mục" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · runbook là một file Markdown nằm ngay trên server</figcaption></figure>

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-thuc-hanh-runbook-van-hanh/ssh-studio-07.webp" alt="Khung Files của SSH Studio: Máy của bạn bên trái, Remote server ở /srv/toinoi/runbook bên phải với file van-hanh.md 1.6 KB" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · mở runbook từ khung Files khi cần</figcaption></figure>

Nguyên tắc viết runbook:

- **Mỗi mục bắt đầu từ triệu chứng** người trực nhìn thấy («bao-gia không trả lời»), không từ nguyên nhân («Python lỗi»).
- **Lệnh cụ thể, chép-dán được**, theo thứ tự. Không «kiểm tra log»: là `journalctl -u bao-gia -n 20`.
- **Nói khi nào dừng và gọi người**: runbook không thay được người hiểu hệ thống.
- **Ngắn**. Thứ dài hơn một màn hình sẽ không ai đọc lúc 3 giờ sáng.

## 3. Diễn tập

Runbook chưa từng dùng là runbook chưa chắc đúng. An dừng `bao-gia` và làm như Cường sẽ làm:

```text
an@toinoi-app-01:~$ sudo systemctl stop bao-gia
an@toinoi-app-01:~$ kiem-tra-van-hanh | grep -E 'LOI|----'
LOI       dich vu bao-gia: inactive
LOI       HTTPS /suc-khoe khong tra loi
---- 2 muc LOI
an@toinoi-app-01:~$ sed -n '/^## bao-gia/,/^## Đĩa/p' /srv/toinoi/runbook/van-hanh.md | head -n 5
## bao-gia không trả lời
1. `systemctl status bao-gia` và `journalctl -u bao-gia -n 20`.
2. Đang dừng: `sudo systemctl start bao-gia`, chạy lại `kiem-tra-van-hanh`.
3. Vừa triển khai: `trien-khai <bản trước>` (xem `ls /srv/toinoi/bao-gia/releases`).

an@toinoi-app-01:~$ journalctl -u bao-gia -n 2 --no-pager -o cat
bao-gia.service: Deactivated successfully.
Stopped bao-gia.service - bao-gia: dich vu bao gia cua Toi Noi.
an@toinoi-app-01:~$ sudo systemctl start bao-gia
an@toinoi-app-01:~$ sleep 2; kiem-tra-van-hanh | tail -n 1
---- 0 muc LOI
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-thuc-hanh-runbook-van-hanh/ssh-studio-05.webp" alt="Terminal của SSH Studio: sudo systemctl stop bao-gia, kiem-tra-van-hanh | grep LOI cho hai dòng LOI: dich vu bao-gia inactive và HTTPS /suc-khoe khong tra loi, tổng 2 muc LOI" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · diễn tập: bao-gia dừng, script kiểm tra bắt được ngay</figcaption></figure>

1. Script báo **2 LOI**: dịch vụ `inactive`, HTTPS không trả lời. Đúng hai triệu chứng của cùng một nguyên nhân.
2. Mở đúng mục của runbook (`sed -n '/^## bao-gia/,/^## Đĩa/p'` in một mục).
3. Bước 1 của runbook: journal cho thấy dịch vụ được **dừng** (`Deactivated successfully`), không phải chết vì lỗi. Có ai đó đã dừng nó.
4. Bước 2: `start`, chạy lại script: **0 LOI**.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-thuc-hanh-runbook-van-hanh/ssh-studio-04.webp" alt="Terminal của SSH Studio: sed in mục bao-gia không trả lời của runbook với ba bước xử lý, journalctl -u bao-gia cho Deactivated successfully và Stopped, systemctl start bao-gia, kiem-tra-van-hanh | tail -n 1 in 0 muc LOI" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · làm theo runbook: đọc log, khởi động lại, kiểm tra lại về 0 lỗi</figcaption></figure>

Sau diễn tập, thêm vào runbook điều vừa học: «journal ghi `Stopped`/`Deactivated` nghĩa là có người dừng: hỏi kênh vận hành trước khi bật lại.»

## 4. Chấm cả Phần 6

```bash
npm run kiem-tra:p06
```

| Nhóm | Bộ chấm kiểm tra |
|---|---|
| Bastion và app02 | Bastion chỉ nhận key, không agent/X11; **từ bastion và app01** tới được cổng 22 của app02; ufw của app02 chỉ mở SSH cho `10.20.0.10` và `10.20.0.11` |
| Vá và gia cố | Vá tự động bật, không tự khởi động lại, nginx ngoài danh sách; fail2ban chạy, `ignoreip` có `10.20.0.1` và bastion, chặn bằng ufw |
| Ansible | `ansible -m ping` tới cả hai máy; playbook chạy `--check` không còn gì thay đổi; `MaxAuthTries 3` trên app02 và bastion |
| Triển khai | `current` → `releases/1.6.0`, HTTPS trả 1.6.0, `trien-khai` từ chối release không tồn tại |
| Sau sự cố | `/du-lieu` dưới 80% chỗ và inode, job sao lưu không lỗi, `authorized_keys` của app02 quyền 600, app01 tin host key mới của app02 |
| Runbook | Đủ các mục chính; `kiem-tra-van-hanh` cho 0 LOI |

Tất cả `PASS` (24 mục) là xong Phần 6, và xong series.

## 5. Lỗi thường gặp

**Script kiểm tra dùng `set -e`.** Kiểm tra đầu tiên thất bại thì các kiểm tra sau không bao giờ chạy.

**Runbook viết một lần rồi để đó.** Mỗi sự cố, mỗi lần đổi hệ thống: cập nhật. Ghi ngày cập nhật ở đầu.

**Runbook chỉ có trên wiki.** Wiki có thể nằm sau chính hệ thống đang sập.

**Chỉ in «OK».** Không thấy được xu hướng (đĩa 79% hôm nay là đầy tuần sau).

## Bài tập

1. Chạy `kiem-tra-van-hanh` mỗi giờ bằng một timer ([LNX-038](/articles/linux-cron-systemd-timer)), ghi vào `/var/log/toinoi/van-hanh.log`, chỉ ghi những lần có LOI hoặc CANH BAO.
2. Thêm kiểm tra: `bao-cao-tai-nguyen.timer` ([LNX-050](/articles/linux-thuc-hanh-mang-giam-sat)) có chạy trong 20 phút gần nhất không (`systemctl show -p LastTriggerUSec`).
3. Viết phiên bản chạy được trên app02 và bastion (bỏ các kiểm tra chỉ có ở app01), đẩy ra bằng Ansible, gọi từ app01 bằng `ansible all -a kiem-tra-van-hanh`.
4. Nhờ một người chưa đọc series làm theo runbook để xử lý sự cố «đĩa đầy» trong lab ([LNX-058](/articles/linux-su-co-dia-day-oom)). Ghi lại mọi chỗ họ phải hỏi, rồi sửa runbook.

## Đọc tiếp

Series khép lại ở đây. Sáu phần đi từ một phiên SSH đầu tiên tới một hệ thống nhỏ nhưng thật: server có quyền và người dùng đúng, SSH chỉ nhận key và OTP, dịch vụ chạy bằng systemd sau nginx có TLS, tường lửa tối thiểu, sao lưu sang máy thứ hai, bastion, vá tự động, và một runbook có bằng chứng. Xem lại toàn bộ [lộ trình của series](/articles/linux-la-gi-va-ssh-studio), và giữ lab: mọi sự cố trong series đều dựng lại được bằng `npm run setup`.
