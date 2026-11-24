# LNX-005 · Tạo, sao chép, di chuyển, xoá
#clear
## Tạo thư mục và file
mkdir tai-lieu
mkdir du-an/bao-gia/cau-hinh
mkdir -p du-an/bao-gia/cau-hinh
tree du-an
touch tai-lieu/ghi-chu.txt
ls -l tai-lieu
## Sao chép
cp /opt/toinoi/bao-gia/bang-gia.csv tai-lieu/
cp tai-lieu/bang-gia.csv tai-lieu/bang-gia.csv.bak
ls -l tai-lieu
#shot 01
cp /opt/toinoi/bao-gia du-an/ban-sao
cp -r /opt/toinoi/bao-gia du-an/ban-sao
ls -l du-an/ban-sao
#shot 05
#clear
## Đổi tên và di chuyển
mv tai-lieu/ghi-chu.txt tai-lieu/nhat-ky.txt
mv tai-lieu/bang-gia.csv.bak du-an/bao-gia/cau-hinh/
tree tai-lieu du-an
## Xoá
rm tai-lieu/nhat-ky.txt
rmdir du-an/ban-sao
rm du-an/ban-sao
rm -r du-an/ban-sao
tree du-an
#shot 02
#clear
## Nhiều file một lúc: wildcard và ngoặc nhọn
touch bao-cao-01.txt bao-cao-02.txt bao-cao-03.txt nhap.tmp
ls
ls bao-cao-*
echo rm bao-cao-0[12].txt
rm bao-cao-0[12].txt nhap.tmp
ls
mkdir -p du-an/bao-gia/{releases,shared}
ls du-an/bao-gia
#shot 06
## File do SSH Studio tải lên (khung Files, mục 6 của bài)
#gui upload-lich-truc
#@ printf 'Lịch trực tuần 47 (lab)\nThứ Hai: An\nThứ Ba: Bình\n' > tai-lieu/lich-truc.txt
ls -l tai-lieu
cat tai-lieu/lich-truc.txt
#shot 07
#xem files /home/an/du-an 08
