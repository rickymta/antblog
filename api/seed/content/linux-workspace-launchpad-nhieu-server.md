---
slug: linux-workspace-launchpad-nhieu-server
title: "Workspace, thư mục và Launchpad: quản lý nhiều server cùng lúc trong SSH Studio"
summary: "Khi Tới Nơi có thêm server: thêm toinoi-bastion vào SSH Studio, gom server theo nhóm thành thư mục, Kết nối tất cả một lần, đối chiếu fingerprint của server mới, theo dõi mọi phiên trên Launchpad, và trên server xem ai đang đăng nhập bằng who, w, last."
category: devops-linux
level: beginner
tags: [linux, lnx-051, ssh-studio, workspace, launchpad, who, last, toi-noi]
language: vi
---

Năm phần đầu, An chỉ làm việc trên một server. Tuần này Bình bàn giao thêm hai máy: **toinoi-bastion**, cửa vào duy nhất của mạng nội bộ, và **toinoi-app-02**, server dự phòng chỉ nằm trong mạng nội bộ. Ba server, rồi sẽ thêm staging, thêm máy cơ sở dữ liệu: danh sách phẳng sẽ sớm rối. Việc đầu tiên của Phần 6 là sắp xếp chỗ làm việc cho gọn. (Hồ sơ thử `toinoi-app-01 (cuong)` của Phần 3 đã được xoá khỏi SSH Studio: nó không phải một server của Tới Nơi.)

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-051 · Phần 6/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/linux-dung-lab-docker-ket-noi-ssh-studio">LNX-002</a> (thêm server, fingerprint), <a href="/articles/linux-ssh-key-ssh-studio">LNX-022</a> (key <code>an-laptop</code>). Trạng thái đầu bài: <code>npm run setup -- 51</code>. Từ Phần 6, lab có ba container: <code>toinoi-app-01</code> (cổng 2222), <code>toinoi-bastion</code> (cổng 2200) và <code>toinoi-app-02</code> (chỉ trong mạng nội bộ). Bastion chỉ nhận key, key <code>an-laptop</code> đã được cài sẵn ở đó.</p>
</div>

## 1. Ba cấp tổ chức của SSH Studio

| Cấp | Là gì | Dùng để |
|---|---|---|
| **Workspace** | Một bộ server riêng (mặc định: «Cá nhân») | Tách hẳn khách hàng, dự án, hay việc riêng với việc công ty |
| **Thư mục** | Nhóm server trong một workspace, lồng tối đa 3 cấp | Production / Staging / Hạ tầng… |
| **Nhóm** | Một trường chữ của từng server | Lọc nhanh; tạo thư mục hàng loạt từ nhóm |

Một server thuộc đúng một workspace và nằm ở gốc hoặc trong một thư mục. Bài này dùng một workspace và các thư mục bên trong nó.

## 2. Thêm toinoi-bastion

**Servers → Thêm server**, như [LNX-002](/articles/linux-dung-lab-docker-ket-noi-ssh-studio), lần này với key:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-workspace-launchpad-nhieu-server/ssh-studio-01.webp" alt="Hộp thoại Thêm server của SSH Studio: Tên hiển thị toinoi-bastion, Địa chỉ server 127.0.0.1, Port 2200, Username an, Nhóm Hạ tầng, Phương thức xác thực SSH Key với key an-laptop" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · thêm <code>toinoi-bastion</code>, xác thực bằng key <code>an-laptop</code></figcaption></figure>

Trước đó, An sửa `toinoi-app-01` cho trường **Nhóm** là `Production` (từ Phần 1 nó ở nhóm `Lab`). Nhóm đặt đúng từ đầu thì bước sau làm một lần là xong.

## 3. Thư mục từ nhóm

