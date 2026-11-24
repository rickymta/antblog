---
slug: linux-la-gi-va-ssh-studio
title: "Linux là gì và SSH Studio giúp gì: lần đầu nhìn vào server của Tới Nơi"
summary: "Mở đầu series Linux qua SSH Studio: Linux gồm kernel, bản phân phối và shell như thế nào, server khác máy cá nhân ở đâu, vì sao phải làm việc qua SSH, và hai cách thao tác trong SSH Studio (terminal và giao diện). Bài đọc năm lệnh đầu tiên trên server toinoi-app-01 chạy Ubuntu 24.04."
category: devops-linux
level: beginner
tags: [linux, lnx-001, ubuntu, ssh, ssh-studio, kernel, shell, toi-noi]
language: vi
---

Tuần đầu ở nhóm backend của Tới Nơi, An nhận một tin nhắn từ Bình bên nhóm hạ tầng: «Tài khoản của bạn trên `toinoi-app-01` đã tạo xong. Dịch vụ báo giá nằm ở `/opt/toinoi/bao-gia`, log ở `/var/log/toinoi`. Bạn vào xem giúp vì sao sáng nay có vài yêu cầu báo giá bị lỗi.»

An đã viết code cho dịch vụ `bao-gia` mấy tháng, nhưng mới chỉ chạy dịch vụ trên máy mình. `toinoi-app-01` là một máy không có màn hình, không có chuột, đặt ở nơi An chưa từng thấy. Muốn làm được việc Bình nhờ, An phải trả lời được ba câu hỏi: máy đó chạy gì, vào bằng cách nào, và vào rồi thì làm việc ra sao khi chỉ có một cửa sổ gõ lệnh.

Series này đi cùng An từ câu hỏi đó tới lúc tự vận hành được vài server. Bài đầu tiên chưa yêu cầu bạn cài gì: bài giải thích các khái niệm nền và cho bạn xem trước những gì sẽ làm từ [bài sau](/articles/linux-dung-lab-docker-ket-noi-ssh-studio).

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-001 · Phần 1/6 · Cơ bản</span>
<p>Không cần biết Linux. Sau bài này, bạn phân biệt được kernel, bản phân phối và shell; biết vì sao server được điều khiển qua SSH; đọc được kết quả của năm lệnh đầu tiên; và biết SSH Studio cho bạn hai cách làm cùng một việc. Lab của series: <code>examples/toi-noi-linux</code> (dựng ở LNX-002).</p>
</div>

## 1. Linux gồm những gì?

Khi nói «server chạy Linux», người ta đang gộp ba lớp khác nhau vào một chữ.

**Kernel** (nhân) là chương trình nằm giữa phần cứng và mọi chương trình khác. Kernel chia CPU cho các tiến trình, cấp bộ nhớ, đọc ghi đĩa, gửi nhận gói tin mạng và quyết định ai được đụng vào file nào. Tên «Linux», nói cho chặt, là tên của kernel này. Không ai làm việc trực tiếp với kernel; chương trình gọi kernel qua các lời gọi hệ thống (system call).

**Bản phân phối** (distribution, hay distro) là kernel cộng với mọi thứ cần có để thành một hệ điều hành dùng được: các lệnh cơ bản (`ls`, `cp`, `grep`), trình quản lý gói để cài phần mềm, hệ thống khởi động dịch vụ, cấu hình mặc định. Ubuntu, Debian, Red Hat Enterprise Linux, Rocky Linux, Alpine là các bản phân phối. Hai bản phân phối có thể dùng cùng một kernel nhưng khác lệnh cài phần mềm (`apt` trên Ubuntu và Debian, `dnf` trên Red Hat và Rocky) và khác vị trí vài file cấu hình. Series này dùng **Ubuntu 24.04 LTS**; phần lớn nội dung đúng với mọi bản phân phối, chỗ nào riêng của Ubuntu bài sẽ nói rõ.

**Shell** là chương trình đọc dòng lệnh bạn gõ rồi chạy các chương trình khác theo dòng lệnh đó. Shell mặc định của Ubuntu là **bash**. Cửa sổ mà An nhìn thấy sau khi đăng nhập chính là bash đang chờ lệnh.

| Lớp | Trên `toinoi-app-01` | Bạn gặp lớp này khi nào |
|---|---|---|
| Kernel | Linux 6.x | Xem phiên bản bằng `uname`, đọc lỗi về bộ nhớ, đĩa, mạng |
| Bản phân phối | Ubuntu 24.04 LTS | Cài phần mềm bằng `apt`, tìm file cấu hình, đọc tài liệu |
| Shell | bash 5.2 | Mỗi lần gõ một lệnh |

