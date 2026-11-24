---
slug: linux-bo-nho-tai-he-thong
title: "Bộ nhớ và tải hệ thống: đọc đúng free, available, swap, vmstat và nhận ra nghẽn CPU hay nghẽn đĩa"
summary: "Vì sao «RAM trống» gần hết mà server vẫn khoẻ (cache, available), /proc/meminfo, swap, bộ nhớ của một dịch vụ trong cgroup, đọc từng cột của vmstat (r, b, us, sy, id, wa, bi, bo), tạo tải CPU để thấy hàng đợi chạy và load average, tạo tải ghi đĩa để thấy bo và wa, và tìm tiến trình tốn bộ nhớ nhất."
category: devops-linux
level: advanced
tags: [linux, lnx-048, memory, free, vmstat, load-average, swap, cgroup, ssh-studio, toi-noi]
language: vi
---

Đội ứng dụng than: «Báo giá lúc trưa chậm hẳn.» Một người mở `free` trên server, thấy cột `free` nhỏ hơn hẳn `total`, và kết luận: «Hết RAM rồi, nâng cấp máy đi.» An biết đọc như vậy thường sai. Trước khi trả tiền cho máy lớn hơn, An muốn biết server thật sự đang nghẽn ở đâu: bộ nhớ, CPU, hay đĩa.

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-048 · Phần 5/6 · Nâng cao</span>
<p>Đọc trước: <a href="/articles/linux-tien-trinh-ps-top">LNX-031</a> (<code>top</code>, load average), <a href="/articles/linux-dia-phan-vung-mount">LNX-047</a> (<code>/du-lieu</code>). Trạng thái đầu bài: <code>npm run setup -- 48</code>. Lab chạy trong Docker: <code>free</code>, <code>vmstat</code>, <code>swapon</code> thấy RAM, CPU (16 nhân) và swap của <b>cả máy chạy Docker</b>, nên tải từ một container chỉ làm thay đổi vài phần trăm. Đĩa của máy chạy Docker cũng nhanh hơn nhiều so với đĩa mạng của máy ảo đám mây.</p>
</div>

## 1. free: «trống» và «dùng được» khác nhau

