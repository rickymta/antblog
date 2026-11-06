# Bài 20 · API integration test với WebApplicationFactory: kiểm chứng pipeline .NET 10

Nội dung: `api/seed/content/dotnet10-webapplicationfactory-integration-test.md`. Target net10.0; chạy từ thư mục gốc repository.

```bash
dotnet test examples/toi-noi-dotnet/20-api-integration-tests
```

Restore package lần đầu cần mạng. Các test dùng dữ liệu giả và nguồn lực riêng; không dùng database production.

Suite 24 ca kiểm tra API 16–19. AuthTests dùng JWT handler thật với khóa ngẫu nhiên riêng của factory. Không chứng minh Kestrel/TLS/proxy hoặc identity provider production.
