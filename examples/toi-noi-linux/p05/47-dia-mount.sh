# LNX-047 · Đĩa, phân vùng, hệ thống file và mount
#clear
## Hệ thống file đang được gắn
findmnt -lo TARGET,SOURCE,FSTYPE,SIZE -t overlay,tmpfs
grep -v '^#' /etc/fstab
## Nhà cung cấp vừa gắn thêm một ổ 512 MB
#root losetup -f /srv/dia/du-lieu-01.img
DIA=$(sudo losetup -j /srv/dia/du-lieu-01.img -n -O NAME)
echo $DIA
lsblk -o NAME,SIZE,TYPE,FSTYPE,MOUNTPOINTS $DIA
sudo blkid $DIA || echo "chua co he thong file"
#shot 01
#clear
## Tạo hệ thống file và gắn vào cây thư mục
sudo mkfs.ext4 -q -L du-lieu $DIA
sudo blkid $DIA
sudo mkdir -p /du-lieu
sudo mount $DIA /du-lieu
df -hT /du-lieu
ls -la /du-lieu
sudo chown an:toinoi /du-lieu
#shot 02
#clear
## Gắn tự động khi khởi động: fstab theo UUID
UUID=$(sudo blkid -s UUID -o value $DIA)
echo "UUID=$UUID /du-lieu ext4 defaults,nofail 0 2" | sudo tee -a /etc/fstab
sudo umount /du-lieu
findmnt /du-lieu || echo "chua gan"
sudo systemctl daemon-reload
sudo mount -a
findmnt -o TARGET,SOURCE,FSTYPE,OPTIONS /du-lieu
echo "luu tren o du lieu" > /du-lieu/thu.txt
ls -l /du-lieu
#shot 03
