---
slug: linux-sao-luu-khoi-phuc-ssh-studio
title: "Sao lưu và khôi phục cấu hình SSH Studio: file mã hoá, khôi phục gộp, và vì sao chép workspace.json là chưa đủ"
summary: "Chuẩn bị đổi laptop: bản sao lưu của SSH Studio chứa gì (server, SSH key, mật khẩu đã lưu, known hosts, thư mục), được mã hoá thế nào (Argon2id và AES-256-GCM), vì sao chép workspace.json sang máy mới không mở được bí mật, khôi phục kiểu gộp với bước xem trước, và xử lý fingerprint khác nhau."
category: devops-linux
level: beginner
tags: [linux, lnx-053, ssh-studio, backup, restore, encryption, toi-noi]
language: vi
---

Laptop của An sắp được thay. Trong SSH Studio giờ có ba server, một jump host, key `an-laptop`, các host key đã đối chiếu, thư mục đã sắp. Làm lại từ đầu trên máy mới thì mất cả buổi, và dễ bấm «Tin cậy» nhầm một fingerprint. An cần một bản sao lưu mang sang được, và cần chắc rằng mất file đó cũng không lộ key.

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-053 · Phần 6/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/linux-ssh-key-ssh-studio">LNX-022</a> (thư viện key), <a href="/articles/linux-host-key-known-hosts">LNX-023</a> (Known Hosts). Bài làm hoàn toàn trên giao diện SSH Studio, không có lệnh nào trên server. «Máy mới» được mô phỏng bằng cách xoá <code>toinoi-app-02</code> khỏi SSH Studio rồi khôi phục.</p>
</div>

## 1. SSH Studio cất dữ liệu ở đâu

Trên Windows, cấu hình nằm trong `%APPDATA%\dev.sshstudio.desktop\workspace.json`. Mở ra xem:

```text
PS> Get-Content $env:APPDATA\dev.sshstudio.desktop\workspace.json  # rút gọn: chỉ key đầu tiên
{
  "keys": [
    {
      "id": "5ad32e25-76f9-4fd4-8dcb-857268f2d4b2",
      "name": "an-laptop",
      "algorithm": "ssh-ed25519",
      "fingerprint": "SHA256:yJ42WJZlgwZsUsCCjvAX237jMd5+ssxlWKtoa…",
      "publicKey": "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIFvQW8mH…",
      "secret": "vaNPkg+15vEPWsht+sMuXx0HWtLTkam5XqGn2X4o13Ft…"
    }
  ]
}

PS> Get-Content .\ssh-studio-2026-11-24.sshstudio-backup  # rút gọn
{
  "format": "sshstudio-backup",
  "version": 1,
  "kdf": {
    "name": "argon2id",
    "memory": 19456,
    "iterations": 2,
    "parallelism": 1,
    "salt": "GJTS/37qQRbZ5bpZ2jmvSQ=="
  },
  "nonce": "xemOcaH4sZ7OMbxu",
  "data": "gL5UpfS/kB0legDfK770gajabnoUCLx2QwRrhss0OFI5…"
}
```

Thông tin server, fingerprint, public key là chữ thường. Còn **private key, mật khẩu đã lưu, mật khẩu proxy** chỉ là chuỗi mã hoá (trường `secret`). Khoá giải mã (*vault master key*) không nằm trong file: nó ở **Windows Credential Manager** (macOS: Keychain; Linux: Secret Service), gắn với tài khoản Windows trên máy này.

Hệ quả: chép `workspace.json` sang laptop mới, SSH Studio thấy danh sách server nhưng không giải mã được key và mật khẩu, vì Credential Manager của máy mới không có khoá đó. Đó là thiết kế đúng: ai lấy được file trên đĩa cũng không có key của bạn. Muốn mang sang máy khác thì dùng **Sao lưu**.

## 2. Sao lưu

Trang **SSH Keys** → **Sao lưu**:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-sao-luu-khoi-phuc-ssh-studio/ssh-studio-01.webp" alt="Hộp thoại Sao lưu workspace của SSH Studio: ghi chú mã hoá AES-256-GCM bằng khoá dẫn xuất từ mật khẩu Argon2id, hai ô Mật khẩu sao lưu và Nhập lại mật khẩu đã điền (hiện dấu chấm), nút Chọn nơi lưu" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <b>Sao lưu workspace</b>: đặt mật khẩu riêng cho file</figcaption></figure>

