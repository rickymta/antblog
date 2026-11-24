# LNX-040 · Thực hành: đưa bao-gia thành dịch vụ systemd hoàn chỉnh
#clear
## 1. Siết unit bằng drop-in
systemd-analyze security bao-gia --no-pager | tail -n 1
sudo mkdir -p /etc/systemd/system/bao-gia.service.d
#sudofile sudoedit /etc/systemd/system/bao-gia.service.d/50-an-toan.conf bao-gia-an-toan.conf 01
sudo systemctl daemon-reload
sudo systemctl restart bao-gia
#wait 1
systemctl status bao-gia --no-pager | head -n 5
systemd-analyze security bao-gia --no-pager | tail -n 1
systemctl show bao-gia --no-pager -p ProtectSystem -p ProtectHome -p PrivateTmp -p MemoryMax -p ReadWritePaths
#shot 02
#clear
## 2. Dịch vụ vẫn làm đúng việc của nó
curl -s http://127.0.0.1:8080/suc-khoe | jq -c .
curl -s 'http://127.0.0.1:8080/bao-gia?dich_vu=delivery&km=7'
tail -n 1 /var/log/toinoi/bao-gia.log
sudo systemctl reload bao-gia
journalctl -u bao-gia --no-pager -n 2 -o cat
## 3. CI khởi động lại bằng tài khoản deploy
sudo -u deploy sudo -n /usr/bin/systemctl restart bao-gia && echo "deploy: restart duoc"
sudo -u deploy sudo -n /usr/bin/systemctl stop bao-gia || echo "deploy: stop bi tu choi"
#wait 1
systemctl is-active bao-gia
#shot 03
#clear
## 4. Bảng kiểm tra trước khi bàn giao
systemctl is-enabled bao-gia bao-gia-don-bao-cao.timer logrotate.timer
systemctl is-active bao-gia bao-gia-don-bao-cao.timer logrotate.timer
systemctl show bao-gia --no-pager -p User -p Restart -p NRestarts
sudo logrotate -d /etc/logrotate.d/bao-gia 2>&1 | grep -ci error
journalctl --disk-usage
systemctl --failed --no-pager
#shot 04
