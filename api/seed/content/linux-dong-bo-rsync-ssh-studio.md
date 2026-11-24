---
slug: linux-dong-bo-rsync-ssh-studio
title: "Đồng bộ thư mục: Đồng bộ của SSH Studio và rsync, xem trước bằng --dry-run, mirror bằng --delete, và rsync qua SSH"
summary: "Đưa thư mục bảng giá mới từ máy bạn lên server bằng tính năng Đồng bộ của SSH Studio (so sánh trước, chọn chiều, mirror có xác nhận riêng), rồi dùng rsync trên server: -a giữ quyền và thời gian, -i đọc từng thay đổi, --dry-run xem trước, --delete làm bản sao giống hệt, và rsync qua SSH sang toinoi-app-02 bằng một key riêng trong ~/.ssh/config."
category: devops-linux
level: intermediate
tags: [linux, lnx-046, rsync, sync, backup, ssh, ssh-studio, toi-noi]
language: vi
---

Nhóm kinh doanh gửi An bảng giá mới, có thêm dịch vụ ô tô 7 chỗ chạy điện, áp dụng từ đầu tháng 12. An cần đưa nó lên server để chuẩn bị bản `bao-gia` 1.6.0, giữ một bản sao các release trên một ổ khác, và thêm một bản nữa trên `toinoi-app-02` phòng khi `toinoi-app-01` hỏng hẳn. Ba việc, một ý tưởng: **đồng bộ** hai thư mục, chỉ chép phần khác nhau.

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-046 · Phần 5/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/linux-tao-sao-chep-di-chuyen-xoa-file">LNX-005</a> (khung Files), <a href="/articles/linux-ssh-agent-passphrase">LNX-028</a> (<code>~/.ssh/config</code>). Trạng thái đầu bài: <code>npm run setup -- 46</code>. Thư mục «trên máy bạn» là <code>examples/toi-noi-linux/may-cua-ban/bang-gia-moi</code>. Key đồng bộ sang app02 được lab cài sẵn (trong thực tế, Bình cấp cho An).</p>
</div>

## 1. Đồng bộ trong SSH Studio

Trong khung **Files** ([LNX-005](/articles/linux-tao-sao-chep-di-chuyen-xoa-file)), mở thư mục `bang-gia-moi` ở bên máy bạn và `/home/an/bang-gia-moi` (vừa tạo, còn trống) ở bên server, rồi bấm **Đồng bộ** ở thanh công cụ:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-dong-bo-rsync-ssh-studio/ssh-studio-01.webp" alt="Hộp thoại Đồng bộ thư mục của SSH Studio: thư mục local bang-gia-moi trên máy Windows, chiều Local → Remote được chọn, thư mục remote /home/an/bang-gia-moi, danh sách bang-gia.csv và ghi-chu.txt chỉ có ở local được đánh dấu để tải lên, ô Mirror chưa chọn, nút Đồng bộ" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <b>Đồng bộ thư mục</b>: so sánh trước, chọn chiều, rồi mới chép</figcaption></figure>

- SSH Studio **so sánh** hai bên theo kích thước và thời gian sửa đổi, liệt kê file chỉ có ở một bên hoặc khác nhau, và chọn sẵn những file sẽ chép theo **chiều** đang chọn (Local → Remote hay ngược lại).
- Mặc định **không xoá gì**. Ô **Mirror** (xoá những gì chỉ có ở đích) phải bật riêng, và còn hỏi xác nhận lần nữa với danh sách sẽ bị xoá.
- Bấm **Đồng bộ**: các file vào hàng đợi truyền như khi kéo thả.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-dong-bo-rsync-ssh-studio/ssh-studio-02.webp" alt="Khung Files của SSH Studio sau khi đồng bộ: bên server /home/an/bang-gia-moi đã có bang-gia.csv và ghi-chu.txt" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · hai file đã lên server</figcaption></figure>

```text
an@toinoi-app-01:~$ ls -l ~/bang-gia-moi
total 8
-rw-rw-r-- 1 an an 234 Nov 24 08:12 bang-gia.csv
-rw-rw-r-- 1 an an 175 Nov 24 08:12 ghi-chu.txt
an@toinoi-app-01:~$ diff /srv/toinoi/bao-gia/current/bang-gia.csv ~/bang-gia-moi/bang-gia.csv
5a6
> car7e,Ô tô 7 chỗ điện,16000,5000,32000
```

`diff` ([LNX-006](/articles/linux-xem-sua-file-van-ban)) xác nhận khác biệt duy nhất: dòng `car7e` mới.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-dong-bo-rsync-ssh-studio/ssh-studio-03.webp" alt="Terminal của SSH Studio: ls -l ~/bang-gia-moi với bang-gia.csv và ghi-chu.txt, diff với bảng giá hiện tại chỉ ra một dòng thêm car7e, Ô tô 7 chỗ điện" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · kiểm tra trên server</figcaption></figure>

