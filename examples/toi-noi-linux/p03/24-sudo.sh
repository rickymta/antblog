# LNX-024 · sudo và sudoers: cấp quyền theo việc
#clear
## sudo của An đến từ đâu?
sudo -l
sudo grep -v -E '^(#|$)' /etc/sudoers
sudo ls -l /etc/sudoers.d
getent group sudo
#shot 01
#clear
## Cấp quyền theo việc cho Cường
#sudofile visudo /etc/sudoers.d/cuong sudoers-cuong 02
sudo ls -l /etc/sudoers.d
sudo visudo -c
sudo chmod 440 /etc/sudoers.d/cuong
sudo ls -l /etc/sudoers.d
sudo visudo -c
sudo -l -U cuong
## Thử bằng tài khoản của Cường
sudo -u cuong sudo -n /usr/bin/systemctl restart cron && echo "khởi động lại cron: được"
sudo -u cuong sudo -n /usr/bin/journalctl -u cron -n 2 --no-pager
sudo -u cuong sudo -n /usr/bin/cat /etc/shadow
sudo -u cuong sudo -n /usr/bin/systemctl stop cron
#shot 03
#clear
## Một lỗi cú pháp trong sudoers
#file ~/tai-lieu/sudoers-loi sudoers-loi
cat ~/tai-lieu/sudoers-loi
sudo visudo -c -f ~/tai-lieu/sudoers-loi
## sudo ghi lại mọi lần dùng
sudo journalctl _COMM=sudo --no-pager | grep -E 'cuong :' | tail -n 3
#shot 04
