using System.Diagnostics;
using System.Diagnostics.Metrics;
namespace ToiNoi.Trips.Application;
public static class TripDiagnostics
{
    public const string Name = "ToiNoi.Trips";
    public static readonly ActivitySource Source = new(Name);
    public static readonly Meter Meter = new(Name);
    public static readonly Counter<long> Requests = Meter.CreateCounter<long>("trips.requests", unit: "{request}");
    public static readonly Histogram<double> Duration = Meter.CreateHistogram<double>("trips.request.duration", unit: "ms");
}
public sealed class ValidationBehavior<TRequest, TResponse>(IEnumerable<IRequestValidator<TRequest>> validators)
    : IPipelineBehavior<TRequest, TResponse> where TRequest : IRequest<TResponse>
{
    public Task<TResponse> HandleAsync(TRequest request, RequestNext<TResponse> next, CancellationToken ct)
    {
        ct.ThrowIfCancellationRequested(); foreach (var validator in validators) validator.Validate(request);
        return next();
    }
}
public sealed class TelemetryBehavior<TRequest, TResponse> : IPipelineBehavior<TRequest, TResponse> where TRequest : IRequest<TResponse>
{
    public async Task<TResponse> HandleAsync(TRequest request, RequestNext<TResponse> next, CancellationToken ct)
    {
        string operation = typeof(TRequest).Name; long start = Stopwatch.GetTimestamp();
        using var activity = TripDiagnostics.Source.StartActivity(operation);
        activity?.SetTag("app.operation", operation);
        TripDiagnostics.Requests.Add(1, new KeyValuePair<string, object?>("operation", operation));
        try { return await next(); }
        catch { activity?.SetStatus(ActivityStatusCode.Error); throw; }
        finally { TripDiagnostics.Duration.Record(Stopwatch.GetElapsedTime(start).TotalMilliseconds, new KeyValuePair<string, object?>("operation", operation)); }
    }
}
