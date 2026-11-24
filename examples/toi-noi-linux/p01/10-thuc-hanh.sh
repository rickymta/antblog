# LNX-010 · Thực hành: chuẩn bị thư mục triển khai dịch vụ báo giá
#clear
## 1. Tạo cây thư mục triển khai
ls -ld /srv
mkdir -p /srv/toinoi/bao-gia
sudo mkdir -p /srv/toinoi/bao-gia/{releases/1.4.2,shared/config,shared/logs}
#shot 05
#gui sudo-mat-khau
sudo chown -R an:toinoi /srv/toinoi
tree /srv/toinoi
#shot 01
#clear
## 2. Đưa bản phát hành 1.4.2 và cấu hình vào
cd /srv/toinoi/bao-gia
cp /opt/toinoi/bao-gia/kiem-tra-cau-hinh.sh /opt/toinoi/bao-gia/bang-gia.csv releases/1.4.2/
cp /opt/toinoi/bao-gia/appsettings.json shared/config/
sudo cp /opt/toinoi/bao-gia/bi-mat.env shared/config/
sudo chown an:toinoi shared/config/bi-mat.env
#shot 07
#clear
## 3. Đặt nhóm và quyền
ls -l releases/1.4.2
chgrp -R toinoi releases shared
chmod 750 releases/1.4.2/kiem-tra-cau-hinh.sh
chmod 640 releases/1.4.2/bang-gia.csv shared/config/appsettings.json
chmod 600 shared/config/bi-mat.env
chmod 750 shared/config
chmod 770 shared/logs
find . -mindepth 1 -exec stat -c '%a %U:%G %n' {} + | sort -k3
#shot 02
#clear
## 4. Liên kết current trỏ tới bản đang chạy
ln -s releases/1.4.2 current
ls -l
current/kiem-tra-cau-hinh.sh
## 5. Trích lỗi của ngày 21/11 từ log
grep ERROR /var/log/toinoi/bao-gia.log > shared/logs/loi-2026-11-21.txt
wc -l shared/logs/loi-2026-11-21.txt
grep -o 'dich_vu=[a-z0-9]*' shared/logs/loi-2026-11-21.txt | sort | uniq -c | sort -rn | tee shared/logs/loi-theo-dich-vu.txt
#shot 06
#gui song-song-trien-khai
#clear
## 6. Người khác nhìn thấy gì?
sudo -u binh head -n 3 /srv/toinoi/bao-gia/shared/config/appsettings.json
sudo -u binh cat /srv/toinoi/bao-gia/shared/config/bi-mat.env
sudo -u nobody ls /srv/toinoi/bao-gia/shared/config
## 7. Kết quả bàn giao
tree -p /srv/toinoi
#shot 04
#xem files /srv/toinoi/bao-gia 08