«LTS» là viết tắt của *long-term support*: bản được vá lỗi bảo mật trong nhiều năm. Server gần như luôn chạy bản LTS, vì việc nâng cấp hệ điều hành của một máy đang phục vụ khách hàng vừa tốn công vừa rủi ro.

## 2. Server khác máy cá nhân ở đâu?

Một server Linux không khác máy tính của bạn về bản chất: cũng CPU, RAM, đĩa, card mạng. Khác biệt nằm ở cách dùng.

| | Máy cá nhân | Server |
|---|---|---|
| Giao diện | Màn hình, cửa sổ, chuột | Thường chỉ có dòng lệnh; không cài giao diện đồ hoạ |
| Người dùng | Một người ngồi trước máy | Nhiều người và nhiều dịch vụ đăng nhập từ xa |
| Thời gian chạy | Tắt mở hằng ngày | Chạy liên tục nhiều tháng |
| Hậu quả khi gõ nhầm | Hỏng việc của mình | Dừng dịch vụ của cả công ty |

Server không cài giao diện đồ hoạ vì ba lý do: giao diện tốn tài nguyên mà dịch vụ không dùng tới, nhiều phần mềm hơn nghĩa là nhiều lỗ hổng hơn, và dòng lệnh thì ghi lại, lặp lại, tự động hoá được. Một thao tác bấm chuột không chép vào tài liệu vận hành được; một dòng lệnh thì được.

Điều này quyết định cách học: bạn phải thoải mái với dòng lệnh. Nhưng thoải mái với dòng lệnh không có nghĩa là từ chối mọi giao diện. Xem một cây thư mục, kéo một file từ máy mình lên server, sửa nhanh một file cấu hình là những việc mà giao diện làm nhanh và ít nhầm hơn. Series này dùng cả hai, và luôn chỉ ra giao diện đang làm việc gì ở bên dưới.

## 3. SSH: cách vào một máy không có màn hình

**SSH** (Secure Shell) là giao thức cho phép bạn mở một phiên shell trên máy ở xa qua mạng, với toàn bộ dữ liệu được mã hoá. Trên server có một chương trình luôn chạy nền, tên là `sshd`, nghe ở cổng 22. Trên máy bạn có một chương trình SSH client. Client kết nối tới `sshd`, hai bên thống nhất khoá mã hoá, bạn chứng minh mình là ai (bằng mật khẩu hoặc SSH key), và `sshd` khởi động một shell cho bạn. Từ đó, mỗi phím bạn gõ đi qua kênh mã hoá tới shell trên server, và mỗi dòng shell in ra đi ngược về màn hình của bạn.

Trên cùng kết nối SSH đó còn chạy được **SFTP**, giao thức truyền file: liệt kê thư mục, tải file lên, tải file về, đổi tên, đổi quyền. SFTP không cần cài thêm gì trên server; có `sshd` là có SFTP.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 720 230" role="img" aria-label="Sơ đồ: SSH Studio trên máy của An kết nối qua kênh SSH mã hoá tới sshd trên server toinoi-app-01; trên kênh đó chạy phiên shell bash và SFTP">
<rect x="20" y="40" width="210" height="150" rx="8" class="f-pk-soft s-pk"/>
<text x="125" y="66" text-anchor="middle" class="t-b">Máy của An</text>
<rect x="40" y="82" width="170" height="40" rx="6" class="f-surface s-line"/>
<text x="125" y="107" text-anchor="middle">Terminal (xterm)</text>
<rect x="40" y="132" width="170" height="40" rx="6" class="f-surface s-line"/>
<text x="125" y="157" text-anchor="middle">Files (SFTP)</text>
<text x="125" y="212" text-anchor="middle" class="t-small">SSH Studio</text>
<line x1="230" y1="102" x2="490" y2="102" class="s-phys" stroke-width="2"/>
<line x1="230" y1="152" x2="490" y2="152" class="s-phys" stroke-width="2"/>
<text x="360" y="92" text-anchor="middle" class="t-small">phím gõ ⇄ kết quả</text>
<text x="360" y="142" text-anchor="middle" class="t-small">liệt kê, tải lên, tải về</text>
<text x="360" y="182" text-anchor="middle" class="t-b">một kết nối SSH, mã hoá</text>
<rect x="490" y="40" width="210" height="150" rx="8" class="f-pk-soft s-pk"/>
<text x="595" y="66" text-anchor="middle" class="t-b">toinoi-app-01</text>
<rect x="510" y="82" width="170" height="40" rx="6" class="f-surface s-line"/>
<text x="595" y="107" text-anchor="middle">sshd → bash</text>
<rect x="510" y="132" width="170" height="40" rx="6" class="f-surface s-line"/>
<text x="595" y="157" text-anchor="middle">sshd → sftp-server</text>
<text x="595" y="212" text-anchor="middle" class="t-small">Ubuntu 24.04, cổng 22</text>
</svg>
</div>
<figcaption>Terminal và khung Files của SSH Studio là hai kênh trên cùng một kết nối SSH tới server.</figcaption>
</figure>

