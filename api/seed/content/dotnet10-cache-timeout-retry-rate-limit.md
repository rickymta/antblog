---
slug: dotnet10-cache-timeout-retry-rate-limit
title: "Cache, timeout, retry và rate limiting: giữ API .NET 10 ổn định"
summary: "Bài 25: cache bảng giá công khai, giới hạn request, retry GET có điều kiện và timeout dependency hợp tác; phân biệt từng cơ chế với đảm bảo giao dịch."
category: csharp-intermediate
level: intermediate
tags: [csharp, dotnet, dotnet-10, toi-noi]
language: vi
---

Đây là **bài 25** trong loạt **C# và .NET 10 qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Các ví dụ target `net10.0`, dùng dữ liệu giả và cùng code đã chạy kiểm chứng trong repository.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Một POST thanh toán timeout có thể retry an toàn chỉ vì dùng HttpClient resilience không?</p>
<details><summary>Xem đáp án</summary><p>Không. Server có thể đã thực hiện hiệu ứng; retry cần hợp đồng idempotency và tra trạng thái, không chỉ một policy kỹ thuật.</p></details>
</div>

## Bốn cơ chế, bốn câu hỏi

Cache tránh tính/đọc lại dữ liệu phù hợp; timeout giới hạn thời gian chờ; retry thử lại lỗi tạm theo hợp đồng; rate limiting quyết định lượng yêu cầu được nhận. Chúng không cùng chữa một loại lỗi và không thay transaction.

Lab tạo GET /tariffs cho bảng giá công khai, không cache báo giá riêng của khách hoặc endpoint có quyền sở hữu. Named HttpClient partner dùng handler giả trong test, không gọi ra Internet. Giá trị bảng giá vẫn là dữ liệu minh họa của Tới Nơi.

Mục tiêu là quan sát từng chính sách. Một bộ limiter cực nhỏ với permit 2 không phải cấu hình production; nó giúp request thứ ba bộc lộ đường từ chối mà không phải bắn tải lớn.

## Cache cần key, thời hạn và cách mất hiệu lực

IMemoryCache dùng key tariff:v1, absolute expiration 10 giây. Hai lần đọc tuần tự trong thời gian cache sống chỉ gọi TariffSource một lần; Remove key rồi đọc lại làm nguồn được gọi lần hai.

GetOrCreate không tự bảo đảm mọi caller đồng thời chỉ có một lần fill. Lab không tuyên bố đã giải cache stampede. Key cần phản ánh version/biến số làm đổi nội dung; nếu thêm thành phố hay loại dịch vụ thì key đơn giản hiện tại chưa đủ.

Memory cache ở mỗi replica khác nhau, mất khi process dừng. Không dùng nó làm nguồn chân lý cho tiền hoặc quyền. Nếu dữ liệu đã lỗi thời có thể ảnh hưởng giao dịch, quy tắc freshness và invalidation phải được định nghĩa cùng nghiệp vụ.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 1. Limiter kiểm soát đầu vào trước cache/handler; cache không xóa giới hạn tải."><defs><marker id="cs25-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Request</text><text x="34" y="123" class="t-small">Qua limiter</text><path d="M280 115 L305 115" class="f-none s-phys" stroke-width="2" marker-end="url(#cs25-0-arrow)"/><rect x="305" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="319" y="95" class="t-b">Cache public</text><text x="319" y="123" class="t-small">Hit / load nguồn</text><path d="M565 115 L590 115" class="f-none s-phys" stroke-width="2" marker-end="url(#cs25-0-arrow)"/><rect x="590" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="604" y="95" class="t-b">Response</text><text x="604" y="123" class="t-small">Bảng giá theo version</text></svg>
</div>
<figcaption>Hình 1. Limiter kiểm soát đầu vào trước cache/handler; cache không xóa giới hạn tải.</figcaption>
</figure>

## Retry phải có ngân sách và loại thao tác

AddStandardResilienceHandler cấu hình một lần retry, delay bằng 0 và tắt jitter chỉ để test xác định. GET gặp 503 rồi thành công được gửi hai lần; POST cùng failure không retry theo DisableForUnsafeHttpMethods. Production thường cần backoff/jitter và quan sát để tránh retry storm.

Attempt timeout 1 giây, tổng budget 3 giây. HttpClient.Timeout được đặt vô hạn để lab có một lớp điều khiển timeout chính rõ ràng; không phải khuyến cáo bỏ mọi deadline. Token của caller và các timeout vẫn cần được phân biệt trong log/metric.

Handler chờ trong test nhận token và hợp tác dừng. Timeout là tín hiệu giới hạn chờ/xử lý theo pipeline, không đảm bảo một dependency phớt lờ cancellation sẽ tự chết. Retry POST cần thiết kế idempotency ngay cả khi cấu hình kỹ thuật cho phép nó.

