# LNX-009 · Tìm kiếm: find, grep và wildcard
#clear
## find theo tên
find /opt/toinoi -name '*.csv'
find ~ -name 'bang-gia*'
find ~ -iname '*.TXT'
## find theo loại, kích thước, thời gian
find ~/du-an -type d
#shot 01
#clear
find /var/log -type f -size +20k
find /var/log -type f -size +20k 2>/dev/null
find /opt/toinoi -type f -newer /opt/toinoi/bao-gia/bang-gia.csv
find ~ -type f -mmin -30 -not -name '.*'
#shot 05
#clear
## find theo quyền và chủ sở hữu
find /opt/toinoi -type f -perm 600
find ~ -type f -perm -u+x
find /opt /var/log -user baogia 2>/dev/null
## Làm gì đó với kết quả
find ~ -name '*.csv' -exec ls -l {} \;
find ~ -name '*.csv' -exec wc -l {} +
#shot 02
#clear
## grep: tìm trong nội dung
grep -c ERROR /var/log/toinoi/bao-gia.log
grep -n 'q-100057' /var/log/toinoi/bao-gia.log
grep -n -B1 -A1 'q-100057' /var/log/toinoi/bao-gia.log
grep -i 'KHOI DONG' /var/log/toinoi/bao-gia.log
#shot 07
#clear
grep -v INFO /var/log/toinoi/bao-gia.log | head -n 4
grep -E 'ERROR|WARN' /var/log/toinoi/bao-gia.log | wc -l
grep -E 'cham: 2[0-9]{3} ms' /var/log/toinoi/bao-gia.log
grep -w 8080 /opt/toinoi/bao-gia/appsettings.json
#shot 03
#gui tim-trong-terminal
#clear
## grep trong cả thư mục
grep -r 'bang-gia.csv' /opt/toinoi
grep -r 'bang-gia.csv' /opt/toinoi 2>/dev/null
grep -rl 'bao-gia' /opt/toinoi ~/du-an 2>/dev/null
grep -rn --include='*.sh' 'exit' /opt/toinoi
## Kết hợp find và grep
find ~ -name '*.csv' -exec grep -H 'bike' {} +
## Lệnh nằm ở đâu?
which ls vim tree
whereis ls
#shot 06
