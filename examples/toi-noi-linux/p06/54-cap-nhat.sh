# LNX-054 · Cập nhật hệ thống an toàn: bản vá, khởi động lại, unattended-upgrades
#clear
## Có gì cần cập nhật
sudo apt update 2>&1 | tail -n 1
apt list --upgradable 2>/dev/null | tail -n +2 | wc -l
apt list --upgradable 2>/dev/null | grep -c -- '-security'
apt list --upgradable 2>/dev/null | head -n 6
#shot 01
#clear
## Vá tự động: unattended-upgrades
sudo apt install -y unattended-upgrades 2>&1 | grep -E '^Setting up unattended-upgrades'
#sudofile sudoedit /etc/apt/apt.conf.d/20auto-upgrades 20auto-upgrades 02
#sudofile sudoedit /etc/apt/apt.conf.d/52unattended-toinoi 52unattended-toinoi
grep -E '^\s*"\$\{distro_id\}' /etc/apt/apt.conf.d/50unattended-upgrades
#shot 07
#clear
sudo unattended-upgrade --dry-run -v 2>&1 | tail -n 3
systemctl list-timers 'apt-daily*' --no-pager | head -n 3
#shot 03
#clear
## Có cần khởi động lại không
ls /var/run/reboot-required 2>/dev/null || echo "chua can khoi dong lai"
cat /var/run/reboot-required.pkgs 2>/dev/null || true
#shot 05
#clear
## Quay lui một bản vá: nginx
apt-cache madison nginx
sudo apt install -y --allow-downgrades nginx=1.24.0-2ubuntu7 nginx-common=1.24.0-2ubuntu7 2>&1 | grep -E '^(Setting up|dpkg: warning)'
nginx -v
sudo apt-mark hold nginx nginx-common
apt list --upgradable 2>/dev/null | grep nginx
sudo nginx -t 2>&1 | tail -n 1
#shot 04
#clear
## Bản vá đã ổn: bỏ giữ, nâng lại
sudo apt-mark unhold nginx nginx-common
sudo apt install -y nginx nginx-common 2>&1 | grep -E '^Setting up nginx '
nginx -v
#shot 06
