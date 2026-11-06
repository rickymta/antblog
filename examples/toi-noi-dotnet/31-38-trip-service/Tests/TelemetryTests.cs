using System.Collections.Concurrent;
using System.Diagnostics;
using System.Diagnostics.Metrics;
using System.Net;
using System.Net.Http.Json;
using ToiNoi.Trips.Application;
using Xunit;
namespace ToiNoi.Trips.Tests;
public sealed class TelemetryTests
{
    [Fact]
    public async Task Incoming_trace_flows_through_handler_and_database_commit()
    {
        await using var lab = await ServiceLab.CreateAsync(); using var factory = new ApiFactory(lab.Connection);
        using var client = factory.Client(Guid.NewGuid(), "customer");
        var captured = new ConcurrentQueue<(string Operation, ActivityTraceId Trace)>();
        using var listener = new ActivityListener
        {
            ShouldListenTo = source => source.Name == TripDiagnostics.Name,
            Sample = (ref ActivityCreationOptions<ActivityContext> _) => ActivitySamplingResult.AllDataAndRecorded,
            ActivityStopped = activity => captured.Enqueue((activity.OperationName, activity.TraceId))
        };
        ActivitySource.AddActivityListener(listener);
        const string trace = "11111111111111111111111111111111";
        using var request = new HttpRequestMessage(HttpMethod.Post, "/trips") { Content = JsonContent.Create(new { distanceKm = 5m }) };
        request.Headers.Add("Idempotency-Key", "trace-1"); request.Headers.Add("traceparent", $"00-{trace}-2222222222222222-01");
        using var response = await client.SendAsync(request); Assert.Equal(HttpStatusCode.Created, response.StatusCode);
        Assert.Contains(captured, item => item.Operation == "CreateTrip" && item.Trace.ToHexString() == trace);
        Assert.Contains(captured, item => item.Operation == "db.commit" && item.Trace.ToHexString() == trace);
    }
    [Fact]
    public async Task Metrics_use_bounded_operation_tag_not_customer_or_trip_ids()
    {
        await using var lab = await ServiceLab.CreateAsync(); using var factory = new ApiFactory(lab.Connection);
        using var client = factory.Client(Guid.NewGuid(), "customer");
        var records = new ConcurrentQueue<(string Name, string[] TagNames, string? Operation)>();
        using var listener = new MeterListener();
        listener.InstrumentPublished = (instrument, current) => { if (instrument.Meter.Name == TripDiagnostics.Name) current.EnableMeasurementEvents(instrument); };
        listener.SetMeasurementEventCallback<long>((instrument, _, tags, _) => Capture(instrument, tags));
        listener.SetMeasurementEventCallback<double>((instrument, _, tags, _) => Capture(instrument, tags));
        void Capture(Instrument instrument, ReadOnlySpan<KeyValuePair<string, object?>> tags)
        {
            var names = new List<string>(); string? operation = null;
            foreach (var tag in tags) { names.Add(tag.Key); if (tag.Key == "operation") operation = tag.Value?.ToString(); }
            records.Enqueue((instrument.Name, names.ToArray(), operation));
        }
        listener.Start();
        using var request = new HttpRequestMessage(HttpMethod.Post, "/trips") { Content = JsonContent.Create(new { distanceKm = 5m }) };
        request.Headers.Add("Idempotency-Key", "metric-1"); using var response = await client.SendAsync(request);
        Assert.Equal(HttpStatusCode.Created, response.StatusCode);
        Assert.Contains(records, item => item.Name == "trips.requests" && item.Operation == "CreateTrip");
        Assert.Contains(records, item => item.Name == "trips.request.duration" && item.Operation == "CreateTrip");
        Assert.All(records, item => Assert.All(item.TagNames, name => Assert.Equal("operation", name)));
    }
}
