# LNX-038 · Lên lịch: cron và systemd timer
#clear
## Lịch đang có: cron
crontab -l | grep -v '^#'
grep -v '^#' /etc/crontab | grep -v '^$'
ls /etc/cron.daily/
## Lịch đang có: timer của systemd
systemctl list-timers --no-pager | cut -c1-120
#shot 01
#clear
## Việc mới: dọn báo cáo cũ hơn 14 ngày
cd /srv/toinoi/bao-gia/shared/logs
touch -d '2026-10-30 01:17' bao-cao-2026-10-30.txt
touch -d '2026-11-05 01:17' bao-cao-2026-11-05.txt
touch -d '2026-11-15 01:17' bao-cao-2026-11-15.txt
ls -l --time-style=+%F bao-cao-*.txt
#shot 05
cd ~
#sudofile sudoedit /etc/systemd/system/bao-gia-don-bao-cao.service don-bao-cao.service 02
#sudofile sudoedit /etc/systemd/system/bao-gia-don-bao-cao.timer don-bao-cao.timer 03
systemd-analyze calendar '*-*-* 02:30:00' | head -n 3
sudo systemd-analyze verify /etc/systemd/system/bao-gia-don-bao-cao.service /etc/systemd/system/bao-gia-don-bao-cao.timer
sudo systemctl daemon-reload
sudo systemctl enable --now bao-gia-don-bao-cao.timer
systemctl list-timers bao-gia-don-bao-cao.timer --no-pager | cut -c1-120
#shot 06
#clear
## Chạy thử ngay, không chờ 02:30
sudo systemctl start bao-gia-don-bao-cao.service
systemctl status bao-gia-don-bao-cao.service --no-pager | head -n 4
journalctl -u bao-gia-don-bao-cao --no-pager -o cat
ls /srv/toinoi/bao-gia/shared/logs
#shot 07
#clear
## Việc bị lỡ khi server tắt: Persistent=true
systemctl show bao-gia-don-bao-cao.timer --no-pager -p Persistent -p Unit -p TimersCalendar
sudo ls /var/lib/systemd/timers/
#shot 04
