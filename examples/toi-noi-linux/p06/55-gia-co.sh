# LNX-055 · Gia cố server: checklist tối thiểu và fail2ban
#clear
## Checklist: cửa nào đang mở, ai vào được
sudo ss -tlnpH | grep -v 127.0.0 | awk '{ split($6, p, "\""); print $4, p[2] }'
sudo ufw status | head -n 7
sudo sshd -T | grep -E '^(permitrootlogin|passwordauthentication|authenticationmethods|maxauthtries|x11forwarding|allowagentforwarding) '
awk -F: '$7 !~ /(nologin|false)$/ { print $1, $7 }' /etc/passwd
getent group sudo
systemctl list-unit-files --type=service --state=enabled --no-pager --no-legend | awk '{ print $1 }' | paste -sd ' '
#shot 01
#clear
## fail2ban: chặn IP dò mật khẩu
sudo apt install -y fail2ban 2>&1 | grep -E '^Setting up fail2ban'
#sudofile sudoedit /etc/fail2ban/jail.d/toinoi.local jail-toinoi.local 02
sudo systemctl restart fail2ban
sleep 3; sudo fail2ban-client status sshd
#may app02
#clear
## Từ app02: ai đó thử đăng nhập bằng tài khoản không tồn tại
for u in admin test oracle ubuntu; do ssh -o BatchMode=yes -o ConnectTimeout=3 -o StrictHostKeyChecking=no $u@toinoi-app-01 true 2>&1 | tail -n 1; done
sleep 2; nc -zv -w 3 toinoi-app-01 22
#may app01
#clear
## Trên app01: fail2ban đã chặn
#wait 3
sudo fail2ban-client status sshd
sudo ufw status | head -n 5
sudo grep -E 'Found|Ban' /var/log/fail2ban.log | tail -n 5
#shot 03
#clear
## Gỡ chặn
sudo fail2ban-client set sshd unbanip 10.20.0.12
sudo fail2ban-client status sshd | grep -E 'Currently banned|Banned IP'
