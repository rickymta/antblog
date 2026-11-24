# LNX-007 · Người dùng, nhóm và quyền file
#clear
## Tôi là ai, thuộc nhóm nào?
whoami
id
id binh
getent passwd an binh baogia
getent group toinoi sudo
#shot 07
#clear
## Đọc quyền của file
ls -l /opt/toinoi/bao-gia
cat /opt/toinoi/bao-gia/bi-mat.env
ls -l /etc/passwd /etc/shadow
cat /etc/shadow
#shot 01
#clear
## Quyền thực thi
cp /opt/toinoi/bao-gia/kiem-tra-cau-hinh.sh /opt/toinoi/bao-gia/bang-gia.csv du-an/bao-gia/
cd du-an/bao-gia
ls -l
./kiem-tra-cau-hinh.sh
chmod u+x kiem-tra-cau-hinh.sh
ls -l kiem-tra-cau-hinh.sh
./kiem-tra-cau-hinh.sh
## chmod dạng chữ và dạng số
chmod go-r bang-gia.csv
ls -l bang-gia.csv
chmod 640 bang-gia.csv
chmod 750 kiem-tra-cau-hinh.sh
stat -c '%a %A %U:%G %n' bang-gia.csv kiem-tra-cau-hinh.sh
#shot 02
#gui chmod-kiem-tra
#clear
## Quyền trên thư mục
ls -ld cau-hinh
chmod a-x cau-hinh
ls cau-hinh
cat cau-hinh/bang-gia.csv.bak
cd cau-hinh
chmod 755 cau-hinh
chmod a-w cau-hinh
touch cau-hinh/thu.txt
rm -f cau-hinh/bang-gia.csv.bak
chmod 755 cau-hinh
#shot 08
#clear
## Quyền mặc định của file mới
umask
touch moi.txt
mkdir moi
ls -ld moi.txt moi
rm -r moi.txt moi
#shot 04
#clear
## Đổi chủ sở hữu và nhóm: cần root
chown binh bang-gia.csv
chgrp toinoi bang-gia.csv
sudo chown an:toinoi bang-gia.csv
#shot 06
#gui sudo-mat-khau
ls -l bang-gia.csv
sudo cat /opt/toinoi/bao-gia/bi-mat.env
sudo -l
## Thêm An vào nhóm toinoi: phải đăng nhập lại mới có hiệu lực
sudo usermod -aG toinoi an
id
id an
#shot 05
