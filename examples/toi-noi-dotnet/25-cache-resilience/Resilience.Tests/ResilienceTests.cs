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
