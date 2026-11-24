---
slug: linux-ssh-key-ssh-studio
title: "SSH key: nhập key vào SSH Studio, cài public key lên server và đăng nhập không cần mật khẩu"
summary: "Vì sao SSH key an toàn hơn mật khẩu, cặp khoá công khai và khoá riêng tư hoạt động thế nào, nhập key vào thư viện của SSH Studio (hoặc tạo key Ed25519 mới), cài public key vào ~/.ssh/authorized_keys với đúng quyền, chuyển server sang xác thực bằng key, kiểm chứng trong log của sshd, và lỗi StrictModes khi quyền file sai."
category: devops-linux
level: beginner
tags: [linux, lnx-022, ssh, ssh-key, ed25519, authorized-keys, ssh-studio, toi-noi]
language: vi
---

Mỗi lần mở SSH Studio, An lại gõ mật khẩu `toinoi-lab`. Mật khẩu đó đang đi qua mạng (mã hoá, nhưng vẫn đi), được dùng chung cho cả `an` lẫn `sudo`, và nếu ai đó đoán ra thì vào được server từ bất cứ đâu. Bình muốn trước cuối tuần, không ai đăng nhập `toinoi-app-01` bằng mật khẩu nữa.

Thay thế là **SSH key**: một cặp khoá mà nửa bí mật không bao giờ rời máy của An. Bài này cài key cho `an`, chuyển SSH Studio sang dùng key, và kiểm chứng bằng log của server. Tắt hẳn mật khẩu là việc của [LNX-025](/articles/linux-sshd-config-an-toan), sau khi chắc chắn key đã chạy.

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-022 · Phần 3/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/linux-dung-lab-docker-ket-noi-ssh-studio">LNX-002</a>, <a href="/articles/linux-quan-ly-nguoi-dung-nhom">LNX-021</a>. Trạng thái đầu bài: <code>npm run setup -- 22</code>. Lab có sẵn một cặp key «trên laptop của An» ở <code>examples/toi-noi-linux/may-cua-ban/khoa-lab/</code> để kết quả của bạn trùng với bài; key đó chỉ dùng cho lab. Sau bài này, SSH Studio đăng nhập vào server bằng key, không hỏi mật khẩu.</p>
</div>

## 1. Key hoạt động thế nào

Một cặp SSH key gồm hai phần sinh ra cùng lúc:

| Phần | Nằm ở đâu | Ai được thấy |
|---|---|---|
| **Khoá riêng tư** (private key) | Máy của bạn, trong SSH Studio hoặc `~/.ssh/id_ed25519` | Chỉ bạn. Lộ là phải thay key |
| **Khoá công khai** (public key) | `~/.ssh/authorized_keys` trên mỗi server bạn được vào | Ai cũng được, chép thoải mái |

Khi đăng nhập, server gửi một thử thách; client ký thử thách đó bằng khoá riêng tư; server kiểm chữ ký bằng khoá công khai đã lưu. **Khoá riêng tư không bao giờ đi qua mạng.** Kẻ nghe lén không lấy được gì, server bị chiếm cũng không lộ khoá riêng tư của bạn, và không có gì để đoán: một key Ed25519 tương đương một mật khẩu dài hàng chục ký tự ngẫu nhiên.

Hôm nay có ba thuật toán hay gặp: **Ed25519** (mặc định nên dùng: ngắn, nhanh, an toàn), ECDSA, và RSA (cũ nhưng vẫn phổ biến; nếu dùng thì ít nhất 3072 bit). DSA đã bị bỏ.

## 2. Server đang nhận những cách đăng nhập nào?