## 4. SSH Studio trong series này

**SSH Studio** là SSH client dạng ứng dụng desktop (Windows, macOS, Linux) dùng xuyên suốt series. Một phiên kết nối trong SSH Studio có năm chế độ xem:

| Chế độ | Bạn thấy gì | Dùng khi |
|---|---|---|
| **Terminal** | Phiên shell trên server | Gõ lệnh: đây là cách làm việc chính |
| **Files** | Hai khung: máy của bạn và server (SFTP) | Duyệt thư mục, tải file lên xuống, sửa file, đổi quyền |
| **Chia đôi** | Terminal ở trên, Files ở dưới | Gõ lệnh và nhìn file thay đổi |
| **Song song** | Terminal bên trái, Files bên phải | Như trên, cho màn hình rộng |
| **Ports** | Danh sách port forwarding | Mở dịch vụ nội bộ của server trên máy mình (Phần 5) |

Mỗi thao tác trong series được làm theo hai cách khi có thể:

1. **Bằng lệnh trong terminal.** Bài đưa khối lệnh kèm kết quả thật chép từ lab.
2. **Trên giao diện của SSH Studio**, kèm ảnh chụp. Bài nói rõ nút đó tương ứng lệnh nào.

Cách thứ nhất là thứ bạn mang theo được sang mọi server và mọi công cụ. Cách thứ hai giúp bạn nhìn thấy kết quả và làm nhanh những việc lặt vặt. Khi SSH Studio không có giao diện cho một việc (quản lý tiến trình, cài phần mềm, sửa cấu hình dịch vụ), bài chỉ dùng lệnh.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-la-gi-va-ssh-studio/ssh-studio-05.webp" alt="Trang Servers của SSH Studio, workspace Cá nhân: một server toinoi-app-01 an@127.0.0.1:2222, nhóm Lab, xác thực Password, trạng thái 1 phiên hoạt động, các nút Kết nối tất cả và Thêm server" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · trang <b>Servers</b>: danh sách server của workspace và trạng thái kết nối</figcaption></figure>

## 5. Năm lệnh đầu tiên trên server

Đây là những gì An gõ ngay sau khi đăng nhập. Bạn chưa cần làm theo; hãy đọc kết quả để quen mặt chữ. Dòng bắt đầu bằng `an@toinoi-app-01:~$` là **dấu nhắc** (prompt) của shell kèm lệnh An gõ; các dòng bên dưới là kết quả.

### Đây là hệ điều hành gì?

```text
an@toinoi-app-01:~$ cat /etc/os-release
PRETTY_NAME="Ubuntu 24.04.5 LTS"
NAME="Ubuntu"
VERSION_ID="24.04"
VERSION="24.04.5 LTS (Noble Numbat)"
VERSION_CODENAME=noble
ID=ubuntu
ID_LIKE=debian
HOME_URL="https://www.ubuntu.com/"
SUPPORT_URL="https://help.ubuntu.com/"
BUG_REPORT_URL="https://bugs.launchpad.net/ubuntu/"
PRIVACY_POLICY_URL="https://www.ubuntu.com/legal/terms-and-policies/privacy-policy"
UBUNTU_CODENAME=noble
LOGO=ubuntu-logo
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-la-gi-va-ssh-studio/ssh-studio-03.webp" alt="Terminal của SSH Studio: cat /etc/os-release in PRETTY_NAME Ubuntu 24.04.5 LTS, VERSION_ID 24.04, VERSION_CODENAME noble, ID ubuntu, ID_LIKE debian" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>cat /etc/os-release</code>: server chạy Ubuntu 24.04 LTS</figcaption></figure>

`cat` in nội dung một file ra màn hình. File `/etc/os-release` có trên mọi bản phân phối hiện đại và là nơi đáng tin nhất để biết máy đang chạy gì. Ba dòng cần đọc: `PRETTY_NAME` (tên đầy đủ), `VERSION_ID` (số phiên bản, dùng khi tra tài liệu) và `ID_LIKE=debian` (Ubuntu thuộc họ Debian, nên cài phần mềm bằng `apt` và dùng gói `.deb`).

### Kernel nào?

```text
an@toinoi-app-01:~$ uname -s
Linux
an@toinoi-app-01:~$ uname -srm
Linux 6.18.58.1-microsoft-standard-WSL2 x86_64
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-la-gi-va-ssh-studio/ssh-studio-04.webp" alt="Terminal của SSH Studio: uname -s in Linux, uname -srm in Linux 6.18.58.1-microsoft-standard-WSL2 x86_64" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>uname</code>: tên kernel, phiên bản và kiến trúc CPU</figcaption></figure>

