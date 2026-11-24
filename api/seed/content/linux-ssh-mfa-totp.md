---
slug: linux-ssh-mfa-totp
title: "Xác thực hai bước cho SSH bằng mã TOTP: PAM, AuthenticationMethods và đăng nhập bằng SSH Studio"
summary: "Thêm lớp thứ hai cho SSH: key là thứ bạn có, mã 6 số từ ứng dụng xác thực là lớp tiếp theo. Bài giải thích TOTP, file ~/.google_authenticator, sửa /etc/pam.d/sshd để PAM hỏi mã OTP thay vì mật khẩu Unix, AuthenticationMethods publickey,keyboard-interactive trong sshd, đăng nhập bằng SSH Studio và đọc log để thấy cả hai bước."
category: devops-linux
level: advanced
tags: [linux, lnx-029, ssh, mfa, totp, pam, google-authenticator, authenticationmethods, ssh-studio, toi-noi]
language: vi
---

Sau [LNX-025](/articles/linux-sshd-config-an-toan), chỉ có key mới vào được `toinoi-app-01`. Nhưng key là một file: nằm trên laptop, trong bản sao lưu, đôi khi trong cả thư mục dự án của ai đó. Với server quan trọng, Bình muốn thêm một lớp: kể cả khi file key bị lộ, kẻ lấy được nó vẫn cần thêm **mã 6 số trên điện thoại của An**.

Đó là **xác thực đa yếu tố** (*MFA*): kết hợp thứ bạn **có** (file key, điện thoại) với thứ khác, để một thứ bị lộ chưa đủ. Bài này dùng **TOTP**, mã đổi sau mỗi 30 giây mà các ứng dụng xác thực trên điện thoại đều tạo được.

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-029 · Phần 3/6 · Nâng cao</span>
<p>Đọc trước: <a href="/articles/linux-sshd-config-an-toan">LNX-025</a> (quy trình sửa sshd an toàn và phiên dự phòng), <a href="/articles/linux-ssh-agent-passphrase">LNX-028</a>. Trạng thái đầu bài: <code>npm run setup -- 29</code>. Lab cài sẵn <code>libpam-google-authenticator</code> và <code>oathtool</code>, và dùng một bí mật cố định để bạn tính được mã OTP; bí mật đó chỉ dùng cho lab.</p>
</div>

## 1. TOTP hoạt động thế nào

Server và điện thoại giữ chung một **bí mật** (một chuỗi base32). Mỗi bên tính `HMAC(bí mật, số khoảng 30 giây kể từ 1970)`, lấy 6 chữ số. Hai bên không cần liên lạc với nhau: chỉ cần cùng bí mật và **đồng hồ** khớp nhau. Mã chỉ sống 30 giây, nên nghe lén được một mã cũng gần như vô dụng.

Trên Linux, phần «server» là module PAM **`pam_google_authenticator`** (tên lịch sử; nó làm việc với mọi ứng dụng TOTP chuẩn, không cần tài khoản Google).

## 2. Bí mật chung: ~/.google_authenticator

Thông thường, mỗi người chạy lệnh `google-authenticator` trên server: nó hỏi vài câu, sinh bí mật, in **mã QR** để quét bằng ứng dụng điện thoại, và ghi file `~/.google_authenticator`. Lab dùng một file có sẵn để mã của bạn tính được:

```text
KRUGS4ZANFZSAYJAORSXG5BAONSWG4TF
" RATE_LIMIT 3 30
" WINDOW_SIZE 3
" DISALLOW_REUSE
" TOTP_AUTH
47120385
91836402
```

| Dòng | Ý nghĩa |
|---|---|
| Dòng đầu | Bí mật base32; ai có nó là tạo được mã |
| `RATE_LIMIT 3 30` | Tối đa 3 lần đăng nhập mỗi 30 giây, chống thử mò |
| `WINDOW_SIZE 3` | Chấp nhận mã của khoảng trước và sau, bù lệch đồng hồ |
| `DISALLOW_REUSE` | Mỗi mã chỉ dùng được một lần |
| `TOTP_AUTH` | Dùng mã theo thời gian |
| Các dòng số | **Mã dự phòng** (*scratch code*), mỗi mã dùng một lần khi mất điện thoại |