## 2. rsync trên một máy

**rsync** so hai cây thư mục và chỉ chép phần khác nhau. Trên server, nó là công cụ chuẩn cho sao lưu và triển khai.

```text
an@toinoi-app-01:~$ cp -a /srv/toinoi/bao-gia/releases/1.5.0 /srv/toinoi/bao-gia/releases/1.6.0
an@toinoi-app-01:~$ cp ~/bang-gia-moi/bang-gia.csv /srv/toinoi/bao-gia/releases/1.6.0/bang-gia.csv
an@toinoi-app-01:~$ sudo install -d -o an -g toinoi -m 2775 /srv/sao-luu
an@toinoi-app-01:~$ rsync -a /srv/toinoi/bao-gia/releases/ /srv/sao-luu/releases/
an@toinoi-app-01:~$ ls /srv/sao-luu/releases
1.4.2  1.5.0  1.6.0
an@toinoi-app-01:~$ echo 'ghi chu cu' > /srv/sao-luu/releases/ghi-chu-cu.txt
an@toinoi-app-01:~$ echo 'ban 1.6.0: them dich vu car7e' > /srv/toinoi/bao-gia/releases/1.6.0/THAY-DOI.txt
an@toinoi-app-01:~$ rsync -ai --dry-run --delete /srv/toinoi/bao-gia/releases/ /srv/sao-luu/releases/
*deleting   ghi-chu-cu.txt
.d..t...... 1.6.0/
>f+++++++++ 1.6.0/THAY-DOI.txt
an@toinoi-app-01:~$ rsync -ai --delete /srv/toinoi/bao-gia/releases/ /srv/sao-luu/releases/
*deleting   ghi-chu-cu.txt
.d..t...... 1.6.0/
>f+++++++++ 1.6.0/THAY-DOI.txt
an@toinoi-app-01:~$ rsync -ai --delete /srv/toinoi/bao-gia/releases/ /srv/sao-luu/releases/
```

| Tuỳ chọn | Ý nghĩa |
|---|---|
| `-a` (*archive*) | Đệ quy, giữ quyền, chủ, thời gian, symlink: bản sao «giống hệt» |
| `-i` | In từng thay đổi bằng mã: `>f+++++++++` file mới, `.d..t......` thư mục đổi thời gian, `*deleting` sắp xoá |
| `--dry-run` (`-n`) | Chỉ in sẽ làm gì, không làm |
| `--delete` | Xoá ở đích những gì không còn ở nguồn: đích thành bản sao chính xác (*mirror*) |

Chi tiết quan trọng nhất: **dấu `/` cuối nguồn**. `releases/` nghĩa là «nội dung của `releases`»; `releases` (không `/`) nghĩa là «chính thư mục `releases`», và sẽ tạo `sao-luu/releases/releases`.

Quy trình an toàn với `--delete` là **luôn chạy `--dry-run` trước**, đọc các dòng `*deleting`, rồi mới chạy thật. Một đường dẫn gõ nhầm với `--delete` có thể xoá cả thư mục đích. Lần chạy thứ ba không in gì: hai bên đã giống nhau, rsync không chép lại.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-dong-bo-rsync-ssh-studio/ssh-studio-04.webp" alt="Terminal của SSH Studio: chuẩn bị release 1.6.0 với bảng giá mới, tạo /srv/sao-luu, rsync -a lần đầu, ls ba release, thêm ghi-chu-cu.txt ở đích và THAY-DOI.txt ở nguồn, rsync -ai --dry-run --delete in *deleting ghi-chu-cu.txt và >f+++++++++ 1.6.0/THAY-DOI.txt, chạy thật in cùng nội dung, lần ba không in gì" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>rsync -ai --dry-run --delete</code>: xem trước rồi mới làm</figcaption></figure>

## 3. rsync qua SSH sang toinoi-app-02

Khi đích là `MÁY:ĐƯỜNG_DẪN`, rsync chạy qua SSH: đăng nhập vào máy kia, chạy một rsync ở đó, và hai bên chỉ trao đổi phần khác nhau. Để chạy được không cần ai gõ gì (sau này là trong một timer), An dùng một **key riêng cho việc đồng bộ**, không passphrase, chỉ được cài trên `toinoi-app-02`:

```text
# Server dự phòng trong mạng nội bộ: chỉ dùng key đồng bộ, không passphrase (LNX-046).
Host toinoi-app-02 app02
    HostName toinoi-app-02
    User an
    IdentityFile ~/.ssh/dong-bo
    IdentitiesOnly yes
    StrictHostKeyChecking accept-new
```

