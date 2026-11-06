---
slug: dotnet10-efcore-postgresql-mapping-migration
title: "EF Core 10 và PostgreSQL: mapping, migration và truy vấn của Tới Nơi"
summary: "Bài 21: dùng Npgsql/EF Core 10 với PostgreSQL thật, mapping tiền và UTC, tạo migration có snapshot và kiểm chứng round-trip trong schema lab riêng."
category: csharp
level: intermediate
tags: [csharp, dotnet, dotnet-10, toi-noi, postgresql, ef-core]
language: vi
---

Đây là **bài 21** trong loạt **C# và .NET 10 qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Các ví dụ target `net10.0`, dùng dữ liệu giả và cùng code đã chạy kiểm chứng trong repository.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Gọi EnsureCreated rồi Migrate trên cùng database có phải một quy trình migration an toàn?</p>
<details><summary>Xem đáp án</summary><p>Không. EnsureCreated và migration phục vụ các đường tạo schema khác nhau; lab dùng migration nhất quán từ đầu.</p></details>
</div>

## Từ HTTP đúng tới dữ liệu lưu đúng

API báo giá từ chặng trước chưa lưu trạng thái. Bây giờ ta bắt đầu persistence: một chuyến có identity, khách, cước, thời điểm tạo và driver còn thiếu. EF Core ánh xạ object sang mô hình quan hệ; Npgsql là provider nối EF Core với PostgreSQL.

Ta dùng EF Core 10 và provider Npgsql 10, không phải EF6 cũ. Có DbContext không có nghĩa mọi biểu thức C# tự thành SQL tốt; mapping, query và transaction vẫn phải được nhìn và kiểm chứng. Bài này dùng PostgreSQL 18 thật cho lab, không dùng InMemory provider để suy ra hành vi cột/constraint.

Persistence project phục vụ bài 21–26 nên chứa cả các bảng sẽ được giải thích ở bài idempotency/Outbox. TripRow là persistence model để quan sát cơ chế; nó chưa phải Trip aggregate có hành vi ở chặng DDD. Các setter public của model này không thay định hướng encapsulation trong domain.

## Mapping là hợp đồng giữa hai hệ kiểu

Fare được cấu hình numeric(18,2), CreatedAt ánh xạ thời điểm có offset theo provider; lab ghi UTC. DriverId nullable, Status có độ dài, quan hệ Trip–Customer có foreign key và index theo CustomerId/CreatedAt. Cột numeric giữ độ chính xác theo scale, không tự chọn quy tắc làm tròn nghiệp vụ.

Ca round-trip cố ý ghi 14255.50 để kiểm khả năng lưu decimal có phần lẻ; nó không đổi policy cước nguyên VND từ bài 14. Nếu model nhận nhiều chữ số hơn scale, database có thể làm tròn theo kiểu cột, nên phải kiểm soát precision trước khi lưu khi hợp đồng yêu cầu.

UTC cần nhất quán tại ranh giới ghi/đọc. PostgreSQL timestamp with time zone không lưu lại tên múi giờ gốc của khách; nếu nghiệp vụ lịch tương lai cần timezone ID, phải có field riêng. Không suy ra kiểu cột thời gian đã giải mọi câu hỏi DST.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 1. Mapping và migration cần đi tới database thật; model C# đúng chưa chứng minh cột và constraint đã đúng."><defs><marker id="cs21-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Persistence model</text><text x="34" y="123" class="t-small">Kiểu + mapping</text><path d="M280 115 L305 115" class="f-none s-phys" stroke-width="2" marker-end="url(#cs21-0-arrow)"/><rect x="305" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="319" y="95" class="t-b">Migration SQL</text><text x="319" y="123" class="t-small">Schema + constraint</text><path d="M565 115 L590 115" class="f-none s-phys" stroke-width="2" marker-end="url(#cs21-0-arrow)"/><rect x="590" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="604" y="95" class="t-b">PostgreSQL</text><text x="604" y="123" class="t-small">Giá trị lưu thực tế</text></svg>
</div>
<figcaption>Hình 1. Mapping và migration cần đi tới database thật; model C# đúng chưa chứng minh cột và constraint đã đúng.</figcaption>
</figure>

## DbContext là một đơn vị theo dõi ngắn hạn

Context giữ change tracker, cấu hình mapping và đơn vị SaveChanges. Tạo context ngắn cho một use case hoặc phạm vi phù hợp, dispose sau khi dùng. Không dùng cùng một context cho nhiều thao tác song song rồi mong EF tự khóa giúp.

Trong test, instance ghi được dispose trước khi một instance mới đọc lại. Điều này tránh cache tracking che mất lỗi mapping và serialization provider. Bài 22 sẽ đi sâu vào tracking, projection và SQL; hiện chỉ dùng AsNoTracking cho query kiểm dữ liệu đã lưu.

Context đã có nhiều cơ chế giống Unit of Work, nhưng kiến trúc đích vẫn cần ranh giới application/domain rõ. Không vội bọc mọi method EF thành một generic repository chỉ để có thêm interface ở bài nhập môn dữ liệu.

## Migration là lịch sử có thể review