```text
an@toinoi-app-01:~$ chmod 400 ~/.google_authenticator
an@toinoi-app-01:~$ ls -l ~/.google_authenticator
-r-------- 1 an an 114 Nov 23 23:36 /home/an/.google_authenticator
an@toinoi-app-01:~$ oathtool --totp -b "$(head -n 1 ~/.google_authenticator)" | wc -c
7
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-ssh-mfa-totp/ssh-studio-06.webp" alt="Terminal của SSH Studio: vim ~/.google_authenticator, chmod 400, ls -l cho quyền -r--------, oathtool --totp sinh mã từ bí mật, wc -c đếm 7 ký tự (6 chữ số và xuống dòng)" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · bí mật chung của mã OTP, chỉ chủ file đọc được</figcaption></figure>

- Quyền **`400`**: chỉ An đọc được. Module từ chối file có quyền rộng hơn.
- **`oathtool --totp -b BÍ_MẬT`** tính mã hiện tại từ bí mật, giống ứng dụng điện thoại. Bài in số ký tự (6 số và dấu xuống dòng) thay vì in mã, vì mã đổi mỗi 30 giây.

Trên máy thật, bạn thêm bí mật này vào ứng dụng xác thực (gõ tay, chọn loại «theo thời gian»), hoặc dùng `oathtool` như lab.

## 3. PAM: hỏi mã OTP thay vì mật khẩu Unix

**PAM** (*Pluggable Authentication Modules*) là lớp xác thực mà `sshd`, `login`, `sudo` cùng dùng. File `/etc/pam.d/sshd` liệt kê các module chạy khi `sshd` nhờ PAM xác thực. Mặc định, dòng `@include common-auth` hỏi **mật khẩu Unix**. Ở đây, An muốn PAM hỏi **mã OTP** thay vào đó (lớp đầu đã là key):

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-ssh-mfa-totp/ssh-studio-01.webp" alt="Vim mở /etc/pam.d/sshd qua sudoedit trong SSH Studio: dòng @include common-auth đã được chú thích, thêm auth required pam_google_authenticator.so nullok và auth required pam_permit.so" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>sudoedit /etc/pam.d/sshd</code>: PAM hỏi mã OTP thay vì mật khẩu</figcaption></figure>

```text
an@toinoi-app-01:~$ grep -n -E 'google|permit|common-auth' /etc/pam.d/sshd
5:# @include common-auth
6:auth required pam_google_authenticator.so nullok
7:auth required pam_permit.so
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-ssh-mfa-totp/ssh-studio-07.webp" alt="Terminal của SSH Studio: sudoedit /etc/pam.d/sshd, grep -n cho dòng 5 # @include common-auth, dòng 6 auth required pam_google_authenticator.so nullok, dòng 7 auth required pam_permit.so" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · PAM của sshd sau khi sửa: hỏi mã OTP, bỏ mật khẩu Unix</figcaption></figure>

- **`# @include common-auth`**: tắt bước hỏi mật khẩu Unix cho SSH. `sudo` vẫn hỏi mật khẩu như cũ (nó dùng `/etc/pam.d/sudo`).
- **`auth required pam_google_authenticator.so nullok`**: hỏi mã OTP. `nullok` cho phép người **chưa có** file `~/.google_authenticator` đi qua bước này; tiện khi triển khai dần cho cả nhóm, nhưng cũng nghĩa là ai chưa đăng ký thì chưa có lớp thứ hai. Bỏ `nullok` khi mọi người đã đăng ký xong.
- **`auth required pam_permit.so`**: khi `nullok` để một người đi qua, module trả về «bỏ qua», và một chuỗi PAM không có module nào nói «thành công» sẽ thất bại. `pam_permit` cung cấp kết quả thành công đó.

Sửa PAM nguy hiểm không kém sửa `sshd`: **giữ phiên đang mở** cho tới khi phiên mới vào được.

