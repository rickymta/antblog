---
slug: linux-acl-setfacl
title: "ACL trên Linux: setfacl và getfacl khi chủ sở hữu, nhóm và người khác không đủ"
summary: "Cấp quyền đọc thư mục log cho đúng một người mà không thêm họ vào nhóm, đọc getfacl, dấu + trong ls -l, ACL mặc định cho file mới, mask giới hạn mọi quyền ACL và vì sao chmod có thể âm thầm vô hiệu ACL, rồi gỡ ACL bằng setfacl -x, -k, -b."
category: devops-linux
level: intermediate
tags: [linux, lnx-027, acl, setfacl, getfacl, permissions, mask, ssh-studio, toi-noi]
language: vi
---

Phòng kiểm toán nội bộ cần đọc log của dịch vụ báo giá trong một tuần. Thư mục `/srv/toinoi/bao-gia/shared/logs` thuộc nhóm `toinoi`, quyền `2770`. Có hai cách quen thuộc, và cả hai đều tệ:

- Thêm tài khoản kiểm toán vào nhóm `toinoi`: họ được **ghi** vào thư mục log, và đọc được mọi thứ khác của nhóm.
- Mở quyền cho «người khác» (`o+rx`): mọi tài khoản trên server đều đọc được log.

Mô hình chủ/nhóm/người khác chỉ có **một** chủ và **một** nhóm cho mỗi file. Khi cần «thêm đúng một người, với đúng quyền đọc», Linux có **ACL** (*Access Control List*).

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-027 · Phần 3/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/linux-nguoi-dung-nhom-quyen-file">LNX-007</a>, <a href="/articles/linux-setuid-setgid-sticky">LNX-026</a> (bit setgid của thư mục log). Trạng thái đầu bài: <code>npm run setup -- 27</code>. Gói <code>acl</code> có sẵn trên Ubuntu; hệ thống file ext4, xfs, btrfs đều hỗ trợ ACL mặc định.</p>
</div>

## 1. Khi ba lớp quyền không đủ

```text
an@toinoi-app-01:~$ sudo useradd --create-home --shell /bin/bash --comment "Kiem toan noi bo" kiemtoan
an@toinoi-app-01:~$ id kiemtoan
uid=1003(kiemtoan) gid=1003(kiemtoan) groups=1003(kiemtoan)
an@toinoi-app-01:~$ ls -ld /srv/toinoi/bao-gia/shared/logs
drwxrws--- 2 an toinoi 4096 Nov 23 23:36 /srv/toinoi/bao-gia/shared/logs
an@toinoi-app-01:~$ sudo -u kiemtoan ls /srv/toinoi/bao-gia/shared/logs
ls: cannot open directory '/srv/toinoi/bao-gia/shared/logs': Permission denied
an@toinoi-app-01:~$ getfacl -p /srv/toinoi/bao-gia/shared/logs
user::rwx
group::rwx
other::---
```

- Tài khoản `kiemtoan` không thuộc `toinoi`, nên rơi vào lớp «người khác»: `---`, `Permission denied`.
- **`getfacl`** in quyền của file dưới dạng danh sách. Khi chưa có ACL, danh sách chỉ có ba dòng tương ứng với `rwx` quen thuộc: `user::` (chủ), `group::` (nhóm), `other::`. `flags: -s-` là bit setgid.
- `-p` giữ dấu `/` đầu đường dẫn tuyệt đối (mặc định `getfacl` bỏ nó đi và in một cảnh báo).

## 2. Cấp quyền cho đúng một người

```bash
setfacl -m u:kiemtoan:rx /srv/toinoi/bao-gia/shared/logs
```

**`setfacl -m`** (*modify*) thêm hoặc sửa một mục: `u:TÊN:QUYỀN` cho người dùng, `g:NHÓM:QUYỀN` cho nhóm. Chủ file tự làm được, không cần `sudo`.

