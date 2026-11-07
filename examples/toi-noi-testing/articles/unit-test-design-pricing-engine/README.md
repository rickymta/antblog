# TEST-056 · Unit Test Design cho Pricing Engine

Chạy từ `examples/toi-noi-testing` sau `npm ci`:

```bash
npx vitest run articles/unit-test-design-pricing-engine
```

36 ca: 14 biên/đầu vào số, 8 tổ hợp quyết định, 8 oracle/tiền tệ, 6 schema/điều phối. Mọi expected là hằng số tính từ SPEC, không gọi hàm sản phẩm để dựng oracle. Đơn vị đầu vào của `roundUpToThousandDong` là milli-đồng.

Không có HTTP hay DB. Test schema không chứng minh HTTP status; test pricing không chứng minh tra cứu thời tiết, giờ cao điểm hoặc hiệu lực voucher. Voucher hợp lệ trong SPEC có mức giảm 1–50%; bài kiểm trường hợp giảm xuống dưới cước sàn, không tạo voucher 200% trái hợp đồng.
