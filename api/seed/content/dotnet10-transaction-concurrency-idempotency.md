---
slug: dotnet10-transaction-concurrency-idempotency
title: "Transaction, optimistic concurrency và idempotency: đặt chuyến an toàn với PostgreSQL"
summary: "Bài 23: kiểm rollback thật, phát hiện version cũ bằng EF Core và dùng khoá idempotency kèm unique constraint để một ý định đặt chuyến có một kết quả bền."
category: csharp-advanced
level: advanced
tags: [csharp, dotnet, dotnet-10, toi-noi, postgresql, ef-core]
language: vi
---

Đây là **bài 23** trong loạt **C# và .NET 10 qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Các ví dụ target `net10.0`, dùng dữ liệu giả và cùng code đã chạy kiểm chứng trong repository.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Bắt concurrency exception rồi SaveChanges lại ngay có tự giải được hai tài xế cùng nhận chuyến?</p>
<details><summary>Xem đáp án</summary><p>Không. Phải tải lại trạng thái và quyết định nghiệp vụ; retry mù có thể ghi đè người đã thắng.</p></details>
</div>

## Ba vấn đề khác nhau thường bị gộp

Transaction giữ tính nguyên tử của một nhóm thay đổi local. Optimistic concurrency phát hiện cập nhật dựa trên phiên bản cũ. Idempotency giữ một ý định request lặp không tạo nhiều hiệu ứng. Một cơ chế không tự thay được hai cơ chế còn lại.

Trong Tới Nơi, mạng có thể mất response sau khi chuyến đã commit; hai tài xế có thể đọc cùng trạng thái; và một use case có thể cần ghi chuyến kèm bản ghi xử lý request. Ta kiểm từng phần trên PostgreSQL thật rồi nối chúng trong một implementation nhỏ.

Lab chưa dựng giao dịch phân tán với dịch vụ tài chính Rust hoặc scoring Python. Không giữ một transaction SQL mở trong lúc gọi các service đó và không coi Unit of Work C# là transaction xuyên service.

## SaveChanges và transaction tường minh

Với provider quan hệ hỗ trợ transaction, một SaveChanges chứa nhiều thay đổi được áp dụng nguyên tử theo cơ chế EF/provider. BookTrip thêm TripRow cùng BookingRequestRow trong một lần SaveChanges: không được commit khoá xử lý nhưng mất chuyến, hoặc commit chuyến mà mất khoá.

Test rollback mở transaction tường minh, sửa fare, SaveChanges rồi Rollback. Context mới đọc lại phải không thấy fare=1. Dùng context mới quan trọng vì rollback database không có nghĩa mọi object trong tracker tự khôi phục trạng thái ban đầu cho bạn.

Transaction càng kéo dài càng giữ tài nguyên/phiên bản và có thể gây tranh chấp. Câu trả lời không phải bỏ transaction mà là chọn phạm vi nhỏ đúng invariant và tránh I/O ngoài database bên trong.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 1. Khoá xử lý request và dữ liệu chuyến cùng transaction, không có một commit riêng dễ tạo khoảng trống."><defs><marker id="cs23-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Một SaveChanges</text><text x="34" y="123" class="t-small">Trip + request key</text><path d="M280 115 L305 115" class="f-none s-phys" stroke-width="2" marker-end="url(#cs23-0-arrow)"/><rect x="305" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="319" y="95" class="t-b">Transaction local</text><text x="319" y="123" class="t-small">Cùng commit / rollback</text><path d="M565 115 L590 115" class="f-none s-phys" stroke-width="2" marker-end="url(#cs23-0-arrow)"/><rect x="590" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="604" y="95" class="t-b">Kết quả bền</text><text x="604" y="123" class="t-small">Một ý định request</text></svg>
</div>
<figcaption>Hình 1. Khoá xử lý request và dữ liệu chuyến cùng transaction, không có một commit riêng dễ tạo khoảng trống.</figcaption>
</figure>

## Version phải tham gia điều kiện UPDATE

TripRow.Version là Guid concurrency token do ứng dụng quản lý. Hai context tải cùng version. Context thứ nhất đổi driver, status và version rồi lưu. Context thứ hai vẫn mang original version cũ; UPDATE không khớp hàng mong đợi nên EF ném DbUpdateConcurrencyException.

Mọi đường ghi cần đổi version theo chính sách đã chọn; IsConcurrencyToken không tự tăng Guid. Demo chưa có interceptor tự quản lý token. Nếu một writer quên đổi version, cơ chế có thể không phát hiện thay đổi theo ý định thiết kế.

