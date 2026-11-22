# LNX-004 · Cây thư mục và di chuyển
#clear
## Tôi đang ở đâu?
pwd
ls /
## Đường dẫn tuyệt đối và tương đối
cd /var/log
pwd
ls
cd toinoi
pwd
cd ..
pwd
cd ../../opt/toinoi/bao-gia
pwd
#shot 01
## Về thư mục home, quay lại thư mục trước
cd
pwd
cd -
cd ~
#clear
## Nhìn một thư mục
ls /opt/toinoi/bao-gia
ls -l /opt/toinoi/bao-gia
ls -lh /var/log/toinoi
ls -ld /opt/toinoi/bao-gia
tree /opt/toinoi
#shot 02
#gui files-goc
#clear
## File ẩn và symlink
ls -a
ls -ld /bin /sbin /lib
## Đây là file gì?
file /opt/toinoi/bao-gia/bang-gia.csv
file /opt/toinoi/bao-gia/kiem-tra-cau-hinh.sh
file /usr/bin/ls
stat /opt/toinoi/bao-gia/bang-gia.csv
#shot 04
