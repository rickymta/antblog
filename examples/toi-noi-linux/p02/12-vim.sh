# LNX-012 · vim cho công việc hằng ngày trên server
#clear
## Phiên đăng nhập mới: ~/bin đã nằm trong PATH
echo $PATH
type cdbg
mkdir -p ~/vim
cp /opt/toinoi/bao-gia/bang-gia.csv /opt/toinoi/bao-gia/appsettings.json ~/vim/
cd ~/vim
cat -n bang-gia.csv
## Sửa bảng giá: nhân đôi dòng, đổi từ, thay thế
#shot 06
#gui vim-bang-gia
#check ~/vim/bang-gia.csv vim-bang-gia.csv
cat -n bang-gia.csv
diff /opt/toinoi/bao-gia/bang-gia.csv bang-gia.csv
## Sửa cấu hình: tìm, thay trong cả file, chạy lệnh từ vim
#shot 07
#gui vim-appsettings
#check ~/vim/appsettings.json vim-appsettings.json
diff /opt/toinoi/bao-gia/appsettings.json appsettings.json
#shot 08
#clear
## Cấu hình vim tối thiểu
#file ~/.vimrc vimrc
cat ~/.vimrc
#shot 09
#gui vim-vimrc-thu
