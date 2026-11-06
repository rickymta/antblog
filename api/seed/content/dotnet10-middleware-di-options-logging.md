---
slug: dotnet10-middleware-di-options-logging
title: "Middleware, DI lifetime, cấu hình và logging trong ASP.NET Core 10"
summary: "Bài 18: theo dõi request qua middleware, quan sát scoped/transient, validate options khi khởi động và tiêm TimeProvider để kiểm thử thời gian."
category: csharp
level: intermediate
tags: [csharp, dotnet, dotnet-10, toi-noi, aspnet-core]
language: vi
---

Đây là **bài 18** trong loạt **C# và .NET 10 qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Các ví dụ target `net10.0`; code trong repository và các đoạn đầy đủ dưới đây dùng cùng implementation được chạy kiểm chứng.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Một singleton trong container có nghĩa là chỉ có một instance cho toàn bộ cluster Tới Nơi không?</p>
<details><summary>Xem đáp án</summary><p>Không. Nó sống theo container/process đang chạy; mỗi replica có container và bộ nhớ riêng.</p></details>
</div>

## Một request đi qua những đâu?

Khi API trả giá sai, ta cần biết bảng giá đã cấu hình, request được xử lý bởi luồng nào và timestamp nào đi ra. Rải Console.WriteLine trong policy không tạo được một cách quan sát thống nhất.

Middleware bọc đường xử lý HTTP; DI quản lý object graph/lifetime; options tạo hợp đồng cấu hình; logging ghi sự kiện có cấu trúc. Tách vai trò giúp code domain không phải biết header hoặc cách đọc file appsettings.

Trong lab, policy cấu hình vẫn chỉ là giá cơ bản với đơn giá phần vượt cố định. Không mô phỏng hệ thống version giá theo thời điểm hiệu lực; snapshot cấu hình và rule đổi giá thật sẽ cần thiết kế rộng hơn.

## Thứ tự middleware có ý nghĩa

UseExceptionHandler được đăng ký trước middleware quan sát để bọc lỗi phía trong. Middleware quan sát lưu trace identifier do server tạo, đăng ký OnStarting để thêm X-Trace-Id, mở logging scope rồi await next.

Đường request đi vào theo thứ tự đăng ký, quay ra sau await theo chiều ngược. Nếu không gọi next, middleware có thể short-circuit; nếu viết header quá muộn sau khi response bắt đầu, hành vi không còn như kỳ vọng. OnStarting là vị trí phù hợp để thêm header tại ranh giới đó.

Log Request completed chỉ chạy ở đường next trở về bình thường của middleware này; đoạn code chưa đo duration hay bảo đảm một log kết thúc cho mọi exception. Nếu cần đo cả lỗi, phải thiết kế try/finally và xác định status cuối do handler ngoài đặt, không gán cho ví dụ những bảo đảm nó chưa có.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 1. Đường xử lý bình thường được quan sát ở biên HTTP; code domain không cần đọc header."><defs><marker id="cs18-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Middleware vào</text><text x="34" y="123" class="t-small">TraceId + scope</text><path d="M280 115 L305 115" class="f-none s-phys" stroke-width="2" marker-end="url(#cs18-0-a)"/><rect x="305" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="319" y="95" class="t-b">Endpoint / policy</text><text x="319" y="123" class="t-small">Đọc options đã hợp lệ</text><path d="M565 115 L590 115" class="f-none s-phys" stroke-width="2" marker-end="url(#cs18-0-a)"/><rect x="590" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="604" y="95" class="t-b">Response</text><text x="604" y="123" class="t-small">Header + log status</text></svg>
</div>
<figcaption>Hình 1. Đường xử lý bình thường được quan sát ở biên HTTP; code domain không cần đọc header.</figcaption>
</figure>

## Scoped, transient và singleton qua một thí nghiệm

TimeProvider.System được đăng ký singleton, không mang dữ liệu riêng của khách. RequestStamp là scoped: cùng một request chia sẻ instance trong scope đó. StampReader là transient: mỗi lần resolve có instance mới, nhưng hai reader trong cùng request nhận chung RequestStamp.

