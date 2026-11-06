---
slug: dotnet10-http-minimal-api
title: "HTTP và Minimal API: xuất bản endpoint báo giá bằng ASP.NET Core 10"
summary: "Bài 16: đưa policy đã kiểm thử lên POST /quotes, hiểu JSON binding, status code và DI; API tính báo giá chưa tạo tài nguyên hay lưu chuyến."
category: programming
level: intermediate
tags: [csharp, dotnet, dotnet-10, toi-noi, aspnet-core]
language: vi
---

Đây là **bài 16** trong loạt **C# và .NET 10 qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Các ví dụ target `net10.0`; code trong repository và các đoạn đầy đủ dưới đây dùng cùng implementation được chạy kiểm chứng.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>POST tính báo giá nhưng không lưu tài nguyên có bắt buộc trả 201 Created không?</p>
<details><summary>Xem đáp án</summary><p>Không. Endpoint này trả kết quả tính toán bằng 200 OK; 201 phù hợp khi có tài nguyên mới được tạo theo hợp đồng.</p></details>
</div>

## Đưa công thức lên một ranh giới mới

Trước đây người đọc gọi policy trong console. Bây giờ client gửi JSON qua HTTP. Ta tiếp tục dùng StandardFarePolicy từ bài 14, không chép công thức vào endpoint. Tầng HTTP chịu nhận dữ liệu, kiểm tra yêu cầu và biểu diễn response; rule tính tiền vẫn ở thư viện riêng.

Minimal API giảm số cấu trúc cần viết cho endpoint, không có nghĩa ứng dụng được miễn tổ chức hay miễn kiểm thử. Bản này có một route để học binding và response; khi sang DDD/CQRS/Mediator, endpoint sẽ gửi use case thay vì chứa mọi thao tác nghiệp vụ.

Ví dụ chỉ tính giá theo bảng giả, chưa tạo quote ID, chưa lưu dữ liệu hay cam kết hiệu lực báo giá. Đừng coi POST ở đây là đặt chuyến hoặc giữ tiền của khách.

## Request là phương thức, đường dẫn, header và body

Hợp đồng: `POST /quotes`, Content-Type application/json, body có distanceKm dạng số. Ví dụ `{"distanceKm":5}` trả JSON chứa fare 25500 và currency VND. Số JSON dùng dấu chấm thập phân theo grammar JSON, không nhận chuỗi giao diện “5 km”.

QuoteRequest.DistanceKm dùng decimal? để phân biệt thiếu/null với giá trị 0 hợp lệ. Nếu chỉ dùng decimal không nullable rồi dựa vào mặc định 0, field bị bỏ có thể không còn phân biệt với khách chủ động gửi 0. Đây là vấn đề hợp đồng, không phải chỉ mẹo tránh warning.

