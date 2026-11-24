# Server Linux của lab "Linux qua SSH Studio và Tới Nơi".
# Ubuntu 24.04 LTS chạy systemd làm PID 1 (như một server thật), sshd là một service.
FROM ubuntu:24.04@sha256:9c7f200c07e3d4a03e7249d2c16bf66a51120fb876aa2242cf31451412543289

ENV DEBIAN_FRONTEND=noninteractive

# Ảnh Ubuntu cho Docker bị "minimize": không có trang man. Bỏ cấu hình loại trừ và bản man
# giả TRƯỚC khi cài gói, để `man ls` dùng được như trên server thường.
RUN rm -f /etc/dpkg/dpkg.cfg.d/excludes \
 && (dpkg-divert --quiet --remove --rename /usr/bin/man || true) \
 && apt-get update \
 && apt-get install -y --no-install-recommends \
      systemd systemd-sysv dbus openssh-server sudo tzdata locales \
      man-db manpages less vim bash-completion \
      procps psmisc iproute2 iputils-ping curl ca-certificates \
      tree file acl cron rsync tar gzip unzip gawk \
 && apt-get install -y --reinstall coreutils bash grep sed findutils util-linux passwd login \
 && rm -rf /var/lib/apt/lists/*

# Múi giờ và locale: tên file và nội dung tiếng Việt hiển thị đúng.
RUN ln -sf /usr/share/zoneinfo/Asia/Ho_Chi_Minh /etc/localtime \
 && echo "Asia/Ho_Chi_Minh" > /etc/timezone \
 && update-locale LANG=C.UTF-8
ENV LANG=C.UTF-8

# systemd trong container: bỏ các unit không có ý nghĩa ở đây.
RUN systemctl mask systemd-udevd.service systemd-udevd-kernel.socket systemd-udevd-control.socket \
      systemd-modules-load.service sys-kernel-config.mount sys-kernel-debug.mount \
      sys-kernel-tracing.mount getty.target console-getty.service \
      systemd-logind.service \
 && systemctl enable ssh.service cron.service \
 && systemctl disable ssh.socket 2>/dev/null || true

# Người dùng và nhóm của lab. Mật khẩu chỉ dùng cho lab, cổng SSH chỉ mở ở 127.0.0.1.
ARG LAB_PASSWORD=toinoi-lab
RUN userdel -r ubuntu 2>/dev/null || true \
 && groupadd --gid 2000 toinoi \
 && useradd --create-home --uid 1000 --shell /bin/bash --comment "Nguyen Van An" an \
 && useradd --create-home --uid 1001 --shell /bin/bash --comment "Tran Thi Binh" binh \
 && useradd --system --uid 990 --gid toinoi --home-dir /opt/toinoi/bao-gia --shell /usr/sbin/nologin baogia \
 && usermod -aG sudo an \
 && install -d -m 700 -o an -g an /home/an/.cache \
 && usermod -aG sudo,toinoi binh \
 && echo "an:${LAB_PASSWORD}" | chpasswd \
 && echo "binh:${LAB_PASSWORD}" | chpasswd

# sshd: đăng nhập bằng mật khẩu (lab), không cho root đăng nhập. Host key sinh lúc build
# nên fingerprint giữ nguyên khi tạo lại container từ cùng một ảnh.
RUN mkdir -p /run/sshd \
 && ssh-keygen -A \
 && printf '%s\n' \
      'PasswordAuthentication yes' \
      'KbdInteractiveAuthentication no' \
      'PermitRootLogin no' \
      'PrintMotd no' \
      > /etc/ssh/sshd_config.d/10-toinoi-lab.conf \
 && rm -f /etc/update-motd.d/* /etc/legal \
 && sed -i 's/^\(session\s\+optional\s\+pam_motd.so\s\+motd=\/run\/motd.dynamic\)/# \1/' /etc/pam.d/sshd

COPY fixtures/motd /etc/motd

# Dữ liệu mẫu: dịch vụ bao-gia và log của nó. Thời điểm sửa đổi cố định để `ls -l` in giống nhau.
COPY fixtures/bang-gia.csv fixtures/appsettings.json fixtures/kiem-tra-cau-hinh.sh fixtures/bi-mat.env /opt/toinoi/bao-gia/
COPY fixtures/bao-gia.log /var/log/toinoi/bao-gia.log
COPY fixtures/huong-dan.txt /home/an/huong-dan.txt
RUN chown -R baogia:toinoi /opt/toinoi /var/log/toinoi \
 && chmod 755 /opt/toinoi /opt/toinoi/bao-gia \
 && chmod 644 /opt/toinoi/bao-gia/bang-gia.csv /opt/toinoi/bao-gia/appsettings.json \
 && chmod 644 /opt/toinoi/bao-gia/kiem-tra-cau-hinh.sh \
 && chmod 600 /opt/toinoi/bao-gia/bi-mat.env \
 && chmod 755 /var/log/toinoi && chmod 644 /var/log/toinoi/bao-gia.log \
 && chown an:an /home/an/huong-dan.txt && chmod 644 /home/an/huong-dan.txt \
 && touch -d '2026-11-20 09:15:00' /opt/toinoi/bao-gia/bang-gia.csv /opt/toinoi/bao-gia/appsettings.json \
 && touch -d '2026-11-20 09:20:00' /opt/toinoi/bao-gia/kiem-tra-cau-hinh.sh /opt/toinoi/bao-gia/bi-mat.env \
 && touch -d '2026-11-21 12:17:05' /var/log/toinoi/bao-gia.log \
 && touch -d '2026-11-20 08:30:00' /home/an/huong-dan.txt \
 && touch -d '2026-11-20 09:20:00' /opt/toinoi/bao-gia /opt/toinoi /var/log/toinoi

# ---- Phần 2 ----
# Thêm ở cuối file: các lớp phía trên giữ nguyên trong cache của Docker, nên host key (sinh ở
# lớp ssh-keygen) không đổi với người đã dựng lab từ Phần 1.
RUN apt-get update \
 && apt-get install -y --no-install-recommends zstd xz-utils shellcheck \
 && rm -rf /var/lib/apt/lists/*
# PATH giống nhau cho phiên SSH và phiên `su -` của runner (hai bên mặc định khác nhau).
RUN printf '%s\n' 'PATH="/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"' 'export PATH' \
      > /etc/profile.d/00-toinoi-path.sh
# Log của 19 và 20/11 cho Phần 2; `chuan-bi.sh` đưa chúng vào /var/log/toinoi ở đầu LNX-011.
COPY fixtures/p02/ /usr/share/toinoi-lab/p02/
RUN cd /usr/share/toinoi-lab/p02 \
 && gzip -n -9 bao-gia-2026-11-19.log \
 && chmod 755 chuan-bi.sh \
 && chmod 644 bao-gia-2026-11-20.log bao-gia-2026-11-19.log.gz

# ---- Phần 3 ----
# PAM cho mã OTP (LNX-029) và oathtool để tính mã. Dữ liệu của lab nằm trong
# /usr/share/toinoi-lab/p03, chỉ root đọc được: key riêng tư ở đây là "key trên laptop"
# của An và của hệ thống CI, server thật không bao giờ giữ chúng.
RUN apt-get update \
 && apt-get install -y --no-install-recommends libpam-google-authenticator oathtool \
 && rm -rf /var/lib/apt/lists/*
COPY fixtures/p03/ /usr/share/toinoi-lab/p03/
RUN chmod 700 /usr/share/toinoi-lab/p03 \
 && chmod 600 /usr/share/toinoi-lab/p03/* \
 && chmod 700 /usr/share/toinoi-lab/p03/askpass-*

# ---- Phần 4 ----
# python3 để chạy bao-gia 1.5.0 (chỉ dùng thư viện chuẩn), logrotate như mọi server Ubuntu
# (ảnh Docker không có sẵn). jq và tmux KHÔNG cài sẵn: LNX-033 cài chúng bằng apt.
# apt in tiến độ dạng thường (không thanh tiến độ màu) để bản ghi của lab đọc được.
RUN apt-get update \
 && apt-get install -y --no-install-recommends python3-minimal logrotate \
 && rm -rf /var/lib/apt/lists/* \
 && printf '%s\n' 'Dpkg::Progress-Fancy "0";' 'APT::Color "0";' > /etc/apt/apt.conf.d/99-toinoi-lab
COPY fixtures/p04/ /usr/share/toinoi-lab/p04/
RUN chmod 755 /usr/share/toinoi-lab/p04/chuan-bi.sh

EXPOSE 22
STOPSIGNAL SIGRTMIN+3
CMD ["/sbin/init"]