Route diagnostics chỉ map trong Development, trả SameScoped=true, SameTransient=false và ScopeId. Hai request liên tiếp phải có ScopeId khác nhau. Đây là thí nghiệm lifetime, không là endpoint tiết lộ thông tin hạ tầng cần public ở production.

Không inject dependency scoped vào singleton rồi giữ lâu: nó có thể bị nâng lifetime và chia sẻ state ngoài ý muốn. Middleware truyền thống sống lâu cũng cần cách nhận dependency scoped đúng phạm vi, ví dụ qua InvokeAsync, thay vì cất vào constructor tùy tiện.

## Options hợp lệ trước khi nhận request

FareOptions được bind từ mục Fare. ValidateOnStart buộc BaseFare nằm trong miền lab trước khi host phục vụ. Nếu cấu hình âm, test mong OptionsValidationException khi khởi động, không chờ tới request đầu rồi trả giá âm.

Trong host mặc định, biến môi trường `Fare__BaseFare` có thể ghi đè khóa cấu hình theo provider order. Không log toàn bộ configuration để debug vì các mục khác có thể chứa bí mật. Chỉ ghi trường đã được phân loại phù hợp hoặc phiên bản cấu hình cần quan sát.

ConfiguredFarePolicy dùng IOptions trong bài; điều này không hứa hot reload theo từng request. IOptionsSnapshot/IOptionsMonitor có lifetime và ý nghĩa cập nhật khác, chỉ đưa vào khi đã chốt yêu cầu. Test thay TimeProvider bằng FixedClock để timestamp có kỳ vọng cố định, không so “gần bằng DateTime.Now”.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 265" role="img" aria-label="Hình 2. Hai transient reader cùng request nhận chung scoped stamp; request khác có stamp khác."><defs><marker id="cs18-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="25" width="235" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="55" class="t-b">Reader A</text><text x="34" y="83" class="t-small">Transient riêng</text><rect x="465" y="90" width="385" height="85" rx="10" class="f-pk-soft s-pk"/><text x="479" y="120" class="t-b">RequestStamp</text><text x="479" y="148" class="t-small">Scoped dùng chung</text><rect x="20" y="155" width="235" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="185" class="t-b">Reader B</text><text x="34" y="213" class="t-small">Transient riêng</text><path d="M255 67 L465 115" class="f-none s-phys" stroke-width="2" marker-end="url(#cs18-1-a)"/><path d="M255 195 L465 150" class="f-none s-phys" stroke-width="2" marker-end="url(#cs18-1-a)"/></svg>
</div>
<figcaption>Hình 2. Hai transient reader cùng request nhận chung scoped stamp; request khác có stamp khác.</figcaption>
</figure>

## Logging theo sự kiện và bài tập

Log dùng placeholder `{StatusCode}` thay vì chỉ nối chuỗi. Scope thêm TraceId cho provider có hỗ trợ scope; cách hiển thị scope tùy cấu hình provider. X-Trace-Id giúp client gửi ID cho đội hỗ trợ mà không cần gửi body chứa dữ liệu khách.

Không log Authorization header, JWT hay giấy tờ OCR. Ví dụ tự dùng server TraceIdentifier thay vì phản chiếu một header tùy ý của client vào log; nếu sau này chấp nhận correlation từ ngoài phải validate/giới hạn theo hợp đồng.

**Bài tập:** đổi StampReader thành scoped. SameTransient còn false không? Đặt BaseFare=-1 rồi chạy host: vì sao fail fast tốt hơn âm thầm thay về 0?

