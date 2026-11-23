# LNX-016 · Shell script đầu tiên: shebang, biến, tham số, mã thoát
#clear
## Viết script bằng vim
#file bin/bao-cao-log bao-cao-log-v1 01
bao-cao-log /var/log/toinoi/bao-gia.log
ls -l bin/bao-cao-log
chmod u+x bin/bao-cao-log
bao-cao-log /var/log/toinoi/bao-gia.log
bash bin/bao-cao-log /var/log/toinoi/bao-gia.log.1
#shot 02
#clear
## Tham số sai và mã thoát
bao-cao-log
echo $?
bao-cao-log /khong-co
echo $?
bao-cao-log /var/log/toinoi/bao-gia.log.1 > /dev/null && echo "báo cáo xong"
## Khi grep không tìm thấy gì
grep -v ' ERROR ' /var/log/toinoi/bao-gia.log > ~/tai-lieu/khong-loi.log
bao-cao-log ~/tai-lieu/khong-loi.log
echo $?
#shot 03
#clear
## Tìm chỗ hỏng bằng bash -x
bash -n bin/bao-cao-log && echo "cú pháp ổn"
bash -x bin/bao-cao-log ~/tai-lieu/khong-loi.log
grep -c ' ERROR ' ~/tai-lieu/khong-loi.log
echo $?
## Sửa bằng sed, chạy lại
sed -i 's/\(grep -c .*\))$/\1 || true)/' bin/bao-cao-log
grep -n 'grep -c' bin/bao-cao-log
bao-cao-log ~/tai-lieu/khong-loi.log
echo $?
#shot 04
