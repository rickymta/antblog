---
slug: linux-dia-phan-vung-mount
title: "Đĩa, phân vùng, hệ thống file và mount: gắn ổ dữ liệu mới, fstab theo UUID và nofail"
summary: "Thiết bị khối, phân vùng và hệ thống file khác nhau ở đâu, nhận một ổ dữ liệu mới (lsblk, blkid), tạo hệ thống file ext4 có nhãn, gắn vào /du-lieu bằng mount, ghi vào /etc/fstab theo UUID với nofail để server vẫn khởi động khi ổ vắng mặt, kiểm tra bằng mount -a và findmnt trước khi khởi động lại."
category: devops-linux
level: intermediate
tags: [linux, lnx-047, disk, mount, fstab, uuid, ext4, lsblk, ssh-studio, toi-noi]
language: vi
---

Bản sao lưu release và log của `bao-gia` đang nằm chung ổ với hệ điều hành. Ổ hệ thống đầy là cả server dừng ([LNX-043](/articles/linux-df-du-dung-luong)). Bình nhờ nhà cung cấp gắn thêm một ổ 512 MB cho dữ liệu, và giao cho An phần còn lại: làm cho ổ mới dùng được, và vẫn dùng được sau mỗi lần khởi động lại.

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-047 · Phần 5/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/linux-df-du-dung-luong">LNX-043</a>, <a href="/articles/linux-cay-thu-muc-duong-dan">LNX-004</a> (một cây thư mục duy nhất). Trạng thái đầu bài: <code>npm run setup -- 47</code>. Lab mô phỏng «ổ mới» bằng một file ảnh 512 MB gắn thành <b>loop device</b>; tên thiết bị (<code>/dev/loop2</code> trong bài) tuỳ máy. Trên máy ảo thật, ổ mới thường là <code>/dev/sdb</code>, <code>/dev/vdb</code> hay <code>/dev/nvme1n1</code>.</p>
</div>

## 1. Ba tầng: thiết bị, phân vùng, hệ thống file

| Tầng | Là gì | Ví dụ | Lệnh |
|---|---|---|---|
| **Thiết bị khối** | Một ổ, nhìn như một dãy byte | `/dev/sdb`, `/dev/loop2` | `lsblk` |
| **Phân vùng** | Một đoạn của ổ, ghi trong bảng phân vùng (GPT) | `/dev/sdb1`, `/dev/nvme1n1p1` | `fdisk`, `parted` |
| **Hệ thống file** | Cấu trúc để lưu file và thư mục trên thiết bị hoặc phân vùng | `ext4`, `xfs` | `mkfs.ext4`, `blkid` |

Rồi **mount** gắn một hệ thống file vào một thư mục của cây duy nhất ([LNX-004](/articles/linux-cay-thu-muc-duong-dan)): sau đó, file trong `/du-lieu` nằm trên ổ mới.

Ổ hệ điều hành thường được chia phân vùng (phân vùng khởi động, phân vùng gốc…). Ổ **dữ liệu** gắn thêm thì thường định dạng **cả ổ**, không chia phân vùng: đơn giản hơn và dễ nới rộng sau này. Bài làm theo cách đó.

