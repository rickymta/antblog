---
slug: linux-job-nen-nohup-tmux
title: "Job nền, nohup và tmux: giữ việc chạy lâu sống sót khi phiên SSH đứt"
summary: "Chạy lệnh nền bằng &, jobs, vì sao đóng terminal hay mất mạng làm job chết (SIGHUP), thử thật trong SSH Studio: job thường chết, job chạy bằng nohup sống; rồi tmux: tạo phiên tách rời, đứt kết nối, đăng nhập lại và gắn vào đúng chỗ đang dở, đọc nội dung bằng capture-pane, tách ra bằng Ctrl+B D."
category: devops-linux
level: intermediate
tags: [linux, lnx-034, jobs, nohup, tmux, sighup, background, ssh-studio, toi-noi]
language: vi
---

Bình cần xuất dữ liệu chuyến của cả năm từ server, một việc chạy vài tiếng. Lần trước Bình chạy nó trong SSH Studio trên laptop, gập máy đi họp, quay lại thì phiên SSH đã đứt và việc xuất dữ liệu chết theo, mới được một nửa. Lần này An muốn chắc việc chạy lâu không phụ thuộc vào kết nối của ai.

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-034 · Phần 4/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/linux-tin-hieu-kill">LNX-032</a> (SIGHUP), <a href="/articles/linux-apt-cai-phan-mem">LNX-033</a> (cài tmux). Trạng thái đầu bài: <code>npm run setup -- 34</code>. Lab có lệnh <code>xuat-chuyen N</code> mô phỏng việc chạy lâu: mỗi 2 giây xuất thêm 10 chuyến và in tiến độ. Bài đóng hẳn phiên SSH trong SSH Studio rồi kết nối lại; bản ghi chữ của lab mô phỏng việc đó bằng cách gửi SIGHUP như bash làm khi mất kết nối.</p>
</div>

## 1. Job nền trong shell

Thêm **`&`** vào cuối lệnh để chạy nó **nền**: shell trả lại dấu nhắc ngay, lệnh vẫn chạy.

```text
an@toinoi-app-01:~$ cd ~
an@toinoi-app-01:~$ xuat-chuyen 40 > xuat-1.log &
an@toinoi-app-01:~$ jobs -l
[1]+  3466 Running                 xuat-chuyen 40 > xuat-1.log &
an@toinoi-app-01:~$ cat xuat-1.log
07:05:36 da xuat 10/40 chuyen
```

- **`jobs -l`** liệt kê các job của **shell hiện tại**: số job `[1]`, PID, trạng thái. `%1` là cách gọi job số 1 (`kill %1`, `fg %1`).
- Trong shell tương tác, **`fg`** đưa job lên trước (Ctrl+C dừng được), **Ctrl+Z** tạm dừng lệnh đang chạy và **`bg`** cho nó chạy tiếp ở nền.
- Job nền vẫn thuộc về shell và terminal đã chạy nó. Đây là điểm mấu chốt.

## 2. Mất kết nối thì job chết

Khi phiên SSH đóng (đóng tab, mất mạng, laptop ngủ), `sshd` đóng terminal ảo; kernel gửi **`SIGHUP`** («hang up», cái tên có từ thời modem cúp máy) cho shell, và **bash chuyển tiếp `SIGHUP` cho mọi job của nó**. Mặc định của `SIGHUP` là thoát ([LNX-032](/articles/linux-tin-hieu-kill)).

**`nohup LỆNH`** chạy lệnh với `SIGHUP` bị bỏ qua. An thử cả hai cách cùng lúc:

```text
an@toinoi-app-01:~$ xuat-chuyen 600 > xuat-2.log 2>&1 &
an@toinoi-app-01:~$ nohup xuat-chuyen 600 > xuat-3.log 2>&1 &
an@toinoi-app-01:~$ jobs -l
[1]   3466 Running                 xuat-chuyen 40 > xuat-1.log &
[2]-  3479 Running                 xuat-chuyen 600 > xuat-2.log 2>&1 &
[3]+  3480 Running                 nohup xuat-chuyen 600 > xuat-3.log 2>&1 &
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-job-nen-nohup-tmux/ssh-studio-01.webp" alt="Terminal của SSH Studio: xuat-chuyen 40 chạy nền, jobs -l, cat xuat-1.log có dòng đầu tiên, rồi xuat-chuyen 600 chạy nền thường và nohup xuat-chuyen 600 chạy nền, jobs -l liệt kê ba job" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · ba job nền trước khi đóng phiên</figcaption></figure>

Bây giờ An **đóng hẳn tab phiên** trong SSH Studio (nút ✕ trên tab, xác nhận Ngắt kết nối), rồi kết nối lại bằng một tab mới:

```text
an@toinoi-app-01:~$ pgrep -a -f 'xuat-chuyen 600'
3480 bash /usr/local/bin/xuat-chuyen 600
an@toinoi-app-01:~$ tail -n 1 xuat-2.log xuat-3.log
==> xuat-2.log <==