```text
an@toinoi-app-01:~$ setfacl -m u:kiemtoan:rx /srv/toinoi/bao-gia/shared/logs
an@toinoi-app-01:~$ getfacl -p /srv/toinoi/bao-gia/shared/logs
user::rwx
user:kiemtoan:r-x
group::rwx
mask::rwx
other::---

an@toinoi-app-01:~$ ls -ld /srv/toinoi/bao-gia/shared/logs
drwxrws---+ 2 an toinoi 4096 Nov 23 23:36 /srv/toinoi/bao-gia/shared/logs
an@toinoi-app-01:~$ sudo -u kiemtoan ls /srv/toinoi/bao-gia/shared/logs
bao-cao-2026-11-21.txt  hang-ngay.log       loi-theo-dich-vu.txt
bao-cao-2026-11-23.txt  loi-2026-11-21.txt
an@toinoi-app-01:~$ sudo -u kiemtoan head -n 3 /srv/toinoi/bao-gia/shared/logs/loi-theo-dich-vu.txt
      6 dich_vu=car4
      5 dich_vu=bike
      4 dich_vu=car7
an@toinoi-app-01:~$ sudo -u kiemtoan touch /srv/toinoi/bao-gia/shared/logs/thu.txt
touch: cannot touch '/srv/toinoi/bao-gia/shared/logs/thu.txt': Permission denied
```

- `getfacl` có thêm hai dòng: **`user:kiemtoan:r-x`** và **`mask::rwx`** (mục 4 nói về mask).
- `ls -ld` có thêm dấu **`+`** cuối chuỗi quyền: «file này có ACL, `ls` không hiển thị hết». Thấy `+` là phải dùng `getfacl`.
- `kiemtoan` liệt kê được thư mục, đọc được file bên trong (file log có quyền `r` cho người khác, hoặc thuộc nhóm). Ghi thì không: chỉ có `r-x`.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-acl-setfacl/ssh-studio-01.webp" alt="Terminal của SSH Studio: getfacl của thư mục logs ban đầu chỉ có user::rwx group::rwx other::---, sau setfacl -m u:kiemtoan:rx getfacl có user:kiemtoan:r-x và mask::rwx, ls -ld hiện drwxrws---+, kiemtoan liệt kê được file log và đọc được loi-theo-dich-vu.txt nhưng touch bị từ chối" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>setfacl -m u:kiemtoan:rx</code>: một người, đúng quyền đọc</figcaption></figure>

Thử lại thư mục cha: để vào được `logs`, `kiemtoan` cũng cần `x` trên `/srv/toinoi/bao-gia/shared` và các thư mục phía trên. Trên lab, các thư mục đó cho người khác `x`; trên server thật, kiểm tra cả đường dẫn bằng `namei -l ĐƯỜNG_DẪN`.

## 3. ACL mặc định cho file mới

ACL ở trên chỉ áp dụng cho **chính thư mục**. File mới tạo trong đó không tự có mục `kiemtoan`. Nếu file log mới có quyền `rw-rw----` (không cho người khác đọc), `kiemtoan` sẽ không đọc được nó. **ACL mặc định** (*default ACL*) giải quyết việc đó: nó là khuôn cho mọi file và thư mục con được tạo sau.

```text
an@toinoi-app-01:~$ setfacl -d -m u:kiemtoan:r /srv/toinoi/bao-gia/shared/logs
an@toinoi-app-01:~$ touch /srv/toinoi/bao-gia/shared/logs/moi.txt
an@toinoi-app-01:~$ getfacl -p /srv/toinoi/bao-gia/shared/logs/moi.txt
user::rw-
user:kiemtoan:r--
group::rwx                      #effective:rw-
mask::rw-
other::---

an@toinoi-app-01:~$ ls -l /srv/toinoi/bao-gia/shared/logs/moi.txt
-rw-rw----+ 1 an toinoi 0 Nov 23 23:36 /srv/toinoi/bao-gia/shared/logs/moi.txt
```

