# Triển khai Tech Knowledge Hub bằng Docker

Toàn bộ stack chạy bằng một file `docker-compose.yml` ở gốc repo này.

```
Người đọc ─▶ Cloudflare (proxy, SSL Full strict) ─▶ nginx :80/:443 ─┬─▶ antblog.xyz          trang đọc (tĩnh)
                                                                    ├─▶ admin.antblog.xyz    trang quản trị (tĩnh)
                                                                    └─▶ <cả hai>/api/        api:4000 ─▶ mongo:27017
Bạn (Compass/mongosh) ─▶ WireGuard UDP :51820 (vpn.antblog.xyz) ──────────────▶ mongo 172.30.0.10:27017
```

| Container | Vai trò | Mở ra ngoài |
|---|---|---|
| `nginx` | Cổng vào duy nhất. `antblog.xyz` proxy tới `web`, `admin.antblog.xyz` phục vụ tĩnh trang quản trị (luôn `noindex`); ở cả hai tên miền `/api/` proxy tới API và `/uploads/` đọc thẳng volume ảnh. Port 80 chỉ chuyển hướng sang HTTPS; host lạ hoặc gọi thẳng IP bị đóng kết nối | TCP 80, 443 |
| `web` | Trang đọc Next.js (render phía server cho SEO: metadata, JSON-LD, `sitemap.xml`, `robots.txt`). Gọi thẳng `api:4000` trong mạng nội bộ | không |
| `api` | Express. Chỉ nghe trong mạng nội bộ; ghi ảnh tải lên vào volume `uploads` | không |
| `mongo` | MongoDB 8.0, **bật xác thực**, 3 tài khoản (root, ứng dụng, chỉ-đọc) | không |
| `certbot` | Xin + tự gia hạn MỘT chứng chỉ Let's Encrypt cho cả `antblog.xyz` và `admin.antblog.xyz`, bằng DNS-01 qua API Cloudflare | không |
| `wireguard` | VPN để bạn kết nối thẳng vào MongoDB | UDP 51820 |
| `seed` | Chạy tay để nạp nội dung mẫu + tài khoản quản trị (`profiles: tools`) | không |

## 1. Chuẩn bị server

- Linux có Docker Engine + Docker Compose v2 (`docker compose version`). Kernel ≥ 5.6 (có sẵn WireGuard).
- Tường lửa mở: TCP 80, TCP 443, UDP 51820. **Không** mở 27017.
- Lấy mã nguồn (một repo gồm cả frontend và API):

```bash
git clone https://github.com/rickymta/antblog.git /srv/antblog
cd /srv/antblog
```

## 2. Chuẩn bị Cloudflare

1. **DNS** của zone `antblog.xyz`:

   | Loại | Tên | Trỏ tới | Proxy |
   |---|---|---|---|
   | A | `antblog.xyz` (`@`) | IP server | **Bật** (đám mây cam) |
   | A | `admin` | IP server | **Bật** (đám mây cam) |
   | A | `vpn` | IP server | **Tắt** (đám mây xám) |

   (Thêm `AAAA` tương ứng nếu server có IPv6.)
2. **VPN:** WireGuard dùng UDP mà Cloudflare không chuyển tiếp, nên `vpn.antblog.xyz` phải tắt proxy. Đặt `WG_SERVER_URL=vpn.antblog.xyz` (hoặc IP server). Lưu ý bản ghi này làm lộ IP thật của server — muốn giấu thì dùng thẳng IP và không tạo bản ghi `vpn`.
3. **SSL/TLS › Overview:** chọn **Full (strict)**. Không dùng "Flexible": Cloudflare sẽ gọi vào port 80, nginx chuyển hướng sang HTTPS ⇒ vòng lặp chuyển hướng.
4. **API token** (My Profile › API Tokens › Create Token › mẫu "Edit zone DNS"): quyền **Zone › DNS › Edit**, Zone Resources = zone `antblog.xyz`. Dán vào `CF_API_TOKEN`.
5. Tuỳ chọn, nên bật: SSL/TLS › Edge Certificates › **Always Use HTTPS**, và HSTS sau khi đã chạy ổn.
6. Tuỳ chọn, nên làm cho trang quản trị: **Zero Trust › Access** tạo ứng dụng cho `admin.antblog.xyz`, chỉ cho email của bạn — người lạ bị chặn ở Cloudflare trước khi tới form đăng nhập. Ứng dụng Access phủ luôn `admin.antblog.xyz/api/*` là đúng, vì trang quản trị gọi API trên chính tên miền đó; trang đọc ở `antblog.xyz` không bị ảnh hưởng.

Chứng chỉ ở server được xin bằng **thử thách DNS-01**: certbot tạo bản ghi TXT tạm qua API Cloudflare, nên chạy được dù tên miền đang bật proxy và không cần mở cổng nào cho Let's Encrypt.

## 3. Tạo `.env`

```bash
sh deploy/init-env.sh
```

