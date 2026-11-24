---
slug: linux-su-co-khong-ssh-duoc
title: "Sự cố: không SSH được vào server, đọc thông báo lỗi và khoanh vùng mạng, sshd, xác thực hay host key"
summary: "Năm tình huống thật khi SSH Studio không vào được server: sai cổng (máy từ chối kết nối), sshd không chạy, tường lửa chặn bastion, server từ chối key vì quyền authorized_keys, và host key đổi. Mỗi tình huống: thông báo của SSH Studio nói gì, khoanh vùng bằng nc từ bastion, đọc log của sshd qua console, sửa và kết nối lại."
category: devops-linux
level: advanced
tags: [linux, lnx-059, ssh, troubleshooting, sshd, ufw, host-key, authorized-keys, ssh-studio, toi-noi]
language: vi
---

«Không SSH được» là một triệu chứng, không phải một lỗi. Phía sau có thể là mạng, tường lửa, sshd đã chết, key bị từ chối, hay một host key thay đổi. Mỗi loại cần một cách sửa khác, và sửa nhầm loại (đổi key khi lỗi là tường lửa) chỉ tốn thêm thời gian. Tin tốt: thông báo lỗi đầu tiên gần như luôn nói rõ nó thuộc loại nào, nếu bạn đọc kỹ.

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-059 · Phần 6/6 · Nâng cao</span>
<p>Đọc trước: <a href="/articles/linux-bastion-jump-host">LNX-052</a> (bastion), <a href="/articles/linux-host-key-known-hosts">LNX-023</a> (host key), <a href="/articles/linux-ssh-key-ssh-studio">LNX-022</a> (authorized_keys). Trạng thái đầu bài: <code>npm run setup -- 59</code>. Lab dựng từng sự cố trên <code>toinoi-app-02</code>. Phần «console của nhà cung cấp» (màn hình điều khiển trên web của máy ảo, vào được kể cả khi SSH chết) được mô phỏng bằng một phiên trên chính app02; trong bản ghi của lab, đó là các dòng có dấu nhắc <code>an@toinoi-app-02</code>.</p>
</div>

## 1. Bản đồ: lỗi nằm ở tầng nào

Một kết nối SSH đi qua các tầng theo thứ tự. Lỗi xuất hiện ở tầng nào thì các tầng trước nó đã ổn:

| Tầng | Lỗi thường thấy | Nghĩa |
|---|---|---|
| Phân giải tên | `No such host is known`, `Could not resolve hostname` | Sai tên, DNS ([LNX-042](/articles/linux-dns-ping-curl)) |
| TCP | **từ chối kết nối** (refused) | Tới được máy, nhưng không có gì nghe ở cổng đó |
| TCP | **hết thời gian** (timed out) | Không có trả lời: tường lửa thả gói, máy tắt, sai IP |
| Host key | **Host key đã thay đổi** | Đầu kia không phải máy bạn đã tin (hoặc máy được dựng lại) |
| Xác thực | **Xác thực thất bại** | Tới đúng sshd, nhưng key/mật khẩu/OTP không được nhận |

## 2. Tình huống 1: sai cổng

An sửa nhầm cổng của `toinoi-bastion` thành 2201:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-su-co-khong-ssh-duoc/ssh-studio-01.webp" alt="SSH Studio báo lỗi khi kết nối toinoi-bastion ở cổng 2201: thông báo của Windows rằng máy đích chủ động từ chối kết nối (os error 10061)" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · cổng 2201: <b>từ chối kết nối</b></figcaption></figure>

Lỗi TCP được SSH Studio hiện nguyên văn từ hệ điều hành (trên Windows là thông báo tiếng Anh kèm `os error 10061`). «Từ chối» nghĩa là gói tin tới được máy (ở đây là chính máy bạn, cổng do Docker mở) và nhận về một gói RST: không có gì nghe ở cổng 2201. Sửa: đúng cổng.

## 3. Tình huống 2: sshd trên app02 không chạy

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-su-co-khong-ssh-duoc/ssh-studio-02.webp" alt="SSH Studio báo lỗi khi kết nối toinoi-app-02 qua jump host: Jump host toinoi-bastion không mở được kết nối tới toinoi-app-02:22: Failed to open channel (ConnectFailed), nút Kết nối lại" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · jump host không mở được kênh tới app02</figcaption></figure>

