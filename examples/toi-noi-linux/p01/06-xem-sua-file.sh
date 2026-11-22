# LNX-006 · Xem và sửa file văn bản
## Xem cả file
cat huong-dan.txt
#clear
cat -n /opt/toinoi/bao-gia/bang-gia.csv
## File dài: đếm, xem đầu, xem cuối
wc -l /var/log/toinoi/bao-gia.log
wc /var/log/toinoi/bao-gia.log
head /var/log/toinoi/bao-gia.log
#clear
head -n 3 /var/log/toinoi/bao-gia.log
tail -n 3 /var/log/toinoi/bao-gia.log
tail -n +361 /var/log/toinoi/bao-gia.log
#shot 01
#gui less-log
## Sửa file bằng trình sửa của SSH Studio (mục 5): giá mở cửa xe máy 8000 -> 9000
#gui sua-bang-gia
#@ sed -i 's/^bike,Xe máy,8000,/bike,Xe máy,9000,/' tai-lieu/bang-gia.csv
#clear
cat tai-lieu/bang-gia.csv
diff /opt/toinoi/bao-gia/bang-gia.csv tai-lieu/bang-gia.csv
## Sửa file bằng vim (mục 4): ghi chú của An
#gui vim-ghi-chu
#@ printf 'Ghi chú của An\n- 22/11: tăng giá mở cửa xe máy lên 9.000 đ ở bản nháp\n' > tai-lieu/ghi-chu.txt
cat tai-lieu/ghi-chu.txt
## Đây có phải file văn bản không?
file tai-lieu/ghi-chu.txt tai-lieu/bang-gia.csv /usr/bin/ls
#shot 06
