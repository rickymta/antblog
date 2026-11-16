# Chụp ảnh DBStudio cho series Cassandra

Ảnh trong bài là **giao diện DBStudio thật** nối với **backend DBStudio thật** và lab Cassandra thật. Không dựng lại hay chỉnh sửa giao diện; chỉ ẩn nhãn `bridge` ở thanh trạng thái (chỉ có ở bản dev) và bản đồ thu nhỏ của sơ đồ ERD.

## Cách hoạt động

```text
Playwright (Edge headless) ──► giao diện DBStudio (Vite, cổng 1440)
                                   │  IPC qua HTTP ("dev bridge")
                                   ▼
                              backend DBStudio 0.1.7 (debug build + feature dev-bridge, cổng 1441,
                              thư mục dữ liệu riêng)  ──►  Cassandra lab 127.0.0.1:19042
```

Bridge chỉ chấp nhận origin `http://localhost:1420`. Bộ chụp chạy giao diện ở cổng khác (1420 thường đang bận bởi `pnpm tauri dev`), nên đặt lại header `Origin` cho các request tới bridge và tắt kiểm tra CORS của trình duyệt headless. Chỉ dùng trên máy phát triển.

## Dựng môi trường (mã nguồn DBStudio ở `../dbstudio`, thẻ `v0.1.7`)

```sh
# 1. backend có dev bridge, thư mục dữ liệu riêng (không đụng kết nối thật của bạn)
cd dbstudio && pnpm build && cargo build -p dbstudio --features dev-bridge
DBSTUDIO_DATA_DIR=/tmp/dbs-shots DBSTUDIO_DEV_BRIDGE_PORT=1441 \
DBSTUDIO_DEV_BRIDGE_TOKEN=<chuỗi ngẫu nhiên ≥ 32 ký tự> ./target/debug/dbstudio &

# 2. giao diện trỏ vào bridge
VITE_IPC_BRIDGE_URL=http://127.0.0.1:1441 VITE_IPC_BRIDGE_TOKEN=<cùng chuỗi> \
  node_modules/.bin/vite --port 1440 --strictPort &

# 3. lab có dữ liệu P01, rồi chụp
cd tech-blog/examples/toi-noi-cassandra
npm run db:up && npm run p01
npm run shots              # hoặc: cd shots && node shoot.mjs cs-006
```

Cần Microsoft Edge (Playwright dùng `channel: 'msedge'`, không tải trình duyệt riêng). Ảnh được ghi dạng WebP, độ phân giải gấp đôi khung nhìn (1280 × 760 → 2560 × 1520).

## Thêm ảnh

Mỗi phần có một file kịch bản (`p01.mjs` …) gồm các mục `{ id, run }`. `id` là `cs-0NN/NN-ten-anh` và cũng là đường dẫn ảnh. Helper trong `lib.mjs`: `newPage`, `openConnection`, `expandTables`, `tableMenu(page, bảng, 'Xem DDL' | 'Mở dữ liệu' | …)`, `keyspaceMenu(page, 'Xem sơ đồ')`, `runCql(page, cql, { all })`, `shot(page, file, { locator | clip })`.

Kịch bản không được ghi vào database (trừ câu ghi có điều kiện chắc chắn không áp dụng), để chụp lại bao nhiêu lần cũng ra cùng ảnh.

## Giới hạn đã biết của DBStudio 0.1.7 với Cassandra

- Nút **Giải thích kế hoạch** (Ctrl+E) chỉ hiện với dialect SQL; với CQL bị ẩn dù backend có hỗ trợ query tracing. Bài về tracing (CS-028) dùng `TRACING ON` trong cqlsh cho tới khi DBStudio mở tính năng này.
- Lưới kết quả hiển thị `timestamp` theo UTC dạng ISO (`2026-11-02T00:42:00.000Z`), trong khi cqlsh của lab hiển thị theo múi giờ của container (`+0700`).