```text
an@toinoi-app-01:~$ ssh -o BatchMode=yes -o StrictHostKeyChecking=accept-new an@localhost true
Warning: Permanently added 'localhost' (ED25519) to the list of known hosts.
an@localhost: Permission denied (publickey,password).
an@toinoi-app-01:~$ ls -la ~/.ssh
total 16
drwx------ 2 an an 4096 Nov 23 23:36 .
drwxr-x--- 1 an an 4096 Nov 23 23:36 ..
-rw-r--r-- 1 an an  142 Nov 23 23:36 known_hosts
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-ssh-key-ssh-studio/ssh-studio-07.webp" alt="Terminal của SSH Studio: ssh -o BatchMode=yes an@localhost true báo Permission denied (publickey,password), ls -la ~/.ssh chỉ có known_hosts" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · dò từ phía client: server nhận <code>publickey</code> và <code>password</code></figcaption></figure>

Đây là một mẹo dò từ phía client: `BatchMode=yes` cấm `ssh` hỏi bất cứ điều gì, nên khi không có key phù hợp, nó dừng và in **danh sách phương thức server chấp nhận**: `publickey,password`. Ở LNX-025, danh sách này sẽ chỉ còn `publickey`. (Lần đầu kết nối, `accept-new` tự tin cậy host key của `localhost`; [LNX-023](/articles/linux-host-key-known-hosts) nói về chuyện đó.)

`~/.ssh` của `an` mới chỉ có `known_hosts`. Chưa có `authorized_keys`.

## 3. Key trong SSH Studio

Vào **SSH Keys** ở cột trái. Có hai cách có key trong thư viện:

- **Tạo key mới**: SSH Studio sinh một cặp Ed25519. Đây là cách nên dùng với key thật của bạn.
- **Nhập key**: dùng key bạn đã có (OpenSSH, PEM, hoặc PuTTY `.ppk`).

Để kết quả trùng với bài, An nhập key của lab: bấm **Nhập key**, đặt tên `an-laptop`, bấm **Chọn file private key** và chọn `may-cua-ban/khoa-lab/an-laptop`:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-ssh-key-ssh-studio/ssh-studio-01.webp" alt="Hộp thoại Nhập SSH key của SSH Studio: tên key an-laptop, nút Private key hiển thị đường dẫn tới file an-laptop trong thư mục khoa-lab của lab, ô Passphrase để trống, ghi chú private key được mã hoá AES-256-GCM với khoá bảo vệ trong Windows Credential Manager" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <b>Nhập SSH key</b> từ file của lab</figcaption></figure>

Bấm **Nhập key**. Key xuất hiện trong thư viện cùng **fingerprint**: dấu vân tay SHA-256 của khoá công khai, thứ để đối chiếu với phía server:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-ssh-key-ssh-studio/ssh-studio-02.webp" alt="Trang SSH Keys của SSH Studio với key an-laptop loại ED25519, fingerprint SHA256:yJ42WJZlgwZsUsCCjvAX237jMd5+ssxlWKtoaNrMTgY, 0 server sử dụng, các nút Public key, Xuất private key, Xoá; thông báo Đã sao chép public key" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · thư viện key: fingerprint và nút sao chép <b>Public key</b></figcaption></figure>

SSH Studio mã hoá khoá riêng tư bằng AES-256-GCM, còn khoá dùng để mã hoá nằm trong kho khoá của hệ điều hành (trên Windows là Credential Manager). Giao diện chỉ hiển thị khoá công khai. Bấm **Public key** để chép khoá công khai vào clipboard, dạng `ssh-ed25519 AAAA… an-laptop`.

## 4. Cài public key lên server

Trong terminal (vẫn đang đăng nhập bằng mật khẩu), dán khoá công khai vào `~/.ssh/authorized_keys`:

```text
an@toinoi-app-01:~$ mkdir -p ~/.ssh
an@toinoi-app-01:~$ chmod 700 ~/.ssh
an@toinoi-app-01:~$ echo 'ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIFvQW8mHkVjU1nOtQ4pdIuRs5X14dOyuz6HzyQOw026x an-laptop' >> ~/.ssh/authorized_keys
an@toinoi-app-01:~$ chmod 600 ~/.ssh/authorized_keys
an@toinoi-app-01:~$ ls -la ~/.ssh
total 20
drwx------ 2 an an 4096 Nov 23 23:36 .
drwxr-x--- 1 an an 4096 Nov 23 23:36 ..
-rw------- 1 an an   91 Nov 23 23:36 authorized_keys
-rw-r--r-- 1 an an  142 Nov 23 23:36 known_hosts
an@toinoi-app-01:~$ ssh-keygen -lf ~/.ssh/authorized_keys
256 SHA256:yJ42WJZlgwZsUsCCjvAX237jMd5+ssxlWKtoaNrMTgY an-laptop (ED25519)
```

| Bước | Vì sao |
|---|---|
| `chmod 700 ~/.ssh` | Chỉ chủ vào được thư mục |
| `echo '…' >> ~/.ssh/authorized_keys` | Mỗi dòng một khoá công khai. **`>>`**, không phải `>`: `>` xoá mọi key cũ ([LNX-008](/articles/linux-chuyen-huong-pipe)) |
| `chmod 600 ~/.ssh/authorized_keys` | Chỉ chủ đọc và ghi |
| `ssh-keygen -lf` | In fingerprint của từng dòng: phải trùng với fingerprint trong SSH Studio |

Fingerprint `SHA256:yJ42WJZl…` trùng với hình trên: đúng key đã được cài. Bước đối chiếu này nghe thừa, nhưng khi bạn dán key qua chat, email hay một tài liệu, một ký tự bị cắt là key không khớp.

Trên máy có OpenSSH, việc này có lệnh riêng: `ssh-copy-id -i ~/.ssh/id_ed25519.pub an@server` làm đủ các bước trên qua một lần đăng nhập bằng mật khẩu.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-ssh-key-ssh-studio/ssh-studio-03.webp" alt="Terminal của SSH Studio: mkdir và chmod 700 cho ~/.ssh, echo khoá công khai ssh-ed25519 vào authorized_keys, chmod 600, ls -la ~/.ssh với authorized_keys quyền rw-------, ssh-keygen -lf in fingerprint SHA256:yJ42WJZl… an-laptop" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · cài khoá công khai vào <code>~/.ssh/authorized_keys</code></figcaption></figure>

## 5. Chuyển SSH Studio sang key

Trên trang **Servers**, bấm biểu tượng bút chì ở dòng `toinoi-app-01`. Ở **Phương thức xác thực**, chọn **SSH Key**, rồi chọn `an-laptop` trong danh sách:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-ssh-key-ssh-studio/ssh-studio-04.webp" alt="Hộp thoại Chỉnh sửa server của SSH Studio cho toinoi-app-01: phương thức xác thực SSH Key được chọn và ô SSH key chọn an-laptop" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · server <code>toinoi-app-01</code> chuyển sang xác thực bằng <b>SSH Key</b></figcaption></figure>

Bấm **Lưu**. **Đừng đóng phiên đang mở.** Phiên đó đăng nhập bằng mật khẩu và là đường lui nếu key có vấn đề. Thay vào đó, mở một phiên mới bằng nút **Mở kết nối khác** (dấu `+` cạnh các tab):

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-ssh-key-ssh-studio/ssh-studio-05.webp" alt="Tab phiên mới tới toinoi-app-01 trong SSH Studio, đã kết nối mà không có hộp thoại mật khẩu nào, hiển thị lời chào của server và dấu nhắc an@toinoi-app-01" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · phiên mới đăng nhập bằng key, không hỏi mật khẩu</figcaption></figure>

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-ssh-key-ssh-studio/ssh-studio-09.webp" alt="Trang Servers của SSH Studio: toinoi-app-01 (cuong) xác thực Password, toinoi-app-01 xác thực SSH key với 2 phiên hoạt động" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · cột Xác thực trên trang Servers cho biết profile nào đã dùng key</figcaption></figure>

Không có hộp thoại mật khẩu nào. Đây là thói quen nên giữ mỗi khi đổi cách đăng nhập: **thử bằng một phiên thứ hai trước, giữ phiên cũ cho tới khi chắc chắn**.

## 6. Kiểm chứng trong log của sshd

```text
an@toinoi-app-01:~$ sudo journalctl -u ssh --no-pager | grep -E 'Accepted (password|publickey) for an' | tail -n 2
Nov 23 23:36:08 toinoi-app-01 sshd[1751]: Accepted publickey for an from 127.0.0.1 port 48568 ssh2: ED25519 SHA256:yJ42WJZlgwZsUsCCjvAX237jMd5+ssxlWKtoaNrMTgY
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-ssh-key-ssh-studio/ssh-studio-08.webp" alt="Terminal của SSH Studio: journalctl lọc hai dòng Accepted publickey for an kèm fingerprint ED25519 SHA256:yJ42WJZ…" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · log của sshd xác nhận đăng nhập bằng key</figcaption></figure>

