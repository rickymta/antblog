# LNX-003 · Shell và dòng lệnh
## Shell đang dùng
echo $SHELL
bash --version
#clear
## Lệnh, tuỳ chọn, đối số
date
date +%F
date --utc +%H:%M
ls
ls -l
ls -a
ls -l -a
#shot 01
ls -la
ls -la /opt/toinoi
#clear
## Lệnh nằm ở đâu?
type ls
type cd
type date
## Tra trợ giúp
whoami --help
#clear
man -f ls
man -f date passwd
#gui man-ls
#@ touch ~/.lesshst && chmod 600 ~/.lesshst
## Khi gõ sai
lss
ls /khong-co
echo $?
ls /opt
echo $?
#shot 03
#gui phim-tat
