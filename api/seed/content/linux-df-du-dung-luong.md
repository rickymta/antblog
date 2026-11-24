---
slug: linux-df-du-dung-luong
title: "Dung lượng đĩa: df, du, tìm thứ chiếm chỗ, hết inode khi còn chỗ và file đã xoá vẫn giữ chỗ"
summary: "Đọc df -h và df -T, tìm thư mục và file lớn bằng du và find, một file log gỡ lỗi 400 MB bị quên, tình huống «No space left on device» khi df vẫn báo còn trống vì hết inode (df -i), và file đã xoá mà dung lượng không về vì một tiến trình vẫn mở nó, tìm bằng lsof +L1."
category: devops-linux
level: beginner
tags: [linux, lnx-043, disk, df, du, inode, lsof, find, ssh-studio, toi-noi]
language: vi
---

Cảnh báo lúc 3 giờ sáng: «`/` đã dùng 92%». Đĩa đầy là một trong những nguyên nhân phổ biến nhất làm server hỏng toàn bộ: dịch vụ không ghi được log, cơ sở dữ liệu dừng, thậm chí không đăng nhập được. An cần trả lời nhanh ba câu: hệ thống file nào đầy, cái gì chiếm chỗ, và vì sao xoá rồi mà vẫn đầy.

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-043 · Phần 5/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/linux-tim-kiem-find-grep">LNX-009</a> (<code>find</code>). Trạng thái đầu bài: <code>npm run setup -- 43</code>. Lab chạy trong Docker: hệ thống file gốc <code>overlay</code> thực ra là đĩa của máy chạy Docker (hàng trăm GB), nên con số tuyệt đối không giống server thật. Hai tình huống hết inode và file đã xoá được dựng trên <code>tmpfs</code> nhỏ để thấy rõ.</p>
</div>

## 1. df: mỗi hệ thống file còn bao nhiêu

