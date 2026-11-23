# LNX-018 · Nén và lưu trữ: gzip, xz, zstd và tar
#clear
mkdir -p ~/sao-luu
cd ~/sao-luu
## gzip một file
cp /var/log/toinoi/bao-gia.log.1 .
ls -l
gzip bao-gia.log.1
ls -l
gzip -l bao-gia.log.1.gz
gunzip bao-gia.log.1.gz
gzip -k bao-gia.log.1
ls -l
#shot 01
#clear
## So sánh gzip, xz và zstd
xz -k bao-gia.log.1
zstd -q -k bao-gia.log.1
ls -l bao-gia.log.1*
xzcat bao-gia.log.1.xz | wc -l
zstdcat bao-gia.log.1.zst | wc -l
rm bao-gia.log.1.gz bao-gia.log.1.xz bao-gia.log.1.zst
## tar: gom nhiều file thành một
tar -cf log.tar -C /var/log/toinoi bao-gia.log bao-gia.log.1 bao-gia.log.2.gz
tar -tvf log.tar
tar -czf log-ba-ngay.tar.gz -C /var/log/toinoi .
tar --zstd -cf log-ba-ngay.tar.zst -C /var/log/toinoi .
tar -tzf log-ba-ngay.tar.gz
ls -l
#shot 02
#clear
## Giải nén vào thư mục khác và kiểm tra
mkdir thu-giai-nen
tar -xzf log-ba-ngay.tar.gz -C thu-giai-nen
ls -l thu-giai-nen
diff -r /var/log/toinoi thu-giai-nen && echo "giống hệt bản gốc"
rm -r thu-giai-nen log.tar bao-gia.log.1
mkdir thu-giai-nen
tar -xzf log-ba-ngay.tar.gz -C thu-giai-nen ./bao-gia.log.1
ls thu-giai-nen
zgrep -c ' ERROR ' /var/log/toinoi/bao-gia.log.2.gz
#shot 03
## Tải bản sao lưu về máy bằng khung Files
#gui download-sao-luu
#clear
## Script sao lưu đầu tiên
#file ~/bin/sao-luu-log sao-luu-log-v1
chmod u+x ~/bin/sao-luu-log
sao-luu-log
ls ~/sao-luu
#shot 05
