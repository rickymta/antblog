# Bài 19 · Authentication và authorization: khách, tài xế và điều phối trong .NET 10

Nội dung: `api/seed/content/dotnet10-authentication-authorization.md`. Target net10.0; chạy từ thư mục gốc repository.

```bash
dotnet run --project examples/toi-noi-dotnet/19-authentication-authorization -- --urls http://127.0.0.1:5199
```

Ví dụ API chưa là service DDD/Clean hoàn chỉnh. Test HTTP tự động nằm tại bài 20; không cần chạy server dev để chạy suite.

Không có khóa/token cố định trong source. Bài viết hướng dẫn dotnet user-secrets và dotnet user-jwts cho Development. API production cần issuer/authority, audience và trust được cấu hình riêng. AuthFactory bài 20 chỉ dành cho test; dùng khóa ngẫu nhiên trong memory.
