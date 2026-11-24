# LNX-046 · Đồng bộ thư mục: Đồng bộ của SSH Studio và rsync
#clear
## Đồng bộ từ máy của bạn lên server
mkdir -p ~/bang-gia-moi
ls -la ~/bang-gia-moi
#gui dong-bo-len
#check ~/bang-gia-moi/bang-gia.csv bang-gia-moi.csv
#check ~/bang-gia-moi/ghi-chu.txt ghi-chu-bang-gia.txt
ls -l ~/bang-gia-moi
diff /srv/toinoi/bao-gia/current/bang-gia.csv ~/bang-gia-moi/bang-gia.csv
#shot 03
#clear
## rsync trên một máy: chuẩn bị bản 1.6.0
cp -a /srv/toinoi/bao-gia/releases/1.5.0 /srv/toinoi/bao-gia/releases/1.6.0
cp ~/bang-gia-moi/bang-gia.csv /srv/toinoi/bao-gia/releases/1.6.0/bang-gia.csv
sudo install -d -o an -g toinoi -m 2775 /srv/sao-luu
rsync -a /srv/toinoi/bao-gia/releases/ /srv/sao-luu/releases/
ls /srv/sao-luu/releases
echo 'ghi chu cu' > /srv/sao-luu/releases/ghi-chu-cu.txt
echo 'ban 1.6.0: them dich vu car7e' > /srv/toinoi/bao-gia/releases/1.6.0/THAY-DOI.txt
rsync -ai --dry-run --delete /srv/toinoi/bao-gia/releases/ /srv/sao-luu/releases/
rsync -ai --delete /srv/toinoi/bao-gia/releases/ /srv/sao-luu/releases/
rsync -ai --delete /srv/toinoi/bao-gia/releases/ /srv/sao-luu/releases/
#shot 04
#clear
## rsync qua SSH sang toinoi-app-02
#root install -m 600 -o an -g an /usr/share/toinoi-lab/p05/dong-bo /home/an/.ssh/dong-bo
#root install -m 644 -o an -g an /usr/share/toinoi-lab/p05/dong-bo.pub /home/an/.ssh/dong-bo.pub
#may app02
#root install -d -m 700 -o an -g an /home/an/.ssh
#root sh -c 'cat /usr/share/toinoi-lab/p05/dong-bo.pub >> /home/an/.ssh/authorized_keys; chown an:an /home/an/.ssh/authorized_keys; chmod 600 /home/an/.ssh/authorized_keys'
#may app01
ls -l ~/.ssh/dong-bo*
#file ~/.ssh/config ssh-config
ssh app02 hostname
ssh app02 mkdir -p sao-luu
rsync -a --delete --stats /srv/sao-luu/releases/ app02:sao-luu/releases/ | grep -E 'Number of (files|regular files transferred)|Total transferred'
ssh app02 'ls sao-luu/releases; cat sao-luu/releases/1.6.0/bang-gia.csv | tail -n 1'
rsync -ai --delete /srv/sao-luu/releases/ app02:sao-luu/releases/
#shot 05
