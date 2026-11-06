# Bài 25 · Cache, timeout, retry và rate limiting: giữ API .NET 10 ổn định

Nội dung: `api/seed/content/dotnet10-cache-timeout-retry-rate-limit.md`. Chạy từ gốc repository, SDK .NET 10.

```bash
dotnet test examples/toi-noi-dotnet/25-cache-resilience/Resilience.Tests
```

HTTP client trong test dùng handler giả, không gửi request ra ngoài. Cache chỉ dành bảng giá công khai; limiter lab không phải quota toàn cluster.
