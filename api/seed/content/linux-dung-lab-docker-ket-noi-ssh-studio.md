---
slug: linux-dung-lab-docker-ket-noi-ssh-studio
title: "Dựng server Linux bằng Docker và kết nối bằng SSH Studio"
summary: "Dựng server lab toinoi-app-01 (Ubuntu 24.04, systemd, sshd) bằng Docker Compose, thêm server vào SSH Studio, đối chiếu fingerprint host key trước khi tin cậy, đăng nhập bằng mật khẩu và chạy những lệnh đầu tiên. Bài giải thích từng dòng cấu hình của lab và vì sao bước xác nhận fingerprint không phải thủ tục."
category: devops-linux
level: beginner
tags: [linux, lnx-002, docker, ssh, ssh-studio, host-key, fingerprint, known-hosts, toi-noi]
language: vi
---

Lần đầu kết nối tới một server, SSH Studio không hỏi mật khẩu ngay. Nó hiện một hộp thoại có chuỗi `SHA256:…` và nút «Tin cậy và kết nối». Phần lớn người mới bấm nút đó trong nửa giây, vì nó trông giống mọi hộp thoại xác nhận khác.

Hộp thoại đó đang hỏi một câu nghiêm túc: *máy ở đầu bên kia có đúng là server của bạn không?* Nếu không phải, mật khẩu bạn gõ ở bước sau sẽ tới tay người khác. Bài này dựng server lab trên máy bạn, rồi kết nối theo đúng cách nên làm trên server thật: xem fingerprint ở phía server trước, đối chiếu, rồi mới tin cậy.

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-002 · Phần 1/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/linux-la-gi-va-ssh-studio">LNX-001</a>. Cần Docker Desktop, Node.js 22 và SSH Studio 0.3.0. Lab: <code>examples/toi-noi-linux</code>. Sau bài này, bạn có server <code>toinoi-app-01</code> chạy trên máy mình, kết nối được bằng SSH Studio với fingerprint đã đối chiếu, và biết cách tạo lại hoặc xoá lab.</p>
</div>

## 1. Lab cần đạt bốn điều

| Yêu cầu | Lab làm thế nào |
|---|---|
| **Giống server thật** | Ubuntu 24.04 LTS, systemd là tiến trình số 1, `sshd` chạy như một dịch vụ, `sudo` hỏi mật khẩu |
| **Tái lập** | Ảnh gốc `ubuntu:24.04` ghim theo digest; dữ liệu mẫu cố định; mỗi bài có kịch bản lệnh và bản ghi mẫu `p01/OUTPUT.txt` |
| **Cô lập** | Chạy trong container, cổng SSH chỉ mở ở `127.0.0.1:2222` |
| **Dọn được** | Một lệnh xoá server; một lệnh tạo lại từ đầu |

## 2. Chuẩn bị

- **Docker Desktop** đang chạy, engine Linux (trên Windows là WSL 2). Server lab bị giới hạn 512 MB RAM.
- **Node.js 22** để chạy runner của lab. Runner không có dependency, không cần `npm install`.
- **SSH Studio 0.3.0**.

```sh
docker version --format '{{.Server.Version}}'
node --version
```

## 3. Dựng server

```sh
cd examples/toi-noi-linux
npm run up
```

Lần đầu Docker tải ảnh Ubuntu và cài các gói, mất vài phút. Khi xong, runner in thông tin kết nối:

```text
Server lab : toinoi-app-01 (Ubuntu 24.04)
Địa chỉ    : 127.0.0.1   Cổng: 2222
Người dùng : an   Mật khẩu: toinoi-lab (chỉ dùng cho lab)
Fingerprint: SHA256:/fjhnPhFNYvAMC3S/UdkbCQ039oKGZN4rJ96pPyUvkI (ED25519)
```

**Fingerprint của bạn sẽ khác** với chuỗi trên: host key được sinh ngẫu nhiên lúc Docker build ảnh. Hãy để cửa sổ này mở; bạn sẽ cần dòng `Fingerprint` ở mục 6. Muốn in lại, chạy `npm run info`.