- **Mật khẩu sao lưu** (ít nhất 8 ký tự) chỉ dùng cho file này. Từ mật khẩu, SSH Studio dẫn xuất khoá bằng **Argon2id** (chậm có chủ đích, để dò mật khẩu tốn kém), rồi mã hoá toàn bộ nội dung bằng **AES-256-GCM** (mã hoá và chống sửa: sửa một byte là file không mở được).
- **Quên mật khẩu này là mất bản sao lưu.** Không có cửa sau. Cất nó trong trình quản lý mật khẩu, không ghi cạnh file.

**Chọn nơi lưu…** mở hộp thoại lưu file của Windows, tên mặc định `ssh-studio-NĂM-THÁNG-NGÀY.sshstudio-backup`.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-sao-luu-khoi-phuc-ssh-studio/ssh-studio-02.webp" alt="SSH Studio sau khi sao lưu: trang SSH Keys với nút Khôi phục, Sao lưu, Nhập key, Tạo key mới, thông báo Đã sao lưu 3 server, 1 key, 3 known host vào đường dẫn file .sshstudio-backup" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · đã sao lưu</figcaption></figure>

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-sao-luu-khoi-phuc-ssh-studio/ssh-studio-06.webp" alt="Hộp thoại Nhật ký hoạt động của SSH Studio: dòng cuối Đã sao lưu 3 server, 1 key, 3 known host vào thư mục sao-luu, phía trên là các lần kết nối toinoi-bastion và toinoi-app-02" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · nhật ký ghi lại lần sao lưu và nơi lưu file</figcaption></figure>

Bên trong file chỉ có một phong bì JSON: định dạng, phiên bản, tham số Argon2id, `nonce` và khối `data` đã mã hoá (xem phần thứ hai của khung ở mục 1). Không có tên server hay key nào đọc được.

| Có trong bản sao lưu | Không có |
|---|---|
| Server (kể cả jump host, port forward, thư mục), workspace | Cài đặt giao diện của SSH Studio |
| Private key (giải mã rồi mã hoá lại bằng mật khẩu sao lưu) | Mã OTP (không bao giờ được lưu) |
| Mật khẩu đăng nhập đã chọn «lưu», mật khẩu proxy | Lịch sử Launchpad, nhật ký hoạt động |
| Known hosts | |

## 3. «Máy mới»: thiếu một server

Để mô phỏng, An xoá `toinoi-app-02` khỏi SSH Studio và quên host key của nó:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-sao-luu-khoi-phuc-ssh-studio/ssh-studio-03.webp" alt="Trang Servers của SSH Studio không còn toinoi-app-02: thư mục Hạ tầng có toinoi-bastion, Production có toinoi-app-01, không còn phiên nào mở" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · «máy mới» chưa có <code>toinoi-app-02</code></figcaption></figure>

## 4. Khôi phục: xem trước, rồi gộp

**SSH Keys → Khôi phục**, chọn file, nhập mật khẩu sao lưu, **Xem trước**:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-sao-luu-khoi-phuc-ssh-studio/ssh-studio-04.webp" alt="Hộp thoại Khôi phục từ bản sao lưu của SSH Studio, bước xem trước: thời điểm tạo bản sao lưu, bảng Server 1 mới trên 3, SSH key 0 mới trên 1, Known hosts 1 mới trên 3, ghi chú server và key đã có được giữ nguyên, nút Quay lại và Khôi phục" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · xem trước: chỉ thêm những gì còn thiếu</figcaption></figure>

Khôi phục của SSH Studio luôn là **gộp** vào workspace hiện tại, không xoá gì:

- Server đã có (cùng id) và key đã có (cùng fingerprint) được giữ nguyên.
- Thư mục còn thiếu được thêm; workspace đang dùng không đổi.
- Known host trong bản sao lưu mà **khác** fingerprint đang tin cậy (server đã đổi host key từ lúc sao lưu): SSH Studio cảnh báo và để bạn chọn từng địa chỉ có thay hay không. Mặc định là giữ fingerprint hiện tại. Chỉ chọn thay khi đã đối chiếu như [LNX-023](/articles/linux-host-key-known-hosts).

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-sao-luu-khoi-phuc-ssh-studio/ssh-studio-05.webp" alt="Trang Servers của SSH Studio sau khi khôi phục: toinoi-app-02 trở lại ở gốc workspace với xác thực SSH key · Jump, thông báo Đã khôi phục 1 server mới, 0 key mới, 1 known host mới, bỏ qua 3 mục đã có" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>toinoi-app-02</code> trở lại, kể cả jump host và host key</figcaption></figure>

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-sao-luu-khoi-phuc-ssh-studio/ssh-studio-07.webp" alt="Trang Known Hosts của SSH Studio sau khi khôi phục: lại đủ ba mục [127.0.0.1]:2200, [127.0.0.1]:2222 và [toinoi-app-02]:22 với fingerprint như trước" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · khôi phục trả lại cả fingerprint của toinoi-app-02</figcaption></figure>

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-sao-luu-khoi-phuc-ssh-studio/ssh-studio-08.webp" alt="Launchpad của SSH Studio sau khi khôi phục, tab Phiên đang mở có toinoi-app-01 đã kết nối, các tab Gần đây và Tất cả server cho 3 server" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · Launchpad: workspace đã đủ ba server, sẵn sàng làm việc tiếp</figcaption></figure>

`toinoi-app-02` quay lại đúng chỗ cũ (gốc workspace: ở [LNX-052](/articles/linux-bastion-jump-host) An chưa chuyển nó vào thư mục Production), vẫn đi qua `toinoi-bastion`, và host key của nó đã được tin: kết nối không hỏi lại fingerprint. Key `an-laptop` và hai server kia đã có nên được bỏ qua.

## 5. Quy trình đổi máy

1. Máy cũ: **Sao lưu**, mật khẩu mạnh cất trong trình quản lý mật khẩu.
2. Chép file `.sshstudio-backup` qua USB hay ổ chia sẻ. File đã mã hoá, nhưng vẫn đừng gửi nó qua kênh công khai.
3. Máy mới: cài SSH Studio, **Khôi phục**, xem trước, khôi phục.
4. Kết nối thử từng server. Nhớ: mã OTP và các mật khẩu không chọn lưu thì nhập lại như thường.
5. Xoá file sao lưu ở chỗ trung gian; nếu nghi lộ, đổi key ([LNX-022](/articles/linux-ssh-key-ssh-studio)).

## 6. Lỗi thường gặp

**Chép `workspace.json` rồi thắc mắc vì sao key không dùng được.** Khoá giải mã ở Credential Manager của máy cũ.

**Mật khẩu sao lưu yếu, file để trên ổ chia sẻ chung.** Argon2id làm chậm việc dò, nhưng không cứu được mật khẩu `12345678`.

**Không bao giờ thử khôi phục.** Bản sao lưu chưa từng khôi phục thử là bản sao lưu chưa chắc dùng được.

**Chọn «thay fingerprint» cho mọi xung đột.** Bản sao lưu cũ có thể chứa host key cũ; host key hiện tại có thể là cái đúng.

## Bài tập

1. Sửa một ký tự trong khối `data` của file sao lưu (bản sao), thử khôi phục. SSH Studio báo gì? Vì sao AES-256-GCM phát hiện được?
2. Tạo workspace «Khách hàng A» có một server, sao lưu, xoá workspace đó, khôi phục. Workspace có được tạo lại không?
3. Trên Windows, mở Credential Manager → Windows Credentials và tìm mục của SSH Studio (chỉ xem, đừng xoá). Nếu mục đó mất, chẳng hạn khi cài lại Windows, thì bí mật trong `workspace.json` còn mở được không? Bản sao lưu còn dùng được không?
4. Viết vào runbook của nhóm ([LNX-060](/articles/linux-thuc-hanh-runbook-van-hanh)) một mục «Đổi máy» dựa trên mục 5.

## Đọc tiếp

[LNX-054 · Cập nhật hệ thống an toàn: bản vá, khởi động lại, unattended-upgrades](/articles/linux-cap-nhat-he-thong-an-toan).
