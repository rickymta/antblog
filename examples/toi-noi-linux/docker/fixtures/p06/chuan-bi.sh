#!/bin/sh
# Đầu Phần 6: mỗi máy của lab có danh tính riêng và nhận key của An. Chạy với quyền root trên
# từng máy (toinoi-app-01, toinoi-app-02, toinoi-bastion); chạy lại vẫn cho cùng kết quả.
set -eu
nguon=/usr/share/toinoi-lab/p06
may=$(hostname)

# Ba container dựng từ cùng một ảnh nên ban đầu dùng chung host key. Server thật thì mỗi máy một
# host key: app02 và bastion nhận host key riêng (cố định, để fingerprint trong bài giống nhau).
dat_host_key() {
  rm -f /etc/ssh/ssh_host_*
  install -m 600 "$nguon/ssh_host_ed25519_key_$1" /etc/ssh/ssh_host_ed25519_key
  install -m 644 "$nguon/ssh_host_ed25519_key_$1.pub" /etc/ssh/ssh_host_ed25519_key.pub
  systemctl restart ssh
}

# Cài một public key cho an trên máy này (không thêm trùng).
nhan_key() {
  install -d -m 700 -o an -g an /home/an/.ssh
  touch /home/an/.ssh/authorized_keys
  grep -qxF "$(cat "$1")" /home/an/.ssh/authorized_keys || cat "$1" >> /home/an/.ssh/authorized_keys
  chown an:an /home/an/.ssh/authorized_keys
  chmod 600 /home/an/.ssh/authorized_keys
}
# Key "trên laptop của An" (Phần 3).
nhan_key_an() { nhan_key /usr/share/toinoi-lab/p03/an-laptop.pub; }

case "$may" in
  toinoi-bastion)
    dat_host_key bastion
    nhan_key_an
    # Key đồng bộ của app01 (LNX-046): để app01 chạy lệnh và Ansible trên bastion (LNX-056).
    nhan_key /usr/share/toinoi-lab/p05/dong-bo.pub
    # Bastion chỉ để đi qua: chỉ nhận key, không mật khẩu, không chuyển tiếp agent hay X11.
    cat > /etc/ssh/sshd_config.d/10-bastion.conf <<'EOF'
# toinoi-bastion: cửa vào duy nhất từ ngoài vào mạng nội bộ 10.20.0.0/24 (Bình dựng, LNX-052).
PasswordAuthentication no
KbdInteractiveAuthentication no
PermitRootLogin no
AllowAgentForwarding no
X11Forwarding no
AllowTcpForwarding yes
EOF
    systemctl reload ssh
    ;;
  toinoi-app-02)
    dat_host_key app-02
    nhan_key_an
    ;;
  toinoi-app-01)
    # app01 đã ghi host key cũ (chung) của app02 ở LNX-046: thay bằng host key mới.
    kh=/home/an/.ssh/known_hosts
    if [ -f "$kh" ]; then
      ssh-keygen -R toinoi-app-02 -f "$kh" >/dev/null 2>&1 || true
      rm -f "$kh.old"
    fi
    printf 'toinoi-app-02 %s\n' "$(cut -d' ' -f1,2 "$nguon/ssh_host_ed25519_key_app-02.pub")" >> "$kh"
    chown an:an "$kh"
    chmod 600 "$kh"
    ;;
esac
