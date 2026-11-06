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
