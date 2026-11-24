# LNX-050 · Thực hành: mạng và giám sát cho toinoi-app-01
#clear
## 1. Tường lửa: chỉ những gì cần
sudo ufw status verbose
sudo ss -tlnpH | grep -v 127.0.0 | awk '{ split($6, p, "\""); print $4, p[2] }'
#may app02
nc -zv -w 3 toinoi-app-01 22
nc -zv -w 3 toinoi-app-01 443
nc -zv -w 3 toinoi-app-01 80
#may app01
#shot 01
#clear
## 2. Báo cáo tài nguyên định kỳ
#sudofile sudoedit /usr/local/bin/bao-cao-tai-nguyen bao-cao-tai-nguyen 02
sudo chmod 755 /usr/local/bin/bao-cao-tai-nguyen
bao-cao-tai-nguyen
#sudofile sudoedit /etc/systemd/system/bao-cao-tai-nguyen.service bao-cao-tai-nguyen.service
#sudofile sudoedit /etc/systemd/system/bao-cao-tai-nguyen.timer bao-cao-tai-nguyen.timer
sudo systemctl daemon-reload
sudo systemctl enable --now bao-cao-tai-nguyen.timer
sudo systemctl start bao-cao-tai-nguyen.service
cat /var/log/toinoi/tai-nguyen.log
systemctl list-timers bao-cao-tai-nguyen.timer --no-pager | head -n 2
#shot 03
#clear
## 3. Bảng kiểm tra bàn giao
findmnt -o TARGET,SOURCE,FSTYPE /du-lieu
grep du-lieu /etc/fstab
sudo nginx -t 2>&1 | tail -n 1
curl -s --cacert /etc/ssl/certs/bao-gia.toinoi.lab.crt https://bao-gia.toinoi.lab/suc-khoe
ssh app02 'ls sao-luu/releases'
systemctl is-active bao-gia nginx ufw bao-cao-tai-nguyen.timer
#shot 04