`npm run up` thực chất là `docker compose up -d --build --wait` cộng với phần in thông tin. Kiểm tra container:

```sh
docker compose ps
```

```text
# đã lược cột COMMAND, CREATED
NAME            IMAGE                  SERVICE   STATUS                   PORTS
toinoi-app-01   toinoi-linux-lab:p01   app01     Up 2 minutes (healthy)   127.0.0.1:2222->22/tcp
```

## 4. Đọc cấu hình của lab

Bạn không cần hiểu hết Docker để học series này, nhưng nên biết lab được dựng thế nào để tin được nó.

### `docker-compose.yml`

```yaml
services:
  app01:
    build:
      context: ./docker
    image: toinoi-linux-lab:p01
    container_name: toinoi-app-01
    hostname: toinoi-app-01
    privileged: true
    cgroup: host
    tmpfs:
      - /run
      - /run/lock
      - /tmp
    volumes:
      - /sys/fs/cgroup:/sys/fs/cgroup:rw
    ports:
      - "127.0.0.1:2222:22"
    mem_limit: 512m
    healthcheck:
      test: ["CMD-SHELL", "systemctl is-active --quiet ssh"]
```

| Dòng | Ý nghĩa |
|---|---|
| `hostname: toinoi-app-01` | Tên máy bạn sẽ thấy trong dấu nhắc và trong kết quả `hostname` |
| `privileged`, `cgroup: host`, `tmpfs`, `/sys/fs/cgroup` | Bốn dòng này để **systemd** chạy được làm tiến trình số 1 trong container. Container thông thường chỉ chạy một chương trình; lab cần cả một hệ thống dịch vụ như máy thật |
| `ports: "127.0.0.1:2222:22"` | Cổng 22 của server được đưa ra cổng 2222 của máy bạn, và **chỉ** nghe trên `127.0.0.1`. Máy khác trong mạng không kết nối được |
| `mem_limit: 512m` | Giới hạn bộ nhớ của server |
| `healthcheck` | Container chỉ «healthy» khi dịch vụ `ssh` đã chạy, nên `--wait` chờ tới lúc bạn kết nối được |

`privileged: true` cho container nhiều quyền trên máy chạy Docker. Điều này chấp nhận được với một lab trên máy cá nhân có cổng chỉ mở cục bộ; đừng dùng cấu hình này cho thứ gì mở ra mạng.

### `docker/Dockerfile` (trích)

```dockerfile
FROM ubuntu:24.04@sha256:9c7f200c07e3d4a03e7249d2c16bf66a51120fb876aa2242cf31451412543289

RUN groupadd --gid 2000 toinoi \
 && useradd --create-home --uid 1000 --shell /bin/bash --comment "Nguyen Van An" an \
 && useradd --create-home --uid 1001 --shell /bin/bash --comment "Tran Thi Binh" binh \
 && useradd --system --uid 990 --gid toinoi --home-dir /opt/toinoi/bao-gia --shell /usr/sbin/nologin baogia \
 && usermod -aG sudo an \
 && usermod -aG sudo,toinoi binh

RUN ssh-keygen -A \
 && printf '%s\n' \
      'PasswordAuthentication yes' \
      'KbdInteractiveAuthentication no' \
      'PermitRootLogin no' \
      > /etc/ssh/sshd_config.d/10-toinoi-lab.conf

CMD ["/sbin/init"]
```

- **Ba tài khoản.** `an` là bạn. `binh` là đồng nghiệp bên hạ tầng, dùng trong các bài về quyền. `baogia` là tài khoản mà dịch vụ báo giá chạy dưới danh nghĩa đó; shell của nó là `/usr/sbin/nologin` nên không ai đăng nhập bằng tài khoản này được.
- **`ssh-keygen -A`** sinh host key cho server. Đây là nguồn gốc của fingerprint ở mục 3.
- **`PermitRootLogin no`**: không đăng nhập được bằng `root` qua SSH. Muốn quyền `root`, đăng nhập bằng tài khoản thường rồi dùng `sudo` ([LNX-007](/articles/linux-nguoi-dung-nhom-quyen-file)).
- **`PasswordAuthentication yes`**: lab cho đăng nhập bằng mật khẩu để bài đầu đơn giản. Phần 3 của series chuyển sang SSH key và tắt mật khẩu.
- **`CMD ["/sbin/init"]`**: chương trình đầu tiên của container là systemd.

