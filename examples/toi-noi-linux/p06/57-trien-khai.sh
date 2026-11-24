# LNX-057 · Triển khai bằng thư mục release và symlink
#clear
## Cấu trúc triển khai hiện tại
cd /srv/toinoi/bao-gia
ls -l
#shot 05
#clear
ls releases
readlink current
curl -s --cacert /etc/ssl/certs/bao-gia.toinoi.lab.crt https://bao-gia.toinoi.lab/suc-khoe
#shot 01
#clear
## Bản build 1.6.0 từ CI
sudo install -m 755 -o an -g toinoi /usr/share/toinoi-lab/p06/bao-gia-1.6.0 releases/1.6.0/bao-gia
diff -rq releases/1.5.0 releases/1.6.0
grep -m1 PHIEN_BAN releases/1.6.0/bao-gia
#shot 06
#clear
## Đổi current: tạo symlink mới rồi đổi tên đè lên (một thao tác nguyên tử)
ln -sfn releases/1.6.0 current.moi
mv -T current.moi current
readlink current
## Đo trong lúc khởi động lại: 60 yêu cầu qua nginx, mỗi 50 ms
(for i in $(seq 60); do curl -s -o /dev/null -w '%{http_code}\n' --cacert /etc/ssl/certs/bao-gia.toinoi.lab.crt https://bao-gia.toinoi.lab/suc-khoe; sleep 0.05; done > /tmp/do-trien-khai.txt) &
sleep 1; sudo systemctl restart bao-gia
wait
sort /tmp/do-trien-khai.txt | uniq -c
curl -s --cacert /etc/ssl/certs/bao-gia.toinoi.lab.crt https://bao-gia.toinoi.lab/suc-khoe
#shot 02
#clear
## Quay lui trong một lệnh
ln -sfn releases/1.5.0 current.moi && mv -T current.moi current && sudo systemctl restart bao-gia
sleep 1; curl -s --cacert /etc/ssl/certs/bao-gia.toinoi.lab.crt https://bao-gia.toinoi.lab/suc-khoe
#shot 07
#clear
## Đóng gói thành script
#file ~/bin/trien-khai trien-khai 03
chmod 755 ~/bin/trien-khai
trien-khai 1.9.9
trien-khai 1.6.0
readlink current
#shot 04
#xem files /srv/toinoi/bao-gia/releases 08
cd ~