```text
an@toinoi-app-01:~$ free -h
               total        used        free      shared  buff/cache   available
Mem:            31Gi       1.6Gi        28Gi        26Mi       1.3Gi        29Gi
Swap:          8.0Gi          0B       8.0Gi
an@toinoi-app-01:~$ grep -E '^(MemTotal|MemFree|MemAvailable|Buffers|Cached|SwapTotal|SwapFree|Dirty):' /proc/meminfo
MemTotal:       32814956 kB
MemFree:        30089204 kB
MemAvailable:   31121424 kB
Buffers:           98000 kB
Cached:          1219796 kB
SwapTotal:       8388608 kB
SwapFree:        8388608 kB
Dirty:              7044 kB
an@toinoi-app-01:~$ swapon --show
NAME     TYPE      SIZE USED PRIO
/dev/sdc partition   8G   0B   -2
an@toinoi-app-01:~$ systemctl show bao-gia --no-pager -p MemoryCurrent -p MemoryMax
MemoryCurrent=9928704
MemoryMax=209715200
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-bo-nho-tai-he-thong/ssh-studio-06.webp" alt="Terminal của SSH Studio: free -h cho Mem total 31Gi, used 1.5Gi, free 28Gi, buff/cache 1.9Gi, available 29Gi, Swap 8.0Gi" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>free -h</code>: nhìn cột available, không nhìn free</figcaption></figure>

| Cột | Nghĩa |
|---|---|
| `used` | Bộ nhớ chương trình đang giữ |
| `free` | Hoàn toàn không ai dùng |
| `buff/cache` | Linux dùng RAM rảnh làm **bộ nhớ đệm** cho file đọc ghi gần đây; nhả ra ngay khi chương trình cần |
| **`available`** | Ước tính bộ nhớ mở được cho chương trình mới mà không cần swap: `free` cộng phần cache nhả được |

**Nhìn `available`, không nhìn `free`.** Một server chạy lâu luôn có `free` nhỏ, vì RAM rảnh được dùng làm cache: đó là dấu hiệu tốt (đọc file nhanh hơn), không phải thiếu RAM. Báo động khi `available` thấp **và** swap bắt đầu được dùng đều đặn.

- **`/proc/meminfo`** là nguồn số liệu của `free`: `MemAvailable`, `Cached`, `Dirty` (dữ liệu đã ghi vào cache, chưa xuống đĩa).
- **Swap** là vùng đĩa dùng thay RAM khi thiếu. Dùng một ít không sao; dùng liên tục (cột `si`/`so` của `vmstat` khác 0) là server đang chậm vì đọc ghi đĩa thay RAM.
- **Bộ nhớ của một dịch vụ**: systemd đếm theo cgroup ([LNX-035](/articles/linux-systemd-systemctl)). `bao-gia` đang dùng khoảng 10 MB, trần là 200 MB (`MemoryMax` của [LNX-040](/articles/linux-thuc-hanh-dich-vu-systemd)).

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-bo-nho-tai-he-thong/ssh-studio-01.webp" alt="Terminal của SSH Studio: grep /proc/meminfo cho MemTotal, MemFree, MemAvailable, Buffers, Cached, SwapTotal, SwapFree, Dirty; swapon --show có /dev/sdc 8G chưa dùng; systemctl show bao-gia cho MemoryCurrent khoảng 9.9 MB và MemoryMax 200 MB" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>/proc/meminfo</code>, swap và bộ nhớ của riêng bao-gia</figcaption></figure>

## 2. vmstat: một dòng mỗi giây

**`vmstat 1 4`** in 4 dòng, mỗi giây một dòng. **Dòng đầu là trung bình từ lúc khởi động**: bỏ qua nó, đọc các dòng sau.

| Nhóm | Cột | Nghĩa |
|---|---|---|
| procs | `r` | Tiến trình đang chạy hoặc **chờ CPU** |
| | `b` | Tiến trình **chờ đĩa** (trạng thái `D`) |
| memory | `free`, `buff`, `cache` | Như `free`, đơn vị KB |
| swap | `si`, `so` | Đọc từ swap / ghi ra swap mỗi giây |
| io | `bi`, `bo` | KB đọc vào / ghi ra đĩa mỗi giây |
| cpu | `us`, `sy` | % CPU cho chương trình / cho kernel |
| | `id` | % rảnh |
| | `wa` | % CPU rảnh nhưng **đang chờ đĩa** |

```text
an@toinoi-app-01:~$ vmstat 1 4
procs -----------memory---------- ---swap-- -----io---- -system-- -------cpu-------
 r  b   swpd   free   buff  cache   si   so    bi    bo   in   cs us sy id wa st gu
 1  0      0 30089204  98000 1306260    0    0   359   922 4084    3  1  1 98  0  0  0
 0  0      0 30089204  98000 1306264    0    0     0   472 4131 5435  0  1 99  0  0  0
 0  0      0 30091496  98000 1306264    0    0     0     0  676  788  0  0 100  0  0  0
 0  0      0 30091000  98024 1305544    0    0    12    28 8435 14519  1  1 97  0  0  0

an@toinoi-app-01:~$ nohup tinh-lai-bao-cao > /dev/null 2>&1 &
an@toinoi-app-01:~$ nohup tinh-lai-bao-cao > /dev/null 2>&1 &
an@toinoi-app-01:~$ vmstat 1 4
procs -----------memory---------- ---swap-- -----io---- -system-- -------cpu-------
 r  b   swpd   free   buff  cache   si   so    bi    bo   in   cs us sy id wa st gu
 3  0      0 30088788  98024 1305544    0    0   359   921 4084    3  1  1 98  0  0  0
 2  0      0 30088564  98048 1305572    0    0     0   132 1446  838 13  0 87  0  0  0
 2  0      0 30088564  98048 1305572    0    0     0     0 4642 5478 13  1 86  0  0  0
 2  0      0 30088564  98048 1305572    0    0     0     0 1425  871 13  0 87  0  0  0
