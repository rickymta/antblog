---
slug: dotnet10-cqrs-mediator-pipeline
title: "CQRS và Mediator: command/query handler và pipeline có thứ tự"
summary: "Bài 32: tách use case ghi/đọc, xây dispatcher typed trong process, kiểm validator trước handler và quan sát toàn bộ đường gọi bằng telemetry behavior."
category: csharp-advanced
level: advanced
tags: [csharp, dotnet, dotnet-10, toi-noi, architecture]
language: vi
---

Đây là **bài 32** của loạt **C# và .NET 10 qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Bài 31–38 cùng giải thích code cuối của service tại `examples/toi-noi-dotnet/31-38-trip-service`, target `net10.0`.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>CQRS có bắt buộc dùng hai database và event sourcing không?</p>
<details><summary>Xem đáp án</summary><p>Không. Lab tách mô hình/đường xử lý ghi và đọc trên cùng PostgreSQL; event sourcing là quyết định khác.</p></details>
</div>

## Command diễn đạt ý định, query diễn đạt nhu cầu đọc

CreateTrip và AcceptTrip là ý định thay đổi trạng thái. GetTrip trả view theo quyền sở hữu. Mỗi loại có handler riêng; API không tự biết cách new aggregate hoặc join dữ liệu để dựng response.

CQRS làm rõ hai đường có yêu cầu khác nhau. Query dùng projection no-tracking, không tải aggregate chỉ để hiển thị. Command gọi hành vi domain, commit qua UoW. Cùng database vẫn có CQRS; không cần dựng hai kho dữ liệu trước khi có nhu cầu.

Tên command không phải lệnh SQL và mediator không là message broker. Gửi request trong process không tạo durability, redelivery hoặc transaction với một service ngoài.

## Dispatcher typed nhỏ để nhìn cơ chế

`IRequest<TResponse>` gắn kiểu kết quả vào request. `IRequestHandler<TRequest,TResponse>` xử lý cặp kiểu đó. SendAsync nhận hai generic type tường minh để việc resolve và hợp đồng compile-time dễ đọc, không dùng dynamic hoặc quét assembly bằng reflection trong mỗi request.

Dispatcher ở Infrastructure lấy handler và các behavior từ scope DI, ghép delegate rồi chạy. Đây là implementation giảng giải có phạm vi nhỏ; không hứa thay thế một thư viện mediator về notifications, source generation, streaming, tối ưu hoặc chẩn đoán đăng ký.

Khi chọn thư viện cho dự án thật cần xem tính năng, license và lifecycle hiện tại. Bài này chủ động dạy pattern bằng code nhìn thấy được, không yêu cầu người đọc tin một lời gọi Send bí ẩn.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 1. Điều phối trong process; không tạo giao hàng bền giữa service."><defs><marker id="cs32-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Typed request</text><text x="34" y="123" class="t-small">Create / Get / Accept</text><path d="M280 108 L305 108" class="f-none s-phys" stroke-width="2" marker-end="url(#cs32-0-a)"/><rect x="305" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="319" y="95" class="t-b">Mediator trong scope</text><text x="319" y="123" class="t-small">Chọn handler + pipeline</text><path d="M565 108 L590 108" class="f-none s-phys" stroke-width="2" marker-end="url(#cs32-0-a)"/><rect x="590" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="604" y="95" class="t-b">Typed response</text><text x="604" y="123" class="t-small">Không phải broker ack</text></svg>
</div>
<figcaption>Hình 1. Điều phối trong process; không tạo giao hàng bền giữa service.</figcaption>
</figure>

## Pipeline có thứ tự và có hậu quả

Telemetry được đăng ký trước Validation; vòng Reverse xây delegate sao cho telemetry bọc validation, validation đứng trước handler. Vì vậy một yêu cầu bị từ chối vẫn được tính vào metric/thời gian ở biên application, nhưng không tạo hàng dữ liệu.

Validator kiểm token hủy, miền distance, precision và key. Handler chịu dùng domain cho rule nghiệp vụ. Không đưa transaction chung bao mọi query chỉ vì muốn thêm một behavior: sẽ mở transaction không cần thiết và làm khó giải thích replay/idempotency.

Mỗi behavior gọi next đúng theo trách nhiệm. Quên next có thể làm use case không chạy; gọi hai lần có thể lặp tác động. Test HTTP với trạng thái DB/Outbox là một cách quan sát hậu quả chứ không chỉ assert delegate đã được gọi.

## Cancellation đi xuyên thay vì bị nuốt

CancellationToken từ request được truyền qua dispatcher, handler, repository/query và SaveChanges. Validation check pre-cancelled token trước khi gọi handler. Test yêu cầu command đã hủy không tạo Trip.

Đây là hợp tác hủy, không bảo đảm request timeout thì database chắc chắn chưa commit. Client cần idempotency và tra kết quả bằng hợp đồng rõ. Một pipeline logging cũng không được bắt mọi exception rồi trả một TResponse mặc định có vẻ thành công.

