---
slug: dotnet10-webapplicationfactory-integration-test
title: "API integration test với WebApplicationFactory: kiểm chứng pipeline .NET 10"
summary: "Bài 20: chạy các API báo giá trong TestServer, kiểm binding, validation, OpenAPI, DI/options và JWT bằng WebApplicationFactory; xác định rõ ranh giới chưa được kiểm chứng."
category: csharp-intermediate
level: intermediate
tags: [csharp, dotnet, dotnet-10, toi-noi, aspnet-core]
language: vi
---

Đây là **bài 20** trong loạt **C# và .NET 10 qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Các ví dụ target `net10.0`; code trong repository và các đoạn đầy đủ dưới đây dùng cùng implementation được chạy kiểm chứng.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>TestServer trả 200 có chứng minh cấu hình TLS/nginx của production đúng không?</p>
<details><summary>Xem đáp án</summary><p>Không. Nó chạy pipeline ứng dụng trong test host; network server, proxy và cấu hình triển khai là ranh giới khác.</p></details>
</div>

## Unit test endpoint chưa nhìn thấy pipeline

Gọi thẳng hàm tính giá không đi qua JSON binding, validation hay authorization. API có thể trả đúng số tiền trong unit test nhưng lại nhận thiếu field, dùng scope sai hoặc bỏ quên policy khi chạy HTTP. WebApplicationFactory dựng host của ứng dụng để client test gửi request qua pipeline.

Suite tham chiếu bốn project API 16–19 bằng alias entry point riêng. Mỗi Program là public partial class có Main tường minh, nên factory chọn đúng ứng dụng. Microsoft.AspNetCore.Mvc.Testing cung cấp hạ tầng TestServer; xUnit chạy test và báo kết quả.

Không cần mở port hoặc server dev cho suite này. BaseAddress của client test không có nghĩa request đi ra Internet. Ngược lại, nếu app thật gọi dependency ngoài, phải chủ động thay/configure dependency đó; WebApplicationFactory không tự fake tất cả hạ tầng.

## Giữ HTTP thật, thay dependency có chủ đích

Ca baseline gửi JSON 5 km và assert status, fare, currency bằng JSON parser. Nhóm validation gửi body thiếu/null/ngoài miền, kiểm 400 cùng errors; hai cận 0/100 vẫn thành công. Ca JSON hỏng chỉ assert 400 ở môi trường Production, đúng phạm vi hợp đồng bài 17.

Ca OpenAPI dựng Development và kiểm route /quotes tồn tại trong tài liệu; một ca Production đảm bảo endpoint tài liệu không được map. Đặt environment rõ ràng để test không phụ thuộc biến môi trường đang có trên máy chạy.