an@toinoi-app-01:~$ uptime
 08:13:06 up  1:14,  0 user,  load average: 0.88, 0.54, 0.36
an@toinoi-app-01:~$ pkill -f tinh-lai-bao-cao
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-bo-nho-tai-he-thong/ssh-studio-04.webp" alt="Terminal của SSH Studio: vmstat 1 4 lúc rảnh, cột r và b bằng 0, id khoảng 98-99%, wa 0" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>vmstat</code> lúc server rảnh</figcaption></figure>

- **Lúc rảnh**: `r` 0, `id` gần 100.
- **Hai tiến trình ăn CPU** (`tinh-lai-bao-cao` từ [LNX-031](/articles/linux-tien-trinh-ps-top)): **`r` = 2** liên tục, `us` tăng lên khoảng 13%. Trên 16 nhân, hai nhân bận là 2/16 ≈ 12,5%: con số khớp. Trên một máy ảo 2 nhân, đó sẽ là 100%.
- **`uptime`**: load average tăng dần từ 0,3 về phía 2: số tiến trình trung bình đang chạy hoặc chờ. Đo trong vài giây nên con số 1 phút mới lên tới 0,88; để thêm vài phút nó sẽ tiến tới 2. So với số nhân ([LNX-031](/articles/linux-tien-trinh-ps-top)).

**Nghẽn CPU** trông như thế này: `r` lớn hơn số nhân, `id` gần 0, `us` (hoặc `sy`) cao.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-bo-nho-tai-he-thong/ssh-studio-02.webp" alt="Terminal của SSH Studio: chạy hai tinh-lai-bao-cao nền, vmstat 1 4 cho cột r lên 2 và us 13%, uptime cho load average 0.26, pkill dừng hai tiến trình" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · nghẽn CPU: cột <code>r</code> và <code>us</code> tăng</figcaption></figure>

## 3. Nghẽn đĩa

