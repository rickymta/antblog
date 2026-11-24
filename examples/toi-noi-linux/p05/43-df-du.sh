# LNX-043 · Dung lượng đĩa: df, du và tìm thứ chiếm chỗ
#clear
## df: mỗi hệ thống file còn bao nhiêu
df -h
df -hT / /tmp /run
## du: thư mục nào lớn
sudo du -sh /var/log /srv /opt /home /usr
sudo du -xh --max-depth=2 /srv | sort -h | tail -n 5
sudo find / -xdev -type f -size +100M -printf '%s\t%p\n' 2>/dev/null | sort -n | tail -n 3
ls -lh /srv/toinoi/bao-gia/shared/logs/debug-2026-11.log
#shot 01
#clear
## Hết inode khi còn chỗ
sudo mkdir -p /mnt/nho
sudo mount -t tmpfs -o size=20m,nr_inodes=64 tmpfs /mnt/nho
sudo chown an:an /mnt/nho
for i in $(seq 1 100); do touch /mnt/nho/f$i 2>/dev/null || { echo "khong tao duoc file thu $i"; break; }; done
df -h /mnt/nho
df -i /mnt/nho
touch /mnt/nho/them-mot-file
sudo umount /mnt/nho
#shot 02
#clear
## File đã xoá mà vẫn giữ chỗ
sudo mkdir -p /mnt/thu
sudo mount -t tmpfs -o size=100m tmpfs /mnt/thu
sudo chown an:an /mnt/thu
head -c 60M /dev/zero > /mnt/thu/log-lon.log
nohup tail -f /mnt/thu/log-lon.log > /dev/null 2>&1 &
#wait 1
df -h /mnt/thu
rm /mnt/thu/log-lon.log
ls /mnt/thu
df -h /mnt/thu
lsof -nP +L1 2>/dev/null | grep -E 'COMMAND|/mnt/thu'
pkill -f 'tail -f /mnt/thu'
#wait 1
df -h /mnt/thu
sudo umount /mnt/thu
#shot 03