Khi workspace chưa có thư mục nào, cây **THƯ MỤC** ở cột trái gợi ý **«Tạo 2 thư mục từ nhóm server»**. Bấm vào, xác nhận: SSH Studio tạo một thư mục cho mỗi nhóm và chuyển server vào đó. Tên nhóm của từng server vẫn giữ nguyên.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-workspace-launchpad-nhieu-server/ssh-studio-02.webp" alt="Trang Servers của SSH Studio sau khi tạo thư mục từ nhóm: cây thư mục bên trái có Hạ tầng và Production; danh sách theo thư mục với toinoi-bastion trong Hạ tầng, toinoi-app-01 trong Production" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · thư mục <b>Hạ tầng</b> và <b>Production</b></figcaption></figure>

Các cách khác để sắp xếp:

- Kéo một server thả vào thư mục ở cây bên trái hay trong danh sách.
- Menu của dòng server: **Chuyển tới thư mục**, hoặc **Thư mục mới chứa server này…**.
- Trường **Thư mục** trong hộp thoại sửa server. Server mới tạo khi đang xem một thư mục thì nằm sẵn trong thư mục đó.
- Xoá một thư mục không xoá server nào: server được chuyển lên gốc workspace.

## 4. Kết nối tất cả, và fingerprint của server mới

Nút **Kết nối tất cả** trên trang Servers (hay trên dòng của một thư mục) mở phiên cho mọi server trong danh sách **chưa có phiên**. `toinoi-app-01` đang mở sẵn, nên chỉ `toinoi-bastion` được kết nối, và vì là lần đầu, SSH Studio hỏi:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-workspace-launchpad-nhieu-server/ssh-studio-03.webp" alt="Hộp thoại Xác nhận danh tính server của SSH Studio cho toinoi-bastion tại 127.0.0.1:2200, fingerprint ED25519 SHA256:raPJwwASajL8O9BfMyJrQ/ADRm/KSFNfLDQmA4krHUA, nút Huỷ kết nối và Tin cậy và kết nối" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · lần đầu kết nối <code>toinoi-bastion</code>: đối chiếu fingerprint</figcaption></figure>

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-workspace-launchpad-nhieu-server/ssh-studio-08.webp" alt="Trang Known Hosts của SSH Studio sau khi Kết nối tất cả: hai mục [127.0.0.1]:2200 của toinoi-bastion và [127.0.0.1]:2222 của toinoi-app-01 cùng fingerprint SHA256" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · Known Hosts có thêm fingerprint của toinoi-bastion</figcaption></figure>

Bastion là máy mới, nên quy tắc của [LNX-002](/articles/linux-dung-lab-docker-ket-noi-ssh-studio) vẫn áp dụng: hỏi Bình fingerprint thật (đọc trên console bằng `ssh-keygen -lf /etc/ssh/ssh_host_ed25519_key.pub`) và chỉ tin khi trùng. Bastion là cửa vào của cả mạng nội bộ: tin nhầm ở đây là đưa mọi kết nối sau đó qua một máy lạ.

Với hơn 3 server, SSH Studio hỏi xác nhận trước khi mở nhiều phiên cùng lúc; server cần mật khẩu hay mã OTP lần lượt hỏi từng cái.

## 5. Trên server: ai đang đăng nhập

Trong tab của bastion:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-workspace-launchpad-nhieu-server/ssh-studio-04.webp" alt="Terminal của SSH Studio trên toinoi-bastion: hostname, who cho thấy phiên của an trên pts/0 từ 10.20.0.1, w -h không in gì, last -n 3 an ghi phiên đang mở (still logged in)" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>who</code>, <code>w</code>, <code>last</code> trên bastion</figcaption></figure>

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-workspace-launchpad-nhieu-server/ssh-studio-06.webp" alt="Terminal của SSH Studio trên toinoi-app-01: who cho an trên pts/0 từ 10.20.0.1, last -n 3 an cho phiên đang mở và một phiên tmux cũ" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · trên toinoi-app-01: cùng câu hỏi, câu trả lời của máy khác</figcaption></figure>