Sau xung đột, application phải đọc trạng thái mới và giải quyết: chuyến đã có driver thì từ chối ý định nhận thứ hai. Không tự động retry ghi đè bằng version mới chỉ để exception biến mất. Token này khác Idempotency-Key của request.

## Idempotency cần khoá bền và so khớp nội dung

BookTrip nhận customerId, key và distance. Khoá chính ghép customerId/key phân vùng theo chủ thể. Nếu đã có, cùng khoảng cách trả TripId cũ; khác input bị từ chối. Các field tham gia ý định phải được so đầy đủ khi mở rộng hợp đồng, không chỉ giữ mỗi khoảng cách như lab.

Hai request có thể cùng không thấy bản ghi ban đầu. Unique constraint là lớp quyết định cuối; candidate thua bị rollback trong SaveChanges. Code chỉ bắt SQLSTATE 23505 rồi đọc lại kết quả đã commit bằng context mới; nếu không tìm thấy khoá request đúng phạm vi thì ném lại, không nuốt mọi lỗi database.

Giới hạn độ dài key và độ chia nhỏ distance được kiểm để mapping numeric(12,6) không âm thầm đổi ý nghĩa fingerprint. Key lấy từ hợp đồng request, không từ một chuỗi nối có thể xung đột. Retention, quyền truy cập và response snapshot đầy đủ cần thiết kế tiếp cho service thật.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 275" role="img" aria-label="Hình 2. Optimistic concurrency phát hiện original version cũ; không tự quyết định nghiệp vụ sau conflict."><defs><marker id="cs23-1-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="25" width="250" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="55" class="t-b">Context A</text><text x="34" y="83" class="t-small">Original version V1</text><rect x="20" y="165" width="250" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="195" class="t-b">Context B</text><text x="34" y="223" class="t-small">Original version V1</text><rect x="550" y="95" width="300" height="85" rx="10" class="f-warn-soft s-warn"/><text x="564" y="125" class="t-b">A lưu → V2</text><text x="564" y="153" class="t-small">B stale → conflict</text><path d="M270 67 L550 115" class="f-none s-phys" stroke-width="2" marker-end="url(#cs23-1-arrow)"/><path d="M270 207 L550 155" class="f-none s-phys" stroke-width="2" marker-end="url(#cs23-1-arrow)"/></svg>
</div>
<figcaption>Hình 2. Optimistic concurrency phát hiện original version cũ; không tự quyết định nghiệp vụ sau conflict.</figcaption>
</figure>

## Kiểm chứng và bài tập

Test đối chiếu rollback, stale update và hai lời đặt cùng key cho cùng TripId; lần lặp sau cũng trả cùng kết quả, input khác bị từ chối. Hai lời gọi Task.WhenAll là yêu cầu chồng lấp, không khẳng định một lịch lock PostgreSQL cụ thể được ép tại từng bước.

**Bài tập:** nếu kiểm key bằng SELECT nhưng không có unique constraint, hai server có thể tạo hai chuyến không? Nếu cùng key nhưng đổi khách, phạm vi khoá nên thế nào?

<details><summary>Xem đáp án</summary><p>Có: khoảng giữa kiểm tra và ghi vẫn race. Phạm vi khoá phải theo hợp đồng chủ thể/use case; lab dùng customerId/key nên hai khách không chia sẻ một request record. Authentication phía API vẫn phải xác định customerId đáng tin cậy.</p></details>

## Chạy ví dụ và kiểm chứng

Cần PostgreSQL local và database riêng tên `toi_noi_dotnet_lab`. Thiết lập biến môi trường `TOI_NOI_PG` bằng connection string của lab trước khi chạy; không đưa mật khẩu vào source. README của persistence lab có hướng dẫn. Fixture từ chối database khác tên hoặc host ngoài localhost, không âm thầm skip khi thiếu kết nối; nó chỉ tạo và dọn schema GUID riêng.

Từ thư mục gốc repository:

```bash
dotnet test examples/toi-noi-dotnet/23-transactions-idempotency/Booking.Tests
```

### BookTrip.cs — 23-transactions-idempotency/Booking/BookTrip.cs