- `-d` (*default*) đặt ACL mặc định cho thư mục.
- `moi.txt` sinh ra đã có `user:kiemtoan:r--`, và dấu `+`.
- Dòng `group::rwx #effective:rw-`: mục nhóm ghi `rwx`, nhưng quyền **thật sự** chỉ là `rw-`. Đó là việc của mask.

## 4. mask: trần của mọi quyền ACL

**`mask`** là giới hạn trên cho mọi mục ACL **trừ** chủ file và «người khác»: mục `user:TÊN`, `group:NHÓM` và cả `group::` của nhóm sở hữu. Quyền hiệu lực = quyền của mục **AND** mask.

Điều dễ vấp: khi file có ACL, **cột quyền nhóm trong `ls -l` hiển thị mask**, không phải quyền của nhóm, và `chmod g=…` sửa **mask**. Một lệnh `chmod` tưởng vô hại có thể vô hiệu toàn bộ ACL:

```text
an@toinoi-app-01:~$ chmod 600 /srv/toinoi/bao-gia/shared/logs/moi.txt
an@toinoi-app-01:~$ getfacl -p /srv/toinoi/bao-gia/shared/logs/moi.txt
user::rw-
user:kiemtoan:r--               #effective:---
group::rwx                      #effective:---
mask::---
other::---

an@toinoi-app-01:~$ sudo -u kiemtoan cat /srv/toinoi/bao-gia/shared/logs/moi.txt
cat: /srv/toinoi/bao-gia/shared/logs/moi.txt: Permission denied
```

`chmod 600` đặt mask thành `---`. Mục `user:kiemtoan:r--` vẫn còn đó, nhưng `#effective:---`, và `kiemtoan` bị từ chối. Mục ACL không mất, chỉ bị «bóp» về không; `chmod 640` sẽ trả lại quyền đọc.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-acl-setfacl/ssh-studio-02.webp" alt="Terminal của SSH Studio: setfacl -d -m u:kiemtoan:r, file moi.txt mới tạo có user:kiemtoan:r-- và group::rwx #effective:rw- với mask::rw-, ls -l hiện -rw-rw----+; sau chmod 600 mask thành --- và cả user:kiemtoan lẫn group đều #effective:---, kiemtoan cat bị Permission denied" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · ACL mặc định cho file mới, và <code>chmod</code> hạ mask làm ACL mất tác dụng</figcaption></figure>

## 5. Gỡ ACL

Hết tuần kiểm toán, thu hồi:

```text
an@toinoi-app-01:~$ setfacl -k /srv/toinoi/bao-gia/shared/logs
an@toinoi-app-01:~$ setfacl -x u:kiemtoan /srv/toinoi/bao-gia/shared/logs
an@toinoi-app-01:~$ getfacl -p /srv/toinoi/bao-gia/shared/logs
user::rwx
group::rwx
mask::rwx
other::---

an@toinoi-app-01:~$ setfacl -b /srv/toinoi/bao-gia/shared/logs
an@toinoi-app-01:~$ ls -ld /srv/toinoi/bao-gia/shared/logs
drwxrws--- 2 an toinoi 4096 Nov 23 23:36 /srv/toinoi/bao-gia/shared/logs
an@toinoi-app-01:~$ sudo -u kiemtoan ls /srv/toinoi/bao-gia/shared/logs
ls: cannot open directory '/srv/toinoi/bao-gia/shared/logs': Permission denied
```

| Lệnh | Gỡ gì |
|---|---|
| `setfacl -x u:kiemtoan DIR` | Một mục của thư mục |
| `setfacl -k DIR` | Toàn bộ ACL **mặc định** |
| `setfacl -b DIR` | Mọi mục ACL mở rộng (cả mask), về lại `rwx` thuần |