Migrations và ModelSnapshot được sinh bằng dotnet-ef 10.0.10 từ model, được giữ trong repository. Không viết EnsureCreated trong test rồi bỏ qua migration: ta muốn kiểm schema đúng theo đường mà ứng dụng sẽ dùng.

Test tạo schema tên ngẫu nhiên trong database lab, đặt SearchPath cho connection và bảng lịch sử migration trong schema đó, rồi MigrateAsync. Model không hard-code tên schema test; mỗi ca có không gian riêng. Khi xong chỉ dọn schema do fixture tạo, không gọi EnsureDeleted để xóa cả database.

Migration kế tiếp cần so với snapshot để sinh delta. Review SQL, thời gian khóa, dữ liệu có sẵn và kế hoạch rollback trước khi áp dụng production. Lab tự migrate lúc dựng fixture để kiểm chứng, không phải khuyến cáo mỗi replica production tranh nhau chạy migration lúc startup.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 2. Mỗi ca sở hữu một schema trong database lab, không xóa database hoặc dùng schema dữ liệu thật."><defs><marker id="cs21-1-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Test riêng</text><text x="34" y="123" class="t-small">Schema lab_GUID</text><path d="M280 115 L305 115" class="f-none s-phys" stroke-width="2" marker-end="url(#cs21-1-arrow)"/><rect x="305" y="65" width="260" height="100" rx="10" class="f-pk-soft s-pk"/><text x="319" y="95" class="t-b">Migrate + Save</text><text x="319" y="123" class="t-small">Các bước kiểm chứng</text><path d="M565 115 L590 115" class="f-none s-phys" stroke-width="2" marker-end="url(#cs21-1-arrow)"/><rect x="590" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="604" y="95" class="t-b">Context đọc mới</text><text x="604" y="123" class="t-small">Kiểm round-trip</text><rect x="295" y="35" width="280" height="150" rx="12" class="f-none s-pk" stroke-dasharray="5 4"/></svg>
</div>
<figcaption>Hình 2. Mỗi ca sở hữu một schema trong database lab, không xóa database hoặc dùng schema dữ liệu thật.</figcaption>
</figure>

## Ba ca có ý nghĩa gì?

Round-trip kiểm fare, instant UTC, DriverId null và không còn migration pending. Ca chạy migration lần nữa yêu cầu dữ liệu còn nguyên. Ca foreign key ghi một CustomerId không tồn tại phải thất bại; object có Guid hợp lệ về cú pháp không có nghĩa khách đó tồn tại.

**Bài tập:** nếu vừa dùng context ghi để đọc lại cùng entity, vì sao có thể bỏ sót lỗi database? Tại sao fixture không xóa cả database khi dọn?

<details><summary>Xem đáp án</summary><p>Tracker có thể trả instance đã có mà không tạo một phép round-trip độc lập đúng ý định kiểm tra. Fixture chỉ sở hữu schema GUID của mình; xóa cả database vượt phạm vi và có thể phá dữ liệu ca khác.</p></details>

## Chạy ví dụ và kiểm chứng

Cần PostgreSQL local và database riêng tên `toi_noi_dotnet_lab`. Thiết lập biến môi trường `TOI_NOI_PG` bằng connection string của lab trước khi chạy; không đưa mật khẩu vào source. README của persistence lab có hướng dẫn. Fixture từ chối database khác tên hoặc host ngoài localhost, không âm thầm skip khi thiếu kết nối; nó chỉ tạo và dọn schema GUID riêng.

Từ thư mục gốc repository:

```bash
dotnet test examples/toi-noi-dotnet/21-efcore-postgres/Persistence.Tests
```

Migration và snapshot nằm ở `examples/toi-noi-dotnet/21-efcore-postgres/Persistence/Migrations`. Để dùng CLI EF cục bộ, chuyển vào `examples/toi-noi-dotnet`, chạy `dotnet tool restore` rồi `dotnet ef migrations list --project 21-efcore-postgres/Persistence`. Không trộn EnsureCreated vào đường migration này.

### TripDb.cs — 21-efcore-postgres/Persistence/TripDb.cs

