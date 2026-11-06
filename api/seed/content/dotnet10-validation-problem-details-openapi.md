---
slug: dotnet10-validation-problem-details-openapi
title: "Validation, Problem Details và OpenAPI: hợp đồng API trong .NET 10"
summary: "Bài 17: bật validation tích hợp của Minimal API .NET 10, phân biệt thiếu/null/ngoài miền, trả lỗi có cấu trúc và xuất tài liệu OpenAPI để kiểm chứng."
category: programming
level: intermediate
tags: [csharp, dotnet, dotnet-10, toi-noi, aspnet-core]
language: vi
---

Đây là **bài 17** trong loạt **C# và .NET 10 qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Các ví dụ target `net10.0`; code trong repository và các đoạn đầy đủ dưới đây dùng cùng implementation được chạy kiểm chứng.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Gắn Range vào property là đủ để mọi Minimal API .NET 10 tự kiểm tra dù chưa đăng ký validation không?</p>
<details><summary>Xem đáp án</summary><p>Không. Cần bật cơ chế validation tương ứng; lab gọi AddValidation và kiểm tra request thật qua pipeline.</p></details>
</div>

## Đừng để client đọc một chuỗi lỗi tự do

Bản API 16 trả một object error đơn giản. Khi có nhiều field, client cần biết field nào lỗi và cùng một cấu trúc phản hồi ổn định. Ta chuyển kiểm tra đầu vào cơ bản sang validation của Minimal API .NET 10, rồi công bố hợp đồng qua OpenAPI.

Đây là tính năng của nhánh .NET 10 trong bài; không sao chép AddValidation sang một bài .NET cũ rồi mặc định hành vi giống nhau. Project dùng package và SDK đã được pin/kiểm thử. Các rule nghiệp vụ trong policy vẫn tồn tại vì người gọi ngoài HTTP cũng có thể dùng nó.

Validation trả lời yêu cầu có đúng shape/miền đầu vào cơ bản không. Nó không thay auth, không kiểm tiền đã thanh toán hay quyền nhận chuyến. Không dùng một attribute cho mọi invariant liên quan dữ liệu đang thay đổi trong database.

## Required và Range giải hai câu hỏi khác nhau

DistanceKm là decimal?, có Required và Range từ 0 tới 100. Required từ chối thiếu/null; Range kiểm giá trị khi có. Nếu bỏ Required thì một attribute range không có nghĩa field bắt buộc hiện diện.

Với positional record, prefix `property:` đặt attribute lên property được sinh, đúng nơi validation đọc trong lab. Các giới hạn dùng kiểu decimal để giữ đúng ý nghĩa, không ép khoảng cách lẻ thành int trước kiểm tra. Hai cận đều được chấp nhận.

AddValidation đăng ký cơ chế .NET 10, với phần khám phá/sinh hỗ trợ cho kiểu endpoint. Dấu ! và .Value trong handler là dựa vào ranh giới validation đã được kiểm tra; chúng không tự thực hiện validation. Nếu gọi logic đó ngoài pipeline, phải tự giữ hợp đồng và guard cần thiết.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 1. Kiểm shape/miền ở biên HTTP trước handler; domain vẫn bảo vệ hợp đồng của nó."><defs><marker id="cs17-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Body JSON</text><text x="34" y="123" class="t-small">Thiếu / null / số</text><path d="M280 115 L305 115" class="f-none s-phys" stroke-width="2" marker-end="url(#cs17-0-a)"/><rect x="305" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="319" y="95" class="t-b">Validation</text><text x="319" y="123" class="t-small">Required + Range</text><path d="M565 115 L590 115" class="f-none s-phys" stroke-width="2" marker-end="url(#cs17-0-a)"/><rect x="590" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="604" y="95" class="t-b">Handler</text><text x="604" y="123" class="t-small">Chỉ chạy khi đạt</text></svg>
</div>
<figcaption>Hình 1. Kiểm shape/miền ở biên HTTP trước handler; domain vẫn bảo vệ hợp đồng của nó.</figcaption>
</figure>

## Problem Details chuẩn hóa gì?

Validation failure của lab trả 400 với application/problem+json, status và errors theo field. Client nên dùng cấu trúc/mã đã cam kết thay vì parse một câu tiếng Việt để ra quyết định. AddProblemDetails và UseExceptionHandler phục vụ đường lỗi tương ứng; đừng nghĩ gọi chúng sẽ tự chuẩn hóa mọi response tùy biến hoặc mọi body 400 ở mọi môi trường.

JSON hỏng có thể dừng ở binding trước validation. Bộ test riêng chỉ assert 400 cho ca đó, không áp một cấu trúc body chưa được hợp đồng hóa. Một lỗi bất ngờ trong server không nên bị gắn nhãn “distanceKm sai” chỉ để tất cả phản hồi giống nhau.

Không gửi stack trace, connection string hay thông tin token trong Problem Details. Trace identifier giúp tra log nội bộ mà không cần đưa chi tiết thực thi cho client. Bài 18 sẽ bổ sung đường quan sát request.

## OpenAPI mô tả, không thay thực thi

