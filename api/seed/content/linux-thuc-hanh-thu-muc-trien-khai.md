---
slug: linux-thuc-hanh-thu-muc-trien-khai
title: "Thực hành Linux: chuẩn bị thư mục triển khai dịch vụ báo giá trên server"
summary: "Bài thực hành cuối Phần 1: dựng /srv/toinoi/bao-gia theo bố cục releases, shared và symlink current, đặt chủ sở hữu, nhóm và quyền cho từng mục, trích báo cáo lỗi từ log, kiểm tra từ góc nhìn của người dùng khác, rồi chấm bằng bộ kiểm tra 15 yêu cầu. Mọi bước làm trong SSH Studio trên server toinoi-app-01."
category: devops-linux
level: advanced
tags: [linux, lnx-010, thuc-hanh, deployment, symlink, permissions, ssh-studio, toi-noi]
language: vi
---

File `huong-dan.txt` mà Bình để lại trong thư mục riêng của An từ bài 2 có một câu: «Việc đầu tiên: làm quen với server, rồi chuẩn bị thư mục triển khai `/srv/toinoi` theo hướng dẫn của series.» Chín bài vừa rồi là phần làm quen. Bài này là phần việc.

Dịch vụ `bao-gia` hiện nằm ở `/opt/toinoi/bao-gia`: mọi thứ trong một thư mục, bản mới chép đè lên bản cũ. Nhóm muốn chuyển sang một bố cục cho phép triển khai bản mới mà **không ghi đè bản đang chạy** và **quay lui trong một lệnh**. An chuẩn bị bố cục đó. Bài không dạy lệnh mới ngoài `ln -s`; nó đòi bạn dùng đúng những lệnh đã học, theo đúng thứ tự, và kiểm chứng kết quả.

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-010 · Phần 1/6 · Nâng cao · Thực hành</span>
<p>Đọc trước: LNX-001 tới <a href="/articles/linux-tim-kiem-find-grep">LNX-009</a>. Trạng thái đầu bài: <code>npm run setup -- 10</code>, rồi kết nối lại bằng SSH Studio. Sản phẩm: <code>/srv/toinoi/bao-gia</code> đạt <code>15 PASS, 0 FAIL</code> khi chạy <code>npm run kiem-tra</code>. Hãy tự làm theo đề ở mục 1 trước khi đọc lời giải từ mục 3.</p>
</div>

## 1. Đề bài

Dựng cây thư mục sau, thuộc về người dùng `an` và nhóm `toinoi`:

```text
/srv/toinoi/bao-gia
├── current -> releases/1.4.2      symlink tương đối tới bản đang chạy
├── releases
│   └── 1.4.2
│       ├── bang-gia.csv           640
│       └── kiem-tra-cau-hinh.sh   750
└── shared
    ├── config                     750
    │   ├── appsettings.json       640
    │   └── bi-mat.env             600
    └── logs                       770
        ├── loi-2026-11-21.txt     các dòng ERROR của log ngày 21/11
        └── loi-theo-dich-vu.txt   số lỗi theo dịch vụ, nhiều nhất ở trên
```

Yêu cầu:

1. Mọi file và thư mục trong `releases` và `shared` có chủ sở hữu `an`, nhóm `toinoi`.
2. Quyền đúng như ghi trong sơ đồ; `releases/1.4.2` là `755`.
3. `current` là symlink **tương đối** tới `releases/1.4.2`, và `current/kiem-tra-cau-hinh.sh` chạy được.
4. `binh` (thuộc nhóm `toinoi`) đọc được `appsettings.json` nhưng **không** đọc được `bi-mat.env`.
5. Người ngoài nhóm `toinoi` không liệt kê được `shared/config`.
6. `loi-2026-11-21.txt` có đúng các dòng `ERROR` của `/var/log/toinoi/bao-gia.log`.

Nguồn: các file trong `/opt/toinoi/bao-gia`.

## 2. Vì sao bố cục này?

