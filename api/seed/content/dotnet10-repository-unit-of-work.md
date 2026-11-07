---
slug: dotnet10-repository-unit-of-work
title: "Repository và Unit of Work với EF Core: aggregate, tracking và commit"
summary: "Bài 33: repository theo Trip aggregate, read port riêng, mapping domain event vào Outbox và một điểm commit local; không bọc mọi API EF thành CRUD generic."
category: csharp-advanced
level: advanced
tags: [csharp, dotnet, dotnet-10, toi-noi, architecture]
language: vi
---

Đây là **bài 33** của loạt **C# và .NET 10 qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Bài 31–38 cùng giải thích code cuối của service tại `examples/toi-noi-dotnet/31-38-trip-service`, target `net10.0`.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Repository gọi SaveChanges ở mỗi Add rồi UoW gọi thêm lần nữa có giữ được một commit chung không?</p>
<details><summary>Xem đáp án</summary><p>Không. Commit rải rác phá ranh giới nguyên tử của use case; lab chỉ commit ở UoW sau khi đã gom các thay đổi.</p></details>
</div>

## Repository là port cho aggregate, không phải DbSet đổi tên

ITripRepository chỉ có GetAsync và Add cho root Trip. Nó không trả IQueryable để caller tùy ý gắn EF query vào domain. ITripQueries dành cho DTO đọc; IBookingRequests giữ receipt của ý định request ở application boundary.

Một generic repository với hàng chục method phản chiếu toàn bộ EF có thể tăng lớp mà không giảm phụ thuộc. Port trong bài được chọn theo hành vi cần của service. Repository không gửi event ra mạng và Add chưa commit.

UoW là ranh giới flush cùng transaction local. EF DbContext đã có tracking và SaveChanges; abstraction của lab thêm việc đồng bộ aggregate/persistence row và đưa integration intent vào Outbox trước đúng commit đó.

## Một aggregate instance trong một scope

Repository giữ cặp aggregate/row theo ID. Đọc lại cùng ID trong scope trả aggregate đã có; row EF giữ original Version để optimistic concurrency hoạt động. Restore không làm tăng version hoặc phát TripRequested như tạo mới.

Trong CommitAsync, UoW chép trạng thái hợp lệ từ aggregate vào row đang tracked, không attach một graph lạ rồi đánh dấu mọi field Modified. Money/Distance được dựng với rule của domain, không nhận setter tự do từ request body.

Cache này chỉ sống trong scope, không dùng static dictionary toàn service. Nó không thay transaction isolation hay lock ở PostgreSQL; hai request khác scope vẫn có thể giữ hai phiên bản của cùng chuyến.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Hình 1. Repository theo dõi cặp trong một scope; không phải cache toàn cluster."><defs><marker id="cs33-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="25" width="250" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="55" class="t-b">Aggregate instance</text><text x="34" y="83" class="t-small">Rule + event</text><rect x="20" y="165" width="250" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="195" class="t-b">Persistence row</text><text x="34" y="223" class="t-small">Original Version</text><rect x="520" y="95" width="330" height="85" rx="10" class="f-pk-soft s-pk"/><text x="534" y="125" class="t-b">Cặp theo dõi scoped</text><text x="534" y="153" class="t-small">Aggregate ↔ EF row</text><path d="M270 67 L520 115" class="f-none s-phys" stroke-width="2" marker-end="url(#cs33-0-a)"/><path d="M270 207 L520 155" class="f-none s-phys" stroke-width="2" marker-end="url(#cs33-0-a)"/></svg>
</div>
<figcaption>Hình 1. Repository theo dõi cặp trong một scope; không phải cache toàn cluster.</figcaption>
</figure>

## Gom event và dữ liệu trước SaveChanges

UoW map TripRequested/TripAccepted sang envelope integration version v1 có payload tối thiểu. Domain event không serialize nguyên graph, request key hoặc token. Outbox row và dữ liệu Trip/request receipt được ghi trong cùng SaveChanges.