```text
an@toinoi-app-01:~$ ls -l ~/.ssh/dong-bo*
-rw------- 1 an an 411 Nov 24 08:12 /home/an/.ssh/dong-bo
-rw-r--r-- 1 an an 103 Nov 24 08:12 /home/an/.ssh/dong-bo.pub
an@toinoi-app-01:~$ vim ~/.ssh/config
an@toinoi-app-01:~$ ssh app02 hostname
Warning: Permanently added 'toinoi-app-02' (ED25519) to the list of known hosts.
toinoi-app-02
an@toinoi-app-01:~$ ssh app02 mkdir -p sao-luu
an@toinoi-app-01:~$ rsync -a --delete --stats /srv/sao-luu/releases/ app02:sao-luu/releases/ | grep -E 'Number of (files|regular files transferred)|Total transferred'
Number of files: 13 (reg: 9, dir: 4)
Number of regular files transferred: 9
Total transferred file size: 12,859 bytes
an@toinoi-app-01:~$ ssh app02 'ls sao-luu/releases; cat sao-luu/releases/1.6.0/bang-gia.csv | tail -n 1'
1.4.2
1.5.0
1.6.0
car7e,Ô tô 7 chỗ điện,16000,5000,32000
an@toinoi-app-01:~$ rsync -ai --delete /srv/sao-luu/releases/ app02:sao-luu/releases/
```

- Lần đầu, `accept-new` ghi nhận host key của app02 ([LNX-023](/articles/linux-host-key-known-hosts)); sau đó host key đổi thì sẽ bị chặn như thường.
- `--stats` tóm tắt: 9 file được chép, khoảng 12 KB. Lần sau chỉ phần thay đổi đi qua mạng.
- Bảng giá của 1.6.0 đã có mặt trên app02.

Một key không passphrase là một rủi ro có chủ đích: giới hạn nó bằng những gì nó mở được (chỉ một tài khoản trên một máy). Có thể chặt hơn nữa bằng `command=` và `restrict` trong `authorized_keys` của app02, để key đó **chỉ** chạy được rsync (bài tập 3).

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-dong-bo-rsync-ssh-studio/ssh-studio-05.webp" alt="Terminal của SSH Studio: key dong-bo quyền 600, vim ~/.ssh/config, ssh app02 hostname với Permanently added toinoi-app-02, rsync --stats qua SSH chép 9 file, ssh app02 liệt kê 1.4.2 1.5.0 1.6.0 và dòng car7e, rsync lần hai không in gì" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>rsync</code> qua SSH sang <code>toinoi-app-02</code></figcaption></figure>

## 4. Đối chiếu thao tác

| SSH Studio | rsync |
|---|---|
| Đồng bộ, chiều Local → Remote | `rsync -a ./thu-muc/ server:thu-muc/` |
| Danh sách so sánh trước khi chép | `rsync -ai --dry-run …` |
| Mirror: xoá mục chỉ có ở đích (có xác nhận) | `--delete` (không hỏi gì) |
| Hàng đợi truyền, tiến độ | `--progress`, `--stats` |

SSH Studio hợp cho việc giữa máy bạn và server, nhìn thấy trước khi làm. `rsync` hợp cho việc lặp lại giữa các server, trong script và timer.

## 5. Lỗi thường gặp

**Thiếu hoặc thừa dấu `/` cuối nguồn.** Lồng thêm một tầng thư mục ở đích. Kiểm tra bằng `--dry-run`.

**`--delete` mà chưa `--dry-run`.** Mirror sai chiều là mất dữ liệu ở đích.

**Thư mục cha ở đích chưa có.** `rsync: mkdir … failed: No such file or directory`: tạo trước (`ssh app02 mkdir -p sao-luu`) hoặc dùng `--mkpath`.

**Đồng bộ ngược chiều.** Trong SSH Studio, nhìn mũi tên chiều trước khi bấm. Trong rsync, nguồn luôn đứng trước, đích đứng sau.

**Dùng key cá nhân có passphrase cho việc tự động.** Hoặc script treo chờ passphrase, hoặc phải bỏ passphrase của key cá nhân. Dùng key riêng, quyền hẹp.

## Bài tập

1. Sửa `ghi-chu.txt` trên máy bạn, mở lại Đồng bộ. SSH Studio đánh dấu file đó thế nào?
2. `rsync -ain --delete /srv/sao-luu/releases /srv/thu/` (không `/` cuối). Đích sẽ có gì?
3. Trên app02, thêm vào đầu dòng key đồng bộ trong `authorized_keys`: `restrict,command="rsync --server -logDtpre.iLsfxCIvu --delete . sao-luu/releases/"`. Thử `ssh app02 hostname` và lại `rsync`. Chuyện gì xảy ra? (Lấy đúng lệnh server bằng `rsync -e 'ssh -v'`.)
4. Viết một timer chạy rsync sang app02 lúc 03:00 mỗi đêm ([LNX-038](/articles/linux-cron-systemd-timer)), chạy bằng `an`.

## Đọc tiếp

[LNX-047 · Đĩa, phân vùng, hệ thống file và mount](/articles/linux-dia-phan-vung-mount).
