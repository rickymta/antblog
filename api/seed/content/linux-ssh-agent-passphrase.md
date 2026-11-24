---
slug: linux-ssh-agent-passphrase
title: "ssh-agent và passphrase: bảo vệ private key mà không phải gõ mật khẩu mỗi lần, và ~/.ssh/config cho nhiều key"
summary: "Vì sao private key nên có passphrase, ssh-agent giữ key đã mở khoá trong bộ nhớ ra sao, ssh-add, ssh-add -l, -D, ~/.ssh/config với Host, IdentityFile, IdentitiesOnly và ssh -G, rủi ro của agent forwarding, và SSH Studio làm gì thay cho agent: vault mã hoá trên máy, phương thức Agent khi Windows chưa bật dịch vụ ssh-agent."
category: devops-linux
level: advanced
tags: [linux, lnx-028, ssh-agent, ssh-add, passphrase, ssh-config, identitiesonly, ssh-studio, toi-noi]
language: vi
---

Key của An ở [LNX-022](/articles/linux-ssh-key-ssh-studio) không có passphrase. File private key là chìa khoá trần: ai chép được file đó (laptop bị mất, bản sao lưu bị lộ, một phần mềm độc hại đọc thư mục `~/.ssh`) là vào được mọi server có public key tương ứng, cho tới khi An phát hiện và gỡ key.

**Passphrase** mã hoá file private key; file bị lộ thì vẫn cần passphrase mới dùng được. Cái giá: phải gõ passphrase mỗi lần dùng key. **`ssh-agent`** xoá cái giá đó: mở khoá một lần, agent giữ key đã giải mã trong bộ nhớ cho cả phiên làm việc.

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-028 · Phần 3/6 · Nâng cao</span>
<p>Đọc trước: <a href="/articles/linux-ssh-key-ssh-studio">LNX-022</a>, <a href="/articles/linux-sshd-config-an-toan">LNX-025</a> (server chỉ nhận key). Trạng thái đầu bài: <code>npm run setup -- 28</code>. Phần OpenSSH chạy ngay trên server (server đóng vai «máy của An» kết nối tới chính nó). Passphrase của lab nằm trong lệnh để bạn chạy lại được; với key thật, đừng bao giờ đặt passphrase trên dòng lệnh.</p>
</div>

## 1. Key có passphrase

