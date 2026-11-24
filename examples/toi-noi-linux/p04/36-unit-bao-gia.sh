# LNX-036 · Viết unit systemd cho bao-gia
#gui ket-noi-lai
#clear
## Bản 1.5.0 và cấu hình
ls -l /srv/toinoi/bao-gia/releases/1.5.0
head -n 3 /srv/toinoi/bao-gia/releases/1.5.0/bao-gia
readlink /srv/toinoi/bao-gia/current
#file /srv/toinoi/bao-gia/shared/config/bao-gia.env bao-gia.env
ls -l /srv/toinoi/bao-gia/shared/config
#shot 05
## File unit
#sudofile sudoedit /etc/systemd/system/bao-gia.service bao-gia.service 01
sudo systemctl daemon-reload
systemctl cat bao-gia --no-pager
#shot 06
#clear
## Lần khởi động đầu tiên
id -nG
sudo systemctl start bao-gia
systemctl status bao-gia --no-pager | head -n 7
sudo systemctl stop bao-gia
journalctl -u bao-gia --no-pager -n 4 -o cat
#shot 02
#clear
## Sửa: chuyển current sang 1.5.0
ln -sfn releases/1.5.0 /srv/toinoi/bao-gia/current
readlink /srv/toinoi/bao-gia/current
sudo systemctl start bao-gia
#wait 1
systemctl status bao-gia --no-pager
ps -o pid,user,group,cmd -p $(systemctl show bao-gia -p MainPID --value)
curl -s 'http://127.0.0.1:8080/bao-gia?dich_vu=car4&km=12.5'
#shot 03
#clear
## Bật lúc boot, tự khởi động lại khi chết
sudo systemctl enable bao-gia
systemctl show bao-gia --no-pager -p MainPID -p NRestarts
sudo kill -KILL $(systemctl show bao-gia -p MainPID --value)
#wait 4
systemctl show bao-gia --no-pager -p MainPID -p NRestarts
journalctl -u bao-gia --no-pager -n 6 -o cat
#shot 07
#clear
## reload: gửi SIGHUP
sudo systemctl reload bao-gia
journalctl -u bao-gia --no-pager -n 2 -o cat
#shot 04
