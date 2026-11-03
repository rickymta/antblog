# Triển khai Tech Knowledge Hub bằng Docker

Toàn bộ stack chạy bằng một file `docker-compose.yml` ở gốc repo này.

```
Người đọc ─▶ Cloudflare (proxy, SSL Full strict) ─▶ nginx :80/:443 ─┬─▶ /  /admin/   tệp tĩnh (React)
                                                                    └─▶ /api/        api:4000 ─▶ mongo:27017
Bạn (Compass/mongosh) ─▶ WireGuard UDP :51820 ─────────────────────────────────▶ mongo 172.30.0.10:27017
```

| Container | Vai trò | Mở ra ngoài |
|---|---|---|
| `nginx` | Cổng vào duy nhất. `/` trang đọc, `/admin/` trang quản trị, `/api/` proxy tới API (giống proxy của Vite khi dev). Port 80 chỉ chuyển hướng sang HTTPS | TCP 80, 443 |
| `api` | Express. Chỉ nghe trong mạng nội bộ | không |
| `mongo` | MongoDB 8.0, **bật xác thực**, 3 tài khoản (root, ứng dụng, chỉ-đọc) | không |
| `certbot` | Xin + tự gia hạn chứng chỉ Let's Encrypt bằng DNS-01 qua API Cloudflare | không |
| `wireguard` | VPN để bạn kết nối thẳng vào MongoDB | UDP 51820 |
| `seed` | Chạy tay để nạp nội dung mẫu + tài khoản quản trị (`profiles: tools`) | không |

## 1. Chuẩn bị server

- Linux có Docker Engine + Docker Compose v2 (`docker compose version`). Kernel ≥ 5.6 (có sẵn WireGuard).
- Tường lửa mở: TCP 80, TCP 443, UDP 51820. **Không** mở 27017.
- Hai repo đặt cạnh nhau:

```bash
mkdir -p /srv/tkh && cd /srv/tkh
git clone <repo tech-blog>      tech-blog
git clone <repo tech-blog-api>  tech-blog-api
cd tech-blog
```

(Repo API ở chỗ khác ⇒ đặt `API_DIR` trong `.env`.)

## 2. Chuẩn bị Cloudflare

1. **DNS:** bản ghi `A` (và `AAAA` nếu có IPv6) cho tên miền blog (vd `blog.example.com`) trỏ về IP server, **bật proxy** (đám mây cam).
2. **VPN:** WireGuard dùng UDP mà Cloudflare không chuyển tiếp. Dùng thẳng IP server, hoặc tạo bản ghi riêng (vd `vpn.example.com`) **tắt proxy** (đám mây xám). Giá trị này đặt vào `WG_SERVER_URL`.
3. **SSL/TLS › Overview:** chọn **Full (strict)**. Không dùng "Flexible": Cloudflare sẽ gọi vào port 80, nginx chuyển hướng sang HTTPS ⇒ vòng lặp chuyển hướng.
4. **API token** (My Profile › API Tokens › Create Token › mẫu "Edit zone DNS"): quyền **Zone › DNS › Edit**, Zone Resources = đúng zone của tên miền. Dán vào `CF_API_TOKEN`.
5. Tuỳ chọn, nên bật: SSL/TLS › Edge Certificates › **Always Use HTTPS**, và HSTS sau khi đã chạy ổn.

Chứng chỉ ở server được xin bằng **thử thách DNS-01**: certbot tạo bản ghi TXT tạm qua API Cloudflare, nên chạy được dù tên miền đang bật proxy và không cần mở cổng nào cho Let's Encrypt.

## 3. Tạo `.env`

```bash
sh deploy/init-env.sh
```

Script chép `.env.example` thành `.env` (quyền 600) và **tự sinh** `JWT_SECRET`, mật khẩu MongoDB (root, ứng dụng, chỉ-đọc) và mật khẩu quản trị blog. Sửa tay các dòng:

| Biến | Ví dụ |
|---|---|
| `DOMAIN` | `blog.example.com` |
| `CERT_EXTRA_DOMAINS` | `www.blog.example.com` (tuỳ chọn) |
| `ACME_EMAIL` | email nhận cảnh báo hết hạn |
| `CF_API_TOKEN` | token ở bước 2.4 |
| `WG_SERVER_URL` | IP server hoặc `vpn.example.com` (đám mây xám) |
| `WG_PEERS` | tên máy khách VPN, vd `laptop,desktop` |
| `SEED_ADMIN_EMAIL` | email đăng nhập trang quản trị |

Lần đầu nên đặt `ACME_STAGING=1` để thử luồng xin chứng chỉ mà không dính giới hạn tần suất của Let's Encrypt. Chạy ổn thì đổi về `0`, xoá chứng chỉ staging (`docker compose run --rm --entrypoint sh certbot -c 'rm -rf /etc/letsencrypt/live /etc/letsencrypt/archive /etc/letsencrypt/renewal'`) rồi `docker compose up -d certbot`.