```text
an@toinoi-app-01:~$ ssh-keygen -q -t ed25519 -C an@toinoi-app-01 -N mot-cum-tu-chi-dung-cho-lab -f ~/.ssh/id_ed25519
an@toinoi-app-01:~$ ls -l ~/.ssh
total 20
-rw------- 1 an an  91 Nov 23 23:36 authorized_keys
-rw------- 1 an an 464 Nov 23 23:36 id_ed25519
-rw-r--r-- 1 an an  98 Nov 23 23:36 id_ed25519.pub
-rw-r--r-- 1 an an 142 Nov 23 23:36 known_hosts
-rw-r--r-- 1 an an 142 Nov 23 23:36 known_hosts.old
an@toinoi-app-01:~$ head -n 1 ~/.ssh/id_ed25519
-----BEGIN OPENSSH PRIVATE KEY-----
an@toinoi-app-01:~$ ssh-keygen -y -P sai-passphrase -f ~/.ssh/id_ed25519
Load key "/home/an/.ssh/id_ed25519": incorrect passphrase supplied to decrypt private key
an@toinoi-app-01:~$ cat ~/.ssh/id_ed25519.pub >> ~/.ssh/authorized_keys
an@toinoi-app-01:~$ ssh -o BatchMode=yes localhost hostname
an@localhost: Permission denied (publickey).
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-ssh-agent-passphrase/ssh-studio-04.webp" alt="Terminal của SSH Studio: ssh-keygen tạo id_ed25519 có passphrase, ls -l ~/.ssh thấy id_ed25519 quyền -rw------- và id_ed25519.pub, head -n 1 in BEGIN OPENSSH PRIVATE KEY" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · tạo key Ed25519 có passphrase ngay trên server</figcaption></figure>

- **`ssh-keygen -t ed25519`** tạo cặp key Ed25519; `-C` là chú thích (thường là `người@máy`), `-N` là passphrase, `-f` là file. Bình thường bạn bỏ `-N` và `ssh-keygen` hỏi passphrase hai lần, không hiện ra màn hình.
- Private key `id_ed25519` có quyền `600`, public key `.pub` là `644`.
- Nhìn bên ngoài, file có passphrase và không có trông giống nhau (`BEGIN OPENSSH PRIVATE KEY`). `ssh-keygen -y` (in public key từ private key) với passphrase sai cho biết file đã được mã hoá.
- Public key được thêm vào `authorized_keys` của chính `an`; nhưng `ssh -o BatchMode=yes` (không được hỏi gì) thất bại: `ssh` cần passphrase để dùng key, và không được phép hỏi.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-ssh-agent-passphrase/ssh-studio-01.webp" alt="Terminal của SSH Studio: ssh-keygen -y với passphrase sai báo incorrect passphrase supplied to decrypt private key, thêm public key vào authorized_keys, ssh -o BatchMode=yes localhost bị Permission denied (publickey)" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · key có passphrase: không có agent thì BatchMode không dùng được</figcaption></figure>

### Passphrase tốt

Passphrase chỉ phải chống lại một người **đã có** file key, đang thử đoán ngoại tuyến. Một cụm 4–5 từ ngẫu nhiên dễ nhớ và đủ dài. Định dạng key OpenSSH mới dùng `bcrypt` để làm chậm việc thử; `ssh-keygen -a 100` tăng số vòng lặp nếu bạn muốn chậm hơn nữa.

Đổi passphrase của key có sẵn, không đổi key: `ssh-keygen -p -f ~/.ssh/id_ed25519`.

## 2. ssh-agent giữ key đã mở khoá

```text
an@toinoi-app-01:~$ eval "$(ssh-agent -s)"
Agent pid 2166
an@toinoi-app-01:~$ ssh-add ~/.ssh/id_ed25519
Identity added: /home/an/.ssh/id_ed25519 (an@toinoi-app-01)
an@toinoi-app-01:~$ ssh-add -l
256 SHA256:CVW2g3F0l0Yc+ybGD3V8Cq6MlzOmETp2tds5piMsbc0 an@toinoi-app-01 (ED25519)
an@toinoi-app-01:~$ ssh -o BatchMode=yes localhost hostname
toinoi-app-01
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-ssh-agent-passphrase/ssh-studio-05.webp" alt="Terminal của SSH Studio: eval ssh-agent in Agent pid, ssh-add hỏi passphrase rồi Identity added, ssh-add -l liệt kê fingerprint ED25519, ssh BatchMode localhost hostname in toinoi-app-01" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · nhập passphrase một lần, agent giữ key cho các lần sau</figcaption></figure>

- **`ssh-agent -s`** khởi động agent và in ra các lệnh `export SSH_AUTH_SOCK=…` (đường dẫn socket để `ssh` tìm agent); `eval "$(…)"` chạy chúng trong shell hiện tại.
- **`ssh-add FILE`** hỏi passphrase **một lần**, rồi giao key đã giải mã cho agent. Trên lab, passphrase được nhập tự động; bạn gõ tay.
- **`ssh-add -l`** liệt kê key agent đang giữ (fingerprint và chú thích).
- Bây giờ `ssh -o BatchMode=yes` thành công: `ssh` nhờ agent ký, không cần đọc file key hay hỏi passphrase.

Agent **không bao giờ đưa key ra ngoài**: `ssh` gửi dữ liệu cần ký vào socket, agent trả lại chữ ký. Socket chỉ chủ của nó đọc được (`/tmp/ssh-XXXX/agent.N`, quyền `600`).

Trên máy làm việc, bạn hiếm khi tự chạy `ssh-agent`: macOS có agent gắn với Keychain, các desktop Linux khởi động agent khi đăng nhập, Windows có dịch vụ **OpenSSH Authentication Agent** (tắt sẵn; bật bằng `Set-Service ssh-agent -StartupType Automatic` trong PowerShell quyền quản trị, rồi `Start-Service ssh-agent`).

**`ssh-add -t 8h`** cho key tự hết hạn sau 8 giờ: hết ngày làm việc, key rời agent.

## 3. ~/.ssh/config: tên ngắn và key riêng cho từng server