`uname` in thông tin về kernel. `-s` là tên kernel, `-r` là phiên bản, `-m` là kiến trúc CPU. `x86_64` là CPU Intel/AMD 64 bit; trên máy ARM (nhiều server đám mây đời mới, máy Mac dùng chip Apple) bạn sẽ thấy `aarch64`.

Phiên bản kernel trong kết quả trên có đuôi `microsoft-standard-WSL2`, vì server lab của series là một container Docker chạy trên Windows: container dùng chung kernel với máy chạy nó. Trên một server Ubuntu 24.04 thật, bạn sẽ thấy dạng `6.8.0-xx-generic`. Đây là một trong số ít chỗ mà lab khác server thật, và bài nào có kết quả phụ thuộc kernel sẽ nhắc lại.

### Tôi là ai, đang ở máy nào?

```text
an@toinoi-app-01:~$ whoami
an
an@toinoi-app-01:~$ hostname
toinoi-app-01
```

Hai lệnh này có vẻ thừa, cho tới ngày bạn mở năm cửa sổ tới năm server và sắp gõ một lệnh xoá. `whoami` in tên người dùng mà shell đang chạy dưới danh nghĩa đó; `hostname` in tên máy. Dấu nhắc `an@toinoi-app-01` cũng mang hai thông tin này, và tập thói quen liếc dấu nhắc trước khi nhấn Enter là kỹ năng vận hành đầu tiên nên có.

### Tiến trình số 1

```text
an@toinoi-app-01:~$ ps -p 1 -o pid,comm
    PID COMMAND
      1 systemd
```

Mỗi chương trình đang chạy là một **tiến trình** (process), có một số định danh là PID. Tiến trình có PID 1 là tiến trình đầu tiên kernel khởi động, và nó khởi động mọi thứ còn lại. Trên Ubuntu, tiến trình đó là **systemd**: systemd bật `sshd` để bạn đăng nhập được, bật các dịch vụ khác, và khởi động lại chúng khi chúng chết. Phần 4 của series dành cho systemd.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-la-gi-va-ssh-studio/ssh-studio-01.webp" alt="Terminal của SSH Studio trên toinoi-app-01: whoami in an, hostname in toinoi-app-01, ps -p 1 -o pid,comm cho thấy tiến trình số 1 là systemd" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>whoami</code>, <code>hostname</code> và tiến trình số 1 của server</figcaption></figure>

Trong ảnh, thanh trên cùng của phiên có năm nút chế độ xem (Terminal, Files, Chia đôi, Song song, Ports), địa chỉ `an@127.0.0.1:2222` và trạng thái «Đã kết nối». Cột bên trái liệt kê workspace, danh sách server và các phiên đang mở.

## 6. Cùng một thư mục, hai cách nhìn

Bình nói dịch vụ báo giá nằm ở `/opt/toinoi/bao-gia`. An xem thư mục đó bằng lệnh `ls -l`, rồi chuyển sang chế độ **Song song** và gõ cùng đường dẫn vào khung **Remote server**:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-la-gi-va-ssh-studio/ssh-studio-02.webp" alt="SSH Studio ở chế độ Song song: terminal bên trái in kết quả ls -l /opt/toinoi/bao-gia, khung Files bên phải hiển thị cùng bốn file của thư mục đó" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · chế độ Song song · <code>ls -l</code> và khung Remote server cùng nhìn vào <code>/opt/toinoi/bao-gia</code></figcaption></figure>

Cả hai bên đều thấy bốn file: `appsettings.json`, `bang-gia.csv`, `bi-mat.env`, `kiem-tra-cau-hinh.sh`. Điểm khác nhau đáng chú ý:

- Khung Files cho bạn tên, kích thước và ngày sửa. `ls -l` cho thêm **quyền** (`-rw-r--r--`), **chủ sở hữu** (`baogia`) và **nhóm** (`toinoi`). Đây là thông tin quyết định An đọc được file nào, và là nội dung của [LNX-007](/articles/linux-nguoi-dung-nhom-quyen-file).
- Khung bên trái của Files, «Máy của bạn», là một thư mục trên máy An. Kéo file giữa hai khung là tải lên hoặc tải về ([LNX-005](/articles/linux-tao-sao-chep-di-chuyen-xoa-file)).
- Khung Files đi qua SFTP với đúng tài khoản `an`. File nào `an` không được đọc bằng lệnh thì cũng không mở được ở khung Files. Giao diện không cho bạn thêm quyền nào.

