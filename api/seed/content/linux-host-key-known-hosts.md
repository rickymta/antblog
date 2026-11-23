---
slug: linux-host-key-known-hosts
title: "Host key và Known Hosts: vì sao SSH chặn kết nối khi server đổi khoá, và xử lý thế nào cho đúng"
summary: "Mỗi server SSH có host key riêng; client nhớ nó trong known_hosts (OpenSSH) hoặc trang Known Hosts (SSH Studio). Bài mô phỏng server được dựng lại với host key mới, đọc cảnh báo REMOTE HOST IDENTIFICATION HAS CHANGED, xem SSH Studio chặn kết nối, xác minh rồi quên host đúng cách, và rút ra bài học khi dựng lại server: đưa host key cũ về."
category: devops-linux
level: beginner
tags: [linux, lnx-023, ssh, host-key, known-hosts, fingerprint, mitm, ssh-studio, toi-noi]
language: vi
---

Thứ Hai, Bình nhắn vào nhóm: «Cuối tuần mình cài lại `toinoi-app-01` từ đầu cho sạch, cấu hình giữ nguyên.» Mười phút sau, An mở SSH Studio thì không kết nối được, kèm một thông báo màu đỏ: host key đã thay đổi, kết nối bị chặn. Cùng lúc đó, Cường dùng OpenSSH trong terminal và nhận về một khung chữ toàn ký tự `@`, có câu «IT IS POSSIBLE THAT SOMEONE IS DOING SOMETHING NASTY!».

Hai cảnh báo nói cùng một điều, và cả hai đều đúng khi lên tiếng: client không có cách nào phân biệt «server được cài lại» với «có kẻ đang giả làm server». Bài này giải thích host key là gì, mô phỏng đúng tình huống đó trên lab, và chỉ ra cách xử lý vừa an toàn vừa không làm phiền cả nhóm mỗi lần dựng lại server.

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-023 · Phần 3/6 · Cơ bản</span>
<p>Đọc trước: phần fingerprint của <a href="/articles/linux-dung-lab-docker-ket-noi-ssh-studio">LNX-002</a> và <a href="/articles/linux-ssh-key-ssh-studio">LNX-022</a>. Trạng thái đầu bài: <code>npm run setup -- 23</code>. Bài thay host key của server lab bằng một key khác rồi đưa key cũ về; sau bài, SSH Studio của bạn lại tin cậy đúng host key ban đầu.</p>
</div>

## 1. Hai loại key, hai chiều tin cậy

| | Chứng minh điều gì | Nằm ở đâu | Client lưu ở đâu |
|---|---|---|---|
| **Host key** | Server là đúng server | `/etc/ssh/ssh_host_*_key` trên server | `~/.ssh/known_hosts`, trang **Known Hosts** của SSH Studio |
| **Key người dùng** ([LNX-022](/articles/linux-ssh-key-ssh-studio)) | Bạn là đúng bạn | Máy của bạn | `~/.ssh/authorized_keys` trên server |

Xác thực SSH đi theo hai chiều: trước khi bạn gửi bất cứ thứ gì (mật khẩu, chữ ký bằng key), server phải chứng minh nó giữ khoá riêng tư của host key mà client đã biết. Nếu không kiểm tra chiều này, mọi thứ bạn gửi có thể tới tay kẻ đứng giữa (*man-in-the-middle*).

```text
an@toinoi-app-01:~$ ls -l /etc/ssh/ssh_host_*
-rw------- 1 root root  513 Nov 22 22:26 /etc/ssh/ssh_host_ecdsa_key
-rw-r--r-- 1 root root  182 Nov 22 22:26 /etc/ssh/ssh_host_ecdsa_key.pub
-rw------- 1 root root  411 Nov 22 22:26 /etc/ssh/ssh_host_ed25519_key
-rw-r--r-- 1 root root  102 Nov 22 22:26 /etc/ssh/ssh_host_ed25519_key.pub
-rw------- 1 root root 2610 Nov 22 22:26 /etc/ssh/ssh_host_rsa_key
-rw-r--r-- 1 root root  574 Nov 22 22:26 /etc/ssh/ssh_host_rsa_key.pub
an@toinoi-app-01:~$ ssh-keygen -lf /etc/ssh/ssh_host_ed25519_key.pub
256 SHA256:/fjhnPhFNYvAMC3S/UdkbCQ039oKGZN4rJ96pPyUvkI root@buildkitsandbox (ED25519)
an@toinoi-app-01:~$ ssh-keyscan -t ed25519 localhost 2>/dev/null | ssh-keygen -lf -
256 SHA256:/fjhnPhFNYvAMC3S/UdkbCQ039oKGZN4rJ96pPyUvkI localhost (ED25519)
```

