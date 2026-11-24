# LNX-034 · Job nền, nohup và tmux
#clear
## Chạy nền trong shell
cd ~
xuat-chuyen 40 > xuat-1.log &
jobs -l
#wait 3
cat xuat-1.log
#shot 05
#clear
## Mất kết nối thì job chết
xuat-chuyen 600 > xuat-2.log 2>&1 &
nohup xuat-chuyen 600 > xuat-3.log 2>&1 &
jobs -l
#wait 1
#shot 01
#ngat
#gui ngat-ket-noi
#clear
pgrep -a -f 'xuat-chuyen 600'
tail -n 1 xuat-2.log xuat-3.log
#wait 2
tail -n 1 xuat-2.log xuat-3.log
pkill -f 'xuat-chuyen 600'
#shot 02
#clear
## tmux: phiên vẫn sống khi SSH đứt
tmux new-session -d -s xuat 'xuat-chuyen 600'
tmux ls
#shot 07
#wait 3
#ngat
#gui ngat-ket-noi
#clear
tmux ls
tmux capture-pane -p -t xuat | grep -v '^$' | tail -n 3
#shot 06
#gui tmux-attach
#clear
## Dọn dẹp
tmux kill-session -t xuat
tmux ls
rm -f xuat-1.log xuat-2.log xuat-3.log nohup.out
#shot 04
