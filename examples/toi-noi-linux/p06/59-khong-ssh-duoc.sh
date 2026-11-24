# LNX-059 · Sự cố: không SSH được vào server
# Mỗi tình huống: dựng sự cố trên app02 (không in), xem SSH Studio báo gì, khoanh vùng từ bastion,
# đọc log bằng console của nhà cung cấp (đoạn #may app02), sửa, kết nối lại.
#phien app01
#clear
## Tình huống 1: sai cổng, không có gì nghe ở đó
#gui sai-cong
## Tình huống 2: sshd trên app02 không chạy
#may app02
#root systemctl stop ssh
#phien app01
#gui ket-noi-app02-loi
#shot 02
#phien bastion
#clear
nc -zv -w 3 toinoi-app-02 22
nc -zv -w 3 toinoi-app-02 443
ping -c 1 -W 2 toinoi-app-02 | tail -n 2
#shot 03
#may app02
## Console của nhà cung cấp, trên app02
systemctl is-active ssh
sudo journalctl -u ssh -n 3 --no-pager -o cat
sudo systemctl start ssh
#phien bastion
nc -zv -w 3 toinoi-app-02 22
## Tình huống 3: tường lửa của app02 chặn bastion
#may app02
#root ufw delete allow from 10.20.0.10 to any port 22 proto tcp
#phien bastion
#clear
nc -zv -w 3 toinoi-app-02 22
#gui ket-noi-app02-loi
#shot 04
#may app02
## Console: quy tắc ufw
sudo ufw status numbered
sudo ufw insert 1 allow from 10.20.0.10 to any port 22 proto tcp comment 'SSH qua bastion'
#phien bastion
#clear
nc -zv -w 3 toinoi-app-02 22
#shot 11
## Tình huống 4: server từ chối key
#may app02
#root chmod 666 /home/an/.ssh/authorized_keys
#phien app01
#gui ket-noi-app02-loi
#shot 05
#clear
ssh -o BatchMode=yes app02 true
#may app02
## Console: sshd ghi lý do
sudo journalctl -u ssh -n 20 --no-pager -o cat | grep -E 'Authentication refused|bad ownership' | tail -n 1
ls -l ~/.ssh/authorized_keys
chmod 600 ~/.ssh/authorized_keys
## Tình huống 5: host key đổi
#root install -m 600 /usr/share/toinoi-lab/p06/ssh_host_ed25519_key_app-02-moi /etc/ssh/ssh_host_ed25519_key && install -m 644 /usr/share/toinoi-lab/p06/ssh_host_ed25519_key_app-02-moi.pub /etc/ssh/ssh_host_ed25519_key.pub && systemctl restart ssh
#phien app01
#gui ket-noi-app02-loi
#shot 06
#may app02
## Console: fingerprint thật của app02 sau khi Bình dựng lại
ssh-keygen -lf /etc/ssh/ssh_host_ed25519_key.pub
#phien app01
#gui tin-host-key-moi
#clear
## app01 cũng ghi host key cũ của app02 (rsync, Ansible): cập nhật sau khi đã đối chiếu
ssh -o BatchMode=yes app02 hostname 2>&1 | grep -E 'IDENTIFICATION|Host key verification'
ssh-keygen -R toinoi-app-02
ssh app02 hostname
ssh-keygen -lF toinoi-app-02 | tail -n 1
#shot 08
#xem nhat-ky 09
#xem launchpad 10
