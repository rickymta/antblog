# LNX-014 · sed: sửa văn bản theo dòng
#clear
cd ~/tai-lieu
## In theo dòng và theo mẫu
sed -n '1,3p' /var/log/toinoi/bao-gia.log
sed -n '$p' /var/log/toinoi/bao-gia.log
sed -n '/SIGTERM/,+2p' /var/log/toinoi/bao-gia.log.1
#shot 04
#clear
## Thay thế
echo "bao-gia 1.4.1 dang chay" | sed 's/1.4.1/1.4.2/'
echo "km=2.5 km=3.0" | sed 's/km=/quang_duong=/'
echo "km=2.5 km=3.0" | sed 's/km=/quang_duong=/g'
echo "1x4y1" | sed 's/1.4.1/1.4.2/'
echo "1x4y1" | sed 's/1\.4\.1/1.4.2/'
#shot 01
#clear
## Nhóm trong biểu thức chính quy: đổi định dạng dòng log
grep -m 3 ' WARN ' /var/log/toinoi/bao-gia.log
grep -m 3 ' WARN ' /var/log/toinoi/bao-gia.log | sed -E 's/^([0-9-]+) ([0-9:]+) +WARN +(q-[0-9]+) .* ([0-9]+) ms$/\3 \1T\2 \4ms/'
zcat -f /var/log/toinoi/bao-gia.log* | sed -n 's/.*cham: \([0-9]*\) ms$/\1/p' | sort -n | tail -n 3
#shot 05
#clear
## Xoá dòng
sed -e '/^#/d' -e '/^$/d' /opt/toinoi/bao-gia/kiem-tra-cau-hinh.sh
#shot 02
#clear
## Sửa file tại chỗ, có bản sao lưu
cp /opt/toinoi/bao-gia/appsettings.json appsettings.json
sed -i.bak 's/"Level": "Information"/"Level": "Warning"/' appsettings.json
ls appsettings.json*
diff appsettings.json.bak appsettings.json
sed -i 's#/opt/toinoi/bao-gia#/srv/toinoi/bao-gia/current#' appsettings.json
grep -n current appsettings.json
#shot 06
#clear
## Chỉ sửa trong một khối
sed -n '/"Fare"/,/}/p' appsettings.json
sed -i '/"Fare"/,/}/ s/1000/500/' appsettings.json
diff appsettings.json.bak appsettings.json
#shot 03