```text
an@toinoi-app-01:~$ df -h
Filesystem      Size  Used Avail Use% Mounted on
overlay        1007G  8.5G  948G   1% /
tmpfs            64M     0   64M   0% /dev
shm              64M     0   64M   0% /dev/shm
tmpfs            16G   56K   16G   1% /run
tmpfs            16G  4.0K   16G   1% /tmp
/dev/sdd       1007G  8.5G  948G   1% /etc/hosts
tmpfs            16G     0   16G   0% /run/lock
an@toinoi-app-01:~$ df -hT / /tmp /run
Filesystem     Type     Size  Used Avail Use% Mounted on
overlay        overlay 1007G  8.5G  948G   1% /
tmpfs          tmpfs     16G  4.0K   16G   1% /tmp
tmpfs          tmpfs     16G   56K   16G   1% /run
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-df-du-dung-luong/ssh-studio-04.webp" alt="Terminal của SSH Studio: df -h liệt kê overlay ở /, các tmpfs /dev, /run, /tmp và /etc/hosts trên /dev/sdd; df -hT / /tmp /run thêm cột Type" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>df</code>: mỗi hệ thống file còn bao nhiêu chỗ</figcaption></figure>

- **`df -h`**: mỗi **hệ thống file** đang gắn (cột `Mounted on`), tổng, đã dùng, còn trống, phần trăm. `-h` đổi ra K, M, G.
- **`-T`** thêm loại: `overlay` (lớp file của container), `tmpfs` (nằm trong RAM, mất khi khởi động lại), `ext4` (đĩa thật, sẽ gặp ở [LNX-047](/articles/linux-dia-phan-vung-mount)).
- Một server thật thường có `/` trên `ext4` hoặc `xfs`, có thể có thêm `/var` hoặc `/data` trên đĩa riêng. Câu hỏi đầu tiên luôn là: **đĩa nào** đầy?

## 2. du và find: cái gì chiếm chỗ

```text
an@toinoi-app-01:~$ sudo du -sh /var/log /srv /opt /home /usr
33M     /var/log
401M    /srv
28K     /opt
412K    /home
331M    /usr
an@toinoi-app-01:~$ sudo du -xh --max-depth=2 /srv | sort -h | tail -n 5
4.0K    /srv/toinoi/chung
36K     /srv/toinoi/sao-luu
401M    /srv
401M    /srv/toinoi
401M    /srv/toinoi/bao-gia
an@toinoi-app-01:~$ sudo find / -xdev -type f -size +100M -printf '%s\t%p\n' 2>/dev/null | sort -n | tail -n 3
419430400       /srv/toinoi/bao-gia/shared/logs/debug-2026-11.log
536870912       /srv/dia/du-lieu-01.img
an@toinoi-app-01:~$ ls -lh /srv/toinoi/bao-gia/shared/logs/debug-2026-11.log
-rw-r----- 1 baogia toinoi 400M Nov 23 23:59 /srv/toinoi/bao-gia/shared/logs/debug-2026-11.log
```

- **`du -sh`**: tổng dung lượng của từng thư mục. `sudo` để đọc được cả thư mục của người khác.
- **`du -xh --max-depth=2 | sort -h`**: đi xuống từng tầng, sắp theo cỡ ([LNX-013](/articles/linux-cut-sort-uniq-tr)). `-x`: không đi sang hệ thống file khác (ví dụ không tính `/proc` hay một ổ gắn ở `/srv/data`).
- **`find / -xdev -size +100M`**: thẳng vào các file lớn. Ở đây có hai: ảnh đĩa của bài 47 (dự tính), và **`debug-2026-11.log`** 400 MB, file log gỡ lỗi ai đó bật rồi quên tắt.

Trước khi xoá, hỏi chủ của nó (ở đây là Bình). Với log đang được ghi, xoá file chưa chắc đã trả lại chỗ (mục 4); cách an toàn là làm rỗng nó (`sudo truncate -s 0 FILE`) hoặc để logrotate ([LNX-039](/articles/linux-logrotate)) lo.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-df-du-dung-luong/ssh-studio-01.webp" alt="Terminal của SSH Studio: sudo du -sh cho /srv 401M lớn nhất, du --max-depth=2 /srv chỉ ra /srv/toinoi/bao-gia, find -size +100M tìm debug-2026-11.log 400M và du-lieu-01.img, ls -lh xác nhận file log 400M" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>du</code> và <code>find</code> chỉ ra file log debug 400 MB</figcaption></figure>

## 3. Hết inode khi vẫn còn chỗ

Mỗi file (và thư mục) tốn một **inode**: bản ghi lưu chủ sở hữu, quyền, thời gian, vị trí dữ liệu. Số inode của một hệ thống file được định sẵn lúc tạo. Hàng triệu file nhỏ (phiên làm việc, cache, hàng đợi email) có thể dùng hết inode trong khi dung lượng còn nhiều.

An dựng một `tmpfs` 20 MB chỉ có 64 inode:

```text
an@toinoi-app-01:~$ sudo mkdir -p /mnt/nho
an@toinoi-app-01:~$ sudo mount -t tmpfs -o size=20m,nr_inodes=64 tmpfs /mnt/nho
an@toinoi-app-01:~$ sudo chown an:an /mnt/nho
an@toinoi-app-01:~$ for i in $(seq 1 100); do touch /mnt/nho/f$i 2>/dev/null || { echo "khong tao duoc file thu $i"; break; }; done
khong tao duoc file thu 64
an@toinoi-app-01:~$ df -h /mnt/nho
Filesystem      Size  Used Avail Use% Mounted on
tmpfs            20M     0   20M   0% /mnt/nho
an@toinoi-app-01:~$ df -i /mnt/nho
Filesystem     Inodes IUsed IFree IUse% Mounted on
tmpfs              64    64     0  100% /mnt/nho
an@toinoi-app-01:~$ touch /mnt/nho/them-mot-file
touch: cannot touch '/mnt/nho/them-mot-file': No space left on device
an@toinoi-app-01:~$ sudo umount /mnt/nho
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-df-du-dung-luong/ssh-studio-05.webp" alt="Terminal của SSH Studio: tạo /mnt/nho, mount tmpfs size=20m,nr_inodes=64, chown cho an, vòng for touch 100 file dừng ở file thứ 64" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · hệ thống file nhỏ chỉ có 64 inode</figcaption></figure>

- Tới file thứ 64 thì không tạo được nữa. (`tmpfs` dùng một inode cho chính thư mục gốc.)
- **`df -h`**: dùng 0%, còn 20 MB. **`df -i`**: `IUse% 100%`. Thông báo vẫn là `No space left on device`, thông báo chung cho cả hai kiểu đầy.

Khi gặp «No space left» mà `df -h` còn trống, chạy `df -i`. Tìm thư mục có nhiều file nhất bằng `sudo find / -xdev -type d -exec sh -c 'echo $(ls -A "$1" | wc -l) "$1"' _ {} \; | sort -n | tail`, hoặc đơn giản hơn với `du --inodes` (có trên coreutils mới).

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-df-du-dung-luong/ssh-studio-02.webp" alt="Terminal của SSH Studio: df -h /mnt/nho còn 20M trống (0% dùng), df -i cho 64 inode đã dùng hết (100%), touch báo No space left on device, umount /mnt/nho" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · còn chỗ nhưng hết inode: <code>df -i</code> mới thấy</figcaption></figure>

## 4. File đã xoá mà vẫn giữ chỗ

