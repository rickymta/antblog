# Bài 15 · Integration test: kiểm chứng JSON và filesystem của báo giá C#

Nội dung: `api/seed/content/csharp-integration-test-json-filesystem.md`. Target net10.0; chạy từ thư mục gốc repository.

```bash
dotnet test examples/toi-noi-dotnet/15-integration-test/QuoteArchive.Tests
```

Restore package lần đầu cần mạng. Các test dùng dữ liệu giả và nguồn lực riêng; không dùng database production.