| Thư mục | Chứa | Vòng đời |
|---|---|---|
| `releases/<phiên bản>/` | Mã và dữ liệu đi kèm của **một** bản phát hành | Không sửa sau khi tạo. Bản mới là một thư mục mới |
| `shared/config/` | Cấu hình và bí mật của môi trường này | Tồn tại qua mọi bản phát hành |
| `shared/logs/` | Log, báo cáo | Tồn tại qua mọi bản phát hành, lớn dần |
| `current` | Symlink tới bản đang chạy | Đổi mỗi lần triển khai |

Dịch vụ luôn được khởi động từ `current/`. Triển khai bản 1.5.0 là: chép vào `releases/1.5.0`, kiểm tra, rồi trỏ `current` sang đó. Bản 1.4.2 vẫn nằm nguyên. Quay lui là trỏ `current` về lại. Phần 6 của series (LNX-057) hoàn thiện quy trình này; ở đây ta dựng nền.

Quyền theo cùng một lý lẽ: mã thì nhóm đọc và chạy được; cấu hình thì chỉ nhóm đọc; bí mật thì chỉ chủ đọc; log thì nhóm ghi được.

## 3. Tạo cây thư mục

```text
an@toinoi-app-01:~$ ls -ld /srv
drwxr-xr-x 2 root root 4096 Nov 10 21:48 /srv
an@toinoi-app-01:~$ mkdir -p /srv/toinoi/bao-gia
mkdir: cannot create directory ‘/srv/toinoi’: Permission denied
an@toinoi-app-01:~$ sudo mkdir -p /srv/toinoi/bao-gia/{releases/1.4.2,shared/config,shared/logs}
an@toinoi-app-01:~$ sudo chown -R an:toinoi /srv/toinoi
an@toinoi-app-01:~$ tree /srv/toinoi
/srv/toinoi
└── bao-gia
    ├── releases
    │   └── 1.4.2
    └── shared
        ├── config
        └── logs

7 directories, 0 files
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-thuc-hanh-thu-muc-trien-khai/ssh-studio-05.webp" alt="Terminal của SSH Studio: ls -ld /srv thuộc root, mkdir -p /srv/toinoi/bao-gia bị Permission denied, sudo mkdir -p với brace expansion đang dừng ở dấu nhắc [sudo] password for an" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>/srv</code> thuộc root: cần <code>sudo</code> để tạo cây thư mục</figcaption></figure>

- `/srv` thuộc về `root` với quyền `755`: `an` ở lớp «người khác», không có `w`, nên `mkdir` bị từ chối. Đây là một `Permission denied` chính đáng: tạo thư mục cấp cao của hệ thống là việc cần `sudo`.
- `sudo mkdir -p` với **brace expansion** tạo cả cây bằng một lệnh. Shell mở rộng `{releases/1.4.2,shared/config,shared/logs}` thành ba đường dẫn trước khi `sudo` chạy.
- Thư mục tạo bằng `sudo` thuộc về `root`. **`sudo chown -R an:toinoi /srv/toinoi`** giao cả cây cho `an` và nhóm `toinoi`. Từ đây trở đi, phần lớn việc không cần `sudo` nữa.

Nguyên tắc: dùng `sudo` để vượt qua ranh giới một lần, giao quyền đúng chỗ, rồi quay về làm việc bằng tài khoản thường.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-thuc-hanh-thu-muc-trien-khai/ssh-studio-01.webp" alt="Terminal của SSH Studio: ls -ld /srv, mkdir bị Permission denied, sudo mkdir -p với brace expansion hỏi mật khẩu, sudo chown -R an:toinoi và cây /srv/toinoi gồm releases/1.4.2, shared/config, shared/logs" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · tạo cây thư mục bằng <code>sudo mkdir -p</code> rồi giao lại cho <code>an:toinoi</code></figcaption></figure>

## 4. Đưa bản phát hành và cấu hình vào

```text
an@toinoi-app-01:~$ cd /srv/toinoi/bao-gia
an@toinoi-app-01:/srv/toinoi/bao-gia$ cp /opt/toinoi/bao-gia/kiem-tra-cau-hinh.sh /opt/toinoi/bao-gia/bang-gia.csv releases/1.4.2/
an@toinoi-app-01:/srv/toinoi/bao-gia$ cp /opt/toinoi/bao-gia/appsettings.json shared/config/
an@toinoi-app-01:/srv/toinoi/bao-gia$ sudo cp /opt/toinoi/bao-gia/bi-mat.env shared/config/
an@toinoi-app-01:/srv/toinoi/bao-gia$ sudo chown an:toinoi shared/config/bi-mat.env
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-thuc-hanh-thu-muc-trien-khai/ssh-studio-07.webp" alt="Terminal của SSH Studio trong /srv/toinoi/bao-gia: cp kiem-tra-cau-hinh.sh và bang-gia.csv vào releases/1.4.2, cp appsettings.json vào shared/config, sudo cp bi-mat.env rồi sudo chown an:toinoi" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · đưa bản phát hành 1.4.2 và cấu hình vào chỗ</figcaption></figure>