Script chép `.env.example` thành `.env` (quyền 600) và **tự sinh** `JWT_SECRET`, mật khẩu MongoDB (root, ứng dụng, chỉ-đọc) và mật khẩu quản trị blog. Sửa tay các dòng:

| Biến | Ví dụ |
|---|---|
| `DOMAIN` | `antblog.xyz` (đã điền sẵn) |
| `ADMIN_DOMAIN` | `admin.antblog.xyz` (đã điền sẵn) |
| `CERT_EXTRA_DOMAINS` | `www.antblog.xyz` (tuỳ chọn — nhớ tạo cả bản ghi DNS) |
| `ACME_EMAIL` | email nhận cảnh báo hết hạn |
| `CF_API_TOKEN` | token ở bước 2.4 — để trống được khi domain chưa chuyển sang Cloudflare: certbot nằm chờ, nginx dùng chứng chỉ tự ký |
| `WG_SERVER_URL` | `vpn.antblog.xyz` (đã điền sẵn) hoặc IP server |
| `WG_PEERS` | tên máy khách VPN, vd `laptop,desktop` |
| `SEED_ADMIN_EMAIL` | email đăng nhập trang quản trị (mặc định `admin@antblog.xyz`) |

Lần đầu nên đặt `ACME_STAGING=1` để thử luồng xin chứng chỉ mà không dính giới hạn tần suất của Let's Encrypt. Chạy ổn thì đổi về `0`, xoá chứng chỉ staging (`docker compose run --rm --entrypoint sh certbot -c 'rm -rf /etc/letsencrypt/live /etc/letsencrypt/archive /etc/letsencrypt/renewal'`) rồi `docker compose up -d certbot`.

## 4. Chạy

```bash
docker compose up -d --build
docker compose ps                    # mọi dịch vụ "healthy"/"running"
docker compose run --rm seed         # nạp chủ đề, bài viết, lộ trình + tài khoản quản trị
```

- Trang đọc: `https://antblog.xyz/` · Quản trị: `https://admin.antblog.xyz/` (email `SEED_ADMIN_EMAIL`, mật khẩu `SEED_ADMIN_PASSWORD` trong `.env`).
- Đăng nhập ở hai tên miền là hai phiên riêng (cookie gắn theo từng tên miền): đăng nhập trang quản trị không làm bạn đăng nhập ở trang đọc và ngược lại.
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
| Cập nhật code | `git pull` ⇒ `docker compose up -d --build` |
| Nạp bài mới / sửa từ `api/seed/content` | `docker compose run --rm --build seed` — bài đã sửa trong CMS được giữ nguyên; ép ghi đè: thêm `npm run seed -- --force-content` vào cuối lệnh |
| Sao lưu ảnh | `docker run --rm -v tech-knowledge-hub_uploads:/u:ro -v "$PWD":/b alpine tar czf /b/uploads-$(date +%F).tgz -C /u .` |
| Xem log | `docker compose logs -f api nginx` |
| Sao lưu DB | `docker compose exec mongo sh -c 'mongodump --archive --gzip -u "$MONGO_INITDB_ROOT_USERNAME" -p "$MONGO_INITDB_ROOT_PASSWORD" --authenticationDatabase admin --db tech_knowledge_hub' > backup-$(date +%F).archive.gz` |
| Khôi phục | `docker compose exec -T mongo sh -c 'mongorestore --archive --gzip --drop -u "$MONGO_INITDB_ROOT_USERNAME" -p "$MONGO_INITDB_ROOT_PASSWORD" --authenticationDatabase admin' < backup-....archive.gz` |
| Cập nhật dải IP Cloudflare | `sh deploy/nginx/update-cloudflare-ips.sh && docker compose up -d --build nginx` |
| Tạm chặn Google lập chỉ mục | `SITE_NOINDEX=true` trong `.env` ⇒ `docker compose up -d web` |
| Dừng | `docker compose down` (giữ dữ liệu) · `docker compose down -v` **xoá cả DB, ảnh và chứng chỉ** |

Lưu ý:
- Mật khẩu MongoDB trong `.env` chỉ có tác dụng ở **lần khởi tạo volume đầu tiên**. Đổi về sau phải dùng `db.changeUserPassword(...)` trong mongosh rồi mới sửa `.env`.
- nginx lấy IP thật của người đọc từ header `CF-Connecting-IP` (chỉ khi request đến từ dải IP của Cloudflare). Muốn chặn hẳn truy cập vào thẳng IP server, giới hạn TCP 80/443 ở tường lửa chỉ cho dải IP Cloudflare (`https://www.cloudflare.com/ips/`).
- nginx giữ HTML trang đọc 60 giây (micro-cache, header `X-Cache-Status: HIT|MISS|STALE`): bài vừa sửa/xuất bản trong CMS hiện ra trên trang đọc sau tối đa ~1 phút. Cần xoá ngay: `docker compose exec nginx sh -c 'rm -rf /var/cache/nginx/tkh_web/*'`.
- Cookie đăng nhập có cờ `Secure` (`COOKIE_SECURE=true`) ⇒ chỉ đăng nhập được qua HTTPS.

