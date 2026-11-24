# LNX-015 · awk: xử lý log có cột
#clear
cd /var/log/toinoi
## Trường: awk tự tách theo khoảng trắng
head -n 4 bao-gia.log | awk '{ print $3, $4 }'
head -n 4 bao-gia.log | awk '{ print NF, $NF }'
#shot 05
#clear
## Lọc theo điều kiện
awk '$3 == "ERROR"' bao-gia.log | head -n 2
awk '$3 == "ERROR" { n++ } END { print n, "lỗi" }' bao-gia.log
awk '$NF == "ms" && $(NF-1) > 2500 { print $2, $4, $(NF-1) }' bao-gia.log
#shot 01
#clear
## Tổng và trung bình
awk '$3 == "INFO" && $NF == "ms" { tong += $(NF-1); n++ } END { printf "%d yêu cầu, trung bình %.1f ms\n", n, tong / n }' bao-gia.log
#shot 06
#clear
## Mảng: gom theo dịch vụ
awk '$3 == "INFO" && $NF == "ms" { split($5, a, "="); n[a[2]]++; t[a[2]] += $(NF-1) } END { for (dv in n) printf "%-9s %4d %6.1f ms\n", dv, n[dv], t[dv] / n[dv] }' bao-gia.log | sort
## Đếm hai loại lỗi theo ngày
zcat -f bao-gia.log.2.gz bao-gia.log.1 bao-gia.log | awk '$3 == "ERROR" { loai = ($0 ~ /bi khoa/) ? "bang-gia" : "ban-do"; n[$1 " " loai]++ } END { for (k in n) print k, n[k] }' | sort
#shot 02
#clear
## File CSV: -F và -v
awk -F, 'NR > 1 { print $1, $3 }' /opt/toinoi/bao-gia/bang-gia.csv
awk -F, -v km=10 'NR > 1 { gia = $3 + $4 * km; if (gia < $5) gia = $5; printf "%-9s %7d đ\n", $1, gia }' /opt/toinoi/bao-gia/bang-gia.csv
#shot 07
#clear
## Chương trình awk trong file
#file ~/bin/tom-tat.awk tom-tat.awk 03
zcat -f bao-gia.log.2.gz bao-gia.log.1 bao-gia.log | awk -f ~/bin/tom-tat.awk
#shot 04