Với jump host, lỗi TCP ở bước cuối được bastion báo lại: **bastion** đã thử mở kết nối tới `toinoi-app-02:22` và thất bại. Lỗi không nói vì sao, nên An khoanh vùng từ chính bastion:

```text
an@toinoi-bastion:~$ nc -zv -w 3 toinoi-app-02 22
nc: connect to toinoi-app-02 (10.20.0.12) port 22 (tcp) failed: Connection refused
an@toinoi-bastion:~$ nc -zv -w 3 toinoi-app-02 443
nc: connect to toinoi-app-02 (10.20.0.12) port 443 (tcp) timed out: Operation now in progress
an@toinoi-bastion:~$ ping -c 1 -W 2 toinoi-app-02 | tail -n 2
1 packets transmitted, 1 received, 0% packet loss, time 0ms
rtt min/avg/max/mdev = 0.050/0.050/0.050/0.000 ms
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-su-co-khong-ssh-duoc/ssh-studio-03.webp" alt="Terminal của SSH Studio trên toinoi-bastion: nc tới toinoi-app-02 cổng 22 Connection refused, cổng 443 timed out, ping 1 received" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · từ bastion: máy sống, cổng 22 bị từ chối</figcaption></figure>

- `ping` có trả lời: máy sống, mạng thông.
- Cổng 22 **refused**: máy trả lời, nhưng không có gì nghe. Nghi sshd.
- Cổng 443 **timed out**: tường lửa của app02 thả gói ([LNX-052](/articles/linux-bastion-jump-host) chỉ mở 22). Hai kiểu lỗi khác nhau cho hai nguyên nhân khác nhau.

Console của nhà cung cấp:

```text
an@toinoi-app-02:~$ systemctl is-active ssh
inactive
an@toinoi-app-02:~$ sudo journalctl -u ssh -n 3 --no-pager -o cat
ssh.service: Unit process 1344 (sshd) remains running after unit stopped.
Stopped ssh.service - OpenBSD Secure Shell server.
ssh.service: Consumed 2.735s CPU time, 38.8M memory peak, 0B memory swap peak.
an@toinoi-app-02:~$ sudo systemctl start ssh
```

Chi tiết đáng chú ý: `Unit process … (sshd) remains running after unit stopped`. Dừng `ssh.service` không cắt các phiên đang mở (systemd của Ubuntu để sshd của từng phiên sống tiếp), nên người đang ở trong phiên không thấy gì bất thường, còn người mới thì không vào được. Thấy sự cố kiểu này, đừng đóng phiên đang mở: nó có thể là đường duy nhất còn lại.

## 4. Tình huống 3: tường lửa chặn bastion

Một quy tắc ufw trên app02 bị xoá nhầm:

```text
an@toinoi-bastion:~$ nc -zv -w 3 toinoi-app-02 22
nc: connect to toinoi-app-02 (10.20.0.12) port 22 (tcp) timed out: Operation now in progress
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-su-co-khong-ssh-duoc/ssh-studio-04.webp" alt="SSH Studio báo lỗi khi kết nối toinoi-app-02: Jump host toinoi-bastion không mở được kết nối tới toinoi-app-02:22: Failed to open channel (ConnectFailed), cùng thông báo như khi sshd dừng" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · tường lửa thả gói: cùng thông báo, đến chậm hơn</figcaption></figure>

Lần này `nc` từ bastion **timed out**. SSH Studio vẫn hiện cùng thông báo `ConnectFailed` như tình huống 2, nhưng chỉ sau khi bastion chờ hết thời gian kết nối, nên thông báo đến chậm hơn hẳn. Thông báo của jump host không phân biệt hai trường hợp; `nc` từ bastion thì có: **refused** là không có gì nghe, **timed out** gần như luôn là tường lửa (hay sai IP). Console:

```text
an@toinoi-app-02:~$ sudo ufw status numbered
Status: active

     To                         Action      From
     --                         ------      ----
[ 1] 22/tcp                     ALLOW IN    10.20.0.11                 # rsync tu app01

an@toinoi-app-02:~$ sudo ufw insert 1 allow from 10.20.0.10 to any port 22 proto tcp comment 'SSH qua bastion'
Rule inserted
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-su-co-khong-ssh-duoc/ssh-studio-11.webp" alt="Terminal của SSH Studio trên toinoi-bastion sau khi thêm lại quy tắc ufw trên app02: nc -zv -w 3 toinoi-app-02 22 báo Connection to toinoi-app-02 (10.20.0.12) 22 port succeeded" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · kiểm lại từ bastion: cổng 22 của app02 đã mở trở lại</figcaption></figure>

Quy tắc cho bastion mất, chỉ còn quy tắc cho app01. Thêm lại vào đầu danh sách (`insert 1`), thử lại bằng `nc` từ bastion trước khi thử SSH Studio.

## 5. Tình huống 4: server từ chối key

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-su-co-khong-ssh-duoc/ssh-studio-05.webp" alt="SSH Studio báo lỗi khi kết nối toinoi-app-02: Xác thực thất bại, đã thử SSH key, server chấp nhận publickey, password, gợi ý kiểm tra public key trong authorized_keys của an" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <b>xác thực thất bại</b>: tới được sshd, key bị từ chối</figcaption></figure>

Đây là tầng cuối: mạng, tường lửa, sshd, host key đều ổn. SSH Studio nói rõ đã thử gì (`SSH key`), server nhận những phương thức nào (`publickey, password`: app02 vẫn cho đăng nhập bằng mật khẩu, việc nên tắt như [LNX-025](/articles/linux-sshd-config-an-toan)), và gợi ý kiểm tra `authorized_keys`. app01 cũng bị từ chối với key đồng bộ:

```text
an@toinoi-app-01:~$ ssh -o BatchMode=yes app02 true
an@toinoi-app-02: Permission denied (publickey,password).
```

Lý do thật nằm ở log của sshd, không bao giờ gửi cho client (để không lộ thông tin cho kẻ dò):

```text
an@toinoi-app-02:~$ sudo journalctl -u ssh -n 20 --no-pager -o cat | grep -E 'Authentication refused|bad ownership' | tail -n 1
Authentication refused: bad ownership or modes for file /home/an/.ssh/authorized_keys
an@toinoi-app-02:~$ ls -l ~/.ssh/authorized_keys
-rw-rw-rw- 1 an an 194 Nov 24 09:39 /home/an/.ssh/authorized_keys
an@toinoi-app-02:~$ chmod 600 ~/.ssh/authorized_keys
```

`authorized_keys` có quyền `666`: **mọi** tài khoản trên máy đều sửa được nó, tức ai cũng có thể thêm key của mình để đăng nhập thành `an`. Với tuỳ chọn StrictModes (mặc định bật), sshd coi file như vậy là không tin được và **bỏ qua cả file**, dù key bên trong đúng. Sửa quyền về `600` ([LNX-022](/articles/linux-ssh-key-ssh-studio)). Cùng lý do: thư mục home hay `~/.ssh` ghi được bởi người khác. (OpenSSH vẫn chấp nhận file ghi được bởi nhóm khi nhóm đó chỉ có chính chủ, như nhóm riêng `an` của Ubuntu: `664` không gây lỗi này.)

## 6. Tình huống 5: host key đổi

Bình dựng lại app02 từ ảnh mới, host key đổi theo:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-su-co-khong-ssh-duoc/ssh-studio-06.webp" alt="SSH Studio chặn kết nối toinoi-app-02: Host key đã thay đổi. Kết nối bị chặn. Hãy xác minh fingerprint với quản trị viên trước khi quên host trong Known Hosts" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <b>Host key đã thay đổi</b>: kết nối bị chặn</figcaption></figure>

SSH Studio chặn, đúng như [LNX-023](/articles/linux-host-key-known-hosts). Không quên host ngay: hỏi Bình fingerprint thật trên console:

```text
an@toinoi-app-02:~$ ssh-keygen -lf /etc/ssh/ssh_host_ed25519_key.pub
256 SHA256:RtHbm5duo8D/RDMzT9K2WtYCVwf4ZTjRpWNu9aFVFhw root@toinoi-app-02 (ED25519)
```

Trùng với fingerprint SSH Studio đưa ra sau khi quên host cũ trong **Known Hosts** và kết nối lại:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-su-co-khong-ssh-duoc/ssh-studio-07.webp" alt="Hộp thoại Xác nhận danh tính server của SSH Studio cho toinoi-app-02 với fingerprint mới SHA256:RtHbm5duo8D/RDMzT9K2WtYCVwf4ZTjRpWNu9aFVFhw" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · fingerprint mới, đã đối chiếu với console</figcaption></figure>

Chỉ khi trùng mới **Tin cậy và kết nối**. Còn một chỗ nữa nhớ host key cũ: `known_hosts` của An trên app01 (rsync, Ansible):

```text
an@toinoi-app-01:~$ ssh -o BatchMode=yes app02 hostname 2>&1 | grep -E 'IDENTIFICATION|Host key verification'
@    WARNING: REMOTE HOST IDENTIFICATION HAS CHANGED!     @
Host key verification failed.
an@toinoi-app-01:~$ ssh-keygen -R toinoi-app-02
/home/an/.ssh/known_hosts updated.
Original contents retained as /home/an/.ssh/known_hosts.old
an@toinoi-app-01:~$ ssh app02 hostname
Warning: Permanently added 'toinoi-app-02' (ED25519) to the list of known hosts.
toinoi-app-02
an@toinoi-app-01:~$ ssh-keygen -lF toinoi-app-02 | tail -n 1
toinoi-app-02 ED25519 SHA256:RtHbm5duo8D/RDMzT9K2WtYCVwf4ZTjRpWNu9aFVFhw
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-su-co-khong-ssh-duoc/ssh-studio-08.webp" alt="Terminal của SSH Studio trên toinoi-app-01: ssh app02 báo REMOTE HOST IDENTIFICATION HAS CHANGED, ssh-keygen -R toinoi-app-02, ssh app02 hostname ghi nhận host key mới, ssh-keygen -lF in fingerprint RtHbm5 trùng với console" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · cập nhật <code>known_hosts</code> của app01 sau khi đã đối chiếu</figcaption></figure>

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-su-co-khong-ssh-duoc/ssh-studio-09.webp" alt="Hộp thoại Nhật ký hoạt động của SSH Studio: các dòng lỗi đỏ theo thứ tự thời gian: toinoi-bastion target machine actively refused, Jump host không mở được kết nối tới toinoi-app-02:22, xác thực thất bại, Host key đã thay đổi, rồi Đã kết nối an@toinoi-app-02" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · nhật ký hoạt động: năm tình huống, năm thông báo lỗi khác nhau</figcaption></figure>

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-su-co-khong-ssh-duoc/ssh-studio-10.webp" alt="Launchpad của SSH Studio, tab Phiên đang mở có đủ ba phiên toinoi-app-01, toinoi-bastion và toinoi-app-02 đều Đã kết nối" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · sau năm sự cố, cả ba server đều kết nối lại được</figcaption></figure>

