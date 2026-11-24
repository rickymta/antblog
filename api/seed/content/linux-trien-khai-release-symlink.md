---
slug: linux-trien-khai-release-symlink
title: "Triển khai bằng thư mục release và symlink: đổi phiên bản trong một lệnh, quay lui trong một lệnh"
summary: "Cấu trúc releases/ và current của bao-gia, đưa bản build 1.6.0 vào chỗ, đổi symlink nguyên tử bằng ln -sfn và mv -T, đo khoảng gián đoạn thật khi khởi động lại (vài yêu cầu 502 qua nginx), quay lui về 1.5.0 trong một dòng, và đóng gói thành script trien-khai có kiểm tra sức khoẻ và tự quay lui."
category: devops-linux
level: intermediate
tags: [linux, lnx-057, deploy, symlink, release, rollback, systemd, nginx, ssh-studio, toi-noi]
language: vi
---

Bản `bao-gia` 1.6.0 với dịch vụ ô tô 7 chỗ điện đã qua kiểm thử. Lần triển khai trước, có người chép đè file lên thư mục đang chạy, dịch vụ hỏng giữa chừng, và không ai nhớ bản cũ trông thế nào. Cấu trúc thư mục An dựng từ [LNX-010](/articles/linux-thuc-hanh-thu-muc-trien-khai) được làm chính cho việc này: mỗi phiên bản một thư mục, `current` chỉ là một symlink.

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-057 · Phần 6/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/linux-thuc-hanh-thu-muc-trien-khai">LNX-010</a> (cấu trúc triển khai), <a href="/articles/linux-systemd-unit-bao-gia">LNX-036</a> (unit), <a href="/articles/linux-nginx-reverse-proxy-tls">LNX-049</a> (nginx). Trạng thái đầu bài: <code>npm run setup -- 57</code>. «Bản build từ CI» được lab đặt sẵn trong <code>/usr/share/toinoi-lab/p06</code>.</p>
</div>

## 1. Cấu trúc hiện tại

```text
an@toinoi-app-01:~$ cd /srv/toinoi/bao-gia
an@toinoi-app-01:/srv/toinoi/bao-gia$ ls -l
total 12
lrwxrwxrwx 1 an an       14 Nov 24 09:38 current -> releases/1.5.0
drwxr-xr-x 5 an toinoi 4096 Nov 24 09:39 releases
drwxr-x--- 2 an toinoi 4096 Nov 24 09:36 scripts
drwxr-xr-x 4 an toinoi 4096 Nov 24 09:36 shared
an@toinoi-app-01:/srv/toinoi/bao-gia$ ls releases
1.4.2  1.5.0  1.6.0
an@toinoi-app-01:/srv/toinoi/bao-gia$ readlink current
releases/1.5.0
an@toinoi-app-01:/srv/toinoi/bao-gia$ curl -s --cacert /etc/ssl/certs/bao-gia.toinoi.lab.crt https://bao-gia.toinoi.lab/suc-khoe
{"trang_thai": "ok", "phien_ban": "1.5.0", "pid": 4279, "moi_truong": "production", "so_dich_vu": 4}
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-trien-khai-release-symlink/ssh-studio-05.webp" alt="Terminal của SSH Studio: cd /srv/toinoi/bao-gia, ls -l cho current là symlink tới releases/1.5.0, cùng các thư mục releases, scripts, shared" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · cấu trúc triển khai: <code>current</code> là symlink</figcaption></figure>

- `releases/` giữ mọi phiên bản đã triển khai: `1.4.2`, `1.5.0`, `1.6.0` (thư mục chuẩn bị ở [LNX-046](/articles/linux-dong-bo-rsync-ssh-studio)).
- `current -> releases/1.5.0`: unit systemd dùng `/srv/toinoi/bao-gia/current/bao-gia` ([LNX-036](/articles/linux-systemd-unit-bao-gia)), nên chỉ cần đổi symlink là đổi bản được chạy.
- `shared/` (cấu hình, log) nằm ngoài mọi release: không bị mất khi đổi bản.
- Symlink **tương đối** (`releases/1.5.0`): chép cả thư mục sang máy khác vẫn đúng.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-trien-khai-release-symlink/ssh-studio-01.webp" alt="Terminal của SSH Studio: ls releases cho 1.4.2, 1.5.0, 1.6.0, readlink current trỏ releases/1.5.0, curl https://bao-gia.toinoi.lab/suc-khoe trả phiên bản 1.5.0" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · đang chạy 1.5.0, đã có sẵn thư mục 1.6.0</figcaption></figure>

## 2. Đưa bản build vào chỗ

```text
an@toinoi-app-01:/srv/toinoi/bao-gia$ sudo install -m 755 -o an -g toinoi /usr/share/toinoi-lab/p06/bao-gia-1.6.0 releases/1.6.0/bao-gia
an@toinoi-app-01:/srv/toinoi/bao-gia$ diff -rq releases/1.5.0 releases/1.6.0
Only in releases/1.6.0: THAY-DOI.txt
Files releases/1.5.0/bang-gia.csv and releases/1.6.0/bang-gia.csv differ
Files releases/1.5.0/bao-gia and releases/1.6.0/bao-gia differ
an@toinoi-app-01:/srv/toinoi/bao-gia$ grep -m1 PHIEN_BAN releases/1.6.0/bao-gia
PHIEN_BAN = '1.6.0'
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-trien-khai-release-symlink/ssh-studio-06.webp" alt="Terminal của SSH Studio: install bản build bao-gia-1.6.0 vào releases/1.6.0, diff -rq với 1.5.0 cho THAY-DOI.txt mới, bang-gia.csv và bao-gia khác, grep PHIEN_BAN cho 1.6.0" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · bản build 1.6.0 đã vào đúng thư mục release</figcaption></figure>

