var unsafeTrip = new UnsafeTrip();
var bothRead = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
int readers = 0;
async Task<bool> UnsafeAccept(string driver)
{
    bool available = unsafeTrip.Driver is null;
    if (Interlocked.Increment(ref readers) == 2) bothRead.TrySetResult();
    await bothRead.Task.WaitAsync(TimeSpan.FromSeconds(5));
    if (!available) return false;
    unsafeTrip.Driver = driver;
    return true;
}
var unsafeResults = await Task.WhenAll(UnsafeAccept("DR-1"), UnsafeAccept("DR-2"));
var guarded = new GuardedTrip();
var start = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
var tasks = Enumerable.Range(1, 20).Select(i => Task.Run(async () =>
{
    await start.Task;
    return guarded.TryAccept($"DR-{i}");
})).ToArray();
start.SetResult();
var safeResults = await Task.WhenAll(tasks);
Console.WriteLine($"Unsafe winners: {unsafeResults.Count(x => x)}");
Console.WriteLine($"Guarded winners: {safeResults.Count(x => x)}");
if (args.Contains("--verify"))
{
    Check(unsafeResults.Count(x => x) == 2, "Forced check-then-act interleaving");
    Check(safeResults.Count(x => x) == 1, "One winner");
    Check(guarded.Driver is not null, "Winner stored");
    Check(!guarded.TryAccept("DR-last"), "Later request rejected");
    Console.WriteLine("PASS: 4 concurrency checks");
}
static void Check(bool ok, string name) { if (!ok) throw new Exception(name); }
sealed class UnsafeTrip { public string? Driver { get; set; } }
sealed class GuardedTrip
{
    private readonly object gate = new();
    private string? driver;
    public string? Driver { get { lock (gate) return driver; } }
    public bool TryAccept(string candidate)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(candidate);
        lock (gate)
        {
            if (driver is not null) return false;
            driver = candidate;
            return true;
        }
    }
}
