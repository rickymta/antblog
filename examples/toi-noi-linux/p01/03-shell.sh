# LNX-003 · Shell và dòng lệnh
#clear
## Shell đang dùng
echo $SHELL
bash --version
#shot 05
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
#shot 06
#clear
## Lệnh nằm ở đâu?
type ls
type cd
type date
## Tra trợ giúp
whoami --help
#shot 07
#clear
man -f ls
man -f date passwd
#shot 08
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
