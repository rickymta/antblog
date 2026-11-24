# LNX-045 · Tường lửa với ufw và nftables
#clear
## Trạng thái ban đầu
sudo ufw status
sudo nft list ruleset | wc -l
sudo ufw app list
sudo ufw app info OpenSSH
## Quy tắc trước, bật sau
sudo ufw default deny incoming
sudo ufw default allow outgoing
sudo ufw limit OpenSSH
sudo ufw allow from 10.20.0.0/24 to any port 8081 proto tcp comment 'thu muc log noi bo'
sudo ufw show added
#shot 01
#clear
## Bật tường lửa: phiên SSH đang mở không bị cắt
sudo ufw --force enable
sudo ufw status verbose
sudo ufw status numbered
#shot 02
#clear
## Thử từ máy khác trong mạng
cd /srv/toinoi/bao-gia/shared/logs
nohup python3 -m http.server 8081 --bind 0.0.0.0 > /dev/null 2>&1 &
nohup python3 -m http.server 8082 --bind 0.0.0.0 > /dev/null 2>&1 &
cd ~
#wait 1
#may app02
nc -zv -w 3 toinoi-app-01 22
nc -zv -w 3 toinoi-app-01 8081
nc -zv -w 3 toinoi-app-01 8082
#may app01
## Sửa và xoá quy tắc
sudo ufw status numbered | grep 8081
sudo ufw delete allow from 10.20.0.0/24 to any port 8081 proto tcp
sudo ufw status numbered
pkill -f 'http.server 808[12]'
## ufw chỉ là lớp vỏ: quy tắc thật nằm trong nftables
sudo nft list ruleset | wc -l
sudo nft list chain ip filter ufw-user-input
#shot 03
