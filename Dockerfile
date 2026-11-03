# Cổng vào duy nhất của stack (port 80 + 443): nginx phục vụ trang đọc (/), trang quản trị (/admin/)
# và proxy /api/ tới API. Build từ thư mục gốc monorepo tech-blog.

FROM node:22-alpine AS build
WORKDIR /repo
COPY package.json package-lock.json tsconfig.base.json ./
COPY apps/web/package.json apps/web/package.json
COPY apps/admin/package.json apps/admin/package.json
COPY packages/shared/package.json packages/shared/package.json
COPY packages/ui/package.json packages/ui/package.json
RUN --mount=type=cache,target=/root/.npm,sharing=locked npm ci
COPY apps ./apps
COPY packages ./packages
# Trang quản trị chạy dưới /admin/, liên kết "Xem trang đọc" cùng origin (VITE_SITE_URL rỗng).
RUN npm run build -w @tkh/web \
 && TKH_ADMIN_BASE=/admin/ VITE_SITE_URL= npm run build -w @tkh/admin

FROM nginx:1.27-alpine AS runtime
# openssl: tạo chứng chỉ tự ký tạm khi certbot chưa xin được chứng chỉ thật.
RUN apk add --no-cache openssl && rm -f /etc/nginx/conf.d/default.conf
COPY deploy/nginx/templates/ /etc/nginx/templates/
COPY deploy/nginx/cloudflare-realip.conf /etc/nginx/conf.d/cloudflare-realip.conf
COPY deploy/nginx/tkh-headers.conf /etc/nginx/snippets/tkh-headers.conf
COPY deploy/nginx/tkh-select-cert.sh /usr/local/bin/tkh-select-cert.sh
COPY deploy/nginx/40-tkh-ssl.sh /docker-entrypoint.d/40-tkh-ssl.sh
# Bỏ CRLF phòng khi build từ bản checkout trên Windows.
RUN sed -i 's/\r$//' /usr/local/bin/tkh-select-cert.sh /docker-entrypoint.d/40-tkh-ssl.sh \
 && chmod +x /usr/local/bin/tkh-select-cert.sh /docker-entrypoint.d/40-tkh-ssl.sh
COPY --from=build /repo/apps/web/dist /usr/share/nginx/html
COPY --from=build /repo/apps/admin/dist /usr/share/nginx/html/admin
EXPOSE 80 443
HEALTHCHECK --interval=15s --timeout=5s --start-period=10s --retries=5 \
  CMD wget -qO- http://127.0.0.1/healthz >/dev/null || exit 1
