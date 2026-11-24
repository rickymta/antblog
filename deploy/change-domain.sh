#!/bin/sh
# Đổi tên miền của stack đang chạy trên server, xin chứng chỉ mới, build lại các ảnh nhúng tên miền.
# Chạy tại thư mục gốc repo tech-blog trên server (đã `git pull` bản mới):
#
#   sh deploy/change-domain.sh <tên-miền-cũ> <tên-miền-mới>      vd: sh deploy/change-domain.sh antblog.xyz antfarms.cloud
#   sh deploy/change-domain.sh --rollback <file .env.bak-...>       khôi phục .env cũ và dựng lại
#
# Thứ tự an toàn (site cũ vẫn chạy tới khi chứng chỉ mới có thật):
#   1. Kiểm DNS các tên mới  2. Sao lưu .env rồi đổi DOMAIN/ADMIN_DOMAIN/CERT_EXTRA_DOMAINS/WG_SERVER_URL
#   3. Dựng lại certbot ⇒ xin chứng chỉ mới qua DNS-01 Cloudflare, CHỜ tới khi có
#   4. Build + dựng lại nginx (trang quản trị nhúng SITE_URL) và web (nhúng NEXT_PUBLIC_SITE_URL)
#   5. Dựng lại wireguard (SERVERURL mới)  6. Kiểm tra chứng chỉ từng tên miền
#
# Trước khi chạy: API token Cloudflare (CF_API_TOKEN) PHẢI có quyền Zone › DNS › Edit trên zone MỚI.
set -eu
cd "$(dirname "$0")/.."

say()  { printf '\n==> %s\n' "$*"; }
warn() { printf 'CẢNH BÁO: %s\n' "$*" >&2; }
die()  { printf 'LỖI: %s\n' "$*" >&2; exit 1; }
# Giá trị biến trong .env như docker compose đọc: bỏ CR, khoảng trắng hai đầu, cặp nháy bao ngoài.
envval() {
  sed -n "s/^$1=//p" .env | tail -n1 \
    | sed -e 's/\r$//' -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//' -e 's/^"\(.*\)"$/\1/' -e "s/^'\(.*\)'$/\1/"
}

command -v docker >/dev/null || die "không thấy lệnh docker."
[ -f .env ] || die "không thấy .env ở $(pwd)."

if [ "${1:-}" = "--rollback" ]; then
  [ -f "${2:-}" ] || die "cần đường dẫn file sao lưu, vd .env.bak-20261124-090000"
  cp "$2" .env && chmod 600 .env
  say "Đã khôi phục .env từ $2 — dựng lại certbot, nginx, web, wireguard"
  docker compose up -d certbot
  docker compose up -d --build nginx web
  docker compose up -d wireguard
  exit 0
fi

OLD="${1:-}"; NEW="${2:-}"
[ -n "$OLD" ] && [ -n "$NEW" ] || die "cách dùng: sh deploy/change-domain.sh <tên-miền-cũ> <tên-miền-mới>"
OLD_RE=$(printf '%s' "$OLD" | sed 's/\./\\./g')
grep -Eq "^(DOMAIN|ADMIN_DOMAIN|CERT_EXTRA_DOMAINS|WG_SERVER_URL)=.*$OLD_RE" .env \
  || die ".env không có $OLD ở DOMAIN/ADMIN_DOMAIN/CERT_EXTRA_DOMAINS/WG_SERVER_URL (đã đổi rồi?)."

# --- 1. Kiểm DNS ---------------------------------------------------------------------------------------------
say "1/6 Kiểm DNS các tên miền mới"
NEW_DOMAIN=$(envval DOMAIN | sed "s/$OLD_RE/$NEW/g")
NEW_ADMIN=$(envval ADMIN_DOMAIN | sed "s/$OLD_RE/$NEW/g")
NEW_EXTRA=$(envval CERT_EXTRA_DOMAINS | sed "s/$OLD_RE/$NEW/g")
OLD_WG=$(envval WG_SERVER_URL)
NEW_WG=$(printf '%s' "$OLD_WG" | sed "s/$OLD_RE/$NEW/g")
missing=0
for h in $NEW_DOMAIN $NEW_ADMIN $(echo "$NEW_EXTRA" | tr ',' ' ') $NEW_WG; do
  ip=$(getent ahostsv4 "$h" 2>/dev/null | awk 'NR==1{print $1}')
  printf '  %-32s -> %s\n' "$h" "${ip:-KHÔNG PHÂN GIẢI ĐƯỢC}"
  [ -n "$ip" ] || missing=1
done
[ "$missing" = 0 ] || die "tạo đủ bản ghi DNS trên Cloudflare (zone $NEW) rồi chạy lại."

# Plugin dns-cloudflare của certbot tìm zone qua đúng API này ⇒ token không thấy zone thì certbot chắc chắn hỏng.
TOKEN=$(envval CF_API_TOKEN)
[ -n "$TOKEN" ] && [ -n "$(envval ACME_EMAIL)" ] || die "CF_API_TOKEN hoặc ACME_EMAIL trong .env đang trống."
CF_API=https://api.cloudflare.com/client/v4
ZONE_JSON=$(curl -s -m 20 -H "Authorization: Bearer $TOKEN" "$CF_API/zones?name=$NEW" || true)
ZONE_HITS=$(printf '%s' "$ZONE_JSON" | grep -o '"count":[0-9]*' | head -n1 | cut -d: -f2)
if [ "${ZONE_HITS:-0}" -lt 1 ]; then
  # In trạng thái token + thông báo lỗi của Cloudflare (không in token) để biết sai ở đâu.
  echo "  Token (độ dài ${#TOKEN} ký tự, 4 ký tự cuối …$(printf '%s' "$TOKEN" | tail -c 4)):"
  curl -s -m 20 -H "Authorization: Bearer $TOKEN" "$CF_API/user/tokens/verify" \
    | grep -o '"status":"[^"]*"\|"expires_on":"[^"]*"\|"message":"[^"]*"' | sed 's/^/    verify  /' || true
  printf '%s' "$ZONE_JSON" | grep -o '"count":[0-9]*\|"message":"[^"]*"' | sed 's/^/    zones   /' || true
  die "CF_API_TOKEN không thấy zone $NEW. Vào Cloudflare › My Profile › API Tokens, sửa token: \
Zone › DNS › Edit, Zone Resources thêm zone $NEW (tạo token mới thì nhớ dán vào CF_API_TOKEN trong .env) — rồi chạy lại. \
(Chưa đổi gì trên server.)"
fi
echo "  CF_API_TOKEN thấy zone $NEW."