`diff -rq` so hai release: khác bảng giá, chương trình, và có thêm ghi chú thay đổi. Chưa có gì đang chạy bị đụng tới.

## 3. Đổi current: nguyên tử

```text
an@toinoi-app-01:/srv/toinoi/bao-gia$ ln -sfn releases/1.6.0 current.moi
an@toinoi-app-01:/srv/toinoi/bao-gia$ mv -T current.moi current
an@toinoi-app-01:/srv/toinoi/bao-gia$ readlink current
releases/1.6.0
```

Tại sao không `ln -sfn releases/1.6.0 current` thẳng? Lệnh đó thực ra là hai bước: **xoá** symlink cũ, **tạo** symlink mới. Giữa hai bước, `current` không tồn tại, và tiến trình nào mở `current/…` đúng lúc đó sẽ lỗi. Cần một thao tác nguyên tử: chỉ có trước và sau, không có nửa chừng.

1. Tạo symlink mới với tên tạm `current.moi`.
2. `mv -T current.moi current`: đổi tên là **một** thao tác `rename()` của kernel, đè lên `current` trong một bước. Ở mọi thời điểm, `current` trỏ tới bản cũ hoặc bản mới, không bao giờ không có. (`-T`: coi `current` là tên đích, không phải thư mục để chuyển vào.)

## 4. Khởi động lại, và đo khoảng gián đoạn

Đổi symlink không làm tiến trình đang chạy đổi mã: nó đã nạp `bao-gia` 1.5.0 vào bộ nhớ. Phải khởi động lại. Trong lúc đó, An cho một vòng lặp gọi qua nginx 60 lần, mỗi 50 ms:

```text
an@toinoi-app-01:/srv/toinoi/bao-gia$ (for i in $(seq 60); do curl -s -o /dev/null -w '%{http_code}\n' --cacert /etc/ssl/certs/bao-gia.toinoi.lab.crt https://bao-gia.toinoi.lab/suc-khoe; sleep 0.05; done > /tmp/do-trien-khai.txt) &
an@toinoi-app-01:/srv/toinoi/bao-gia$ sleep 1; sudo systemctl restart bao-gia
an@toinoi-app-01:/srv/toinoi/bao-gia$ wait
an@toinoi-app-01:/srv/toinoi/bao-gia$ sort /tmp/do-trien-khai.txt | uniq -c
     58 200
      2 502
```

