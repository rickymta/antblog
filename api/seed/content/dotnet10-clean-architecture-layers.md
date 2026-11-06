---
slug: dotnet10-clean-architecture-layers
title: "Clean Architecture: chia Domain, Application, Infrastructure và API của Tới Nơi"
summary: "Bài 31: ghép service mẫu bốn tầng, giữ phụ thuộc hướng vào lõi và phân biệt chiều gọi runtime với chiều tham chiếu mã nguồn."
category: csharp
level: advanced
tags: [csharp, dotnet, dotnet-10, toi-noi, architecture]
language: vi
---

Đây là **bài 31** của loạt **C# và .NET 10 qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Bài 31–38 cùng giải thích code cuối của service tại `examples/toi-noi-dotnet/31-38-trip-service`, target `net10.0`.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>API gọi repository ở runtime có nghĩa Domain phải tham chiếu project Infrastructure không?</p>
<details><summary>Xem đáp án</summary><p>Không. Lõi định nghĩa abstraction; Infrastructure hiện thực nó và composition root ghép các implementation.</p></details>
</div>

## Từ các thí nghiệm sang một service

Ba mươi bài đầu đã kiểm từng cơ chế. Tám bài cuối cùng dùng chung service Trips để thấy chúng phối hợp: khách tạo chuyến, đọc chuyến của mình; tài xế nhận một chuyến còn trống; dữ liệu và Outbox commit trên PostgreSQL.

Không thêm tám bản chép công thức khác nhau. Thư mục 31-38-trip-service là code cuối của cùng một lab, mỗi bài giải thích một lát cắt. Nó chưa là toàn bộ sản phẩm Tới Nơi: chưa có điều phối thực, thu tiền, fraud/OCR hoặc broker production.

Clean Architecture ở đây phục vụ ranh giới thay đổi và kiểm thử. Bốn project trông gọn không tự chứng minh clean; điều cần xem là ai sở hữu rule, ai tham chiếu ai và chi tiết hạ tầng có rò vào mô hình hay không.

## Bốn tầng có việc gì?

Domain giữ Trip, Money, DistanceKm, invariant và domain event; không biết EF, HTTP hay DI container. Application mô tả command/query, handler, validator và port cần dùng. Infrastructure triển khai repository, UoW, query projection và dispatcher. API bind HTTP, xác thực/phân quyền và là composition root.

Application tham chiếu Domain. Infrastructure tham chiếu Application và dùng các kiểu Domain. API tham chiếu các lớp cần lắp. Không có ProjectReference từ Domain/Application sang Infrastructure; architecture test còn kiểm referenced assemblies để phát hiện EF/ASP.NET lọt vào lõi.

Chiều gọi runtime có thể đi từ handler tới repository implementation. Chiều phụ thuộc source vẫn hướng vào interface do Application sở hữu. Đừng vẽ hai chiều đó thành một mũi tên duy nhất rồi kết luận dependency inversion đã bị vi phạm.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 1. Chiều tham chiếu hướng vào lõi; chiều gọi runtime qua interface có thể đi tới adapter."><defs><marker id="cs31-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Domain</text><text x="34" y="123" class="t-small">Rule + identity</text><rect x="305" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="319" y="95" class="t-b">Application</text><text x="319" y="123" class="t-small">Use case + port</text><rect x="590" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="604" y="95" class="t-b">Infrastructure / API</text><text x="604" y="123" class="t-small">Adapter + composition</text><path d="M305 175 L280 175" class="f-none s-phys" stroke-width="2" marker-end="url(#cs31-0-a)"/><path d="M590 175 L565 175" class="f-none s-phys" stroke-width="2" marker-end="url(#cs31-0-a)"/><text x="290" y="205" class="t-small">Chiều tham chiếu vào lõi</text></svg>
</div>
<figcaption>Hình 1. Chiều tham chiếu hướng vào lõi; chiều gọi runtime qua interface có thể đi tới adapter.</figcaption>
</figure>

## Domain khác persistence row

TripRow có setter để EF materialize; Trip aggregate chỉ đổi qua hành vi. Adapter rehydrate Trip bằng Restore, kiểm trạng thái lưu hợp lệ và không phát một sự kiện tạo chuyến mới mỗi lần đọc.

