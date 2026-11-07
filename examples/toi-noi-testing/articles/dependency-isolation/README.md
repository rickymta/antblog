# TEST-058 · Dependency Isolation

Chạy từ `examples/toi-noi-testing`, sau `npm ci`:

```bash
npx vitest run articles/dependency-isolation
```

17 ca: 8 biên giờ VN, weather thành công/lỗi, 2 biên hạn báo giá, 3 ca bộ quét tìm tài xế, 2 ca timer thanh toán. Dùng FixedClock cho thời gian nghiệp vụ; dùng fake timers cho setTimeout. Không sleep, không DB hay mạng.

Kiểm độc lập múi giờ trên PowerShell (khôi phục biến môi trường sau khi chạy):

```powershell
$previousTz = $env:TZ
try {
  $env:TZ = 'UTC'
  npx vitest run articles/dependency-isolation
  $env:TZ = 'America/Los_Angeles'
  npx vitest run articles/dependency-isolation
} finally { $env:TZ = $previousTz }
```

Bộ quét được gọi tường minh; không kiểm scheduler 15 giây của server hay tranh chấp nhiều tiến trình. Ca weather lỗi mô tả hành vi code hiện tại, không tự thêm chính sách fallback vào SPEC. Ví giả chỉ mô phỏng idempotency, không chứng minh hợp đồng ví bên ngoài.
