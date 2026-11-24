# LNX-019 · Script an toàn: dấu nháy, set -u, trap, mktemp, flock và shellcheck
#clear
mkdir -p ~/thu-an-toan
cd ~/thu-an-toan
## Dấu nháy và tên file có dấu cách
touch "bao cao 21-11.txt" bao-cao-20-11.txt
ls
f="bao cao 21-11.txt"
ls -l $f
ls -l "$f"
#shot 06
#clear
for x in $(ls); do echo "[$x]"; done
for x in *; do echo "[$x]"; done
#shot 01
#clear
## Một chữ gõ sai và lệnh rm -rf
#file xoa-tam-loi xoa-tam-loi
bash xoa-tam-loi
bash -u xoa-tam-loi
echo $?
#shot 07
#clear
shellcheck xoa-tam-loi
#shot 02
#clear
## trap và mktemp: dọn file tạm kể cả khi script hỏng giữa chừng
mktemp
#file thu-trap thu-trap
bash thu-trap
echo $?
ls /tmp/bao-cao.* 2>/dev/null | wc -l
#shot 08
#clear
## flock: không cho hai bản chạy chồng nhau
flock -n /tmp/thu.lock echo "lấy được khoá"
flock -n /tmp/thu.lock bash -c 'flock -n /tmp/thu.lock echo "lấy được khoá" || echo "khoá đang bị giữ"'
#shot 03
#clear
## Script sao lưu phiên bản 2
#file ~/bin/sao-luu-log sao-luu-log-v2 04
shellcheck ~/bin/sao-luu-log ~/bin/bao-cao-log && echo "shellcheck: không có cảnh báo"
NGAY=2026-11-19 sao-luu-log -d ~/sao-luu/thu
NGAY=2026-11-20 sao-luu-log -d ~/sao-luu/thu
NGAY=2026-11-21 sao-luu-log -d ~/sao-luu/thu -g 2
ls -l ~/sao-luu/thu
#shot 09
#clear
flock -n ~/sao-luu/thu/.sao-luu.lock sao-luu-log -d ~/sao-luu/thu
echo $?
sao-luu-log -g abc
echo $?
#shot 05