```text
an@toinoi-app-01:~$ for i in 1 2 3 4 5; do dd if=/dev/zero of=/du-lieu/ghi-thu-$i bs=1M count=50 oflag=direct status=none; sleep 1; done &
an@toinoi-app-01:~$ vmstat 1 4
procs -----------memory---------- ---swap-- -----io---- -system-- -------cpu-------
 r  b   swpd   free   buff  cache   si   so    bi    bo   in   cs us sy id wa st gu
 1  1      0 29710976 170704 1711412    0    0   289  1161 4164    3  1  1 98  0  0  0
 0  0      0 29708136 172856 1714372    0    0     0 125628 11112 17517  1  2 94  2  0  0
 0  0      0 29707876 172912 1714724    0    0     0 51360 7796 11678  1  2 97  0  0  0
 0  1      0 29703352 174608 1715400    0    0     0 57592 9526 15464  1  2 95  2  0  0
an@toinoi-app-01:~$ wait
an@toinoi-app-01:~$ rm /du-lieu/ghi-thu-*

an@toinoi-app-01:~$ ps -eo pid,user,rss,comm --sort=-rss | head -n 5
    PID USER       RSS COMMAND
   4281 baogia   20240 bao-gia
   3985 root     18020 systemd-journal
      1 root     12760 systemd
   1833 root      8100 sshd
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-bo-nho-tai-he-thong/ssh-studio-05.webp" alt="Terminal của SSH Studio: vòng lặp dd ghi 50 MB với oflag=direct vào /du-lieu chạy nền, vmstat 1 4 cho cột bo khoảng 51200 và b lên 1, wait rồi xoá file thử" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · nghẽn đĩa: cột <code>bo</code> tăng khi ghi thẳng xuống ổ</figcaption></figure>

- `dd … oflag=direct` ghi thẳng xuống ổ dữ liệu, bỏ qua cache, để thấy đĩa thật làm việc. Đĩa của máy chạy Docker (SSD cục bộ) ghi 200 MB chỉ mất khoảng 0,1 giây, nên vòng lặp ghi 50 MB mỗi giây trong 5 giây để `vmstat` kịp thấy.
- Từ dòng thứ hai của `vmstat`: **`bo` ≈ 50 000** KB/giây (dòng có 125 628 là lúc hai lần ghi rơi vào cùng một giây), **`b` = 1** ở vài dòng (`dd` đang chờ đĩa), `wa` lên 2%.
- Trên một máy ảo với đĩa mạng (vài nghìn IOPS), cùng thao tác kéo dài nhiều giây, cột **`b`** (chờ đĩa) lớn hơn 0 và **`wa`** lên vài chục phần trăm. Đó là dấu hiệu của **nghẽn đĩa**: CPU còn rảnh, nhưng tiến trình phải đợi đĩa. Nâng CPU không giúp gì.

**Ai tốn bộ nhớ nhất**: `ps --sort=-rss` (RSS: bộ nhớ thật, [LNX-031](/articles/linux-tien-trinh-ps-top)). Trên server lab nhỏ, `bao-gia` và `journald` đứng đầu, mỗi cái khoảng 20 MB.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-bo-nho-tai-he-thong/ssh-studio-03.webp" alt="Terminal của SSH Studio: ps -eo pid,user,rss,comm --sort=-rss | head -n 5 cho bao-gia của baogia dùng nhiều bộ nhớ nhất, sau đó systemd-journald, systemd, sshd" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · ai đang dùng nhiều bộ nhớ nhất</figcaption></figure>

## 4. Chẩn đoán nhanh

| Triệu chứng trong `vmstat` | Nghẽn ở | Hướng xử lý |
|---|---|---|
| `r` > số nhân, `id` ≈ 0 | CPU | Tìm tiến trình bằng `top`; tối ưu hoặc thêm CPU |
| `b` > 0, `wa` cao, `bi`/`bo` lớn | Đĩa | Tìm ai đọc ghi (`iotop`, `pidstat -d`); đĩa nhanh hơn, giảm ghi |
| `si`/`so` > 0 liên tục, `available` thấp | Bộ nhớ | Tìm tiến trình tốn RAM; giới hạn (`MemoryMax`), thêm RAM |
| Mọi thứ rảnh mà vẫn chậm | Không phải server này | Mạng, DNS ([LNX-042](/articles/linux-dns-ping-curl)), dịch vụ phụ thuộc |

## 5. Lỗi thường gặp

**Kết luận «hết RAM» từ cột `free`.** Nhìn `available`.

**Đọc dòng đầu của `vmstat`.** Đó là trung bình từ lúc boot.

**So load average mà không biết số nhân.** Load 4 là quá tải với 2 nhân, rảnh với 16 nhân.

**Tin số liệu trong container.** `free`, `vmstat` thường là của máy chủ; giới hạn thật nằm trong cgroup (`systemctl show -p MemoryCurrent`, `/sys/fs/cgroup`).

**Tắt swap để «tăng tốc».** Không có swap, hết RAM là kernel giết tiến trình (OOM killer) ngay.

## Bài tập

1. `vmstat -s | head -n 10` và `vmstat -d`. Mỗi lệnh cho thêm thông tin gì?
2. Chạy bốn `tinh-lai-bao-cao` cùng lúc, xem `vmstat 1 5` và `uptime` sau 2 phút. `r` và load 1 phút là bao nhiêu?
3. `cat /proc/pressure/cpu /proc/pressure/memory /proc/pressure/io` (PSI): các con số `some` và `full` có nghĩa gì?
4. `journalctl -k | grep -i 'out of memory'`: đã từng có tiến trình bị OOM killer giết chưa? Nếu có, nó là gì?

## Đọc tiếp

[LNX-049 · TLS và reverse proxy nginx cho bao-gia](/articles/linux-nginx-reverse-proxy-tls).