Khi có nhiều key (cá nhân, công việc, deploy), `ssh` mặc định thử **lần lượt mọi key** trong agent và trong `~/.ssh`. Mỗi lần thử bị server tính là một lần xác thực; với `MaxAuthTries 3` ([LNX-025](/articles/linux-sshd-config-an-toan)), key thứ tư chưa kịp thử đã bị ngắt: `Too many authentication failures`. File `~/.ssh/config` chỉ định rõ key nào cho server nào:

```text
Host app01
    HostName localhost
    User an
    IdentityFile ~/.ssh/id_ed25519
    IdentitiesOnly yes
```

| Dòng | Ý nghĩa |
|---|---|
| `Host app01` | Tên ngắn bạn gõ (`ssh app01`); có thể dùng `*` (`Host *.toinoi.internal`) |
| `HostName` | Địa chỉ thật |
| `User`, `Port` | Thay cho `an@…`, `-p …` |
| `IdentityFile` | Key dùng cho server này |
| `IdentitiesOnly yes` | **Chỉ** dùng key ở `IdentityFile`, không thử mọi key trong agent |

```text
an@toinoi-app-01:~$ chmod 600 ~/.ssh/config
an@toinoi-app-01:~$ ssh -G app01 | grep -E '^(hostname|user|port|identityfile|identitiesonly) '
user an
hostname localhost
port 22
identitiesonly yes
identityfile ~/.ssh/id_ed25519
an@toinoi-app-01:~$ ssh app01 hostname
toinoi-app-01
```

**`ssh -G TÊN`** in cấu hình cuối cùng `ssh` sẽ dùng cho tên đó, sau khi gộp mọi khối `Host` và mặc định, giống `sshd -T` ở phía server. Như với `sshd`, giá trị **đầu tiên** khớp sẽ thắng: đặt khối cụ thể lên trước, `Host *` xuống cuối file.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-ssh-agent-passphrase/ssh-studio-02.webp" alt="Terminal của SSH Studio: vim ~/.ssh/config, chmod 600, ssh -G app01 cho user an, hostname localhost, port 22, identitiesonly yes, identityfile ~/.ssh/id_ed25519, ssh app01 hostname in toinoi-app-01" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>~/.ssh/config</code>: tên ngắn <code>app01</code> và key riêng</figcaption></figure>

## 4. Gỡ key, tắt agent

