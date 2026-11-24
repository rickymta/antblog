# LNX-023 · Host key, Known Hosts và cảnh báo host key thay đổi
#clear
## Host key của server
ls -l /etc/ssh/ssh_host_*
ssh-keygen -lf /etc/ssh/ssh_host_ed25519_key.pub
ssh-keyscan -t ed25519 localhost 2>/dev/null | ssh-keygen -lf -
#shot 07
#clear
## known_hosts ở phía client
cat ~/.ssh/known_hosts
ssh-keygen -F localhost
#shot 01
#clear
## Server được dựng lại: host key mới
sudo cp -a /etc/ssh/ssh_host_ed25519_key /etc/ssh/ssh_host_ed25519_key.pub /root/
#root install -m 600 /usr/share/toinoi-lab/p03/ssh_host_ed25519_key_moi /etc/ssh/ssh_host_ed25519_key && install -m 644 /usr/share/toinoi-lab/p03/ssh_host_ed25519_key_moi.pub /etc/ssh/ssh_host_ed25519_key.pub
sudo systemctl restart ssh
ssh-keygen -lf /etc/ssh/ssh_host_ed25519_key.pub
ssh -o BatchMode=yes an@localhost true
#shot 02
#gui host-key-doi
#clear
## Xử lý ở phía client OpenSSH
ssh-keygen -R localhost
ssh -o BatchMode=yes -o StrictHostKeyChecking=accept-new an@localhost true
#shot 09
#clear
## Dựng lại server thì đưa host key cũ về
sudo cp -a /root/ssh_host_ed25519_key /root/ssh_host_ed25519_key.pub /etc/ssh/
sudo systemctl restart ssh
ssh-keygen -lf /etc/ssh/ssh_host_ed25519_key.pub
ssh-keygen -R localhost
ssh -o BatchMode=yes -o StrictHostKeyChecking=accept-new an@localhost true
#gui host-key-tra-lai
#shot 06
#xem known-hosts 10