## 7. Trình tự khi không SSH được

1. **Đọc nguyên văn thông báo đầu tiên**, xếp nó vào một tầng ở bảng mục 1.
2. **Khoanh vùng từ một máy khác trong cùng mạng** (bastion): `nc -zv -w 3 MÁY 22`, `ping`.
3. **Console của nhà cung cấp** cho phần còn lại: `systemctl status ssh`, `journalctl -u ssh`, `ufw status`, quyền `~/.ssh`.
4. **Sửa một thứ, thử lại**, ghi lại đã sửa gì.
5. Giữ phiên đang mở trong suốt quá trình (nếu còn), và không đổi `sshd_config` mà không `sshd -t`.

## 8. Lỗi thường gặp

**Quên host key ngay khi thấy cảnh báo.** Đó đúng là cảnh báo duy nhất trước một tấn công xen giữa.

**Đoán thay vì đọc log.** Lý do từ chối key luôn nằm trong `journalctl -u ssh`.

**Đóng phiên cuối cùng khi đang sửa sshd hay tường lửa.** Mất đường vào duy nhất.

**`chmod 777 ~/.ssh` cho «chắc ăn».** Ngược lại: sshd từ chối ngay.

## Bài tập

1. Tạo tình huống «sai tên máy»: sửa địa chỉ của app02 thành `toinoi-app-03`. SSH Studio báo gì? Lỗi đến từ máy nào?
2. Trên app02, `sudo chmod 777 /home/an` rồi kết nối. Log của sshd ghi gì? So với `chmod 775`? Sửa lại về `750`.
3. Trên bastion, tắt `AllowTcpForwarding` (file trong `sshd_config.d`, `sshd -t`, reload). Kết nối app02 báo gì? Bật lại.
4. Viết mục «Không SSH được» cho runbook ([LNX-060](/articles/linux-thuc-hanh-runbook-van-hanh)) từ mục 7, kèm lệnh cụ thể của Tới Nơi.

## Đọc tiếp

[LNX-060 · Thực hành: runbook vận hành toinoi-app-01](/articles/linux-thuc-hanh-runbook-van-hanh).
