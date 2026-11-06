# Bài 22 · Tracking, projection và N+1: đọc SQL trước khi tối ưu EF Core

Nội dung: `api/seed/content/dotnet10-efcore-tracking-projection-n-plus-one.md`. Chạy từ gốc repository, SDK .NET 10.

```bash
dotnet test examples/toi-noi-dotnet/22-tracking-projection
```

Cần TOI_NOI_PG trỏ tới database local riêng `toi_noi_dotnet_lab`; xem README của bài 21. Test không dùng SQLite/InMemory và không skip nếu thiếu kết nối. Mỗi ca dùng schema GUID và dọn đúng schema của nó.
