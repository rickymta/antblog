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
