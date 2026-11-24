# LNX-060 · Thực hành: runbook vận hành toinoi-app-01
#clear
## 1. Một lệnh cho mọi bằng chứng
#sudofile sudoedit /usr/local/bin/kiem-tra-van-hanh kiem-tra-van-hanh 01
sudo chmod 755 /usr/local/bin/kiem-tra-van-hanh
kiem-tra-van-hanh; echo "ma thoat: $?"
#shot 02
#clear
## 2. Runbook nằm cạnh server
sudo install -d -o an -g toinoi -m 2775 /srv/toinoi/runbook
#file /srv/toinoi/runbook/van-hanh.md van-hanh.md 03
grep -c '^## ' /srv/toinoi/runbook/van-hanh.md
#shot 06
#xem files /srv/toinoi/runbook 07
#clear
## 3. Diễn tập: bao-gia dừng giữa giờ
sudo systemctl stop bao-gia
kiem-tra-van-hanh | grep -E 'LOI|----'
#shot 05
#clear
sed -n '/^## bao-gia/,/^## Đĩa/p' /srv/toinoi/runbook/van-hanh.md | head -n 5
journalctl -u bao-gia -n 2 --no-pager -o cat
sudo systemctl start bao-gia
sleep 2; kiem-tra-van-hanh | tail -n 1
#shot 04