Ba file đầu `an` đọc được nên `cp` thường là đủ. `bi-mat.env` thuộc `baogia` với quyền `600`, `an` không đọc được ([LNX-007](/articles/linux-nguoi-dung-nhom-quyen-file)). `sudo cp` chép được, nhưng bản sao thuộc về `root`, nên cần thêm `sudo chown`.

Đường dẫn đích ở đây là tương đối (`releases/1.4.2/`, `shared/config/`), và chúng đúng vì lệnh đầu tiên là `cd /srv/toinoi/bao-gia`. Dấu nhắc xác nhận điều đó ở mọi dòng.

## 5. Đặt nhóm và quyền

```text
an@toinoi-app-01:/srv/toinoi/bao-gia$ ls -l releases/1.4.2
total 8
-rw-r--r-- 1 an an 188 Nov 22 22:27 bang-gia.csv
-rw-r--r-- 1 an an 403 Nov 22 22:27 kiem-tra-cau-hinh.sh
an@toinoi-app-01:/srv/toinoi/bao-gia$ chgrp -R toinoi releases shared
an@toinoi-app-01:/srv/toinoi/bao-gia$ chmod 750 releases/1.4.2/kiem-tra-cau-hinh.sh
an@toinoi-app-01:/srv/toinoi/bao-gia$ chmod 640 releases/1.4.2/bang-gia.csv shared/config/appsettings.json
an@toinoi-app-01:/srv/toinoi/bao-gia$ chmod 600 shared/config/bi-mat.env
an@toinoi-app-01:/srv/toinoi/bao-gia$ chmod 750 shared/config
an@toinoi-app-01:/srv/toinoi/bao-gia$ chmod 770 shared/logs
an@toinoi-app-01:/srv/toinoi/bao-gia$ find . -mindepth 1 -exec stat -c '%a %U:%G %n' {} + | sort -k3
755 an:toinoi ./releases
755 an:toinoi ./releases/1.4.2
640 an:toinoi ./releases/1.4.2/bang-gia.csv
750 an:toinoi ./releases/1.4.2/kiem-tra-cau-hinh.sh
755 an:toinoi ./shared
750 an:toinoi ./shared/config
640 an:toinoi ./shared/config/appsettings.json
600 an:toinoi ./shared/config/bi-mat.env
770 an:toinoi ./shared/logs
```

`ls -l` cho thấy một điều dễ bỏ sót: các file vừa chép có nhóm **`an`**, không phải `toinoi`. Thư mục thuộc nhóm `toinoi` không làm file tạo bên trong thuộc nhóm đó; file mới luôn nhận **nhóm chính của người tạo**. Nếu dừng ở đây, `binh` sẽ không đọc được gì.

**`chgrp -R toinoi releases shared`** sửa điều đó. Lệnh này **không cần `sudo`**: `an` là chủ của các file và, từ khi đăng nhập lại ở bài 8, thuộc nhóm `toinoi`. Ở bài 7, cũng lệnh `chgrp toinoi` này đã bị từ chối, vì lúc đó phiên của `an` chưa có nhóm `toinoi`.

(Có cách để file mới trong một thư mục tự nhận nhóm của thư mục: bit setgid, nội dung của Phần 3.)

Rồi `chmod` từng mục theo đề. Để ý thứ tự của lý lẽ, không phải của lệnh:

