# LNX-022 · SSH key: tạo trong SSH Studio và cài lên server
#clear
## Server đang chấp nhận những cách đăng nhập nào?
ssh -o BatchMode=yes -o StrictHostKeyChecking=accept-new an@localhost true
ls -la ~/.ssh
#shot 07
#gui tao-va-nhap-key
#clear
## Cài public key cho tài khoản an
mkdir -p ~/.ssh
chmod 700 ~/.ssh
echo 'ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIFvQW8mHkVjU1nOtQ4pdIuRs5X14dOyuz6HzyQOw026x an-laptop' >> ~/.ssh/authorized_keys
chmod 600 ~/.ssh/authorized_keys
ls -la ~/.ssh
ssh-keygen -lf ~/.ssh/authorized_keys
#shot 03
#gui chuyen-sang-key
#xem servers 09
#login an an-laptop
#clear
## Bằng chứng trong log của sshd
sudo journalctl -u ssh --no-pager | grep -E 'Accepted (password|publickey) for an' | tail -n 2
#shot 08
#clear
## Quyền sai: sshd từ chối key
chmod o+w ~/.ssh/authorized_keys
ls -l ~/.ssh/authorized_keys
#login an an-laptop
sudo journalctl -u ssh --no-pager | grep -o 'Authentication refused: .*' | tail -n 1
chmod 600 ~/.ssh/authorized_keys
ls -l ~/.ssh/authorized_keys
#shot 06