==> xuat-3.log <==
07:05:39 da xuat 10/600 chuyen
an@toinoi-app-01:~$ tail -n 1 xuat-2.log xuat-3.log
==> xuat-2.log <==

==> xuat-3.log <==
07:05:41 da xuat 20/600 chuyen
an@toinoi-app-01:~$ pkill -f 'xuat-chuyen 600'
```

- Chỉ còn bản chạy bằng `nohup`. Bản chạy nền thường đã chết theo phiên cũ: `xuat-2.log` không tăng nữa. (Trong bản ghi của lab, nó chết trước khi kịp in dòng nào; trong ảnh chụp SSH Studio, nó kịp in hai dòng rồi dừng ở «20/600» đúng lúc tab đóng.)
- `xuat-3.log` vẫn tăng đều; trong ảnh nó đã lên 220, rồi 230 chuyến, vì kết nối lại (kèm mã OTP) mất khoảng nửa phút. `nohup` không «tách» lệnh khỏi phiên, nó chỉ khiến lệnh không chết vì `SIGHUP`.
- `jobs` trong phiên mới trống: job là khái niệm của từng shell, shell mới không biết gì về tiến trình của shell cũ. Tìm lại bằng `pgrep`.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-job-nen-nohup-tmux/ssh-studio-02.webp" alt="Terminal của SSH Studio sau khi kết nối lại: pgrep chỉ còn tiến trình chạy bằng nohup, tail cho thấy xuat-2.log dừng ở 20/600 còn xuat-3.log tiếp tục tăng từ 220 lên 230/600, pkill dừng nó" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · sau khi kết nối lại: chỉ job chạy bằng <code>nohup</code> còn sống</figcaption></figure>

`nohup` đủ cho việc «chạy rồi để đó». Nhưng bạn không xem tiến độ trực tiếp được nữa (chỉ đọc file), không trả lời được nếu chương trình hỏi gì, và không quay lại đúng màn hình cũ được.

## 3. tmux: phiên terminal sống trên server

**tmux** (*terminal multiplexer*) chạy một **server nhỏ trên chính máy chủ**, giữ các phiên terminal của nó. Terminal của bạn (SSH Studio) chỉ là một cửa sổ **gắn** (*attach*) vào phiên tmux. Đứt SSH thì cửa sổ mất, phiên tmux vẫn chạy; kết nối lại thì gắn vào lại, thấy đúng màn hình đang dở.

```text
an@toinoi-app-01:~$ tmux new-session -d -s xuat 'xuat-chuyen 600'
an@toinoi-app-01:~$ tmux ls
xuat: 1 windows (created Tue Nov 24 07:05:41 2026)
```

- **`tmux new-session -d -s xuat 'LỆNH'`**: tạo phiên tên `xuat`, chạy lệnh trong đó, `-d` là tách ngay (không gắn vào). Gõ tay thì thường là `tmux new -s xuat`, làm việc bên trong, rồi tách ra.
- **`tmux ls`**: các phiên đang có.

An lại đóng tab phiên, kết nối lại:

```text
an@toinoi-app-01:~$ tmux ls
xuat: 1 windows (created Tue Nov 24 07:05:41 2026)
an@toinoi-app-01:~$ tmux capture-pane -p -t xuat | grep -v '^$' | tail -n 3
07:05:43 da xuat 10/600 chuyen
07:05:45 da xuat 20/600 chuyen
```

Phiên `xuat` vẫn còn, tiến độ vẫn tăng. **`tmux capture-pane -p`** in nội dung đang hiển thị trong phiên mà không cần gắn vào, tiện cho script. Gắn vào để xem và làm việc tiếp:

```bash
tmux attach -t xuat
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-job-nen-nohup-tmux/ssh-studio-03.webp" alt="Phiên tmux xuat đang gắn trong terminal của SSH Studio: các dòng tiến độ da xuat …/600 chuyen tiếp tục in ra, thanh trạng thái màu xanh của tmux ở dưới với tên phiên xuat" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>tmux attach</code>: quay lại đúng màn hình đang dở sau khi SSH đứt</figcaption></figure>