Sau commit thành công mới clear PendingEvents. Nếu commit lỗi, scope sẽ kết thúc với lỗi; không clear rồi giả vờ đã có intent bền. Nếu muốn retry cùng context sau một failure, cần thiết kế reset trạng thái tracker/outbox rõ; lab không tự retry commit tùy ý.

Service này lưu intent, còn dispatcher/broker/Inbox ngoài process vẫn phải được ghép theo bài 26. Có Outbox table không đồng nghĩa mọi thông báo đã được gửi hay consumer đã hoàn tất.

## Dịch lỗi provider ở adapter

DbUpdateConcurrencyException được dịch thành ConcurrencyConflictException của application. Unique violation chỉ được coi là duplicate request khi SQLSTATE và constraint name pk_requests khớp; các lỗi khác không bị nuốt.

Version int là concurrency token do ứng dụng cập nhật qua domain Accept. UPDATE dùng original Version; hai người cùng nhận chỉ một người có thể commit theo invariant. Không bật retry mù để dùng version mới ghi đè tài xế đã thắng.

Transaction local không bao network. Nếu sau này thêm nhiều SaveChanges cho cùng một use case, phải xem lại ranh giới transaction và execution strategy, không mặc định tên UoW tự làm tất cả nguyên tử.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 2. Không commit ở từng Add hoặc clear event trước khi dữ liệu bền."><defs><marker id="cs33-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Gom thay đổi</text><text x="34" y="123" class="t-small">Trip + request + event</text><path d="M280 108 L305 108" class="f-none s-phys" stroke-width="2" marker-end="url(#cs33-1-a)"/><rect x="305" y="65" width="260" height="85" rx="10" class="f-pk-soft s-pk"/><text x="319" y="95" class="t-b">Một SaveChanges</text><text x="319" y="123" class="t-small">Transaction local</text><path d="M565 108 L590 108" class="f-none s-phys" stroke-width="2" marker-end="url(#cs33-1-a)"/><rect x="590" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="604" y="95" class="t-b">Commit thành công</text><text x="604" y="123" class="t-small">Clear pending event</text></svg>
</div>
<figcaption>Hình 2. Không commit ở từng Add hoặc clear event trước khi dữ liệu bền.</figcaption>
</figure>

## Bài tập

**Bài tập:** nếu lắp ITripRepository và EfUnitOfWork với hai instance repository khác nhau, điều gì có thể mất? Vì sao query GetTrip không cần đi qua dictionary aggregate?

<details><summary>Xem đáp án</summary><p>UoW có thể không nhìn thấy aggregate/event đã thay đổi để flush. Query chỉ đọc DTO, không thực hiện hành vi nên dùng projection có filter ownership riêng là phù hợp.</p></details>

## Code và kiểm chứng

Thiết lập `TOI_NOI_PG` cho database local riêng `toi_noi_dotnet_lab` theo README bài 21, rồi từ gốc repository:

```bash
dotnet test examples/toi-noi-dotnet/31-38-trip-service/Tests
```

Fixture dùng schema GUID riêng, không xóa database; JWT dùng khóa ngẫu nhiên trong test và vẫn đi qua handler xác thực thật. Test không tự skip khi thiếu PostgreSQL. Docker/CI/cloud spec xem README service và cloud/README.md để biết phạm vi, prerequisites và phần chưa được triển khai thật.

### Persistence.cs — 31-38-trip-service/Infrastructure/Persistence.cs