| Mục | Quyền | Ai làm được gì |
|---|---|---|
| `kiem-tra-cau-hinh.sh` | `750` | Chủ sửa và chạy; nhóm đọc và chạy; người khác không gì cả |
| `bang-gia.csv`, `appsettings.json` | `640` | Chủ sửa; nhóm đọc |
| `bi-mat.env` | `600` | Chỉ chủ |
| `shared/config` | `750` | Nhóm vào và liệt kê được; người khác không vào được |
| `shared/logs` | `770` | Chủ và nhóm tạo, xoá file được |

Lệnh cuối là một phép kiểm tra tự viết: `find` đưa mọi mục cho `stat`, `stat` in quyền dạng số, chủ, nhóm và đường dẫn, `sort -k3` xếp theo đường dẫn. Chín dòng, đọc một lượt là đối chiếu được với đề. **Đừng tin rằng mình đã gõ đúng; hãy in ra và nhìn.**

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-thuc-hanh-thu-muc-trien-khai/ssh-studio-02.webp" alt="Terminal của SSH Studio: ls -l releases/1.4.2 thấy file mới chép thuộc nhóm an, chgrp -R toinoi, các lệnh chmod 750, 640, 600, 750, 770, rồi find -exec stat in quyền và chủ của từng mục" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · đặt nhóm và quyền đúng cho từng loại file</figcaption></figure>

## 6. Symlink `current`

```text
an@toinoi-app-01:/srv/toinoi/bao-gia$ ln -s releases/1.4.2 current
an@toinoi-app-01:/srv/toinoi/bao-gia$ ls -l
total 8
lrwxrwxrwx 1 an an       14 Nov 22 22:27 current -> releases/1.4.2
drwxr-xr-x 3 an toinoi 4096 Nov 22 22:27 releases
drwxr-xr-x 4 an toinoi 4096 Nov 22 22:27 shared
an@toinoi-app-01:/srv/toinoi/bao-gia$ current/kiem-tra-cau-hinh.sh
bao-gia: bang gia co 4 loai dich vu
bao-gia: cau hinh hop le
```

**`ln -s ĐÍCH TÊN`** tạo một symlink tên `TÊN` trỏ tới `ĐÍCH`. Thứ tự đối số giống `cp`: cái có sẵn trước, cái tạo ra sau.

Trong `ls -l`, symlink bắt đầu bằng `l`, có mũi tên `->` tới đích, và quyền luôn hiện là `rwxrwxrwx`. Quyền đó không có nghĩa gì: quyền thật là quyền của **đích**.

`current/kiem-tra-cau-hinh.sh` chạy qua symlink như thể `current` là một thư mục thật. Script in «bang gia co 4 loai dich vu»: nó đọc `bang-gia.csv` nằm cạnh nó trong `releases/1.4.2`.

### Tương đối hay tuyệt đối?

Symlink lưu đích như **một chuỗi**. Chuỗi đó có thể là đường dẫn tương đối, và khi ấy nó được tính **từ thư mục chứa symlink**, không phải từ thư mục bạn đang đứng lúc tạo.

- `ln -s releases/1.4.2 current` (tương đối): cả cây `bao-gia` có thể được đổi tên, di chuyển, sao lưu rồi giải nén ở chỗ khác mà symlink vẫn đúng. Đề bài yêu cầu dạng này.
- `ln -s /srv/toinoi/bao-gia/releases/1.4.2 current` (tuyệt đối): vẫn đúng khi symlink bị chuyển đi nơi khác, nhưng gãy khi cây bị chuyển.

Symlink trỏ tới đích không tồn tại gọi là symlink gãy; `ls -l` vẫn hiện nó (thường tô đỏ), còn mọi lệnh đọc qua nó báo `No such file or directory`.

Để trỏ `current` sang bản khác: `ln -sfn releases/1.5.0 current`. `-f` thay symlink cũ, `-n` ngăn `ln` đi **vào** thư mục mà `current` đang trỏ tới rồi tạo symlink ở trong đó. Quên `-n` là lỗi kinh điển của thao tác này.

## 7. Trích báo cáo lỗi

