# LNX-011 · Biến, biến môi trường, PATH và file khởi động của shell
#root /usr/share/toinoi-lab/p02/chuan-bi.sh
#clear
## Log sau khi Bình bật xoay vòng log
ls -l /var/log/toinoi
## Biến của shell
dich_vu=bao-gia
echo $dich_vu
echo "Dịch vụ: $dich_vu"
echo 'Dịch vụ: $dich_vu'
#shot 05
#clear
phien_ban=1.4.2
echo "$dich_vu-$phien_ban.tar.gz"
echo "$dich_vu_$phien_ban.tar.gz"
echo "${dich_vu}_${phien_ban}.tar.gz"
dich_vu = bao-gia
#shot 06
#clear
## Lưu kết quả của một lệnh vào biến
so_loi=$(grep -c ERROR /var/log/toinoi/bao-gia.log)
ngay=$(head -n 1 /var/log/toinoi/bao-gia.log | cut -d' ' -f1)
echo "Ngày $ngay có $so_loi lỗi"
#shot 01
#clear
## Biến môi trường
printenv HOME USER SHELL LANG
bash -c 'echo "Shell con thấy: [$dich_vu]"'
export dich_vu
bash -c 'echo "Shell con thấy: [$dich_vu]"'
date -d '2026-11-21 06:37:58' '+%F %T %Z'
TZ=UTC date -d '2026-11-21 06:37:58 +07' '+%F %T %Z'
echo $TZ
#shot 07
#clear
## PATH
echo $PATH
type -a ls
mkdir -p ~/bin
#file bin/xin-chao xin-chao
chmod u+x bin/xin-chao
xin-chao
./bin/xin-chao
export PATH="$HOME/bin:$PATH"
echo $PATH
xin-chao
which xin-chao
#shot 02
#clear
## File khởi động: ~/.profile và ~/.bashrc
grep -n -A2 'HOME/bin' ~/.profile
grep -n '^alias l' ~/.bashrc
#shot 08
#gui vim-bashrc
#check ~/.bashrc bashrc
tail -n 3 ~/.bashrc
loibg
source ~/.bashrc
loibg
cdbg
pwd
type cdbg loibg
#shot 04
