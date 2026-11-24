# LNX-039 · logrotate và dọn log
#clear
## Log của bao-gia đang lớn dần
ls -l /var/log/toinoi/
for i in 1 2 3 4 5; do curl -s -o /dev/null "http://127.0.0.1:8080/bao-gia?dich_vu=bike&km=$i"; done
tail -n 2 /var/log/toinoi/bao-gia.log
## logrotate đã có sẵn
systemctl list-timers logrotate.timer --no-pager | cut -c1-110
grep -v '^#' /etc/logrotate.conf | grep -v '^$'
ls /etc/logrotate.d/
cat /etc/logrotate.d/apt
#shot 01
#clear
## Cấu hình cho bao-gia
#sudofile sudoedit /etc/logrotate.d/bao-gia logrotate-bao-gia 02
sudo logrotate -d /etc/logrotate.d/bao-gia 2>&1 | grep -E 'rotating pattern|log needs|log does not need|considering log|error' | head -n 6
#clear
## Xoay vòng ngay bây giờ
sudo logrotate -f -v /etc/logrotate.d/bao-gia 2>&1 | grep -E 'bao-gia\.log(\.1)? to|^(creating|compressing|running)' | cut -c1-90
ls -l /var/log/toinoi/
journalctl -u bao-gia --no-pager -n 2 -o cat
curl -s -o /dev/null 'http://127.0.0.1:8080/bao-gia?dich_vu=car7&km=3'
cat /var/log/toinoi/bao-gia.log
tail -n 1 /var/log/toinoi/bao-gia.log.1
zcat /var/log/toinoi/bao-gia.log.2.gz | head -n 1
sudo cat /var/lib/logrotate/status | grep toinoi
#shot 03
