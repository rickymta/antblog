# LNX-008 · Chuyển hướng và pipe
#gui dang-nhap-lai
## An đăng nhập lại sau bài 7: đã ở trong nhóm toinoi
id
cd tai-lieu
## Ghi kết quả ra file: > và >>
ls /opt/toinoi/bao-gia > danh-sach.txt
cat danh-sach.txt
date +%F >> danh-sach.txt
cat danh-sach.txt
date +%F > danh-sach.txt
cat danh-sach.txt
#clear
## Hai luồng ra: stdout và stderr
ls /opt/toinoi /khong-co
ls /opt/toinoi /khong-co > ket-qua.txt
cat ket-qua.txt
ls /opt/toinoi /khong-co 2> loi.txt
cat loi.txt
ls /opt/toinoi /khong-co > tat-ca.txt 2>&1
cat tat-ca.txt
ls /opt/toinoi /khong-co 2> /dev/null
#shot 02
## Luồng vào: <
wc -l /var/log/toinoi/bao-gia.log
wc -l < /var/log/toinoi/bao-gia.log
#clear
## Pipe: nối lệnh với nhau
grep ERROR /var/log/toinoi/bao-gia.log | wc -l
grep ERROR /var/log/toinoi/bao-gia.log | head -n 3
grep ERROR /var/log/toinoi/bao-gia.log | tail -n 2 | cut -d' ' -f1,2,4
## Đếm theo nhóm: cut | sort | uniq -c
cut -c21-25 /var/log/toinoi/bao-gia.log | sort | uniq -c
grep -o 'dich_vu=[a-z0-9]*' /var/log/toinoi/bao-gia.log | sort | uniq -c | sort -rn
grep WARN /var/log/toinoi/bao-gia.log | sort -t: -k4 -n | tail -n 3
#shot 03
#clear
## Vừa xem vừa ghi: tee
grep ERROR /var/log/toinoi/bao-gia.log | cut -d' ' -f5 | sort | uniq -c | tee loi-theo-dich-vu.txt
ls -l
#gui chia-doi-tai-lieu
## Pipe chỉ mang stdout
ls /khong-co | wc -l
ls /khong-co 2>&1 | wc -l
## Mã thoát và nối lệnh có điều kiện
grep -q ERROR /var/log/toinoi/bao-gia.log && echo "co loi trong log"
grep -q FATAL /var/log/toinoi/bao-gia.log || echo "khong co dong FATAL"
rm ket-qua.txt loi.txt tat-ca.txt danh-sach.txt