- Server có ba cặp host key: RSA, ECDSA, Ed25519. Client và server thống nhất dùng một loại; với client hiện đại, đó là Ed25519. Khoá riêng tư chỉ `root` đọc được (`-rw-------`).
- `ssh-keygen -lf` in fingerprint từ file trên server. `ssh-keyscan` hỏi server qua mạng xem nó đưa ra key nào, rồi in fingerprint của key đó. Hai fingerprint trùng nhau.
- Đây chính là chuỗi `SHA256:/fjhnPhFN…` bạn đã đối chiếu khi kết nối lần đầu ở LNX-002, và là chuỗi nằm trong trang Known Hosts.

### known_hosts của OpenSSH

```text
an@toinoi-app-01:~$ cat ~/.ssh/known_hosts
|1|Chv/veqwRyhffmU2n7M9cJNC4hU=|9RR/LyyZ2RVL/ooGWDPGouWJNaM= ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIFerLccDCe220G1XWu5QIk3RBNneU4iXYx05S0FLtxn2
an@toinoi-app-01:~$ ssh-keygen -F localhost
|1|Chv/veqwRyhffmU2n7M9cJNC4hU=|9RR/LyyZ2RVL/ooGWDPGouWJNaM= ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIFerLccDCe220G1XWu5QIk3RBNneU4iXYx05S0FLtxn2
```

Mỗi dòng của `known_hosts` là một cặp «tên máy, host key». Trên Ubuntu, phần tên máy được **băm** (`|1|…|…`, do `HashKnownHosts yes` trong `/etc/ssh/ssh_config`): ai đọc được file cũng không biết bạn hay vào những server nào. Vì vậy không `grep` tên máy trong file được nữa; `ssh-keygen -F TÊN` tìm giúp bạn.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-host-key-known-hosts/ssh-studio-01.webp" alt="Terminal của SSH Studio: ls -l các file host key trong /etc/ssh, fingerprint Ed25519 SHA256:/fjhnPhFN… đọc từ file và từ ssh-keyscan trùng nhau, nội dung known_hosts với tên máy đã băm, ssh-keygen -F localhost tìm ra dòng 1" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · host key của server và <code>known_hosts</code> phía client</figcaption></figure>

## 2. Mô phỏng: server được dựng lại

Lab mô phỏng việc cài lại bằng cách thay host key Ed25519 của server bằng một key khác (key mới được tạo sẵn để kết quả trùng với bài). Trước đó, An giữ một bản host key cũ trong `/root`:

```text
an@toinoi-app-01:~$ sudo cp -a /etc/ssh/ssh_host_ed25519_key /etc/ssh/ssh_host_ed25519_key.pub /root/
an@toinoi-app-01:~$ sudo systemctl restart ssh
an@toinoi-app-01:~$ ssh-keygen -lf /etc/ssh/ssh_host_ed25519_key.pub
256 SHA256:5b2/eh6nvp+9USu4fWrlujTEiIJSNBhLX51PTh+yjLY root@toinoi-app-01 (ED25519)
an@toinoi-app-01:~$ ssh -o BatchMode=yes an@localhost true
@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@
@    WARNING: REMOTE HOST IDENTIFICATION HAS CHANGED!     @
@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@
IT IS POSSIBLE THAT SOMEONE IS DOING SOMETHING NASTY!
Someone could be eavesdropping on you right now (man-in-the-middle attack)!
It is also possible that a host key has just been changed.
The fingerprint for the ED25519 key sent by the remote host is
SHA256:5b2/eh6nvp+9USu4fWrlujTEiIJSNBhLX51PTh+yjLY.
Please contact your system administrator.
Add correct host key in /home/an/.ssh/known_hosts to get rid of this message.
Offending ED25519 key in /home/an/.ssh/known_hosts:1
  remove with:
  ssh-keygen -f '/home/an/.ssh/known_hosts' -R 'localhost'
Host key for localhost has changed and you have requested strict checking.
Host key verification failed.
```

Server giờ đưa ra fingerprint `SHA256:5b2/eh6n…`, khác với dòng đã lưu trong `known_hosts`. OpenSSH từ chối kết nối và nói rõ:

- **Fingerprint mới** mà server vừa đưa ra.
- **Dòng nào** trong file nào đang giữ key cũ (`known_hosts:1`).
- **Lệnh để xoá** dòng đó, nếu bạn đã xác minh thay đổi là hợp lệ.

