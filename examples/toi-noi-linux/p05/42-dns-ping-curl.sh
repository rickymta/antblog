# LNX-042 · DNS và kiểm tra kết nối: ping, dig, curl
#clear
## Tên được phân giải ở đâu
grep -v '^#' /etc/resolv.conf | grep -v '^$'
grep -E '^(hosts):' /etc/nsswitch.conf
getent hosts toinoi-app-02
dig +short toinoi-app-02
dig toinoi-app-02 | grep -A1 'ANSWER SECTION'
## /etc/hosts đi trước DNS, nhưng dig không đọc nó
echo '127.0.0.1 bao-gia.noi-bo' | sudo tee -a /etc/hosts
getent hosts bao-gia.noi-bo
dig +short bao-gia.noi-bo
curl -s http://bao-gia.noi-bo:8080/suc-khoe
#shot 01
#clear
## ping: máy kia có trả lời không
ping -c 3 toinoi-app-02
ping -c 2 -W 1 192.0.2.10
## Bốn kiểu lỗi khác nhau
curl -sS http://khong-co.toinoi.invalid/
curl -sS http://toinoi-app-02:8080/
curl -sS --connect-timeout 3 http://192.0.2.10:8080/
curl -s -o /dev/null -w '%{http_code}\n' 'http://bao-gia.noi-bo:8080/bao-gia?dich_vu=xe-tai&km=1'
#shot 02
#clear
## curl -v: từng bước của một yêu cầu
curl -sv http://bao-gia.noi-bo:8080/suc-khoe 2>&1 | grep -E '^(\*|>|<) ' | grep -v -E 'Date:|Content-Length'
curl -s -o /dev/null -w 'dns %{time_namelookup}s  ket noi %{time_connect}s  tong %{time_total}s\n' http://bao-gia.noi-bo:8080/suc-khoe
#shot 03