```text
an@toinoi-app-01:~$ findmnt -lo TARGET,SOURCE,FSTYPE,SIZE -t overlay,tmpfs
TARGET    SOURCE  FSTYPE     SIZE
/         overlay overlay 1006.9G
/dev      tmpfs   tmpfs       64M
/dev/shm  shm     tmpfs       64M
/run      tmpfs   tmpfs     15.6G
/tmp      tmpfs   tmpfs     15.6G
/run/lock tmpfs   tmpfs     15.6G
an@toinoi-app-01:~$ grep -v '^#' /etc/fstab

an@toinoi-app-01:~$ DIA=$(sudo losetup -j /srv/dia/du-lieu-01.img -n -O NAME)
an@toinoi-app-01:~$ echo $DIA
/dev/loop2
an@toinoi-app-01:~$ lsblk -o NAME,SIZE,TYPE,FSTYPE,MOUNTPOINTS $DIA
NAME   SIZE TYPE FSTYPE MOUNTPOINTS
loop2  512M loop
an@toinoi-app-01:~$ sudo blkid $DIA || echo "chua co he thong file"
chua co he thong file
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-dia-phan-vung-mount/ssh-studio-04.webp" alt="Terminal của SSH Studio: findmnt -lo TARGET,SOURCE,FSTYPE,SIZE -t overlay,tmpfs liệt kê / overlay 1006.9G và các tmpfs /dev, /dev/shm, /run, /tmp, /run/lock; grep /etc/fstab không có dòng nào" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · các hệ thống file đang được gắn, fstab còn trống</figcaption></figure>

- `findmnt` liệt kê hệ thống file đã gắn ở đâu; `-l` in dạng danh sách thay vì cây, `-t` lọc theo loại. Trong lab, `/` là `overlay` (ổ gốc của container); trên máy ảo thật nó thường là `ext4` hay `xfs` trên `/dev/sda1`. Các `tmpfs` nằm trong RAM. `/etc/fstab` của lab chưa có dòng nào.
- Ổ mới xuất hiện: `lsblk` thấy thiết bị 512 MB, chưa có `FSTYPE`, chưa gắn đâu. `blkid` không đọc ra gì: ổ trống.
- `DIA=$(…)` lưu tên thiết bị vào một biến shell, để các lệnh sau không phải gõ lại (và để bài chạy đúng dù trên máy bạn nó là `loop3`).

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-dia-phan-vung-mount/ssh-studio-01.webp" alt="Terminal của SSH Studio: DIA=$(sudo losetup -j … -O NAME) cho /dev/loop2, lsblk cho loop2 512M loại loop chưa có FSTYPE, blkid báo chua co he thong file" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · ổ mới 512 MB: có thiết bị nhưng chưa có hệ thống file</figcaption></figure>

## 2. Tạo hệ thống file và gắn vào

```text
an@toinoi-app-01:~$ sudo mkfs.ext4 -q -L du-lieu $DIA
an@toinoi-app-01:~$ sudo blkid $DIA
/dev/loop2: LABEL="du-lieu" UUID="eb774cd9-4027-4678-9eeb-047a2bfe097a" BLOCK_SIZE="4096" TYPE="ext4"
an@toinoi-app-01:~$ sudo mkdir -p /du-lieu
an@toinoi-app-01:~$ sudo mount $DIA /du-lieu
an@toinoi-app-01:~$ df -hT /du-lieu
Filesystem     Type  Size  Used Avail Use% Mounted on
/dev/loop2     ext4  488M   24K  452M   1% /du-lieu
an@toinoi-app-01:~$ ls -la /du-lieu
total 24
drwxr-xr-x 3 root root  4096 Nov 24 08:12 .
drwxr-xr-x 1 root root  4096 Nov 24 08:12 ..
drwx------ 2 root root 16384 Nov 24 08:12 lost+found
an@toinoi-app-01:~$ sudo chown an:toinoi /du-lieu
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-dia-phan-vung-mount/ssh-studio-05.webp" alt="Terminal của SSH Studio: sudo mkfs.ext4 -q -L du-lieu $DIA, blkid cho /dev/loop2 LABEL du-lieu, UUID và TYPE ext4" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>mkfs.ext4</code> tạo hệ thống file, <code>blkid</code> đọc nhãn và UUID</figcaption></figure>

- **`mkfs.ext4 -L du-lieu`** tạo hệ thống file `ext4`, đặt **nhãn** `du-lieu`. **Mọi dữ liệu cũ trên thiết bị bị xoá**: kiểm tra đúng tên thiết bị trước khi chạy.
- **`blkid`** giờ thấy nhãn, loại, và **UUID**: mã định danh duy nhất sinh lúc tạo hệ thống file.
- **`mount THIẾT_BỊ THƯ_MỤC`** gắn vào (thư mục phải có sẵn). `df -hT` thấy 488 MB dùng được (phần còn lại là bảng inode, nhật ký của ext4 và 5% để dành cho `root`).
- `lost+found` là thư mục của `fsck`, nơi chứa mảnh file cứu được sau lỗi. Thư mục gốc của ổ mới thuộc `root`; An đổi cho `an:toinoi`.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-dia-phan-vung-mount/ssh-studio-02.webp" alt="Terminal của SSH Studio: mkdir /du-lieu, mount $DIA /du-lieu, df -hT cho /dev/loop2 ext4 488M, ls -la thấy lost+found, chown an:toinoi /du-lieu" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · gắn ổ vào <code>/du-lieu</code> và giao cho An</figcaption></figure>

## 3. Gắn tự động khi khởi động: fstab

`mount` bằng tay mất khi khởi động lại. Danh sách gắn lúc boot nằm trong **`/etc/fstab`**, mỗi dòng sáu cột:

```text
UUID=eb774cd9-…  /du-lieu  ext4  defaults,nofail  0  2
thiết bị          gắn ở     loại  tuỳ chọn         dump  thứ tự fsck
```