`sshd` ghi mỗi lần đăng nhập thành công vào journal của systemd (Phần 4 dạy `journalctl` kỹ hơn), kèm **phương thức** (`publickey`) và **fingerprint của key đã dùng**. Đây là cách duy nhất biết chắc ai vào server bằng key nào. Khi server có nhiều người, đặt tên cho từng key (phần chú thích ở cuối dòng `authorized_keys`, ở đây là `an-laptop`) để đọc log dễ hơn.

Bản ghi của lab có địa chỉ nguồn `127.0.0.1`, vì runner mô phỏng client từ trong server; lần đăng nhập từ SSH Studio trên máy bạn có địa chỉ của mạng Docker (`172.x.x.x`). Fingerprint thì trùng.

## 7. Quyền sai: sshd lặng lẽ từ chối key

```text
an@toinoi-app-01:~$ chmod o+w ~/.ssh/authorized_keys
an@toinoi-app-01:~$ ls -l ~/.ssh/authorized_keys
-rw-----w- 1 an an 91 Nov 23 23:36 /home/an/.ssh/authorized_keys
an@toinoi-app-01:~$ sudo journalctl -u ssh --no-pager | grep -o 'Authentication refused: .*' | tail -n 1
Authentication refused: bad ownership or modes for file /home/an/.ssh/authorized_keys
an@toinoi-app-01:~$ chmod 600 ~/.ssh/authorized_keys
an@toinoi-app-01:~$ ls -l ~/.ssh/authorized_keys
-rw------- 1 an an 91 Nov 23 23:36 /home/an/.ssh/authorized_keys
```

