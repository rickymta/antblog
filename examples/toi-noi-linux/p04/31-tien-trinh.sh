# LNX-031 · Tiến trình: ps, top, PID và cây tiến trình
#root /usr/share/toinoi-lab/p04/chuan-bi.sh
#root setsid -f setpriv --reuid=binh --regid=binh --init-groups /usr/local/bin/tinh-lai-bao-cao </dev/null >/dev/null 2>&1
#root setsid -f setpriv --reuid=binh --regid=binh --init-groups /usr/local/bin/xuat-du-lieu </dev/null >/dev/null 2>&1
#wait 2
#clear
## Shell của bạn cũng là một tiến trình
echo $$
ps
ps -o pid,ppid,user,stat,etime,cmd -p $$
ls /proc/$$ | head -n 8
readlink /proc/$$/exe
#shot 05
#clear
## Mọi tiến trình trên server
ps aux | head -n 6
ps aux | wc -l
#shot 06
#clear
ps -ef --forest | grep -v ' \[' | cut -c1-100 | head -n 25
#shot 01
#clear
## Ai đang ăn CPU?
ps aux --sort=-%cpu | head -n 4
pgrep -a -u binh
ps -o pid,user,%cpu,%mem,rss,etime,stat,cmd -u binh
grep -E '^(Name|State|PPid|Uid|VmRSS|Threads)' /proc/$(pgrep -u binh -f tinh-lai)/status
#shot 02
#gui top
#clear
## top chạy một lần (chế độ batch)
top -b -n 1 -o %CPU | head -n 12
uptime
#shot 07
#clear
nproc
free -h
#shot 04