Giới hạn đầu vào vẫn là 0–100. JSON sai cú pháp hoặc sai kiểu có thể bị framework từ chối trước khi handler chạy. Không dùng một guard nghiệp vụ trong handler để kết luận mọi lỗi HTTP đều đi qua handler.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 1. HTTP đưa dữ liệu tới policy; endpoint không trở thành nơi chép lại công thức."><defs><marker id="cs16-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">HTTP client</text><text x="34" y="123" class="t-small">POST /quotes + JSON</text><path d="M280 115 L305 115" class="f-none s-phys" stroke-width="2" marker-end="url(#cs16-0-a)"/><rect x="305" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="319" y="95" class="t-b">Binding + handler</text><text x="319" y="123" class="t-small">Kiểm tra dữ liệu</text><path d="M565 115 L590 115" class="f-none s-phys" stroke-width="2" marker-end="url(#cs16-0-a)"/><rect x="590" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="604" y="95" class="t-b">Policy → response</text><text x="604" y="123" class="t-small">200 + fare + currency</text></svg>
</div>
<figcaption>Hình 1. HTTP đưa dữ liệu tới policy; endpoint không trở thành nơi chép lại công thức.</figcaption>
</figure>

## Đọc Program theo thứ tự khởi động

CreateBuilder chuẩn bị cấu hình, logging và container. AddSingleton đăng ký IFarePolicy với StandardFarePolicy; policy không giữ state từng request nên dùng chung được trong lab. Build tạo ứng dụng. MapPost mô tả route, còn Run bắt đầu host xử lý yêu cầu.

Handler nhận QuoteRequest từ body và IFarePolicy từ DI. Parameter binding dựa vào kiểu và metadata của framework; nếu một kiểu không đăng ký DI hoặc không có nguồn binding rõ, lỗi có thể xuất hiện khi dựng endpoint. Không dựa vào tên biến policy để tự tìm service.

Handler từ chối khi DistanceKm thiếu hoặc ngoài miền, trả 400; khi hợp lệ gọi policy và trả 200 với QuoteReply. Record response cố định currency để người đọc biết đơn vị. Không trả raw decimal thiếu ngữ cảnh rồi để client đoán đó là đồng hay nghìn đồng.

## Status code là một phần hợp đồng

| Tình huống | Ý nghĩa ở lab |
| --- | --- |
| Hợp lệ | 200, JSON báo giá |
| Thiếu/ngoài miền | 400, yêu cầu chưa được chấp nhận |
| Route không có | 404 |
| Lỗi bất ngờ | Không được biến thành 200 với fare = 0 |

Response 201 không được chọn vì không có tài nguyên mới lưu lại và không có Location. Nếu sau này quote được cấp ID, thời hạn và lưu bền, hợp đồng sẽ được thiết kế lại cùng vòng đời tài nguyên. Đừng giữ status cũ chỉ vì endpoint vẫn dùng POST.

API chưa bật xác thực; chỉ chạy localhost để học. Bài 19 thêm auth bằng handler thật. CORS không phải cơ chế xác thực, HTTPS không thay authorization, và việc đổi tên route không giấu được một API công khai.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 2. Các kết quả theo hợp đồng; cùng status 400 chưa có nghĩa mọi lỗi binding đi qua code handler."><defs><marker id="cs16-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Thiếu khoảng cách</text><text x="34" y="123" class="t-small">400</text><rect x="305" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="319" y="95" class="t-b">0 ≤ km ≤ 100</text><text x="319" y="123" class="t-small">200 + giá tính được</text><rect x="590" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="604" y="95" class="t-b">Route không có</text><text x="604" y="123" class="t-small">404</text></svg>
</div>
<figcaption>Hình 2. Các kết quả theo hợp đồng; cùng status 400 chưa có nghĩa mọi lỗi binding đi qua code handler.</figcaption>
</figure>

## Chạy một request có thể đối chiếu

Chạy project với `--urls http://127.0.0.1:5196`. Giữ terminal đó mở, rồi dùng curl từ terminal khác. Trong PowerShell nên gọi curl.exe để tránh alias ở các phiên bản cũ. Ví dụ đọc JSON từ file giúp tránh khác biệt escaping giữa shell.

```bash
curl --json @examples/toi-noi-dotnet/16-minimal-api/request.json http://127.0.0.1:5196/quotes
```

Kỳ vọng {"fare":25500,"currency":"VND"}; thứ tự/whitespace JSON không phải điều cần assert. Bài 20 chạy cùng endpoint trong host test và kiểm giá trị theo kiểu decimal.

**Bài tập:** gửi `{}` rồi `{"distanceKm":0}`. Vì sao không nên cho cả hai tự thành một yêu cầu 0 km? Nếu handler trả 201 mà không tạo gì, client có thể hiểu sai điều nào?

<details><summary>Xem đáp án</summary><p>Thiếu dữ liệu và chủ động gửi cận dưới là hai ý định khác nhau. 201 có thể làm client tin tài nguyên đã được tạo; hợp đồng nên phản ánh đúng thao tác và có định danh/Location khi cần.</p></details>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
dotnet run --project examples/toi-noi-dotnet/16-minimal-api -- --urls http://127.0.0.1:5196
```

Đây là host học trên localhost; dừng bằng Ctrl+C sau khi thử. Bộ test của bài 20 kiểm tra các bước API 16–19 bằng cùng source mà không cần mở các port này. Không triển khai ví dụ chưa có đầy đủ cấu hình service lên public endpoint.

### Program.cs — 16-minimal-api/Program.cs

```csharp
using ToiNoi.Fares;
namespace ToiNoi.Api16;
public partial class Program
{
    public static void Main(string[] args)
    {
        var builder = WebApplication.CreateBuilder(args);
        builder.Services.AddSingleton<IFarePolicy, StandardFarePolicy>();
        var app = builder.Build();
        app.MapPost("/quotes", (QuoteRequest input, IFarePolicy policy) =>
        {
            if (input.DistanceKm is not decimal km || km < 0m || km > 100m)
                return Results.BadRequest(new { error = "distanceKm must be between 0 and 100" });
            return Results.Ok(new QuoteReply(policy.Calculate(km), "VND"));
        });
        app.Run();
    }
}
public sealed record QuoteRequest(decimal? DistanceKm);
public sealed record QuoteReply(decimal Fare, string Currency);
```

Bài trước: [bài 15](/articles/csharp-integration-test-json-filesystem). Bài tiếp: [bài 17](/articles/dotnet10-validation-problem-details-openapi).

## Tài liệu gốc

- [Minimal APIs](https://learn.microsoft.com/en-us/aspnet/core/fundamentals/minimal-apis?view=aspnetcore-10.0).
- [Minimal API parameter binding](https://learn.microsoft.com/en-us/aspnet/core/fundamentals/minimal-apis/parameter-binding?view=aspnetcore-10.0).
- [Minimal API responses](https://learn.microsoft.com/en-us/aspnet/core/fundamentals/minimal-apis/responses?view=aspnetcore-10.0).