```text
an@toinoi-app-01:~$ ssh-add -D
All identities removed.
an@toinoi-app-01:~$ ssh-add -l
The agent has no identities.
an@toinoi-app-01:~$ ssh-agent -k
unset SSH_AUTH_SOCK;
unset SSH_AGENT_PID;
echo Agent pid 2166 killed;
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-ssh-agent-passphrase/ssh-studio-06.webp" alt="Terminal của SSH Studio: ssh-add -D báo All identities removed, ssh-add -l báo The agent has no identities, ssh-agent -k dừng agent" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · gỡ key khỏi agent và tắt agent</figcaption></figure>

`ssh-add -D` xoá mọi key khỏi agent (`-d FILE` xoá một key); `ssh-agent -k` tắt agent của phiên này. Rời máy giữa chừng: `ssh-add -D`, hoặc khoá màn hình nếu agent gắn với phiên đăng nhập.

### Agent forwarding: tiện và nguy hiểm

`ssh -A server` (hoặc `ForwardAgent yes`) cho phép từ server đó dùng agent trên máy bạn, ví dụ để `git pull` bằng key của bạn. Nhưng **root trên server đó** (hoặc bất kỳ ai chiếm được server) cũng dùng được socket đó trong lúc bạn đang kết nối, để đăng nhập vào mọi nơi key của bạn mở được. Chỉ bật forwarding tới server bạn tin như máy của chính mình; đi qua bastion thì dùng **`ProxyJump`** (`ssh -J`), không cần forwarding.

## 5. SSH Studio và agent

SSH Studio có cách riêng để giải quyết bài toán «key an toàn mà không gõ lại passphrase»: khi nhập key ([LNX-022](/articles/linux-ssh-key-ssh-studio)), private key được **mã hoá bằng AES-256-GCM** và khoá bảo vệ nằm trong **Windows Credential Manager**. Key có passphrase thì bạn nhập passphrase một lần lúc **Nhập key**; sau đó, mỗi lần kết nối, SSH Studio tự mở khoá từ vault, không hỏi lại.

Nếu bạn đã dùng agent (ví dụ key nằm trên thẻ phần cứng, hoặc nhóm quản lý key bằng một agent chung), SSH Studio có phương thức xác thực **Agent**: nó nhờ agent của hệ điều hành ký, không cần key trong thư viện. Thử trên máy chưa bật dịch vụ `ssh-agent` của Windows:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-ssh-agent-passphrase/ssh-studio-03.webp" alt="Tab phiên mới tới toinoi-app-01 trong SSH Studio với phương thức Agent báo Lỗi kết nối, thông báo toinoi-app-01: early eof và nút Kết nối lại" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · phương thức <b>Agent</b> khi Windows chưa bật dịch vụ ssh-agent</figcaption></figure>

Kết nối thất bại với thông báo ngắn `early eof`: SSH Studio tìm agent của OpenSSH trên Windows, không thấy, thử tiếp Pageant (agent của PuTTY), và không có agent nào trả lời. Thông báo không nói rõ nguyên nhân, nên khi gặp `early eof` với phương thức Agent, việc đầu tiên là kiểm tra agent: trong PowerShell, `Get-Service ssh-agent` phải ở trạng thái `Running`, và `ssh-add -l` phải liệt kê được key. Có hai lựa chọn: bật dịch vụ OpenSSH Authentication Agent như ở mục 2, hoặc chuyển server về phương thức **SSH Key** với key `an-laptop` trong thư viện. Lab chọn cách thứ hai và không thay đổi cấu hình Windows của bạn.

## 6. Đối chiếu thao tác

| Việc | OpenSSH | SSH Studio |
|---|---|---|
| Tạo key có passphrase | `ssh-keygen -t ed25519` | **Tạo key mới** |
| Đổi passphrase | `ssh-keygen -p -f FILE` | Xuất, đổi, nhập lại |
| Mở khoá một lần | `ssh-add FILE` | Passphrase lúc **Nhập key**, vault giữ |
| Xem key đang giữ | `ssh-add -l` | Trang **SSH Keys** |
| Chọn key theo server | `IdentityFile` + `IdentitiesOnly yes` | Ô **SSH key** khi sửa server |
| Dùng agent của hệ thống | mặc định | Phương thức **Agent** |

## 7. Lỗi thường gặp

**Key không passphrase trên laptop.** File lộ là đủ để vào server.

**`Too many authentication failures`.** Agent có nhiều key, `ssh` thử hết. Dùng `IdentitiesOnly yes` và `IdentityFile` cho từng server.

**`Could not open a connection to your authentication agent`.** Shell không có `SSH_AUTH_SOCK`: agent chưa chạy, hoặc bạn chạy `ssh-agent` mà quên `eval`.

**`ForwardAgent yes` trong `Host *`.** Mọi server bạn vào đều mượn được agent của bạn.

**`~/.ssh/config` quyền rộng.** `ssh` từ chối file cấu hình người khác ghi được (`Bad owner or permissions`); đặt `600`.

## Bài tập

1. Đổi passphrase của `~/.ssh/id_ed25519` bằng `ssh-keygen -p`. Fingerprint có đổi không? Vì sao không cần sửa `authorized_keys`?
2. Thêm key với `ssh-add -t 60`, đợi hơn một phút rồi `ssh-add -l`. Chuyện gì xảy ra?
3. Thêm khối `Host *` với `ServerAliveInterval 30` vào **đầu** `~/.ssh/config`, rồi thử đặt `User binh` trong `Host *` ở đầu file. `ssh -G app01 | grep ^user` in gì? Chuyển `Host *` xuống cuối và so sánh.
4. Trên máy Windows của bạn (nếu muốn và có quyền quản trị), bật dịch vụ OpenSSH Authentication Agent, `ssh-add` key `an-laptop` của lab, rồi đổi server trong SSH Studio sang phương thức Agent. Ghi lại các bước để tắt lại sau đó.

## Đọc tiếp

[LNX-029 · Xác thực hai bước cho SSH (TOTP)](/articles/linux-ssh-mfa-totp): key là «thứ bạn có», mã từ điện thoại là lớp thứ hai.