## Rate limiting đứng trước handler

Fixed-window limiter nhận 2 request trong cửa sổ một phút, queue bằng 0 và tự replenishment. Request vượt nhận 429, kể cả bảng giá đang cache: giới hạn nhận request và cache hit là hai quyết định riêng.

Policy demo không partition theo người dùng nên là giới hạn chia sẻ trong process cho endpoint gắn policy. Nó không phải quota toàn cluster; production có thể cần partition theo danh tính đã xác thực, IP đáng tin qua proxy hoặc một hạ tầng phân tán. Không tin header userId tự gửi để cấp quota.

Không xếp một hàng dài ở gateway, thêm hàng dài ở limiter rồi thêm Channel vô hạn bên trong. Tổng thời gian và số yêu cầu đang giữ tài nguyên cần có budget rõ, nếu không “thử lại” chỉ kéo dài một sự cố.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 275" role="img" aria-label="Hình 2. Không áp cùng chính sách retry cho mọi request có thể tạo hiệu ứng."><defs><marker id="cs25-1-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="95" width="235" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="125" class="t-b">Dependency lỗi</text><text x="34" y="153" class="t-small">Xem loại thao tác</text><rect x="465" y="25" width="385" height="85" rx="10" class="f-warn-soft s-warn"/><text x="479" y="55" class="t-b">GET + 503</text><text x="479" y="83" class="t-small">Retry trong budget</text><rect x="465" y="165" width="385" height="85" rx="10" class="f-phys-soft s-phys"/><text x="479" y="195" class="t-b">POST + 503</text><text x="479" y="223" class="t-small">Không retry trong lab</text><path d="M255 120 L465 68" class="f-none s-phys" stroke-width="2" marker-end="url(#cs25-1-arrow)"/><path d="M255 150 L465 208" class="f-none s-phys" stroke-width="2" marker-end="url(#cs25-1-arrow)"/></svg>
</div>
<figcaption>Hình 2. Không áp cùng chính sách retry cho mọi request có thể tạo hiệu ứng.</figcaption>
</figure>

## Bốn ca kiểm chứng và bài tập

Các ca kiểm cache hit/429, invalidation, retry GET nhưng không POST và timeout dependency hợp tác. Cache expiry thực tế 10 giây là cấu hình được đặt, chưa có fake-clock test chứng minh biên hết hạn trong bộ này.

**Bài tập:** nếu bảng giá khác theo city, key tariff:v1 có còn đúng? Khi request thứ ba cache hit nhưng bị 429, đó có phải cache hỏng không?

