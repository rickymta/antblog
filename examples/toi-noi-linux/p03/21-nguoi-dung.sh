# LNX-021 · Quản lý người dùng và nhóm
#clear
## Ai có tài khoản trên server?
getent passwd | awk -F: '$3 >= 1000 && $3 < 60000 { print $1, $3, $6, $7 }'
getent passwd | wc -l
awk -F: '$7 !~ /(nologin|false)$/ { print $1, $7 }' /etc/passwd
#shot 05
#clear
## Tạo tài khoản cho Cường
sudo useradd --create-home --shell /bin/bash --comment "Le Van Cuong" cuong
id cuong
sudo ls -la /home/cuong
ls -A /etc/skel
#shot 01
#clear
## Mật khẩu ban đầu, bắt đổi ở lần đăng nhập đầu
sudo passwd -S cuong
echo 'cuong:Tam-thoi-2611' | sudo chpasswd
sudo passwd -S cuong
sudo chage -d 0 cuong
sudo chage -l cuong
#shot 06
#gui dang-nhap-cuong
#root echo 'cuong:Cuong-doi-mat-khau-2026' | chpasswd
sudo passwd -S cuong
#clear
## Nhóm
sudo groupadd trien-khai
sudo usermod -aG toinoi,trien-khai cuong
id cuong
getent group trien-khai toinoi
sudo gpasswd -d cuong trien-khai
groups cuong
#shot 07
#clear
## Khoá, hết hạn và mở khoá
sudo passwd -l cuong
sudo passwd -S cuong
sudo chage -E 2026-12-31 cuong
sudo chage -l cuong
sudo chage -E -1 cuong
sudo passwd -u cuong
sudo passwd -S cuong
#shot 03
#clear
## Tài khoản dịch vụ
getent passwd baogia
sudo su - baogia
echo $?
#shot 08
#clear
sudo -u baogia id
sudo useradd --system --no-create-home --shell /usr/sbin/nologin --gid toinoi bao-cao
getent passwd bao-cao
sudo userdel bao-cao
getent passwd bao-cao || echo "đã xoá"
#shot 04