Câu chốt là «Host key verification failed.»: không có gì được gửi đi. Kể cả khi bạn dùng mật khẩu, mật khẩu chưa bao giờ rời máy.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-host-key-known-hosts/ssh-studio-02.webp" alt="Terminal của SSH Studio: sau khi thay host key và khởi động lại ssh, fingerprint mới SHA256:5b2/eh6n…, và cảnh báo WARNING: REMOTE HOST IDENTIFICATION HAS CHANGED! của OpenSSH với fingerprint mới, dòng Offending ED25519 key in known_hosts:1 và Host key verification failed" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · cảnh báo host key thay đổi của OpenSSH</figcaption></figure>

## 3. SSH Studio chặn kết nối

Phiên SSH Studio đang mở không bị ảnh hưởng: host key chỉ được kiểm lúc bắt tay. Mở một phiên mới tới cùng server:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-host-key-known-hosts/ssh-studio-03.webp" alt="Tab phiên mới tới toinoi-app-01 trong SSH Studio không kết nối được, thông báo host key đã thay đổi, kết nối bị chặn, hãy xác minh fingerprint với quản trị viên trước khi quên host trong Known Hosts" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · kết nối mới bị chặn vì host key đổi</figcaption></figure>

SSH Studio không cho «bấm bỏ qua» ngay tại đó. Lối duy nhất là vào trang **Known Hosts** và chủ động quên host, sau khi đã xác minh:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-host-key-known-hosts/ssh-studio-04.webp" alt="Trang Known Hosts của SSH Studio với mục [127.0.0.1]:2222, fingerprint cũ SHA256:/fjhnPhFN… và nút Quên host" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · trang <b>Known Hosts</b> vẫn giữ fingerprint cũ</figcaption></figure>

### Xác minh trước khi quên

**Bước này quyết định mọi thứ.** Quên host rồi tin cậy lại mà không xác minh chính là để kẻ đứng giữa thắng. Hỏi người quản trị server (ở đây là Bình) qua một kênh khác SSH, ví dụ chat của công ty hay điện thoại: «Fingerprint Ed25519 mới của `toinoi-app-01` là gì?». Bình đọc được nó từ console của nhà cung cấp hoặc chạy `ssh-keygen -lf /etc/ssh/ssh_host_ed25519_key.pub` trên server. Trùng thì tiếp tục; không trùng thì dừng lại và báo động.

Đã xác minh: bấm **Quên host**, xác nhận, rồi kết nối lại. SSH Studio hỏi như lần đầu:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-host-key-known-hosts/ssh-studio-05.webp" alt="Hộp thoại Xác nhận danh tính server của SSH Studio cho [127.0.0.1]:2222 với fingerprint mới SHA256:5b2/eh6nvp+9USu4fWrlujTEiIJSNBhLX51PTh+yjLY và các nút Huỷ kết nối, Tin cậy và kết nối" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · sau khi quên host: xác nhận fingerprint mới như lần kết nối đầu</figcaption></figure>

## 4. Xử lý ở phía OpenSSH

```text
an@toinoi-app-01:~$ ssh-keygen -R localhost
/home/an/.ssh/known_hosts updated.
Original contents retained as /home/an/.ssh/known_hosts.old
an@toinoi-app-01:~$ ssh -o BatchMode=yes -o StrictHostKeyChecking=accept-new an@localhost true
Warning: Permanently added 'localhost' (ED25519) to the list of known hosts.
an@localhost: Permission denied (publickey,password).
```

**`ssh-keygen -R TÊN`** xoá mọi dòng của tên đó khỏi `known_hosts` (kể cả dòng đã băm) và giữ bản cũ ở `known_hosts.old`. Lần kết nối sau, `ssh` hỏi như với một server mới. Đừng xoá cả file `known_hosts` cho nhanh: bạn sẽ mất dấu mọi server khác.

Dòng cần xem là `Permanently added 'localhost' (ED25519)`: host key mới đã được ghi nhận. `Permission denied` sau đó là bình thường, vì trên server không có key riêng tư nào của An để đăng nhập; bước host key đã qua.

`StrictHostKeyChecking` của OpenSSH có ba mức đáng biết:

| Giá trị | Server mới | Server có key đổi |
|---|---|---|
| `ask` (mặc định) | Hỏi bạn | Từ chối |
| `accept-new` | Tự tin cậy | Từ chối |
| `no` | Tự tin cậy | **Vẫn kết nối** (chỉ tắt một số tính năng) |

`accept-new` hợp cho script lần đầu chạm tới một server. **`no` không bao giờ nên dùng**: đó là tắt hẳn lớp bảo vệ của bài này.

## 5. Bài học: dựng lại server thì giữ host key

Cảnh báo ở trên tốn công của cả nhóm, và tệ hơn, nó dạy mọi người thói quen «cứ quên host rồi bấm tin cậy». Lần sau, khi có kẻ đứng giữa thật, thói quen đó làm việc thay cho hắn. Cách làm đúng khi cài lại một server có cùng tên: **sao lưu host key trước, khôi phục sau khi cài**.

