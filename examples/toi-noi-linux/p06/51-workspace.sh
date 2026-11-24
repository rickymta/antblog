# LNX-051 · Workspace, thư mục và Kết nối tất cả: nhiều server cùng lúc
#root /usr/share/toinoi-lab/p06/chuan-bi.sh
#may bastion
#root /usr/share/toinoi-lab/p06/chuan-bi.sh
#may app02
#root /usr/share/toinoi-lab/p06/chuan-bi.sh
#phien app01
#clear
#gui them-bastion
#gui thu-muc
#gui ket-noi-tat-ca
#xem known-hosts 08
#phien bastion
#clear
## Trên toinoi-bastion: ai đang đăng nhập
hostname
who
w -h
last -n 3 an
#shot 04
#phien app01
#clear
## Trên toinoi-app-01
who
last -n 3 an | head -n 3
#shot 06
#gui launchpad
#xem nhat-ky 07