Không dùng reflection để tự sửa private state của aggregate. Một đường Restore tường minh giúp nhìn rõ invariant và khác biệt giữa tạo mới với khôi phục. Restore không đồng nghĩa tin mọi dữ liệu từ API: dữ liệu ngoài phải qua binding/validation, còn dữ liệu lưu hỏng cần được nhìn thấy như lỗi hệ thống.

Không nhất thiết mọi ứng dụng đều cần hai mô hình riêng. Trong lab, chi phí mapping được chấp nhận để giữ domain thuần và làm rõ bài học. Khi đánh giá một cách map khác, hãy nói cả lợi ích và cái giá thay vì chỉ đếm số class.

## Composition root là nơi biết implementation

Program đăng ký repository concrete scoped một lần, rồi map ITripRepository về cùng instance; EfUnitOfWork dùng chính repository đó để gom các aggregate đang tham gia use case. Nếu đăng ký hai instance repository khác nhau cho hai interface, UoW có thể không thấy thay đổi domain cần flush.

DbContext là scoped theo request. TimeProvider sống dùng chung và được thay bằng FixedClock trong test. Domain không gọi IServiceProvider; việc tìm handler nằm ở dispatcher hạ tầng có phạm vi rõ, không biến service locator thành thói quen trong mọi lớp.

Cấu hình storage/auth được bind và validate khi host khởi động. API không tự migrate; readiness kiểm kết nối và pending migration. Live không query DB để lỗi database không dẫn đến vòng restart của chính process.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 2. Ba kiểu dữ liệu có vai trò riêng, không dùng một entity tracking cho mọi ranh giới."><defs><marker id="cs31-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Trip aggregate</text><text x="34" y="123" class="t-small">Hành vi có invariant</text><rect x="305" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="319" y="95" class="t-b">TripRow</text><text x="319" y="123" class="t-small">Mapping persistence</text><rect x="590" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="604" y="95" class="t-b">TripView</text><text x="604" y="123" class="t-small">DTO phía đọc</text></svg>
</div>
<figcaption>Hình 2. Ba kiểu dữ liệu có vai trò riêng, không dùng một entity tracking cho mọi ranh giới.</figcaption>
</figure>

## Bài tập và bằng chứng

**Bài tập:** chuyển một câu UseNpgsql vào Domain có làm test giá tiền sai ngay không? Nếu không, tại sao architecture test vẫn đáng giữ?

<details><summary>Xem đáp án</summary><p>Test số tiền có thể vẫn qua, nhưng dependency boundary đã đổi. Architecture test bảo vệ một thuộc tính khác: lõi không phụ thuộc ORM/web/hạ tầng, giúp test và thay đổi chi tiết độc lập.</p></details>

Phạm vi được kiểm ở đây là bốn tầng của service Trips. Không suy ra một repository hay một database có thể dùng chung cho mọi service Python/Rust/Go trong Tới Nơi.

## Code và kiểm chứng

Thiết lập `TOI_NOI_PG` cho database local riêng `toi_noi_dotnet_lab` theo README bài 21, rồi từ gốc repository:

```bash
dotnet test examples/toi-noi-dotnet/31-38-trip-service/Tests
```

Fixture dùng schema GUID riêng, không xóa database; JWT dùng khóa ngẫu nhiên trong test và vẫn đi qua handler xác thực thật. Test không tự skip khi thiếu PostgreSQL. Docker/CI/cloud spec xem README service và cloud/README.md để biết phạm vi, prerequisites và phần chưa được triển khai thật.

### Contracts.cs — 31-38-trip-service/Application/Contracts.cs

