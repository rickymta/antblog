---
slug: dotnet10-opentelemetry-traces-metrics-logs
title: "OpenTelemetry: lần theo request qua HTTP, handler và PostgreSQL"
summary: "Bài 37: ActivitySource/Meter cùng instrumentation ASP.NET/HTTP, traceparent đi qua handler tới commit và metric có tag hữu hạn; xuất OTLP chỉ khi cấu hình rõ."
category: csharp-advanced
level: advanced
tags: [csharp, dotnet, dotnet-10, toi-noi, architecture]
language: vi
---

Đây là **bài 37** của loạt **C# và .NET 10 qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Bài 31–38 cùng giải thích code cuối của service tại `examples/toi-noi-dotnet/31-38-trip-service`, target `net10.0`.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Đưa TripId và CustomerId làm metric tag có tiện tra từng request mà không có cái giá nào không?</p>
<details><summary>Xem đáp án</summary><p>Không. Tag có số giá trị tăng theo dữ liệu sẽ làm cardinality bùng nổ; metric cần dimension có giới hạn, còn trace/log có chính sách riêng.</p></details>
</div>

## Từ một status 500 tới đường request cụ thể

Một dòng log “lỗi database” chưa cho biết request nào, use case nào và đoạn nào chậm. Trace nối các span có quan hệ nhân quả; metric cho biết phân bố và tần suất; log ghi sự kiện có cấu trúc. Chúng bổ sung nhau và không thay thế được debugger hoặc test.

Service dùng nguồn ToiNoi.Trips cho handler và các điểm database đã chọn. Instrumentation ASP.NET tạo span HTTP, HttpClient hỗ trợ propagation khi có lời gọi ra ngoài. Lab không có thành phần phụ thuộc Python/Rust/Go chạy thật để khẳng định đã trace toàn hệ thống đa ngôn ngữ.

Test gửi traceparent cố định, quan sát cùng TraceId ở CreateTrip và db.commit. Đây là bằng chứng propagation qua pipeline local, không phải ảnh chụp dashboard cloud thu từ production.

## ActivitySource cần listener

StartActivity có thể trả null khi không có listener/sampling phù hợp. Code dùng ?. khi gắn tag/status. OpenTelemetry provider đăng ký source chính là một listener; test còn dùng ActivityListener để quan sát độc lập mà không cần gửi telemetry ra ngoài.

TelemetryBehavior bọc validation/handler, đo thời gian bằng Stopwatch và đánh dấu Error khi lỗi truyền lên. Các span repository/commit chỉ mô tả điểm đang đo; duration db.commit bao trùm đường SaveChanges của adapter, không đồng nghĩa với thời gian CPU thuần của PostgreSQL.

