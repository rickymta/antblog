# LNX-020 · Thực hành: báo cáo và sao lưu log của bao-gia, chạy hằng ngày bằng cron
#clear
## 1. Đưa script vào thư mục dùng chung
mkdir -p /srv/toinoi/bao-gia/scripts
cp ~/bin/bao-cao-log ~/bin/sao-luu-log ~/bin/tom-tat.awk /srv/toinoi/bao-gia/scripts/
#file /srv/toinoi/bao-gia/scripts/hang-ngay hang-ngay 01
cd /srv/toinoi/bao-gia/scripts
chgrp toinoi . bao-cao-log sao-luu-log tom-tat.awk hang-ngay
chmod 750 . bao-cao-log sao-luu-log hang-ngay
chmod 640 tom-tat.awk
ls -la
shellcheck bao-cao-log sao-luu-log hang-ngay && echo "shellcheck: sạch"
#shot 02
#clear
## 2. Chạy thử với môi trường trống như của cron
env -i /usr/bin/env
env -i NGAY=2026-11-21 /srv/toinoi/bao-gia/scripts/hang-ngay
echo $?
env -i HOME=/home/an NGAY=2026-11-21 /srv/toinoi/bao-gia/scripts/hang-ngay
cat /srv/toinoi/bao-gia/shared/logs/bao-cao-2026-11-21.txt
ls -l /srv/toinoi/sao-luu
#shot 03
#clear
## 3. Lên lịch bằng cron: chạy thử mỗi phút
crontab -l
#file ~/hang-ngay.cron hang-ngay.cron
crontab ~/hang-ngay.cron
crontab -l
#wait 65
cat /srv/toinoi/bao-gia/shared/logs/hang-ngay.log
ls /srv/toinoi/sao-luu
#shot 04
## 4. Đổi sang lịch thật: 01:17 mỗi ngày
sed -i -e 's/^[*] [*] [*] [*] [*]/17 1 * * */' -e 's/Đang chạy thử mỗi phút/Chạy lúc 01:17 mỗi ngày/' ~/hang-ngay.cron
crontab ~/hang-ngay.cron
crontab -l
#shot 05
