using System.Net;
using System.Net.Http.Json;

using var http = new HttpClient(new StubHandler()) { BaseAddress = new Uri("https://quote.invalid/") };
var quote = await GetQuoteAsync(http, CancellationToken.None);
Console.WriteLine($"Fare: {quote.Fare} VND");
if (args.Contains("--verify"))
{
    Check(quote.Fare == 25500m, "Decoded response");
    using var cancelled = new CancellationTokenSource();
    cancelled.Cancel();
    await ExpectAsync<OperationCanceledException>(() => GetQuoteAsync(http, cancelled.Token));
    using var broken = new HttpClient(new StubHandler(HttpStatusCode.ServiceUnavailable)) { BaseAddress = http.BaseAddress };
    await ExpectAsync<HttpRequestException>(() => GetQuoteAsync(broken, CancellationToken.None));
    var gate = new BlockingHandler();
    using var waiting = new HttpClient(gate) { BaseAddress = http.BaseAddress };
    using var cts = new CancellationTokenSource();
    var pending = GetQuoteAsync(waiting, cts.Token);
    await gate.Started.Task.WaitAsync(TimeSpan.FromSeconds(5));
    cts.Cancel();
    await ExpectAsync<OperationCanceledException>(() => pending);
    Console.WriteLine("PASS: 4 async/cancellation checks");
}
static async Task<QuoteReply> GetQuoteAsync(HttpClient client, CancellationToken cancellationToken)
{
    using var response = await client.GetAsync("quotes/TN-1", cancellationToken);
    response.EnsureSuccessStatusCode();
    return await response.Content.ReadFromJsonAsync<QuoteReply>(cancellationToken)
        ?? throw new InvalidDataException("Quote response was null.");
}
static async Task ExpectAsync<T>(Func<Task> action) where T : Exception
{
    try { await action(); } catch (T) { return; }
    throw new Exception($"Expected {typeof(T).Name}");
}
static void Check(bool ok, string name) { if (!ok) throw new Exception(name); }
sealed record QuoteReply(decimal Fare);
sealed class StubHandler(HttpStatusCode status = HttpStatusCode.OK) : HttpMessageHandler
{
    protected override Task<HttpResponseMessage> SendAsync(HttpRequestMessage request, CancellationToken ct)
    {
        ct.ThrowIfCancellationRequested();
        return Task.FromResult(new HttpResponseMessage(status) { Content = JsonContent.Create(new QuoteReply(25500m)) });
    }
}
sealed class BlockingHandler : HttpMessageHandler
{
    public TaskCompletionSource Started { get; } = new(TaskCreationOptions.RunContinuationsAsynchronously);
    protected override async Task<HttpResponseMessage> SendAsync(HttpRequestMessage request, CancellationToken ct)
    {
        Started.TrySetResult();
        await Task.Delay(Timeout.InfiniteTimeSpan, ct);
        throw new InvalidOperationException("Unreachable without cancellation.");
    }
}
