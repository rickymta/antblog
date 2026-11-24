# LNX-058 · Sự cố: đĩa đầy, hết inode, OOM killer
# Dựng sự cố (không in): job sao lưu đêm ghi vào /du-lieu cho tới khi đầy.
#root install -d -o an -g toinoi /du-lieu/sao-luu
#root for d in 18 19 20 21 22; do f=/du-lieu/sao-luu/bao-gia-2026-11-$d.tar.gz; fallocate -l 75M $f; chown an:toinoi $f; touch -d "2026-11-$d 02:00" $f; done
#root f=/du-lieu/sao-luu/bao-gia-2026-11-23.tar.gz; fallocate -l $(( $(df --output=avail -B1 /du-lieu | tail -n 1) - 4096 )) $f; chown an:toinoi $f; touch -d "2026-11-23 02:00" $f
#root printf '[Unit]\nDescription=Sao luu dem bao-gia vao /du-lieu\n\n[Service]\nType=oneshot\nUser=an\nExecStart=/usr/bin/tar -czf /du-lieu/sao-luu/bao-gia-2026-11-24.tar.gz -C /srv/toinoi/bao-gia releases shared/logs\n' > /etc/systemd/system/sao-luu-dem.service
#root systemctl daemon-reload; systemctl start sao-luu-dem.service 2>/dev/null || true
#clear
## 1. Triệu chứng: job sao lưu đêm thất bại
systemctl --failed --no-legend
journalctl -u sao-luu-dem --no-pager -o cat | grep -E 'tar:|gzip:|Failed' | head -n 4
## 2. Đĩa nào đầy, cái gì chiếm chỗ
df -h /du-lieu
sudo du -xh --max-depth=1 /du-lieu | sort -h
ls -lh /du-lieu/sao-luu
#shot 01
#clear
## 3. Xử lý: giữ 3 bản mới nhất, chạy lại job
ls -t /du-lieu/sao-luu/*.tar.gz | tail -n +4
ls -t /du-lieu/sao-luu/*.tar.gz | tail -n +4 | xargs rm -v
sudo systemctl reset-failed sao-luu-dem
sudo systemctl start sao-luu-dem
systemctl is-failed sao-luu-dem; df -h /du-lieu
#shot 02
#clear
## 4. Còn chỗ mà không tạo được file: hết inode
#root install -d -o an -g toinoi /du-lieu/tam
#root cd /du-lieu/tam && seq -f 'phien-%06g' 1 60000 | xargs touch 2>/dev/null; chown -R an:toinoi /du-lieu/tam; true
touch /du-lieu/bao-cao-moi.txt
df -h /du-lieu
df -i /du-lieu
sudo find /du-lieu -xdev -type f | cut -d/ -f1-3 | sort | uniq -c | sort -n | tail -n 3
find /du-lieu/tam -type f -name 'phien-*' -delete
df -i /du-lieu
#shot 03
#clear
## 5. Tiến trình bị giết: OOM killer
sudo systemd-run --unit bao-cao-thang -p MemoryMax=64M -p MemorySwapMax=0 /usr/bin/python3 -c "a = [bytearray(10**6) for _ in range(500)]"
sleep 3; systemctl is-failed bao-cao-thang
systemctl status bao-cao-thang --no-pager | grep -E 'Active|Main PID|oom'
journalctl -u bao-cao-thang --no-pager -o cat | tail -n 4
sudo systemctl reset-failed bao-cao-thang
#shot 04