Sau `-x`, `mask::rwx` vẫn còn và `ls` vẫn có `+`; `-b` dọn sạch. Gỡ ACL trên thư mục **không** gỡ ACL đã sinh ra trên các file con; dùng `setfacl -R -x u:kiemtoan DIR` (`-R`: đệ quy) nếu cần. Một cách dễ nhớ hơn cho quyền tạm thời: cấp qua một nhóm (`g:kiemtoan-tam:rx`), rồi thu hồi bằng cách xoá người khỏi nhóm.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-acl-setfacl/ssh-studio-03.webp" alt="Terminal của SSH Studio: setfacl -k gỡ ACL mặc định, setfacl -x u:kiemtoan gỡ mục kiemtoan, getfacl còn user::rwx group::rwx mask::rwx other::---, setfacl -b, ls -ld trở lại drwxrws--- không có dấu cộng, kiemtoan lại bị Permission denied" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · gỡ ACL bằng <code>-k</code>, <code>-x</code>, <code>-b</code></figcaption></figure>

## 6. ACL và các công cụ khác

- **Sao lưu**: `cp -a` và `rsync -A` giữ ACL; `tar` cần `--acls` khi tạo và giải nén ([LNX-018](/articles/linux-tar-gzip-nen-luu-tru)).
- **Sao lưu và khôi phục ACL riêng**: `getfacl -R -p DIR > acl.txt`, rồi `setfacl --restore=acl.txt`.
- **Samba, NFSv4** có mô hình ACL riêng, phong phú hơn; ACL POSIX ở đây là của hệ thống file Linux.

## 7. Đối chiếu thao tác

| Việc | Lệnh |
|---|---|
| Xem ACL | `getfacl -p FILE` |
| Cấp cho một người | `setfacl -m u:TÊN:rx FILE` |
| Cấp cho một nhóm | `setfacl -m g:NHÓM:rx FILE` |
| Khuôn cho file mới | `setfacl -d -m u:TÊN:r DIR` |
| Áp cho cả cây | thêm `-R` |
| Gỡ một mục / mặc định / tất cả | `setfacl -x u:TÊN` / `-k` / `-b` |

## 8. Lỗi thường gặp

**Không thấy dấu `+`.** `ls -l` không hiển thị ACL; ai chỉ nhìn `rwx` sẽ hiểu sai quyền thật. Thấy `+` là `getfacl`.

**Đặt ACL cho thư mục mà quên file bên trong.** `-m` trên thư mục không áp cho file đã có; cần `-R` cho file cũ và `-d` cho file mới.

**`chmod` sau khi đặt ACL.** `chmod g-…` hạ mask, làm mọi mục ACL mất tác dụng mà không báo gì.

**Dùng ACL cho mọi thứ.** ACL khó nhìn, khó kiểm toán. Thứ đầu tiên vẫn là chủ/nhóm hợp lý (LNX-026); ACL cho ngoại lệ.

## Bài tập

1. Cấp cho nhóm `toinoi` quyền `rx` trên `/home/an/tai-lieu` bằng ACL thay vì `chmod`. Bình (thuộc `toinoi`) đọc được chưa? Cần thêm quyền gì trên `/home/an`?
2. Đặt `setfacl -m u:kiemtoan:rwx` trên một file, rồi `chmod 640`. `getfacl` hiện `#effective` gì cho `kiemtoan`?
3. Sao lưu ACL của `/srv/toinoi` bằng `getfacl -R -p`, gỡ hết bằng `setfacl -R -b`, rồi khôi phục bằng `setfacl --restore`.
4. Đặt ACL mặc định cho `kiemtoan` trên thư mục log, rồi tạo một **thư mục con**. Thư mục con có ACL thường và ACL mặc định không?

## Đọc tiếp

[LNX-028 · ssh-agent và passphrase](/articles/linux-ssh-agent-passphrase): bảo vệ private key bằng passphrase mà không phải gõ nó mỗi lần.
