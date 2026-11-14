---
slug: dotnet10-use-case-dat-chuyen-end-to-end
title: "Ghép use case đặt chuyến: validation, transaction, domain event và kiểm thử"
summary: "Bài 34: lần theo POST /trips từ subject đã xác thực đến aggregate và commit PostgreSQL; kiểm tra replay, conflict và hai tài xế cùng nhận bằng HTTP thật trong test host."
category: csharp-advanced
level: advanced
tags: [csharp, dotnet, dotnet-10, toi-noi, architecture]
language: vi
---

Đây là **bài 34** của loạt **C# và .NET 10 qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Bài 31–38 cùng giải thích code cuối của service tại `examples/toi-noi-dotnet/31-38-trip-service`, target `net10.0`.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>CustomerId trong body có thể thay subject của token nếu client gửi đúng định dạng Guid không?</p>
<details><summary>Xem đáp án</summary><p>Không. Danh tính nghiệp vụ phải lấy từ phần ánh xạ danh tính đã xác thực; một Guid đúng cú pháp không chứng minh quyền sở hữu.</p></details>
</div>

## Một đường xử lý có thể lần theo

Khách gửi distanceKm và Idempotency-Key. API lấy sub đã xác thực, ánh xạ thành Guid theo hợp đồng của lab rồi tạo CreateTrip. Pipeline kiểm tra dữ liệu; handler tìm receipt cũ hoặc tạo DistanceKm/Money/Trip mới, thêm receipt và commit qua UoW.

Fare dùng cùng quy tắc minh hoạ từ đầu loạt: 12.000 gồm 2 km, vượt 4.500/km, làm tròn cuối tới 1 VND AwayFromZero. CreatedAt lấy từ TimeProvider, không từ trường tuỳ ý của client. Test thay đồng hồ để assert thời điểm cố định.

Dữ liệu từ API được bind xong vẫn chưa phải là một command hợp lệ. Validation hình thức, policy quyền, invariant domain và constraint database đều có phần việc riêng; không xoá lớp sau chỉ vì lớp trước đã kiểm.

## Replay và xung đột nội dung

Receipt có khoá là CustomerId/key và chứa input chuẩn hoá và kết quả TripId/Fare. Replay cùng ý định trả cùng kết quả, lab dùng 201 và cùng Location ở cả lần đầu lẫn lần lặp. Cùng key mà đổi distance thì trả 409.

Nếu hai request đều chưa thấy receipt, constraint là nơi phân xử cuối cùng. Ứng viên thua cuộc rollback cả Trip và Outbox của nó, rồi handler đọc kết quả đã commit. AsNoTracking ở đường đọc receipt giúp identity map không trả lại object ứng viên còn trong tracker sau khi lỗi.

