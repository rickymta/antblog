---
slug: csharp-integration-test-json-filesystem
title: "Integration test: kiểm chứng JSON và filesystem của báo giá C#"
summary: "Bài 15: dùng rule thật, serializer thật và file tạm để kiểm tra round-trip báo giá; phân biệt lỗi I/O, JSON lỗi và giới hạn transaction của file demo."
category: programming
level: intermediate
tags: [csharp, dotnet, dotnet-10, toi-noi]
language: vi
---

Đây là **bài 15** trong loạt **C# và .NET 10 qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Các ví dụ target `net10.0`; code trong repository và các đoạn đầy đủ dưới đây dùng cùng implementation được chạy kiểm chứng.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Thay cả filesystem bằng một fake Dictionary có còn kiểm tra được quyền đọc/ghi hoặc JSON trên đĩa không?</p>
<details><summary>Xem đáp án</summary><p>Không. Fake có thể hữu ích cho unit test nhưng đã bỏ chính ranh giới tích hợp cần chứng minh ở bài này.</p></details>
</div>

## Rule đúng nhưng dữ liệu đọc lại khác

Unit test chứng minh 2,501 km có cước 14.255 VND. Nhưng nếu cước bị serialize thành chuỗi theo culture rồi đọc sai, hoặc thời điểm mất offset, dữ liệu lưu trữ vẫn lỗi. Integration test ở đây ghép rule thật, System.Text.Json và filesystem thật trong thư mục tạm.

Phạm vi được gọi tên cụ thể: đây là test tích hợp archive JSON/file, chưa phải PostgreSQL, chưa đo fsync/durability khi mất điện, chưa kiểm tra quyền của account production. Không dùng một nhãn integration để ngụ ý mọi hạ tầng đã được kiểm chứng.

QuoteSnapshot có Id, Fare và IssuedAt. Ta ghi snapshot có cước từ StandardFarePolicy, sau đó tạo instance JsonQuoteArchive mới để đọc. Đọc qua object mới ngăn một implementation chỉ trả cache trong cùng object vượt test mà chưa thực sự lấy dữ liệu file.

## Mỗi ca có thư mục riêng

Test tạo thư mục trong Path.GetTempPath với tiền tố toi-noi và GUID. Tên file quote.json cố định bên trong thư mục đó; không nhận path từ dữ liệu khách. Khi test kết thúc, Dispose dọn đúng thư mục được test tạo.

Mỗi instance test có state riêng; các ca chạy đồng thời không giẫm cùng một file. Không dùng một đường dẫn cố định trên desktop, không yêu cầu sửa file của người dùng và không đưa dữ liệu giấy tờ thật vào fixture.

Cleanup là một phần thiết kế fixture. Nếu mở file mà không dispose, hệ điều hành có thể không cho dọn; lỗi dọn là dấu hiệu cần xem chứ không vội tắt test. Tính đúng nghiệp vụ và quản lý tài nguyên cần được quan sát cùng nhau.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 1. Integration test đi qua serializer và file thật, khác unit test dùng storage giả."><defs><marker id="cs15-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Domain policy</text><text x="34" y="123" class="t-small">Giá decimal thật</text><path d="M280 115 L305 115" class="f-none s-phys" stroke-width="2" marker-end="url(#cs15-0-a)"/><rect x="305" y="65" width="260" height="100" rx="10" class="f-pk-soft s-pk"/><text x="319" y="95" class="t-b">JSON + file</text><text x="319" y="123" class="t-small">Ranh giới tích hợp</text><path d="M565 115 L590 115" class="f-none s-phys" stroke-width="2" marker-end="url(#cs15-0-a)"/><rect x="590" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="604" y="95" class="t-b">Instance đọc mới</text><text x="604" y="123" class="t-small">Snapshot phục hồi</text><rect x="295" y="35" width="280" height="150" rx="12" class="f-none s-pk" stroke-dasharray="5 4"/></svg>
</div>
<figcaption>Hình 1. Integration test đi qua serializer và file thật, khác unit test dùng storage giả.</figcaption>
</figure>

