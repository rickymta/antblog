---
slug: dotnet10-authentication-authorization
title: "Authentication và authorization: khách, tài xế và điều phối trong .NET 10"
summary: "Bài 19: xác thực JWT bearer bằng handler thật, tách 401/403, role policy và ownership; không tin role hoặc chủ sở hữu do client tự gửi."
category: csharp-intermediate
level: intermediate
tags: [csharp, dotnet, dotnet-10, toi-noi, aspnet-core]
language: vi
---

Đây là **bài 19** trong loạt **C# và .NET 10 qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Các ví dụ target `net10.0`; code trong repository và các đoạn đầy đủ dưới đây dùng cùng implementation được chạy kiểm chứng.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>JWT giải mã được và có role customer có đủ để cho phép đọc mọi chuyến không?</p>
<details><summary>Xem đáp án</summary><p>Không. Phải xác thực token trước, rồi kiểm policy và quyền trên tài nguyên; đọc payload không phải xác thực chữ ký.</p></details>
</div>

## Ba người dùng, ba quyền khác nhau

Khách yêu cầu báo giá, tài xế xem khả năng nhận chuyến, điều phối xem hàng đợi. Hệ thống cần biết người gọi là ai và được phép làm gì. Authentication tạo principal từ credential đã kiểm; authorization quyết định principal đó có quyền trên thao tác/tài nguyên không.

Không nhận một header X-Role=dispatcher rồi coi như đã đăng nhập. Không lấy customerId trong body làm bằng chứng chủ sở hữu. Dữ liệu client có thể là tham số yêu cầu, nhưng quyền phải dựa vào danh tính đã xác thực và dữ liệu sở hữu do server tin cậy.

Bản demo có route `/trips/{ownerId}` chỉ để phân biệt role với ownership bằng một ID tổng hợp; nó không đọc kho chuyến thật. Service hoàn chỉnh sẽ nhận tripId, tải aggregate/read model và kiểm OwnerId lưu tại server, không coi ownerId caller truyền là nguồn chân lý.

## JWT cần kiểm nhiều hơn payload

AddJwtBearer dùng handler thật. Tối thiểu hợp đồng token cần xác thực chữ ký với khoá tin cậy, issuer, audience và thời hạn; production còn phải quản lý algorithm, metadata/rotation và các yêu cầu riêng của hệ thống danh tính. Không tự viết một parser base64 rồi dựng ClaimsPrincipal.

MapInboundClaims=false giữ tên claim theo hợp đồng; bài dùng sub cho chủ thể và role cho vai trò. Nhà cung cấp token khác có thể dùng tên/shape khác, nên phải mapping có chủ đích. Role hiển thị ở UI không thay enforcement phía server.

JWT bearer không mặc định mã hoá payload. Không đưa giấy tờ định danh hoặc bí mật vào token chỉ vì token có chữ ký. Truyền bearer qua TLS ở môi trường triển khai; không ghi token vào log và không gửi qua query string để tiện copy.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 1. Đọc payload chưa đủ; principal chỉ được dùng sau xác thực trước khi quyết định quyền."><defs><marker id="cs19-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Bearer token</text><text x="34" y="123" class="t-small">Credential từ caller</text><path d="M280 115 L305 115" class="f-none s-phys" stroke-width="2" marker-end="url(#cs19-0-a)"/><rect x="305" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="319" y="95" class="t-b">Authentication</text><text x="319" y="123" class="t-small">Chữ ký / issuer / aud / exp</text><path d="M565 115 L590 115" class="f-none s-phys" stroke-width="2" marker-end="url(#cs19-0-a)"/><rect x="590" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="604" y="95" class="t-b">Authorization</text><text x="604" y="123" class="t-small">Role + quyền tài nguyên</text></svg>
</div>
<figcaption>Hình 1. Đọc payload chưa đủ; principal chỉ được dùng sau xác thực trước khi quyết định quyền.</figcaption>
</figure>

## Policy và ownership là hai lớp kiểm tra

Policy Customer yêu cầu đã xác thực và có role customer; Driver và Dispatcher tương tự. RequireAuthorization gắn policy vào endpoint. Route demo về chuyến còn so claim sub với ownerId để chặn khách A xem dữ liệu của khách B.