## 4. Chạy

```bash
docker compose up -d --build
docker compose ps                    # mọi dịch vụ "healthy"/"running"
docker compose run --rm seed         # nạp chủ đề, bài viết, lộ trình + tài khoản quản trị
```

- Trang đọc: `https://<DOMAIN>/` · Quản trị: `https://<DOMAIN>/admin/` (email `SEED_ADMIN_EMAIL`, mật khẩu `SEED_ADMIN_PASSWORD` trong `.env`).
- Trong vài phút đầu nginx dùng **chứng chỉ tự ký tạm**: Cloudflare ở chế độ Full (strict) sẽ báo lỗi 526. Khi certbot xin xong (log `certbot: đã có chứng chỉ…`), nginx tự nạp chứng chỉ thật trong ≤ 5 phút.

```bash
docker compose logs -f certbot       # theo dõi xin chứng chỉ
docker compose logs nginx | grep tkh # nguồn chứng chỉ nginx đang dùng
```

Gia hạn: certbot kiểm tra mỗi 12 giờ (chỉ gia hạn khi còn < 30 ngày); nginx tự nạp lại mỗi 6 giờ.

Dữ liệu demo (bình luận, lượt xem 30 ngày, 8 tài khoản `@demo.local`) để xem bảng thống kê:

```bash
docker compose run --rm seed npm run seed:demo
docker compose run --rm seed npm run seed:demo -- --remove   # gỡ
```

## 5. Kết nối database qua VPN

Container `wireguard` sinh cấu hình cho từng tên trong `WG_PEERS` ở lần chạy đầu:

```
deploy/wireguard/config/peer_<tên>/peer_<tên>.conf   # file cấu hình máy khách (chứa khoá bí mật)
deploy/wireguard/config/peer_<tên>/peer_<tên>.png    # mã QR cho điện thoại
```

1. Cài WireGuard trên máy bạn (Windows/macOS/Linux/điện thoại), **Import tunnel from file** ⇒ chọn file `.conf` (chép về bằng `scp`, đừng gửi qua chat/email).
2. Bật tunnel. VPN chỉ định tuyến **đúng một địa chỉ** — MongoDB `172.30.0.10` — nên Internet của máy bạn không đi qua server.
3. Kết nối bằng MongoDB Compass hoặc mongosh với tài khoản **chỉ đọc**:

```
mongodb://tkh_inspector:<MONGO_INSPECTOR_PASSWORD>@172.30.0.10:27017/tech_knowledge_hub?authSource=tech_knowledge_hub&directConnection=true
```

Cần sửa dữ liệu thì dùng tài khoản root (`authSource=admin`) — hạn chế dùng.

Thêm máy khách: thêm tên vào `WG_PEERS` rồi `docker compose up -d wireguard`. Thu hồi: bỏ tên khỏi `WG_PEERS`, xoá thư mục `peer_<tên>`, `docker compose up -d --force-recreate wireguard`.

## 6. Vận hành

| Việc | Lệnh |
|---|---|
| Cập nhật code | `git pull` ở cả hai repo ⇒ `docker compose up -d --build` |
| Xem log | `docker compose logs -f api nginx` |
| Sao lưu DB | `docker compose exec mongo sh -c 'mongodump --archive --gzip -u "$MONGO_INITDB_ROOT_USERNAME" -p "$MONGO_INITDB_ROOT_PASSWORD" --authenticationDatabase admin --db tech_knowledge_hub' > backup-$(date +%F).archive.gz` |
| Khôi phục | `docker compose exec -T mongo sh -c 'mongorestore --archive --gzip --drop -u "$MONGO_INITDB_ROOT_USERNAME" -p "$MONGO_INITDB_ROOT_PASSWORD" --authenticationDatabase admin' < backup-....archive.gz` |
| Cập nhật dải IP Cloudflare | `sh deploy/nginx/update-cloudflare-ips.sh && docker compose up -d --build nginx` |
| Dừng | `docker compose down` (giữ dữ liệu) · `docker compose down -v` **xoá cả DB và chứng chỉ** |

Lưu ý:
- Mật khẩu MongoDB trong `.env` chỉ có tác dụng ở **lần khởi tạo volume đầu tiên**. Đổi về sau phải dùng `db.changeUserPassword(...)` trong mongosh rồi mới sửa `.env`.
- nginx lấy IP thật của người đọc từ header `CF-Connecting-IP` (chỉ khi request đến từ dải IP của Cloudflare). Muốn chặn hẳn truy cập vào thẳng IP server, giới hạn TCP 80/443 ở tường lửa chỉ cho dải IP Cloudflare (`https://www.cloudflare.com/ips/`).
- Cookie đăng nhập có cờ `Secure` (`COOKIE_SECURE=true`) ⇒ chỉ đăng nhập được qua HTTPS.
