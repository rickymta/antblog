# LNX-013 · Trích cột và dòng: cut, sort, uniq, tr, wc
#clear
## Ba ngày log, một file đã nén
cd /var/log/toinoi
ls -l
file bao-gia.log.2.gz
zcat bao-gia.log.2.gz | head -n 3
wc -l bao-gia.log.1 bao-gia.log
zcat bao-gia.log.2.gz | wc -l
zcat -f bao-gia.log.2.gz bao-gia.log.1 bao-gia.log | wc -l
zcat -f bao-gia.log.2.gz bao-gia.log.1 bao-gia.log | cut -d' ' -f1 | uniq -c
#shot 01
#clear
## Dấu cách kép: vì sao cut lấy sai cột và tr -s sửa thế nào
head -n 4 bao-gia.log | cut -d' ' -f3,4
head -n 4 bao-gia.log | tr -s ' ' | cut -d' ' -f3,4
## Mức log theo từng ngày
zcat -f bao-gia.log.2.gz bao-gia.log.1 bao-gia.log | cut -c1-10,21-25 --output-delimiter=' ' | sort | uniq -c
## tr: đổi ký tự
head -n 1 bao-gia.log | tr ' ' '\n' | head -n 4
echo "dich_vu=car4" | tr 'a-z' 'A-Z'
#shot 02
#clear
## Hai loại lỗi trong ba ngày
zcat -f bao-gia.log.2.gz bao-gia.log.1 bao-gia.log | grep ' ERROR ' | cut -d' ' -f6- | sort | uniq -c | sort -rn
zcat -f bao-gia.log.2.gz bao-gia.log.1 bao-gia.log | grep 'bi khoa' | cut -d' ' -f1,2 | head -n 3
## sort theo cột số: yêu cầu chậm nhất của ba ngày
zcat -f bao-gia.log.2.gz bao-gia.log.1 bao-gia.log | grep ' WARN ' | tr -s ' ' | sort -t' ' -k10,10n | tail -n 3
## Lỗi theo ngày và dịch vụ, xếp thành bảng
zcat -f bao-gia.log.2.gz bao-gia.log.1 bao-gia.log | grep ' ERROR ' | cut -d' ' -f1,5 | sort | uniq -c | column -t
#shot 03
#clear
## column với file CSV; wc đếm byte và ký tự
column -t -s, /opt/toinoi/bao-gia/bang-gia.csv
wc -l -c /opt/toinoi/bao-gia/bang-gia.csv
wc -m /opt/toinoi/bao-gia/bang-gia.csv
## Phiên bản đã chạy; dòng chỉ có ở một file: uniq -u
zcat -f bao-gia.log.2.gz bao-gia.log.1 bao-gia.log | grep -o 'bao-gia 1\.4\.[0-9] khoi dong' | sort | uniq -c
cut -d, -f1 /opt/toinoi/bao-gia/bang-gia.csv ~/vim/bang-gia.csv | sort | uniq -u
#shot 04
