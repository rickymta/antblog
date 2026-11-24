# LNX-035 · systemd: systemctl và trạng thái dịch vụ
#clear
## systemd là PID 1
ps -o pid,cmd -p 1
systemctl --version | head -n 1
systemctl list-units --type=service --state=running --no-pager
#shot 04
#clear
## Đọc status
systemctl status cron --no-pager
#shot 01
#clear
## Hỏi nhanh: is-active, is-enabled, show
systemctl is-active cron ssh
systemctl is-enabled cron ssh
systemctl show cron --no-pager -p MainPID -p ActiveState -p SubState -p Restart -p NRestarts
systemctl cat cron --no-pager
#shot 05
#clear
## Dừng, khởi động, bật lúc boot
sudo systemctl stop cron
systemctl is-active cron
systemctl status cron --no-pager | head -n 3
#shot 06
#clear
sudo systemctl start cron
sudo systemctl disable cron
ls /etc/systemd/system/multi-user.target.wants/
sudo systemctl enable --now cron
systemctl is-enabled cron
#shot 02
#clear
## Phụ thuộc giữa các unit
systemctl list-dependencies ssh --no-pager | head -n 8
systemctl show ssh --no-pager -p After -p Wants | cut -c1-100
systemctl --failed --no-pager
#shot 07
#clear
systemctl list-unit-files --type=timer --no-pager | head -n 6
## Ai được đọc journal?
journalctl -u cron -n 2 --no-pager
id -nG
sudo usermod -aG systemd-journal an
#shot 03