<details><summary>Xem đáp án</summary><p>Key phải mang đủ biến số như city và phiên bản. 429 đến từ limiter trước handler; cache đúng không có nghĩa mọi request được nhận vô hạn.</p></details>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
dotnet test examples/toi-noi-dotnet/25-cache-resilience/Resilience.Tests
```

### Program.cs — 25-cache-resilience/ResilienceApi/Program.cs

```csharp
using Microsoft.AspNetCore.RateLimiting;
using Microsoft.Extensions.Caching.Memory;
using Microsoft.Extensions.Http.Resilience;
using System.Threading.RateLimiting;
namespace ToiNoi.ResilienceApi;
public partial class Program
{
    public static void Main(string[] args)
    {
        var builder = WebApplication.CreateBuilder(args);
        builder.Services.AddMemoryCache(); builder.Services.AddSingleton<TariffSource>();
        builder.Services.AddHttpClient("partner", client =>
        { client.BaseAddress = new Uri("https://partner.invalid/"); client.Timeout = Timeout.InfiniteTimeSpan; })
            .AddStandardResilienceHandler(options =>
            {
                options.Retry.MaxRetryAttempts = 1; options.Retry.Delay = TimeSpan.Zero; options.Retry.UseJitter = false;
                options.Retry.DisableForUnsafeHttpMethods();
                options.AttemptTimeout.Timeout = TimeSpan.FromSeconds(1);
                options.TotalRequestTimeout.Timeout = TimeSpan.FromSeconds(3);
            });
        builder.Services.AddRateLimiter(options =>
        {
            options.RejectionStatusCode = 429;
            options.AddFixedWindowLimiter("public-tariffs", limiter =>
            { limiter.PermitLimit = 2; limiter.Window = TimeSpan.FromMinutes(1); limiter.QueueLimit = 0; limiter.AutoReplenishment = true; });
        });
        var app = builder.Build(); app.UseRateLimiter();
        app.MapGet("/tariffs", (IMemoryCache cache, TariffSource source) =>
            cache.GetOrCreate("tariff:v1", entry =>
            { entry.AbsoluteExpirationRelativeToNow = TimeSpan.FromSeconds(10); return source.Read(); }))
            .RequireRateLimiting("public-tariffs");
        app.Run();
    }
}
public sealed record Tariff(decimal BaseFare, string Currency, int Version);
public sealed class TariffSource
{
    private int loads;
    public int Loads => Volatile.Read(ref loads);
    public Tariff Read() { Interlocked.Increment(ref loads); return new Tariff(12000m, "VND", 1); }
}
```

### ResilienceTests.cs — 25-cache-resilience/Resilience.Tests/ResilienceTests.cs

```csharp
using System.Net;
using Microsoft.AspNetCore.Mvc.Testing;
using Microsoft.AspNetCore.Hosting;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Caching.Memory;
using Polly.Timeout;
using ToiNoi.ResilienceApi;
using Xunit;
using Entry = ToiNoi.ResilienceApi.Program;
namespace ToiNoi.ResilienceTests;
public sealed class ResilienceTests
{
    [Fact]
    public async Task Public_cache_reuses_value_and_third_request_is_rate_limited()
    {
        using var factory = new WebApplicationFactory<Entry>(); using var client = factory.CreateClient();
        using var first = await client.GetAsync("/tariffs"); using var second = await client.GetAsync("/tariffs");
        Assert.Equal(HttpStatusCode.OK, first.StatusCode); Assert.Equal(HttpStatusCode.OK, second.StatusCode);
        Assert.Equal(1, factory.Services.GetRequiredService<TariffSource>().Loads);
        using var third = await client.GetAsync("/tariffs"); Assert.Equal((HttpStatusCode)429, third.StatusCode);
    }
    [Fact]
    public async Task Explicit_invalidation_reloads_public_data()
    {
        using var factory = new WebApplicationFactory<Entry>(); using var client = factory.CreateClient();
        using var first = await client.GetAsync("/tariffs"); factory.Services.GetRequiredService<IMemoryCache>().Remove("tariff:v1");
        using var second = await client.GetAsync("/tariffs"); Assert.Equal(2, factory.Services.GetRequiredService<TariffSource>().Loads);
    }
    [Fact]
    public async Task Get_retries_one_transient_failure_post_does_not_retry()
    {
        var handler = new SequenceHandler();
        using var factory = new WebApplicationFactory<Entry>().WithWebHostBuilder(b => b.ConfigureServices(s =>
            s.AddHttpClient("partner").ConfigurePrimaryHttpMessageHandler(() => handler)));
        var http = factory.Services.GetRequiredService<IHttpClientFactory>().CreateClient("partner");
        using var get = await http.GetAsync("tariff"); Assert.Equal(HttpStatusCode.OK, get.StatusCode); Assert.Equal(2, handler.Calls);
        handler.Calls = 0; using var post = await http.PostAsync("payment", null);
        Assert.Equal(HttpStatusCode.ServiceUnavailable, post.StatusCode); Assert.Equal(1, handler.Calls);
    }
    [Fact]
    public async Task Cooperative_dependency_is_bounded_by_resilience_timeout()
    {
        using var factory = new WebApplicationFactory<Entry>().WithWebHostBuilder(b => b.ConfigureServices(s =>
            s.AddHttpClient("partner").ConfigurePrimaryHttpMessageHandler(() => new HangingHandler())));
        var http = factory.Services.GetRequiredService<IHttpClientFactory>().CreateClient("partner");
        await Assert.ThrowsAsync<TimeoutRejectedException>(() => http.GetAsync("slow").WaitAsync(TimeSpan.FromSeconds(8)));
    }
    private sealed class SequenceHandler : HttpMessageHandler
    {
        public int Calls;
        protected override Task<HttpResponseMessage> SendAsync(HttpRequestMessage request, CancellationToken ct) =>
            Task.FromResult(new HttpResponseMessage(Interlocked.Increment(ref Calls) == 1 ? HttpStatusCode.ServiceUnavailable : HttpStatusCode.OK));
    }
    private sealed class HangingHandler : HttpMessageHandler
    {
        protected override async Task<HttpResponseMessage> SendAsync(HttpRequestMessage request, CancellationToken ct)
        { await Task.Delay(Timeout.InfiniteTimeSpan, ct); throw new InvalidOperationException("Unreachable."); }
    }
}
```

Bài trước: [bài 24](/articles/dotnet10-backgroundservice-channel). Bài tiếp: [bài 26](/articles/dotnet10-outbox-inbox-idempotent-consumer).

## Tài liệu gốc

- [HTTP resilience](https://learn.microsoft.com/en-us/dotnet/core/resilience/http-resilience).
- [ASP.NET Core rate limiting](https://learn.microsoft.com/en-us/aspnet/core/performance/rate-limit?view=aspnetcore-10.0).
- [In-memory cache](https://learn.microsoft.com/en-us/aspnet/core/performance/caching/memory?view=aspnetcore-10.0).