```text
an@toinoi-app-01:~$ UUID=$(sudo blkid -s UUID -o value $DIA)
an@toinoi-app-01:~$ echo "UUID=$UUID /du-lieu ext4 defaults,nofail 0 2" | sudo tee -a /etc/fstab
UUID=eb774cd9-4027-4678-9eeb-047a2bfe097a /du-lieu ext4 defaults,nofail 0 2
an@toinoi-app-01:~$ sudo umount /du-lieu
an@toinoi-app-01:~$ findmnt /du-lieu || echo "chua gan"
chua gan
an@toinoi-app-01:~$ sudo systemctl daemon-reload
an@toinoi-app-01:~$ sudo mount -a
an@toinoi-app-01:~$ findmnt -o TARGET,SOURCE,FSTYPE,OPTIONS /du-lieu
TARGET   SOURCE     FSTYPE OPTIONS
/du-lieu /dev/loop2 ext4   rw,relatime
an@toinoi-app-01:~$ echo "luu tren o du lieu" > /du-lieu/thu.txt
an@toinoi-app-01:~$ ls -l /du-lieu
total 20
drwx------ 2 root root 16384 Nov 24 08:12 lost+found
-rw-rw-r-- 1 an   an      19 Nov 24 08:12 thu.txt
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-dia-phan-vung-mount/ssh-studio-06.webp" alt="Terminal của SSH Studio: lấy UUID bằng blkid -s UUID -o value, sudo tee -a thêm dòng UUID=… /du-lieu ext4 defaults,nofail 0 2 vào fstab, umount /du-lieu, findmnt báo chua gan" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · thêm dòng fstab theo UUID, gỡ ổ ra để thử</figcaption></figure>

- **Theo UUID, không theo tên thiết bị.** Tên `/dev/sdb`, `/dev/loop2` có thể đổi giữa các lần khởi động (thêm ổ khác, đổi thứ tự). UUID đi theo hệ thống file.
- **`nofail`**: nếu ổ vắng mặt lúc boot, server vẫn khởi động, chỉ không có `/du-lieu`. Thiếu nó, một ổ dữ liệu hỏng có thể khiến server dừng ở chế độ khẩn cấp: đúng loại sự cố không SSH vào được để sửa. Lab rất cần `nofail`: loop device mất khi container khởi động lại.
- Cột 6 = `2`: kiểm tra bằng `fsck` sau ổ gốc.
- **Thử trước khi khởi động lại**: `umount`, rồi `sudo mount -a` gắn mọi thứ trong `fstab` chưa được gắn. Sai cú pháp, sai UUID sẽ lộ ra ngay ở đây, chứ không phải lúc boot. `systemctl daemon-reload` vì systemd đọc `fstab` thành các unit `.mount`.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-dia-phan-vung-mount/ssh-studio-03.webp" alt="Terminal của SSH Studio: systemctl daemon-reload, mount -a gắn lại từ fstab, findmnt cho /du-lieu /dev/loop2 ext4 rw,relatime, ghi thu.txt rồi ls -l /du-lieu" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>mount -a</code> đọc fstab và gắn lại ổ</figcaption></figure>

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-dia-phan-vung-mount/ssh-studio-07.webp" alt="Khung Files của SSH Studio: Máy của bạn bên trái, Remote server ở /du-lieu bên phải với lost+found và thu.txt 19 B" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · khung Files mở thẳng thư mục trên ổ vừa gắn</figcaption></figure>

## 4. Đối chiếu thao tác

| Việc | Lệnh |
|---|---|
| Ổ và phân vùng | `lsblk -o NAME,SIZE,TYPE,FSTYPE,MOUNTPOINTS` |
| Loại, nhãn, UUID | `sudo blkid` |
| Tạo hệ thống file | `sudo mkfs.ext4 -L NHÃN THIẾT_BỊ` |
| Gắn / gỡ | `sudo mount THIẾT_BỊ DIR`, `sudo umount DIR` |
| Đang gắn gì ở đâu | `findmnt`, `df -hT` |
| Gắn khi boot | dòng `UUID=… DIR ext4 defaults,nofail 0 2` trong `/etc/fstab` |
| Thử fstab | `sudo mount -a` (sau `systemctl daemon-reload`) |

## 5. Lỗi thường gặp

**`mkfs` nhầm thiết bị.** Mất sạch dữ liệu. Đọc `lsblk` hai lần; ổ mới là ổ không có `MOUNTPOINTS` và đúng dung lượng.

**fstab theo `/dev/sdb`.** Sau khi thêm một ổ, tên đổi, server gắn nhầm hoặc không lên.

**Thiếu `nofail` cho ổ dữ liệu.** Ổ hỏng là server không boot.

**Không `mount -a` trước khi khởi động lại.** Lỗi gõ trong `fstab` chỉ lộ ra lúc boot.

**`umount: target is busy`.** Có tiến trình đang dùng thư mục (kể cả shell đang `cd` ở đó). Tìm bằng `sudo lsof +D /du-lieu` hoặc `fuser -vm /du-lieu`.

## Bài tập

1. `sudo tune2fs -l $DIA | grep -E 'Block count|Reserved block count|Inode count'`. 5% dành cho root là bao nhiêu MB? Giảm xuống 1% bằng `tune2fs -m 1` cho ổ dữ liệu.
2. Ghi nhãn thay UUID: `LABEL=du-lieu /du-lieu ext4 defaults,nofail 0 2`. Ưu, nhược điểm?
3. `cd /du-lieu` rồi `sudo umount /du-lieu`. Lỗi gì? Cách giải quyết?
4. Chạy `npm run kiem-tra:p05` sau khi `docker restart toinoi-app-01`. `/du-lieu` có được gắn không? Server có khởi động không? Vì sao?

## Đọc tiếp

[LNX-048 · Bộ nhớ và tải: free, vmstat, load average](/articles/linux-bo-nho-tai-he-thong).