Không có hoặc token không hợp lệ dẫn tới challenge, thường là 401 ở bearer API. Có danh tính hợp lệ nhưng không đạt role/ownership dẫn tới forbid, 403. Bài chưa triển khai chiến lược 404 để che tồn tại tài nguyên; nếu chọn chiến lược đó phải thống nhất hợp đồng và kiểm thử riêng.

UseAuthentication đứng trước UseAuthorization để principal được dựng trước khi kiểm quyền. Cấu hình default scheme giúp framework biết handler nào xác thực/challenge. Không đặt một middleware tự cấp role tuỳ ý phía sau rồi coi policy là đủ.

## Chạy local mà không nhúng khoá vào source

Project không chứa khoá ký dùng chung, endpoint cấp token hoặc tài khoản mặc định. Có thể chạy host và thử request không credential để thấy 401. Để thử local có token, dùng dotnet user-jwts với user-secrets trong môi trường Development theo hướng dẫn README; không dùng công cụ dev đó như identity provider production.

Bộ test bài 20 tạo khoá đối xứng ngẫu nhiên riêng trong AuthFactory, cấu hình JWT validator bằng issuer/audience test rồi ký token trong process test. HMAC chỉ là fixture có phạm vi rõ; production nên dùng hệ thống danh tính/metadata được tin cậy và quản lý khoá phù hợp, không copy AuthFactory vào service.

User-secrets không phải kho bí mật production và không phải cam kết mã hoá file local. Khi cài cấu hình triển khai thật, quản lý secret/authority qua cơ chế môi trường và quyền truy cập đã thiết kế. Bài này không tự mở một token issuer cho website.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 275" role="img" aria-label="Hình 2. 401 và 403 thể hiện hai nguyên nhân khác nhau; thành công chỉ khi policy và ownership đều đạt."><defs><marker id="cs19-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="95" width="235" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="125" class="t-b">Yêu cầu bảo vệ</text><text x="34" y="153" class="t-small">Kiểm tra credential</text><rect x="465" y="25" width="385" height="85" rx="10" class="f-warn-soft s-warn"/><text x="479" y="55" class="t-b">Chưa xác thực</text><text x="479" y="83" class="t-small">401 challenge</text><rect x="465" y="165" width="385" height="85" rx="10" class="f-phys-soft s-phys"/><text x="479" y="195" class="t-b">Đã xác thực</text><text x="479" y="223" class="t-small">Policy fail → 403</text><path d="M255 120 L465 68" class="f-none s-phys" stroke-width="2" marker-end="url(#cs19-1-a)"/><path d="M255 150 L465 208" class="f-none s-phys" stroke-width="2" marker-end="url(#cs19-1-a)"/></svg>
</div>
<figcaption>Hình 2. 401 và 403 thể hiện hai nguyên nhân khác nhau; thành công chỉ khi policy và ownership đều đạt.</figcaption>
</figure>

## Bảng quyền phải có cả ca bị từ chối

| Principal / credential | Thao tác | Kỳ vọng lab |
| --- | --- | --- |
| Không token | Đọc route chuyến | 401 |
| Token customer hợp lệ | Đọc owner cùng sub | 200 |
| Token customer hợp lệ | Owner khác sub | 403 |
| Token driver hợp lệ | Route customer | 403 |
| Token driver hợp lệ | Availability | 200 |
| Token dispatcher hợp lệ | Queue | 200 |
| Sai audience / hết hạn / sai chữ ký | Route cần auth | 401 |

Test authorization dùng token đã ký thật để không bỏ qua handler xác thực. Test dùng một fake authentication handler có thể hữu ích cho một số policy test, nhưng không thay được bằng chứng JWT được kiểm đúng.

**Bài tập:** token có role customer và sub customer-1 truy cập dữ liệu customer-2: đã qua role thì có được phép không? Nếu token đổi payload role mà không ký lại, test nào phải thất bại?