Lỗi đã hiểu được map ở API: validation 400, không có tài nguyên được phép đọc 404, conflict 409. Application không tham chiếu Results hay HttpContext để quyết định HTTP; nó cung cấp loại lỗi có ngữ nghĩa.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 2. Thứ tự behavior là một phần hành vi có thể kiểm chứng, không chỉ là thứ tự tên trong DI."><defs><marker id="cs32-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Telemetry</text><text x="34" y="123" class="t-small">Bọc cả đường gọi</text><path d="M280 108 L305 108" class="f-none s-phys" stroke-width="2" marker-end="url(#cs32-1-a)"/><rect x="305" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="319" y="95" class="t-b">Validation</text><text x="319" y="123" class="t-small">Từ chối trước ghi</text><path d="M565 108 L590 108" class="f-none s-phys" stroke-width="2" marker-end="url(#cs32-1-a)"/><rect x="590" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="604" y="95" class="t-b">Handler</text><text x="604" y="123" class="t-small">Domain + commit</text></svg>
</div>
<figcaption>Hình 2. Thứ tự behavior là một phần hành vi có thể kiểm chứng, không chỉ là thứ tự tên trong DI.</figcaption>
</figure>

## Bài tập

**Bài tập:** đổi thứ tự hai behavior: metric của request bị validator từ chối có thể đổi thế nào? Có cần làm GetTrip đi qua repository aggregate để được gọi là CQRS không?

<details><summary>Xem đáp án</summary><p>Nếu validation bọc ngoài telemetry rồi ném trước next, đoạn telemetry trong không chạy. Query không bắt buộc đi qua aggregate; DTO projection riêng chính là ranh giới đọc đang được lựa chọn.</p></details>

## Code và kiểm chứng

Thiết lập `TOI_NOI_PG` cho database local riêng `toi_noi_dotnet_lab` theo README bài 21, rồi từ gốc repository:

```bash
dotnet test examples/toi-noi-dotnet/31-38-trip-service/Tests
```

Fixture dùng schema GUID riêng, không xóa database; JWT dùng khóa ngẫu nhiên trong test và vẫn đi qua handler xác thực thật. Test không tự skip khi thiếu PostgreSQL. Docker/CI/cloud spec xem README service và cloud/README.md để biết phạm vi, prerequisites và phần chưa được triển khai thật.

### Dispatcher.cs — 31-38-trip-service/Infrastructure/Dispatcher.cs

```csharp
using Microsoft.Extensions.DependencyInjection;
using ToiNoi.Trips.Application;
namespace ToiNoi.Trips.Infrastructure;
// Typed in-process teaching mediator: no broker, notification fan-out or library compatibility claim.
public sealed class RequestDispatcher(IServiceProvider services) : IRequestDispatcher
{
    public Task<TResponse> SendAsync<TRequest, TResponse>(TRequest request, CancellationToken ct) where TRequest : IRequest<TResponse>
    {
        var handler = services.GetRequiredService<IRequestHandler<TRequest, TResponse>>();
        RequestNext<TResponse> next = () => handler.HandleAsync(request, ct);
        foreach (var behavior in services.GetServices<IPipelineBehavior<TRequest, TResponse>>().Reverse())
        {
            var captured = next; next = () => behavior.HandleAsync(request, captured, ct);
        }
        return next();
    }
}
```

### Pipeline.cs — 31-38-trip-service/Application/Pipeline.cs

```csharp
using System.Diagnostics;
using System.Diagnostics.Metrics;
namespace ToiNoi.Trips.Application;
public static class TripDiagnostics
{
    public const string Name = "ToiNoi.Trips";
    public static readonly ActivitySource Source = new(Name);
    public static readonly Meter Meter = new(Name);
    public static readonly Counter<long> Requests = Meter.CreateCounter<long>("trips.requests", unit: "{request}");
    public static readonly Histogram<double> Duration = Meter.CreateHistogram<double>("trips.request.duration", unit: "ms");
}
public sealed class ValidationBehavior<TRequest, TResponse>(IEnumerable<IRequestValidator<TRequest>> validators)
    : IPipelineBehavior<TRequest, TResponse> where TRequest : IRequest<TResponse>
{
    public Task<TResponse> HandleAsync(TRequest request, RequestNext<TResponse> next, CancellationToken ct)
    {
        ct.ThrowIfCancellationRequested(); foreach (var validator in validators) validator.Validate(request);
        return next();
    }
}
public sealed class TelemetryBehavior<TRequest, TResponse> : IPipelineBehavior<TRequest, TResponse> where TRequest : IRequest<TResponse>
{
    public async Task<TResponse> HandleAsync(TRequest request, RequestNext<TResponse> next, CancellationToken ct)
    {
        string operation = typeof(TRequest).Name; long start = Stopwatch.GetTimestamp();
        using var activity = TripDiagnostics.Source.StartActivity(operation);
        activity?.SetTag("app.operation", operation);
        TripDiagnostics.Requests.Add(1, new KeyValuePair<string, object?>("operation", operation));
        try { return await next(); }
        catch { activity?.SetStatus(ActivityStatusCode.Error); throw; }
        finally { TripDiagnostics.Duration.Record(Stopwatch.GetElapsedTime(start).TotalMilliseconds, new KeyValuePair<string, object?>("operation", operation)); }
    }
}
```

Bài trước: [bài 31](/articles/dotnet10-clean-architecture-layers). Bài tiếp: [bài 33](/articles/dotnet10-repository-unit-of-work).

## Tài liệu gốc

- [Application layer and Mediator](https://learn.microsoft.com/en-us/dotnet/architecture/microservices/microservice-ddd-cqrs-patterns/microservice-application-layer-implementation-web-api).
- [CQRS](https://learn.microsoft.com/en-us/azure/architecture/patterns/cqrs).