AddOpenApi đăng ký sinh tài liệu. MapOpenApi mở endpoint `/openapi/v1.json` trong Development của lab. OpenAPI JSON không đồng nghĩa đã có một giao diện Swagger UI; project không cài UI riêng.

TypedResults giúp framework biết metadata của response; ProducesValidationProblem công bố đường 400 validation. Dù schema có mô tả đúng, code hoặc middleware vẫn có thể sai: test HTTP phải đối chiếu tài liệu với hành vi quan sát được. Không dùng một tệp schema đẹp làm bằng chứng kiểm tra request đã bật.

Project pin Microsoft.OpenApi 2.7.5 trên dòng 2.x thay cho dependency cũ có advisory. Đây là phiên bản kiểm chứng của lab; khi nâng package cần chạy lại test/schema, không đổi major tùy ý mà chỉ nhìn build thành công.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 2. Ba phần bổ sung nhau. Schema đúng không chứng minh validation đã chạy."><defs><marker id="cs17-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Problem Details</text><text x="34" y="123" class="t-small">Lỗi có cấu trúc</text><rect x="305" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="319" y="95" class="t-b">OpenAPI</text><text x="319" y="123" class="t-small">Mô tả hợp đồng</text><rect x="590" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="604" y="95" class="t-b">HTTP tests</text><text x="604" y="123" class="t-small">Bằng chứng hành vi</text></svg>
</div>
<figcaption>Hình 2. Ba phần bổ sung nhau. Schema đúng không chứng minh validation đã chạy.</figcaption>
</figure>

## Ma trận kiểm chứng và bài tập

| Body | Kỳ vọng |
| --- | --- |
| {} hoặc distanceKm = null | 400 validation |
| -0.001 hoặc 100.001 | 400 validation |
| 0 hoặc 100 | 200, đúng giá cận |
| JSON hỏng | 400 binding; handler không thành công |

Chạy API cùng cách bài 16 ở port 5197. Muốn đọc OpenAPI, đặt environment Development bằng tùy chọn dòng lệnh `--environment Development` ở môi trường lab; ở Production route tài liệu không được map theo code. Đây là lựa chọn phạm vi xuất tài liệu, không phải biện pháp thay auth cho API.

**Bài tập:** bỏ Required nhưng giữ Range rồi gửi null. Test nào phải bắt thay đổi? Nếu schema có đường 400 mà endpoint vẫn trả 200 cho -1 km, tài liệu hay runtime quyết định dữ liệu có bị chặn?

<details><summary>Xem đáp án</summary><p>Ca thiếu/null phải phát hiện lỗi hợp đồng. Runtime quyết định request có đi tiếp; metadata không thay thế enforcement. Khôi phục rule và kiểm cả hai thay vì sửa tài liệu để che lỗi.</p></details>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
dotnet run --project examples/toi-noi-dotnet/17-validation-openapi -- --urls http://127.0.0.1:5197 --environment Development
```

Đây là host học trên localhost; dừng bằng Ctrl+C sau khi thử. Bộ test của bài 20 kiểm tra các bước API 16–19 bằng cùng source mà không cần mở các port này. Không triển khai ví dụ chưa có đầy đủ cấu hình service lên public endpoint.

### Program.cs — 17-validation-openapi/Program.cs

```csharp
using System.ComponentModel.DataAnnotations;
using ToiNoi.Fares;
namespace ToiNoi.Api17;
public partial class Program
{
    public static void Main(string[] args)
    {
        var builder = WebApplication.CreateBuilder(args);
        builder.Services.AddValidation();
        builder.Services.AddProblemDetails();
        builder.Services.AddOpenApi();
        builder.Services.AddSingleton<IFarePolicy, StandardFarePolicy>();
        var app = builder.Build();
        app.UseExceptionHandler();
        if (app.Environment.IsDevelopment()) app.MapOpenApi();
        app.MapPost("/quotes", (QuoteRequest input, IFarePolicy policy) =>
            TypedResults.Ok(new QuoteReply(policy.Calculate(input.DistanceKm!.Value), "VND")))
            .ProducesValidationProblem();
        app.Run();
    }
}
public sealed record QuoteRequest(
    [property: Required]
    [property: Range(typeof(decimal), "0", "100")] decimal? DistanceKm);
public sealed record QuoteReply(decimal Fare, string Currency);
```

Bài trước: [bài 16](/articles/dotnet10-http-minimal-api). Bài tiếp: [bài 18](/articles/dotnet10-middleware-di-options-logging).

## Tài liệu gốc

- [Validation in .NET 10](https://learn.microsoft.com/en-us/aspnet/core/validation/overview?view=aspnetcore-10.0).
- [Error handling for APIs](https://learn.microsoft.com/en-us/aspnet/core/fundamentals/error-handling-api?view=aspnetcore-10.0).
- [OpenAPI overview](https://learn.microsoft.com/en-us/aspnet/core/fundamentals/openapi/overview?view=aspnetcore-10.0).
- [OpenAPI.NET advisory and patched versions](https://github.com/advisories/GHSA-v5pm-xwqc-g5wc).