## 4. sshd: cần cả key lẫn mã OTP

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-ssh-mfa-totp/ssh-studio-02.webp" alt="Vim mở /etc/ssh/sshd_config.d/05-an-toan.conf qua sudoedit trong SSH Studio, với KbdInteractiveAuthentication yes và AuthenticationMethods publickey,keyboard-interactive cùng các dòng gia cố của LNX-025" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>AuthenticationMethods publickey,keyboard-interactive</code></figcaption></figure>

```text
# Gia cố sshd cho toinoi-app-01 (LNX-025, LNX-029).
PasswordAuthentication no
KbdInteractiveAuthentication yes
AuthenticationMethods publickey,keyboard-interactive
PermitRootLogin no
AllowGroups ssh-users
MaxAuthTries 3
X11Forwarding no
```

- **`KbdInteractiveAuthentication yes`**: bật kiểu xác thực hỏi-đáp, kênh để PAM hỏi «Verification code:».
- **`AuthenticationMethods publickey,keyboard-interactive`**: dấu **phẩy** nghĩa là **phải qua cả hai, theo thứ tự**: key trước, rồi hỏi-đáp. Dấu **cách** giữa hai danh sách nghĩa là «một trong các danh sách» (ví dụ `publickey,keyboard-interactive publickey,password`).
- `PasswordAuthentication no` vẫn giữ: không ai vào chỉ bằng mật khẩu.