<details><summary>Xem đáp án</summary><p>Không, ownership vẫn phải kiểm. Sửa payload làm chữ ký không còn hợp lệ; handler phải trả 401. Ca wrong signing key trong suite kiểm chứng một token có chữ ký không được tin cậy.</p></details>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
dotnet run --project examples/toi-noi-dotnet/19-authentication-authorization -- --urls http://127.0.0.1:5199
```

Đây là host học trên localhost; dừng bằng Ctrl+C sau khi thử. Bộ test của bài 20 kiểm tra các bước API 16–19 bằng cùng source mà không cần mở các port này. Không triển khai ví dụ chưa có đầy đủ cấu hình service lên public endpoint.

Để thử JWT local thủ công, từ checkout riêng dùng:

```bash
dotnet user-secrets init --project examples/toi-noi-dotnet/19-authentication-authorization
dotnet user-jwts create --project examples/toi-noi-dotnet/19-authentication-authorization --role customer --audience toi-noi-api
dotnet run --project examples/toi-noi-dotnet/19-authentication-authorization -- --environment Development --urls http://127.0.0.1:5199
```

Công cụ dev tạo token và lưu khoá trong user-secrets local, có thể cập nhật cấu hình audience của project; không đưa token/khoá lên Git. Gửi token qua Authorization: Bearer ở client local. Cách này dành cho dev; suite bài 20 tự tạo fixture trong memory, không cần chạy các lệnh tạo token trên.

### Program.cs — 19-authentication-authorization/Program.cs

```csharp
using System.ComponentModel.DataAnnotations;
using System.Security.Claims;
using Microsoft.AspNetCore.Authentication.JwtBearer;
using ToiNoi.Fares;
namespace ToiNoi.Api19;
public partial class Program
{
    public static void Main(string[] args)
    {
        var builder = WebApplication.CreateBuilder(args);
        builder.Services.AddValidation();
        builder.Services.AddProblemDetails();
        builder.Services.AddSingleton<IFarePolicy, StandardFarePolicy>();
        builder.Services.AddAuthentication(JwtBearerDefaults.AuthenticationScheme).AddJwtBearer(options =>
        {
            options.MapInboundClaims = false;
            options.TokenValidationParameters.NameClaimType = "sub";
            options.TokenValidationParameters.RoleClaimType = "role";
            options.TokenValidationParameters.RequireSignedTokens = true;
            options.TokenValidationParameters.ValidateIssuerSigningKey = true;
            options.TokenValidationParameters.ValidateIssuer = true;
            options.TokenValidationParameters.ValidateAudience = true;
            options.TokenValidationParameters.ValidateLifetime = true;
        });
        builder.Services.AddAuthorization(options =>
        {
            options.AddPolicy("Customer", p => p.RequireAuthenticatedUser().RequireRole("customer"));
            options.AddPolicy("Driver", p => p.RequireAuthenticatedUser().RequireRole("driver"));
            options.AddPolicy("Dispatcher", p => p.RequireAuthenticatedUser().RequireRole("dispatcher"));
        });
        var app = builder.Build();
        app.UseExceptionHandler();
        app.UseAuthentication();
        app.UseAuthorization();
        app.MapPost("/quotes", (QuoteRequest input, IFarePolicy policy) =>
            TypedResults.Ok(new { Fare = policy.Calculate(input.DistanceKm!.Value), Currency = "VND" }))
            .RequireAuthorization("Customer");
        app.MapGet("/driver/availability", () => new { Available = true }).RequireAuthorization("Driver");
        app.MapGet("/dispatch/queue", () => new { Pending = 0 }).RequireAuthorization("Dispatcher");
        app.MapGet("/trips/{ownerId}", (string ownerId, ClaimsPrincipal user) =>
            user.FindFirst("sub")?.Value == ownerId
                ? Results.Ok(new { OwnerId = ownerId }) : Results.Forbid())
            .RequireAuthorization("Customer");
        app.Run();
    }
}
public sealed record QuoteRequest([property: Required][property: Range(typeof(decimal), "0", "100")] decimal? DistanceKm);
```

Bài trước: [bài 18](/articles/dotnet10-middleware-di-options-logging). Bài tiếp: [bài 20](/articles/dotnet10-webapplicationfactory-integration-test).

## Tài liệu gốc

- [JWT bearer authentication](https://learn.microsoft.com/en-us/aspnet/core/security/authentication/configure-jwt-bearer-authentication?view=aspnetcore-10.0).
- [Role authorization](https://learn.microsoft.com/en-us/aspnet/core/security/authorization/roles?view=aspnetcore-10.0).
- [dotnet user-jwts](https://learn.microsoft.com/en-us/aspnet/core/security/authentication/jwt-authn?view=aspnetcore-10.0).
