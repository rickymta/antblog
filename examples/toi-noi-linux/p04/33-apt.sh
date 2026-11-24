# LNX-033 · Cài phần mềm bằng apt
#clear
## Danh sách gói từ kho
grep -v '^#' /etc/apt/sources.list.d/ubuntu.sources | grep -E '^(Types|URIs|Suites|Components):'
sudo apt update 2>&1 | tail -n 3
#shot 01
#clear
## Tìm và xem gói trước khi cài
jq --version
apt search --names-only '^(jq|tmux|sysstat)$' 2>/dev/null | grep -v -E '^$|\.\.\.$'
apt show jq 2>/dev/null | grep -E '^(Package|Version|Depends|Installed-Size|Section|Description):'
apt-cache policy tmux
## Cài
sudo apt install -y jq tmux sysstat 2>&1 | grep -E -A2 '^(The following NEW|[0-9]+ upgraded)'
jq --version
tmux -V
#shot 02
#clear
## Gói đã cài: dpkg
dpkg -l jq tmux sysstat | tail -n 3
dpkg -L jq | grep bin/
dpkg -S /usr/bin/pstree
dpkg -S "$(command -v crontab)"
echo '{"dich_vu":"car4","gia":69000}' | jq .gia
## Gỡ gói
sudo apt remove -y sysstat 2>&1 | grep -E '^(Removing|The following packages will be REMOVED)' -A1
dpkg -l sysstat | tail -n 1
ls /etc/sysstat
sudo apt purge -y sysstat 2>&1 | grep -E '^Purging'
dpkg -l sysstat 2>&1 | tail -n 1
ls /etc/sysstat
sudo apt autoremove -y 2>&1 | grep -E '^Removing'
#shot 03
#clear
## Ghim phiên bản
sudo apt-mark hold tmux
apt-mark showhold
apt list --upgradable 2>/dev/null | wc -l
sudo apt-mark unhold tmux
ls /var/log/apt/
grep -E '^(Start-Date|Commandline)' /var/log/apt/history.log | tail -n 6
#shot 04