## 7. Kiểm tra ảnh bìa và cây danh mục sau deploy

Next.js standalone không tự đóng gói thư mục public. Dockerfile.web phải chép cả apps/web/public và .next/static vào cạnh server.js; thiếu public khiến /images/articles/*.webp trả 404 dù giao diện và API vẫn hoạt động. Build chạy covers:check trước khi tạo bundle.

Rebuild bản web sau khi kéo commit mới:

```bash
docker compose up -d --build web
docker compose exec nginx nginx -s reload
curl -I https://antblog.xyz/images/articles/page-don-vi-doc-ghi.webp
```

Mong đợi HTTP 200 và Content-Type: image/webp. Nếu CDN còn giữ phản hồi 404 cũ, purge cache cho /images/articles/* trên Cloudflare. Từ checkout có Node/dependencies, kiểm tra mọi ảnh qua HTTP:

```bash
npm run covers:check -w @tkh/web -- --base-url https://antblog.xyz
```

Cây danh mục tối đa 3 cấp, và chủ đề cha (có chủ đề con) không chứa bài viết riêng: bài chỉ gắn vào chủ đề lá, trang chủ đề cha chỉ liệt kê các chủ đề con. Cập nhật cây và chuyển bài theo `category` trong frontmatter bằng seed:

```bash
docker compose up -d --build api web nginx
docker compose run --rm --build seed
```

Seed cập nhật tên, mô tả, biểu tượng, thứ tự và quan hệ cha–con của chủ đề theo `api/seed/categories.json`, rồi chuyển bài sang chủ đề lá ghi trong frontmatter. Bài đã sửa trong CMS vẫn được giữ nguyên nội dung; riêng bài đang nằm ở chủ đề cha (trạng thái không hợp lệ) thì seed chỉ đổi chủ đề của bài, không đụng nội dung, tiêu đề, trạng thái hay cờ đã sửa trong CMS. Bài đang ở một chủ đề lá không bao giờ bị seed chuyển đi. Seed bỏ qua tệp .md có `category` là chủ đề cha và ghi log. Cuối lần chạy, seed ghi cảnh báo `CẢNH BÁO: … bài vẫn gắn trực tiếp vào chủ đề cha` kèm danh sách mẫu slug (ví dụ bài chỉ tạo trong CMS); hãy chuyển các bài đó sang chủ đề lá trong CMS.

Cần build lại cả `api`, `web` và `nginx`: API để kiểm tra chủ đề lá khi lưu bài, `web` cho trang chủ đề và ảnh bìa (ảnh banner nằm trong image `web`), `nginx` để nhận lại upstream.

Để đồng bộ cả cây danh mục, banner và phân loại bài theo bản rà soát mới, triển khai lại web (banner được đóng gói trong web) và chạy migration metadata dưới đây. Migration dùng slug bài trong seed để chuyển đúng danh mục lá, kể cả bài hiện ở một danh mục lá khác; không sửa nội dung, tiêu đề, trạng thái hoặc lộ trình học. Xem kỹ danh sách `moves` trước khi áp dụng. Bài chỉ có trong CMS không được tự đoán danh mục; nếu còn bài CMS ở danh mục cha, lệnh apply dừng trước khi ghi. Không sửa cây/bài trong CMS trong lúc migration chạy. Chạy lại an toàn nếu bị gián đoạn.

```bash
docker compose run --rm --build seed npm run categories:content
mkdir -p backups/categories
docker compose run --rm -v "$PWD/backups/categories:/app/.migration-backups" seed npm run categories:content -- --apply
```

Backup chứa cây trước thay đổi và ánh xạ danh mục của bài. Sau migration, kiểm tra `/topics/programming`, `/topics/python`, `/topics/location-routing`: trang cha chỉ có thẻ danh mục con; trang lá mới có bộ lọc và bài viết. Cache cây có thể cần tối đa 5 phút để làm mới.

Nếu chỉ cần cập nhật cấu trúc cây từ `categories.json` mà không nạp lại nội dung bài viết, dùng `categories:tree`. Lệnh này chỉ thêm chủ đề còn thiếu và đặt quan hệ cha–con, không đổi tên/mô tả và không chuyển bài viết:

```bash
docker compose run --rm --build seed npm run categories:tree
mkdir -p backups/categories
docker compose run --rm -v "$PWD/backups/categories:/app/.migration-backups" seed npm run categories:tree -- --apply
```

Lệnh đầu của categories:tree chỉ xem kế hoạch. Lệnh --apply thêm các chủ đề có trong categories.json mà DB chưa có và đặt quan hệ cha–con theo trường `parent` của từng mục. Backup các danh mục trước thay đổi nằm trên host trong backups/categories. Không chạy đồng thời với thao tác sửa cây trong CMS. Tài khoản inspector chỉ đọc, không dùng để migrate; container tools dùng tài khoản ứng dụng đã cấu hình. Sau cập nhật, cache danh mục của Next có thể cần tối đa 5 phút để làm mới.