```text
an@toinoi-app-01:~$ sudo sshd -t && echo "cú pháp ổn"
cú pháp ổn
an@toinoi-app-01:~$ sudo sshd -T | grep -E '^(usepam|kbdinteractiveauthentication|authenticationmethods) '
usepam yes
kbdinteractiveauthentication yes
authenticationmethods publickey,keyboard-interactive
an@toinoi-app-01:~$ sudo systemctl reload ssh
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-ssh-mfa-totp/ssh-studio-08.webp" alt="Terminal của SSH Studio: sudoedit 05-an-toan.conf, sshd -t báo cú pháp ổn, sshd -T cho usepam yes, kbdinteractiveauthentication yes, authenticationmethods publickey,keyboard-interactive, systemctl reload ssh" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · sshd đòi cả key lẫn mã OTP</figcaption></figure>

## 5. Đăng nhập bằng SSH Studio

Không cần đổi gì trong hồ sơ server: SSH Studio vẫn dùng key `an-laptop`. Mở một phiên mới; sau khi key được chấp nhận, server hỏi tiếp:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-ssh-mfa-totp/ssh-studio-03.webp" alt="SSH Studio mở phiên mới tới toinoi-app-01, sau khi key được chấp nhận hiện hộp thoại hỏi Verification code từ server với ô nhập mã và nút Tiếp tục" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · server hỏi mã OTP sau bước key</figcaption></figure>

An mở ứng dụng xác thực, gõ 6 số đang hiện, bấm **Tiếp tục**:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-ssh-mfa-totp/ssh-studio-04.webp" alt="Tab phiên mới tới toinoi-app-01 trong SSH Studio đã kết nối sau khi nhập đúng mã OTP, hiển thị dấu nhắc an@toinoi-app-01" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · đã vào sau hai bước: key và mã OTP</figcaption></figure>

Hộp thoại này là câu hỏi của **server** (qua PAM), SSH Studio chỉ hiển thị nó; vì vậy cùng cấu hình này cũng chạy với OpenSSH, PuTTY hay bất kỳ client nào hỗ trợ keyboard-interactive. Với phương thức **MFA / OTP** trong hồ sơ server, SSH Studio còn dùng được kiểu mật khẩu cộng OTP trên những server cấu hình như vậy.

## 6. Bằng chứng trong log

Lab thử hai lần đăng nhập: một lần chỉ có key (không trả lời được câu hỏi mã), một lần có cả key và mã:

```text
an@toinoi-app-01:~$ sudo journalctl -u ssh --no-pager | grep -E 'Accepted|Partial|Failed|Connection closed by authenticating' | tail -n 4
Nov 23 23:36:14 toinoi-app-01 sshd[2189]: Accepted publickey for an from ::1 port 53200 ssh2: ED25519 SHA256:CVW2g3F0l0Yc+ybGD3V8Cq6MlzOmETp2tds5piMsbc0
Nov 23 23:36:15 toinoi-app-01 sshd[2254]: Connection closed by authenticating user an 127.0.0.1 port 48592 [preauth]
Nov 23 23:36:15 toinoi-app-01 sshd(pam_google_authenticator)[2261]: Accepted google_authenticator for an
Nov 23 23:36:15 toinoi-app-01 sshd[2259]: Accepted keyboard-interactive/pam for an from 127.0.0.1 port 48600 ssh2
```

- Lần chỉ có key: `Connection closed by authenticating user an … [preauth]`. Key đúng, nhưng chưa đủ; kết nối bị đóng ở bước hỏi mã.
- Lần đủ hai bước: `pam_google_authenticator` ghi `Accepted google_authenticator for an`, rồi `sshd` ghi **`Accepted keyboard-interactive/pam`**: bước cuối của chuỗi phương thức.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-ssh-mfa-totp/ssh-studio-05.webp" alt="Terminal của SSH Studio: journalctl lọc log sshd với dòng Connection closed by authenticating user an preauth, Accepted google_authenticator for an và Accepted keyboard-interactive/pam for an" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · log: key một mình không đủ, key và mã OTP thì vào được</figcaption></figure>

## 7. Vận hành MFA cho cả nhóm

- **Mã dự phòng**: in ra, cất nơi an toàn. Mất điện thoại mà không có mã dự phòng thì cần người khác có `sudo` dọn hộ (xoá hoặc tạo lại `~/.google_authenticator`).
- **Đồng hồ server**: lệch quá `WINDOW_SIZE` là mọi mã đều sai. Đồng bộ giờ bằng NTP (`timedatectl`, sẽ gặp lại ở Phần 4).
- **Tài khoản tự động** (deploy, CI) không có điện thoại: cho chúng ngoại lệ bằng khối `Match User`, giới hạn chặt việc chúng được làm ([LNX-030](/articles/linux-thuc-hanh-khoa-dang-nhap-ssh)).
- **Đừng cất bí mật TOTP cạnh key.** Bí mật TOTP lưu trong trình quản lý mật khẩu cùng nơi với key là gộp hai yếu tố về một.

## 8. Lỗi thường gặp

**Bỏ `@include common-auth` mà quên `KbdInteractiveAuthentication yes`.** Server không có kênh nào để hỏi mã: `Permission denied (publickey)` sau khi key đã đúng.

**Dùng dấu cách thay dấu phẩy trong `AuthenticationMethods`.** `publickey keyboard-interactive` nghĩa là **một trong hai**: chỉ key là vào được, không có lớp thứ hai.

**`~/.google_authenticator` quyền rộng hoặc không thuộc người dùng.** Module từ chối, ghi lý do vào journal.

**Đồng hồ lệch.** Mã từ điện thoại «luôn sai» dù gõ đúng.

**Thử mã đã dùng.** Với `DISALLOW_REUSE`, cùng một mã không đăng nhập được hai lần; đợi mã kế tiếp.

## Bài tập

1. Tính mã hiện tại bằng `oathtool`, đăng nhập bằng SSH Studio, rồi thử dùng lại đúng mã đó cho một phiên khác trong cùng 30 giây. Journal ghi gì?
2. Đăng nhập bằng một mã dự phòng. Mở `~/.google_authenticator` (cần `chmod 600` tạm thời): mã đó còn không?
3. Đổi `AuthenticationMethods` thành `publickey,keyboard-interactive publickey` (có dấu cách). Chuyện gì xảy ra với lớp thứ hai? Đặt lại đúng.
4. Bình chưa có `~/.google_authenticator`. Với `nullok`, Bình đăng nhập bằng key có bị hỏi mã không? Bỏ `nullok` thì sao? Thử bằng `sudo -u binh` và `ssh -o BatchMode=yes` (Bình cần key trong `authorized_keys`).

## Đọc tiếp

[LNX-030 · Thực hành: khoá đăng nhập SSH cho toinoi-app-01](/articles/linux-thuc-hanh-khoa-dang-nhap-ssh): ghép mọi thứ của Phần 3, thêm tài khoản `deploy` cho CI chỉ dùng key, và một danh sách kiểm tra cuối.
