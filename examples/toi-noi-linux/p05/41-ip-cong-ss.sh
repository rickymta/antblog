# LNX-041 · Địa chỉ IP, cổng và ss: dịch vụ đang nghe ở đâu
#root /usr/share/toinoi-lab/p05/chuan-bi.sh
#clear
## Địa chỉ của server
hostname
ip -br addr
#shot 04
#clear
ip route
hostname -I
grep toinoi /etc/hosts
#shot 01
#clear
## Dịch vụ đang nghe ở đâu
ss -tln
sudo ss -tlnp
curl -s http://127.0.0.1:8080/suc-khoe
curl -sS --connect-timeout 3 http://10.20.0.11:8080/suc-khoe
#shot 05
#clear
## 127.0.0.1 hay 0.0.0.0
cd /srv/toinoi/bao-gia/shared/logs
nohup python3 -m http.server 8081 --bind 0.0.0.0 > /dev/null 2>&1 &
cd ~
#wait 1
sudo ss -tlnp | grep -E ':(22|8080|8081) '
#shot 02
#may app02
hostname
nc -zv -w 3 toinoi-app-01 22
nc -zv -w 3 toinoi-app-01 8080
nc -zv -w 3 toinoi-app-01 8081
curl -s http://toinoi-app-01:8081/ | grep -o 'href="[^"]*"' | head -n 3
#may app01
#clear
## Các kết nối đang mở
ss -tn
#shot 06
#clear
sudo ss -tnp state established | grep -v 127.0.0.11
pkill -f 'http.server 8081'
sudo ss -tlnp | grep -c ':8081 '
#shot 03
