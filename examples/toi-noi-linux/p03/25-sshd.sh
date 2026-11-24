# LNX-025 · Cấu hình sshd: tắt mật khẩu, tắt root, giới hạn người dùng
#clear
## Cấu hình đang có hiệu lực
sudo sshd -T | grep -E '^(permitrootlogin|pubkeyauthentication|passwordauthentication|kbdinteractiveauthentication|maxauthtries|allowgroups|x11forwarding) '
grep -v -E '^(#|$)' /etc/ssh/sshd_config
#shot 07
#clear
ls /etc/ssh/sshd_config.d
cat /etc/ssh/sshd_config.d/10-toinoi-lab.conf
#shot 01
#clear
## Nhóm được phép SSH
sudo groupadd ssh-users
sudo usermod -aG ssh-users an
sudo usermod -aG ssh-users binh
getent group ssh-users
#shot 08
#clear
## Viết file cấu hình mới
#sudofile sudoedit /etc/ssh/sshd_config.d/20-an-toan.conf sshd-an-toan 02
sudo sshd -t && echo "cú pháp ổn"
sudo sshd -T | grep -E '^(passwordauthentication|allowgroups|maxauthtries) '
#shot 09
#clear
## Giá trị đầu tiên thắng
sudo mv /etc/ssh/sshd_config.d/20-an-toan.conf /etc/ssh/sshd_config.d/05-an-toan.conf
ls /etc/ssh/sshd_config.d
sudo sshd -t && echo "cú pháp ổn"
sudo sshd -T | grep -E '^(permitrootlogin|passwordauthentication|kbdinteractiveauthentication|maxauthtries|allowgroups|x11forwarding) '
#shot 03
#clear
## Áp dụng mà không tự khoá mình ngoài cửa
sudo systemctl reload ssh
#gui thu-phien-thu-hai
ssh -o BatchMode=yes an@localhost true
ssh -o BatchMode=yes cuong@localhost true
#login an an-laptop
sudo journalctl -u ssh --no-pager | grep -E 'not allowed because|Accepted publickey' | tail -n 3
#shot 06
