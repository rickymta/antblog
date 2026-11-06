# Bài 17 · Validation, Problem Details và OpenAPI: hợp đồng API trong .NET 10

Nội dung: `api/seed/content/dotnet10-validation-problem-details-openapi.md`. Target net10.0; chạy từ thư mục gốc repository.

```bash
dotnet run --project examples/toi-noi-dotnet/17-validation-openapi -- --urls http://127.0.0.1:5197 --environment Development
```

Ví dụ API chưa là service DDD/Clean hoàn chỉnh. Test HTTP tự động nằm tại bài 20; không cần chạy server dev để chạy suite.

Pin Microsoft.OpenApi 2.7.5 cho dòng 2.x đã vá advisory GHSA-v5pm-xwqc-g5wc; ASP.NET packages dùng 10.0.10 tương thích runtime kiểm chứng.