```text
an@toinoi-app-01:~$ sudo cp -a /root/ssh_host_ed25519_key /root/ssh_host_ed25519_key.pub /etc/ssh/
an@toinoi-app-01:~$ sudo systemctl restart ssh
an@toinoi-app-01:~$ ssh-keygen -lf /etc/ssh/ssh_host_ed25519_key.pub
256 SHA256:/fjhnPhFNYvAMC3S/UdkbCQ039oKGZN4rJ96pPyUvkI root@buildkitsandbox (ED25519)
an@toinoi-app-01:~$ ssh-keygen -R localhost
/home/an/.ssh/known_hosts updated.
Original contents retained as /home/an/.ssh/known_hosts.old
an@toinoi-app-01:~$ ssh -o BatchMode=yes -o StrictHostKeyChecking=accept-new an@localhost true
Warning: Permanently added 'localhost' (ED25519) to the list of known hosts.
an@localhost: Permission denied (publickey,password).
```

Host key gốc trở lại, fingerprint `SHA256:/fjhnPhFN…` như trước. Trên lab, An phải quên host thêm một lần vì vừa tin cậy key tạm; trên server thật, nếu khôi phục host key ngay khi cài lại, không client nào thấy cảnh báo cả.

Các công cụ tạo máy ảo và cấu hình (cloud-init, Ansible, Terraform) có cách giữ hoặc đặt sẵn host key. Với hệ thống lớn có nhiều server tạo và huỷ liên tục, người ta dùng **chứng chỉ SSH** (host certificate) ký bởi một CA nội bộ: client tin CA, không cần nhớ từng host key.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-host-key-known-hosts/ssh-studio-06.webp" alt="Terminal của SSH Studio: ssh-keygen -R localhost xoá dòng cũ, accept-new thêm key mới, chép host key cũ từ /root về /etc/ssh, khởi động lại ssh, fingerprint trở về SHA256:/fjhnPhFN…, xoá và thêm lại dòng của localhost trong known_hosts" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · đưa host key cũ về sau khi «dựng lại» server</figcaption></figure>

## 6. Đối chiếu thao tác

| Trên SSH Studio | Với OpenSSH |
|---|---|
| Hộp thoại Xác nhận danh tính server | Câu hỏi `Are you sure you want to continue connecting` |
| Trang Known Hosts | `~/.ssh/known_hosts`, tìm bằng `ssh-keygen -F` |
| Kết nối bị chặn vì host key đổi | `REMOTE HOST IDENTIFICATION HAS CHANGED`, `Host key verification failed` |
| Quên host | `ssh-keygen -R TÊN` |

## 7. Lỗi thường gặp

**Quên host mà không xác minh.** Toàn bộ giá trị của host key nằm ở bước xác minh. Bỏ bước đó là bỏ lớp bảo vệ.

**Dùng `StrictHostKeyChecking=no` trong script cho «khỏi phiền».** Dùng `accept-new` cho server mới; cảnh báo host key đổi trong script phải làm script dừng.

**Xoá cả `known_hosts`.** Mất dấu mọi server khác; lần sau kết nối, bạn không còn gì để so.

**Cài lại server mà không giữ host key.** Cả nhóm nhận cảnh báo, và quen dần với việc bỏ qua nó.

**Nhầm địa chỉ.** SSH Studio và OpenSSH lưu host key theo địa chỉ **và cổng** (`[127.0.0.1]:2222`). Hai server khác nhau sau cùng một địa chỉ (ví dụ đổi máy sau một load balancer) sẽ gây cảnh báo dù không ai tấn công.

## Bài tập

1. In fingerprint của cả ba host key (RSA, ECDSA, Ed25519) của server. Vì sao SSH Studio chỉ hiển thị fingerprint Ed25519?
2. `ssh-keyscan -t rsa,ecdsa,ed25519 localhost` in gì? Vì sao `ssh-keyscan` không phải là cách xác minh fingerprint lần đầu (gợi ý: nó hỏi qua chính đường mạng bạn đang nghi ngờ)?
3. Đặt `HashKnownHosts no` cho riêng bạn trong `~/.ssh/config`, xoá và thêm lại `localhost`. Dòng mới trông thế nào? Ưu, nhược điểm?
4. Viết các bước «sao lưu host key trước khi cài lại server» thành một danh sách kiểm tra ngắn cho nhóm Tới Nơi, gồm cả quyền file khi khôi phục.

## Đọc tiếp

[LNX-024 · sudo và sudoers: cấp quyền theo việc](/articles/linux-sudo-sudoers): cho Cường đúng những lệnh cần dùng, không hơn.