## 7. Lộ trình của Phần 1

| Bài | Bạn làm được gì |
|---|---|
| [LNX-002](/articles/linux-dung-lab-docker-ket-noi-ssh-studio) | Dựng server lab bằng Docker, kết nối bằng SSH Studio, đối chiếu fingerprint |
| [LNX-003](/articles/linux-shell-dong-lenh-tro-giup) | Đọc dấu nhắc, viết lệnh có tuỳ chọn và đối số, tự tra trợ giúp |
| [LNX-004](/articles/linux-cay-thu-muc-duong-dan) | Đi lại trong cây thư mục, hiểu `/etc`, `/var`, `/opt`, `/srv`, `/home` |
| [LNX-005](/articles/linux-tao-sao-chep-di-chuyen-xoa-file) | Tạo, sao chép, di chuyển, xoá file; tải file lên bằng khung Files |
| [LNX-006](/articles/linux-xem-sua-file-van-ban) | Xem file dài, sửa file bằng `vim` và bằng trình sửa file của SSH Studio |
| [LNX-007](/articles/linux-nguoi-dung-nhom-quyen-file) | Đọc và đặt quyền, dùng `sudo` |
| [LNX-008](/articles/linux-chuyen-huong-pipe) | Nối lệnh bằng pipe, ghi kết quả ra file |
| [LNX-009](/articles/linux-tim-kiem-find-grep) | Tìm file và tìm trong nội dung file |
| [LNX-010](/articles/linux-thuc-hanh-thu-muc-trien-khai) | Thực hành: chuẩn bị thư mục triển khai cho `bao-gia`, có bộ chấm tự động |

Các phần sau lần lượt là shell script (Phần 2), người dùng và SSH an toàn (Phần 3), tiến trình và dịch vụ (Phần 4), mạng, lưu trữ và giám sát (Phần 5), vận hành nhiều server và xử lý sự cố (Phần 6).

## 8. Hiểu lầm thường gặp

**«Linux là Ubuntu.»** Ubuntu là một bản phân phối. Lệnh cơ bản giống nhau giữa các bản phân phối, nhưng lệnh cài phần mềm và vài đường dẫn cấu hình thì khác. Khi tra tài liệu, luôn kiểm tra tài liệu viết cho bản phân phối nào.

**«Có giao diện rồi thì không cần học lệnh.»** Giao diện chỉ phủ một phần nhỏ công việc. Tài liệu vận hành, script triển khai, câu trả lời trên diễn đàn đều viết bằng lệnh. Giao diện là công cụ tăng tốc cho người đã hiểu lệnh.

**«Dùng tài khoản root cho tiện.»** `root` là tài khoản có mọi quyền; một lệnh gõ nhầm dưới quyền `root` không có gì chặn lại. Trong series, An đăng nhập bằng tài khoản thường và chỉ mượn quyền `root` cho từng lệnh bằng `sudo`. Server lab còn cấm hẳn việc đăng nhập bằng `root` qua SSH.

**«Lab chạy trong Docker thì không giống server thật.»** Lab của series chạy systemd và `sshd` như một máy Ubuntu thật; với nội dung của Phần 1, bạn sẽ không thấy khác biệt nào ngoài phiên bản kernel. Chỗ nào lab khác server thật, bài sẽ nói.

## Bài tập

1. Không nhìn lại bài, hãy viết một câu cho mỗi khái niệm: kernel, bản phân phối, shell, SSH, SFTP.
2. Trong kết quả `cat /etc/os-release` ở mục 5, dòng nào cho bạn biết nên tìm tài liệu cài phần mềm của họ Debian?
3. Một đồng nghiệp gửi bạn kết quả `uname -srm` là `Linux 6.8.0-48-generic aarch64`. Máy đó dùng CPU loại gì? Một chương trình biên dịch cho `x86_64` có chạy được trên máy đó không?
4. Vì sao khung Files của SSH Studio không mở được file mà lệnh `cat` báo `Permission denied`?

## Đọc tiếp

[LNX-002 · Dựng server Linux bằng Docker và kết nối bằng SSH Studio](/articles/linux-dung-lab-docker-ket-noi-ssh-studio): bạn dựng `toinoi-app-01` trên máy mình và đăng nhập lần đầu.
