# LNX-027 · ACL: quyền chi tiết hơn chủ sở hữu, nhóm và người khác
#clear
## Khi ba lớp quyền không đủ
sudo useradd --create-home --shell /bin/bash --comment "Kiem toan noi bo" kiemtoan
id kiemtoan
ls -ld /srv/toinoi/bao-gia/shared/logs
sudo -u kiemtoan ls /srv/toinoi/bao-gia/shared/logs
getfacl -p /srv/toinoi/bao-gia/shared/logs
#shot 04
#clear
## Cấp quyền cho đúng một người
setfacl -m u:kiemtoan:rx /srv/toinoi/bao-gia/shared/logs
getfacl -p /srv/toinoi/bao-gia/shared/logs
ls -ld /srv/toinoi/bao-gia/shared/logs
sudo -u kiemtoan ls /srv/toinoi/bao-gia/shared/logs
sudo -u kiemtoan head -n 3 /srv/toinoi/bao-gia/shared/logs/loi-theo-dich-vu.txt
sudo -u kiemtoan touch /srv/toinoi/bao-gia/shared/logs/thu.txt
#shot 01
#clear
## ACL mặc định cho file mới
setfacl -d -m u:kiemtoan:r /srv/toinoi/bao-gia/shared/logs
touch /srv/toinoi/bao-gia/shared/logs/moi.txt
getfacl -p /srv/toinoi/bao-gia/shared/logs/moi.txt
ls -l /srv/toinoi/bao-gia/shared/logs/moi.txt
#shot 05
#clear
## mask giới hạn mọi quyền ACL
chmod 600 /srv/toinoi/bao-gia/shared/logs/moi.txt
getfacl -p /srv/toinoi/bao-gia/shared/logs/moi.txt
sudo -u kiemtoan cat /srv/toinoi/bao-gia/shared/logs/moi.txt
rm /srv/toinoi/bao-gia/shared/logs/moi.txt
#shot 02
#clear
## Gỡ ACL
setfacl -k /srv/toinoi/bao-gia/shared/logs
setfacl -x u:kiemtoan /srv/toinoi/bao-gia/shared/logs
getfacl -p /srv/toinoi/bao-gia/shared/logs
setfacl -b /srv/toinoi/bao-gia/shared/logs
ls -ld /srv/toinoi/bao-gia/shared/logs
sudo -u kiemtoan ls /srv/toinoi/bao-gia/shared/logs
#shot 03
#xem files /srv/toinoi/bao-gia/shared/logs 06
