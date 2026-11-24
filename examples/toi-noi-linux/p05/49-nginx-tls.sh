# LNX-049 · TLS và reverse proxy nginx cho bao-gia
#clear
## Cài nginx
sudo apt update 2>&1 | tail -n 1
sudo apt install -y nginx 2>&1 | grep -E '^Setting up nginx'
systemctl is-active nginx
sudo ss -tlnpH | grep nginx | awk '{ split($6, p, "\""); print $4, p[2] }'
ps -C nginx -o user=,cmd= | sort | uniq -c
curl -s -o /dev/null -w '%{http_code} %{content_type}\n' http://127.0.0.1/
#shot 01
#clear
## Chứng chỉ tự ký cho lab
sudo openssl req -x509 -newkey rsa:2048 -nodes -days 365 -subj '/CN=bao-gia.toinoi.lab' -addext 'subjectAltName=DNS:bao-gia.toinoi.lab' -keyout /etc/ssl/private/bao-gia.toinoi.lab.key -out /etc/ssl/certs/bao-gia.toinoi.lab.crt 2>/dev/null
sudo ls -l /etc/ssl/private/bao-gia.toinoi.lab.key /etc/ssl/certs/bao-gia.toinoi.lab.crt
openssl x509 -in /etc/ssl/certs/bao-gia.toinoi.lab.crt -noout -subject -issuer -dates
## Cấu hình site
#sudofile sudoedit /etc/nginx/sites-available/bao-gia nginx-bao-gia 02
sudo ln -s /etc/nginx/sites-available/bao-gia /etc/nginx/sites-enabled/bao-gia
sudo rm /etc/nginx/sites-enabled/default
sudo nginx -t
sudo systemctl reload nginx
#wait 1
#clear
## Thử từ chính server
echo '127.0.0.1 bao-gia.toinoi.lab' | sudo tee -a /etc/hosts
curl -sI http://bao-gia.toinoi.lab/suc-khoe | head -n 3
curl -sS https://bao-gia.toinoi.lab/suc-khoe
curl -s --cacert /etc/ssl/certs/bao-gia.toinoi.lab.crt https://bao-gia.toinoi.lab/suc-khoe
echo | openssl s_client -connect bao-gia.toinoi.lab:443 -servername bao-gia.toinoi.lab 2>/dev/null | grep -E '^(subject|issuer)=|Protocol|Cipher is'
sudo tail -n 2 /var/log/nginx/bao-gia.access.log
#shot 03
#clear
## Mở 443 cho mạng nội bộ, thử từ toinoi-app-02
sudo ufw allow from 10.20.0.0/24 to any port 443 proto tcp comment 'https bao-gia noi bo'
sudo ufw status | grep 443
#may app02
curl -sk --resolve bao-gia.toinoi.lab:443:10.20.0.11 https://bao-gia.toinoi.lab/suc-khoe
curl -sS --connect-timeout 3 http://toinoi-app-01:8080/suc-khoe
#may app01
sudo tail -n 1 /var/log/nginx/bao-gia.access.log
#shot 04
