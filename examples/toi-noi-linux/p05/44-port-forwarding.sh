# LNX-044 · Port forwarding trong SSH Studio: Local, Remote, Dynamic
#clear
## bao-gia chỉ nghe ở 127.0.0.1
sudo ss -tlnp | grep -E ':(8080|22) '
#shot 01
#gui forward-local
#xem servers 07
## Phía server thấy gì khi có đường hầm
sudo ss -tnp | grep ':8080' | grep -v -E 'TIME-WAIT'
#gui forward-xong
#xem nhat-ky 08
#may app02
## Mô phỏng laptop của An: một kết nối SSH mở cả ba kiểu đường hầm
mkdir -p ~/may-an && echo 'trang thu tu laptop cua An' > ~/may-an/index.html
nohup python3 -m http.server 8000 --bind 127.0.0.1 --directory ~/may-an > /dev/null 2>&1 &
#root SSH_ASKPASS=/usr/share/toinoi-lab/p03/askpass-otp SSH_ASKPASS_REQUIRE=force DISPLAY=lab setsid -f ssh -i /usr/share/toinoi-lab/p03/an-laptop -o IdentitiesOnly=yes -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o LogLevel=ERROR -o ExitOnForwardFailure=yes -N -L 18080:127.0.0.1:8080 -R 9090:127.0.0.1:8000 -D 1080 an@toinoi-app-01 < /dev/null
#wait 4
ss -tln | grep -E ':(18080|1080|8000) '
curl -s http://127.0.0.1:18080/suc-khoe
curl -s --socks5-hostname 127.0.0.1:1080 http://127.0.0.1:8080/suc-khoe
curl -s --socks5-hostname 127.0.0.1:1080 -o /dev/null -w '%{http_code}\n' http://toinoi-app-01:8080/suc-khoe
#may app01
## Remote: server mở cổng, kết nối quay về laptop
sudo ss -tlnp | grep ':9090 '
curl -s http://127.0.0.1:9090/
## deploy không được mở đường hầm
sudo sshd -T -C user=deploy,host=ci,addr=10.0.0.5 | grep allowtcpforwarding
#shot 06
#may app02
#root pkill -f 'ssh -i /usr/share/toinoi-lab/p03/an-laptop'
pkill -f 'http.server 8000'