```text
an@toinoi-app-01:/srv/toinoi/bao-gia$ grep ERROR /var/log/toinoi/bao-gia.log > shared/logs/loi-2026-11-21.txt
an@toinoi-app-01:/srv/toinoi/bao-gia$ wc -l shared/logs/loi-2026-11-21.txt
17 shared/logs/loi-2026-11-21.txt
an@toinoi-app-01:/srv/toinoi/bao-gia$ grep -o 'dich_vu=[a-z0-9]*' shared/logs/loi-2026-11-21.txt | sort | uniq -c | sort -rn | tee shared/logs/loi-theo-dich-vu.txt
      6 dich_vu=car4
      5 dich_vu=bike
      4 dich_vu=car7
      2 dich_vu=delivery
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-thuc-hanh-thu-muc-trien-khai/ssh-studio-06.webp" alt="Terminal của SSH Studio: ln -s releases/1.4.2 current, ls -l thấy current -> releases/1.4.2, chạy current/kiem-tra-cau-hinh.sh, trích 17 dòng ERROR và đếm theo dịch vụ bằng tee" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · symlink <code>current</code> và báo cáo lỗi của ngày 21/11</figcaption></figure>

Ba lệnh, ba kỹ thuật của [LNX-008](/articles/linux-chuyen-huong-pipe):

1. `grep … > file`: lọc và ghi ra file.
2. `wc -l`: kiểm tra ngay rằng file có 17 dòng, đúng con số đã biết.
3. `grep -o … | sort | uniq -c | sort -rn | tee`: đếm theo dịch vụ, xếp giảm dần, vừa xem vừa lưu.

Hai file mới nằm trong `shared/logs` với nhóm `an`, vì `chgrp -R` ở mục 5 chạy trước khi chúng được tạo. Đề bài không ràng buộc nhóm của hai file báo cáo, nhưng đây chính là tình huống mà bit setgid sinh ra để giải quyết.

Chuyển sang chế độ **Song song** và trỏ khung Remote server vào `/srv/toinoi/bao-gia`:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-thuc-hanh-thu-muc-trien-khai/ssh-studio-03.webp" alt="SSH Studio ở chế độ Song song: terminal bên trái với ln -s, ls -l hiển thị current -> releases/1.4.2, kết quả chạy script qua symlink và báo cáo lỗi theo dịch vụ; khung Remote server bên phải ở /srv/toinoi/bao-gia liệt kê releases, shared và current có biểu tượng liên kết" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · chế độ <b>Song song</b>: symlink <code>current</code> trong terminal và trong khung Files</figcaption></figure>

Trong khung Files, `current` có biểu tượng riêng của symlink. Nhấp đúp `current` đưa bạn vào nội dung của `releases/1.4.2`: SSH Studio mở được symlink trỏ tới thư mục.

## 8. Nhìn bằng mắt người khác

Bạn không thể kiểm tra quyền bằng chính tài khoản của mình: chủ sở hữu thì cái gì cũng đọc được. Phải thử dưới danh nghĩa những người mà quyền nhắm tới.

```text
an@toinoi-app-01:/srv/toinoi/bao-gia$ sudo -u binh head -n 3 /srv/toinoi/bao-gia/shared/config/appsettings.json
{
  "Service": {
    "Name": "bao-gia",
an@toinoi-app-01:/srv/toinoi/bao-gia$ sudo -u binh cat /srv/toinoi/bao-gia/shared/config/bi-mat.env
cat: /srv/toinoi/bao-gia/shared/config/bi-mat.env: Permission denied
an@toinoi-app-01:/srv/toinoi/bao-gia$ sudo -u nobody ls /srv/toinoi/bao-gia/shared/config
ls: cannot open directory '/srv/toinoi/bao-gia/shared/config': Permission denied
```

**`sudo -u NGƯỜI_DÙNG LỆNH`** chạy lệnh dưới danh nghĩa người dùng đó thay vì `root`.

| Thử | Kết quả | Vì |
|---|---|---|
| `binh` đọc `appsettings.json` | Được | `binh` thuộc `toinoi`; file là `640`, thư mục `config` là `750` |
| `binh` đọc `bi-mat.env` | `Permission denied` | File là `600`: nhóm không có quyền gì |
| `nobody` liệt kê `shared/config` | `Permission denied` | `nobody` là «người khác»; thư mục `750` không cho người khác quyền nào |

`nobody` là tài khoản có sẵn trên mọi hệ thống, không thuộc nhóm nào đáng kể; nó là hình mẫu tốt cho «một tiến trình bất kỳ trên máy». Kiểm tra cả trường hợp **được phép** lẫn trường hợp **bị cấm**: quyền quá chặt làm hỏng dịch vụ, quyền quá lỏng làm lộ bí mật, và chỉ thử một phía thì không phát hiện được phía kia.

## 9. Kết quả bàn giao

```text
an@toinoi-app-01:/srv/toinoi/bao-gia$ tree -p /srv/toinoi
[drwxr-xr-x]  /srv/toinoi
└── [drwxr-xr-x]  bao-gia
    ├── [lrwxrwxrwx]  current -> releases/1.4.2
    ├── [drwxr-xr-x]  releases
    │   └── [drwxr-xr-x]  1.4.2
    │       ├── [-rw-r-----]  bang-gia.csv
    │       └── [-rwxr-x---]  kiem-tra-cau-hinh.sh
    └── [drwxr-xr-x]  shared
        ├── [drwxr-x---]  config
        │   ├── [-rw-r-----]  appsettings.json
        │   └── [-rw-------]  bi-mat.env
        └── [drwxrwx---]  logs
            ├── [-rw-rw-r--]  loi-2026-11-21.txt
            └── [-rw-rw-r--]  loi-theo-dich-vu.txt

8 directories, 6 files
```

`tree -p` in quyền của từng mục; so với sơ đồ ở mục 1.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-thuc-hanh-thu-muc-trien-khai/ssh-studio-04.webp" alt="Terminal của SSH Studio: binh đọc được ba dòng đầu của appsettings.json, binh bị Permission denied với bi-mat.env, nobody không liệt kê được shared/config, và cây tree -p /srv/toinoi với quyền của từng mục" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · kiểm tra bằng tài khoản khác và cây thư mục bàn giao</figcaption></figure>

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-thuc-hanh-thu-muc-trien-khai/ssh-studio-08.webp" alt="Chế độ Files của SSH Studio: khung Remote server mở /srv/toinoi/bao-gia với hai thư mục releases, shared và symlink current có biểu tượng liên kết" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · thư mục triển khai bàn giao nhìn từ khung Files</figcaption></figure>

### Chấm tự động

Trên máy của bạn (không phải trên server), trong `examples/toi-noi-linux`:

```sh
npm run kiem-tra
```

```text
PASS  releases/1.4.2 có quyền và chủ sở hữu 755 an:toinoi
PASS  releases/1.4.2/kiem-tra-cau-hinh.sh có quyền và chủ sở hữu 750 an:toinoi
PASS  releases/1.4.2/bang-gia.csv có quyền và chủ sở hữu 640 an:toinoi
PASS  shared/config có quyền và chủ sở hữu 750 an:toinoi
PASS  shared/config/appsettings.json có quyền và chủ sở hữu 640 an:toinoi
PASS  shared/config/bi-mat.env có quyền và chủ sở hữu 600 an:toinoi
PASS  shared/logs có quyền và chủ sở hữu 770 an:toinoi
PASS  current là symlink tương đối tới releases/1.4.2
PASS  an chạy được current/kiem-tra-cau-hinh.sh
PASS  loi-2026-11-21.txt có đúng 17 dòng ERROR
PASS  loi-theo-dich-vu.txt xếp car4 (6 lỗi) lên đầu
PASS  binh (nhóm toinoi) đọc được appsettings.json
PASS  binh KHÔNG đọc được bi-mat.env
PASS  người ngoài nhóm KHÔNG liệt kê được shared/config
PASS  an thuộc nhóm toinoi

15 PASS, 0 FAIL
```

Bộ chấm đọc trạng thái thật của server qua `docker exec`: nó `stat` từng đường dẫn, đọc symlink, chạy script dưới tài khoản `an`, và thử đọc file dưới tài khoản `binh` và `nobody`. Dòng `FAIL` kèm giá trị thực tế, ví dụ `thực tế: 644 an:an`, để bạn biết sửa ở đâu.

Ảnh chụp trong cả Phần 1 được tạo bằng cách cho SSH Studio gõ đúng các lệnh của từng bài lên một server mới; sau lượt đó, bộ chấm này cũng cho `15 PASS`.

## 10. Lỗi thường gặp trong bài này

**`FAIL … thực tế: 640 an:an`.** Quên `chgrp`, hoặc chạy `chgrp -R` trước khi chép file vào. Chạy lại `chgrp -R toinoi releases shared`.

**`chgrp: … Operation not permitted`.** Phiên của bạn chưa có nhóm `toinoi`. Chạy `id`; nếu thiếu, `exit` rồi **Kết nối lại**. (Nếu bạn dùng `npm run setup -- 10`, nhớ kết nối lại sau khi server được tạo lại.)

**`bi-mat.env` thuộc `root:root`.** Quên `sudo chown` sau `sudo cp`. Khi đó `chmod 600` cũng thất bại, vì bạn không phải chủ của file.

**`current` trỏ sai hoặc là symlink tuyệt đối.** Xoá và tạo lại: `rm current` (không có `-r` và không có dấu `/` ở cuối: `rm -r current/` xoá nội dung của **đích**, không xoá symlink), rồi `ln -s releases/1.4.2 current` từ trong `/srv/toinoi/bao-gia`.

**`current -> releases/1.4.2` nhưng `current/…` báo không tồn tại.** Bạn đã tạo symlink từ thư mục khác; đường dẫn tương đối được tính từ nơi symlink nằm. `readlink current` in đích đã lưu.

**`binh` không đọc được `appsettings.json`.** Kiểm tra quyền của **mọi thư mục dọc đường**: `namei -l /srv/toinoi/bao-gia/shared/config/appsettings.json`.

## 11. Bạn đã có gì sau Phần 1

| Kỹ năng | Bài |
|---|---|
| Kết nối tới server, đối chiếu fingerprint, đọc dấu nhắc | LNX-002, LNX-003 |
| Tự tra lệnh lạ bằng `--help` và `man` | LNX-003 |
| Đi lại trong cây thư mục, hiểu `/etc`, `/var`, `/opt`, `/srv` | LNX-004 |
| Tạo, chép, chuyển, xoá; tải file qua khung Files | LNX-005 |
| Đọc log dài, sửa file bằng `vim` và trình sửa file | LNX-006 |
| Đọc và đặt quyền, dùng `sudo` có chủ đích | LNX-007 |
| Nối lệnh bằng pipe, ghi kết quả ra file | LNX-008 |
| Tìm file và tìm trong nội dung | LNX-009 |
| Ghép tất cả thành một việc có kiểm chứng | LNX-010 |

Phần 2 biến những dòng lệnh bạn vừa gõ tay thành **script**: biến, điều kiện, vòng lặp, `sed`, `awk`, và một script sao lưu và báo cáo log chạy theo lịch.

## Bài tập mở rộng

1. Tạo `releases/1.5.0` bằng cách chép `releases/1.4.2` với `cp -a`, sửa giá mở cửa xe máy thành 9000 trong bản mới bằng `vim`, rồi trỏ `current` sang 1.5.0 bằng `ln -sfn`. Chạy `current/kiem-tra-cau-hinh.sh`. Quay lui về 1.4.2 bằng một lệnh. Bộ chấm còn `15 PASS` không?
2. Thử `ln -sf releases/1.5.0 current` (thiếu `-n`) khi `current` đang trỏ tới 1.4.2. Chuyện gì xảy ra? Tìm symlink lạc bằng `find /srv/toinoi -type l`.
3. Viết **một** lệnh `find` liệt kê mọi file dưới `/srv/toinoi` mà nhóm `toinoi` không đọc được (gợi ý: `-not -perm -g+r`). Kết quả có đúng với ý đồ của đề không?
4. Dùng `sudo -u binh` thử tạo một file trong `shared/logs` và trong `shared/config`. Giải thích hai kết quả bằng quyền của hai thư mục.
5. Xoá `/srv/toinoi` và làm lại toàn bộ bài mà không nhìn lời giải, chỉ dùng đề ở mục 1 và `npm run kiem-tra`.