Kết quả thật: phần lớn `200`, và **vài yêu cầu `502`**. Trong khoảng một phần mười giây giữa lúc tiến trình cũ thoát và tiến trình mới mở cổng 8080, nginx không có ai để chuyển yêu cầu tới. Với một tiến trình duy nhất, khởi động lại luôn hở một khoảng như vậy.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-trien-khai-release-symlink/ssh-studio-02.webp" alt="Terminal của SSH Studio: tạo current.moi và mv -T, readlink current là releases/1.6.0, vòng lặp curl chạy nền trong lúc sudo systemctl restart bao-gia, sort uniq -c đếm phần lớn 200 và vài 502, curl HTTPS trả phien_ban 1.6.0" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · đổi sang 1.6.0: vài yêu cầu <code>502</code> trong lúc khởi động lại</figcaption></figure>

Muốn không mất yêu cầu nào thì cần ít nhất **hai tiến trình** sau nginx (hai cổng, `upstream` có hai `server`), khởi động lại lần lượt từng cái: bài tập 4. Với Tới Nơi lúc này, khoảng hở 0,1 giây lúc 22:00 là chấp nhận được, miễn là **biết** nó tồn tại và đã đo.

## 5. Quay lui trong một dòng

```text
an@toinoi-app-01:/srv/toinoi/bao-gia$ ln -sfn releases/1.5.0 current.moi && mv -T current.moi current && sudo systemctl restart bao-gia
an@toinoi-app-01:/srv/toinoi/bao-gia$ sleep 1; curl -s --cacert /etc/ssl/certs/bao-gia.toinoi.lab.crt https://bao-gia.toinoi.lab/suc-khoe
{"trang_thai": "ok", "phien_ban": "1.5.0", "pid": 9024, "moi_truong": "production", "so_dich_vu": 4}
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-trien-khai-release-symlink/ssh-studio-07.webp" alt="Terminal của SSH Studio: ln -sfn releases/1.5.0 current.moi &amp;&amp; mv -T current.moi current &amp;&amp; restart bao-gia trên một dòng, curl /suc-khoe trả lại phiên bản 1.5.0" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · quay lui về 1.5.0 bằng một dòng lệnh</figcaption></figure>

Bản cũ vẫn nằm nguyên trong `releases/1.5.0`, nên quay lui chỉ là trỏ `current` về nó và khởi động lại. Không chép, không tải lại, không đoán.

## 6. Đóng gói: script trien-khai

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-trien-khai-release-symlink/ssh-studio-03.webp" alt="Vim mở ~/bin/trien-khai trong SSH Studio: script bash nhận PHIEN_BAN, kiểm tra release tồn tại, hàm doi_sang tạo current.moi và mv -T rồi restart, hàm kiem_tra gọi /suc-khoe tối đa 5 lần, quay về bản cũ nếu không khoẻ" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · script <code>trien-khai</code></figcaption></figure>

```bash
#!/usr/bin/env bash
# Triển khai một release của bao-gia (LNX-057): trien-khai PHIEN_BAN
# Đổi current sang releases/PHIEN_BAN, khởi động lại, kiểm tra sức khoẻ; hỏng thì quay về bản cũ.
set -euo pipefail
GOC=/srv/toinoi/bao-gia
URL=https://bao-gia.toinoi.lab/suc-khoe
CA=/etc/ssl/certs/bao-gia.toinoi.lab.crt

moi=${1:?Dung: trien-khai PHIEN_BAN}
[ -x "$GOC/releases/$moi/bao-gia" ] || { echo "LOI: khong co release $moi" >&2; exit 1; }
cu=$(basename "$(readlink "$GOC/current")")

doi_sang() {
  ln -sfn "releases/$1" "$GOC/current.moi"
  mv -T "$GOC/current.moi" "$GOC/current"
  sudo systemctl restart bao-gia
}