## 5. Thêm server vào SSH Studio

Mở SSH Studio. Lần đầu, workspace «Cá nhân» chưa có server nào:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-dung-lab-docker-ket-noi-ssh-studio/ssh-studio-01.webp" alt="SSH Studio lúc mới mở: workspace Cá nhân chưa có server, giữa màn hình là nút Thêm server đầu tiên" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · trang Servers của workspace trống</figcaption></figure>

Bấm **Thêm server đầu tiên** và điền:

| Ô | Giá trị | Ghi chú |
|---|---|---|
| Tên hiển thị | `toinoi-app-01` | Tên tab và tên trong danh sách; đặt trùng hostname để khỏi nhầm |
| Địa chỉ server | `127.0.0.1` | Server lab nằm trên chính máy bạn |
| Port | `2222` | Cổng lab mở ra, không phải 22 |
| Username | `an` | Ô này mặc định là `root`; nhớ đổi |
| Nhóm | `Lab` | Nhãn để lọc danh sách |
| Phương thức xác thực | Password | Để trống ô «Lưu mật khẩu đăng nhập»: SSH Studio sẽ hỏi khi kết nối |

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-dung-lab-docker-ket-noi-ssh-studio/ssh-studio-02.webp" alt="Hộp thoại Thêm server của SSH Studio đã điền tên toinoi-app-01, địa chỉ 127.0.0.1, port 2222, username an, nhóm Lab, phương thức xác thực Password" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · hộp thoại <b>Thêm server</b> đã điền thông tin của lab</figcaption></figure>

Mục **Kết nối qua** để «Trực tiếp». Jump host và proxy là chuyện của Phần 6, khi server nằm sau một máy trung gian. Bấm **Lưu server** ở cuối hộp thoại.

Thao tác này tương đương việc viết một mục trong file `~/.ssh/config` của OpenSSH:

```text
Host toinoi-app-01
    HostName 127.0.0.1
    Port 2222
    User an
```

## 6. Kết nối lần đầu: đối chiếu fingerprint

Bấm **Kết nối** ở dòng `toinoi-app-01`. SSH Studio dừng lại ở hộp thoại này:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-dung-lab-docker-ket-noi-ssh-studio/ssh-studio-03.webp" alt="Hộp thoại Xác nhận danh tính server của SSH Studio hiển thị địa chỉ [127.0.0.1]:2222, fingerprint SHA256 của host key và hai nút Huỷ kết nối, Tin cậy và kết nối" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <b>Xác nhận danh tính server</b> khi kết nối lần đầu</figcaption></figure>

### Hộp thoại này hỏi gì?

Mỗi server SSH có một cặp khoá riêng gọi là **host key**. Khi bạn kết nối, server chứng minh nó đang giữ khoá bí mật của cặp đó. **Fingerprint** là bản rút gọn của khoá công khai, đủ ngắn để mắt người so được.

Lần đầu kết nối, máy bạn chưa biết host key của server, nên không có gì để so. Nếu có kẻ đứng giữa đường truyền và giả làm server, hắn sẽ đưa ra host key *của hắn*, và mọi thứ trông hoàn toàn bình thường, trừ một điều: fingerprint không trùng với fingerprint thật của server. Đối chiếu fingerprint là cách duy nhất phát hiện chuyện đó ở lần kết nối đầu.

### So với cái gì?

