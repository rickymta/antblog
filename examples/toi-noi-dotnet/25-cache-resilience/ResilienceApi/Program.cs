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