kiem_tra() {
  for _ in 1 2 3 4 5; do
    curl -s --cacert "$CA" "$URL" | grep -q "\"phien_ban\": \"$1\"" && return 0
    sleep 1
  done
  return 1
}

echo "trien khai $moi (dang chay: $cu)"
doi_sang "$moi"
if kiem_tra "$moi"; then
  echo "OK: bao-gia $moi dang chay"
else
  echo "LOI: $moi khong khoe, quay ve $cu" >&2
  doi_sang "$cu"
  exit 1
fi
```

- Kiểm tra trước khi đụng vào gì: release không tồn tại thì dừng ngay.
- **Kiểm tra sức khoẻ sau khi đổi**, và kiểm tra đúng phiên bản (không chỉ «có trả lời»): một tiến trình cũ chưa kịp thoát cũng trả lời được.
- Không khoẻ sau 5 giây thì tự quay về bản cũ và thoát với mã lỗi, để người chạy (hay CI) biết.

```text
an@toinoi-app-01:/srv/toinoi/bao-gia$ trien-khai 1.9.9
LOI: khong co release 1.9.9
an@toinoi-app-01:/srv/toinoi/bao-gia$ trien-khai 1.6.0
trien khai 1.6.0 (dang chay: 1.5.0)
OK: bao-gia 1.6.0 dang chay
an@toinoi-app-01:/srv/toinoi/bao-gia$ readlink current
releases/1.6.0
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-trien-khai-release-symlink/ssh-studio-04.webp" alt="Terminal của SSH Studio: trien-khai 1.9.9 báo LOI khong co release, trien-khai 1.6.0 in trien khai 1.6.0 dang chay 1.5.0 rồi OK bao-gia 1.6.0 dang chay, readlink current là releases/1.6.0" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>trien-khai 1.6.0</code>: đổi, kiểm tra, xong</figcaption></figure>

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-trien-khai-release-symlink/ssh-studio-08.webp" alt="Khung Files của SSH Studio: Máy của bạn bên trái, Remote server ở /srv/toinoi/bao-gia/releases bên phải với ba thư mục 1.4.2, 1.5.0, 1.6.0" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · khung Files: mỗi bản phát hành một thư mục</figcaption></figure>

## 7. Lỗi thường gặp

**Chép đè vào thư mục đang chạy.** Tiến trình đọc nửa file cũ nửa file mới; không còn bản cũ để quay lui.

**`ln -sfn` thẳng lên `current`.** Có một khoảnh khắc không có `current`. Dùng tên tạm và `mv -T`.

**Kiểm tra sức khoẻ chỉ bằng mã 200.** Có thể vẫn là bản cũ. Kiểm tra phiên bản.

**Giữ mọi release mãi mãi.** Đĩa đầy ([LNX-058](/articles/linux-su-co-dia-day-oom)). Giữ vài bản gần nhất.

## Bài tập

1. Thêm vào `trien-khai`: sau khi thành công, xoá các release cũ, chỉ giữ 3 bản mới nhất (trừ bản đang chạy). `ls -1t`, `tail -n +4`.
2. Làm hỏng 1.6.0 (`chmod -x releases/1.6.0/bao-gia`), chạy `trien-khai 1.6.0`. Script báo gì? `current` trỏ đi đâu? (Sửa lại quyền sau đó.)
3. Ghi mỗi lần triển khai vào `shared/logs/trien-khai.log` (thời điểm, ai, từ bản nào sang bản nào).
4. Không gián đoạn thật: chạy thêm một `bao-gia` thứ hai ở cổng 8081 (unit `bao-gia@.service` với `BAO_GIA_CONG=%i`), cho nginx `upstream` gồm 8080 và 8081, khởi động lại lần lượt từng cái và đo lại như mục 4.

## Đọc tiếp

[LNX-058 · Sự cố: đĩa đầy, hết inode, OOM killer](/articles/linux-su-co-dia-day-oom).