# --- 2. Sao lưu + sửa .env -----------------------------------------------------------------------------------
BAK=".env.bak-$(date +%Y%m%d-%H%M%S)"
cp -p .env "$BAK"
say "2/6 Đã sao lưu .env -> $BAK; đổi $OLD -> $NEW"
sed -i -E "/^(DOMAIN|ADMIN_DOMAIN|CERT_EXTRA_DOMAINS|WG_SERVER_URL)=/ s/$OLD_RE/$NEW/g" .env
grep -E '^(DOMAIN|ADMIN_DOMAIN|CERT_EXTRA_DOMAINS|WG_SERVER_URL)=' .env | sed 's/^/  /'

# --- 3. Xin chứng chỉ mới ------------------------------------------------------------------------------------
say "3/6 Dựng lại certbot và chờ chứng chỉ cho $NEW_DOMAIN (tối đa ~6 phút)"
SINCE=$(date -u +%Y-%m-%dT%H:%M:%SZ)
docker compose up -d certbot
i=0
until docker compose exec -T certbot test -s "/etc/letsencrypt/live/$NEW_DOMAIN/fullchain.pem" 2>/dev/null; do
  if docker compose logs --since "$SINCE" certbot 2>/dev/null | grep -q "thất bại\|tạm dừng"; then
    docker compose logs --since "$SINCE" certbot | tail -n 25
    printf '\n'
    warn "certbot chưa xin được chứng chỉ (lỗi ở trên). nginx/web chưa bị đụng nên site vẫn chạy như cũ."
    warn "Trả .env về như cũ:  sh deploy/change-domain.sh --rollback $BAK"
    warn "sửa nguyên nhân rồi chạy lại:  sh deploy/change-domain.sh $OLD $NEW"
    exit 1
  fi
  i=$((i + 1)); [ "$i" -le 72 ] || die "quá thời gian chờ chứng chỉ — xem: docker compose logs certbot"
  sleep 5
done
echo "  Đã có chứng chỉ /etc/letsencrypt/live/$NEW_DOMAIN/"

# --- 4. Build + dựng lại nginx, web ---------------------------------------------------------------------------
say "4/6 Build lại và dựng lại web + nginx (nhúng https://$NEW_DOMAIN)"
docker compose up -d --build web nginx

# --- 5. VPN ----------------------------------------------------------------------------------------------------
say "5/6 Dựng lại wireguard với SERVERURL=$NEW_WG"
docker compose up -d wireguard

# --- 6. Kiểm tra -----------------------------------------------------------------------------------------------
say "6/6 Kiểm tra chứng chỉ nginx đang phục vụ"
i=0
until [ "$(docker inspect -f '{{.State.Health.Status}}' "$(docker compose ps -q nginx)" 2>/dev/null)" = healthy ]; do
  i=$((i + 1)); [ "$i" -le 30 ] || { warn "nginx chưa healthy — xem: docker compose logs nginx"; break; }
  sleep 3
done
for h in $NEW_DOMAIN $NEW_ADMIN $(echo "$NEW_EXTRA" | tr ',' ' '); do
  info=$(echo | openssl s_client -connect 127.0.0.1:443 -servername "$h" 2>/dev/null \
    | openssl x509 -noout -subject -issuer -enddate 2>/dev/null | tr '\n' ' ')
  code=$(curl -sk -o /dev/null -w '%{http_code}' --resolve "$h:443:127.0.0.1" "https://$h/" || echo "---")
  printf '  %-32s HTTP %s  %s\n' "$h" "$code" "${info:-KHÔNG ĐỌC ĐƯỢC CHỨNG CHỈ}"
done

if [ "$NEW_WG" = "$OLD_WG" ]; then
  VPN_NOTE="không đổi (WG_SERVER_URL=$NEW_WG), giữ nguyên cấu hình đang dùng."
else
  VPN_NOTE="lấy lại cấu hình mới ở deploy/wireguard/config/peer_*/peer_*.conf (Endpoint = $NEW_WG)."
fi
cat <<EOF

XONG. Việc còn lại làm tay:
  - Máy khách VPN: $VPN_NOTE
  - Khi chạy ổn, xoá chứng chỉ tên cũ (tên cũ không còn xin/gia hạn được):
      docker compose exec certbot certbot delete --cert-name $OLD --non-interactive
  - Cloudflare zone $OLD (khi tên cũ hoạt động lại): Redirect Rule 301 $OLD/* -> https://$NEW_DOMAIN/\${1}
  - Google Search Console: thêm property $NEW_DOMAIN, gửi sitemap, dùng "Change of address" từ $OLD.
  - Sao lưu .env cũ: $BAK (chứa secret — xoá khi không cần).
EOF
