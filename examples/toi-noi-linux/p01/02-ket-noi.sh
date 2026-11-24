# LNX-002 · Dựng server lab và kết nối bằng SSH Studio
#gui them-server-va-ket-noi
## Những lệnh đầu tiên sau khi đăng nhập
whoami
hostname
pwd
ls
cat huong-dan.txt
#shot 08
#clear
## Fingerprint host key của server (so với hộp thoại của SSH Studio)
ssh-keygen -lf /etc/ssh/ssh_host_ed25519_key.pub
## Dịch vụ SSH đang chạy
systemctl is-active ssh
#shot 06
#gui known-hosts
#xem nhat-ky 09
#xem launchpad 10
