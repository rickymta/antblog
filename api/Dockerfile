# API Tech Knowledge Hub — build từ thư mục gốc repo tech-blog-api.
# Stage `tools` (có devDependencies + seed) dùng cho `docker compose run --rm seed`.
# Stage `runtime` (chỉ dependencies chạy) là ảnh API thật.

FROM node:22-alpine AS deps
WORKDIR /app
COPY package.json package-lock.json ./
RUN --mount=type=cache,target=/root/.npm,sharing=locked npm ci

FROM deps AS tools
COPY tsconfig.json tsconfig.build.json ./
COPY src ./src
COPY seed ./seed
RUN npm run build

FROM node:22-alpine AS runtime
ENV NODE_ENV=production
WORKDIR /app
COPY package.json package-lock.json ./
RUN --mount=type=cache,target=/root/.npm,sharing=locked npm ci --omit=dev && npm cache clean --force
COPY --from=tools /app/dist ./dist
USER node
EXPOSE 4000
HEALTHCHECK --interval=15s --timeout=5s --start-period=20s --retries=5 \
  CMD wget -qO- http://127.0.0.1:4000/api/v1/site/summary >/dev/null || exit 1
CMD ["node", "dist/server.js"]
