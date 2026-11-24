# LNX-052 · Bastion và jump host: vào server nội bộ qua một bước nhảy
#phien bastion
#clear
## Bastion được dựng thế nào
cat /etc/ssh/sshd_config.d/10-bastion.conf
sudo sshd -T | grep -E '^(passwordauthentication|kbdinteractiveauthentication|permitrootlogin|allowagentforwarding|allowtcpforwarding|x11forwarding) '
ip -br addr show eth0
nc -zv -w 3 toinoi-app-02 22
#shot 01
#gui them-app02
#gui ket-noi-app02
#phien app02
#clear
## Trên toinoi-app-02, qua bastion
hostname
who
echo $SSH_CONNECTION
#shot 04
#phien bastion
#clear
## Trên bastion: đường hầm tới app02 là một kết nối của sshd
sudo ss -tnp | grep -E 'State|10.20.0.12:22'
#shot 05
#phien app02
#clear
## Chỉ nhận SSH từ bastion (và từ app01 cho rsync)
sudo ufw allow from 10.20.0.10 to any port 22 proto tcp comment 'SSH qua bastion'
sudo ufw allow from 10.20.0.11 to any port 22 proto tcp comment 'rsync tu app01'
sudo ufw default deny incoming
sudo ufw --force enable
sudo ufw status
#shot 06
#phien app01
#clear
## Từ app01: rsync sang app02 vẫn chạy
ssh app02 hostname
nc -zv -w 3 toinoi-app-02 22