So với fingerprint lấy **từ phía server, qua một kênh bạn tin**: người quản trị gửi cho bạn, bảng điều khiển của nhà cung cấp đám mây in ra khi tạo máy, hoặc bạn đọc trực tiếp trên console của máy. Với lab, kênh tin cậy đó là Docker trên chính máy bạn: dòng `Fingerprint` mà `npm run up` in ra ở mục 3 được đọc bằng `docker exec`, không đi qua SSH.

So hai chuỗi. Không cần so từng ký tự; so vài ký tự đầu, vài ký tự cuối và một đoạn giữa là đủ để loại trừ nhầm lẫn. Trùng thì bấm **Tin cậy và kết nối**. Không trùng thì bấm **Huỷ kết nối** và tìm hiểu: bạn có đang trỏ nhầm địa chỉ hoặc cổng không?

Sau khi đăng nhập, bạn có thể tự đọc fingerprint ngay trên server để thấy nó đúng là chuỗi đó:

```text
an@toinoi-app-01:~$ ssh-keygen -lf /etc/ssh/ssh_host_ed25519_key.pub
256 SHA256:/fjhnPhFNYvAMC3S/UdkbCQ039oKGZN4rJ96pPyUvkI root@buildkitsandbox (ED25519)
```

`ssh-keygen -l` in fingerprint của một khoá, `-f` chỉ file khoá công khai. `256` là độ dài khoá tính bằng bit, `ED25519` là thuật toán. (Lệnh này chỉ để bạn hiểu fingerprint từ đâu ra. Nó chạy *sau* khi đã kết nối nên không thay được bước đối chiếu ở trên.)

## 7. Đăng nhập

Sau khi bạn tin cậy host key, SSH Studio hỏi mật khẩu:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-dung-lab-docker-ket-noi-ssh-studio/ssh-studio-04.webp" alt="Hộp thoại Mật khẩu cho an@127.0.0.1 của SSH Studio, ô Password đã nhập, bên dưới là ô đánh dấu Lưu mật khẩu, lần sau không cần nhập lại" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · hộp thoại mật khẩu cho <code>an@127.0.0.1</code></figcaption></figure>

Gõ `toinoi-lab` rồi bấm **Tiếp tục**. Ô «Lưu mật khẩu, lần sau không cần nhập lại» đang bỏ trống: mật khẩu chỉ dùng cho lần xác thực này. Nếu đánh dấu, SSH Studio mã hoá mật khẩu bằng AES-256-GCM và cất trong vault trên máy bạn. Với lab thì tuỳ bạn; series để trống để mỗi lần kết nối lại bạn đều thấy bước này.

Kết nối xong, tab của phiên mở ở chế độ Terminal:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-dung-lab-docker-ket-noi-ssh-studio/ssh-studio-05.webp" alt="Terminal của SSH Studio ngay sau khi đăng nhập: lời chào của server toinoi-app-01, gợi ý dùng sudo của Ubuntu và dấu nhắc an@toinoi-app-01:~$" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · lần đăng nhập đầu tiên vào <code>toinoi-app-01</code></figcaption></figure>

Đọc màn hình từ trên xuống:

- **«Tới Nơi · toinoi-app-01 · môi trường LAB»** là nội dung file `/etc/motd` (*message of the day*). Người quản trị dùng file này để nhắc bạn đang ở máy nào, môi trường nào. Trên server thật, dòng «môi trường PRODUCTION» ở đây đã cứu nhiều người khỏi gõ nhầm.
- **«To run a command as administrator…»** là gợi ý của Ubuntu cho tài khoản thuộc nhóm `sudo` chưa dùng `sudo` lần nào. Nó biến mất sau lần đầu bạn chạy `sudo`.
- **`an@toinoi-app-01:~$`** là dấu nhắc: shell đang chờ lệnh.

Góc trên bên phải của phiên ghi «Đã kết nối». Cột bên trái có thêm mục `toinoi-app-01` dưới «Phiên đang mở».

## 8. Những lệnh đầu tiên

Bấm vào vùng terminal và gõ từng lệnh, Enter sau mỗi lệnh:

```text
an@toinoi-app-01:~$ whoami
an
an@toinoi-app-01:~$ hostname
toinoi-app-01
an@toinoi-app-01:~$ pwd
/home/an
an@toinoi-app-01:~$ ls
huong-dan.txt
an@toinoi-app-01:~$ cat huong-dan.txt
Chào An,

Đây là server toinoi-app-01 của nhóm backend Tới Nơi (bản lab).
Dịch vụ báo giá nằm ở /opt/toinoi/bao-gia, log ở /var/log/toinoi.

Việc đầu tiên: làm quen với server, rồi chuẩn bị thư mục triển khai
/srv/toinoi theo hướng dẫn của series.

— Bình (nhóm hạ tầng)
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-dung-lab-docker-ket-noi-ssh-studio/ssh-studio-08.webp" alt="Terminal của SSH Studio ngay sau khi đăng nhập: whoami in an, hostname in toinoi-app-01, pwd in /home/an, ls thấy huong-dan.txt, cat huong-dan.txt in lời nhắn của Bình" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · những lệnh đầu tiên: <code>whoami</code>, <code>hostname</code>, <code>pwd</code>, <code>ls</code>, <code>cat</code></figcaption></figure>

`pwd` in thư mục bạn đang đứng; sau khi đăng nhập đó là thư mục riêng của bạn, `/home/an`. `ls` liệt kê thư mục đó: có một file `huong-dan.txt`. `cat` in nội dung file. Bình đã để lại lời nhắn về việc cần làm; đó là bài thực hành cuối phần.

Kiểm tra dịch vụ SSH, chính thứ đang giữ kết nối của bạn:

```text
an@toinoi-app-01:~$ systemctl is-active ssh
active
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-dung-lab-docker-ket-noi-ssh-studio/ssh-studio-06.webp" alt="Terminal của SSH Studio: ssh-keygen -lf /etc/ssh/ssh_host_ed25519_key.pub in fingerprint SHA256 ED25519 của server, systemctl is-active ssh in active" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · fingerprint đọc trên chính server khớp với hộp thoại lúc kết nối; dịch vụ <code>ssh</code> đang chạy</figcaption></figure>

## 9. Known Hosts: nơi SSH Studio nhớ host key

Bấm **Known Hosts** ở cột bên trái:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-dung-lab-docker-ket-noi-ssh-studio/ssh-studio-07.webp" alt="Trang Known Hosts của SSH Studio có một mục [127.0.0.1]:2222 kèm fingerprint SHA256 và nút Quên host" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · trang <b>Known Hosts</b> sau lần kết nối đầu</figcaption></figure>

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-dung-lab-docker-ket-noi-ssh-studio/ssh-studio-09.webp" alt="Hộp thoại Nhật ký hoạt động của SSH Studio với hai dòng: Đã lưu server và toinoi-app-01 Đã kết nối an@127.0.0.1, bộ lọc Tất cả và Lỗi, ô tìm kiếm, nút Xoá" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <b>Nhật ký hoạt động</b> (nút Hoạt động ở thanh trạng thái): mỗi lần lưu server, kết nối, lỗi đều được ghi lại trên máy bạn</figcaption></figure>

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-dung-lab-docker-ket-noi-ssh-studio/ssh-studio-10.webp" alt="Launchpad của SSH Studio, tab Phiên đang mở: một phiên toinoi-app-01 an@127.0.0.1:2222 thuộc workspace Cá nhân, trạng thái Đã kết nối, chế độ Terminal" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <b>Launchpad</b>: mọi phiên đang mở của mọi workspace ở một chỗ (Ctrl+Shift+D)</figcaption></figure>

Host key bạn vừa tin cậy được lưu ở đây, gắn với `[127.0.0.1]:2222`. Từ lần kết nối sau, SSH Studio so host key server đưa ra với bản đã lưu. Trùng thì kết nối thẳng, không hỏi nữa. Khác thì SSH Studio **chặn kết nối**: hoặc server đã được cài lại, hoặc có ai đó đang giả làm server.

