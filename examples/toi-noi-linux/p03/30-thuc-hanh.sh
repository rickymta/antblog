# LNX-030 · Thực hành: khoá chặt đăng nhập của toinoi-app-01
#clear
## 1. Tài khoản deploy cho CI
sudo useradd --create-home --shell /bin/bash --comment "Trien khai tu dong (CI)" --groups toinoi,ssh-users deploy
id deploy
sudo passwd -S deploy
sudo install -d -m 700 -o deploy -g deploy /home/deploy/.ssh
echo 'ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAICZUGM/RB5spPmpjCeCYkOgP1vCe9YV5XT3kEL8MCG3A deploy@ci' | sudo tee /home/deploy/.ssh/authorized_keys
sudo chown deploy:deploy /home/deploy/.ssh/authorized_keys
sudo chmod 600 /home/deploy/.ssh/authorized_keys
sudo ls -la /home/deploy/.ssh
#shot 01
#clear
## 2. sudo đúng việc
#sudofile visudo /etc/sudoers.d/deploy sudoers-deploy
sudo chmod 440 /etc/sudoers.d/deploy
sudo visudo -c
sudo -l -U deploy
## 3. sshd: deploy chỉ dùng key
#sudofile sudoedit /etc/ssh/sshd_config.d/05-an-toan.conf sshd-an-toan-thuc-hanh 02
sudo sshd -t && echo "cú pháp ổn"
sudo sshd -T -C user=deploy,host=ci,addr=10.0.0.5 | grep -E '^(authenticationmethods|allowtcpforwarding|kbdinteractiveauthentication) '
sudo sshd -T -C user=an,host=laptop,addr=10.0.0.9 | grep -E '^(authenticationmethods|allowtcpforwarding|kbdinteractiveauthentication) '
sudo systemctl reload ssh
#shot 03
#clear
## 4. Kiểm tra bằng những lần đăng nhập thật
#login deploy deploy-ci
#login an an-laptop otp
#login root an-laptop
#login cuong an-laptop
sudo journalctl -u ssh --no-pager | grep -E 'Accepted|not allowed because|ROOT LOGIN REFUSED' | tail -n 5
ssh -o BatchMode=yes -o PreferredAuthentications=password an@localhost true
#shot 04