Khi mở rộng body có pickup, destination hay loại xe, fingerprint và phép so khớp phải mở rộng theo tất cả trường ảnh hưởng ý định. Đừng chỉ dùng distance như lab cho hợp đồng nhiều trường, rồi nhận nhầm replay.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 1. Chủ thể không được lấy từ body; handler điều phối một ý định có thể retry theo key."><defs><marker id="cs34-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">HTTP + JWT</text><text x="34" y="123" class="t-small">Subject đáng tin cậy</text><path d="M280 108 L305 108" class="f-none s-phys" stroke-width="2" marker-end="url(#cs34-0-a)"/><rect x="305" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="319" y="95" class="t-b">CreateTrip</text><text x="319" y="123" class="t-small">Validate + domain</text><path d="M565 108 L590 108" class="f-none s-phys" stroke-width="2" marker-end="url(#cs34-0-a)"/><rect x="590" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="604" y="95" class="t-b">UoW / PostgreSQL</text><text x="604" y="123" class="t-small">Trip + receipt + Outbox</text></svg>
</div>
<figcaption>Hình 1. Chủ thể không được lấy từ body; handler điều phối một ý định có thể retry theo key.</figcaption>
</figure>

## Read ownership và accept concurrency

GetTrip lọc cả TripId và CustomerId ở query. Khác chủ hoặc không tồn tại đều trả 404 trong service tổng hợp này; đây là lựa chọn khác với demo đơn giản ở bài 19, vốn trả 403 cho owner sai. Hợp đồng mới được nêu rõ và có test riêng để tránh tiết lộ sự tồn tại tài nguyên.

DriverId của Accept lấy từ sub, expectedVersion từ request. Handler kiểm tra version đã quan sát được, gọi hành vi, sau đó database kiểm original version lúc commit. Hai request chồng lấp cho ra một kết quả 200 và một kết quả 409; test còn kiểm tra rằng chỉ có một trip.accepted.v1 trong Outbox.

Policy dành cho tài xế ở lab chưa kiểm tra hồ sơ, khu vực hay khả năng nhận thêm chuyến. Đừng biến demo này thành hệ thống dispatch chạy production chỉ vì đã có JWT role và concurrency token.

## Bằng chứng chạy qua ranh giới thật

Test dùng WebApplicationFactory, JWT handler thật với khoá fixture và PostgreSQL thật trong schema GUID. Không giả lập toàn bộ HTTP response hoặc thay EF bằng một dictionary rồi gọi đó là kiểm tra transaction.

Các test case gồm thiếu giá trị, ngoài miền, sai độ chính xác thập phân, key lỗi, duplicate/replay, key đổi input, ownership, role, nhận trùng, rehydrate và huỷ trước command. Khi validation thất bại, database không được có Trip/Outbox mới. Health, architecture và telemetry được kiểm tra trong cùng suite nhưng có tiêu chí riêng.

Không có user/token/password production trong fixture. Việc test trust bằng khoá trong memory không chứng minh được identity provider thật, metadata rotation hay TLS của môi trường triển khai.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Hình 2. Test kiểm cả response và Outbox để không bỏ sót một hiệu ứng phát lặp."><defs><marker id="cs34-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="25" width="250" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="55" class="t-b">Driver A</text><text x="34" y="83" class="t-small">ExpectedVersion 0</text><rect x="20" y="165" width="250" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="195" class="t-b">Driver B</text><text x="34" y="223" class="t-small">ExpectedVersion 0</text><rect x="520" y="95" width="330" height="85" rx="10" class="f-pk-soft s-pk"/><text x="534" y="125" class="t-b">Commit phân xử</text><text x="534" y="153" class="t-small">Một 200, một 409</text><path d="M270 67 L520 115" class="f-none s-phys" stroke-width="2" marker-end="url(#cs34-1-a)"/><path d="M270 207 L520 155" class="f-none s-phys" stroke-width="2" marker-end="url(#cs34-1-a)"/></svg>
</div>
<figcaption>Hình 2. Test kiểm cả response và Outbox để không bỏ sót một hiệu ứng phát lặp.</figcaption>
</figure>

## Bài tập

**Bài tập:** nếu trả 200 với fare=0 khi SaveChanges lỗi, invariant nào về phản hồi đã bị phá? Nếu ghi Trip và receipt bằng hai commit rồi chèn một network call vào giữa, rủi ro gì xuất hiện?

<details><summary>Xem đáp án</summary><p>Client có thể coi một lỗi hệ thống là báo giá thành công. Hai commit mở khoảng trống khiến ý định request và chuyến không còn nguyên tử; dùng một ranh giới local đúng và để cơ chế giao tiếp nằm ngoài transaction.</p></details>

## Code và kiểm chứng

Thiết lập `TOI_NOI_PG` cho database local riêng `toi_noi_dotnet_lab` theo README bài 21, rồi từ gốc repository:

```bash
dotnet test examples/toi-noi-dotnet/31-38-trip-service/Tests
```

Fixture dùng schema GUID riêng, không xoá database; JWT dùng khoá ngẫu nhiên trong test và vẫn đi qua handler xác thực thật. Test không tự bỏ qua khi thiếu PostgreSQL. Docker/CI/cloud spec xem README service và cloud/README.md để biết phạm vi, điều kiện tiên quyết và phần chưa triển khai thật.

### Handlers.cs — 31-38-trip-service/Application/Handlers.cs

```csharp
using ToiNoi.Trips.Domain;
namespace ToiNoi.Trips.Application;
public sealed record CreateTrip(Guid CustomerId, string Key, decimal? DistanceKm) : IRequest<CreateTripResult>;
public sealed record CreateTripResult(Guid TripId, decimal Fare, string Currency);
public sealed class CreateTripValidator : IRequestValidator<CreateTrip>
{
    public void Validate(CreateTrip request)
    {
        if (request.CustomerId == Guid.Empty) throw new ValidationFailureException("customer", "Identity required.");
        if (string.IsNullOrWhiteSpace(request.Key) || request.Key.Length > 128 || request.Key.Any(c => !char.IsAsciiLetterOrDigit(c) && c is not '-' and not '_'))
            throw new ValidationFailureException("Idempotency-Key", "Use 1..128 ASCII letters, digits, hyphen or underscore.");
        if (request.DistanceKm is not decimal value || value < 0m || value > 100m || value != decimal.Round(value, 6))
            throw new ValidationFailureException("distanceKm", "Expected 0..100 km with up to 6 fractional digits.");
    }
}
public sealed class CreateTripHandler(ITripRepository trips, IBookingRequests requests, IUnitOfWork uow, TimeProvider clock)
    : IRequestHandler<CreateTrip, CreateTripResult>
{
    public async Task<CreateTripResult> HandleAsync(CreateTrip request, CancellationToken ct)
    {
        var previous = await requests.FindAsync(request.CustomerId, request.Key, ct);
        if (previous is not null) return Resolve(previous, request.DistanceKm!.Value);
        var distance = new DistanceKm(request.DistanceKm!.Value);
        var trip = Trip.Request(Guid.NewGuid(), request.CustomerId, distance, StandardFare.Calculate(distance), clock.GetUtcNow());
        trips.Add(trip); requests.Add(new BookingReceipt(request.CustomerId, request.Key, distance.Value, trip.Id, trip.Fare.Amount));
        try { await uow.CommitAsync(ct); }
        catch (DuplicateRequestException)
        {
            var committed = await requests.FindAsync(request.CustomerId, request.Key, ct);
            if (committed is null) throw;
            return Resolve(committed, distance.Value);
        }
        return new(trip.Id, trip.Fare.Amount, trip.Fare.Currency);
    }
    private static CreateTripResult Resolve(BookingReceipt previous, decimal distance)
        => previous.DistanceKm == distance ? new(previous.TripId, previous.Fare, "VND")
            : throw new ConflictException("Idempotency key reused with different input.");
}
public sealed record GetTrip(Guid TripId, Guid CustomerId) : IRequest<TripView>;
public sealed class GetTripHandler(ITripQueries queries) : IRequestHandler<GetTrip, TripView>
{
    public async Task<TripView> HandleAsync(GetTrip request, CancellationToken ct)
        => await queries.FindOwnedAsync(request.TripId, request.CustomerId, ct) ?? throw new NotFoundException();
}
public sealed record AcceptTrip(Guid TripId, Guid DriverId, int? ExpectedVersion) : IRequest<AcceptTripResult>;
public sealed record AcceptTripResult(Guid TripId, string Status, int Version);
public sealed class AcceptTripValidator : IRequestValidator<AcceptTrip>
{
    public void Validate(AcceptTrip request)
    {
        if (request.DriverId == Guid.Empty) throw new ValidationFailureException("driver", "Identity required.");
        if (request.ExpectedVersion is null or < 0) throw new ValidationFailureException("expectedVersion", "Nonnegative version required.");
    }
}
public sealed class AcceptTripHandler(ITripRepository trips, IUnitOfWork uow) : IRequestHandler<AcceptTrip, AcceptTripResult>
{
    public async Task<AcceptTripResult> HandleAsync(AcceptTrip request, CancellationToken ct)
    {
        var trip = await trips.GetAsync(request.TripId, ct) ?? throw new NotFoundException();
        if (trip.Version != request.ExpectedVersion) throw new ConflictException("Stale expected version.");
        trip.Accept(request.DriverId); await uow.CommitAsync(ct);
        return new(trip.Id, trip.Status.ToString(), trip.Version);
    }
}
```

### ApiErrors.cs — 31-38-trip-service/Api/ApiErrors.cs

```csharp
using Microsoft.AspNetCore.Diagnostics;
using Microsoft.AspNetCore.Mvc;
using ToiNoi.Trips.Application;
using ToiNoi.Trips.Domain;
namespace ToiNoi.Trips.Api;
public sealed class ApiErrors : IExceptionHandler
{
    public async ValueTask<bool> TryHandleAsync(HttpContext http, Exception error, CancellationToken ct)
    {
        if (error is OperationCanceledException && http.RequestAborted.IsCancellationRequested) return false;
        int status = error switch
        {
            ValidationFailureException => 400, NotFoundException => 404, UnauthorizedAccessException => 403,
            ConflictException or ConcurrencyConflictException or DomainRuleException => 409,
            _ => 500
        };
        ProblemDetails problem = error is ValidationFailureException validation
            ? new HttpValidationProblemDetails(new Dictionary<string, string[]> { [validation.Field] = [validation.Message] })
            : new ProblemDetails();
        problem.Status = status;
        problem.Title = status switch { 400 => "Invalid request", 403 => "Forbidden", 404 => "Trip not found", 409 => "Request conflicts with current state", _ => "Unexpected server error" };
        problem.Extensions["traceId"] = System.Diagnostics.Activity.Current?.TraceId.ToString() ?? http.TraceIdentifier;
        http.Response.StatusCode = status;
        await http.Response.WriteAsJsonAsync<object>(problem, options: null, contentType: "application/problem+json", cancellationToken: ct);
        return true;
    }
}
```

Bài trước: [bài 33](/articles/dotnet10-repository-unit-of-work). Bài tiếp: [bài 35](/articles/dotnet10-container-ci-cd).

## Tài liệu gốc

- [DDD-oriented microservice](https://learn.microsoft.com/en-us/dotnet/architecture/microservices/microservice-ddd-cqrs-patterns/ddd-oriented-microservice).
- [Integration tests](https://learn.microsoft.com/en-us/aspnet/core/test/integration-tests?view=aspnetcore-10.0).