```csharp
using System.Text.Json;
using Microsoft.EntityFrameworkCore;
using Npgsql;
using ToiNoi.Trips.Application;
using ToiNoi.Trips.Domain;
namespace ToiNoi.Trips.Infrastructure;
public sealed class TripRepository(ServiceDb db) : ITripRepository
{
    internal Dictionary<Guid, (Trip Aggregate, TripRow Row)> Tracked { get; } = [];
    public async Task<Trip?> GetAsync(Guid id, CancellationToken ct)
    {
        if (Tracked.TryGetValue(id, out var existing)) return existing.Aggregate;
        using var activity = TripDiagnostics.Source.StartActivity("db.trip.load"); activity?.SetTag("db.system.name", "postgresql");
        var row = await db.Trips.SingleOrDefaultAsync(x => x.Id == id, ct); if (row is null) return null;
        var trip = Trip.Restore(row.Id, row.CustomerId, new DistanceKm(row.DistanceKm), new Money(row.Fare), row.CreatedAt,
            Enum.Parse<TripStatus>(row.Status), row.DriverId, row.Version);
        Tracked.Add(id, (trip, row)); return trip;
    }
    public void Add(Trip trip)
    {
        var row = new TripRow { Id = trip.Id, CustomerId = trip.CustomerId, DistanceKm = trip.Distance.Value, Fare = trip.Fare.Amount, CreatedAt = trip.CreatedAt };
        db.Trips.Add(row); Tracked.Add(trip.Id, (trip, row));
    }
}
public sealed class BookingRequests(ServiceDb db) : IBookingRequests
{
    public Task<BookingReceipt?> FindAsync(Guid customerId, string key, CancellationToken ct) => db.Requests.AsNoTracking()
        .Where(x => x.CustomerId == customerId && x.Key == key).Select(x => new BookingReceipt(x.CustomerId, x.Key, x.DistanceKm, x.TripId, x.Fare)).SingleOrDefaultAsync(ct);
    public void Add(BookingReceipt r) => db.Requests.Add(new RequestRow { CustomerId = r.CustomerId, Key = r.Key, DistanceKm = r.DistanceKm, TripId = r.TripId, Fare = r.Fare });
}
public sealed class TripQueries(ServiceDb db) : ITripQueries
{
    public Task<TripView?> FindOwnedAsync(Guid tripId, Guid customerId, CancellationToken ct) => db.Trips.AsNoTracking()
        .Where(x => x.Id == tripId && x.CustomerId == customerId)
        .Select(x => new TripView(x.Id, x.CustomerId, x.Fare, "VND", x.Status, x.DriverId, x.Version)).SingleOrDefaultAsync(ct);
}
public sealed class EfUnitOfWork(ServiceDb db, TripRepository repository, TimeProvider clock) : IUnitOfWork
{
    public async Task CommitAsync(CancellationToken ct)
    {
        foreach (var (trip, row) in repository.Tracked.Values)
        {
            row.Status = trip.Status.ToString(); row.DriverId = trip.DriverId; row.Version = trip.Version;
            foreach (var message in trip.PendingEvents)
            {
                (string type, string payload) = message switch
                {
                    TripRequested requested => ("trip.requested.v1", JsonSerializer.Serialize(new { requested.TripId, requested.Fare, Currency = "VND" })),
                    TripAccepted accepted => ("trip.accepted.v1", JsonSerializer.Serialize(new { accepted.TripId, accepted.DriverId, accepted.Version })),
                    _ => throw new InvalidOperationException("No integration mapping for domain event.")
                };
                db.Outbox.Add(new OutboxRow { Id = message.EventId, TripId = trip.Id, Type = type, Payload = payload, CreatedAt = clock.GetUtcNow() });
            }
        }
        using var activity = TripDiagnostics.Source.StartActivity("db.commit"); activity?.SetTag("db.system.name", "postgresql");
        try { await db.SaveChangesAsync(ct); }
        catch (DbUpdateConcurrencyException ex) { throw new ConcurrencyConflictException(ex); }
        catch (DbUpdateException ex) when (ex.InnerException is PostgresException { SqlState: "23505", ConstraintName: "pk_requests" })
        { throw new DuplicateRequestException(ex); }
        foreach (var pair in repository.Tracked.Values) pair.Aggregate.ClearPendingEvents();
    }
}
```

Bài trước: [bài 32](/articles/dotnet10-cqrs-mediator-pipeline). Bài tiếp: [bài 34](/articles/dotnet10-use-case-dat-chuyen-end-to-end).

## Tài liệu gốc

- [Infrastructure persistence layer](https://learn.microsoft.com/en-us/dotnet/architecture/microservices/microservice-ddd-cqrs-patterns/infrastructure-persistence-layer-design).
- [EF transactions](https://learn.microsoft.com/en-us/ef/core/saving/transactions).