```csharp
using ToiNoi.Trips.Domain;
namespace ToiNoi.Trips.Application;
public interface IRequest<TResponse> { }
public delegate Task<TResponse> RequestNext<TResponse>();
public interface IRequestHandler<TRequest, TResponse> where TRequest : IRequest<TResponse>
{ Task<TResponse> HandleAsync(TRequest request, CancellationToken ct); }
public interface IPipelineBehavior<TRequest, TResponse> where TRequest : IRequest<TResponse>
{ Task<TResponse> HandleAsync(TRequest request, RequestNext<TResponse> next, CancellationToken ct); }
public interface IRequestValidator<TRequest> { void Validate(TRequest request); }
public interface IRequestDispatcher
{ Task<TResponse> SendAsync<TRequest, TResponse>(TRequest request, CancellationToken ct) where TRequest : IRequest<TResponse>; }
public interface ITripRepository
{
    Task<Trip?> GetAsync(Guid id, CancellationToken ct);
    void Add(Trip trip);
}
public interface IUnitOfWork { Task CommitAsync(CancellationToken ct); }
public sealed record BookingReceipt(Guid CustomerId, string Key, decimal DistanceKm, Guid TripId, decimal Fare);
public interface IBookingRequests
{
    Task<BookingReceipt?> FindAsync(Guid customerId, string key, CancellationToken ct);
    void Add(BookingReceipt receipt);
}
public sealed record TripView(Guid Id, Guid CustomerId, decimal Fare, string Currency, string Status, Guid? DriverId, int Version);
public interface ITripQueries { Task<TripView?> FindOwnedAsync(Guid tripId, Guid customerId, CancellationToken ct); }
public sealed class ValidationFailureException(string field, string error) : Exception(error)
{ public string Field { get; } = field; }
public sealed class NotFoundException() : Exception("Trip not found.");
public sealed class ConflictException(string message) : Exception(message);
public sealed class DuplicateRequestException(Exception inner) : Exception("Request key already committed.", inner);
public sealed class ConcurrencyConflictException(Exception inner) : Exception("Trip changed concurrently.", inner);
```

### ServiceTests.cs — 31-38-trip-service/Tests/ServiceTests.cs