```csharp
using Microsoft.EntityFrameworkCore;
using Npgsql;
using ToiNoi.Persistence;
using ToiNoi.Fares;
namespace ToiNoi.Booking;
public sealed class BookTrip(Func<TripDb> open)
{
    public async Task<Guid> ExecuteAsync(Guid customerId, string key, decimal distanceKm)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(key);
        if (key.Length > 128 || distanceKm != decimal.Round(distanceKm, 6)) throw new ArgumentException("Unsupported lab request precision/key.");
        decimal fare = new StandardFarePolicy().Calculate(distanceKm);
        await using var db = open();
        var existing = await db.BookingRequests.FindAsync(customerId, key);
        if (existing is not null) return Resolve(existing, distanceKm);
        var trip = new TripRow { Id = Guid.NewGuid(), CustomerId = customerId, Fare = fare, CreatedAt = DateTimeOffset.UtcNow };
        db.Trips.Add(trip);
        db.BookingRequests.Add(new BookingRequestRow { CustomerId = customerId, Key = key, DistanceKm = distanceKm, TripId = trip.Id });
        try { await db.SaveChangesAsync(); return trip.Id; }
        catch (DbUpdateException ex) when (ex.InnerException is PostgresException { SqlState: "23505" })
        {
            // The failed SaveChanges transaction rolls back its candidate trip too.
            await using var retryRead = open();
            var committed = await retryRead.BookingRequests.AsNoTracking().SingleOrDefaultAsync(x => x.CustomerId == customerId && x.Key == key);
            if (committed is null) throw;
            return Resolve(committed, distanceKm);
        }
    }
    private static Guid Resolve(BookingRequestRow existing, decimal distanceKm) => existing.DistanceKm == distanceKm
        ? existing.TripId : throw new InvalidOperationException("Idempotency key reused with different input.");
}
```

### BookingTests.cs — 23-transactions-idempotency/Booking.Tests/BookingTests.cs

```csharp
using Microsoft.EntityFrameworkCore;
using ToiNoi.LabTests;
using ToiNoi.Booking;
using ToiNoi.Persistence;
using Xunit;
namespace ToiNoi.BookingTests;
public sealed class BookingTests
{
    [Fact]
    public async Task Explicit_rollback_undoes_saved_changes()
    {
        await using var lab = await PgLab.CreateAsync(); await lab.SeedAsync();
        await using (var db = lab.Open())
        {
            await using var transaction = await db.Database.BeginTransactionAsync();
            var row = await db.Trips.FirstAsync(); row.Fare = 1m; await db.SaveChangesAsync(); await transaction.RollbackAsync();
        }
        await using var read = lab.Open(); Assert.DoesNotContain(await read.Trips.ToListAsync(), x => x.Fare == 1m);
    }
    [Fact]
    public async Task Stale_version_cannot_overwrite_first_accept()
    {
        await using var lab = await PgLab.CreateAsync(); await lab.SeedAsync();
        await using var a = lab.Open(); await using var b = lab.Open();
        var first = await a.Trips.OrderBy(x => x.CreatedAt).FirstAsync(); var stale = await b.Trips.SingleAsync(x => x.Id == first.Id);
        first.Status = "accepted"; first.DriverId = "DR-1"; first.Version = Guid.NewGuid(); await a.SaveChangesAsync();
        stale.Status = "accepted"; stale.DriverId = "DR-2"; stale.Version = Guid.NewGuid();
        await Assert.ThrowsAsync<DbUpdateConcurrencyException>(() => b.SaveChangesAsync());
        await using var read = lab.Open(); Assert.Equal("DR-1", (await read.Trips.SingleAsync(x => x.Id == first.Id)).DriverId);
    }
    [Fact]
    public async Task Repeated_or_concurrent_request_has_one_durable_result()
    {
        await using var lab = await PgLab.CreateAsync(); var customer = await lab.SeedAsync(); var booking = new BookTrip(() => lab.Open());
        var results = await Task.WhenAll(booking.ExecuteAsync(customer, "request-1", 5m), booking.ExecuteAsync(customer, "request-1", 5m));
        Assert.Equal(results[0], results[1]); Assert.Equal(results[0], await booking.ExecuteAsync(customer, "request-1", 5m));
        await Assert.ThrowsAsync<InvalidOperationException>(() => booking.ExecuteAsync(customer, "request-1", 6m));
        await using var read = lab.Open(); Assert.Equal(4, await read.Trips.CountAsync()); Assert.Equal(1, await read.BookingRequests.CountAsync());
    }
}
```

Bài trước: [bài 22](/articles/dotnet10-efcore-tracking-projection-n-plus-one). Bài tiếp: [bài 24](/articles/dotnet10-backgroundservice-channel).

## Tài liệu gốc

- [Transactions](https://learn.microsoft.com/en-us/ef/core/saving/transactions).
- [Optimistic concurrency](https://learn.microsoft.com/en-us/ef/core/saving/concurrency).
- [PostgreSQL unique constraints](https://www.postgresql.org/docs/18/ddl-constraints.html).
