# LNX-026 · Quyền đặc biệt: setuid, setgid, sticky bit
#clear
## setuid: chạy với quyền của chủ file
ls -l /usr/bin/passwd /usr/bin/sudo /etc/shadow
find /usr/bin /usr/sbin -perm -4000 -type f
stat -c '%a %A %U %n' /usr/bin/passwd
#shot 03
#clear
## setgid trên thư mục: file mới thừa kế nhóm
ls -ld /srv/toinoi/bao-gia/shared/logs
touch /srv/toinoi/bao-gia/shared/logs/thu-truoc.txt
chmod g+s /srv/toinoi/bao-gia/shared/logs
ls -ld /srv/toinoi/bao-gia/shared/logs
touch /srv/toinoi/bao-gia/shared/logs/thu-sau.txt
ls -l /srv/toinoi/bao-gia/shared/logs/thu-*.txt
stat -c '%a %A %n' /srv/toinoi/bao-gia/shared/logs
rm /srv/toinoi/bao-gia/shared/logs/thu-*.txt
#shot 01
#clear
## Sticky bit: /tmp
ls -ld /tmp
sudo -u binh touch /tmp/cua-binh.txt
ls -l /tmp/cua-binh.txt
rm -f /tmp/cua-binh.txt
sudo -u binh rm /tmp/cua-binh.txt
#shot 04
#clear
## Thư mục dùng chung của nhóm: setgid và sticky
sudo mkdir /srv/toinoi/chung
sudo chown root:toinoi /srv/toinoi/chung
sudo chmod 3770 /srv/toinoi/chung
ls -ld /srv/toinoi/chung
touch /srv/toinoi/chung/cua-an.txt
sudo -u binh touch /srv/toinoi/chung/cua-binh.txt
ls -l /srv/toinoi/chung
#shot 05
#clear
sudo -u binh rm -f /srv/toinoi/chung/cua-an.txt
sudo -u binh rm /srv/toinoi/chung/cua-binh.txt
ls /srv/toinoi/chung
#shot 02