Trong tmux, mọi phím tắt bắt đầu bằng **tiền tố Ctrl+B**, nhả ra rồi bấm phím lệnh:

| Phím | Tác dụng |
|---|---|
| `Ctrl+B` `d` | Tách khỏi phiên (*detach*), phiên vẫn chạy |
| `Ctrl+B` `c` | Mở cửa sổ mới trong phiên |
| `Ctrl+B` `n` / `p` | Sang cửa sổ sau / trước |
| `Ctrl+B` `%` / `"` | Chia đôi màn hình dọc / ngang |
| `Ctrl+B` `[` | Cuộn lên xem lại (thoát bằng `q`) |

Thoát hẳn chương trình trong phiên (hoặc `exit`) thì phiên tmux kết thúc.

```text
an@toinoi-app-01:~$ tmux kill-session -t xuat
an@toinoi-app-01:~$ tmux ls
no server running on /tmp/tmux-1000/default
an@toinoi-app-01:~$ rm -f xuat-1.log xuat-2.log xuat-3.log nohup.out
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-job-nen-nohup-tmux/ssh-studio-04.webp" alt="Terminal của SSH Studio: tmux kill-session -t xuat, tmux ls báo no server running on /tmp/tmux-1000/default, rm dọn các file log thử" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · dọn phiên tmux</figcaption></figure>

## 4. Chọn cách nào

| Tình huống | Dùng |
|---|---|
| Lệnh chạy vài giây, muốn làm việc khác trong lúc chờ | `&`, `jobs`, `fg` |
| Việc chạy một lần, lâu, không cần tương tác, chỉ cần kết quả trong file | `nohup LỆNH > file 2>&1 &` |
| Việc chạy lâu muốn theo dõi, có thể phải trả lời, hoặc nhiều cửa sổ làm việc trên server | `tmux` |
| Việc chạy **định kỳ** hoặc **mãi mãi** (dịch vụ) | Không phải những cách trên: systemd ([LNX-035](/articles/linux-systemd-systemctl)), timer ([LNX-038](/articles/linux-cron-systemd-timer)) |

Có một bẫy với `tmux`: phiên tmux chạy dưới tên **bạn**. Nếu nhóm dựa vào «việc Bình để chạy trong tmux», việc đó mất khi Bình đăng xuất có dọn dẹp, server khởi động lại, hoặc tài khoản Bình bị khoá. Việc quan trọng thì cho thành dịch vụ.

## 5. Lỗi thường gặp

**Chạy `nohup LỆNH &` mà quên chuyển hướng.** Output đổ vào `nohup.out` ở thư mục hiện tại, có khi lớn dần không ai để ý.

**Tưởng `&` là đủ.** Job nền vẫn chết theo phiên. Ngược lại, `exit` bình thường thì mặc định bash **không** gửi `SIGHUP` (trừ khi bật `shopt -s huponexit`), nên đôi khi «có vẻ sống» khi bạn thoát đúng cách, rồi lại chết khi mạng đứt.

**Gắn vào tmux từ hai nơi rồi bối rối.** Hai cửa sổ thấy cùng một màn hình, gõ ở đâu cũng vào cùng chỗ. `tmux attach -d` gắn vào và tách nơi kia ra.

**Lồng tmux trong tmux.** Phím `Ctrl+B` đi tới phiên ngoài. Kiểm tra biến `$TMUX` trước khi `tmux new`.

## Bài tập

1. Chạy `xuat-chuyen 60`, bấm Ctrl+Z, rồi `bg`, `jobs`. Rồi `fg` và Ctrl+C. Mỗi bước gửi tín hiệu gì?
2. Thử `setsid xuat-chuyen 600 > x.log 2>&1 &` rồi đóng tab. Nó có sống không? So với `nohup`, `setsid` làm khác gì (gợi ý: `ps -o sid,pgid,cmd`)?
3. Trong tmux, tạo hai cửa sổ: một chạy `top`, một để gõ lệnh. Tách ra, kết nối lại, gắn vào. Cả hai còn không?
4. Dùng `tmux send-keys -t xuat 'echo xin chao' Enter` để gõ vào một phiên đang tách. Khi nào cách này hữu ích?

## Đọc tiếp

[LNX-035 · systemd và systemctl](/articles/linux-systemd-systemctl): cách đúng để một chương trình chạy mãi, tự khởi động cùng server, và có người trông.