<details><summary>Xem đáp án</summary><p>Cùng scope sẽ nhận cùng StampReader nên kết quả so instance đổi. Cấu hình sai cần được nhìn thấy và sửa; thay về 0 có thể biến lỗi vận hành thành báo giá miễn phí.</p></details>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
dotnet run --project examples/toi-noi-dotnet/18-middleware-di-options -- --urls http://127.0.0.1:5198 --environment Development
```

Đây là host học trên localhost; dừng bằng Ctrl+C sau khi thử. Bộ test của bài 20 kiểm tra các bước API 16–19 bằng cùng source mà không cần mở các port này. Không triển khai ví dụ chưa có đầy đủ cấu hình service lên public endpoint.

### Program.cs — 18-middleware-di-options/Program.cs

```csharp
using System.ComponentModel.DataAnnotations;
using Microsoft.Extensions.Options;
using ToiNoi.Fares;
namespace ToiNoi.Api18;
public partial class Program
{
    public static void Main(string[] args)
    {
        var builder = WebApplication.CreateBuilder(args);
        builder.Services.AddValidation();
        builder.Services.AddProblemDetails();
        builder.Services.AddOptions<FareOptions>().BindConfiguration("Fare")
            .Validate(o => o.BaseFare >= 0m && o.BaseFare <= 1_000_000m, "BaseFare outside demo range.")
            .ValidateOnStart();
        builder.Services.AddSingleton<TimeProvider>(TimeProvider.System);
        builder.Services.AddScoped<RequestStamp>();
        builder.Services.AddTransient<StampReader>();
        builder.Services.AddScoped<IFarePolicy, ConfiguredFarePolicy>();
        var app = builder.Build();
        app.UseExceptionHandler();
        app.Use(async (context, next) =>
        {
            string traceId = context.TraceIdentifier;
            context.Response.OnStarting(() =>
            {
                context.Response.Headers["X-Trace-Id"] = traceId;
                return Task.CompletedTask;
            });
            using (app.Logger.BeginScope(new Dictionary<string, object> { ["TraceId"] = traceId }))
            {
                await next(context);
                app.Logger.LogInformation("Request completed with {StatusCode}", context.Response.StatusCode);
            }
        });
        app.MapPost("/quotes", (QuoteRequest input, IFarePolicy policy, TimeProvider clock) =>
            TypedResults.Ok(new { Fare = policy.Calculate(input.DistanceKm!.Value), Currency = "VND", IssuedAt = clock.GetUtcNow() }))
            .ProducesValidationProblem();
        if (app.Environment.IsDevelopment())
            app.MapGet("/diagnostics/scope", (StampReader left, StampReader right) =>
                new { SameScoped = ReferenceEquals(left.Stamp, right.Stamp), SameTransient = ReferenceEquals(left, right), ScopeId = left.Stamp.Id });
        app.Run();
    }
}
public sealed class FareOptions { public decimal BaseFare { get; set; } = 12000m; }
public sealed class RequestStamp { public Guid Id { get; } = Guid.NewGuid(); }
public sealed class StampReader(RequestStamp stamp) { public RequestStamp Stamp { get; } = stamp; }
public sealed class ConfiguredFarePolicy(IOptions<FareOptions> options) : IFarePolicy
{
    public decimal Calculate(decimal distanceKm)
    {
        if (distanceKm < 0m || distanceKm > 100m) throw new ArgumentOutOfRangeException(nameof(distanceKm));
        return Math.Round(options.Value.BaseFare + Math.Max(0m, distanceKm - 2m) * 4500m,
            0, MidpointRounding.AwayFromZero);
    }
}
public sealed record QuoteRequest([property: Required][property: Range(typeof(decimal), "0", "100")] decimal? DistanceKm);
```

Cấu hình appsettings.json của project:

```json
{
  "Fare": { "BaseFare": 12000 },
  "Logging": { "LogLevel": { "Default": "Information", "Microsoft.AspNetCore": "Warning" } }
}
```

Bài trước: [bài 17](/articles/dotnet10-validation-problem-details-openapi). Bài tiếp: [bài 19](/articles/dotnet10-authentication-authorization).

## Tài liệu gốc

- [Middleware](https://learn.microsoft.com/en-us/aspnet/core/fundamentals/middleware/?view=aspnetcore-10.0).
- [Dependency injection](https://learn.microsoft.com/en-us/aspnet/core/fundamentals/dependency-injection?view=aspnetcore-10.0).
- [Options pattern](https://learn.microsoft.com/en-us/aspnet/core/fundamentals/configuration/options?view=aspnetcore-10.0).
- [Logging](https://learn.microsoft.com/en-us/aspnet/core/fundamentals/logging/?view=aspnetcore-10.0).
