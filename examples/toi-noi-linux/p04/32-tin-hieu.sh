# LNX-032 · Tín hiệu và dừng tiến trình: kill, SIGTERM, SIGKILL
#clear
## Danh sách tín hiệu
kill -l | head -n 4
kill -l 15 9 1
## Gửi SIGTERM cho tiến trình của người khác
pgrep -a -u binh
kill $(pgrep -u binh -f tinh-lai)
sudo kill $(pgrep -u binh -f tinh-lai)
#wait 1
pgrep -a -u binh
cat /tmp/tinh-lai-bao-cao.log
ls /tmp/tinh-lai-bao-cao.*.tmp
#shot 01
#clear
## Một chương trình bỏ qua SIGTERM
sudo kill -TERM $(pgrep -u binh -f xuat-du-lieu)
#wait 1
ps -o pid,stat,etime,cmd -u binh
grep -E '^Sig(Ign|Cgt)' /proc/$(pgrep -u binh -f xuat-du-lieu)/status
## SIGKILL: lựa chọn cuối
sudo kill -KILL $(pgrep -u binh -f xuat-du-lieu)
#wait 1
pgrep -a -u binh || echo "binh khong con tien trinh nao"
ls -l /tmp/xuat-du-lieu.lock
cat /tmp/xuat-du-lieu.lock
sudo rm /tmp/xuat-du-lieu.lock
#shot 02
#clear
## pkill, killall và tín hiệu theo tên
sleep 600 &
sleep 601 &
jobs -l
pkill -e -f 'sleep 60[01]'
#wait 1
jobs
sleep 602 &
killall -v sleep
#shot 03