Factory có thể thay TimeProvider bằng FixedClock và cấu hình BaseFare=13000. Assertion mong 26.500 cho 5 km và timestamp UnixEpoch: kỳ vọng đó chứng minh DI/config đi tới endpoint, không chỉ chứng minh một fake trả sẵn toàn bộ HTTP response.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 1. Test giữ pipeline ứng dụng, chỉ thay dependency/cấu hình đã nêu; không giả toàn bộ HTTP response."><defs><marker id="cs20-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">xUnit + HttpClient</text><text x="34" y="123" class="t-small">Request theo hợp đồng</text><path d="M280 115 L305 115" class="f-none s-phys" stroke-width="2" marker-end="url(#cs20-0-a)"/><rect x="305" y="65" width="260" height="100" rx="10" class="f-pk-soft s-pk"/><text x="319" y="95" class="t-b">Factory / TestServer</text><text x="319" y="123" class="t-small">Pipeline ứng dụng thật</text><path d="M565 115 L590 115" class="f-none s-phys" stroke-width="2" marker-end="url(#cs20-0-a)"/><rect x="590" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="604" y="95" class="t-b">Assertion</text><text x="604" y="123" class="t-small">HTTP + JSON + quyền</text><rect x="295" y="35" width="280" height="150" rx="12" class="f-none s-pk" stroke-dasharray="5 4"/></svg>
</div>
<figcaption>Hình 1. Test giữ pipeline ứng dụng, chỉ thay dependency/cấu hình đã nêu; không giả toàn bộ HTTP response.</figcaption>
</figure>

## Kiểm lifetime và cấu hình sai

Diagnostics ở bài 18 cho thấy hai transient reader trong cùng request giữ chung scoped stamp. Test so identity trong response rồi gửi request khác, yêu cầu ScopeId đổi. Chỉ assert “không ném exception” sẽ bỏ qua bug đăng ký sai lifetime.

Ca options âm mong host khởi động thất bại. Tình huống này không cần gửi một request thành công giả để “warm up”; nếu cấu hình sai vẫn nhận traffic thì đã khác fail-fast contract. Mỗi factory được dispose để host, client và tài nguyên gắn với nó được dọn.

Test configuration override có phạm vi trong host test; không sửa appsettings của source để chạy rồi cố khôi phục bằng tay. Nhờ vậy chạy song song/CI không ghi đè cấu hình nhau.

## JWT đi qua handler thật trong test

AuthFactory post-configure JwtBearerOptions với khoá ngẫu nhiên, issuer toi-noi-tests, audience toi-noi-api và ClockSkew=0 cho fixture. Token có hạn ngắn và role/sub theo từng ca. Không có secret thật hoặc một token cố định được commit.

Suite kiểm role và owner, token hết hạn, audience sai, chữ ký từ khoá khác. Đây là bằng chứng cho JWT pipeline đã cấu hình trong fixture, không chứng minh identity provider production, rotation metadata hoặc độ lệch đồng hồ thật. Các ranh giới đó cần test và vận hành riêng.

Điều không thay là authentication handler và authorization policy của ứng dụng. Điều thay là nguồn trust/config trong host test. Ghi rõ sự thay thế giúp người đọc không nhầm việc ký một token test với dựng hệ thống đăng nhập thật.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 2. Ba ranh giới kiểm chứng riêng, không suy từ một bộ xanh sang toàn bộ hệ thống đúng."><defs><marker id="cs20-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">TestServer</text><text x="34" y="123" class="t-small">Logic/pipeline app</text><rect x="305" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="319" y="95" class="t-b">Database thật</text><text x="319" y="123" class="t-small">Transaction/schema</text><rect x="590" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="604" y="95" class="t-b">Môi trường deploy</text><text x="604" y="123" class="t-small">TLS/proxy/secret</text></svg>
</div>
<figcaption>Hình 2. Ba ranh giới kiểm chứng riêng, không suy từ một bộ xanh sang toàn bộ hệ thống đúng.</figcaption>
</figure>

## 24 ca API và những gì vẫn chưa biết

Suite hiện có 24 ca API: baseline, validation/binding, schema theo environment, lifetime, options/clock và auth. Các ca chỉ kiểm những phần đã thực thi; không đếm số assertion làm thước đo bao phủ nghiệp vụ.

| Đã đi qua | Chưa được chứng minh bởi suite này |
| --- | --- |
| HTTP pipeline trong TestServer | Kestrel socket, TLS, nginx/CDN |
| DI, binding, validation, policy | PostgreSQL/transaction/migration |
| JWT validator với trust test | Identity provider thật và rotation |
| JSON theo model endpoint | Độ trễ, throughput và tải production |

**Bài tập:** nếu mock toàn bộ HTTP response 200 thì còn bắt được lỗi quên RequireAuthorization không? Vì sao một health check production vẫn cần khi integration suite đã xanh?

<details><summary>Xem đáp án</summary><p>Không, response giả đã bỏ pipeline cần kiểm. Deployment vẫn có thể sai port, quyền, secret, proxy hoặc kết nối hạ tầng; test ứng dụng không thay kiểm tra môi trường triển khai.</p></details>

Sau bài này, chặng dữ liệu sẽ đưa EF Core 10/PostgreSQL vào với integration test đúng database. DDD, Clean Architecture, CQRS/Mediator và Repository/UoW vẫn là kiến trúc đích; các ví dụ hiện tại đang xây từng nền tảng cần thiết trước khi ghép service hoàn chỉnh.

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
dotnet test examples/toi-noi-dotnet/20-api-integration-tests
```

Lần đầu cần restore các package test từ NuGet. Project được pin phiên bản trong .csproj; mỗi ca test có dữ liệu và tài nguyên riêng để chạy lại được.

### ApiTests.cs — 20-api-integration-tests/ApiTests.cs

```csharp
using System.Net;
using System.Net.Http.Json;
using System.Text.Json;
using Microsoft.AspNetCore.Hosting;
using Microsoft.AspNetCore.Mvc.Testing;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.DependencyInjection.Extensions;
using Microsoft.Extensions.Options;
using Xunit;
using Entry16 = ToiNoi.Api16.Program;
using Entry17 = ToiNoi.Api17.Program;
using Entry18 = ToiNoi.Api18.Program;

namespace ToiNoi.ApiTests;
public sealed class ApiTests
{
    [Fact]
    public async Task Baseline_endpoint_binds_json_and_returns_fare()
    {
        using var factory = new WebApplicationFactory<Entry16>();
        using var client = factory.CreateClient();
        using var response = await client.PostAsJsonAsync("/quotes", new { distanceKm = 5m });
        Assert.Equal(HttpStatusCode.OK, response.StatusCode);
        var body = await response.Content.ReadFromJsonAsync<JsonElement>();
        Assert.Equal(25500m, body.GetProperty("fare").GetDecimal());
        Assert.Equal("VND", body.GetProperty("currency").GetString());
    }
    [Theory]
    [InlineData("{}")]
    [InlineData("""{"distanceKm":null}""")]
    [InlineData("""{"distanceKm":-0.001}""")]
    [InlineData("""{"distanceKm":100.001}""")]
    public async Task Validation_rejects_missing_null_and_out_of_range(string json)
    {
        using var factory = new WebApplicationFactory<Entry17>();
        using var client = factory.CreateClient();
        using var response = await client.PostAsync("/quotes", new StringContent(json, System.Text.Encoding.UTF8, "application/json"));
        Assert.Equal(HttpStatusCode.BadRequest, response.StatusCode);
        Assert.Equal("application/problem+json", response.Content.Headers.ContentType?.MediaType);
        var body = await response.Content.ReadFromJsonAsync<JsonElement>();
        Assert.Equal(400, body.GetProperty("status").GetInt32());
        Assert.True(body.GetProperty("errors").EnumerateObject().Any());
    }
    [Theory]
    [InlineData(0, 12000)]
    [InlineData(100, 453000)]
    public async Task Validation_preserves_inclusive_boundaries(int distance, int expected)
    {
        using var factory = new WebApplicationFactory<Entry17>();
        using var client = factory.CreateClient();
        using var response = await client.PostAsJsonAsync("/quotes", new { distanceKm = distance });
        Assert.Equal(HttpStatusCode.OK, response.StatusCode);
        var body = await response.Content.ReadFromJsonAsync<JsonElement>();
        Assert.Equal((decimal)expected, body.GetProperty("fare").GetDecimal());
    }
    [Fact]
    public async Task Malformed_json_does_not_reach_success_endpoint()
    {
        using var factory = new WebApplicationFactory<Entry17>().WithWebHostBuilder(b => b.UseEnvironment("Production"));
        using var client = factory.CreateClient();
        using var response = await client.PostAsync("/quotes", new StringContent("{bad", System.Text.Encoding.UTF8, "application/json"));
        Assert.Equal(HttpStatusCode.BadRequest, response.StatusCode);
    }
    [Fact]
    public async Task Openapi_contains_quote_route_in_development()
    {
        using var factory = new WebApplicationFactory<Entry17>().WithWebHostBuilder(b => b.UseEnvironment("Development"));
        using var client = factory.CreateClient();
        var doc = await client.GetFromJsonAsync<JsonElement>("/openapi/v1.json");
        Assert.True(doc.GetProperty("paths").TryGetProperty("/quotes", out _));
    }
    [Fact]
    public async Task Openapi_is_not_mapped_in_production()
    {
        using var factory = new WebApplicationFactory<Entry17>().WithWebHostBuilder(b => b.UseEnvironment("Production"));
        using var client = factory.CreateClient();
        using var response = await client.GetAsync("/openapi/v1.json");
        Assert.Equal(HttpStatusCode.NotFound, response.StatusCode);
    }
    [Fact]
    public async Task Scoped_identity_is_shared_inside_request_not_across_requests()
    {
        using var factory = new WebApplicationFactory<Entry18>().WithWebHostBuilder(b => b.UseEnvironment("Development"));
        using var client = factory.CreateClient();
        var a = await client.GetFromJsonAsync<JsonElement>("/diagnostics/scope");
        var b = await client.GetFromJsonAsync<JsonElement>("/diagnostics/scope");
        Assert.True(a.GetProperty("sameScoped").GetBoolean());
        Assert.False(a.GetProperty("sameTransient").GetBoolean());
        Assert.NotEqual(a.GetProperty("scopeId").GetGuid(), b.GetProperty("scopeId").GetGuid());
    }
    [Fact]
    public async Task Options_and_clock_are_replaceable_at_host_boundary()
    {
        using var factory = new WebApplicationFactory<Entry18>().WithWebHostBuilder(b =>
        {
            b.ConfigureAppConfiguration((_, config) => config.AddInMemoryCollection(new Dictionary<string, string?> { ["Fare:BaseFare"] = "13000" }));
            b.ConfigureServices(services => services.Replace(ServiceDescriptor.Singleton<TimeProvider>(new FixedClock())));
        });
        using var client = factory.CreateClient();
        using var response = await client.PostAsJsonAsync("/quotes", new { distanceKm = 5m });
        var body = await response.Content.ReadFromJsonAsync<JsonElement>();
        Assert.Equal(26500m, body.GetProperty("fare").GetDecimal());
        Assert.Equal(DateTimeOffset.UnixEpoch, body.GetProperty("issuedAt").GetDateTimeOffset());
        Assert.True(response.Headers.Contains("X-Trace-Id"));
    }
    [Fact]
    public void Invalid_options_fail_startup()
    {
        using var factory = new WebApplicationFactory<Entry18>().WithWebHostBuilder(b =>
            b.ConfigureAppConfiguration((_, config) => config.AddInMemoryCollection(new Dictionary<string, string?> { ["Fare:BaseFare"] = "-1" })));
        Assert.Throws<OptionsValidationException>(() => factory.CreateClient());
    }
    private sealed class FixedClock : TimeProvider
    {
        public override DateTimeOffset GetUtcNow() => DateTimeOffset.UnixEpoch;
    }
}
```

### AuthTests.cs — 20-api-integration-tests/AuthTests.cs

```csharp
using System.IdentityModel.Tokens.Jwt;
using System.Net;
using System.Net.Http.Headers;
using System.Security.Claims;
using System.Security.Cryptography;
using Microsoft.AspNetCore.Authentication.JwtBearer;
using Microsoft.AspNetCore.Hosting;
using Microsoft.AspNetCore.Mvc.Testing;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.IdentityModel.Tokens;
using Xunit;
using Entry19 = ToiNoi.Api19.Program;

namespace ToiNoi.ApiTests;
public sealed class AuthFactory : WebApplicationFactory<Entry19>
{
    private readonly SymmetricSecurityKey key = new(RandomNumberGenerator.GetBytes(32));
    protected override void ConfigureWebHost(IWebHostBuilder builder)
    {
        builder.UseEnvironment("Testing");
        builder.ConfigureServices(services => services.PostConfigure<JwtBearerOptions>(JwtBearerDefaults.AuthenticationScheme, options =>
        {
            options.TokenValidationParameters = new TokenValidationParameters
            {
                ValidateIssuerSigningKey = true, IssuerSigningKey = key,
                RequireSignedTokens = true, ValidAlgorithms = [SecurityAlgorithms.HmacSha256],
                ValidateIssuer = true, ValidIssuer = "toi-noi-tests",
                ValidateAudience = true, ValidAudience = "toi-noi-api",
                ValidateLifetime = true, RequireExpirationTime = true, ClockSkew = TimeSpan.Zero,
                NameClaimType = "sub", RoleClaimType = "role"
            };
        }));
    }
    public string Token(string role, string subject = "customer-1", string audience = "toi-noi-api", bool expired = false, bool wrongKey = false)
    {
        var now = DateTime.UtcNow;
        var signingKey = wrongKey ? new SymmetricSecurityKey(RandomNumberGenerator.GetBytes(32)) : key;
        var jwt = new JwtSecurityToken("toi-noi-tests", audience,
            [new Claim("sub", subject), new Claim("role", role)],
            now.AddMinutes(-10), expired ? now.AddMinutes(-1) : now.AddMinutes(5),
            new SigningCredentials(signingKey, SecurityAlgorithms.HmacSha256));
        return new JwtSecurityTokenHandler().WriteToken(jwt);
    }
}
public sealed class AuthTests
{
    [Fact]
    public async Task Anonymous_request_is_401()
    {
        using var factory = new AuthFactory(); using var client = factory.CreateClient();
        using var response = await client.GetAsync("/trips/customer-1");
        Assert.Equal(HttpStatusCode.Unauthorized, response.StatusCode);
    }
    [Theory]
    [InlineData("customer", "/trips/customer-1", 200)]
    [InlineData("customer", "/trips/customer-2", 403)]
    [InlineData("driver", "/trips/customer-1", 403)]
    [InlineData("driver", "/driver/availability", 200)]
    [InlineData("customer", "/driver/availability", 403)]
    [InlineData("dispatcher", "/dispatch/queue", 200)]
    [InlineData("driver", "/dispatch/queue", 403)]
    public async Task Role_and_owner_are_both_enforced(string role, string route, int expected)
    {
        using var factory = new AuthFactory(); using var client = factory.CreateClient();
        client.DefaultRequestHeaders.Authorization = new AuthenticationHeaderValue("Bearer", factory.Token(role));
        using var response = await client.GetAsync(route);
        Assert.Equal((HttpStatusCode)expected, response.StatusCode);
    }
    [Theory]
    [InlineData("other-api", false, false)]
    [InlineData("toi-noi-api", true, false)]
    [InlineData("toi-noi-api", false, true)]
    public async Task Wrong_audience_expired_and_wrong_signature_are_401(string audience, bool expired, bool wrongKey)
    {
        using var factory = new AuthFactory(); using var client = factory.CreateClient();
        client.DefaultRequestHeaders.Authorization = new AuthenticationHeaderValue("Bearer", factory.Token("customer", audience: audience, expired: expired, wrongKey: wrongKey));
        using var response = await client.GetAsync("/trips/customer-1");
        Assert.Equal(HttpStatusCode.Unauthorized, response.StatusCode);
    }
}
```

Bài trước: [bài 19](/articles/dotnet10-authentication-authorization). Bài tiếp: [bài 21 — EF Core 10 và PostgreSQL](/articles/dotnet10-efcore-postgresql-mapping-migration).

## Tài liệu gốc

- [WebApplicationFactory integration tests](https://learn.microsoft.com/en-us/aspnet/core/test/integration-tests?view=aspnetcore-10.0).
- [Unit testing best practices](https://learn.microsoft.com/en-us/dotnet/core/testing/unit-testing-best-practices).