```csharp
using Microsoft.EntityFrameworkCore;
namespace ToiNoi.Persistence;
public sealed class CustomerRow
{
    public Guid Id { get; set; }
    public string Name { get; set; } = "";
    public List<TripRow> Trips { get; set; } = [];
}
public sealed class TripRow
{
    public Guid Id { get; set; }
    public Guid CustomerId { get; set; }
    public CustomerRow Customer { get; set; } = null!;
    public decimal Fare { get; set; }
    public string Status { get; set; } = "requested";
    public string? DriverId { get; set; }
    public DateTimeOffset CreatedAt { get; set; }
    public Guid Version { get; set; } = Guid.NewGuid();
}
public sealed class BookingRequestRow
{
    public Guid CustomerId { get; set; }
    public string Key { get; set; } = "";
    public decimal DistanceKm { get; set; }
    public Guid TripId { get; set; }
}
public sealed class OutboxRow
{
    public Guid Id { get; set; }
    public Guid TripId { get; set; }
    public string Payload { get; set; } = "";
    public bool Published { get; set; }
}
public sealed class InboxRow
{
    public string Consumer { get; set; } = "";
    public Guid EventId { get; set; }
}
public sealed class ReceiptRow
{
    public Guid EventId { get; set; }
    public Guid TripId { get; set; }
}
public sealed class TripDb(DbContextOptions<TripDb> options) : DbContext(options)
{
    public DbSet<CustomerRow> Customers => Set<CustomerRow>();
    public DbSet<TripRow> Trips => Set<TripRow>();
    public DbSet<BookingRequestRow> BookingRequests => Set<BookingRequestRow>();
    public DbSet<OutboxRow> Outbox => Set<OutboxRow>();
    public DbSet<InboxRow> Inbox => Set<InboxRow>();
    public DbSet<ReceiptRow> Receipts => Set<ReceiptRow>();
    protected override void OnModelCreating(ModelBuilder model)
    {
        model.Entity<CustomerRow>(e => { e.ToTable("customers"); e.HasKey(x => x.Id); e.Property(x => x.Name).HasMaxLength(120); });
        model.Entity<TripRow>(e =>
        {
            e.ToTable("trips"); e.HasKey(x => x.Id);
            e.Property(x => x.Fare).HasPrecision(18, 2);
            e.Property(x => x.Status).HasMaxLength(32);
            e.Property(x => x.DriverId).HasMaxLength(64);
            e.Property(x => x.Version).IsConcurrencyToken();
            e.HasIndex(x => new { x.CustomerId, x.CreatedAt });
            e.HasOne(x => x.Customer).WithMany(x => x.Trips).HasForeignKey(x => x.CustomerId);
        });
        model.Entity<BookingRequestRow>(e =>
        {
            e.ToTable("booking_requests"); e.HasKey(x => new { x.CustomerId, x.Key });
            e.Property(x => x.Key).HasMaxLength(128); e.Property(x => x.DistanceKm).HasPrecision(12, 6);
        });
        model.Entity<OutboxRow>(e => { e.ToTable("outbox"); e.HasKey(x => x.Id); e.Property(x => x.Payload).HasColumnType("text"); });
        model.Entity<InboxRow>(e => { e.ToTable("inbox"); e.HasKey(x => new { x.Consumer, x.EventId }); e.Property(x => x.Consumer).HasMaxLength(64); });
        model.Entity<ReceiptRow>(e => { e.ToTable("receipts"); e.HasKey(x => x.EventId); });
    }
}
```

### MappingTests.cs — 21-efcore-postgres/Persistence.Tests/MappingTests.cs

```csharp
using Microsoft.EntityFrameworkCore;
using ToiNoi.Persistence;
using Xunit;
namespace ToiNoi.LabTests;
public sealed class MappingTests
{
    [Fact]
    public async Task Migration_round_trip_preserves_decimal_utc_and_nullable_driver()
    {
        await using var lab = await PgLab.CreateAsync(); var customer = await lab.SeedAsync();
        Guid id;
        await using (var write = lab.Open())
        {
            var trip = new TripRow { Id = Guid.NewGuid(), CustomerId = customer, Fare = 14255.50m, CreatedAt = DateTimeOffset.UnixEpoch };
            write.Trips.Add(trip); await write.SaveChangesAsync(); id = trip.Id;
        }
        await using var read = lab.Open(); var restored = await read.Trips.AsNoTracking().SingleAsync(x => x.Id == id);
        Assert.Equal(14255.50m, restored.Fare); Assert.Equal(DateTimeOffset.UnixEpoch, restored.CreatedAt); Assert.Null(restored.DriverId);
        Assert.Empty(await read.Database.GetPendingMigrationsAsync());
    }
    [Fact]
    public async Task Migration_can_be_applied_again_without_losing_rows()
    {
        await using var lab = await PgLab.CreateAsync(); await lab.SeedAsync();
        await using var db = lab.Open(); await db.Database.MigrateAsync(); Assert.Equal(3, await db.Trips.CountAsync());
    }
    [Fact]
    public async Task Foreign_key_rejects_unknown_customer()
    {
        await using var lab = await PgLab.CreateAsync(); await using var db = lab.Open();
        db.Trips.Add(new TripRow { Id = Guid.NewGuid(), CustomerId = Guid.NewGuid(), CreatedAt = DateTimeOffset.UnixEpoch });
        await Assert.ThrowsAsync<DbUpdateException>(() => db.SaveChangesAsync());
    }
}
```

Bài trước: [bài 20](/articles/dotnet10-webapplicationfactory-integration-test). Bài tiếp: [bài 22](/articles/dotnet10-efcore-tracking-projection-n-plus-one).

## Tài liệu gốc

- [EF Core 10](https://learn.microsoft.com/en-us/ef/core/what-is-new/ef-core-10.0/whatsnew).
- [Npgsql EF Core 10](https://www.npgsql.org/efcore/release-notes/10.0.html).
- [Migrations overview](https://learn.microsoft.com/en-us/ef/core/managing-schemas/migrations/).
- [Npgsql date/time mapping](https://www.npgsql.org/doc/types/datetime.html).
