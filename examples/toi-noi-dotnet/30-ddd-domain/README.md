# Bài 30 · DDD: bounded context, aggregate và invariant của chuyến Tới Nơi

Nội dung: `api/seed/content/dotnet10-ddd-bounded-context-aggregate.md`. Chạy từ gốc repository, SDK .NET 10.

```bash
dotnet test examples/toi-noi-dotnet/30-ddd-domain/Domain.Tests
```

Domain thuần, không tham chiếu EF/ASP.NET. Persistence model bài 21 và aggregate bài 30 là hai mô hình khác; mapping được ghép ở chặng kiến trúc sau.
