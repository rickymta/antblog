# LNX-048 · Bộ nhớ và tải: free, vmstat, load average
#clear
## Bộ nhớ: free và available
free -h
#shot 06
#clear
grep -E '^(MemTotal|MemFree|MemAvailable|Buffers|Cached|SwapTotal|SwapFree|Dirty):' /proc/meminfo
swapon --show
systemctl show bao-gia --no-pager -p MemoryCurrent -p MemoryMax
#shot 01
#clear
## vmstat lúc rảnh
vmstat 1 4
#shot 04
#clear
## Nghẽn CPU
nohup tinh-lai-bao-cao > /dev/null 2>&1 &
nohup tinh-lai-bao-cao > /dev/null 2>&1 &
#wait 2
vmstat 1 4
uptime
pkill -f tinh-lai-bao-cao
#shot 02
#clear
## Nghẽn đĩa: ghi thẳng xuống ổ, bỏ qua cache
for i in 1 2 3 4 5; do dd if=/dev/zero of=/du-lieu/ghi-thu-$i bs=1M count=50 oflag=direct status=none; sleep 1; done &
vmstat 1 4
wait
rm /du-lieu/ghi-thu-*
#shot 05
#clear
## Ai đang dùng nhiều bộ nhớ
ps -eo pid,user,rss,comm --sort=-rss | head -n 5
#shot 03
