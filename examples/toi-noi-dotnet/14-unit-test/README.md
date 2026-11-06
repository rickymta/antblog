# Bài 14 · Unit test và test double: kiểm thử chính sách cước C# bằng xUnit

Nội dung: `api/seed/content/csharp-unit-test-test-double.md`. Target net10.0; chạy từ thư mục gốc repository.

```bash
dotnet test examples/toi-noi-dotnet/14-unit-test/FareRules.Tests
```

Restore package lần đầu cần mạng. Các test dùng dữ liệu giả và nguồn lực riêng; không dùng database production.
