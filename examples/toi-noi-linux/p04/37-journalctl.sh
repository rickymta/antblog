# LNX-037 · journalctl: đọc log của dịch vụ
#clear
## Log của một unit
journalctl -u bao-gia --no-pager | head -n 8
journalctl -u bao-gia --no-pager -n 3
journalctl -u bao-gia --no-pager --since '5 min ago' -o short-iso | tail -n 3
## Mức ưu tiên
curl -s 'http://127.0.0.1:8080/bao-gia?dich_vu=xe-tai&km=5'
curl -s 'http://127.0.0.1:8080/bao-gia?dich_vu=bike&km=abc'
journalctl -u bao-gia --no-pager -p warning -o short-iso
#shot 01
#clear
## Một lỗi thật: bảng giá biến mất khi reload
mv /srv/toinoi/bao-gia/releases/1.5.0/bang-gia.csv /tmp/bang-gia.csv
sudo systemctl reload bao-gia
#wait 1
journalctl -u bao-gia --no-pager -p err -n 1 -o short-iso
curl -s http://127.0.0.1:8080/suc-khoe
mv /tmp/bang-gia.csv /srv/toinoi/bao-gia/releases/1.5.0/bang-gia.csv
sudo systemctl reload bao-gia
#wait 1
journalctl -u bao-gia --no-pager -n 2 -o cat
## Một bản ghi có những trường gì
journalctl --no-pager -n 1 -o json-pretty SYSLOG_IDENTIFIER=bao-gia | grep -E '"(MESSAGE|PRIORITY|_PID|_UID|_SYSTEMD_UNIT|_COMM|SYSLOG_IDENTIFIER)"'
journalctl --no-pager _UID=990 -n 2 -o cat
#shot 02
#clear
## Theo dõi trực tiếp
(sleep 2; curl -s -o /dev/null 'http://127.0.0.1:8080/bao-gia?dich_vu=tau-thuy&km=1') &
timeout 4 journalctl -u bao-gia -f -n 0 -o short-iso
#gui theo-doi-journal
#clear
## Dung lượng và giới hạn của journal
journalctl --disk-usage
ls /var/log/journal/
journalctl --list-boots --no-pager | tail -n 2
sudo mkdir -p /etc/systemd/journald.conf.d
#sudofile sudoedit /etc/systemd/journald.conf.d/50-toinoi.conf journald-toinoi.conf
sudo systemctl restart systemd-journald
systemd-analyze cat-config systemd/journald.conf | grep -v '^#' | grep -v '^$'
sudo journalctl --vacuum-time=30d
#shot 04