Không bật việc log tham số SQL, body hay token để “trace cho đủ”. Các thông tin nhạy cảm phải có chính sách thu thập, che giấu dữ liệu (redaction) và thời gian lưu giữ (retention); trace ID giúp nối sự kiện mà không cần nhét dữ liệu khách vào mọi tag.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 1. Propagation được test qua pipeline local; chưa phải là trace xuyên các service cloud thật."><defs><marker id="cs37-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">HTTP traceparent</text><text x="34" y="123" class="t-small">Context từ caller</text><path d="M280 108 L305 108" class="f-none s-phys" stroke-width="2" marker-end="url(#cs37-0-a)"/><rect x="305" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="319" y="95" class="t-b">CreateTrip span</text><text x="319" y="123" class="t-small">Use case local</text><path d="M565 108 L590 108" class="f-none s-phys" stroke-width="2" marker-end="url(#cs37-0-a)"/><rect x="590" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="604" y="95" class="t-b">db.commit span</text><text x="604" y="123" class="t-small">Cùng TraceId</text></svg>
</div>
<figcaption>Hình 1. Propagation được test qua pipeline local; chưa phải là trace xuyên các service cloud thật.</figcaption>
</figure>

## Metric cần đơn vị và cardinality rõ

trips.requests là counter đếm số request của application, trips.request.duration là histogram ms. Tag operation lấy từ một tập kiểu request được đăng ký, không lấy từ ID chuyến, URL tuỳ ý hoặc idempotency key.

Với thứ tự behavior hiện tại, counter tính cả request bị validator từ chối, nên không phải là số chuyến đã commit thành công. Nếu cần metric committed trips, đặt nó ở một mốc được định nghĩa riêng và test theo đúng ngữ nghĩa đó.

Test MeterListener kiểm tra cả counter, duration và tên tag. Test này không chứng minh backend metric giữ mọi sample, dashboard tính percentile đúng hoặc sampling production phù hợp. Query và report ở hệ thống quan sát cần được review cùng đơn vị và phép gộp (aggregation) tương ứng.

## Export có cấu hình và phạm vi tin cậy

Mặc định Telemetry:Export=false nên lab không gửi dữ liệu đi. Khi bật, OTLP exporter dùng endpoint đã được cấu hình và tin cậy trong môi trường. Collector, TLS, auth, sampling và retention là hạ tầng cần thiết kế, không hard-code một endpoint public trong source.

Logging OpenTelemetry giữ scope và trạng thái có cấu trúc, không thu body/Authorization. Việc dùng chung resource service.name giúp nối dữ liệu nhưng không tự đồng bộ thời gian hay bổ sung instrument còn thiếu. Khi đổi phiên bản dịch vụ hoặc image, nên phản ánh thay đổi đó trong metadata release theo quy ước của đội.

**Bài tập:** nếu chỉ thu span HTTP mà thiếu span handler, có thể biết SQL cụ thể chậm ở đâu không? Một counter request tăng có chứng minh transaction commit không?

<details><summary>Xem đáp án</summary><p>Không đủ chi tiết để xác định nguyên nhân; phải đo đúng các ranh giới cần quan sát. Counter hiện tại đếm request đi vào behavior, không cam kết commit thành công.</p></details>

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 2. Ba tín hiệu có hợp đồng riêng; không dùng ID khách làm dimension của metric có số giá trị vô hạn."><defs><marker id="cs37-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Trace</text><text x="34" y="123" class="t-small">Quan hệ + thời gian</text><rect x="305" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="319" y="95" class="t-b">Metric</text><text x="319" y="123" class="t-small">Tập hợp + phân bố</text><rect x="590" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="604" y="95" class="t-b">Log</text><text x="604" y="123" class="t-small">Sự kiện có cấu trúc</text></svg>
</div>
<figcaption>Hình 2. Ba tín hiệu có hợp đồng riêng; không dùng ID khách làm dimension của metric có số giá trị vô hạn.</figcaption>
</figure>

## Code và kiểm chứng

Thiết lập `TOI_NOI_PG` cho database local riêng `toi_noi_dotnet_lab` theo README bài 21, rồi từ gốc repository:

```bash
dotnet test examples/toi-noi-dotnet/31-38-trip-service/Tests
```

Fixture dùng schema GUID riêng, không xoá database; JWT dùng khoá ngẫu nhiên trong test và vẫn đi qua handler xác thực thật. Test không tự bỏ qua khi thiếu PostgreSQL. Docker/CI/cloud spec xem README service và cloud/README.md để biết phạm vi, điều kiện tiên quyết và phần chưa triển khai thật.

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

### TelemetryTests.cs — 31-38-trip-service/Tests/TelemetryTests.cs

```csharp
using System.Collections.Concurrent;
using System.Diagnostics;
using System.Diagnostics.Metrics;
using System.Net;
using System.Net.Http.Json;
using ToiNoi.Trips.Application;
using Xunit;
namespace ToiNoi.Trips.Tests;
public sealed class TelemetryTests
{
    [Fact]
    public async Task Incoming_trace_flows_through_handler_and_database_commit()
    {
        await using var lab = await ServiceLab.CreateAsync(); using var factory = new ApiFactory(lab.Connection);
        using var client = factory.Client(Guid.NewGuid(), "customer");
        var captured = new ConcurrentQueue<(string Operation, ActivityTraceId Trace)>();
        using var listener = new ActivityListener
        {
            ShouldListenTo = source => source.Name == TripDiagnostics.Name,
            Sample = (ref ActivityCreationOptions<ActivityContext> _) => ActivitySamplingResult.AllDataAndRecorded,
            ActivityStopped = activity => captured.Enqueue((activity.OperationName, activity.TraceId))
        };
        ActivitySource.AddActivityListener(listener);
        const string trace = "11111111111111111111111111111111";
        using var request = new HttpRequestMessage(HttpMethod.Post, "/trips") { Content = JsonContent.Create(new { distanceKm = 5m }) };
        request.Headers.Add("Idempotency-Key", "trace-1"); request.Headers.Add("traceparent", $"00-{trace}-2222222222222222-01");
        using var response = await client.SendAsync(request); Assert.Equal(HttpStatusCode.Created, response.StatusCode);
        Assert.Contains(captured, item => item.Operation == "CreateTrip" && item.Trace.ToHexString() == trace);
        Assert.Contains(captured, item => item.Operation == "db.commit" && item.Trace.ToHexString() == trace);
    }
    [Fact]
    public async Task Metrics_use_bounded_operation_tag_not_customer_or_trip_ids()
    {
        await using var lab = await ServiceLab.CreateAsync(); using var factory = new ApiFactory(lab.Connection);
        using var client = factory.Client(Guid.NewGuid(), "customer");
        var records = new ConcurrentQueue<(string Name, string[] TagNames, string? Operation)>();
        using var listener = new MeterListener();
        listener.InstrumentPublished = (instrument, current) => { if (instrument.Meter.Name == TripDiagnostics.Name) current.EnableMeasurementEvents(instrument); };
        listener.SetMeasurementEventCallback<long>((instrument, _, tags, _) => Capture(instrument, tags));
        listener.SetMeasurementEventCallback<double>((instrument, _, tags, _) => Capture(instrument, tags));
        void Capture(Instrument instrument, ReadOnlySpan<KeyValuePair<string, object?>> tags)
        {
            var names = new List<string>(); string? operation = null;
            foreach (var tag in tags) { names.Add(tag.Key); if (tag.Key == "operation") operation = tag.Value?.ToString(); }
            records.Enqueue((instrument.Name, names.ToArray(), operation));
        }
        listener.Start();
        using var request = new HttpRequestMessage(HttpMethod.Post, "/trips") { Content = JsonContent.Create(new { distanceKm = 5m }) };
        request.Headers.Add("Idempotency-Key", "metric-1"); using var response = await client.SendAsync(request);
        Assert.Equal(HttpStatusCode.Created, response.StatusCode);
        Assert.Contains(records, item => item.Name == "trips.requests" && item.Operation == "CreateTrip");
        Assert.Contains(records, item => item.Name == "trips.request.duration" && item.Operation == "CreateTrip");
        Assert.All(records, item => Assert.All(item.TagNames, name => Assert.Equal("operation", name)));
    }
}
```

Bài trước: [bài 36](/articles/dotnet10-aws-azure-deployment). Bài tiếp: [bài 38](/articles/dotnet10-tong-hop-service-dat-chuyen).

## Tài liệu gốc

- [OpenTelemetry in .NET](https://learn.microsoft.com/en-us/dotnet/core/diagnostics/observability-with-otel).
- [OpenTelemetry .NET traces](https://github.com/open-telemetry/opentelemetry-dotnet/blob/main/docs/trace/README.md).