| Lệnh | Cho biết |
|---|---|
| `who` | Ai đang đăng nhập, ở terminal nào (`pts/N`), từ đâu |
| `w` | Như `who`, thêm thời gian rảnh và lệnh đang chạy; `-h` bỏ dòng tiêu đề. Bản `w` của Ubuntu 24.04 lấy danh sách phiên từ systemd-logind, nên trong container của lab nó không in gì; trên máy ảo thật thì có |
| `last -n 3 an` | Các lần đăng nhập gần nhất của `an`, kể cả đã thoát (đọc `/var/log/wtmp`) |

Phiên của An đến từ `10.20.0.1`: cổng 2200 trên máy bạn được Docker chuyển vào, nên với server, mọi kết nối từ máy bạn đều đi ra từ gateway của mạng lab. Trên máy ảo thật, đây sẽ là IP công khai của bạn.

`who` là cách nhanh nhất để biết có ai khác đang làm việc trên server trước khi bạn khởi động lại dịch vụ. Nhiều nhóm có quy ước: thấy người khác trong `who` thì nhắn trước khi làm gì ảnh hưởng tới họ.

## 6. Launchpad: mọi phiên ở một chỗ

**Launchpad** không mở kết nối. Nó là bảng theo dõi của mọi workspace: phiên đang mở, server dùng gần đây, port forward, truyền file, và mục **Cần chú ý** (phiên lỗi, kết nối thất bại, truyền file hỏng). Số trên mục Launchpad ở cột trái là số việc cần chú ý; khi không có gì, nó là số phiên đang mở.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-workspace-launchpad-nhieu-server/ssh-studio-05.webp" alt="Launchpad của SSH Studio, tab Phiên đang mở: hai phiên toinoi-app-01 và toinoi-bastion với workspace, thời gian kết nối và nút Mở, Đóng phiên" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · Launchpad: hai phiên đang mở</figcaption></figure>

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-workspace-launchpad-nhieu-server/ssh-studio-07.webp" alt="Hộp thoại Nhật ký hoạt động của SSH Studio: các dòng Đã lưu server, Đã kết nối an@127.0.0.1 của toinoi-app-01 và toinoi-bastion, cùng các lần tải file và đồng bộ trước đó" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · nhật ký hoạt động ghi lại việc thêm server và kết nối của cả hai máy</figcaption></figure>

Phím tắt: `1`…`6` đổi tab, `J`/`K` chọn dòng, `Enter` mở phiên, `/` tìm. Mở Launchpad từ bất cứ đâu bằng **Ctrl+Shift+D**.

## 7. Lỗi thường gặp

**Bấm «Tin cậy và kết nối» theo phản xạ.** Mỗi server mới là một fingerprint mới cần đối chiếu.

**Đặt nhóm lộn xộn rồi tạo thư mục.** Mỗi biến thể chữ (`Prod`, `Production`, `production`) thành một thư mục. Sửa nhóm trước.

**Dùng thư mục để tách khách hàng.** Thư mục vẫn chung một workspace và chung danh sách tìm kiếm. Khách hàng khác nhau nên ở workspace khác nhau.

**Tưởng Launchpad là nơi kết nối hàng loạt.** «Kết nối tất cả» nằm ở trang Servers và trên dòng thư mục.

## Bài tập

1. Tạo workspace thứ hai «Khách hàng A» (menu workspace ở góc trên cột trái → **Tạo workspace mới**), thêm một server thử vào đó. Chuyển nó qua lại bằng **Chuyển sang workspace**. Cột trái thay đổi thế nào khi đổi workspace?
2. Trong thư mục Production, tạo thư mục con «Ứng dụng» và kéo `toinoi-app-01` vào. Lồng thêm được mấy cấp nữa?
3. Mở hai tab tới `toinoi-app-01` (bấm Kết nối lần nữa). `who` liệt kê mấy dòng? `w` cho biết mỗi phiên đang chạy gì?
4. Đóng phiên bastion, xem tab **Gần đây** và **Cần chú ý** của Launchpad. Phiên đã đóng xuất hiện ở đâu?

## Đọc tiếp

[LNX-052 · Bastion và jump host: vào server nội bộ qua một bước nhảy](/articles/linux-bastion-jump-host).