```text
an@toinoi-app-01:~$ sudo mkdir -p /mnt/thu
an@toinoi-app-01:~$ sudo mount -t tmpfs -o size=100m tmpfs /mnt/thu
an@toinoi-app-01:~$ sudo chown an:an /mnt/thu
an@toinoi-app-01:~$ head -c 60M /dev/zero > /mnt/thu/log-lon.log
an@toinoi-app-01:~$ nohup tail -f /mnt/thu/log-lon.log > /dev/null 2>&1 &
an@toinoi-app-01:~$ df -h /mnt/thu
Filesystem      Size  Used Avail Use% Mounted on
tmpfs           100M   60M   40M  60% /mnt/thu
an@toinoi-app-01:~$ rm /mnt/thu/log-lon.log
an@toinoi-app-01:~$ ls /mnt/thu
an@toinoi-app-01:~$ df -h /mnt/thu
Filesystem      Size  Used Avail Use% Mounted on
tmpfs           100M   60M   40M  60% /mnt/thu
an@toinoi-app-01:~$ lsof -nP +L1 2>/dev/null | grep -E 'COMMAND|/mnt/thu'
COMMAND  PID USER   FD   TYPE DEVICE SIZE/OFF NLINK NODE NAME
tail    4594   an    3r   REG  0,103 62914560     0    2 /mnt/thu/log-lon.log (deleted)
an@toinoi-app-01:~$ pkill -f 'tail -f /mnt/thu'
an@toinoi-app-01:~$ df -h /mnt/thu
Filesystem      Size  Used Avail Use% Mounted on
tmpfs           100M     0  100M   0% /mnt/thu
an@toinoi-app-01:~$ sudo umount /mnt/thu
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-df-du-dung-luong/ssh-studio-06.webp" alt="Terminal của SSH Studio: mount tmpfs 100m ở /mnt/thu, tạo log-lon.log 60M, tail -f giữ file, df -h 60% dùng, rm log-lon.log, ls trống nhưng df vẫn 60M dùng" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · xoá file mà dung lượng không trả lại</figcaption></figure>

1. Một file log 60 MB, đang được một tiến trình (`tail -f`) mở để đọc.
2. `rm` xoá file: `ls` không còn thấy nó.
3. **`df` vẫn báo 60 MB đã dùng.** `rm` chỉ xoá **tên** (liên kết) của file. Dữ liệu chỉ được giải phóng khi không còn tên nào **và** không còn tiến trình nào mở nó.
4. **`lsof +L1`** liệt kê file đang mở có số liên kết nhỏ hơn 1, tức file đã bị xoá: cột `NAME` ghi `(deleted)`, cột `PID` chỉ ra tiến trình giữ nó.
5. Dừng tiến trình đó (hoặc khởi động lại dịch vụ), dung lượng về ngay.

Đây là chuyện hằng ngày trên server: ai đó xoá file log lớn của một dịch vụ đang chạy để «giải phóng chỗ», và chẳng có gì được giải phóng. Dịch vụ vẫn ghi tiếp vào file không còn tên. Cách đúng: `truncate -s 0` thay vì `rm`, hoặc `rm` rồi `systemctl reload`/`restart` dịch vụ ([LNX-039](/articles/linux-logrotate)).

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-df-du-dung-luong/ssh-studio-03.webp" alt="Terminal của SSH Studio: lsof +L1 cho tiến trình tail giữ log-lon.log (deleted), pkill tail, df -h /mnt/thu trở về 0 dùng, umount /mnt/thu" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>lsof +L1</code> tìm tiến trình giữ file đã xoá</figcaption></figure>

## 5. Đối chiếu thao tác

| Câu hỏi | Lệnh |
|---|---|
| Hệ thống file nào đầy | `df -h`, `df -hT` |
| Hết inode chưa | `df -i` |
| Thư mục nào lớn | `sudo du -xh --max-depth=1 / \| sort -h` |
| File nào lớn | `sudo find / -xdev -type f -size +100M` |
| File đã xoá còn bị giữ | `sudo lsof -nP +L1` |
| Làm rỗng log đang ghi | `sudo truncate -s 0 FILE` |

Trong SSH Studio, khung **Files** cũng hiện cỡ từng file, hữu ích khi đã biết thư mục cần xem.

## 6. Lỗi thường gặp

**`rm` log của dịch vụ đang chạy.** Chỗ không về; dùng `truncate` hoặc khởi động lại dịch vụ.

**Chạy `du` không có `-x` trên `/`.** Đi cả vào `/proc`, ổ mạng, ổ gắn thêm: chậm và số liệu sai.

**Chỉ xem `df -h`.** Bỏ qua hết inode.

**Đợi tới 100% mới xử lý.** Đặt cảnh báo ở khoảng 80–85%; [LNX-050](/articles/linux-thuc-hanh-mang-giam-sat) ghi dung lượng vào báo cáo định kỳ.

## Bài tập

1. `sudo du -xh --max-depth=1 /var | sort -h`. Thư mục nào lớn nhất trong `/var`? Vì sao?
2. Hỏi Bình xong, làm rỗng `debug-2026-11.log` bằng `sudo truncate -s 0` (đừng `rm`). `ls -l` và `du -sh /srv` thay đổi thế nào?
3. `stat` một file bất kỳ: dòng `Inode:` và `Links:` là gì? Tạo hard link bằng `ln FILE FILE2`, `stat` lại.
4. `df -i /` trên server lab. Bao nhiêu inode, đã dùng bao nhiêu?

## Đọc tiếp

[LNX-044 · Port forwarding trong SSH Studio](/articles/linux-ssh-port-forwarding): mở dịch vụ chỉ nghe ở `127.0.0.1` của server ngay trên máy bạn, qua SSH.