Với lab, bạn sẽ gặp trường hợp này khi **build lại ảnh Docker** (ví dụ sau khi cập nhật lab): host key mới được sinh ra, fingerprint đổi. Cách xử lý đúng: chạy `npm run info` để lấy fingerprint mới, vào Known Hosts bấm **Quên host**, kết nối lại và đối chiếu như mục 6. Tạo lại container từ ảnh cũ (`npm run reset`) thì không đổi host key. Phần 3 của series ([LNX-023](/articles/linux-host-key-known-hosts)) bàn kỹ về host key trên server thật.

Với OpenSSH, dữ liệu tương ứng nằm trong file `~/.ssh/known_hosts`, và thao tác «Quên host» là `ssh-keygen -R '[127.0.0.1]:2222'`.

## 10. Tạo lại và dọn lab

| Việc | Lệnh (trong `examples/toi-noi-linux`) |
|---|---|
| Xem thông tin kết nối và fingerprint | `npm run info` |
| Tạo lại server, xoá mọi thay đổi bạn đã làm | `npm run reset` |
| Đưa server về trạng thái đầu một bài, ví dụ LNX-005 | `npm run setup -- 05` |
| Xoá server | `npm run down` |

Khi server bị tạo lại, phiên đang mở trong SSH Studio báo «Phiên đã kết thúc». Bấm **Kết nối lại** trên thanh thông báo và nhập mật khẩu.

Mọi thứ bạn làm trên server nằm trong container. `npm run reset` xoá hết, nên bạn có thể thử bất cứ lệnh nào trong series mà không sợ hỏng: hỏng thì tạo lại.

## 11. Đối chiếu thao tác

| Trên SSH Studio | Với OpenSSH trong terminal của máy bạn |
|---|---|
| Thêm server | Thêm mục `Host` vào `~/.ssh/config` |
| Kết nối | `ssh -p 2222 an@127.0.0.1` |
| Hộp thoại Xác nhận danh tính server | Câu hỏi `Are you sure you want to continue connecting (yes/no/[fingerprint])?` |
| Trang Known Hosts | File `~/.ssh/known_hosts` |
| Quên host | `ssh-keygen -R '[127.0.0.1]:2222'` |

## 12. Lỗi thường gặp

**Kết nối bị từ chối (connection refused).** Container chưa chạy hoặc bạn gõ nhầm cổng. Chạy `docker compose ps`; cột STATUS phải là `healthy` và cột PORTS phải có `127.0.0.1:2222->22/tcp`.

**Sai mật khẩu.** Kiểm tra ô Username trước: nếu vẫn là `root` thì mật khẩu nào cũng sai, vì lab cấm `root` đăng nhập.

**SSH Studio báo host key đã thay đổi.** Bạn đã build lại ảnh. Xử lý như mục 9; đừng bấm bỏ qua cảnh báo theo thói quen.

**`npm run up` báo cổng 2222 đã được dùng.** Một chương trình khác đang nghe ở cổng đó. Đổi `2222` trong `docker-compose.yml` thành cổng khác và dùng cổng mới trong SSH Studio.

## Bài tập

1. Chạy `npm run info` và so fingerprint với mục trong trang Known Hosts.
2. Thêm một server thứ hai tên `toinoi-app-01 (binh)` với username `binh`, cùng địa chỉ và cổng. Kết nối: SSH Studio có hỏi fingerprint lần nữa không? Vì sao?
3. Trong phiên của `binh`, chạy `whoami`, `pwd` và `ls /home`. Thư mục riêng của `binh` ở đâu?
4. Chạy `npm run reset`. Phiên trong SSH Studio báo gì? Kết nối lại: có bước fingerprint không? Giải thích bằng nội dung mục 9.

## Đọc tiếp

[LNX-003 · Shell và dòng lệnh](/articles/linux-shell-dong-lenh-tro-giup): một dòng lệnh gồm những phần nào, và cách tự tra khi gặp lệnh lạ.