## Round-trip kiểm tra nhiều hơn “file có tồn tại”

SaveAsync serialize QuoteSnapshot bằng System.Text.Json rồi ghi text. ReadAsync đọc text và deserialize. Test so toàn bộ record, số tiền 14.255 và offset UTC. Chỉ assert File.Exists không phát hiện field mất, kiểu tiền đổi hoặc object đọc lại không cùng dữ liệu.

JSON null được chuyển thành InvalidDataException; JSON sai cú pháp tạo JsonException; thiếu file tạo FileNotFoundException. Ta không gom cả ba thành một báo giá 0 VND. CancellationToken được truyền vào API file để người gọi có thể hủy phần việc còn hợp tác được.

Bộ test có ca token hủy từ trước khi ghi và kiểm tra byte cũ không đổi theo hành vi API được dùng. Kết quả đó không chứng minh một thao tác đang ghi dở có rollback an toàn. Cần phân biệt chính xác điểm hủy được kiểm tra.

## Đây chưa là Repository/Unit of Work production

WriteAllTextAsync ghi trực tiếp một file; nó không cung cấp transaction nhiều object, concurrency giữa process, atomic replace hoặc cơ chế phục hồi khi process chết giữa chừng. Trong bài không có nhiều writer cùng path. Đừng đặt tên class là UnitOfWork rồi suy ra đã có những đảm bảo đó.

Khi tới EF Core/PostgreSQL, test sẽ chạy database thật có transaction, unique constraint, optimistic concurrency và migration. SQLite/in-memory provider cũng không tự chứng minh hành vi PostgreSQL. Mỗi công cụ chỉ có giá trị cho ranh giới mà nó thực thi.

Ở kiến trúc đích Tới Nơi, abstraction lưu trữ đứng sau use case và implementation hạ tầng được test theo hợp đồng. Archive nhỏ hiện tại chỉ tạo nền để hiểu vì sao unit test và integration test bổ sung nhau.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 2. Các failure mode không được biến thành một giá trị tiền mặc định."><defs><marker id="cs15-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Không có file</text><text x="34" y="123" class="t-small">FileNotFoundException</text><rect x="305" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="319" y="95" class="t-b">JSON hỏng</text><text x="319" y="123" class="t-small">JsonException</text><rect x="590" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="604" y="95" class="t-b">JSON null</text><text x="604" y="123" class="t-small">InvalidDataException</text></svg>
</div>
<figcaption>Hình 2. Các failure mode không được biến thành một giá trị tiền mặc định.</figcaption>
</figure>

## Bốn tình huống và bài tập

Bốn ca gồm round-trip thật, thiếu file, JSON hỏng và pre-cancelled write giữ dữ liệu đã có. Các assertion dựa vào đầu ra và exception theo hợp đồng, không kiểm private field của serializer.

**Bài tập:** nếu thay SaveAsync bằng chỉ gán một field trong memory, round-trip qua instance mới có qua không? Muốn bảo vệ trước process crash giữa lần ghi, cần thêm loại thí nghiệm nào?