Một người khác ghi được vào `authorized_keys` thì người đó tự thêm key của họ và vào bằng tài khoản của bạn. Vì vậy `sshd` có chế độ **`StrictModes`** (bật mặc định): nếu file `authorized_keys`, thư mục `~/.ssh` hoặc thư mục riêng ghi được bởi người khác, key bị từ chối. Phía client chỉ thấy «Permission denied (publickey)» như thể key sai. Lý do thật chỉ có trong log của server: `bad ownership or modes`.

Debian và Ubuntu có một bản vá nới lỏng: file ghi được bởi **nhóm riêng** của chính bạn (nhóm chỉ có mình bạn) vẫn được chấp nhận. Vì vậy ví dụ trên dùng `o+w` (người khác ghi được) thay cho `g+w`.

Quy tắc cần thuộc:

| Đường dẫn | Quyền |
|---|---|
| Thư mục riêng (`/home/an`) | Không ai khác ghi được (755 hoặc 750) |
| `~/.ssh` | 700 |
| `~/.ssh/authorized_keys` | 600 |
| Khoá riêng tư (`~/.ssh/id_ed25519`) | 600 |

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-ssh-key-ssh-studio/ssh-studio-06.webp" alt="Terminal của SSH Studio: chmod o+w authorized_keys, ls -l thấy -rw----w-, journalctl ghi Authentication refused: bad ownership or modes for file authorized_keys, chmod 600 trả lại quyền -rw-------" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · quyền sai trên <code>authorized_keys</code>: sshd lặng lẽ từ chối key</figcaption></figure>

## 8. Đối chiếu thao tác

| Trên SSH Studio | Với OpenSSH |
|---|---|
| SSH Keys → Tạo key mới | `ssh-keygen -t ed25519 -C "an@laptop"` |
| SSH Keys → Nhập key | Đặt file vào `~/.ssh/` hoặc chỉ định bằng `-i` |
| Nút Public key | `cat ~/.ssh/id_ed25519.pub` |
| Fingerprint trong thư viện | `ssh-keygen -lf ~/.ssh/id_ed25519.pub` |
| Sửa server → SSH Key | `ssh -i ~/.ssh/id_ed25519 an@server` hoặc `IdentityFile` trong `~/.ssh/config` ([LNX-028](/articles/linux-ssh-agent-passphrase)) |
| (không có) | `ssh-copy-id` để cài key |

## 9. Lỗi thường gặp

**Dán khoá riêng tư thay vì khoá công khai.** Khoá riêng tư bắt đầu bằng `-----BEGIN OPENSSH PRIVATE KEY-----`. Nếu lỡ dán lên server hay vào chat, coi như key đã lộ: xoá khỏi `authorized_keys` mọi nơi và tạo key mới.

**`>` thay vì `>>`.** Mất các key khác, có thể khoá chính mình ngoài cửa.

**Key bị xuống dòng khi dán.** Mỗi khoá công khai phải nằm trên **một** dòng. `ssh-keygen -lf` báo lỗi nếu dòng hỏng.

**Quyền sai.** Xem mục 7; đọc log của server, đừng đoán.

**Đóng phiên cũ trước khi thử phiên mới.** Nếu key không chạy và server đã tắt mật khẩu, bạn cần console của nhà cung cấp hoặc người khác cứu.

## Bài tập

1. Trong SSH Studio, bấm **Tạo key mới** để tạo key `an-thu`, chép public key và cài thêm vào `authorized_keys`. `ssh-keygen -lf` liệt kê mấy key?
2. Gỡ key `an-thu` khỏi server bằng `sed -i '/an-thu$/d' ~/.ssh/authorized_keys` ([LNX-014](/articles/linux-sed-sua-van-ban)). Kiểm tra lại.
3. Trong `authorized_keys` có thể đặt tuỳ chọn trước một key, ví dụ `from="10.0.0.0/8" ssh-ed25519 …`. Tra `man sshd` mục AUTHORIZED_KEYS FILE FORMAT: `from=`, `command=`, `no-port-forwarding` làm gì?
4. Đặt `chmod 777 ~` (tạm thời!) rồi mở thêm một phiên bằng SSH Studio. Chuyện gì xảy ra? Log báo gì? Trả lại `chmod 750 ~`.
5. So sánh: vì sao khi một người nghỉ việc, xoá key của họ khỏi `authorized_keys` quan trọng hơn khoá mật khẩu (xem [LNX-021](/articles/linux-quan-ly-nguoi-dung-nhom))?

## Đọc tiếp

[LNX-023 · Host key, Known Hosts và cảnh báo host key thay đổi](/articles/linux-host-key-known-hosts): key của server, và vì sao SSH Studio chặn kết nối khi nó đổi.
