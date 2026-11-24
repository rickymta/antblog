# LNX-017 · Điều kiện, vòng lặp và hàm trong bash
#clear
## test, [ ] và [[ ]]
[ -r /var/log/toinoi/bao-gia.log ] && echo "đọc được"
[ -w /var/log/toinoi/bao-gia.log ] || echo "không ghi được"
so_loi=$(grep -c ' ERROR ' /var/log/toinoi/bao-gia.log)
[ "$so_loi" -gt 10 ] && echo "$so_loi lỗi: nhiều hơn 10"
#shot 05
#clear
[ "$so_loi" -gt 9 ] && echo "so sánh số: 17 > 9"
[ "$so_loi" \> 9 ] || echo "so sánh chuỗi: \"17\" đứng trước \"9\""
[[ bao-gia.log.2.gz == *.gz ]] && echo "file nén"
## if, elif, else
if [ "$so_loi" -eq 0 ]; then echo "không có lỗi"; elif [ "$so_loi" -lt 20 ]; then echo "ít lỗi"; else echo "nhiều lỗi"; fi
#shot 01
#clear
## for: lặp qua danh sách và qua file
for dv in bike car4 car7 delivery; do echo "$dv: $(grep -c "dich_vu=$dv " /var/log/toinoi/bao-gia.log) yêu cầu"; done
for f in /var/log/toinoi/bao-gia.log*; do echo "$f"; done
#shot 06
#clear
## while read: lặp qua từng dòng
grep ' ERROR ' /var/log/toinoi/bao-gia.log | head -n 3 | while read -r ngay gio muc ma dv _; do echo "$gio $ma $dv"; done
#shot 07
#clear
## case: rẽ nhánh theo mẫu
for f in /var/log/toinoi/bao-gia.log*; do case $f in *.gz) echo "$f: nén gzip" ;; *.log) echo "$f: đang ghi" ;; *) echo "$f: đã xoay vòng" ;; esac; done
#shot 02
#clear
## Script phiên bản 2: hàm, nhiều file, file nén
#file bin/bao-cao-log bao-cao-log-v2 03
bao-cao-log /var/log/toinoi/bao-gia.log*
#shot 08
#clear
bao-cao-log /var/log/toinoi/bao-gia.log /khong-co /var/log/toinoi/bao-gia.log.1
echo $?
bao-cao-log
echo $?
#shot 04