```csharp
using System.Net;
using System.Net.Http.Json;
using System.Text.Json;
using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.DependencyInjection;
using ToiNoi.Trips.Application;
using ToiNoi.Trips.Domain;
using ToiNoi.Trips.Infrastructure;
using Xunit;

[assembly: CollectionBehavior(DisableTestParallelization = true)]
namespace ToiNoi.Trips.Tests;

public sealed class ServiceTests : IAsyncLifetime
{
    private ServiceLab lab = null!;
    private ApiFactory factory = null!;
    private HttpClient customer = null!;
    private readonly Guid owner = Guid.NewGuid();
    public async Task InitializeAsync()
    {
        lab = await ServiceLab.CreateAsync(); factory = new ApiFactory(lab.Connection);
        customer = factory.Client(owner, "customer");
    }
    public async Task DisposeAsync() { customer?.Dispose(); factory?.Dispose(); if (lab is not null) await lab.DisposeAsync(); }
    private static async Task<HttpResponseMessage> CreateAsync(HttpClient client, string key, decimal? distance)
    {
        using var request = new HttpRequestMessage(HttpMethod.Post, "/trips") { Content = JsonContent.Create(new { distanceKm = distance }) };
        request.Headers.Add("Idempotency-Key", key); return await client.SendAsync(request);
    }
    [Fact]
    public async Task Create_commits_trip_request_and_outbox_with_one_result()
    {
        using var response = await CreateAsync(customer, "create-1", 5m);
        Assert.Equal(HttpStatusCode.Created, response.StatusCode);
        var result = (await response.Content.ReadFromJsonAsync<CreateTripResult>())!;
        Assert.Equal(25500m, result.Fare); Assert.Equal("VND", result.Currency);
        Assert.Equal($"/trips/{result.TripId}", response.Headers.Location?.ToString());
        await using var db = lab.Open();
        var row = await db.Trips.SingleAsync(); Assert.Equal(owner, row.CustomerId); Assert.Equal(DateTimeOffset.UnixEpoch, row.CreatedAt);
        Assert.Equal(1, await db.Requests.CountAsync()); var message = await db.Outbox.SingleAsync();
        Assert.Equal("trip.requested.v1", message.Type); Assert.Equal(row.Id, message.TripId);
    }
    [Theory]
    [InlineData(null)]
    [InlineData("-0.001")]
    [InlineData("100.001")]
    [InlineData("2.0000001")]
    public async Task Invalid_distance_is_400_and_creates_no_rows(string? input)
    {
        decimal? distance = input is null ? null : decimal.Parse(input, System.Globalization.CultureInfo.InvariantCulture);
        using var response = await CreateAsync(customer, "invalid-1", distance);
        Assert.Equal(HttpStatusCode.BadRequest, response.StatusCode);
        Assert.Equal("application/problem+json", response.Content.Headers.ContentType?.MediaType);
        var problem = await response.Content.ReadFromJsonAsync<JsonElement>(); Assert.True(problem.GetProperty("errors").TryGetProperty("distanceKm", out _));
        await using var db = lab.Open(); Assert.Equal(0, await db.Trips.CountAsync()); Assert.Equal(0, await db.Outbox.CountAsync());
    }
    [Theory]
    [InlineData("")]
    [InlineData("with space")]
    public async Task Invalid_key_is_400(string key)
    {
        using var response = await CreateAsync(customer, key, 5m); Assert.Equal(HttpStatusCode.BadRequest, response.StatusCode);
    }
    [Fact]
    public async Task Concurrent_duplicate_and_replay_return_same_committed_trip()
    {
        var responses = await Task.WhenAll(CreateAsync(customer, "repeat-1", 5m), CreateAsync(customer, "repeat-1", 5m));
        try
        {
            Assert.All(responses, r => Assert.Equal(HttpStatusCode.Created, r.StatusCode));
            var first = (await responses[0].Content.ReadFromJsonAsync<CreateTripResult>())!;
            var second = (await responses[1].Content.ReadFromJsonAsync<CreateTripResult>())!; Assert.Equal(first, second);
            using var replay = await CreateAsync(customer, "repeat-1", 5m);
            Assert.Equal(first, await replay.Content.ReadFromJsonAsync<CreateTripResult>());
            await using var db = lab.Open(); Assert.Equal(1, await db.Trips.CountAsync()); Assert.Equal(1, await db.Requests.CountAsync()); Assert.Equal(1, await db.Outbox.CountAsync());
        }
        finally { foreach (var response in responses) response.Dispose(); }
    }
    [Fact]
    public async Task Same_key_different_input_is_conflict()
    {
        using var first = await CreateAsync(customer, "conflict-1", 5m);
        using var second = await CreateAsync(customer, "conflict-1", 6m); Assert.Equal(HttpStatusCode.Conflict, second.StatusCode);
        await using var db = lab.Open(); Assert.Equal(1, await db.Trips.CountAsync());
    }
    [Fact]
    public async Task Ownership_is_from_verified_subject_and_wrong_owner_sees_404()
    {
        using var created = await CreateAsync(customer, "owned-1", 5m); var trip = (await created.Content.ReadFromJsonAsync<CreateTripResult>())!;
        using var own = await customer.GetAsync($"/trips/{trip.TripId}"); Assert.Equal(HttpStatusCode.OK, own.StatusCode);
        using var other = factory.Client(Guid.NewGuid(), "customer"); using var forbidden = await other.GetAsync($"/trips/{trip.TripId}");
        Assert.Equal(HttpStatusCode.NotFound, forbidden.StatusCode);
    }
    [Fact]
    public async Task Unauthenticated_and_wrong_role_are_rejected()
    {
        using var anonymous = factory.CreateClient(); using var a = await anonymous.GetAsync($"/trips/{Guid.NewGuid()}"); Assert.Equal(HttpStatusCode.Unauthorized, a.StatusCode);
        using var driver = factory.Client(Guid.NewGuid(), "driver"); using var b = await CreateAsync(driver, "role-1", 5m); Assert.Equal(HttpStatusCode.Forbidden, b.StatusCode);
    }
    [Fact]
    public async Task Two_drivers_have_one_winner_and_one_accept_event()
    {
        using var created = await CreateAsync(customer, "accept-1", 5m); var trip = (await created.Content.ReadFromJsonAsync<CreateTripResult>())!;
        using var a = factory.Client(Guid.NewGuid(), "driver"); using var b = factory.Client(Guid.NewGuid(), "driver");
        var responses = await Task.WhenAll(a.PostAsJsonAsync($"/trips/{trip.TripId}/accept", new { expectedVersion = 0 }), b.PostAsJsonAsync($"/trips/{trip.TripId}/accept", new { expectedVersion = 0 }));
        try
        {
            Assert.Single(responses, r => r.StatusCode == HttpStatusCode.OK); Assert.Single(responses, r => r.StatusCode == HttpStatusCode.Conflict);
            await using var db = lab.Open(); var row = await db.Trips.SingleAsync(); Assert.Equal("Accepted", row.Status); Assert.Equal(1, row.Version); Assert.NotNull(row.DriverId);
            Assert.Equal(2, await db.Outbox.CountAsync()); Assert.Equal(1, await db.Outbox.CountAsync(x => x.Type == "trip.accepted.v1"));
        }
        finally { foreach (var response in responses) response.Dispose(); }
    }
    [Fact]
    public async Task Rehydration_has_no_new_creation_events()
    {
        using var created = await CreateAsync(customer, "restore-1", 5m); var trip = (await created.Content.ReadFromJsonAsync<CreateTripResult>())!;
        await using var scope = factory.Services.CreateAsyncScope(); var repository = scope.ServiceProvider.GetRequiredService<ITripRepository>();
        var a = (await repository.GetAsync(trip.TripId, CancellationToken.None))!; var b = await repository.GetAsync(trip.TripId, CancellationToken.None);
        Assert.Same(a, b); Assert.Empty(a.PendingEvents);
    }
    [Fact]
    public async Task Precancelled_command_never_writes()
    {
        await using var scope = factory.Services.CreateAsyncScope(); var dispatcher = scope.ServiceProvider.GetRequiredService<IRequestDispatcher>();
        using var cts = new CancellationTokenSource(); cts.Cancel();
        await Assert.ThrowsAnyAsync<OperationCanceledException>(() => dispatcher.SendAsync<CreateTrip, CreateTripResult>(new(owner, "cancel-1", 5m), cts.Token));
        await using var db = lab.Open(); Assert.Equal(0, await db.Trips.CountAsync());
    }
    [Fact]
    public async Task Readiness_is_healthy_after_migration()
    {
        using var response = await customer.GetAsync("/health/ready"); Assert.Equal(HttpStatusCode.OK, response.StatusCode);
    }
}

public sealed class HealthTests
{
    [Fact]
    public async Task Live_can_be_healthy_while_schema_is_not_ready_and_startup_does_not_migrate()
    {
        await using var lab = await ServiceLab.CreateAsync(migrate: false); using var factory = new ApiFactory(lab.Connection); using var client = factory.CreateClient();
        using var live = await client.GetAsync("/health/live"); using var ready = await client.GetAsync("/health/ready");
        Assert.Equal(HttpStatusCode.OK, live.StatusCode); Assert.Equal(HttpStatusCode.ServiceUnavailable, ready.StatusCode);
    }
}

public sealed class ArchitectureTests
{
    [Fact]
    public void Domain_and_application_do_not_reference_orm_web_or_infrastructure()
    {
        foreach (var assembly in new[] { typeof(Trip).Assembly, typeof(CreateTrip).Assembly })
            Assert.DoesNotContain(assembly.GetReferencedAssemblies(), reference => reference.Name!.Contains("EntityFrameworkCore") || reference.Name.Contains("AspNetCore") || reference.Name == "Infrastructure");
    }
    [Fact]
    public void Restore_rejects_invalid_state_and_does_not_raise_events()
    {
        Assert.Throws<ArgumentException>(() => Trip.Restore(Guid.NewGuid(), Guid.NewGuid(), new DistanceKm(5m), new Money(25500m), DateTimeOffset.UnixEpoch, TripStatus.Accepted, null, 0));
        var restored = Trip.Restore(Guid.NewGuid(), Guid.NewGuid(), new DistanceKm(5m), new Money(25500m), DateTimeOffset.UnixEpoch, TripStatus.Requested, null, 0);
        Assert.Empty(restored.PendingEvents);
    }
}
```

Bài trước: [bài 30](/articles/dotnet10-ddd-bounded-context-aggregate). Bài tiếp: [bài 32](/articles/dotnet10-cqrs-mediator-pipeline).

## Tài liệu gốc

- [Common application architectures](https://learn.microsoft.com/en-us/dotnet/standard/modern-web-apps-azure-architecture/common-web-application-architectures).