<details><summary>Xem đáp án</summary><p>Không, instance đọc mới không có dữ liệu và file mong đợi chưa được ghi. Crash consistency cần cơ chế ghi/commit phù hợp cùng test gián đoạn; không suy ra từ test happy path hoặc token đã hủy trước khi mở file.</p></details>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
dotnet test examples/toi-noi-dotnet/15-integration-test/QuoteArchive.Tests
```

Lần đầu cần restore các package test từ NuGet. Project được pin phiên bản trong .csproj; mỗi ca test có dữ liệu và tài nguyên riêng để chạy lại được.

### JsonQuoteArchive.cs — 15-integration-test/QuoteArchive/JsonQuoteArchive.cs

```csharp
using System.Text.Json;
namespace ToiNoi.Archive;
public sealed record QuoteSnapshot(string Id, decimal Fare, DateTimeOffset IssuedAt);
// One trusted file path per test/use; not a production repository or transaction boundary.
public sealed class JsonQuoteArchive(string filePath)
{
    public async Task SaveAsync(QuoteSnapshot quote, CancellationToken ct)
    {
        await File.WriteAllTextAsync(filePath, JsonSerializer.Serialize(quote), ct);
    }
    public async Task<QuoteSnapshot> ReadAsync(CancellationToken ct)
    {
        var json = await File.ReadAllTextAsync(filePath, ct);
        return JsonSerializer.Deserialize<QuoteSnapshot>(json)
            ?? throw new InvalidDataException("Snapshot was null.");
    }
}
```

### ArchiveTests.cs — 15-integration-test/QuoteArchive.Tests/ArchiveTests.cs

```csharp
using ToiNoi.Archive;
using ToiNoi.Fares;
using Xunit;
namespace ToiNoi.ArchiveTests;
public sealed class ArchiveTests : IDisposable
{
    private readonly string directory = Path.Combine(Path.GetTempPath(), "toi-noi-" + Guid.NewGuid().ToString("N"));
    private string FilePath => Path.Combine(directory, "quote.json");
    public ArchiveTests() => Directory.CreateDirectory(directory);
    [Fact]
    public async Task Real_serialization_and_file_round_trip()
    {
        var quote = new QuoteSnapshot("TN-1", new StandardFarePolicy().Calculate(2.501m),
            new DateTimeOffset(2026, 11, 6, 1, 30, 0, TimeSpan.Zero));
        await new JsonQuoteArchive(FilePath).SaveAsync(quote, CancellationToken.None);
        var restored = await new JsonQuoteArchive(FilePath).ReadAsync(CancellationToken.None);
        Assert.Equal(quote, restored);
        Assert.Equal(14255m, restored.Fare);
        Assert.Equal(TimeSpan.Zero, restored.IssuedAt.Offset);
        Assert.True(File.Exists(FilePath));
    }
    [Fact]
    public async Task Missing_file_is_not_a_free_quote() =>
        await Assert.ThrowsAsync<FileNotFoundException>(() => new JsonQuoteArchive(FilePath).ReadAsync(CancellationToken.None));
    [Fact]
    public async Task Malformed_json_fails_at_serialization_boundary()
    {
        await File.WriteAllTextAsync(FilePath, "not-json");
        await Assert.ThrowsAsync<System.Text.Json.JsonException>(() => new JsonQuoteArchive(FilePath).ReadAsync(CancellationToken.None));
    }
    [Fact]
    public async Task Cancelled_write_does_not_replace_existing_bytes()
    {
        await File.WriteAllTextAsync(FilePath, "original");
        using var cts = new CancellationTokenSource();
        cts.Cancel();
        await Assert.ThrowsAnyAsync<OperationCanceledException>(() => new JsonQuoteArchive(FilePath)
            .SaveAsync(new("TN-1", 25500m, DateTimeOffset.UnixEpoch), cts.Token));
        Assert.Equal("original", await File.ReadAllTextAsync(FilePath));
    }
    public void Dispose() => Directory.Delete(directory, recursive: true);
}
```

Bài trước: [bài 14](/articles/csharp-unit-test-test-double). Bài tiếp: [bài 16](/articles/dotnet10-http-minimal-api).

## Tài liệu gốc

- [System.Text.Json](https://learn.microsoft.com/en-us/dotnet/standard/serialization/system-text-json/overview).
- [File.WriteAllTextAsync](https://learn.microsoft.com/en-us/dotnet/api/system.io.file.writealltextasync).
- [Integration tests](https://learn.microsoft.com/en-us/aspnet/core/test/integration-tests?view=aspnetcore-10.0).
